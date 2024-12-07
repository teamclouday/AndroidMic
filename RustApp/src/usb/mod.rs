// Copied from https://github.com/ZeroErrors/aoa-rs
// Not installing it because it has some dependency issues
//! This crate provides the ability to use the [Android Open Accessory Protocol 1.0](https://source.android.com/devices/accessories/aoa)
use std::ffi::{CString, NulError};
use std::mem::size_of;
use std::time::Duration;

use byteorder::{ByteOrder, LittleEndian};
use rusb::{
    request_type, Device, DeviceHandle, Direction, Recipient, RequestType, TransferType, UsbContext,
};
use thiserror::Error;

pub const USB_ACCESSORY_VENDOR_ID: u16 = 0x18D1;

pub const USB_ACCESSORY_PRODUCT_ID: u16 = 0x2D00;
pub const USB_ACCESSORY_ADB_PRODUCT_ID: u16 = 0x2D01;

pub const ACCESSORY_STRING_MANUFACTURER: u16 = 0x00;
pub const ACCESSORY_STRING_MODEL: u16 = 0x01;
pub const ACCESSORY_STRING_DESCRIPTION: u16 = 0x02;
pub const ACCESSORY_STRING_VERSION: u16 = 0x03;
pub const ACCESSORY_STRING_URI: u16 = 0x04;
pub const ACCESSORY_STRING_SERIAL: u16 = 0x05;

pub const ACCESSORY_GET_PROTOCOL: u8 = 0x33;
pub const ACCESSORY_SEND_STRING: u8 = 0x34;
pub const ACCESSORY_START: u8 = 0x35;

#[derive(Error, Debug)]
pub enum AccessoryError {
    #[error("libusb error")]
    RusbError(#[from] rusb::Error),
    #[error("invalid length (size: {0})")]
    InvalidLength(usize),
    #[error("unsupported accessory protocol (size: {0})")]
    UnsupportedProtocol(u16),
}

#[derive(Error, Debug)]
pub enum EndpointError {
    #[error("libusb error")]
    RusbError(#[from] rusb::Error),
    #[error("unable to find endpoints (in: {}, out: {})", match .0 {
        Some(n) => format!("Some({})", n.address),
        None => "None".to_string()
    }, match .1 {
        Some(n) => format!("Some({})", n.address),
        None => "None".to_string()
    })]
    InvalidEndpoints(Option<Endpoint>, Option<Endpoint>),
}

#[derive(Copy, Clone, Debug)]
pub struct Endpoint {
    pub config: u8,
    pub iface: u8,
    pub setting: u8,
    pub address: u8,
}

