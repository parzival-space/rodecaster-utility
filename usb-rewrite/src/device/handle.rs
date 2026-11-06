use std::panic::RefUnwindSafe;
use flume::{Receiver, Sender};
use futures::select;
use hidapi::{DeviceInfo, HidApi, HidError};
use log::{debug, warn};
use crate::device::common::RodeCasterDeviceInfo;

pub trait RodeCasterDeviceHandle: RodeCasterDeviceInfo + RefUnwindSafe + Sync + Send {
    // todo: open/coonect - init transport
    // todo: handle message
    // todo: store state
}