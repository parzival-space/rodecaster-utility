pub mod rodecaster_pro_ii;
pub mod dummy_device;
pub mod open;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceType {
    DummyDevice,
    RodeCasterProII,
}