impl Endpoint {
    pub fn config_endpoint<T: UsbContext>(
        &self,
        handle: &DeviceHandle<T>,
    ) -> Result<(), rusb::Error> {
        handle.set_active_configuration(self.config)?;
        handle.claim_interface(self.iface)?;
        handle.set_alternate_setting(self.iface, self.setting)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Endpoints(pub Endpoint, pub Endpoint);

impl Endpoints {
    pub fn endpoint_in(&self) -> Endpoint {
        self.0
    }

    pub fn endpoint_out(&self) -> Endpoint {
        self.1
    }
}

pub trait AccessoryDevice {
    /// Checks if the device is in accessory mode.
    ///
    /// See: https://source.android.com/devices/accessories/aoa#establish-communication-with-the-device
    fn in_accessory_mode(&self) -> Result<bool, AccessoryError>;
    /// Finds the bulk in and out endpoints for this accessory.
    fn find_endpoints(&self) -> Result<Endpoints, EndpointError>;
}

impl<T: UsbContext> AccessoryDevice for Device<T> {
    fn in_accessory_mode(&self) -> Result<bool, AccessoryError> {
        let device_desc = self.device_descriptor()?;
        let vid = device_desc.vendor_id();
        let pid = device_desc.product_id();
        Ok(vid == USB_ACCESSORY_VENDOR_ID
            && (pid == USB_ACCESSORY_PRODUCT_ID || pid == USB_ACCESSORY_ADB_PRODUCT_ID))
    }

    fn find_endpoints(&self) -> Result<Endpoints, EndpointError> {
        let device_desc = self.device_descriptor()?;

        let mut endpoint_in: Option<Endpoint> = None;
        let mut endpoint_out: Option<Endpoint> = None;

        'outer: for config_index in 0..device_desc.num_configurations() {
            let config = self.config_descriptor(config_index)?;
            for iface in config.interfaces() {
                for iface_desc in iface.descriptors() {
                    for endpoint_desc in iface_desc.endpoint_descriptors() {
                        if let TransferType::Bulk = endpoint_desc.transfer_type() {
                            match endpoint_desc.direction() {
                                Direction::In => {
                                    if endpoint_in.is_none() {
                                        endpoint_in = Some(Endpoint {
                                            config: config.number(),
                                            iface: iface_desc.interface_number(),
                                            setting: iface_desc.setting_number(),
                                            address: endpoint_desc.address(),
                                        });
                                        if endpoint_out.is_some() {
                                            break 'outer;
                                        }
                                    }
                                }
                                Direction::Out => {
                                    if endpoint_out.is_none() {
                                        endpoint_out = Some(Endpoint {
                                            config: config.number(),
                                            iface: iface_desc.interface_number(),
                                            setting: iface_desc.setting_number(),
                                            address: endpoint_desc.address(),
                                        });
                                        if endpoint_in.is_some() {
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let (Some(endpoint_in), Some(endpoint_out)) = (endpoint_in, endpoint_out) {
            Ok(Endpoints(endpoint_in, endpoint_out))
        } else {
            Err(EndpointError::InvalidEndpoints(endpoint_in, endpoint_out))
        }
    }
}

/// Contains string information about the accessory
#[derive(Clone, Debug)]
pub struct AccessoryStrings {
    manufacturer: CString,
    model: CString,
    description: CString,
    version: CString,
    uri: CString,
    serial: CString,
}

impl AccessoryStrings {
    pub fn new<T: Into<Vec<u8>>>(
        manufacturer: T,
        model: T,
        description: T,
        version: T,
        uri: T,
        serial: T,
    ) -> Result<Self, NulError> {
        Ok(Self {
            manufacturer: CString::new(manufacturer)?,
            model: CString::new(model)?,
            description: CString::new(description)?,
            version: CString::new(version)?,
            uri: CString::new(uri)?,
            serial: CString::new(serial)?,
        })
    }

    pub fn new_cstring(
        manufacturer: CString,
        model: CString,
        description: CString,
        version: CString,
        uri: CString,
        serial: CString,
    ) -> Self {
        Self {
            manufacturer,
            model,
            description,
            version,
            uri,
            serial,
        }
    }
}

pub trait AccessoryHandle {
    /// Sends the `ACCESSORY_GET_PROTOCOL` control request and reads the reply.
    ///
    /// Returns the protocol version supported by the device.
    ///
    /// See: https://source.android.com/devices/accessories/aoa#attempt-to-start-in-accessory-mode
    fn get_protocol(&mut self, timeout: Duration) -> Result<u16, AccessoryError>;

    /// Sends the `ACCESSORY_STRING` control request with string data.
    ///
    /// See: https://source.android.com/devices/accessories/aoa#attempt-to-start-in-accessory-mode
    fn send_string(
        &mut self,
        index: u16,
        str: &CString,
        timeout: Duration,
    ) -> Result<(), AccessoryError>;

    /// Sends the `ACCESSORY_STRING` control request for all strings in `AccessoryStrings`.
    ///
    /// See: https://source.android.com/devices/accessories/aoa#attempt-to-start-in-accessory-mode
    fn send_accessory_strings(
        &mut self,
        accessory: &AccessoryStrings,
        timeout: Duration,
    ) -> Result<(), AccessoryError>;

    /// Sends the `ACCESSORY_START` control request.
    ///
    /// See: https://source.android.com/devices/accessories/aoa#attempt-to-start-in-accessory-mode
    fn send_start(&mut self, timeout: Duration) -> Result<(), AccessoryError>;

    /// Attempts to start the accessory mode.
    ///
    /// Returns the protocol version supported by the device.
    ///
    /// See: https://source.android.com/devices/accessories/aoa#attempt-to-start-in-accessory-mode
    fn start_accessory(
        &mut self,
        accessory: &AccessoryStrings,
        timeout: Duration,
    ) -> Result<u16, AccessoryError>;
}

impl<T: UsbContext> AccessoryHandle for DeviceHandle<T> {
    fn get_protocol(&mut self, timeout: Duration) -> Result<u16, AccessoryError> {
        let mut buf = [0; size_of::<u16>()];
        let size = self.read_control(
            request_type(Direction::In, RequestType::Vendor, Recipient::Device),
            ACCESSORY_GET_PROTOCOL,
            0,
            0,
            &mut buf,
            timeout,
        )?;
        if size != buf.len() {
            return Err(AccessoryError::InvalidLength(size));
        }

        Ok(LittleEndian::read_u16(&buf))
    }

    fn send_string(
        &mut self,
        index: u16,
        str: &CString,
        timeout: Duration,
    ) -> Result<(), AccessoryError> {
        let buf = str.as_bytes_with_nul();
        let size = self.write_control(
            request_type(Direction::Out, RequestType::Vendor, Recipient::Device),
            ACCESSORY_SEND_STRING,
            0,
            index,
            buf,
            timeout,
        )?;
        if size != buf.len() {
            return Err(AccessoryError::InvalidLength(size));
        }

        Ok(())
    }

    fn send_accessory_strings(
        &mut self,
        strings: &AccessoryStrings,
        timeout: Duration,
    ) -> Result<(), AccessoryError> {
        self.send_string(
            ACCESSORY_STRING_MANUFACTURER,
            &strings.manufacturer,
            timeout,
        )?;
        self.send_string(ACCESSORY_STRING_MODEL, &strings.model, timeout)?;
        self.send_string(ACCESSORY_STRING_DESCRIPTION, &strings.description, timeout)?;
        self.send_string(ACCESSORY_STRING_VERSION, &strings.version, timeout)?;
        self.send_string(ACCESSORY_STRING_URI, &strings.uri, timeout)?;
        self.send_string(ACCESSORY_STRING_SERIAL, &strings.serial, timeout)?;

        Ok(())
    }

    fn send_start(&mut self, timeout: Duration) -> Result<(), AccessoryError> {
        self.write_control(
            request_type(Direction::Out, RequestType::Vendor, Recipient::Device),
            ACCESSORY_START,
            0,
            0,
            &[],
            timeout,
        )?;

        Ok(())
    }

    fn start_accessory(
        &mut self,
        strings: &AccessoryStrings,
        timeout: Duration,
    ) -> Result<u16, AccessoryError> {
        let protocol = self.get_protocol(timeout)?;

        // Any protocol >= 1 is accepted
        if protocol == 0 {
            return Err(AccessoryError::UnsupportedProtocol(protocol));
        }

        self.send_accessory_strings(strings, timeout)?;
        self.send_start(timeout)?;

        Ok(protocol)
    }
}
