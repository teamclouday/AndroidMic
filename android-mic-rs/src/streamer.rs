use std::{io, net::IpAddr};

use rtrb::Producer;

pub enum WriteError {
    Io(io::Error),
    BufferOverfilled(usize, usize), // moved, lossed
}

// first we read, next we send
pub const DEVICE_CHECK_EXPECTED: &str = "AndroidMicCheck";
pub const DEVICE_CHECK: &str = "AndroidMicCheckAck";

pub const DEFAULT_PORT: u16 = 55555;
pub const MAX_PORT: u16 = 60000;
pub const IO_BUF_SIZE: usize = 1024;

#[derive(Clone, Debug)]
pub enum Status {
    Default,
    Listening,
    Connected,
}

pub trait Streamer {
    fn new(shared_buf: Producer<u8>, ip: IpAddr) -> Option<Self>
    where
        Self: Sized;

    /// return the number of item moved
    /// or an error
    fn process(&mut self) -> Result<usize, WriteError>;
}
