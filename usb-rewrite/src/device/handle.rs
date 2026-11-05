use std::panic::RefUnwindSafe;
use crate::device::common::RodeCasterDeviceInfo;

pub trait RodeCasterDeviceHandle: RodeCasterDeviceInfo + RefUnwindSafe + Sync + Send {
    // todo: open/coonect - init transport
    // todo: handle message
    // todo: store state
}

pub(crate) trait RodeCasterDeviceHandleTransport {
    // todo: spawn io thread

    // async fn receive()
}