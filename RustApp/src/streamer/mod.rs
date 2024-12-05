use std::io;

use adb_streamer::AdbStreamer;
use enum_dispatch::enum_dispatch;
use prost::DecodeError;
use rtrb::{chunks::ChunkError, Producer};
use tcp_streamer::TcpStreamer;
use thiserror::Error;

// mod udp_streamer;
mod adb_streamer;
mod streamer_sub;
mod tcp_streamer;
mod message;

pub use streamer_sub::{sub, ConnectOption, StreamerCommand, StreamerMsg};
pub use message::AudioPacketMessage;

/// Status reported from the streamer
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

/// Default port on the PC
const DEFAULT_PC_PORT: u16 = 55555;
const MAX_PORT: u16 = 60000;
const IO_BUF_SIZE: usize = 1024;

#[enum_dispatch]
trait StreamerTrait {
    /// I know it seems weird to have a next method like that, but it is actually the easiest way i found
    /// to handle the multiple async functions of streamers (process, accept) while still receiving command from the app.
    /// This method make them behave like a state machine, always reaching the next state. (init -> accepted -> read data -> read data ...).
    ///
    /// A nice benefit of this pattern is that there is no usage of Atomic what so ever.
    async fn next(&mut self) -> Result<Option<Status>, ConnectError>;

    fn set_buff(&mut self, buff: Producer<u8>);

    fn status(&self) -> Option<Status>;
}
#[allow(clippy::enum_variant_names)]
#[enum_dispatch(StreamerTrait)]
enum Streamer {
    TcpStreamer,
    AdbStreamer,
    DummyStreamer,
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
    #[error(transparent)]
    Deserializer(#[from] DecodeError),
    #[error(transparent)]
    Chunk(#[from] ChunkError)
}

/// Used to simplified futures logic
#[derive(Default)]
struct DummyStreamer;

impl DummyStreamer {
    #[allow(clippy::new_ret_no_self)]
    fn new() -> Streamer {
        Streamer::DummyStreamer(DummyStreamer)
    }
}

impl StreamerTrait for DummyStreamer {
    async fn next(&mut self) -> Result<Option<Status>, ConnectError> {
        std::future::pending::<()>().await;
        unreachable!()
    }

    fn set_buff(&mut self, _buff: Producer<u8>) {}

    fn status(&self) -> Option<Status> {
        None
    }
}