//! Platform facts are read by the shell, outside the pure page part.
use crate::mapping::Primary;
#[cfg(target_arch = "wasm32")]
mod web;
pub mod worker;
#[cfg(target_arch = "wasm32")]
pub use web::{Browser, clocks, facts};
#[cfg(target_arch = "wasm32")]
pub use worker::{Worker, worker_address};

pub trait Platform {
    fn read_all(&self) -> Vec<crate::store::Record>;
    fn write(&mut self, key: &[u8], value: &[u8]) -> bool;
    fn remove(&mut self, key: &[u8]);
    /// The kept store's records, the browser's local storage (ui/web/README.md, 8); none by default.
    fn read_kept(&self) -> Vec<crate::store::Record> {
        Vec::new()
    }
    /// Writes one kept record; refused by default.
    fn write_kept(&mut self, _key: &[u8], _value: &[u8]) -> bool {
        false
    }
    /// Removes one kept record.
    fn remove_kept(&mut self, _key: &[u8]) {}
    fn send(&mut self, exchange: &crate::fetch::Exchange);
    fn navigate(&mut self, to: &[u8]);
    /// Opens the page's tab holding a line of words, in the turn of a person's act;
    /// false when no tab opened (ui/web/README.md, 8).
    fn open_tab(&mut self, _words: &[u8]) -> bool {
        false
    }
    /// Points the page's tab at an address, its opener cut first, and lets it go (ui/web/README.md, 8).
    fn point_tab(&mut self, _to: &[u8]) {}
    /// Closes the page's tab, if one is held (ui/web/README.md, 8).
    fn close_tab(&mut self) {}
    fn stream(&mut self, duty: crate::transport::StreamDuty);
    /// Abandons an exchange in flight: nothing more is reported of it (ui/web/README.md, 8).
    fn abandon(&mut self, exchange: u32);
    /// Asks the stream's next piece, after the step that took the last (ui/web/README.md, 4.2).
    fn read_on(&mut self);
}

/// Navigation policy of a browser platform (ui/web/README.md, 8).
#[cfg(any(target_arch = "wasm32", test))]
#[derive(Default)]
struct Navigation {
    loopback: bool,
}
#[cfg(any(target_arch = "wasm32", test))]
impl Navigation {
    fn allowed(&self, to: &[u8]) -> bool {
        crate::navigation::allowed(to, self.loopback)
    }
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

#[cfg(test)]
mod tests {
    use super::Navigation;

    #[test]
    fn browser_navigation_requires_application_opt_in_for_loopback() {
        let mut navigation = Navigation::default();
        for target in [b"http://127.0.0.1:80/".as_slice(), b"http://[::1]:80/"] {
            assert!(!navigation.allowed(target));
            navigation.loopback = true;
            assert!(navigation.allowed(target));
            navigation.loopback = false;
            assert!(!navigation.allowed(target));
        }
        assert!(navigation.allowed(b"/welcome"));
        assert!(navigation.allowed(b"https://example.com/"));
    }
}
