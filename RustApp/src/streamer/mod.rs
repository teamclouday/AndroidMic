use adb_streamer::AdbStreamer;
use anyhow::Result;
use byteorder::{ByteOrder, NativeEndian};
use enum_dispatch::enum_dispatch;
use prost::DecodeError;
use rtrb::{chunks::ChunkError, Producer};
use std::io;
use tcp_streamer::TcpStreamer;
use thiserror::Error;
use udp_streamer::UdpStreamer;
use usb_streamer::UsbStreamer;

mod adb_streamer;
mod message;
mod streamer_sub;
mod tcp_streamer;
mod udp_streamer;
mod usb_streamer;

pub use message::{AudioPacketMessage, AudioPacketMessageOrdered};
pub use streamer_sub::{sub, ConnectOption, StreamerCommand, StreamerMsg};

use crate::{
    config::AudioFormat,
    usb::aoa::{AccessoryError, EndpointError},
};

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
    #[error("can't open usb handle: {0}, make sure phone is set to charging only mode")]
    CantOpenUsbHandle(nusb::Error),
    #[error("can't open usb accessory: {0}")]
    CantOpenUsbAccessory(AccessoryError),
    #[error("can't open usb accessory endpoint: {0}")]
    CantOpenUsbAccessoryEndpoint(EndpointError),
    #[error("device disconnected")]
    Disconnected,
    #[error(transparent)]
    CantJoin(#[from] tokio::task::JoinError),
    #[error("command failed: {code:?}:{stderr}")]
    StatusCommand { code: Option<i32>, stderr: String },
    #[error("command failed: {0}")]
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

    fn set_buff(&mut self, _buff: Producer<u8>) {}

    fn status(&self) -> Option<Status> {
        None
    }
}

trait AudioWaveData {
    fn to_f32_vec(&self) -> Option<Vec<(f32, f32)>>;
}

impl AudioWaveData for AudioPacketMessage {
    fn to_f32_vec(&self) -> Option<Vec<(f32, f32)>> {
        let channel_count = self.channel_count as usize;
        let audio_format = AudioFormat::from_android_format(self.audio_format).unwrap();

        let iter = self
            .buffer
            .chunks_exact(audio_format.sample_size() * channel_count);
        let mut result =
            Vec::with_capacity(self.buffer.len() / audio_format.sample_size() / channel_count);

        let window_size = 50;

        match audio_format {
            AudioFormat::U8 => {
                for chunk in iter {
                    result.push((chunk[0] as f32 - 128.0) / 128.0);
                }

                // iterate every window samples to find max and min in each window
                Some(
                    result
                        .chunks_exact(window_size)
                        .map(|window| {
                            let max = window.iter().fold(f32::MIN, |acc, &x| acc.max(x));
                            let min = window.iter().fold(f32::MAX, |acc, &x| acc.min(x));
                            (max, min)
                        })
                        .collect(),
                )
            }
            AudioFormat::I16 => {
                for chunk in iter {
                    let sample = NativeEndian::read_i16(chunk);
                    result.push(sample as f32 / i16::MAX as f32);
                }
                Some(
                    result
                        .chunks_exact(window_size)
                        .map(|window| {
                            let max = window.iter().fold(f32::MIN, |acc, &x| acc.max(x));
                            let min = window.iter().fold(f32::MAX, |acc, &x| acc.min(x));
                            (max, min)
                        })
                        .collect(),
                )
            }
            AudioFormat::I24 => {
                for chunk in iter {
                    let sample = NativeEndian::read_i24(chunk);
                    result.push(sample as f32 / (1 << 23) as f32);
                }
                Some(
                    result
                        .chunks_exact(window_size)
                        .map(|window| {
                            let max = window.iter().fold(f32::MIN, |acc, &x| acc.max(x));
                            let min = window.iter().fold(f32::MAX, |acc, &x| acc.min(x));
                            (max, min)
                        })
                        .collect(),
                )
            }
            AudioFormat::I32 => {
                for chunk in iter {
                    let sample = NativeEndian::read_i32(chunk);
                    result.push(sample as f32 / i32::MAX as f32);
                }
                Some(
                    result
                        .chunks_exact(window_size)
                        .map(|window| {
                            let max = window.iter().fold(f32::MIN, |acc, &x| acc.max(x));
                            let min = window.iter().fold(f32::MAX, |acc, &x| acc.min(x));
                            (max, min)
                        })
                        .collect(),
                )
            }
            AudioFormat::F32 => {
                for chunk in iter {
                    let sample = NativeEndian::read_f32(chunk);
                    result.push(sample);
                }
                Some(
                    result
                        .chunks_exact(window_size)
                        .map(|window| {
                            let max = window.iter().fold(f32::MIN, |acc, &x| acc.max(x));
                            let min = window.iter().fold(f32::MAX, |acc, &x| acc.min(x));
                            (max, min)
                        })
                        .collect(),
                )
            }
            _ => None,
        }
    }
}
