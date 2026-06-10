use std::thread::sleep;
use std::time::Duration;
use crossbeam::channel::{Receiver, Sender, TryRecvError};
use hidapi::{DeviceInfo, HidApi};
use log::trace;
use crate::devices::rodecaster_pro_ii::handle::{RodeCasterProIICommand, RodeCasterProIIEvent};
use crate::error::UsbError;
use crate::protocol::packets::{parse_packet, Packet};
use crate::transport::HidTransport;

const HID_REPORT_ID_SEND: u8 = 0x03;
const HID_REPORT_ID_RECEIVE: u8 = 0x04;

// the device expects the host to send 4 magic bytes to initialize further communication
const INIT_PAYLOAD: &[u8] = &[0xAD, 0x10, 0xA7, 0xB0];

pub fn run_io_loop(
    device_info: DeviceInfo,
    cmd_rx: Receiver<RodeCasterProIICommand>,
    event_tx: Sender<RodeCasterProIIEvent>,
) -> Result<(), UsbError> {
    let hid_api = HidApi::new()?;
    let transport = HidTransport::open(&device_info, &hid_api, HID_REPORT_ID_SEND, HID_REPORT_ID_RECEIVE)?;

    transport.write_packet(INIT_PAYLOAD)?;
    event_tx.send(RodeCasterProIIEvent::Connected).unwrap_or_default();

    loop {
        match cmd_rx.try_recv() {
            Ok(RodeCasterProIICommand::Stop) => break,
            Ok(RodeCasterProIICommand::SendPacket(data)) => transport.write_packet(&data)?,
            Err(TryRecvError::Disconnected) => break,
            Err(TryRecvError::Empty) => {},
        }

        let raw_packet = transport.read_packet()?;
        if raw_packet.is_empty() {
            sleep(Duration::from_millis(2));
            continue;
        }
        
        match parse_packet(&*raw_packet) {
            Ok((_, Packet::PropertyPatch(packet))) => 
                event_tx.send(RodeCasterProIIEvent::PropertyPatchReceived(packet)).unwrap_or_default(),
            Ok((_, Packet::DeviceStatus(packet))) =>
                event_tx.send(RodeCasterProIIEvent::DeviceReportReceived(packet)).unwrap_or_default(),
            Ok((_, Packet::Unknown(packet))) => {
                trace!("Received unknown packet: {:02X?}", packet);
                event_tx.send(RodeCasterProIIEvent::UnknownPacket(packet)).unwrap_or_default();
            }
            Err(err) =>
                event_tx.send(RodeCasterProIIEvent::Error(format!("Failed to parse packet: {:?}", err))).unwrap_or_default(),
        }
    }

    event_tx.send(RodeCasterProIIEvent::Disconnect).unwrap_or_default();
    Ok(())
}