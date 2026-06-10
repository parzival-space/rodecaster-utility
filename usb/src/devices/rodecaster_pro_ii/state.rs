use log::{debug, trace};
use crate::protocol::packets::device_status_packet::DeviceStatusPacket;
use crate::protocol::packets::property_patch_packet::PropertyPatchPacket;
use crate::protocol::types::composite::Composite;

#[derive(Debug, Clone, Default)]
pub struct RodeCasterProIIState {
    pub root: Option<Composite>, // todo replace with domain object
}

impl RodeCasterProIIState {
    pub(crate) fn apply_device_report(&mut self, report: DeviceStatusPacket) {
        debug!("Applying RodeCasterProII report: {:?}", report);
        self.root = Some(report.get_status().clone());
    }

    pub(crate) fn apply_property_update(&mut self, update: PropertyPatchPacket) {
        if let Some(root) = self.root.as_mut() {
            trace!("Applying RodeCasterProII property update: {:?}", update);
            root.apply_patch(update.get_indices().clone(), update.get_name().clone(), update.get_value().clone()).unwrap_or_default();
        }
    }
}