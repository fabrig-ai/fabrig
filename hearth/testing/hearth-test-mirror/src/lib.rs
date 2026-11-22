//! The pinned local mirror for test tiers (hearth/docs/design/testing/real-builds.md, 2).
pub mod mirror;
pub mod pin;
pub mod proxy;
pub use mirror::{CACHE, Mirror, Missing};
pub use pin::{Pin, PinError, Target, Wrong};
pub use proxy::{HEAD, Method, ORIGIN, Proxy, Served, target};
#[doc(hidden)]
pub mod testing;
#[cfg(test)]
mod tests;
