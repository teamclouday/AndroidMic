use std::{io, net::IpAddr, time::Duration};

use futures::StreamExt;
use prost::Message;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::streamer::{DEFAULT_PC_PORT, MAX_PORT, StreamerMsg, WriteError};

use super::{AudioPacketMessage, AudioStream, ConnectError, StreamerTrait};

const MAX_WAIT_TIME: Duration = Duration::from_millis(1500);

const DISCONNECT_LOOP_DETECTER_MAX: u32 = 1000;

pub struct TcpStreamer {
    ip: IpAddr,
    pub port: u16,
    pub state: TcpStreamerState,
    stream_config: AudioStream,
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

pub async fn new(ip: IpAddr, stream_config: AudioStream) -> Result<TcpStreamer, ConnectError> {
    let mut listener = None;

    // try to always bind the same port, to not change it everytime Android side
    for p in DEFAULT_PC_PORT..=MAX_PORT {
        if let Ok(l) = TcpListener::bind((ip, p)).await {
            listener = Some(l);
            break;
        }
    }

    let listener = if let Some(listener) = listener {
        listener
    } else {
        TcpListener::bind((ip, 0))
            .await
            .map_err(ConnectError::CantBindPort)?
    };

    let addr = TcpListener::local_addr(&listener).map_err(ConnectError::NoLocalAddress)?;

    let streamer = TcpStreamer {
        ip,
        port: addr.port(),
        stream_config,
        state: TcpStreamerState::Listening { listener },
    };

    Ok(streamer)
}

impl StreamerTrait for TcpStreamer {
    fn reconfigure_stream(&mut self, stream_config: AudioStream) {
        self.stream_config = stream_config;
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
            },
        }
    }

    async fn next(&mut self) -> Result<Option<StreamerMsg>, ConnectError> {
        match &mut self.state {
            TcpStreamerState::Listening { listener } => {
                let addr =
                    TcpListener::local_addr(listener).map_err(ConnectError::NoLocalAddress)?;

                info!("TCP server listening on {}", addr);

                let (stream, addr) = listener.accept().await.map_err(ConnectError::CantAccept)?;

                info!("connection accepted, remote address: {}", addr);

                self.state = TcpStreamerState::Streaming {
                    framed: Framed::new(stream, LengthDelimitedCodec::new()),
                    disconnect_loop_detecter: 0,
                };

                Ok(Some(StreamerMsg::Connected {
                    ip: Some(self.ip),
                    port: Some(self.port),
                }))
            }
            TcpStreamerState::Streaming {
                framed,
                disconnect_loop_detecter: _,
            } => {
                match framed.next().await {
                    Some(Ok(frame)) => {
                        let mut res = None;

                        match AudioPacketMessage::decode(frame) {
                            Ok(packet) => {
                                let buffer_size = packet.buffer.len();

                                if let Ok(buffer) = self.stream_config.process_audio_packet(packet)
                                {
                                    // compute the audio wave from the buffer
                                    res = Some(StreamerMsg::UpdateAudioWave {
                                        data: AudioPacketMessage::to_wave_data(&buffer),
                                    });

                                    debug!("received {} bytes", buffer_size);
                                };
                            }
                            Err(e) => {
                                return Err(ConnectError::WriteError(WriteError::Deserializer(e)));
                            }
                        }

                        Ok(res)
                    }

                    Some(Err(e)) => {
                        match e.kind() {
                            io::ErrorKind::TimedOut => Ok(None), // timeout use to check for input on stdin
                            io::ErrorKind::WouldBlock => Ok(None), // trigger on Linux when there is no stream input
                            _ => Err(WriteError::Io(e))?,
                        }
                    }
                    None => Err(ConnectError::Disconnected),
                }
            }
        }
    }
}
