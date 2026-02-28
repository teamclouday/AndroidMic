use std::time::Duration;

use futures::StreamExt;
use nusb::io::{EndpointRead, EndpointWrite};
use prost::Message;
use tokio::io::AsyncWriteExt;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};

use super::AudioStream;
use crate::{
    config::ConnectionMode,
    streamer::{
        CHECK_2, WriteError,
        message::{MessageWrapper, message_wrapper::Payload},
        usb::aoa::{
            AccessoryDeviceExt, AccessoryDeviceInfoExt, AccessoryInterfaceExt, AccessoryStrings,
        },
    },
};

use super::{AudioPacketMessage, ConnectError, StreamerMsg, StreamerTrait};

const TRANSFER_BUFFER_SIZE: usize = 1024;

pub struct UsbStreamer {
    stream_config: AudioStream,
    reader: FramedRead<EndpointRead<nusb::transfer::Bulk>, LengthDelimitedCodec>,
    writer: EndpointWrite<nusb::transfer::Bulk>,
    is_listening: bool,
    tracked_sequence: u32,
}

// switch a USB device to accessory mode
async fn switch_to_accessory(device_info: &nusb::DeviceInfo) -> Result<(), ConnectError> {
    info!(
        "Checking USB device {} (address=0x{:X}, vid=0x{:X}, pid=0x{:X})",
        device_info.product_string().unwrap_or("unknown"),
        device_info.device_address(),
        device_info.vendor_id(),
        device_info.product_id()
    );

    // first check if already in accessory mode
    if device_info.in_accessory_mode() {
        info!(
            "USB device 0x{:X} is already in accessory mode",
            device_info.device_address()
        );

        // NOTE: we run the handshake here regardless of whether it is already in AOA mode
        // return Ok(());
    }

    // otherwise open device and send AOA control signal
    let device = device_info
        .open()
        .await
        .map_err(|e| ConnectError::CantOpenUsbHandle(e.into()))?;

    // claim the interface
    let mut iface = device
        .claim_interface(0)
        .await
        .map_err(|e| ConnectError::CantClaimUsbInterface(e.into()))?;

    let strings = AccessoryStrings {
        manufacturer: "AndroidMic".to_string(),
        model: if cfg!(debug_assertions) {
            "AndroidMicUSBStreamer-Dev".to_string()
        } else {
            "AndroidMicUSBStreamer".to_string()
        },
        description: "Accessory device for AndroidMic app".to_string(),
        version: "1.0".to_string(),
        uri: "https://github.com/teamclouday/AndroidMic".to_string(),
        serial: "34335e34-bccf-11eb-8529-0242ac130003".to_string(),
    };

    iface
        .start_accessory(&strings)
        .await
        .map_err(ConnectError::CantSwitchUsbAOAMode)?;

    info!(
        "USB device {} (address=0x{:X}) switched to accessory mode",
        device_info.product_string().unwrap_or("unknown"),
        device_info.device_address()
    );

    drop(device); // disconnect the device
    Ok(())
}

pub async fn new(stream_config: AudioStream) -> Result<UsbStreamer, ConnectError> {
    // first try to switch any connected device to accessory mode
    for device in nusb::list_devices()
        .await
        .map_err(|e| ConnectError::NoUsbDevice(e.into()))?
    {
        if let Err(error) = switch_to_accessory(&device).await {
            warn!(
                "Cannot switch USB device 0x{:X} to accessory mode: {}",
                device.device_address(),
                error
            )
        } else {
            break;
        }
    }

    let (device_info, endpoints, reader, writer) = {
        let device_info = loop {
            match nusb::list_devices()
                .await
                .map_err(|e| ConnectError::NoUsbDevice(e.into()))?
                .find(|d| d.in_accessory_mode())
            {
                Some(info) => break info,
                None => {
                    return Err(ConnectError::NoUsbDevice(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "No USB device found in accessory mode.\nIf this is your first connection attempt, the device has been switched to accessory mode.\nPlease click Connect again to establish the connection.",
                    )));
                }
            }
        };

        let device = device_info
            .open()
            .await
            .map_err(|e| ConnectError::CantOpenUsbHandle(e.into()))?;

        let iface = device
            .claim_interface(0)
            .await
            .map_err(|e| ConnectError::CantClaimUsbInterface(e.into()))?;

        let endpoints = device.get_bulk_endpoints().ok_or_else(|| {
            ConnectError::CantLoadUsbConfig(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Accessory interface does not have required bulk endpoints",
            ))
        })?;

        let reader = iface
            .endpoint::<nusb::transfer::Bulk, nusb::transfer::In>(endpoints.0)
            .map_err(|e| ConnectError::CantLoadUsbConfig(e.into()))?
            .reader(TRANSFER_BUFFER_SIZE)
            .with_num_transfers(8);

        let writer = iface
            .endpoint::<nusb::transfer::Bulk, nusb::transfer::Out>(endpoints.1)
            .map_err(|e| ConnectError::CantLoadUsbConfig(e.into()))?
            .writer(TRANSFER_BUFFER_SIZE)
            .with_num_transfers(8);

        (device_info, endpoints, reader, writer)
    };

    info!(
        "Connected to USB device {} (address=0x{:X}, in=0x{:X} out=0x{:X}))",
        device_info.product_string().unwrap_or("unknown"),
        device_info.device_address(),
        endpoints.0,
        endpoints.1
    );

    let streamer = UsbStreamer {
        stream_config,
        reader: FramedRead::new(reader, LengthDelimitedCodec::new()),
        writer,
        is_listening: true,
        tracked_sequence: 0,
    };

    Ok(streamer)
}

