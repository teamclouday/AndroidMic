use std::io;

use enum_dispatch::enum_dispatch;
use rtrb::Producer;
use thiserror::Error;

use super::{adb_streamer::AdbStreamer, tcp_streamer_async::TcpStreamer};

// first we read, next we send
pub const DEVICE_CHECK_EXPECTED: &str = "AndroidMicCheck";
pub const DEVICE_CHECK: &str = "AndroidMicCheckAck";

pub const DEFAULT_PORT: u16 = 55555;
pub const MAX_PORT: u16 = 60000;
pub const IO_BUF_SIZE: usize = 1024;

#[enum_dispatch]
pub trait StreamerTrait {
    /// return the number of item moved or an error
    async fn process(&mut self, shared_buf: &mut Producer<u8>) -> Result<usize, WriteError>;

    fn port(&self) -> Option<u16> {
        None
    }
}

#[enum_dispatch(StreamerTrait)]
pub enum Streamer {
    TcpStreamer,
    AdbStreamer,
}

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("can't bind a port on the pc: {0}")]
    CantBindPort(io::Error),
    #[error("can't find a local address: {0}")]
    NoLocalAddress(io::Error),
    #[error("read check fail: expected = {expected}, received = {received}")]
    CheckFailed {
        expected: &'static str,
        received: String,
    },
    #[error("check fail: {0}")]
    CheckFailedIo(io::Error),
    #[error("accept failed: {0}")]
    CantAccept(io::Error),
}

pub enum WriteError {
    Io(io::Error),
    BufferOverfilled(usize, usize), // moved, lossed
}
