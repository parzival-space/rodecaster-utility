use crate::common::device::{AttachableUsbDevice};
use crate::{DeviceIdentifier};
use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt};
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
            match Self::read_full_report(&device) {
                Ok(message_buffer) => match parse_rodecaster_packet(&message_buffer) {
                    Ok((remaining_buffer, packet)) => {
                        info!("Received packet from device: {:?}", packet);

                        if !remaining_buffer.is_empty() {
                            warn!("There were {} unparsed bytes remaining.", remaining_buffer.len());
                        }
                    }
                    Err(nom::Err::Incomplete(needed)) =>
                        error!("Incomplete message received from device: needed {:?}", needed),
                    Err(nom::Err::Error(needed)) => {
                        error!("Failed to parse message received from device.\nNext 10 bytes: {:2x?}\nAs ASCII: {:?}",
                            &needed.input[..10], String::from_utf8_lossy(&needed.input[..10]));
                    }
                    Err(e) =>
                        error!("Failed to parse message received from device: {:?}", e),
                }
                Err(e) => {
                    warn!("Failed to read message from device: {}", e);
                }
            }
        }
    }

    fn read_full_report(device: &HidDevice) -> Result<Vec<u8>> {
        let mut hid_report_buffer = [0u8; 256];
        hid_report_buffer[0] = HID_REPORT_ID_RECEIVE; // first byte is used as argument for hidapi

        let firs_report_read = &device.read(&mut hid_report_buffer)?;
        if firs_report_read < &4usize {
            return Err(anyhow!(
                "First read report of packet is too short to even contain the message length."))
        }

        // begin reading the full message
        let mut reader = Cursor::new(&hid_report_buffer[1..]);
        let packet_length = reader.read_u32::<LittleEndian>()?;
        let mut packet_buffer = vec![0u8; packet_length as usize];
        let mut packet_bytes_read = 0;

        while packet_bytes_read < packet_length {
            let packet_bytes_read_now = reader.read(&mut packet_buffer[packet_bytes_read as usize..])?;
            packet_bytes_read += packet_bytes_read_now as u32;

            if packet_bytes_read_now <= 1 {
                warn!("No more data to read from device, but packet length is not satisfied yet.");
                break;
            }

            if packet_bytes_read < packet_length {
                hid_report_buffer[1..].fill(0); // clear except report id
                match &device.read(&mut hid_report_buffer) {
                    Ok(_) => reader = Cursor::new(&hid_report_buffer[1..]),
                    Err(e) => {
                        warn!("Failed to read continuation message from device: {}", e);
                        break;
                    }
                }
            }
        }

        Ok(packet_buffer)
    }
}