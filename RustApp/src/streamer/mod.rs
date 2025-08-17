use adb_streamer::AdbStreamer;
use anyhow::Result;
use enum_dispatch::enum_dispatch;
use prost::DecodeError;
use rtrb::{Producer, chunks::ChunkError};
use std::{fmt::Debug, io};
use tcp_streamer::TcpStreamer;
use thiserror::Error;
use udp_streamer::UdpStreamer;

mod adb_streamer;
mod message;
mod streamer_runner;
mod tcp_streamer;
mod udp_streamer;

#[cfg(feature = "usb")]
mod usb;

#[cfg(feature = "usb")]
mod usb_streamer;

#[cfg(feature = "usb")]
use crate::streamer::usb_streamer::UsbStreamer;

pub use message::{AudioPacketMessage, AudioPacketMessageOrdered};
pub use streamer_runner::{ConnectOption, StreamerCommand, StreamerMsg, sub};

use crate::{audio::process::AudioProcessParams, config::AudioFormat};

/// Default port on the PC
const DEFAULT_PC_PORT: u16 = 55555;
const MAX_PORT: u16 = 60000;

pub struct AudioStream {
    pub buff: Producer<u8>,
    pub audio_params: AudioProcessParams,
}

impl Debug for AudioStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioStream")
            .field("audio_params", &self.audio_params)
            .finish()
    }
}

#[enum_dispatch]
trait StreamerTrait {
    /// I know it seems weird to have a next method like that, but it is actually the easiest way i found
    /// to handle the multiple async functions of streamers (process, accept) while still receiving command from the app.
    /// This method make them behave like a state machine, always reaching the next state. (init -> accepted -> read data -> read data ...).
    ///
    /// A nice benefit of this pattern is that there is no usage of Atomic what so ever.
    async fn next(&mut self) -> Result<Option<StreamerMsg>, ConnectError>;

    fn reconfigure_stream(&mut self, stream_config: AudioStream);

    fn status(&self) -> StreamerMsg;
}
#[allow(clippy::enum_variant_names)]
#[enum_dispatch(StreamerTrait)]
enum Streamer {
    TcpStreamer,
    AdbStreamer,
    UdpStreamer,
    #[cfg(feature = "usb")]
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
    #[cfg(feature = "usb")]
    #[error("no usb device found: {0}")]
    NoUsbDevice(nusb::Error),
    #[error("no adb device found")]
    NoAdbDevice,
    #[cfg(feature = "usb")]
    #[error("can't open usb handle: {0}")]
    CantOpenUsbHandle(nusb::Error),
    #[cfg(feature = "usb")]
    #[error("can't load usb device config: {0}")]
    CantLoadUsbConfig(nusb::Error),
    #[cfg(feature = "usb")]
    #[error("can't claim usb device interface: {0}")]
    CantClaimUsbInterface(nusb::Error),

    #[cfg(feature = "usb")]
    #[error("can't open usb accessory: {0}")]
    CantOpenUsbAccessory(usb::aoa::AccessoryError),

    #[cfg(feature = "usb")]
    #[error("can't open usb accessory endpoint: {0}")]
    CantOpenUsbAccessoryEndpoint(usb::aoa::EndpointError),
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
    async fn next(&mut self) -> Result<Option<StreamerMsg>, ConnectError> {
        std::future::pending::<()>().await;
        unreachable!()
    }

    fn reconfigure_stream(&mut self, _config: AudioStream) {}

    fn status(&self) -> StreamerMsg {
        unreachable!()
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
