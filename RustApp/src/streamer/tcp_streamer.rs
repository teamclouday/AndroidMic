use std::{
    io,
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use futures::StreamExt;
use prost::Message;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpSocket, TcpStream},
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{
    audio::process::ProcessCache,
    config::ConnectionMode,
    streamer::{CHECK_1, CHECK_2, StreamerMsg, WriteError},
};

use super::{AudioPacketMessage, AudioStream, ConnectError, StreamerTrait};

const MAX_WAIT_TIME: Duration = Duration::from_millis(1500);

const DISCONNECT_LOOP_DETECTER_MAX: u32 = 1000;

/// delay before the first rebind after a dropped connection
const REBIND_INITIAL_DELAY: Duration = Duration::from_millis(500);
/// maximum delay between two rebind attempts
const REBIND_MAX_DELAY: Duration = Duration::from_secs(5);
/// backoff multiplier applied after each failed rebind attempt
const REBIND_BACKOFF_FACTOR: u32 = 2;

pub struct TcpStreamer {
    ip: IpAddr,
    pub port: u16,
    pub state: TcpStreamerState,
    stream_config: AudioStream,
    process_cache: ProcessCache,
    rebind_delay: Duration,
}

#[allow(clippy::large_enum_variant)]
pub enum TcpStreamerState {
    Listening {
        listener: TcpListener,
    },
    Streaming {
        framed: Framed<TcpStream, LengthDelimitedCodec>,
        disconnect_loop_detecter: u32,
    },
}

pub async fn new(
    ip: IpAddr,
    port: u16,
    stream_config: AudioStream,
) -> Result<TcpStreamer, ConnectError> {
    let listener = bind_reusable(ip, port).await?;

    let addr = TcpListener::local_addr(&listener).map_err(ConnectError::NoLocalAddress)?;

    let streamer = TcpStreamer {
        ip,
        port: addr.port(),
        stream_config,
        state: TcpStreamerState::Listening { listener },
        process_cache: ProcessCache::new(),
        rebind_delay: REBIND_INITIAL_DELAY,
    };

    Ok(streamer)
}

/// Bind a listening socket with `SO_REUSEADDR` set, so a quick rebind after a
/// dropped connection does not fail while the previous socket is in `TIME_WAIT`.
async fn bind_reusable(ip: IpAddr, port: u16) -> Result<TcpListener, ConnectError> {
    let socket = match ip {
        IpAddr::V4(_) => TcpSocket::new_v4(),
        IpAddr::V6(_) => TcpSocket::new_v6(),
    }
    .map_err(|e| ConnectError::CantBindPort(port, e))?;

    socket
        .set_reuseaddr(true)
        .map_err(|e| ConnectError::CantBindPort(port, e))?;
    socket
        .bind(SocketAddr::from((ip, port)))
        .map_err(|e| ConnectError::CantBindPort(port, e))?;
    socket
        .listen(128)
        .map_err(|e| ConnectError::CantBindPort(port, e))
}

impl StreamerTrait for TcpStreamer {
    fn reconfigure_stream(&mut self, stream_config: AudioStream) {
        self.stream_config = stream_config;
        self.process_cache.clear();
    }

    fn status(&self) -> StreamerMsg {
        match &self.state {
            TcpStreamerState::Listening { .. } => StreamerMsg::Listening {
                ip: Some(self.ip),
                port: Some(self.port),
            },
            TcpStreamerState::Streaming { .. } => StreamerMsg::Connected {
                ip: Some(self.ip),
                port: Some(self.port),
                mode: ConnectionMode::Tcp,
            },
        }
    }

