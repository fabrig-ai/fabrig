//! A worker's program sources and route (jig/docs/design/protocol/service-parts.md, 1.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
mod generated {
    pub mod channel {
        include!("generated/channel.rs");
    }
}

pub mod route;
pub mod sources;
#[cfg(test)]
mod tests;
