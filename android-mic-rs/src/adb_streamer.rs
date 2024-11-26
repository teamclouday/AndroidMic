use std::net::IpAddr;

use rtrb::Producer;

use crate::streamer::{self, StreamerTrait, WriteError};

pub struct AdbStreamer {}

pub fn new(ip: IpAddr) -> Result<AdbStreamer, streamer::Error> {
    Ok(AdbStreamer {})
}

impl StreamerTrait for AdbStreamer {
    async fn process(&mut self, shared_buf: &mut Producer<u8>) -> Result<usize, WriteError> {
        todo!()
    }
}
