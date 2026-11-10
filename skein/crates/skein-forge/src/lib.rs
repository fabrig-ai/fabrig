//! Neutral forge vocabulary and dialects (forge/README.md).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod api;
pub mod credential;
pub mod decode;
mod directory;
pub mod forgejo;
pub mod github;
pub mod host;
pub mod limits;
pub mod locate;
pub mod measure;
pub mod path;
pub mod plan;
pub mod text;
pub mod time;
pub mod users;

#[cfg(test)]
mod tests;

pub mod marker;
