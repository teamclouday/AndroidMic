use anyhow::Result;
use futures::StreamExt;
use prost::Message;
use rtrb::Producer;
use std::{net::IpAddr, sync::Arc};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::Mutex,
    time::Duration,
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{
    config::AudioFormat,
    streamer::{AudioPacketMessage, AudioWaveData, WriteError, DEFAULT_PC_PORT, MAX_PORT},
};

use super::{ConnectError, Status, StreamerTrait};

const MAX_WAIT_TIME: Duration = Duration::from_millis(100);

pub struct TcpStreamer {
    ip: IpAddr,
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

#[allow(clippy::large_enum_variant)]
pub enum TcpStreamerState {
    Listening {
        listener: TcpListener,
    },
    Streaming {
        framed: Framed<TcpStream, LengthDelimitedCodec>,
    },
}

pub fn new(ip: IpAddr, producer: Producer<u8>) -> TcpStreamer {
    TcpStreamer {
        ip,
        producer: Arc::new(Mutex::new(producer)),
        status: Arc::new(Mutex::new(None)),
        action: Arc::new(Mutex::new(None)),
        task: None,
        task_running: Arc::new(Mutex::new(false)),
    }
}

impl StreamerTrait for TcpStreamer {
    async fn start(&mut self) -> Result<(), ConnectError> {
        let mut listener = None;

        // try to always bind the same port, to not change it everytime Android side
        for p in DEFAULT_PC_PORT..=MAX_PORT {
            if let Ok(l) = TcpListener::bind((self.ip, p)).await {
                listener = Some(l);
                break;
            }
        }

        let listener = if let Some(listener) = listener {
            listener
        } else {
            TcpListener::bind((self.ip, 0))
                .await
                .map_err(ConnectError::CantBindPort)?
        };
        let addr = TcpListener::local_addr(&listener).map_err(ConnectError::NoLocalAddress)?;

        info!("TCP server listening on {:?}:{}", addr.ip(), addr.port());

        *self.status.lock().await = Some(Status::Listening {
            port: Some(addr.port()),
        });
        *self.task_running.lock().await = true;

        let status = self.status.clone();
        let action = self.action.clone();
        let producer = self.producer.clone();
        let task_running = self.task_running.clone();

        self.task = Some(tokio::task::spawn(async move {
            let mut framed_created: Option<Framed<TcpStream, LengthDelimitedCodec>> = None;

            loop {
                if !*task_running.lock().await {
                    info!("TCP server is shutting down");
                    break;
                }

                let mut status_lock = status.lock().await;

                match *status_lock {
                    Some(Status::Listening { .. }) => {
                        match listener.accept().await {
                            Ok((stream, addr)) => {
                                info!("Connection accepted from: {:?}:{}", addr.ip(), addr.port());
                                *status_lock = Some(Status::Connected);
                                framed_created =
                                    Some(Framed::new(stream, LengthDelimitedCodec::new()));
                                continue; // continue to skip the sleep
                            }
                            Err(e) => {
                                warn!("Error accepting connection: {}", e);
                                *action.lock().await = Some(Err(ConnectError::CantAccept(e)));
                            }
                        };
                    }
                    Some(Status::Connected) => {
                        if let Some(framed) = &mut framed_created {
                            if let Some(Ok(frame)) = framed.next().await {
                                match AudioPacketMessage::decode(frame) {
                                    Ok(packet) => {
                                        let mut producer_lock = producer.lock().await;

                                        let buffer_size = packet.buffer.len();
                                        let chunk_size =
                                            std::cmp::min(buffer_size, producer_lock.slots());

                                        // mapping from android AudioFormat to encoding size
                                        let audio_format =
                                            AudioFormat::from_android_format(packet.audio_format)
                                                .unwrap();
                                        let encoding_size = audio_format.sample_size()
                                            * packet.channel_count as usize;

                                        // make sure chunk_size is a multiple of encoding_size
                                        let correction = chunk_size % encoding_size;

                                        match producer_lock
                                            .write_chunk_uninit(chunk_size - correction)
                                        {
                                            Ok(chunk) => {
                                                // compute the audio wave from the buffer
                                                if let Some(audio_wave_data) = packet.to_f32_vec() {
                                                    *action.lock().await =
                                                        Some(Ok(Some(Status::UpdateAudioWave {
                                                            data: audio_wave_data,
                                                        })));
                                                }

                                                chunk.fill_from_iter(packet.buffer.into_iter());
                                                info!(
                                                    "received {} bytes, corrected {} bytes, lost {} bytes",
                                                    buffer_size,
                                                    correction,
                                                    buffer_size - chunk_size + correction
                                                );

                                                continue; // continue to skip the sleep
                                            }
                                            Err(e) => {
                                                warn!("dropped packet: {}", e);
                                            }
                                        };
                                    }
                                    Err(e) => {
                                        *action.lock().await = Some(Err(ConnectError::WriteError(
                                            WriteError::Deserializer(e),
                                        )));
                                    }
                                }
                            } else {
                                info!("frame not ready");
                            }
                        }
                    }
                    _ => {}
                }

                // sleep for a while to not consume all CPU
                tokio::time::sleep(MAX_WAIT_TIME).await;
            }

            if let Some(framed) = &mut framed_created {
                framed.get_mut().shutdown().await.ok();
            }

            drop(framed_created);

            info!("TCP server stopped successfully");
        }));

        Ok(())
    }

    async fn poll_status(&mut self) -> Result<Option<Status>, ConnectError> {
        let status = self.status.lock().await.clone();
        let mut action_lock = self.action.lock().await;
        if let Some(action) = action_lock.take() {
            return action;
        }

        Ok(status)
    }

    async fn set_buff(&mut self, producer: Producer<u8>) {
        *self.producer.lock().await = producer;
    }

    async fn shutdown(&mut self) {
        *self.task_running.lock().await = false;

        if let Some(task) = self.task.take() {
            task.await.ok();
        }
    }
}
