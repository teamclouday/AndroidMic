use adb_streamer::AdbStreamer;
use anyhow::Result;
use enum_dispatch::enum_dispatch;
use prost::DecodeError;
use rtrb::{Producer, chunks::ChunkError};
use std::io;
use tcp_streamer::TcpStreamer;
use thiserror::Error;
use udp_streamer::UdpStreamer;
use usb_streamer::UsbStreamer;

mod adb_streamer;
mod message;
mod streamer_runner;
mod tcp_streamer;
mod udp_streamer;
mod usb;
mod usb_streamer;

pub use message::{AudioPacketMessage, AudioPacketMessageOrdered};
pub use streamer_runner::{ConnectOption, StreamerCommand, StreamerMsg, sub};

use crate::{
    audio::{AudioPacketFormat, process::AudioProcessParams},
    config::AudioFormat,
};
use usb::aoa::{AccessoryError, EndpointError};

/// Status reported from the streamer
#[derive(Clone, Debug)]
pub enum Status {
    UpdateAudioWave { data: Vec<(f32, f32)> },
    Error(String),
    Listening { port: Option<u16> },
    Connected,
}

impl Status {
    fn is_error(&self) -> bool {
        matches!(self, Status::Error(..))
    }
}

/// Default port on the PC
const DEFAULT_PC_PORT: u16 = 55555;
const MAX_PORT: u16 = 60000;

#[derive(Debug)]
pub struct StreamConfig {
    pub buff: Producer<u8>,
    pub audio_config: AudioPacketFormat,
    pub denoise: bool,
}

impl StreamConfig {
    pub fn to_audio_params(&self) -> AudioProcessParams {
        AudioProcessParams {
            target_format: self.audio_config.clone(),
            denoise: self.denoise,
        }
    }
}

#[enum_dispatch]
trait StreamerTrait {
    /// I know it seems weird to have a next method like that, but it is actually the easiest way i found
    /// to handle the multiple async functions of streamers (process, accept) while still receiving command from the app.
    /// This method make them behave like a state machine, always reaching the next state. (init -> accepted -> read data -> read data ...).
    ///
    /// A nice benefit of this pattern is that there is no usage of Atomic what so ever.
    async fn next(&mut self) -> Result<Option<Status>, ConnectError>;

    fn reconfigure_stream(&mut self, stream_config: StreamConfig);

    fn status(&self) -> Option<Status>;
}
#[allow(clippy::enum_variant_names)]
#[enum_dispatch(StreamerTrait)]
enum Streamer {
    TcpStreamer,
    AdbStreamer,
    UdpStreamer,
    UsbStreamer,
    DummyStreamer,
}

#[derive(Debug, Error)]
enum ConnectError {
    #[error("can't bind a port on the pc: {0}")]
    CantBindPort(io::Error),
    #[error("can't find a local address: {0}")]
    NoLocalAddress(io::Error),
    #[error("accept failed: {0}")]
    CantAccept(io::Error),
    #[error(transparent)]
    WriteError(#[from] WriteError),
    #[error("no usb device found: {0}")]
    NoUsbDevice(nusb::Error),
    #[error("no adb device found")]
    NoAdbDevice,
    #[error("can't open usb handle: {0}")]
    CantOpenUsbHandle(nusb::Error),
    #[error("can't load usb device config: {0}")]
    CantLoadUsbConfig(nusb::Error),
    #[error("can't claim usb device interface: {0}")]
    CantClaimUsbInterface(nusb::Error),
    #[error("can't open usb accessory: {0}")]
    CantOpenUsbAccessory(AccessoryError),
    #[error("can't open usb accessory endpoint: {0}")]
    CantOpenUsbAccessoryEndpoint(EndpointError),
    #[error("device disconnected")]
    Disconnected,
    #[error(transparent)]
    CantJoin(#[from] tokio::task::JoinError),
    #[error("command failed: {code:?}:{stderr}")]
    AdbStatusCommand { code: Option<i32>, stderr: String },
    #[error("command failed: {0} make sure adb is installed and in your PATH")]
    CommandFailed(io::Error),
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
    Chunk(#[from] ChunkError),
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

    fn reconfigure_stream(&mut self, _config: StreamConfig) {}

    fn status(&self) -> Option<Status> {
        None
    }
}

impl AudioPacketMessage {
    fn to_wave_data(buffer: &[f32]) -> Vec<(f32, f32)> {
        let window_size = 50;

        buffer
            .chunks_exact(window_size)
            .map(|window| {
                let max = window.iter().fold(f32::MIN, |acc, &x| acc.max(x));
                let min = window.iter().fold(f32::MAX, |acc, &x| acc.min(x));
                (max, min)
            })
            .collect()
    }

    fn sub_packets(&self, samples: usize) -> Vec<Self> {
        let mut packets = Vec::new();
        let channel_count = self.channel_count as usize;
        let audio_format = AudioFormat::from_android_format(self.audio_format).unwrap();

        // calculate the size of each packet
        let packet_size = audio_format.sample_size() * channel_count * samples;

        // split the buffer into packets of the specified size
        for chunk in self.buffer.chunks(packet_size) {
            let mut packet = self.clone();
            packet.buffer = chunk.to_vec();
            packets.push(packet);
        }

        packets
    }
}
