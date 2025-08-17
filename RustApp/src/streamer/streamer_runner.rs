use cosmic::iced::futures::SinkExt;
use cosmic::iced::stream;
use either::Either;
use futures::Stream;
use futures::{
    future::{self},
    pin_mut,
};
use std::net::IpAddr;
use tokio::sync::mpsc::{self, Sender};

use crate::streamer::{StreamerTrait, WriteError};

use super::{
    AudioStream, ConnectError, DummyStreamer, Streamer, adb_streamer, tcp_streamer, udp_streamer,
};

#[derive(Debug)]
pub enum ConnectOption {
    Tcp {
        ip: IpAddr,
    },
    Udp {
        ip: IpAddr,
    },
    Adb,
    #[cfg(feature = "usb")]
    Usb,
}

/// App -> Streamer
#[derive(Debug)]
pub enum StreamerCommand {
    Connect(ConnectOption, AudioStream),
    ReconfigureStream(AudioStream),
    Stop,
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
                            StreamerCommand::Connect(connect_option, stream_config) => {
                                let new_streamer: Result<Streamer, ConnectError> =
                                    match connect_option {
                                        ConnectOption::Tcp { ip } => {
                                            tcp_streamer::new(ip, stream_config)
                                                .await
                                                .map(Streamer::from)
                                        }
                                        ConnectOption::Adb => adb_streamer::new(stream_config)
                                            .await
                                            .map(Streamer::from),
                                        ConnectOption::Udp { ip } => {
                                            udp_streamer::new(ip, stream_config)
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
                                        streamer = new_streamer;
                                    }
                                    Err(e) => {
                                        error!("{e}");
                                        send(&mut sender, StreamerMsg::Error(e.to_string())).await;
                                    }
                                }
                            }
                            StreamerCommand::ReconfigureStream(stream_config) => {
                                streamer.reconfigure_stream(stream_config);
                            }
                            StreamerCommand::Stop => {
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
