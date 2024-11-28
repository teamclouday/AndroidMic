use std::io;

use adb_streamer::AdbStreamer;
use enum_dispatch::enum_dispatch;
use rtrb::Producer;
use tcp_streamer_async::TcpStreamer;
use thiserror::Error;

// mod tcp_streamer;
// mod udp_streamer;
mod adb_streamer;
mod streamer_sub;
mod tcp_streamer_async;

pub use streamer_sub::{sub, ConnectOption, StreamerCommand, StreamerMsg};

#[derive(Clone, Debug)]
pub enum Status {
    Error(String),
    Listening { port: Option<u16> },
    Connected,
}

impl Status {
    fn is_error(&self) -> bool {
        matches!(self, Status::Error(..))
    }
}

// first we read, next we send
const DEVICE_CHECK_EXPECTED: &str = "AndroidMicCheck";
const DEVICE_CHECK: &str = "AndroidMicCheckAck";

const DEFAULT_PORT: u16 = 55555;
const MAX_PORT: u16 = 60000;
const IO_BUF_SIZE: usize = 1024;

#[enum_dispatch]
trait StreamerTrait {
    async fn next(&mut self) -> Result<Option<Status>, ConnectError>;

    fn set_buff(&mut self, buff: Producer<u8>);

    fn status(&self) -> Option<Status>;
}

#[enum_dispatch(StreamerTrait)]
enum Streamer {
    TcpStreamer,
    AdbStreamer,
    Dummy,
}

#[derive(Debug, Error)]
enum ConnectError {
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
    #[error("command failed: {0}")]
    CommandFailed(io::Error),
    #[error("command failed: {code:?}:{stderr}")]
    StatusCommand { code: Option<i32>, stderr: String },

    #[error(transparent)]
    WriteError(#[from] WriteError),
}

#[derive(Debug, Error)]
enum WriteError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("BufferOverfilled")]
    BufferOverfilled(usize, usize), // moved, lossed
}

#[derive(Default)]
struct Dummy;

impl Dummy {
    fn new_streamer() -> Streamer {
        Streamer::Dummy(Dummy::default())
    }
}

impl StreamerTrait for Dummy {
    async fn next(&mut self) -> Result<Option<Status>, ConnectError> {
        std::future::pending::<()>().await;
        unreachable!()
    }

    fn set_buff(&mut self, buff: Producer<u8>) {}

    fn status(&self) -> Option<Status> {
        None
    }
}

#[derive(Default)]
struct StreamerContainer(Option<Streamer>);

impl StreamerContainer {
    fn take(&mut self) {
        self.0.take();
    }

    fn replace(&mut self, value: Streamer) -> Option<Streamer> {
        self.0.replace(value)
    }
}

impl StreamerTrait for StreamerContainer {
    async fn next(&mut self) -> Result<Option<Status>, ConnectError> {
        std::future::pending::<()>().await;
        unreachable!()
    }

    fn set_buff(&mut self, _buff: Producer<u8>) {}

    fn status(&self) -> Option<Status> {
        None
    }
}

impl From<Streamer> for StreamerContainer {
    fn from(value: Streamer) -> Self {
        Self(Some(value))
    }
}