impl StreamerTrait for UsbStreamer {
    fn reconfigure_stream(&mut self, stream_config: AudioStream) {
        self.stream_config = stream_config;
    }

    fn status(&self) -> StreamerMsg {
        if self.is_listening {
            StreamerMsg::Listening {
                ip: None,
                port: None,
            }
        } else {
            StreamerMsg::Connected {
                ip: None,
                port: None,
                mode: ConnectionMode::Usb,
            }
        }
    }

    async fn next(&mut self) -> Result<Option<StreamerMsg>, ConnectError> {
        match tokio::time::timeout(
            Duration::from_secs(if self.is_listening {
                Duration::MAX.as_secs()
            } else {
                1
            }),
            self.reader.next(),
        )
        .await
        {
            Ok(res) => match res {
                Some(Ok(frame)) => {
                    match MessageWrapper::decode(frame) {
                        Ok(packet) => {
                            match packet.payload {
                                Some(payload) => {
                                    let message = match payload {
                                        Payload::AudioPacket(packet) => {
                                            if packet.sequence_number < self.tracked_sequence {
                                                // drop packet
                                                info!(
                                                    "dropped packet: old sequence number {} < {}",
                                                    packet.sequence_number, self.tracked_sequence
                                                );
                                            }
                                            self.tracked_sequence = packet.sequence_number;

                                            let packet = packet.audio_packet.unwrap();
                                            let buffer_size = packet.buffer.len();
                                            let sample_rate = packet.sample_rate;

                                            match self.stream_config.process_audio_packet(packet) {
                                                Ok(Some(buffer)) => {
                                                    debug!("received {} bytes", buffer_size);
                                                    Some(StreamerMsg::UpdateAudioWave {
                                                        data: AudioPacketMessage::to_wave_data(
                                                            &buffer,
                                                            sample_rate,
                                                        ),
                                                    })
                                                }
                                                _ => None,
                                            }
                                        }
                                        Payload::Connect(_) => {
                                            info!("Received connect message from device");
                                            self.writer
                                                .write_all(CHECK_2.as_bytes())
                                                .await
                                                .map_err(|e| {
                                                    ConnectError::HandShakeFailed("writing", e)
                                                })?;
                                            self.writer.flush_end_async().await.map_err(|e| {
                                                ConnectError::HandShakeFailed("flushing", e)
                                            })?;

                                            None
                                        }
                                    };

                                    if self.is_listening {
                                        self.is_listening = false;
                                        Ok(Some(StreamerMsg::Connected {
                                            ip: None,
                                            port: None,
                                            mode: ConnectionMode::Usb,
                                        }))
                                    } else {
                                        Ok(message)
                                    }
                                }
                                None => todo!(),
                            }
                        }
                        Err(e) => Err(ConnectError::WriteError(WriteError::Deserializer(e))),
                    }
                }

                Some(Err(e)) => {
                    match e.kind() {
                        std::io::ErrorKind::TimedOut => Ok(None), // timeout use to check for input on stdin
                        std::io::ErrorKind::WouldBlock => Ok(None), // trigger on Linux when there is no stream input
                        _ => Err(WriteError::Io(e))?,
                    }
                }
                None => {
                    todo!()
                }
            },
            Err(_) => {
                self.is_listening = true;
                self.tracked_sequence = 0;
                Ok(Some(StreamerMsg::Listening {
                    ip: None,
                    port: None,
                }))
            }
        }
    }
}
