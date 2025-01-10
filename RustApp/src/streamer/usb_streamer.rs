use std::time::Duration;

use futures::StreamExt;
use prost::Message;
use rtrb::Producer;
use tokio::time::sleep;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{
    config::AudioFormat,
    streamer::{AudioWaveData, WriteError},
    usb::{
        aoa::{AccessoryConfigurations, AccessoryDeviceInfo, AccessoryInterface, AccessoryStrings},
        frame::UsbStream,
    },
};

use super::{AudioPacketMessage, ConnectError, Status, StreamerTrait};

const MAX_WAIT_TIME: Duration = Duration::from_millis(100);

const VENDOR_NAME_LIST: [&str; 2] = ["samsung", "oneplus"];

pub struct UsbStreamer {
    producer: Producer<u8>,
    state: UsbStreamerState,
    framed: Framed<UsbStream, LengthDelimitedCodec>,
}

pub enum UsbStreamerState {
    Listening,
    Streaming,
}

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
                if manufacturer.to_lowercase().contains(vendor_name)
                    || product.to_lowercase().contains(vendor_name)
                {
                    return true;
                }
            }

            false
        })
        .ok_or(nusb::Error::other("No android phone found"))
        .map_err(ConnectError::NoUsbDevice)?;

    let (iface, endpoints) = if info.in_accessory_mode() {
        info!("Opening USB device 0x{:X}", info.device_address());
        let device = info.open().map_err(ConnectError::CantOpenUsbHandle)?;

        let configs = device
            .active_configuration()
            .map_err(|e| ConnectError::CantOpenUsbHandle(e.into()))?;

        let iface = device
            .detach_and_claim_interface(0)
            .map_err(ConnectError::CantOpenUsbHandle)?;

        // find endpoints
        let endpoints = configs
            .find_endpoints()
            .map_err(ConnectError::CantOpenUsbAccessoryEndpoint)?;

        (iface, endpoints)
    } else {
        let device = info.open().map_err(ConnectError::CantOpenUsbHandle)?;
        let configs = device
            .active_configuration()
            .map_err(|e| ConnectError::CantOpenUsbHandle(e.into()))?;

        // switch to accessory mode
        info!(
            "USB device 0x{:X} switching to accessory mode",
            info.device_address()
        );

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

        sleep(Duration::from_secs(1)).await;

        info!(
            "USB device 0x{:X} switched to accessory mode with protocol 0x{:X}",
            info.device_address(),
            protocol
        );

        // close device and reopen
        drop(iface);
        drop(configs);

        let info = nusb::list_devices()
            .map_err(ConnectError::NoUsbDevice)?
            .inspect(|d| {
                let manufacturer = d.manufacturer_string().unwrap_or("unknown");
                let product = d.product_string().unwrap_or("unknown");

                info!(
                    "Checking2 USB device at 0x{:X}: {}, {}, 0x{:X} 0x{:X}",
                    d.device_address(),
                    manufacturer,
                    product,
                    d.vendor_id(),
                    d.product_id()
                );
            })
            .find(|d| d.in_accessory_mode())
            .ok_or(nusb::Error::other(
                "No android phone found after switching to accessory",
            ))
            .map_err(ConnectError::NoUsbDevice)?;

        let device = info.open().map_err(ConnectError::CantOpenUsbHandle)?;
        let configs = device
            .active_configuration()
            .map_err(|e| ConnectError::CantOpenUsbHandle(e.into()))?;

        let iface = device
            .detach_and_claim_interface(0)
            .map_err(ConnectError::CantOpenUsbHandle)?;

        // find endpoints
        let endpoints = configs
            .find_endpoints()
            .map_err(ConnectError::CantOpenUsbAccessoryEndpoint)?;

        (iface, endpoints)
    };

    let read_endpoint = endpoints.endpoint_in();
    let write_endpoint = endpoints.endpoint_out();

    info!(
        "USB device 0x{:X} opened, read endpoint: 0x{:X}, write endpoint: 0x{:X}",
        info.device_address(),
        read_endpoint.address,
        write_endpoint.address
    );

    let read_queue = iface.bulk_in_queue(read_endpoint.address);
    let write_queue = iface.bulk_out_queue(write_endpoint.address);

    let framed = Framed::new(
        UsbStream::new(read_queue, write_queue),
        LengthDelimitedCodec::new(),
    );

    let streamer = UsbStreamer {
        framed,
        producer,
        state: UsbStreamerState::Listening,
    };

    Ok(streamer)
}

impl StreamerTrait for UsbStreamer {
    fn set_buff(&mut self, producer: Producer<u8>) {
        self.producer = producer;
    }

    fn status(&self) -> Option<Status> {
        match &self.state {
            UsbStreamerState::Listening => Some(Status::Listening { port: None }),
            UsbStreamerState::Streaming => Some(Status::Connected),
        }
    }

    async fn next(&mut self) -> Result<Option<Status>, ConnectError> {
        match self.framed.next().await {
            Some(Ok(frame)) => {
                self.state = UsbStreamerState::Streaming;

                let mut res = None;
                match AudioPacketMessage::decode(frame) {
                    Ok(packet) => {
                        let buffer_size = packet.buffer.len();
                        let chunk_size = std::cmp::min(buffer_size, self.producer.slots());

                        // mapping from android AudioFormat to encoding size
                        let audio_format =
                            AudioFormat::from_android_format(packet.audio_format).unwrap();
                        let encoding_size =
                            audio_format.sample_size() * packet.channel_count as usize;

                        // make sure chunk_size is a multiple of encoding_size
                        let correction = chunk_size % encoding_size;

                        match self.producer.write_chunk_uninit(chunk_size - correction) {
                            Ok(chunk) => {
                                // compute the audio wave from the buffer
                                if let Some(audio_wave_data) = packet.to_f32_vec() {
                                    res = Some(Status::UpdateAudioWave {
                                        data: audio_wave_data,
                                    });
                                }

                                chunk.fill_from_iter(packet.buffer.into_iter());
                                debug!(
                                    "received {} bytes, corrected {} bytes, lost {} bytes",
                                    buffer_size,
                                    correction,
                                    buffer_size - chunk_size + correction
                                );
                            }
                            Err(e) => {
                                warn!("dropped packet: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        return Err(ConnectError::WriteError(WriteError::Deserializer(e)));
                    }
                }

                Ok(res)
            }
            Some(Err(e)) => {
                panic!("{}", e);
            }
            None => todo!(),
        }
    }
}
