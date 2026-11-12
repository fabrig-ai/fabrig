//! The managed sitting's Ready on the fake (temper/docs/design/testing/process-tier.md)
//! and its owner-run relay against github.com (temper/docs/design/testing/live-tier.md).
#![expect(dead_code, reason = "the existing relay also serves other test binaries")]
#[path = "support/managed_client.rs"]
mod managed_client;
#[path = "support/managed_sitting.rs"]
mod managed_sitting;
use temper_box_kit::patience;
use temper_box_kit::relay;
#[path = "support/relay_managed.rs"]
mod relay_managed;
use temper_box_kit::socket;

fn write_private(path: &std::path::Path, bytes: &[u8]) {
    use std::{io::Write, os::unix::fs::OpenOptionsExt};
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .expect("private sitting file")
        .write_all(bytes)
        .expect("private sitting bytes");
}

#[test]
#[ignore = "live: github.com, where a person makes and installs the App; scripts/proof-live-github-managed.sh live"]
fn managed_relay() {
    relay_managed::run().expect("managed relay failed (details deliberately suppressed)");
}

/// Runs the fake sitting, or its client with the sitting's mode and stdin link
/// when `TEMPER_TEST_MANAGED_CLIENT` is set (temper/docs/design/testing/process-tier.md).
#[test]
fn managed_ready_client() {
    match managed_client::role() {
        Some(mode) => managed_client::run(mode),
        None => managed_sitting::run(),
    }
}
