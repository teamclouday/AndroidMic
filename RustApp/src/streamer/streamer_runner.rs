use cosmic::iced::futures::SinkExt;
use cosmic::iced::stream;
use either::Either;
use futures::Stream;
use futures::{
    future::{self},
    pin_mut,
};
use rtrb::Producer;
use std::fmt::Debug;
use std::net::IpAddr;
use tokio::sync::mpsc::{self, Sender};

use crate::audio::AudioProcessParams;
use crate::config::ConnectionMode;
use crate::streamer::{StreamerTrait, WriteError};

use super::{AudioStream, ConnectError, DummyStreamer, Streamer, tcp_streamer, udp_streamer};

#[derive(Debug)]
pub enum ConnectOption {
    Tcp {
        ip: IpAddr,
        port: u16,
    },
    Udp {
        ip: IpAddr,
        port: u16,
    },
    #[cfg(feature = "adb")]
    Adb,
    #[cfg(feature = "usb")]
    Usb,
}

/// App -> Streamer
pub enum StreamerCommand {
    Connect {
        connect_options: ConnectOption,
        buff: Producer<u8>,
        audio_params: AudioProcessParams,
        is_window_visible: bool,
    },
    ReconfigureStream {
        buff: Producer<u8>,
        audio_params: AudioProcessParams,
        is_window_visible: bool,
    },
    Stop,
}

impl Debug for StreamerCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect {
                connect_options,
                buff: _,
                audio_params,
                is_window_visible,
            } => f
                .debug_struct("Connect")
                .field("connect_options", connect_options)
                .field("audio_params", audio_params)
                .field("is_window_visible", is_window_visible)
                .finish(),
            Self::ReconfigureStream {
                buff: _,
                audio_params,
                is_window_visible,
            } => f
                .debug_struct("ReconfigureStream")
                .field("audio_params", audio_params)
                .field("is_window_visible", is_window_visible)
                .finish(),
            Self::Stop => write!(f, "Stop"),
        }
    }
}

/// Streamer -> App
#[derive(Debug, Clone)]
pub enum StreamerMsg {
    UpdateAudioWave {
        data: Vec<(f32, f32)>,
    },
    Error(String),
    Listening {
        ip: Option<IpAddr>,
        port: Option<u16>,
    },
    Connected {
        ip: Option<IpAddr>,
        port: Option<u16>,
        mode: ConnectionMode,
    },
    Ready(Sender<StreamerCommand>),
}

impl StreamerMsg {
    fn is_error(&self) -> bool {
        matches!(self, StreamerMsg::Error(..))
    }
}

async fn send(sender: &mut futures::channel::mpsc::Sender<StreamerMsg>, msg: StreamerMsg) {
    sender.send(msg).await.unwrap();
}

pub fn sub() -> impl Stream<Item = StreamerMsg> {
    stream::channel(5, |mut sender| async move {
        let (command_sender, mut command_receiver) = mpsc::channel(100);

        let mut streamer: Streamer = DummyStreamer::new();

        send(&mut sender, StreamerMsg::Ready(command_sender)).await;

        loop {
            let either = {
                let recv_future = command_receiver.recv();
                let process_future = streamer.next();

                pin_mut!(recv_future);
                pin_mut!(process_future);

                // This map it to remove the weird lifetime of future::Either
                match future::select(recv_future, process_future).await {
                    future::Either::Left((res, _)) => Either::Left(res),
                    future::Either::Right((res, _)) => Either::Right(res),
                }
            };

            match either {
                Either::Left(command) => {
                    if let Some(command) = command {
                        info!("received command {command:?}");
                        match command {
                            StreamerCommand::Connect {
                                connect_options,
                                buff,
                                audio_params,
                                is_window_visible,
                            } => {
                                let stream_config =
                                    AudioStream::new(buff, audio_params, is_window_visible);
                                let new_streamer: Result<Streamer, ConnectError> =
                                    match connect_options {
                                        ConnectOption::Tcp { ip, port } => {
                                            tcp_streamer::new(ip, port, stream_config)
                                                .await
                                                .map(Streamer::from)
                                        }
                                        #[cfg(feature = "adb")]
                                        ConnectOption::Adb => {
                                            crate::streamer::adb_streamer::new(stream_config)
                                                .await
                                                .map(Streamer::from)
                                        }
                                        ConnectOption::Udp { ip, port } => {
                                            udp_streamer::new(ip, port, stream_config)
                                                .await
                                                .map(Streamer::from)
                                        }
                                        #[cfg(feature = "usb")]
                                        ConnectOption::Usb => {
                                            crate::streamer::usb_streamer::new(stream_config)
                                                .await
                                                .map(Streamer::from)
                                        }
                                    };

                                match new_streamer {
                                    Ok(new_streamer) => {
                                        send(&mut sender, new_streamer.status()).await;
                                        drop(streamer);
                                        streamer = new_streamer;
                                    }
                                    Err(e) => {
                                        error!("{e}");
                                        send(&mut sender, StreamerMsg::Error(e.to_string())).await;
                                    }
                                }
                            }
                            StreamerCommand::ReconfigureStream {
                                buff,
                                audio_params,
                                is_window_visible,
                            } => {
                                let stream_config =
                                    AudioStream::new(buff, audio_params, is_window_visible);

                                streamer.reconfigure_stream(stream_config);
                            }
                            StreamerCommand::Stop => {
                                drop(streamer);
                                streamer = DummyStreamer::new();
                            }
                        }
                    }
                }
                Either::Right(res) => match res {
                    Ok(status) => {
                        if let Some(status) = status {
                            send(&mut sender, status).await;
                        }
                    }
                    Err(connect_error) => {
                        error!("{connect_error}");

                        if !matches!(
                            connect_error,
                            ConnectError::WriteError(WriteError::BufferOverfilled(..))
                        ) {
                            send(&mut sender, StreamerMsg::Error(connect_error.to_string())).await;
                            streamer = DummyStreamer::new();
                        }
                    }
                },
            }
        }
    })
}
