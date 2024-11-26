use std::{
    net::IpAddr,
    pin::{pin, Pin},
};

use cosmic::iced::futures::{SinkExt, Stream};
use futures::{
    future::{self, Either},
    pin_mut,
};
use rtrb::Producer;
use tokio::{
    select,
    sync::mpsc::{self, Sender},
};

use crate::{
    adb_streamer,
    app::AppMsg,
    config::ConnectionMode,
    streamer::{self, Status, Streamer, StreamerTrait},
    tcp_streamer_async::{self, TcpStreamer},
};

use cosmic::iced::stream;

/// Streamer -> App
#[derive(Debug, Clone)]
pub enum StreamerMsg {
    Status(Status),
    Ready(Sender<StreamerCommand>),
}

/// App -> Streamer
#[derive(Debug)]
pub enum StreamerCommand {
    Connect(ConnectionMode, Producer<u8>),
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
                    Either::Right((process_result, _)) => {
                        // todo: stats ?
                    }
                }
            } else {
                command = Some(rx.recv().await);
            }

            if let Some(command) = command.take() {
                match command {
                    Some(command) => match command {
                        StreamerCommand::Connect(connection_mode, producer) => {
                            let ip: IpAddr = str::parse("ip").unwrap();

                            let streamer2: Result<Streamer, streamer::ConnectError> = match connection_mode
                            {
                                ConnectionMode::Tcp => {
                                    tcp_streamer_async::new(ip).await.map(Streamer::from)
                                }
                                ConnectionMode::Udp => todo!(),
                                ConnectionMode::Adb => adb_streamer::new(ip).map(Streamer::from),
                            };

                            match streamer2 {
                                Ok(streamer2) => {
                                    streamer.replace(streamer2);
                                    shared_buf.replace(producer);
                                }
                                Err(e) => {
                                    error!("{e}");
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
