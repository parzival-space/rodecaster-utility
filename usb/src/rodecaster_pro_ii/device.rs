use crate::common::device::{AttachableUsbDevice, ExecutableUsbDevice};
use crate::{DeviceIdentifier};
use anyhow::{Result};
use byteorder::{ReadBytesExt};
use hidapi::{DeviceInfo, HidApi, HidDevice};
use log::{error, info, warn};
use std::io::{Cursor, Read};
use std::thread;
use crate::parser::parse_rodecaster_packet;

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
            Self::begin_read_loop(device_identifier_clone.device_info, hid_api);
        });

        Ok(RodeCasterProIIDevice { device_identifier })
    }

    fn get_device_info(&self) -> DeviceInfo {
        self.device_identifier.device_info.clone()
    }
}

impl ExecutableUsbDevice for RodeCasterProIIDevice {
    fn write(&mut self, data: Vec<u8>) -> Result<()> {
        // todo: implement this
        Ok(())
    }

    fn read(&mut self) -> Result<Vec<u8>> {
        // todo: implement this
        Ok(Vec::new())
    }
}

impl RodeCasterProIIDevice {

    fn begin_read_loop(device_info: DeviceInfo, hid_api: HidApi) {
        let device = match device_info.open_device(&hid_api) {
            Ok(device) => device,
            Err(e) => {
                warn!("Failed to open device for reading: {}", e);
                return;
            }
        };

        // todo: remove this, this is just for testing purpose until the send implementation is fixed
        let Ok(_) = device.send_output_report(&*vec![HID_REPORT_ID_SEND, 0x04, 0x00, 0x00, 0x00, 0xAD, 0x10, 0xA7, 0xB0]) else {
            warn!("Failed to write initial message to device");
            return;
        };

        let mut failed_read_attempts: u8 = 0;
        loop {
            // messages have a max size of 256 bytes, the first byte is reserved for the report ID.
            let mut report_buffer = vec![0u8; 256];
            report_buffer[0] = HID_REPORT_ID_RECEIVE;

            // read new incoming messages from the device
            match Self::read_next_message(&device) {
                Ok(message_buffer) => {
                    failed_read_attempts = 0;

                    match parse_rodecaster_packet(&message_buffer) {
                        Ok((remaining_buffer, packet)) => {
                            info!("Received packet from device: {:?}", packet);

                            if !remaining_buffer.is_empty() {
                                warn!("There were {} unparsed bytes remaining after parsing message from device.", remaining_buffer.len());
                            }
                        }
                        Err(nom::Err::Incomplete(needed)) =>
                            error!("Incomplete message received from device: needed {:?}", needed),
                        Err(e) =>
                            error!("Failed to parse message received from device: {:?}", e),
                    }
                }
                Err(e) => {
                    warn!("Failed to read message from device: {}", e);
                    if failed_read_attempts >= 5 {
                        error!("Quitting read loop after 5 failed read attempts.");
                        break;
                    }

                    failed_read_attempts = failed_read_attempts + 1;
                }
            }
        }
    }

    fn read_next_message(device: &HidDevice) -> Result<Vec<u8>> {
        // messages have a max size of 256 bytes, the first byte is reserved for the report ID.
        let mut report_buffer = vec![0u8; 256];
        report_buffer[0] = HID_REPORT_ID_RECEIVE;

        // read next message
        let _ = &device.read(&mut report_buffer)?;
        let mut reader = Cursor::new(&report_buffer[1..]);

        // first 4 bytes are the message length, the rest is the message data
        let message_length = reader.read_u32::<byteorder::LittleEndian>()?;
        let mut bytes_read_total: u32 = 0;
        let mut message_buffer = vec![0u8; message_length as usize];

        while bytes_read_total < message_length {
            let bytes_read_now =  reader.read(&mut message_buffer[bytes_read_total as usize..])?;
            bytes_read_total += bytes_read_now as u32;

            if bytes_read_now == 0 {
                warn!("No more data to read from device, but message length is not satisfied yet.");
                break;
            }

            // read next message if the message length is not satisfied yet
            if bytes_read_total < message_length {
                // reuse report buffer, clear all values after the first byte
                report_buffer[1..].fill(0);

                // read next report
                match &device.read(&mut report_buffer) {
                    Ok(_) => {
                        reader = Cursor::new(&report_buffer[1..]);
                    }
                    Err(e) => {
                        warn!("Failed to read continuation message from device: {}", e);
                        break;
                    }
                }
            }
        }

        Ok(message_buffer)
    }
}