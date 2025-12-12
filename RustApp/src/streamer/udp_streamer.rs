use std::{io, net::IpAddr, time::Duration};

use futures::StreamExt;
use prost::Message;
use tokio::net::UdpSocket;
use tokio_util::{codec::LengthDelimitedCodec, udp::UdpFramed};

use crate::{
    config::ConnectionMode,
    streamer::{AudioPacketMessage, CHECK_1, CHECK_2, DEFAULT_PC_PORT, MAX_PORT, WriteError},
};

use super::{AudioPacketMessageOrdered, AudioStream, ConnectError, StreamerMsg, StreamerTrait};

const MAX_WAIT_TIME: Duration = Duration::from_millis(1500);

const DISCONNECT_LOOP_DETECTER_MAX: u32 = 1000;

pub struct UdpStreamer {
    ip: IpAddr,
    pub port: u16,
    stream_config: AudioStream,
    state: UdpStreamerState,
    framed: UdpFramed<LengthDelimitedCodec>,
}

enum UdpStreamerState {
    Listening,
    Streaming { tracked_sequence: u32 },
}

pub async fn new(ip: IpAddr, stream_config: AudioStream) -> Result<UdpStreamer, ConnectError> {
    let mut socket = None;

    // try to always bind the same port, to not change it everytime Android side
    for p in DEFAULT_PC_PORT..=MAX_PORT {
        if let Ok(l) = UdpSocket::bind((ip, p)).await {
            socket = Some(l);
            break;
        }
    }

    let socket = if let Some(socket) = socket {
        socket
    } else {
        UdpSocket::bind((ip, 0))
            .await
            .map_err(ConnectError::CantBindPort)?
    };

    let addr = socket.local_addr().map_err(ConnectError::NoLocalAddress)?;

    let streamer = UdpStreamer {
        ip,
        port: addr.port(),
        stream_config,
        state: UdpStreamerState::Listening,
        framed: UdpFramed::new(socket, LengthDelimitedCodec::new()),
    };

    Ok(streamer)
}

impl StreamerTrait for UdpStreamer {
    fn reconfigure_stream(&mut self, stream_config: AudioStream) {
        self.stream_config = stream_config;
    }

    fn status(&self) -> StreamerMsg {
        match self.state {
            UdpStreamerState::Listening => StreamerMsg::Listening {
                ip: Some(self.ip),
                port: Some(self.port),
            },
            UdpStreamerState::Streaming { .. } => StreamerMsg::Connected {
                ip: Some(self.ip),
                port: Some(self.port),
                mode: ConnectionMode::Udp,
            },
        }
    }

    async fn next(&mut self) -> Result<Option<StreamerMsg>, ConnectError> {
        match &mut self.state {
            UdpStreamerState::Listening => {
                let mut buf1 = [0u8; CHECK_1.len()];

                match self.framed.get_ref().recv_from(&mut buf1).await {
                    Ok((_, src_addr)) => {
                        if buf1 != CHECK_1.as_bytes() {
                            let s = String::from_utf8_lossy(&buf1);

                            return Err(ConnectError::HandShakeFailed2(format!(
                                "{} != {}",
                                CHECK_1, s
                            )));
                        }

                        // send back the same check bytes
                        self.framed
                            .get_ref()
                            .send_to(CHECK_2.as_bytes(), &src_addr)
                            .await
                            .map_err(|e| ConnectError::HandShakeFailed("writing", e))?;
                    }
                    Err(e) => {
                        // error 10040 is when the buffer is too small
                        // probably because the app is already in a connected state,
                        // by sending audio data
                        if !matches!(e.raw_os_error(), Some(10040)) {
                            return Err(ConnectError::HandShakeFailed("reading", e));
                        }
                    }
                }

                self.state = UdpStreamerState::Streaming {
                    tracked_sequence: 0,
                };

                Ok(Some(StreamerMsg::Connected {
                    ip: Some(self.ip),
                    port: Some(self.port),
                    mode: ConnectionMode::Udp,
                }))
            }

            UdpStreamerState::Streaming { tracked_sequence } => {
                match self.framed.next().await {
                    Some(Ok((frame, addr))) => {
                        match AudioPacketMessageOrdered::decode(frame) {
                            Ok(packet) => {
                                if packet.sequence_number < *tracked_sequence {
                                    // drop packet
                                    info!(
                                        "dropped packet: old sequence number {} < {}",
                                        packet.sequence_number, tracked_sequence
                                    );
                                }
                                *tracked_sequence = packet.sequence_number;

                                let packet = packet.audio_packet.unwrap();
                                let buffer_size = packet.buffer.len();
                                let sample_rate = packet.sample_rate;

                                match self.stream_config.process_audio_packet(packet) {
                                    Ok(Some(buffer)) => {
                                        debug!("From {:?}, received {} bytes", addr, buffer_size);
                                        Ok(Some(StreamerMsg::UpdateAudioWave {
                                            data: AudioPacketMessage::to_wave_data(
                                                &buffer,
                                                sample_rate,
                                            ),
                                        }))
                                    }
                                    _ => Ok(None),
                                }
                            }
                            Err(e) => Err(ConnectError::WriteError(WriteError::Deserializer(e))),
                        }
                    }

                    Some(Err(e)) => {
                        match e.kind() {
                            io::ErrorKind::TimedOut => Ok(None), // timeout use to check for input on stdin
                            io::ErrorKind::WouldBlock => Ok(None), // trigger on Linux when there is no stream input
                            _ => Err(WriteError::Io(e))?,
                        }
                    }
                    None => {
                        todo!()
                    }
                }
            }
        }
    }
}
