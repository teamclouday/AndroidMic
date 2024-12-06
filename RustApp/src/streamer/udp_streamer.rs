use futures::StreamExt;
use prost::Message;
use rtrb::Producer;
use std::net::IpAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio_util::codec::LengthDelimitedCodec;
use tokio_util::udp::UdpFramed;

use crate::{
    config::AudioFormat,
    streamer::{AudioWaveData, WriteError, DEFAULT_PC_PORT, MAX_PORT},
};

use super::{AudioPacketMessageOrdered, ConnectError, Status, StreamerTrait};

const MAX_WAIT_TIME: Duration = Duration::from_millis(100);

pub struct UdpStreamer {
    ip: IpAddr,
    port: u16,
    producer: Producer<u8>,
    framed: UdpFramed<LengthDelimitedCodec, UdpSocket>,
    tracked_sequence: u32,
}

pub async fn new(ip: IpAddr, producer: Producer<u8>) -> Result<UdpStreamer, ConnectError> {
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
        producer,
        framed: UdpFramed::new(socket, LengthDelimitedCodec::new()),
        tracked_sequence: 0,
    };

    Ok(streamer)
}

impl StreamerTrait for UdpStreamer {
    fn set_buff(&mut self, producer: Producer<u8>) {
        self.producer = producer;
    }

    fn status(&self) -> Option<Status> {
        Some(Status::Connected {
            port: Some(self.port),
        })
    }

    async fn next(&mut self) -> Result<Option<Status>, ConnectError> {
        if let Some(Ok((frame, addr))) = self.framed.next().await {
            match AudioPacketMessageOrdered::decode(frame) {
                Ok(packet) => {
                    if packet.sequence_number < self.tracked_sequence {
                        // drop packet
                        info!(
                            "dropped packet: old sequence number {} < {}",
                            packet.sequence_number, self.tracked_sequence
                        );
                        return Ok(None);
                    }
                    self.tracked_sequence = packet.sequence_number;

                    let packet = packet.audio_packet.unwrap();
                    let buffer_size = packet.buffer.len();
                    let chunk_size = std::cmp::min(buffer_size, self.producer.slots());

                    // mapping from android AudioFormat to encoding size
                    let audio_format =
                        AudioFormat::from_android_format(packet.audio_format).unwrap();
                    let encoding_size = audio_format.sample_size() * packet.channel_count as usize;

                    // make sure chunk_size is a multiple of encoding_size
                    let correction = chunk_size % encoding_size;

                    match self.producer.write_chunk_uninit(chunk_size - correction) {
                        Ok(chunk) => {
                            // compute the audio wave from the buffer
                            let response = if let Some(audio_wave_data) = packet.to_f32_vec() {
                                Some(Status::UpdateAudioWave {
                                    data: audio_wave_data,
                                })
                            } else {
                                None
                            };

                            chunk.fill_from_iter(packet.buffer.into_iter());
                            info!(
                                "From {}, received {} bytes, corrected {} bytes, lost {} bytes",
                                addr,
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
