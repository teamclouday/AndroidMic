// Adapted from https://github.com/ZeroErrors/aoa-rs
// Not installing it because it has some dependency issues
//! This crate provides the ability to use the [Android Open Accessory Protocol 1.0](https://source.android.com/devices/accessories/aoa)
use std::ffi::{CString, NulError};
use std::time::Duration;

use byteorder::{ByteOrder, LittleEndian};
use futures::executor::block_on;
use nusb::descriptors::Configuration;
use nusb::transfer::{ControlIn, ControlOut, ControlType, Direction, EndpointType, Recipient};
use nusb::{DeviceInfo, Interface};
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
    NusbError(#[from] nusb::Error),
    #[error("invalid length (size: {0})")]
    InvalidLength(usize),
    #[error("unsupported accessory protocol (size: {0})")]
    UnsupportedProtocol(u16),
}

#[derive(Error, Debug)]
pub enum EndpointError {
    #[error("libusb error")]
    NusbError(#[from] nusb::Error),
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
    pub iface: u8,
    pub setting: u8,
    pub address: u8,
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

pub trait AccessoryDeviceInfo {
    /// Checks if the device is in accessory mode.
    fn in_accessory_mode(&self) -> bool;
}

impl AccessoryDeviceInfo for DeviceInfo {
    fn in_accessory_mode(&self) -> bool {
        let vid = self.vendor_id();
        let pid = self.product_id();
        vid == USB_ACCESSORY_VENDOR_ID
            && (pid == USB_ACCESSORY_PRODUCT_ID || pid == USB_ACCESSORY_ADB_PRODUCT_ID)
    }
}

pub trait AccessoryConfigurations {
    /// Finds the bulk in and out endpoints for this accessory.
    fn find_endpoints(&self) -> Result<Endpoints, EndpointError>;
}

impl AccessoryConfigurations for Configuration<'_> {
    fn find_endpoints(&self) -> Result<Endpoints, EndpointError> {
        let mut endpoint_in: Option<Endpoint> = None;
        let mut endpoint_out: Option<Endpoint> = None;

        'outer: for iface in self.interface_alt_settings() {
            for endpoint in iface.endpoints() {
                if let EndpointType::Bulk = endpoint.transfer_type() {
                    match endpoint.direction() {
                        Direction::In => {
                            if endpoint_in.is_none() {
                                endpoint_in = Some(Endpoint {
                                    iface: iface.interface_number(),
                                    setting: iface.alternate_setting(),
                                    address: endpoint.address(),
                                });
                                if endpoint_out.is_some() {
                                    break 'outer;
                                }
                            }
                        }
                        Direction::Out => {
                            if endpoint_out.is_none() {
                                endpoint_out = Some(Endpoint {
                                    iface: iface.interface_number(),
                                    setting: iface.alternate_setting(),
                                    address: endpoint.address(),
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

pub trait AccessoryInterface {
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

impl AccessoryInterface for Interface {
    fn get_protocol(&mut self, timeout: Duration) -> Result<u16, AccessoryError> {
        let size = block_on(
            tokio::time::timeout(
                timeout,
                self.control_in(ControlIn {
                    control_type: ControlType::Vendor,
                    recipient: Recipient::Device,
                    request: ACCESSORY_GET_PROTOCOL,
                    value: 0,
                    index: 0,
                    length: size_of::<u16>() as u16,
                }),
            )
            .into_inner(),
        )
        .into_result()
        .map_err(nusb::Error::other)?;

        if size.len() != size_of::<u16>() {
            return Err(AccessoryError::InvalidLength(size.len()));
        }

        Ok(LittleEndian::read_u16(size.as_ref()))
    }

    fn send_string(
        &mut self,
        index: u16,
        str: &CString,
        timeout: Duration,
    ) -> Result<(), AccessoryError> {
        let data = str.as_bytes_with_nul();
        let size = block_on(
            tokio::time::timeout(
                timeout,
                self.control_out(ControlOut {
                    control_type: ControlType::Vendor,
                    recipient: Recipient::Device,
                    request: ACCESSORY_SEND_STRING,
                    index,
                    value: 0,
                    data,
                }),
            )
            .into_inner(),
        )
        .into_result()
        .map_err(nusb::Error::other)?;

        if size.actual_length() != data.len() {
            return Err(AccessoryError::InvalidLength(size.actual_length()));
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
        block_on(
            tokio::time::timeout(
                timeout,
                self.control_out(ControlOut {
                    control_type: ControlType::Vendor,
                    recipient: Recipient::Device,
                    request: ACCESSORY_START,
                    index: 0,
                    value: 0,
                    data: &[],
                }),
            )
            .into_inner(),
        )
        .into_result()
        .map_err(nusb::Error::other)?;

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
