use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::{Receiver, Sender, TryRecvError};
use hidapi::{DeviceInfo, HidApi};
use log::trace;
use crate::error::UsbError;
use crate::protocol::packets::device_status_packet::DeviceStatusPacket;
use crate::protocol::packets::{parse_packet, Packet};
use crate::protocol::packets::handshake_packet::HandshakePacket;
use crate::protocol::packets::property_patch_packet::PropertyPatchPacket;
use crate::protocol::Parseable;
use crate::transport::HidTransport;

const HID_REPORT_ID_SEND: u8 = 0x03;

const HID_REPORT_ID_RECEIVE: u8 = 0x04;

pub(crate) enum IoRxEvent {
    DeviceReportReceived(DeviceStatusPacket),
    PropertyPatchReceived(PropertyPatchPacket),
    UnknownPacket(Vec<u8>),
    Error(String),
    Disconnect,
}

pub(crate) enum IoTxEvent {
    SendHandshake,
    SendPropertyPatch(PropertyPatchPacket),
    Close,
}

pub(crate) fn run_rx_loop(device_info: DeviceInfo, sender: Sender<IoRxEvent>, shutdown_requested: Arc<AtomicCell<bool>>) -> Result<(), UsbError> {
    let hid_api = HidApi::new()?;
    let transport = HidTransport::open(&device_info, &hid_api, HID_REPORT_ID_RECEIVE)?;

    loop {
        if shutdown_requested.load() {
            break;
        }

        let raw_packet = transport.read_packet()?;
        if raw_packet.is_empty() {
            sleep(Duration::from_millis(2));
            continue;
        }

        match parse_packet(&*raw_packet) {
            Ok((_, Packet::PropertyPatch(packet))) =>
                sender.send(IoRxEvent::PropertyPatchReceived(packet)).unwrap_or_default(),
            Ok((_, Packet::DeviceStatus(packet))) =>
                sender.send(IoRxEvent::DeviceReportReceived(packet)).unwrap_or_default(),
            Ok((_, Packet::Unknown(packet))) => {
                trace!("Received unknown packet: {:02X?}", packet);
                sender.send(IoRxEvent::UnknownPacket(packet)).unwrap_or_default();
            }
            Ok((_, Packet::Handshake(_))) => {
                // this case should never happen
                sender.send(IoRxEvent::Error("Received Handshake packet".to_string())).unwrap_or_default();
            },
            Err(err) =>
                sender.send(IoRxEvent::Error(format!("Failed to parse packet: {:?}", err))).unwrap_or_default(),
        }
    }

    Ok(())
}

pub(crate) fn run_tx_loop(device_info: DeviceInfo, receiver: Receiver<IoTxEvent>) -> Result<(), UsbError> {
    let hid_api = HidApi::new()?;
    let transport = HidTransport::open(&device_info, &hid_api, HID_REPORT_ID_SEND)?;

    loop {
        match receiver.try_recv() {
            Ok(IoTxEvent::Close) => break,
            Ok(IoTxEvent::SendHandshake) => {
                let mut packet = Vec::new();
                HandshakePacket::default().write_to(&mut packet)?;
                transport.write_packet(&*packet).unwrap_or_default()
            }
            Ok(IoTxEvent::SendPropertyPatch(patch_packet)) => {
                let mut packet = Vec::new();
                patch_packet.write_to(&mut packet)?;
                transport.write_packet(&*packet).unwrap_or_default()
            }
            Err(TryRecvError::Empty) => {
                sleep(Duration::from_millis(2));
                continue;
            }
            Err(TryRecvError::Disconnected) => break,
        }
    }

    Ok(())
}