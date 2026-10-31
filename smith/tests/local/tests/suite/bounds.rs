//! Iteration bounds for the world's waiting, delivery, pressure and teardown paths.
pub(crate) const SCHEDULE_SLICE: u32 = 5;
pub(crate) const CANCELLATION: u32 = 10;
pub(crate) const PRESSURE: u32 = 20;
pub(crate) const CHECKPOINT: u32 = 100;
pub(crate) const REPLY: u32 = 200;
pub(crate) const CHAT: u32 = 300;
pub(crate) const TOOL: u32 = 400;
pub(crate) const DELIVERY: u32 = 500;
pub(crate) const RECOVERY: u32 = 600;
pub(crate) const DRAIN: u32 = 2_000;
pub(crate) const SCHEDULED_DRAIN: u32 = 4_000;
