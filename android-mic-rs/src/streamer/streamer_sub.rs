use std::net::IpAddr;

use cosmic::iced::futures::{SinkExt, Stream};
use futures::{
    future::{self, Either},
    pin_mut,
};
use rtrb::Producer;
use tokio::sync::mpsc::{self, Sender};

use cosmic::iced::stream;

use crate::streamer::StreamerTrait;

use super::{
    adb_streamer, tcp_streamer_async, ConnectError, Dummy, Status, Streamer, StreamerContainer,
};

pub struct StatusSender(futures::channel::mpsc::Sender<StreamerMsg>);

impl StatusSender {
    async fn send(&mut self, status: Status) {
        self.0.send(StreamerMsg::Status(status)).await.unwrap();
    }
}

/// Streamer -> App
#[derive(Debug, Clone)]
pub enum StreamerMsg {
    Status(Status),
    Ready(Sender<StreamerCommand>),
}

#[derive(Debug)]
pub enum ConnectOption {
    Tcp { ip: IpAddr },
    Udp { ip: IpAddr },
    Adb,
}

/// App -> Streamer
#[derive(Debug)]
pub enum StreamerCommand {
    Connect(ConnectOption, Producer<u8>),
    ChangeBuff(Producer<u8>),
    Stop,
}

async fn send(sender: &mut futures::channel::mpsc::Sender<StreamerMsg>, msg: StreamerMsg) {
    sender.send(msg).await.unwrap();
}

pub fn sub() -> impl Stream<Item = StreamerMsg> {
    stream::channel(5, |mut sender| async move {
        let (tx, mut rx) = mpsc::channel(100);

        let mut streamer: StreamerContainer = StreamerContainer::default();

        send(&mut sender, StreamerMsg::Ready(tx)).await;

        loop {
            let recv_future = rx.recv();
            let process_future = streamer.next();

            pin_mut!(recv_future);
            pin_mut!(process_future);

            match future::select(recv_future, process_future).await {
                Either::Left((command, _)) => match command {
                    Some(command) => match command {
                        StreamerCommand::Connect(connect_option, producer) => {
                            let new_streamer: Result<Streamer, ConnectError> = match connect_option
                            {
                                ConnectOption::Tcp { ip } => tcp_streamer_async::new(ip, producer)
                                    .await
                                    .map(Streamer::from),
                                ConnectOption::Udp { ip: _ip } => todo!(),
                                ConnectOption::Adb => {
                                    adb_streamer::new(producer).await.map(Streamer::from)
                                }
                            };

                            match new_streamer {
                                Ok(new_streamer) => {
                                    send(
                                        &mut sender,
                                        StreamerMsg::Status(new_streamer.status().unwrap()),
                                    )
                                    .await;
                                    streamer.replace(new_streamer);
                                }
                                Err(e) => {
                                    error!("{e}");
                                    send(
                                        &mut sender,
                                        StreamerMsg::Status(Status::Error(e.to_string())),
                                    )
                                    .await;
                                }
                            }
                        }
                        StreamerCommand::ChangeBuff(producer) => {
                            streamer.set_buff(producer);
                        }
                        StreamerCommand::Stop => {
                            streamer.take();
                        }
                    },
                    None => todo!(),
                },
                Either::Right((res, _)) => match res {
                    Ok(status) => {
                        if let Some(status) = status {
                            send(&mut sender, StreamerMsg::Status(status)).await;
                        }
                    }
                    Err(e) => {
                        send(
                            &mut sender,
                            StreamerMsg::Status(Status::Error(e.to_string())),
                        )
                        .await;
                        streamer.take();
                    }
                },
            }
        }
    })
}
