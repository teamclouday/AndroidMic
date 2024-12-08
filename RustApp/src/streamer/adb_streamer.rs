use adb_client::ADBServer;
use anyhow::Result;
use rtrb::Producer;

use crate::streamer::tcp_streamer;

use super::{tcp_streamer::TcpStreamer, ConnectError, Status, StreamerTrait};

pub struct AdbStreamer {
    tcp_streamer: TcpStreamer,
}

fn start_reverse_proxy(port: u16) -> Result<(), ConnectError> {
    let mut server = ADBServer::default();
    let mut device = server.get_device().map_err(ConnectError::AdbFailed)?;

    // remove all reverse proxy
    device
        .reverse_remove_all()
        .map_err(ConnectError::AdbFailed)?;

    // start reverse proxy
    device
        .reverse("tcp:6000".to_string(), format!("tcp:{port}"))
        .map_err(ConnectError::AdbFailed)?;

    Ok(())
}

fn stop_reverse_proxy() -> Result<(), ConnectError> {
    let mut server = ADBServer::default();
    let mut device = server.get_device().map_err(ConnectError::AdbFailed)?;

    // remove all reverse proxy
    device
        .reverse_remove_all()
        .map_err(ConnectError::AdbFailed)?;

    Ok(())
}

pub fn new(producer: Producer<u8>) -> AdbStreamer {
    let tcp_streamer = tcp_streamer::new(str::parse("127.0.0.1").unwrap(), producer);
    AdbStreamer { tcp_streamer }
}

impl StreamerTrait for AdbStreamer {
    async fn poll_status(&mut self) -> Result<Option<Status>, ConnectError> {
        self.tcp_streamer.poll_status().await
    }

    async fn start(&mut self) -> Result<(), ConnectError> {
        start_reverse_proxy(6000)?;
        self.tcp_streamer.start().await
    }

    async fn set_buff(&mut self, buff: Producer<u8>) {
        self.tcp_streamer.set_buff(buff).await
    }

    async fn shutdown(&mut self) {
        stop_reverse_proxy().ok();
        self.tcp_streamer.shutdown().await
    }
}
