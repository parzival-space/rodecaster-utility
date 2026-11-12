use hidapi::{DeviceInfo, HidApi, HidDevice, HidError};
use std::time::{Duration, Instant};
use log::trace;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum RodeCasterHidTransportError {
    #[error(transparent)]
    HidError(#[from] HidError),

    #[error("The operation has timed out")]
    TimeoutError,

    #[error("The received packet size '{0}' exceeds the maximum allowed message size")]
    PayloadTooLargeError(usize),
}

#[derive(Debug)]
pub(crate) struct RodeCasterHidTransport {
    handle: HidDevice,

    tx_report_id: u8,
    rx_report_id: u8,
}

impl RodeCasterHidTransport {
    const MAX_PAYLOAD_LEN: usize = 256 * 1024 * 1024; // 256 MiB
    const FRAME_SIZE: usize = 256; // 1 report byte + 255 data bytes

    pub(crate) fn new(device_info: DeviceInfo, tx_report_id: u8, rx_report_id: u8) -> Result<Self, HidError> {
        let hid_api = HidApi::new()?;

        let handle = device_info.open_device(&hid_api)?;
        handle.set_blocking_mode(true)?; // shouldn't be necessary, but just in case, we don't even use read()

        Ok(Self {
            handle,
            tx_report_id,
            rx_report_id,
        })
    }

    pub(crate) fn read_timeout(&self, timeout: Duration) -> Result<Vec<u8>, RodeCasterHidTransportError> {
        let mut packet: Vec<u8> = Vec::new();

        let mut buffer = vec![0u8; Self::FRAME_SIZE];
        buffer[0] = self.rx_report_id;

        let start = Instant::now();
        while start.elapsed() < timeout {
            match self.handle.read_timeout(&mut buffer, timeout.as_millis() as i32) {
                Ok(bytes_read) => match bytes_read {
                    _ if bytes_read == 0 => { /* no message was received yet */ }
                    _ => {
                        packet.extend_from_slice(&buffer[1..bytes_read]);
                        buffer[1..].fill(0);

                        let Some(payload_length) = packet
                            .get(..4)
                            .and_then(|value| value.try_into().ok())
                            .map(u32::from_le_bytes)
                            .map(|value| value as usize)
                        else {
                            // not enough bytes to parse length yet or parsing failed
                            continue;
                        };

                        if payload_length > Self::MAX_PAYLOAD_LEN {
                            return Err(RodeCasterHidTransportError::PayloadTooLargeError(payload_length));
                        }

                        let full_packet_length = payload_length + 4;
                        if packet.len() < full_packet_length {
                            // still missing some bytes
                            continue;
                        }

                        return Ok(packet[4..full_packet_length].to_vec());
                    }
                },
                Err(err) => {
                    return Err(RodeCasterHidTransportError::HidError(err));
                }
            }
        }

        // timeout was reached
        Err(RodeCasterHidTransportError::TimeoutError)
    }

    pub(crate) fn write_timeout(&self, buffer: &[u8], timeout: Duration) -> Result<(), RodeCasterHidTransportError> {
        if buffer.len() > Self::MAX_PAYLOAD_LEN {
            return Err(RodeCasterHidTransportError::PayloadTooLargeError(buffer.len()));
        }

        let length_bytes = (buffer.len() as u32).to_le_bytes();
        let packet: Vec<u8> = length_bytes.iter().chain(buffer.iter()).copied().collect();

        let start = Instant::now();
        for chunk in packet.chunks(Self::FRAME_SIZE - 1) {
            if start.elapsed() > timeout {
                // operation took too long
                return Err(RodeCasterHidTransportError::TimeoutError)
            }

            let mut frame = vec![self.tx_report_id];
            frame.extend_from_slice(chunk);
            frame.resize(Self::FRAME_SIZE, 0); // pad to full frame size

            self.handle.send_output_report(&frame)?;
        }

        // message was send
        Ok(())
    }
}