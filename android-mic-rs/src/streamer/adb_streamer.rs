use local_ip_address::local_ip;
use rtrb::Producer;

use super::{
    streamer_trait::{ConnectError, StreamerTrait, WriteError},
    tcp_streamer_async::{self, TcpStreamer},
};

pub struct AdbStreamer {
    tcp_streamer: TcpStreamer,
}

pub async fn new() -> Result<AdbStreamer, ConnectError> {
    let ip = local_ip().unwrap();

    let tcp_streamer = tcp_streamer_async::new(ip).await?;

    let streamer = AdbStreamer { tcp_streamer };
    Ok(streamer)
}

impl StreamerTrait for AdbStreamer {
    async fn process(&mut self, shared_buf: &mut Producer<u8>) -> Result<usize, WriteError> {
        self.tcp_streamer.process(shared_buf).await
    }
}
