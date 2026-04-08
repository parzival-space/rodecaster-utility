use crate::common::device::{AttachableUsbDevice};
use crate::{DeviceIdentifier};
use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use hidapi::{DeviceInfo, HidApi, HidDevice, HidError};
use log::{debug, error, info, trace, warn};
use std::io::{Cursor, Read};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::protocol::framing::{read_framed_message, RodeCasterPacketResult};
use crate::protocol::types::Structured;

const HID_REPORT_ID_SEND: u8 = 0x03;
const HID_REPORT_ID_RECEIVE: u8 = 0x04;

pub struct RodeCasterProIIDevice {
    device_identifier: DeviceIdentifier,
    state: Arc<Mutex<Structured>>
}

impl AttachableUsbDevice for RodeCasterProIIDevice {
    fn open(device_identifier: DeviceIdentifier) -> Result<RodeCasterProIIDevice> {
        let device_identifier_clone = device_identifier.clone();
        let state = Arc::new(Mutex::new(Structured {
            name: "".to_string(),
            properties: Default::default(),
            children: vec![],
        }));

        let state_clone = Arc::clone(&state);
        thread::spawn(move || {
            let Ok(hid_api) = HidApi::new() else {
                error!("Failed to initialize HID API for device read loop.");
                return;
            };
            Self::begin_read_loop(device_identifier_clone.device_info, hid_api, state_clone)
                .expect("Failed to begin read loop.");
        });

        Ok(RodeCasterProIIDevice { device_identifier, state })
    }

    fn get_device_info(&self) -> DeviceInfo {
        self.device_identifier.device_info.clone()
    }

    fn get_state(&self) -> Result<Structured> {
        let Ok(state) = self.state.lock() else {
            return Err(anyhow!("Failed to acquire lock on device state. Cannot get state."))
        };
        Ok(state.clone())
    }
}

impl RodeCasterProIIDevice {

    // this is the main io loop that is run in a new thread
    fn begin_read_loop(device_info: DeviceInfo, hid_api: HidApi, state: Arc<Mutex<Structured>>) -> Result<()> {
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
                    match packet {
                        RodeCasterPacketResult::Unknown(data) => warn!("Received unknown RODECaster packet {:?}", data),
                        RodeCasterPacketResult::PropertyUpdate(update) => {
                            let Ok(mut state) = state.lock() else {
                                error!("Failed to acquire lock on device state. Cannot apply property update.");
                                continue;
                            };
                            trace!("Applying property update: {:?}", update);
                            state
                                .set_property(update.indices, update.name, update.value)
                                .unwrap_or_else(|e| { warn!("Failed to set property. {}", e); });
                        },
                        RodeCasterPacketResult::DeviceReport(report) => {
                            let Ok(mut state) = state.lock() else {
                                error!("Failed to acquire lock on device state. Cannot apply device report update.");
                                continue;
                            };
                            trace!("Applying status update: {:?}", report.report);
                            *state = report.report;
                        }
                    }
                }
                Err(error) => {
                    error!("Error from device read: {:?}", error);
                    return Err(anyhow!("Error from device read: {:?}", error));
                }
            }
        }
    }
}