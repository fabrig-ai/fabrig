//! Platform facts are read by the shell, outside the pure page part.
use crate::mapping::Primary;
#[cfg(target_arch = "wasm32")]
mod web;
pub mod worker;
#[cfg(target_arch = "wasm32")]
pub use web::{Browser, clocks, facts};

pub trait Platform {
    fn read_all(&self) -> Vec<crate::store::Record>;
    fn write(&mut self, key: &[u8], value: &[u8]) -> bool;
    fn remove(&mut self, key: &[u8]);
    fn send(&mut self, exchange: &crate::fetch::Exchange);
    fn navigate(&mut self, to: &[u8]);
    fn stream(&mut self, duty: crate::transport::StreamDuty);
    /// Abandons an exchange in flight: nothing more is reported of it (ui-web.md, 10).
    fn abandon(&mut self, exchange: u32);
    /// Asks the stream's next piece, after the step that took the last (ui-web.md, 5.2).
    fn read_on(&mut self);
}

pub struct Facts {
    pub now: skein_lib::Time,
    pub wall: skein_lib::Wall,
    pub seed: u64,
    pub offset: i32,
    pub languages: Box<[u8]>,
    pub primary: Primary,
}
#[must_use]
pub fn primary(platform: Option<&str>) -> Primary {
    if platform
        .is_some_and(|platform| ["Mac", "iPhone", "iPad", "iPod"].iter().any(|prefix| platform.starts_with(prefix)))
    {
        Primary::Meta
    } else {
        Primary::Control
    }
}
