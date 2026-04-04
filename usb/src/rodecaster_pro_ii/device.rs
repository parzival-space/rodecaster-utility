use crate::common::device::{AttachableUsbDevice};
use crate::{DeviceIdentifier};
use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use hidapi::{DeviceInfo, HidApi, HidDevice};
use log::{error, info, warn};
use std::io::{Cursor, Read};
use std::thread;
use crate::protocol::framing::read_framed_message;

const HID_REPORT_ID_SEND: u8 = 0x03;
const HID_REPORT_ID_RECEIVE: u8 = 0x04;

pub struct RodeCasterProIIDevice {
    device_identifier: DeviceIdentifier
}

impl AttachableUsbDevice for RodeCasterProIIDevice {
    fn open(device_identifier: DeviceIdentifier) -> Result<RodeCasterProIIDevice> {
        let device_identifier_clone = device_identifier.clone();
        thread::spawn(move || {
            let Ok(hid_api) = HidApi::new() else {
                error!("Failed to initialize HID API for device read loop.");
                return;
            };
            Self::begin_read_loop(device_identifier_clone.device_info, hid_api)
                .expect("Failed to begin read loop.");
        });

        Ok(RodeCasterProIIDevice { device_identifier })
    }

    fn get_device_info(&self) -> DeviceInfo {
        self.device_identifier.device_info.clone()
    }
}

impl RodeCasterProIIDevice {

    // this is the main io loop that is run in a new thread
    fn begin_read_loop(device_info: DeviceInfo, hid_api: HidApi) -> Result<()> {
        let device = device_info.open_device(&hid_api)?;
        device.set_blocking_mode(true)?;

        // todo: remove this, this is just for testing purpose until the send implementation is fixed
        device.send_output_report(&*vec![HID_REPORT_ID_SEND, 0x04, 0x00, 0x00, 0x00, 0xAD, 0x10, 0xA7, 0xB0])?;

        loop {
            match read_framed_message(|| {
                // read next HID report
                let mut hid_report_buffer = [0u8; 256];
                hid_report_buffer[0] = HID_REPORT_ID_RECEIVE;
                let _ = &device.read(&mut hid_report_buffer)?;

                Ok(hid_report_buffer[1..].to_vec()) // skip first byte, as it's the report id
            }) {
                Ok(packet) => {
                    info!("Received packet {:?}", packet);
                }
                Err(error) => {
                    error!("Failed to read framed message: {:?}", error);
                }
            }
        }
    }
}