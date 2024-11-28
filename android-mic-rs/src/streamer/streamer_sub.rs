use std::net::IpAddr;

use cosmic::iced::futures::{SinkExt, Stream};
use futures::{
    future::{self, Either},
    pin_mut,
};
use rtrb::Producer;
use tokio::sync::mpsc::{self, Sender};

use cosmic::iced::stream;

use crate::streamer::streamer_trait::StreamerTrait;

use super::{
    adb_streamer,
    streamer_trait::{self, Streamer},
    tcp_streamer_async,
};

#[derive(Clone, Debug)]
pub enum Status {
    Error(String),
    Listening,
    Connected { port: Option<u16> },
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

        let mut shared_buf: Option<Producer<u8>> = None;
        let mut streamer: Option<Streamer> = None;

        send(&mut sender, StreamerMsg::Ready(tx)).await;

        let mut command: Option<Option<StreamerCommand>> = None;

        loop {
            if let (Some(streamer2), Some(shared_buf2)) = (&mut streamer, &mut shared_buf) {
                let recv_future = rx.recv();
                let process_future = streamer2.process(shared_buf2);

                pin_mut!(recv_future);
                pin_mut!(process_future);

                match future::select(recv_future, process_future).await {
                    Either::Left((res, _)) => {
                        command = Some(res);
                    }
                    Either::Right((_process_result, _)) => {
                        // todo: stats ?
                    }
                }
            } else {
                command = Some(rx.recv().await);
            }

            if let Some(command) = command.take() {
                match command {
                    Some(command) => match command {
                        StreamerCommand::Connect(connect_option, producer) => {
                            let new_streamer: Result<Streamer, streamer_trait::ConnectError> =
                                match connect_option {
                                    ConnectOption::Tcp { ip } => {
                                        tcp_streamer_async::new(ip).await.map(Streamer::from)
                                    }
                                    ConnectOption::Udp { ip: _ip } => todo!(),
                                    ConnectOption::Adb => {
                                        adb_streamer::new().await.map(Streamer::from)
                                    }
                                };

                            match new_streamer {
                                Ok(new_streamer) => {
                                    send(
                                        &mut sender,
                                        StreamerMsg::Status(Status::Connected {
                                            port: new_streamer.port(),
                                        }),
                                    )
                                    .await;
                                    streamer.replace(new_streamer);
                                    shared_buf.replace(producer);
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
                            shared_buf.replace(producer);
                        }
                        StreamerCommand::Stop => {
                            streamer.take();
                            shared_buf.take();
                        }
                    },
                    None => todo!(),
                }
            }
        }
    })
}
