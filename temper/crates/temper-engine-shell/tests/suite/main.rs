//! Stopped deployment commands on the real kernel (protocol/checkpoints.md, 2).
#![cfg(test)]
use std::{os::unix::ffi::OsStrExt, path::Path};
use temper_engine_shell as shell;

use temper_deployment_fixtures as deployment;

mod checkpoint;
mod fixture;
mod restore;
mod upgrade;

fn path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}
