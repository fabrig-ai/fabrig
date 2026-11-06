//! A client domain world: durable scripted parent, independently typed fake
//! forge, protocol translation, referee observations, and one seeded loop.
mod referee;
pub mod translate;
mod world;
pub use skein_forge_kit::client::LIMITS;
pub use world::{REPO, Settings, Stats, World};
