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

struct Sub {
    sender: futures::channel::mpsc::Sender<StreamerMsg>,
    shared_buf: Option<Producer<u8>>,
    streamer: Option<Streamer>,
}

impl Sub {
    async fn process_command(&mut self, command: StreamerCommand) {
        match command {
            StreamerCommand::Connect(connection_mode, producer) => {
                let ip: IpAddr = str::parse("ip").unwrap();

                let streamer: Result<Streamer, streamer::Error> = match connection_mode {
                    ConnectionMode::Tcp => tcp_streamer_async::new(ip).await.map(Streamer::from),
                    ConnectionMode::Udp => todo!(),
                    ConnectionMode::Adb => adb_streamer::new(ip).map(Streamer::from),
                };

                match streamer {
                    Ok(streamer) => {
                        self.streamer = Some(streamer);
                        self.shared_buf = Some(producer);
                    }
                    Err(e) => {
                        error!("{e}");
                    }
                }
            }
            StreamerCommand::ChangeBuff(producer) => self.shared_buf = Some(producer),
            StreamerCommand::Stop => {
                self.streamer = None;
                self.shared_buf = None
            }
        }
    }

    async fn send(&mut self, msg: StreamerMsg) {
        self.sender.send(msg).await.unwrap();
    }
}

pub fn sub() -> impl Stream<Item = StreamerMsg> {
    stream::channel(500, | sender| async move {
        let (tx, mut rx) = mpsc::channel(100);

        let mut sub = Sub {
            sender,
            shared_buf: None,
            streamer: None,
        };

        sub.send(StreamerMsg::Ready(tx));

        loop {
            if let (Some(streamer), Some(shared_buf)) = (&mut sub.streamer, &mut sub.shared_buf) {
                let recv_future = rx.recv();
                let process_future = streamer.process(shared_buf);

                pin_mut!(recv_future);
                pin_mut!(process_future);

                match future::select(recv_future, process_future).await {
                    Either::Left((Some(command), _)) => {
                        sub.process_command(command).await;
                    }
                    Either::Left((None, _)) => {
                        todo!()
                    }
                    Either::Right((process_result, _)) => {
                        todo!()
                    }
                }
            } else {
                match rx.recv().await {
                    Some(command) => {
                        sub.process_command(command).await;
                    }
                    None => todo!(),
                }
            }
        }
    })
}
