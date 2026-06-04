use log::debug;
use crate::protocol::packet::PropertyUpdatePacket;
use crate::protocol::types::Structured;

#[derive(Debug, Clone, Default)]
pub struct RodeCasterProIIState {
    pub root: Option<Structured>, // todo replace with domain object
}

impl RodeCasterProIIState {
    pub fn apply_device_report(&mut self, report: Structured) {
        debug!("Applying RodeCasterProII report: {:?}", report);
        self.root = Some(report);
    }
    
    pub fn apply_property_update(&mut self, update: PropertyUpdatePacket) {
        if let Some(root) = self.root.as_mut() {
            debug!("Applying RodeCasterProII property update: {:?}", update);
            root.set_property(update.indices, update.name, update.value).unwrap_or_default();
        }
    }
}