use std::time::Duration;

use futures::StreamExt;
use prost::Message;
use rtrb::Producer;
use tokio::time::sleep;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use super::usb::{
    aoa::{AccessoryConfigurations, AccessoryDeviceInfo, AccessoryInterface, AccessoryStrings},
    frame::UsbStream,
};
use crate::{
    audio::{process::convert_audio_stream, AudioPacketFormat},
    streamer::WriteError,
};

use super::{AudioPacketMessage, ConnectError, Status, StreamerTrait};

const MAX_WAIT_TIME: Duration = Duration::from_millis(100);

pub struct UsbStreamer {
    producer: Producer<u8>,
    format: AudioPacketFormat,
    state: UsbStreamerState,
    framed: Framed<UsbStream, LengthDelimitedCodec>,
}

pub enum UsbStreamerState {
    Listening,
    Streaming,
}

// switch a USB device to accessory mode
pub fn switch_to_accessory(info: &nusb::DeviceInfo) -> Result<(), ConnectError> {
    info!(
        "Checking USB device at 0x{:X}: {}, {}, 0x{:X} 0x{:X}",
        info.device_address(),
        info.manufacturer_string().unwrap_or("unknown"),
        info.product_string().unwrap_or("unknown"),
        info.vendor_id(),
        info.product_id()
    );

    let device = info.open().map_err(ConnectError::CantOpenUsbHandle)?;
    let _configs = device
        .active_configuration()
        .map_err(|e| ConnectError::CantOpenUsbHandle(e.into()))?;

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

    info!(
        "USB device 0x{:X} switched to accessory mode with protocol 0x{:X}",
        info.device_address(),
        protocol
    );

    // close device
    drop(device);

    Ok(())
}

pub async fn new(
    producer: Producer<u8>,
    format: AudioPacketFormat,
) -> Result<UsbStreamer, ConnectError> {
    // switch all usb devices to accessory mode and ignore errors
    nusb::list_devices()
        .map_err(ConnectError::NoUsbDevice)?
        .for_each(|info| {
            switch_to_accessory(&info).unwrap_or_default();
        });

    // wait for the app to open and connect
    sleep(Duration::from_secs(1)).await;

    let (info, iface, endpoints) = {
        let info = nusb::list_devices()
            .map_err(ConnectError::NoUsbDevice)?
            .find(|d| d.in_accessory_mode())
            .ok_or(nusb::Error::other(
                "No android phone found after switching to accessory. Make sure the phone is set to charging only mode.",
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

        (info, iface, endpoints)
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
        format,
        state: UsbStreamerState::Listening,
    };

    Ok(streamer)
}

impl StreamerTrait for UsbStreamer {
    fn set_buff(&mut self, producer: Producer<u8>, format: AudioPacketFormat) {
        self.producer = producer;
        self.format = format;
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

                        if let Ok(buffer) =
                            convert_audio_stream(&mut self.producer, packet, &self.format)
                        {
                            // compute the audio wave from the buffer
                            res = Some(Status::UpdateAudioWave {
                                data: AudioPacketMessage::to_wave_data(&buffer),
                            });

                            debug!("received {} bytes", buffer_size);
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
