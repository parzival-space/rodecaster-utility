use std::error::Error;
use std::io::{Read, Write};
use std::time::Duration;
use log::{debug, info};
use nusb::{Device, DeviceInfo, Interface, MaybeFuture};
use nusb::io::{EndpointRead, EndpointWrite};
use nusb::transfer::{In, Interrupt, Out};

const HID_INTERFACE: u8 = 0x9;
const HID_INPUT_ENDPOINT: u8 = 0x85;
const HID_OUTPUT_ENDPOINT: u8 = 0x05;

pub struct RodecasterProII {
    device: Device,

    hid_interface: Interface,
    hid_reader: EndpointRead<Interrupt>,
    hid_writer: EndpointWrite<Interrupt>,
}

impl RodecasterProII {
    pub fn open(device_info: DeviceInfo) -> Result<Self, Box<dyn Error>> {
        let timeout = Duration::from_secs(1);
        let device = device_info.open().wait()?;

        info!("Connected to possible Rodecaster Pro II device: {:?}", device_info.device_address());

        // device.set_configuration(1).wait().expect("Failed to set configuration");
        debug!("Rodecaster Pro II configuration: {:?}", device.active_configuration()?.configuration_value());

        // acces hid interface
        let hid_interface = device.detach_and_claim_interface(HID_INTERFACE)
            .wait().expect("Failed to detach HID interface");

        let hid_reader = device.claim_interface(HID_INTERFACE)
            .wait().expect("Failed to claim HID interface")
            .endpoint::<Interrupt, In>(HID_INPUT_ENDPOINT)
            .expect("Failed to get HID input endpoint")
            .reader(256);

        let hid_writer = device.claim_interface(HID_INTERFACE)
            .wait().expect("Failed to claim HID interface")
            .endpoint::<Interrupt, Out>(HID_OUTPUT_ENDPOINT)
            .expect("Failed to get HID output endpoint")
            .writer(256);

        let mut instance = Self { device, hid_interface, hid_reader, hid_writer };

        info!("Initializing Rodecaster Pro II...");
        let init_command: [u8; 9] = [0x03, 0x04, 0x00, 0x00, 0x00, 0xAD, 0x10, 0xA7, 0xB0];
        instance.write_interrupt(&init_command).expect("Failed to send init command");

        info!("Rodecaster Pro II initialized successfully");
        Ok(instance)
    }

    pub fn read_interrupt(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buf: Vec<u8> = vec![0u8; 256];
        let bytes_read = self.hid_reader.read(&mut buf)?;
        buf.truncate(bytes_read);
        Ok(buf)
    }

    pub fn write_interrupt(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let mut buf: Vec<u8> = vec![0u8; 256];
        // fill the remaining bytes with zeros
        buf[..data.len()].copy_from_slice(data);


        self.hid_writer.write_all(buf.as_slice())?;
        self.hid_writer.flush()?;
        Ok(())
    }
}