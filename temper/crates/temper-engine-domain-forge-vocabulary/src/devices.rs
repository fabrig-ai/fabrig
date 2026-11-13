//! A person's push devices and deliveries (temper/docs/design/domain/forge/attention.md, 7.1–7.2).
use alloc::boxed::Box;
use skein_lib::Wall;

pub const DEVICES_SUBJECT: u64 = 1 << 59;
pub const DEVICES: u32 = 8;

/// One need's delivery to a device (attention.md, 7.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Push {
    pub party: u64,
    pub device: u32,
    pub endpoint: Box<[u8]>,
    pub p256dh: [u8; 65],
    pub auth: [u8; 16],
    pub topic: Box<[u8]>,
    pub payload: Box<[u8]>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PushAnswer {
    Made,
    Gone,
    Failed,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Device {
    pub party: u64,
    pub number: u32,
    pub endpoint: Box<[u8]>,
    pub p256dh: [u8; 65],
    pub auth: [u8; 16],
    pub label: Box<[u8]>,
    pub at: Wall,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AddDevice {
    pub endpoint: Box<[u8]>,
    pub p256dh: [u8; 65],
    pub auth: [u8; 16],
    pub label: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DeviceRefusal {
    Person,
    Off,
    Service,
    Malformed,
    TooMany,
    Unknown,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DeviceAnswer {
    Added { number: u32 },
    Removed,
    Refused(DeviceRefusal),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DeviceView {
    pub number: u32,
    pub label: Box<[u8]>,
    pub at: Wall,
    pub endpoint: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DevicesView {
    pub rows: Box<[DeviceView]>,
}
