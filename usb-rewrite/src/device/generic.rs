use crate::device::common::{RodeCasterDeviceInfo, RodeCasterDeviceInternal, RodeCasterDeviceKind};
use crate::device::io::{RodeCasterDeviceIOCommand, RodeCasterDeviceIOEvent, RodeCasterDeviceIOHandler};
use hidapi::{DeviceInfo, HidError};
use std::marker::PhantomData;
use std::panic::RefUnwindSafe;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use flume::{Receiver, Sender};
use crate::device::protocol::{RodeCasterDeviceProtocolHandler, RodeCasterDeviceProtocolHandlerCommand, RodeCasterDeviceProtocolHandlerEvent};

/// A generic device implementation for a RodeCaster protocol compatible devices.
#[derive(Debug)]
pub(crate) struct RodeCasterDevice<K: RodeCasterDeviceKind> {
    pid: u16,
    serial: String,
    firmware_version: String,

    protocol_sender: Sender<RodeCasterDeviceProtocolHandlerCommand>,
    protocol_receiver: Receiver<RodeCasterDeviceProtocolHandlerEvent>,
    
    io_sender: Sender<RodeCasterDeviceIOCommand>,
    io_receiver: Receiver<RodeCasterDeviceIOEvent>,

    _kind: PhantomData<K>,
}

impl<K: RodeCasterDeviceKind + RefUnwindSafe> RodeCasterDeviceInfo for RodeCasterDevice<K> {
    fn get_product_id(&self) -> u16 {
        self.pid
    }

    fn get_serial(&self) -> String {
        self.serial.clone()
    }

    fn get_version(&self) -> String {
        self.firmware_version.clone()
    }
}

impl<K: RodeCasterDeviceKind + RefUnwindSafe> RodeCasterDeviceInternal for RodeCasterDevice<K> {
    async fn connect(device: DeviceInfo) -> Result<Arc<Box<dyn RodeCasterDeviceHandle>>, HidError> {
        let (io_event_sender, io_event_receiver) = flume::bounded(64);
        let (io_command_sender, io_command_receiver) = flume::bounded(64);
        Self::spawn_io_handler(device.clone(), io_event_sender, io_command_receiver);
        
        let (protocol_event_sender, protocol_event_receiver) = flume::bounded(64);
        let (protocol_command_sender, protocol_command_receiver) = flume::bounded(64);
        Self::spawn_protocol_handler(
            device.clone(),
            protocol_event_sender,
            protocol_command_receiver,
            io_command_sender.clone(),
            io_event_receiver.clone(),
        );

        Ok(Arc::new(Box::new(RodeCasterDevice {
            pid: device.product_id(),
            serial: device.serial_number().unwrap_or_default().to_string(),
            firmware_version: String::new(),
            protocol_sender: protocol_command_sender,
            protocol_receiver: protocol_event_receiver,
            io_sender: io_command_sender,
            io_receiver: io_event_receiver,
            _kind: PhantomData::<K>,
        })))
    }
}
impl<K: RodeCasterDeviceKind + RefUnwindSafe> RodeCasterDeviceIOHandler for RodeCasterDevice<K> {}

impl<K: RodeCasterDeviceKind> Drop for RodeCasterDevice<K> {
    fn drop(&mut self) {
        let _ = self.io_sender.send(RodeCasterDeviceIOCommand::QuitThread);
    }
}

impl<K: RodeCasterDeviceKind + RefUnwindSafe> RodeCasterDeviceHandle for RodeCasterDevice<K> {}

impl<K: RodeCasterDeviceKind + RefUnwindSafe> RodeCasterDeviceProtocolHandler for RodeCasterDevice<K> {}

pub trait RodeCasterDeviceHandle: RodeCasterDeviceInfo + RefUnwindSafe + Sync + Send {}