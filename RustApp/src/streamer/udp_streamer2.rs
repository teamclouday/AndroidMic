use anyhow::Result;
use futures::StreamExt;
use prost::Message;
use rtrb::Producer;
use std::{net::IpAddr, sync::Arc};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::Duration;
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
    producer: Arc<Mutex<Producer<u8>>>,
    // set once, read many times
    status: Arc<Mutex<Option<Status>>>,
    // read once, discarded after
    action: Arc<Mutex<Option<Result<Option<Status>, ConnectError>>>>,
    // the task handle
    task: Option<tokio::task::JoinHandle<()>>,
    // task cancel signal sender
    task_running: Arc<Mutex<bool>>,
}

pub fn new(ip: IpAddr, producer: Producer<u8>) -> UdpStreamer {
    UdpStreamer {
        ip,
        port: 0,
        producer: Arc::new(Mutex::new(producer)),
        status: Arc::new(Mutex::new(None)),
        action: Arc::new(Mutex::new(None)),
        task: None,
        task_running: Arc::new(Mutex::new(false)),
    }
}

impl StreamerTrait for UdpStreamer {
    async fn start(&mut self) -> Result<(), ConnectError> {
        let mut socket = None;

        // try to always bind the same port, to not change it everytime Android side
        for p in DEFAULT_PC_PORT..=MAX_PORT {
            if let Ok(l) = UdpSocket::bind((self.ip, p)).await {
                socket = Some(l);
                break;
            }
        }

        let socket = if let Some(socket) = socket {
            socket
        } else {
            UdpSocket::bind((self.ip, 0))
                .await
                .map_err(ConnectError::CantBindPort)?
        };

        let addr = socket.local_addr().map_err(ConnectError::NoLocalAddress)?;
        self.port = addr.port();

        info!("UDP server listening on {:?}:{}", addr.ip(), addr.port());

        *self.status.lock().await = Some(Status::Listening {
            port: Some(addr.port()),
        });
        *self.task_running.lock().await = true;

        let status = self.status.clone();
        let action = self.action.clone();
        let producer = self.producer.clone();
        let task_running = self.task_running.clone();

        self.task = Some(tokio::task::spawn(async move {
            let mut framed = UdpFramed::new(socket, LengthDelimitedCodec::new());
            let mut tracked_sequence = 0u32;

            loop {
                if !*task_running.lock().await {
                    info!("UDP server is shutting down");
                    break;
                }

                if let Some(Ok((frame, addr))) = framed.next().await {
                    *status.lock().await = Some(Status::Connected);
                    match AudioPacketMessageOrdered::decode(frame) {
                        Ok(packet) => {
                            let mut producer_lock = producer.lock().await;

                            if packet.sequence_number < tracked_sequence {
                                // drop packet
                                info!(
                                    "dropped packet: old sequence number {} < {}",
                                    packet.sequence_number, tracked_sequence
                                );
                            }
                            tracked_sequence = packet.sequence_number;

                            let packet = packet.audio_packet.unwrap();
                            let buffer_size = packet.buffer.len();
                            let chunk_size = std::cmp::min(buffer_size, producer_lock.slots());

                            // mapping from android AudioFormat to encoding size
                            let audio_format =
                                AudioFormat::from_android_format(packet.audio_format).unwrap();
                            let encoding_size =
                                audio_format.sample_size() * packet.channel_count as usize;

                            // make sure chunk_size is a multiple of encoding_size
                            let correction = chunk_size % encoding_size;

                            match producer_lock.write_chunk_uninit(chunk_size - correction) {
                                Ok(chunk) => {
                                    // compute the audio wave from the buffer
                                    if let Some(audio_wave_data) = packet.to_f32_vec() {
                                        *action.lock().await =
                                            Some(Ok(Some(Status::UpdateAudioWave {
                                                data: audio_wave_data,
                                            })));
                                    };

                                    chunk.fill_from_iter(packet.buffer.into_iter());
                                    info!(
                                        "From {:?}, received {} bytes, corrected {} bytes, lost {} bytes",
                                        addr,
                                        buffer_size,
                                        correction,
                                        buffer_size - chunk_size + correction
                                    );

                                    continue; // continue to skip the sleep
                                }
                                Err(e) => {
                                    warn!("dropped packet: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            *action.lock().await = Some(Err(WriteError::Deserializer(e).into()));
                        }
                    }
                } else {
                    info!("frame not ready");
                }

                // sleep for a while to not consume all CPU
                tokio::time::sleep(MAX_WAIT_TIME).await;
            }

            drop(framed);

            info!("UDP server stopped successfully");
        }));

        Ok(())
    }

    async fn set_buff(&mut self, buff: Producer<u8>) {
        *self.producer.lock().await = buff;
    }

    async fn poll_status(&mut self) -> Result<Option<Status>, ConnectError> {
        let status = self.status.lock().await.clone();
        let mut action_lock = self.action.lock().await;
        if let Some(action) = action_lock.take() {
            return action;
        }

        Ok(status)
    }

    async fn shutdown(&mut self) {
        *self.task_running.lock().await = false;

        if let Some(task) = self.task.take() {
            task.await.ok();
        }
    }
}
