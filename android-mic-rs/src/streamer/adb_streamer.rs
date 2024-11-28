use std::fs;

use rtrb::Producer;
use tokio::fs::read_dir;

use crate::utils;

use super::{
    streamer_trait::{ConnectError, StreamerTrait, WriteError},
    tcp_streamer_async::{self, TcpStreamer},
};

pub struct AdbStreamer {
    tcp_streamer: TcpStreamer,
}

pub async fn new() -> Result<AdbStreamer, ConnectError> {

    
    dbg!(&utils::resource_dir().display());
    
    let tcp_streamer = tcp_streamer_async::new(str::parse("127.0.0.1").unwrap()).await?;
    
    

    let streamer = AdbStreamer { tcp_streamer };
    Ok(streamer)
}

impl StreamerTrait for AdbStreamer {
    async fn process(&mut self, shared_buf: &mut Producer<u8>) -> Result<usize, WriteError> {
        self.tcp_streamer.process(shared_buf).await
    }

    fn port(&self) -> Option<u16> {
        self.tcp_streamer.port()
    }
}