    async fn next(&mut self) -> Result<Option<StreamerMsg>, ConnectError> {
        let outcome = match &mut self.state {
            TcpStreamerState::Listening { listener } => {
                let addr =
                    TcpListener::local_addr(listener).map_err(ConnectError::NoLocalAddress)?;

                info!("TCP server listening on {}", addr);

                let (mut stream, addr) =
                    listener.accept().await.map_err(ConnectError::CantAccept)?;

                let mut buf1 = [0u8; CHECK_1.len()];

                stream
                    .read_exact(&mut buf1)
                    .await
                    .map_err(|e| ConnectError::HandShakeFailed("reading", e))?;

                if buf1 != CHECK_1.as_bytes() {
                    let s = String::from_utf8_lossy(&buf1);

                    return Err(ConnectError::HandShakeFailed2(format!(
                        "{} != {}",
                        CHECK_1, s
                    )));
                }

                stream
                    .write_all(CHECK_2.as_bytes())
                    .await
                    .map_err(|e| ConnectError::HandShakeFailed("writing", e))?;

                info!("connection accepted, remote address: {}", addr);

                self.state = TcpStreamerState::Streaming {
                    framed: Framed::new(stream, LengthDelimitedCodec::new()),
                    disconnect_loop_detecter: 0,
                };

                Outcome::Msg(Some(StreamerMsg::Connected {
                    ip: Some(self.ip),
                    port: Some(self.port),
                    mode: ConnectionMode::Tcp,
                }))
            }
            TcpStreamerState::Streaming {
                framed,
                disconnect_loop_detecter: _,
            } => match framed.next().await {
                Some(Ok(frame)) => match AudioPacketMessage::decode(frame) {
                    Ok(packet) => {
                        let buffer_size = packet.buffer.len();
                        let sample_rate = packet.sample_rate;

                        match self
                            .stream_config
                            .process_audio_packet(packet, &mut self.process_cache)
                        {
                            Ok(Some(buffer)) => {
                                debug!("received {} bytes", buffer_size);
                                Outcome::Msg(Some(StreamerMsg::UpdateAudioWave {
                                    data: AudioPacketMessage::to_wave_data(&buffer, sample_rate),
                                }))
                            }
                            _ => Outcome::Msg(None),
                        }
                    }
                    Err(e) => Outcome::Fatal(ConnectError::WriteError(WriteError::Deserializer(e))),
                },

                Some(Err(e)) => match e.kind() {
                    io::ErrorKind::TimedOut => Outcome::Msg(None), // timeout use to check for input on stdin
                    io::ErrorKind::WouldBlock => Outcome::Msg(None), // trigger on Linux when there is no stream input
                    _ => {
                        warn!("stream error: {e}, going back to listening");
                        Outcome::Drop
                    }
                },
                None => Outcome::Drop,
            },
        };

        match outcome {
            Outcome::Msg(msg) => Ok(msg),
            Outcome::Drop => self.relisten().await,
            Outcome::Fatal(e) => Err(e),
        }
    }
}

impl TcpStreamer {
    /// Drop the active connection and rebind the same port, going back to
    /// `Listening` with a bounded exponential backoff. Used after a recoverable
    /// drop so another phone can reconnect without a manual Connect.
    async fn relisten(&mut self) -> Result<Option<StreamerMsg>, ConnectError> {
        info!(
            "connection closed, waiting to listen again on {}",
            self.port
        );

        loop {
            match bind_reusable(self.ip, self.port).await {
                Ok(listener) => {
                    self.rebind_delay = REBIND_INITIAL_DELAY;
                    self.process_cache.clear();
                    self.state = TcpStreamerState::Listening { listener };

                    return Ok(Some(StreamerMsg::Listening {
                        ip: Some(self.ip),
                        port: Some(self.port),
                    }));
                }
                Err(e) => {
                    warn!(
                        "can't rebind {}:{}: {e}; retrying in {:?}",
                        self.ip, self.port, self.rebind_delay
                    );
                    tokio::time::sleep(self.rebind_delay).await;
                    self.rebind_delay = self
                        .rebind_delay
                        .saturating_mul(REBIND_BACKOFF_FACTOR)
                        .min(REBIND_MAX_DELAY);
                }
            }
        }
    }
}

enum Outcome {
    Msg(Option<StreamerMsg>),
    /// the connection dropped; rebind the port and go back to listening
    Drop,
    /// unrecoverable protocol/device error, propagate to the runner
    Fatal(ConnectError),
}
