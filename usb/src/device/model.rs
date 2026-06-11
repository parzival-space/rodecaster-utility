use hidapi::DeviceInfo;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DeviceModel {
    ProII,
    DUMMY,
    // the Duo and StreamerX share the same firmware, so they should also work
}

impl DeviceModel {
    pub fn capabilities(&self) -> &'static ModelCapabilities {
        match self {
            DeviceModel::ProII => &PRO_II,
            DeviceModel::DUMMY => &DUMMY,
        }
    }

    pub(crate) fn from_device_info(device_info: DeviceInfo) -> Option<Self> {
        for model in
            [DeviceModel::ProII, DeviceModel::DUMMY]
        {
            if model.capabilities().vendor_id == device_info.vendor_id() &&
                model.capabilities().product_ids.contains(&device_info.product_id()){
                return Some(model);
            }
        }
        None
    }
}

pub struct ModelCapabilities {
    name: &'static str,
    vendor_id: u16,
    product_ids: &'static [u16],
}

static PRO_II: ModelCapabilities = ModelCapabilities {
    name: "RØDECaster Pro II",
    vendor_id: 0x19f7,
    product_ids: &[0x0037, 0x0072, 0x0078, 0x0030, 0x0094, 0x0092],
};

static DUMMY: ModelCapabilities = ModelCapabilities {
    name: "Dummy",
    vendor_id: 0x19f7,
    product_ids: &[],
};

// store rodecaster duo pids, vids and model specific stuff
// store rodecaster streamer x pids, vids and model specific stuff