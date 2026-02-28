use std::{
    ffi::CString,
    io::{Error, ErrorKind},
    time::Duration,
};

use nusb::{
    Device, DeviceInfo, Interface,
    descriptors::TransferType,
    transfer::{ControlIn, ControlOut, ControlType, Direction, Recipient},
};

const USB_AOA_VID: u16 = 0x18D1;
const USB_AOA_PID_MIN: u16 = 0x2D00;
const USB_AOA_PID_MAX: u16 = 0x2D05;

const ACCESSORY_STRING_MANUFACTURER: u16 = 0x00;
const ACCESSORY_STRING_MODEL: u16 = 0x01;
const ACCESSORY_STRING_DESCRIPTION: u16 = 0x02;
const ACCESSORY_STRING_VERSION: u16 = 0x03;
const ACCESSORY_STRING_URI: u16 = 0x04;
const ACCESSORY_STRING_SERIAL: u16 = 0x05;

const REQUEST_GET_PROTOCOL: u8 = 0x33;
const REQUEST_SEND_STRING: u8 = 0x34;
const REQUEST_START: u8 = 0x35;

#[derive(Clone, Debug, Default)]
pub struct AccessoryStrings {
    pub manufacturer: String,
    pub model: String,
    pub description: String,
    pub version: String,
    pub uri: String,
    pub serial: String,
}

pub trait AccessoryDeviceInfoExt {
    // Check if the device is in accessory mode
    fn in_accessory_mode(&self) -> bool;
}

impl AccessoryDeviceInfoExt for DeviceInfo {
    fn in_accessory_mode(&self) -> bool {
        self.vendor_id() == USB_AOA_VID
            && (USB_AOA_PID_MIN..=USB_AOA_PID_MAX).contains(&self.product_id())
    }
}

pub trait AccessoryDeviceExt {
    // Get bulk in and out endpoints for the accessory interface
    fn get_bulk_endpoints(&self) -> Option<(u8, u8)>;
}

impl AccessoryDeviceExt for Device {
    fn get_bulk_endpoints(&self) -> Option<(u8, u8)> {
        let config = self.active_configuration().ok()?;

        let iface = config.interfaces().next()?;
        let mut bulk_in = None;
        let mut bulk_out = None;

        for id in iface.alt_settings() {
            for ep in id.endpoints() {
                if ep.transfer_type() == TransferType::Bulk {
                    match ep.direction() {
                        Direction::In => bulk_in = Some(ep.address()),
                        Direction::Out => bulk_out = Some(ep.address()),
                    }
                }
            }
        }

        match (bulk_in, bulk_out) {
            (Some(in_ep), Some(out_ep)) => Some((in_ep, out_ep)),
            _ => None,
        }
    }
}

pub trait AccessoryInterfaceExt {
    // Get the protocol version supported by the device.
    async fn get_protocol(&mut self, timeout: Duration) -> Result<u16, Error>;

    // Send a string to the device.
    async fn send_string(&mut self, index: u16, text: &str, timeout: Duration)
    -> Result<(), Error>;

    // Send all accessory strings to the device.
    async fn send_accessory_strings(
        &mut self,
        accessory: &AccessoryStrings,
        timeout: Duration,
    ) -> Result<(), Error>;

    // Send the command to start accessory mode.
    async fn send_start(&mut self, timeout: Duration) -> Result<(), Error>;

    // Switch the device to accessory mode.
    async fn start_accessory(&mut self, accessory: &AccessoryStrings) -> Result<u16, Error>;
}

impl AccessoryInterfaceExt for Interface {
    async fn get_protocol(&mut self, timeout: Duration) -> Result<u16, Error> {
        let req = ControlIn {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request: REQUEST_GET_PROTOCOL,
            value: 0,
            index: 0,
            length: 2, // 2 byte response
        };

        let res = self.control_in(req, timeout).await?;

        if res.len() != 2 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Expected 2 bytes response for protocol version",
            )
            .into());
        }

        Ok(u16::from_le_bytes([res[0], res[1]]))
    }

    async fn send_string(
        &mut self,
        index: u16,
        text: &str,
        timeout: Duration,
    ) -> Result<(), Error> {
        let c_str = CString::new(text)?;
        let data = c_str.as_bytes_with_nul();

        let req = ControlOut {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request: REQUEST_SEND_STRING,
            value: 0,
            index,
            data,
        };

        self.control_out(req, timeout).await?;

        Ok(())
    }

    async fn send_accessory_strings(
        &mut self,
        accessory: &AccessoryStrings,
        timeout: Duration,
    ) -> Result<(), Error> {
        self.send_string(
            ACCESSORY_STRING_MANUFACTURER,
            &accessory.manufacturer,
            timeout,
        )
        .await?;
        self.send_string(ACCESSORY_STRING_MODEL, &accessory.model, timeout)
            .await?;
        self.send_string(
            ACCESSORY_STRING_DESCRIPTION,
            &accessory.description,
            timeout,
        )
        .await?;
        self.send_string(ACCESSORY_STRING_VERSION, &accessory.version, timeout)
            .await?;
        self.send_string(ACCESSORY_STRING_URI, &accessory.uri, timeout)
            .await?;
        self.send_string(ACCESSORY_STRING_SERIAL, &accessory.serial, timeout)
            .await?;

        Ok(())
    }

    async fn send_start(&mut self, timeout: Duration) -> Result<(), Error> {
        let req = ControlOut {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request: REQUEST_START,
            value: 0,
            index: 0,
            data: &[],
        };

        self.control_out(req, timeout).await?;

        Ok(())
    }

    async fn start_accessory(&mut self, accessory: &AccessoryStrings) -> Result<u16, Error> {
        let timeout = Duration::from_secs(1);

        let protocol = self.get_protocol(timeout).await?;
        if protocol < 1 {
            return Err(Error::new(
                ErrorKind::Unsupported,
                format!("Unsupported AOA protocol version: {}", protocol),
            ));
        }

        self.send_accessory_strings(accessory, timeout).await?;
        self.send_start(timeout).await?;

        Ok(protocol)
    }
}
