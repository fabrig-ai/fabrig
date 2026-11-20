//! One probe per scenario on each Linux backend (testing/conformance.md, 1).

use skein_conformance::{Check, reach};
use skein_conformance_real::Real;
use std::os::unix::ffi::OsStrExt;

use super::BACKEND;

fn on<S: Check>(scenario: fn(&mut Real, &[u8]) -> S) {
    scenario(&mut Real::new(BACKEND), skein_scratch::paths::binary("signal_sender_fixture").as_os_str().as_bytes())
        .check();
}

#[test]
fn granted_and_refused() {
    on(reach::granted_and_refused);
}

#[test]
fn no_privilege_gained() {
    on(reach::no_privilege_gained);
}

#[test]
fn signal_to_parent() {
    on(reach::signal_to_parent);
}

#[test]
fn missing_abi() {
    on(reach::missing_abi);
}

#[test]
fn missing_rule() {
    on(reach::missing_rule);
}

#[test]
fn child_narrows() {
    on(reach::child_narrows);
}

#[test]
fn descriptors() {
    on(reach::descriptors);
}
