use futures::StreamExt;
use prost::Message;
use rtrb::Producer;
use std::{net::IpAddr, time::Duration};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{
    config::AudioFormat,
    streamer::{AudioPacketMessage, AudioWaveData, WriteError, DEFAULT_PC_PORT, MAX_PORT},
};

use super::{ConnectError, Status, StreamerTrait};

const MAX_WAIT_TIME: Duration = Duration::from_millis(100);

const DISCONNECT_LOOP_DETECTER_MAX: u32 = 1000;

pub struct TcpStreamer {
    ip: IpAddr,
    pub port: u16,
    producer: Producer<u8>,
    state: TcpStreamerState,
}

#[allow(clippy::large_enum_variant)]
pub enum TcpStreamerState {
    Listening {
        listener: TcpListener,
    },
    Streaming {
        framed: Framed<TcpStream, LengthDelimitedCodec>,
    },
}

pub async fn new(ip: IpAddr, producer: Producer<u8>) -> Result<TcpStreamer, ConnectError> {
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
        producer,
        state: TcpStreamerState::Listening { listener },
    };

    Ok(streamer)
}

impl StreamerTrait for TcpStreamer {
    fn set_buff(&mut self, producer: Producer<u8>) {
        self.producer = producer;
    }

    fn status(&self) -> Option<Status> {
        match &self.state {
            TcpStreamerState::Listening { .. } => Some(Status::Listening {
                port: Some(self.port),
            }),
            TcpStreamerState::Streaming { .. } => Some(Status::Connected),
        }
    }

    async fn next(&mut self) -> Result<Option<Status>, ConnectError> {
        match &mut self.state {
            TcpStreamerState::Listening { listener } => {
                let addr =
                    TcpListener::local_addr(listener).map_err(ConnectError::NoLocalAddress)?;

                info!("TCP server listening on {}", addr);

                let (stream, addr) = listener.accept().await.map_err(ConnectError::CantAccept)?;

                info!("connection accepted, remote address: {}", addr);

                self.state = TcpStreamerState::Streaming {
                    framed: Framed::new(stream, LengthDelimitedCodec::new()),
                };

                Ok(Some(Status::Connected))
            }
            TcpStreamerState::Streaming { framed } => {
                if let Some(Ok(frame)) = framed.next().await {
                    match AudioPacketMessage::decode(frame) {
                        Ok(packet) => {
                            let buffer_size = packet.buffer.len();
                            let chunk_size = std::cmp::min(buffer_size, self.producer.slots());

                            // mapping from android AudioFormat to encoding size
                            let audio_format =
                                AudioFormat::from_android_format(packet.audio_format).unwrap();
                            let encoding_size =
                                audio_format.sample_size() * packet.channel_count as usize;

                            // make sure chunk_size is a multiple of encoding_size
                            let correction = chunk_size % encoding_size;

                            match self.producer.write_chunk_uninit(chunk_size - correction) {
                                Ok(chunk) => {
                                    // compute the audio wave from the buffer
                                    let response =
                                        if let Some(audio_wave_data) = packet.to_f32_vec() {
                                            Some(Status::UpdateAudioWave {
                                                data: audio_wave_data,
                                            })
                                        } else {
                                            None
                                        };

                                    chunk.fill_from_iter(packet.buffer.into_iter());
                                    info!(
                                        "received {} bytes, corrected {} bytes, lost {} bytes",
                                        buffer_size,
                                        correction,
                                        buffer_size - chunk_size + correction
                                    );

                                    Ok(response)
                                }
                                Err(e) => {
                                    warn!("dropped packet: {}", e);
                                    Ok(None)
                                }
                            }
                        }
                        Err(e) => {
                            return Err(WriteError::Deserializer(e).into());
                        }
                    }
                } else {
                    info!("frame not ready");
                    // sleep for a while to not consume all CPU
                    tokio::time::sleep(MAX_WAIT_TIME).await;
                    Ok(None)
                }
            }
        }
    }
}
