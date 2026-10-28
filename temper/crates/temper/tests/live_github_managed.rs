//! The owner-run sitting's private relay. Never runs in a default test invocation.
#![expect(dead_code, reason = "the existing relay also serves other test binaries")]
#[path = "support/managed_client.rs"]
mod managed_client;
#[path = "support/relay.rs"]
mod relay;
#[path = "support/relay_managed.rs"]
mod relay_managed;

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
#[ignore = "owner-run only, through scripts/proof-live-github-managed.sh"]
fn managed_relay() {
    relay_managed::run().expect("managed relay failed (details deliberately suppressed)");
}

#[test]
#[ignore = "fake sitting only, through scripts/proof-live-github-managed.sh"]
fn managed_ready_client() {
    managed_client::run();
}
