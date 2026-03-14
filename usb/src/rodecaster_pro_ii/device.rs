use crate::common::device::{AttachableUsbDevice, ExecutableUsbDevice};
use crate::rodecaster_pro_ii::command::RodeCasterProIIExecutable;
use crate::rodecaster_pro_ii::{PID_RODECASTER_PRO_II, PID_RODECASTER_PRO_II_DYNAMIC_EXTENDED, PID_RODECASTER_PRO_II_DYNAMIC_EXTENDED_OUTPUT, PID_RODECASTER_PRO_II_EXTENDED, PID_RODECASTER_PRO_II_EXTENDED_INPUT, PID_RODECASTER_PRO_II_EXTENDED_OUTPUT};
use crate::VID_RODE;
use anyhow::{anyhow, bail, Result};
use byteorder::ReadBytesExt;
use hidapi::{DeviceInfo, HidApi, HidDevice, HidResult};
use log::{debug, warn};
use std::io::{Cursor, Read};
use tokio::sync::{broadcast, mpsc};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

const HID_REPORT_ID_SEND: u8 = 0x03;
const HID_REPORT_ID_RECEIVE: u8 = 0x04;

pub struct RodeCasterProIIDevice {
    device: HidDevice,
    device_info: DeviceInfo,

}

impl AttachableUsbDevice for RodeCasterProIIDevice {
    fn open(api: &mut HidApi, serial: &str) -> Result<Self> {
        let device_info = api.device_list()
            .find(|device|
                VID_RODE.eq(&device.vendor_id()) &&
                    Self::is_device_supported(device) == true &&
                device.serial_number().map_or(false, |s| serial.eq(s)))
            .ok_or_else(|| anyhow!("No RODECaster Pro II found with Serial {}", serial))?;

        let device = match device_info.open_device(&api) {
            Ok(device) => device,
            Err(e) => {
                bail!("Failed to open RODECaster Pro II with Serial {}: {}", serial, e);
            }
        };

        let mut rodecaster_device = RodeCasterProIIDevice {
            device,
            device_info: device_info.clone()
        };

        let hid_api = HidApi::new()?;
        let loop_device_info = device_info.clone();
        thread::spawn(move ||
            RodeCasterProIIDevice::begin_read_loop(loop_device_info, hid_api)
        );

        debug!("Successfully connected to RODECaster Pro II. Initializing device...");
        rodecaster_device.request_device_status()?;

        Ok(rodecaster_device)
    }

    fn is_device_supported(device: &DeviceInfo) -> bool
    where
        Self: Sized
    {
        // multitrack devices
        PID_RODECASTER_PRO_II.eq(&device.product_id()) ||
        PID_RODECASTER_PRO_II_EXTENDED.eq(&device.product_id()) ||
        PID_RODECASTER_PRO_II_EXTENDED_INPUT.eq(&device.product_id()) ||
        PID_RODECASTER_PRO_II_EXTENDED_OUTPUT.eq(&device.product_id()) ||
        // 1.7.3 dynamic multitrack devices
        PID_RODECASTER_PRO_II_DYNAMIC_EXTENDED.eq(&device.product_id()) ||
        PID_RODECASTER_PRO_II_DYNAMIC_EXTENDED_OUTPUT.eq(&device.product_id())
    }

    fn get_vendor_id(&self) -> u16 {
        self.device_info.vendor_id()
    }

    fn get_product_id(&self) -> u16 {
        self.device_info.product_id()
    }

    fn get_manufacturer_string(&self) -> Option<&str> {
        self.device_info.manufacturer_string()
    }

    fn get_product_string(&self) -> Option<&str> {
        self.device_info.product_string()
    }

    fn get_serial_number_string(&self) -> Option<&str> {
        self.device_info.serial_number()
    }
}

impl ExecutableUsbDevice for RodeCasterProIIDevice {
    fn write(&mut self, data: Vec<u8>) -> Result<()> {
        if data.len() > 255 {
            bail!("Data length exceeds maximum allowed size of 255 bytes");
        }

        // messages has a size of 256 bytes, but the first byte is reserved for the report ID.
        let mut buffer = vec![0u8; 256];
        buffer[0] = HID_REPORT_ID_SEND;
        buffer[1..(data.len() + 1)].copy_from_slice(&data);

        debug!("Writing report with id ({}), data: {:02x?}", buffer[0], buffer[1..].to_vec());
        match self.device.write(&buffer) {
            Ok(_) => Ok(()),
            Err(e) => {
                Err(anyhow!("Failed to write message: {}", e))
            }
        }
    }

    fn read(&mut self) -> Result<Vec<u8>> {
        sleep(Duration::from_millis(100)); // wait a bit to give the read loop time to read incoming messages
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

        if let Err(blocking_mode_err) = device.set_blocking_mode(true) {
            warn!("Failed to set blocking mode: {}", blocking_mode_err);
        }

        // continuously read incoming messages from the device in a separate thread
        loop {
            // messages have a max size of 256 bytes, the first byte is reserved for the report ID.
            let mut report_buffer = vec![0u8; 256];
            report_buffer[0] = HID_REPORT_ID_RECEIVE;

            // read new incoming messages from the device
            match device.read(&mut report_buffer) {
                Ok(_) => {
                    let mut reader = Cursor::new(&report_buffer[1..]);

                    // first 4 bytes are the message length, the rest is the message data
                    let Ok(message_length) = reader.read_u32::<byteorder::LittleEndian>() else {
                        warn!("Failed to read message length from device.");
                        continue;
                    };

                    debug!("Reading new new message with length {}", message_length);

                    // build a buffer, add remaining data from the message and read the rest of the message if necessary
                    let mut bytes_read: u32 = 0;
                    let mut message_buffer = vec![0u8; message_length as usize];
                    while bytes_read < message_length {
                        let Ok(bytes_read_now) = reader.read(&mut message_buffer[bytes_read as usize..]) else {
                            warn!("Failed to read message data from device.");
                            continue;
                        };
                        bytes_read += bytes_read_now as u32;

                        if bytes_read_now == 0 {
                            warn!("No more data to read from device, but message length is not satisfied yet.");
                            break;
                        }

                        // read next message if the message length is not satisfied yet
                        if bytes_read < message_length {
                            // reuse report buffer, clear all values after the first byte
                            report_buffer[1..].fill(0);

                            // read next report
                            match device.read(&mut report_buffer) {
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

                    // full message is read, process it
                    debug!("Received message from device: {:02x?}", message_buffer);
                    // todo: add message sender/handler here

                }
                Err(e) => {
                    warn!("Failed to read message from device: {}", e);
                }
            }
        }
    }
}