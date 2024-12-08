use anyhow::Result;
use futures::{executor::block_on, StreamExt};
use nusb::{descriptors::Configuration, transfer::RequestBuffer, Interface};
use prost::Message;
use rtrb::Producer;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
    thread::sleep,
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    time::timeout,
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{
    config::AudioFormat,
    streamer::{AudioWaveData, WriteError},
    usb::{AccessoryConfigurations, AccessoryInterface, AccessoryStrings, Endpoint},
};

use super::{AudioPacketMessage, ConnectError, Status, StreamerTrait};

const MAX_WAIT_TIME: Duration = Duration::from_millis(100);

pub struct UsbStreamer {
    producer: Producer<u8>,
    framed: Framed<UsbStream, LengthDelimitedCodec>,
}

const VENDOR_NAME_LIST: [&'static str; 1] = ["samsung"];

pub async fn new(producer: Producer<u8>) -> Result<UsbStreamer, ConnectError> {
    let mut info_list = nusb::list_devices().map_err(ConnectError::NoUsbDevice)?;

    // find the first android phone device
    let info = info_list
        .find(|d| {
            let manufacturer = d.manufacturer_string().unwrap_or("unknown");
            let product = d.product_string().unwrap_or("unknown");

            info!(
                "Checking USB device at 0x{:X}: {}, {}, 0x{:X} 0x{:X}",
                d.device_address(),
                manufacturer,
                product,
                d.vendor_id(),
                d.product_id()
            );

            for vendor_name in VENDOR_NAME_LIST {
                if manufacturer.to_lowercase().contains(vendor_name) {
                    return true;
                }
            }

            false
        })
        .ok_or(nusb::Error::other("No android phone found"))
        .map_err(ConnectError::NoUsbDevice)?;

    // open the device
    info!("Opening USB device 0x{:X}", info.device_address());
    let device = info.open().map_err(ConnectError::CantOpenUsbHandle)?;
    let configs = device
        .active_configuration()
        .map_err(|e| ConnectError::CantOpenUsbHandle(e.into()))?;

    // switch to accessory mode
    info!(
        "USB device 0x{:X} switching to accessory mode",
        info.device_address()
    );

    sleep(Duration::from_secs(1));

    // claim the interface
    let mut iface = device
        .detach_and_claim_interface(0)
        .map_err(ConnectError::CantOpenUsbHandle)?;

    let strings = AccessoryStrings::new(
        "AndroidMic",
        "Android Mic USB Streamer",
        "Accessory device for AndroidMic app",
        "1.0",
        "https://github.com/teamclouday/AndroidMic",
        "34335e34-bccf-11eb-8529-0242ac130003",
    )
    .map_err(|_| {
        ConnectError::CantOpenUsbHandle(nusb::Error::other("Invalid accessory settings"))
    })?;

    let protocol = iface
        .start_accessory(&strings, Duration::from_secs(1))
        .map_err(ConnectError::CantOpenUsbAccessory)?;

    sleep(Duration::from_secs(1));

    let endpoints = configs
        .find_endpoints()
        .map_err(ConnectError::CantOpenUsbAccessoryEndpoint)?;

    let read_endpoint = endpoints.endpoint_in();
    let write_endpoint = endpoints.endpoint_out();

    info!(
        "USB device 0x{:X} opened, read endpoint: 0x{:X}, write endpoint: 0x{:X}, protocol: 0x{:X}",
        info.device_address(),
        read_endpoint.address,
        write_endpoint.address,
        protocol
    );

    Ok(UsbStreamer {
        producer,
        framed: Framed::new(
            UsbStream::new(iface, read_endpoint, write_endpoint),
            LengthDelimitedCodec::new(),
        ),
    })
}

impl StreamerTrait for UsbStreamer {
    fn set_buff(&mut self, producer: Producer<u8>) {
        self.producer = producer;
    }

    fn status(&self) -> Option<Status> {
        Some(Status::Listening { port: None })
    }

    async fn next(&mut self) -> Result<Option<Status>, ConnectError> {
        if let Some(Ok(frame)) = self.framed.next().await {
            match AudioPacketMessage::decode(frame) {
                Ok(packet) => {
                    let buffer_size = packet.buffer.len();
                    let chunk_size = std::cmp::min(buffer_size, self.producer.slots());

                    // mapping from android AudioFormat to encoding size
                    let audio_format =
                        AudioFormat::from_android_format(packet.audio_format).unwrap();
                    let encoding_size = audio_format.sample_size() * packet.channel_count as usize;

                    // make sure chunk_size is a multiple of encoding_size
                    let correction = chunk_size % encoding_size;

                    match self.producer.write_chunk_uninit(chunk_size - correction) {
                        Ok(chunk) => {
                            // compute the audio wave from the buffer
                            let response = if let Some(audio_wave_data) = packet.to_f32_vec() {
                                Some(Status::UpdateAudioWave {
                                    data: audio_wave_data,
                                })
                            } else {
                                None
                            };

                            chunk.fill_from_iter(packet.buffer.into_iter());
                            info!(
                                "received {} bytes, corrected {} bytes, lost {} bytes",
                                buffer_size,
                                correction,
                                buffer_size - chunk_size + correction
                            );

                            Ok(response)
                        }
                        Err(e) => {
                            warn!("dropped packet: {}", e);
                            Ok(None)
                        }
                    }
                }
                Err(e) => {
                    return Err(WriteError::Deserializer(e).into());
                }
            }
        } else {
            // info!("frame not ready");
            // sleep for a while to not consume all CPU
            tokio::time::sleep(MAX_WAIT_TIME).await;
            Ok(None)
        }
    }
}

pub struct UsbStream {
    iface: Interface,
    read_endpoint: Endpoint,
    write_endpoint: Endpoint,
}

impl UsbStream {
    pub fn new(iface: Interface, read_endpoint: Endpoint, write_endpoint: Endpoint) -> Self {
        Self {
            iface,
            read_endpoint,
            write_endpoint,
        }
    }
}

impl AsyncRead for UsbStream {
    fn poll_read(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match block_on(
            timeout(
                Duration::from_secs(1),
                self.iface.bulk_in(
                    self.read_endpoint.address,
                    RequestBuffer::new(buf.capacity()),
                ),
            )
            .into_inner(),
        )
        .into_result()
        {
            Ok(bytes_read) => {
                buf.put_slice(bytes_read.as_slice());
                Poll::Ready(Ok(()))
            }
            Err(e) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
        }
    }
}

impl AsyncWrite for UsbStream {
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match block_on(
            timeout(
                Duration::from_secs(1),
                self.iface
                    .bulk_out(self.write_endpoint.address, Vec::from(buf)),
            )
            .into_inner(),
        )
        .into_result()
        {
            Ok(bytes_written) => std::task::Poll::Ready(Ok(bytes_written.actual_length())),
            Err(e) => {
                std::task::Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e)))
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
