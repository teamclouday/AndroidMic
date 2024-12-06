use futures::StreamExt;
use prost::Message;
use rtrb::Producer;
use rusb::{DeviceHandle, GlobalContext};
use std::{
    io,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::Duration,
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{
    config::AudioFormat,
    streamer::{AudioWaveData, WriteError},
    usb::{AccessoryDevice, AccessoryHandle, AccessoryStrings},
};

use super::{AudioPacketMessage, ConnectError, Status, StreamerTrait};

const MAX_WAIT_TIME: Duration = Duration::from_millis(100);

pub struct UsbStreamer {
    producer: Producer<u8>,
    framed: Framed<UsbStream, LengthDelimitedCodec>,
}

const VENDOR_NAME_LIST: [&'static str; 1] = ["samsung"];

fn open_usb_accessory(handle: &DeviceHandle<GlobalContext>) -> Result<(), rusb::Error> {
    let request_type = (rusb::RequestType::Vendor as u8) | (rusb::Direction::Out as u8);
    let data: Vec<u8> = vec![];
    handle.write_control(request_type, 51, 0, 0, &data, Duration::from_secs(1))?;

    Ok(())
}

pub async fn new(producer: Producer<u8>) -> Result<UsbStreamer, ConnectError> {
    let devices = rusb::devices().map_err(ConnectError::NoUsbDevice)?;

    // find the first android phone device
    let device = devices
        .iter()
        .find(|d| {
            if let Ok(desc) = d.device_descriptor() {
                if let Some(product) =
                    usb_ids::Device::from_vid_pid(desc.vendor_id(), desc.product_id())
                {
                    info!(
                        "Checking USB device at address 0x{:X}: {} from {}, 0x{:X} 0x{:X}",
                        d.address(),
                        product.name(),
                        product.vendor().name(),
                        desc.vendor_id(),
                        desc.product_id()
                    );

                    for vendor_name in VENDOR_NAME_LIST {
                        if product.vendor().name().to_lowercase().contains(vendor_name) {
                            return true;
                        }
                    }
                }
            }

            false
        })
        .ok_or(rusb::Error::NotFound)
        .map_err(ConnectError::NoUsbDevice)?;

    let mut handle = device.open().map_err(ConnectError::CantOpenUsbHandle)?;

    // open the device
    if device
        .in_accessory_mode()
        .map_err(ConnectError::CantOpenUsbAccessory)?
    {
        info!("USB device 0x{:X} is in accessory mode", device.address());

        let protocol = handle
            .get_protocol(Duration::from_secs(1))
            .map_err(ConnectError::CantOpenUsbAccessory)?;

        let endpoints = device
            .find_endpoints()
            .map_err(ConnectError::CantOpenUsbAccessoryEndpoint)?;

        let read_endpoint = endpoints.endpoint_in();
        let write_endpoint = endpoints.endpoint_out();

        info!(
            "USB device {:X} opened, read endpoint: {:X}, write endpoint: {:X}, protocol {:X}",
            device.address(),
            read_endpoint,
            write_endpoint,
            protocol
        );

        Ok(UsbStreamer {
            producer,
            framed: Framed::new(
                UsbStream::new(handle, read_endpoint, write_endpoint),
                LengthDelimitedCodec::new(),
            ),
        })
    } else {
        info!(
            "USB device 0x{:X} is not in accessory mode, trying to switch",
            device.address()
        );

        let strings = AccessoryStrings::new(
            "AndroidMic",
            "Android Mic USB Streamer",
            "Accessory device for AndroidMic app",
            "1.0",
            "https://github.com/teamclouday/AndroidMic",
            "34335e34-bccf-11eb-8529-0242ac130003",
        )
        .map_err(|_| ConnectError::CantOpenUsbHandle(rusb::Error::InvalidParam))?;

        handle
            .claim_interface(0)
            .map_err(ConnectError::CantOpenUsbHandle)?;

        let protocol = handle
            .start_accessory(&strings, Duration::from_secs(1))
            .map_err(ConnectError::CantOpenUsbAccessory)?;

        let endpoints = device
            .find_endpoints()
            .map_err(ConnectError::CantOpenUsbAccessoryEndpoint)?;

        let read_endpoint = endpoints.endpoint_in();
        let write_endpoint = endpoints.endpoint_out();

        info!(
            "USB device {:X} opened, read endpoint: {:X}, write endpoint: {:X}, protocol {:X}",
            device.address(),
            read_endpoint,
            write_endpoint,
            protocol
        );

        Ok(UsbStreamer {
            producer,
            framed: Framed::new(
                UsbStream::new(handle, read_endpoint, write_endpoint),
                LengthDelimitedCodec::new(),
            ),
        })
    }
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
            info!("frame not ready");
            // sleep for a while to not consume all CPU
            tokio::time::sleep(MAX_WAIT_TIME).await;
            Ok(None)
        }
    }
}

pub struct UsbStream {
    handle: Arc<Mutex<DeviceHandle<GlobalContext>>>,
    read_endpoint: u8,
    write_endpoint: u8,
}

impl UsbStream {
    pub fn new(handle: DeviceHandle<GlobalContext>, read_endpoint: u8, write_endpoint: u8) -> Self {
        Self {
            handle: Arc::new(Mutex::new(handle)),
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
        let handle = self.handle.lock().unwrap();
        let mut temp_buf = vec![0; buf.capacity()];

        match handle.read_bulk(
            self.read_endpoint,
            &mut temp_buf,
            std::time::Duration::from_secs(1),
        ) {
            Ok(bytes_read) => {
                buf.put_slice(&temp_buf[..bytes_read]);
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
        let handle = self.handle.lock().unwrap();

        match handle.write_bulk(self.write_endpoint, buf, std::time::Duration::from_secs(1)) {
            Ok(bytes_written) => std::task::Poll::Ready(Ok(bytes_written)),
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
