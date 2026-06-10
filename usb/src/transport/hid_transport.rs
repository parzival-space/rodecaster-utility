use std::cmp::min;
use hidapi::{DeviceInfo, HidApi, HidDevice};
use crate::error::UsbError;

const FRAME_PAYLOAD_SIZE: usize = 255; // 256 byte hid report, minus 1 report id byte

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