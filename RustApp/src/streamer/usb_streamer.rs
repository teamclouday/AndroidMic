use std::time::Duration;

use futures::StreamExt;
use prost::Message;
use tokio::time::sleep;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use super::{
    AudioStream,
    usb::{
        aoa::{AccessoryConfigurations, AccessoryDeviceInfo, AccessoryInterface, AccessoryStrings},
        frame::UsbStream,
    },
};
use crate::streamer::WriteError;

use super::{AudioPacketMessage, ConnectError, StreamerMsg, StreamerTrait};

const MAX_WAIT_TIME: Duration = Duration::from_millis(100);

pub struct UsbStreamer {
    stream_config: AudioStream,
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
        .map_err(|e| ConnectError::CantLoadUsbConfig(e.into()))?;

    // claim the interface
    let mut iface = device
        .detach_and_claim_interface(0)
        .map_err(|e| {
            if e.to_string().contains("dg_ssudbus") {
                ConnectError::CantClaimUsbInterface(nusb::Error::other(
                    "Samsung device is using Samsung driver. Please use Zadig to replace with WinUSB driver."
                ))
            } else {
                ConnectError::CantClaimUsbInterface(e)
            }
        })?;

    let strings = AccessoryStrings::new(
        "AndroidMic",
        if cfg!(debug_assertions) {
            "AndroidMicUSBStreamer-Dev"
        } else {
            "AndroidMicUSBStreamer"
        },
        "Accessory device for AndroidMic app",
        "1.0",
        "https://github.com/teamclouday/AndroidMic",
        "34335e34-bccf-11eb-8529-0242ac130003",
    )
    .map_err(|_| {
        ConnectError::CantLoadUsbConfig(nusb::Error::other("Invalid accessory settings"))
    })?;

    let protocol = iface
        .start_accessory(&strings, Duration::from_secs(1))
        .map_err(ConnectError::CantOpenUsbAccessory)?;

    info!(
        "USB device 0x{:X} switched to accessory mode with protocol 0x{:X}",
        info.device_address(),
        protocol
    );

    drop(device); // disconnect the device
    Ok(())
}

pub async fn new(stream_config: AudioStream) -> Result<UsbStreamer, ConnectError> {
    let mut retries = 0;
    // switch all usb devices to accessory mode until one succeeds
    for info in nusb::list_devices().map_err(ConnectError::NoUsbDevice)? {
        if let Err(error) = switch_to_accessory(&info) {
            warn!(
                "Cannot switch USB device 0x{:X} to accessory mode: {}",
                info.device_address(),
                error
            )
        } else {
            retries = 5;
            break;
        }
    }

    let (info, iface, endpoints) = {
        let info = loop {
            match nusb::list_devices()
                .map_err(ConnectError::NoUsbDevice)?
                .find(|d| d.in_accessory_mode())
            {
                Some(device) => break device,
                None => {
                    if retries == 0 {
                        return Err(ConnectError::NoUsbDevice(nusb::Error::other(
                            "No android phone found after switching to accessory. Check logs for details.",
                        )));
                    }
                    retries -= 1;
                    info!(
                        "Waiting for device to appear in accessory mode... ({} retries left)",
                        retries
                    );
                    sleep(Duration::from_secs(1)).await;
                }
            }
        };

        let device = info.open().map_err(ConnectError::CantOpenUsbHandle)?;
        let configs = device
            .active_configuration()
            .map_err(|e| ConnectError::CantLoadUsbConfig(e.into()))?;

        let iface = device
            .detach_and_claim_interface(0)
            .map_err(ConnectError::CantClaimUsbInterface)?;

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
        stream_config,
        state: UsbStreamerState::Listening,
    };

    Ok(streamer)
}

impl StreamerTrait for UsbStreamer {
    fn reconfigure_stream(&mut self, stream_config: AudioStream) {
        self.stream_config = stream_config;
    }

    fn status(&self) -> StreamerMsg {
        match &self.state {
            UsbStreamerState::Listening => StreamerMsg::Listening {
                ip: None,
                port: None,
            },
            UsbStreamerState::Streaming => StreamerMsg::Connected {
                ip: None,
                port: None,
            },
        }
    }

    async fn next(&mut self) -> Result<Option<StreamerMsg>, ConnectError> {
        match self.framed.next().await {
            Some(Ok(frame)) => {
                self.state = UsbStreamerState::Streaming;

                let mut res = None;
                match AudioPacketMessage::decode(frame) {
                    Ok(packet) => {
                        let buffer_size = packet.buffer.len();

                        if let Ok(buffer) = self.stream_config.process_audio_packet(packet) {
                            // compute the audio wave from the buffer
                            res = Some(StreamerMsg::UpdateAudioWave {
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
