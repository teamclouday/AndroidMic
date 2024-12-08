use cosmic::iced::futures::{SinkExt, Stream};
use cosmic::iced::stream;
use either::Either;
use futures::{
    future::{self},
    pin_mut,
};
use rtrb::Producer;
use std::net::IpAddr;
use tokio::sync::mpsc::{self, Sender};

use crate::streamer::{StreamerTrait, WriteError};

use super::{
    adb_streamer, tcp_streamer, udp_streamer, usb_streamer, ConnectError, DummyStreamer, Status,
    Streamer,
};

#[derive(Debug)]
pub enum ConnectOption {
    Tcp { ip: IpAddr },
    Udp { ip: IpAddr },
    Adb,
    Usb,
}

/// App -> Streamer
#[derive(Debug)]
pub enum StreamerCommand {
    Connect(ConnectOption, Producer<u8>),
    ChangeBuff(Producer<u8>),
    Stop,
}

/// Streamer -> App
#[derive(Debug, Clone)]
pub enum StreamerMsg {
    Status(Status),
    Ready(Sender<StreamerCommand>),
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
                let process_future = streamer.poll_status();

                pin_mut!(recv_future);
                pin_mut!(process_future);

                // This map it to remove the weird lifetime of future::Either
                match future::select(recv_future, process_future).await {
                    future::Either::Left((res, _)) => Either::Left(res),
                    future::Either::Right((res, _)) => Either::Right(res),
                }
            };

            match either {
                Either::Left(command) => match command {
                    Some(command) => match command {
                        StreamerCommand::Connect(connect_option, producer) => {
                            let mut new_streamer = match connect_option {
                                ConnectOption::Tcp { ip } => {
                                    Streamer::from(tcp_streamer::new(ip, producer))
                                }
                                ConnectOption::Udp { ip } => {
                                    Streamer::from(udp_streamer::new(ip, producer))
                                }
                                ConnectOption::Adb => Streamer::from(adb_streamer::new(producer)),
                                ConnectOption::Usb => Streamer::from(usb_streamer::new(producer)),
                            };

                            match new_streamer.start().await {
                                Ok(()) => {
                                    streamer = new_streamer;
                                    let status = streamer.poll_status().await.unwrap().unwrap();
                                    send(&mut sender, StreamerMsg::Status(status)).await;
                                }
                                Err(e) => {
                                    error!("{:#?}", e);
                                    send(
                                        &mut sender,
                                        StreamerMsg::Status(Status::Error(e.to_string())),
                                    )
                                    .await;
                                }
                            }
                        }
                        StreamerCommand::ChangeBuff(producer) => {
                            streamer.set_buff(producer).await;
                        }
                        StreamerCommand::Stop => {
                            streamer.shutdown().await;
                            streamer = DummyStreamer::new();
                        }
                    },
                    None => {}
                },
                Either::Right(res) => match res {
                    Ok(status) => {
                        if let Some(status) = status {
                            send(&mut sender, StreamerMsg::Status(status)).await;
                        }
                    }
                    Err(connect_error) => {
                        error!("{connect_error}");

                        if !matches!(
                            connect_error,
                            ConnectError::WriteError(WriteError::BufferOverfilled(..))
                        ) {
                            send(
                                &mut sender,
                                StreamerMsg::Status(Status::Error(connect_error.to_string())),
                            )
                            .await;
                            streamer = DummyStreamer::new();
                        }
                    }
                },
            }
        }
    })
}
