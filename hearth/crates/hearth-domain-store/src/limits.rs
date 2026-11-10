//! The store's bounds and lock pace (hearth/docs/design/domain/store/README.md, 6.1, 6.2).
use skein_lib::Duration;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub resolutions: u32,
    pub layers: u32,
    pub cache_files: u32,
    pub page: u32,
    pub retry: Duration,
    pub said: Duration,
}
pub const SHIPPED: Limits = Limits {
    resolutions: 1_024,
    layers: 4_096,
    cache_files: 262_144,
    page: 1_024,
    retry: Duration::from_millis(200),
    said: Duration::from_secs(1),
};
