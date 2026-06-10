use std::cmp::min;
use std::env;
use hidapi::{DeviceInfo, HidApi, HidDevice};
use rodecaster_usb::devices;
use rodecaster_usb::error::UsbError;

// todo: Remove this code or replace it with something that does not introduce so much duplication.

pub(crate) const PID_RODECASTER_PRO_II: &[u16] = &[0x0037, 0x0072, 0x0078, 0x0030, 0x0094, 0x0092];
const INIT_PAYLOAD: &[u8] = &[0xAD, 0x10, 0xA7, 0xB0];
pub(crate) const HID_REPORT_ID_SEND: u8 = 0x03;
pub(crate) const HID_REPORT_ID_RECEIVE: u8 = 0x04;
const OUTPUT_FILE: &str = "rodecaster_pro_ii_status.bin";

const FRAME_PAYLOAD_SIZE: usize = 255; // 256 byte hid report, minus 1 report id byte

fn main() {
    let args = env::args().collect::<Vec<_>>();

    let hid_api = HidApi::new().expect("Failed to initialize HID API");
    let devices = hid_api.device_list().collect::<Vec<_>>();

    let pro_ii_devices = devices.iter()
        .filter(|device| device.vendor_id() == devices::VID_RODE)
        .filter(|device| PID_RODECASTER_PRO_II.contains(&device.product_id()))
        .collect::<Vec<_>>();
    if pro_ii_devices.is_empty() {
        println!("No devices found");
        return;
    }
    println!("Found {} devices. Using device with serial {}",
             pro_ii_devices.len(),
             pro_ii_devices[0].serial_number().unwrap_or_default());

    let transport = HidTransport::open(pro_ii_devices[0], &hid_api, HID_REPORT_ID_SEND, HID_REPORT_ID_RECEIVE)
        .expect("Failed to open device");

    println!("Device opened. Sending init payload...");
    transport.write_packet(INIT_PAYLOAD).expect("Failed to send init payload");

    println!("Init payload sent. Reading status packet...");
    let status_packet = transport.read_packet().expect("Failed to read status packet");

    println!("Status packet read ({} bytes). Writing to file {}...", status_packet.len(), OUTPUT_FILE);
    std::fs::write(OUTPUT_FILE, status_packet).expect("Failed to write status packet to file");

    println!("Completed. Re-plug your device.")
}

pub struct HidTransport {
    device: HidDevice,
    report_id_send: u8,
    report_id_recv: u8,
}

impl HidTransport {
    pub fn open(
        device_info: &DeviceInfo,
        hid_api: &HidApi,
        report_id_send: u8,
        report_id_recv: u8,
    ) -> Result<Self, UsbError> {
        let device = device_info.open_device(hid_api)?;

        // todo: it might be possible to run the transport in non-blocking mode
        // we are running the read loop with a 2ms delay anyways
        device.set_blocking_mode(true)?;
        Ok(Self { device, report_id_send, report_id_recv })
    }

    /// Read one fully reassembled packet from the device.
    pub fn read_packet(&self) -> Result<Vec<u8>, UsbError> {
        // read first frame, extract 4-byte LE length prefix
        let mut current_frame = self.read_frame()?;
        if current_frame.len() < 4 {
            return Err(UsbError::FrameParse("First frame too short to contain length prefix".into()));
        }

        let packet_length = u32::from_le_bytes(current_frame[0..4].try_into().unwrap()) as usize;
        let mut packet = Vec::with_capacity(packet_length);

        // first frame starts after 4-byte length prefix
        let mut offset = 4usize;
        while packet.len() < packet_length {
            let available = &current_frame[offset..];
            let take = min(packet_length - packet.len(), available.len());
            packet.extend_from_slice(&available[..take]);

            if packet.len() >= packet_length {
                break;
            }

            current_frame = self.read_frame()?;
            offset = 0; // continuation frames are pure payload
        }

        Ok(packet)
    }

    /// Send raw bytes as one or more framed HID output reports.
    pub fn write_packet(&self, data: &[u8]) -> Result<(), UsbError> {
        let length_prefix = (data.len() as u32).to_le_bytes();
        let payload: Vec<u8> = length_prefix.iter().chain(data.iter()).copied().collect();

        for chunk in payload.chunks(FRAME_PAYLOAD_SIZE) {
            let mut frame = vec![self.report_id_send];
            frame.extend_from_slice(chunk);
            frame.resize(FRAME_PAYLOAD_SIZE + 1, 0); // pad to full frame size
            self.device.send_output_report(&frame)?;
        }
        Ok(())
    }

    fn read_frame(&self) -> Result<Vec<u8>, UsbError> {
        let mut buf = [0u8; FRAME_PAYLOAD_SIZE + 1];
        buf[0] = self.report_id_recv;
        self.device.read(&mut buf)?;
        Ok(buf[1..].to_vec()) // skip report id byte
    }
}