//! macOS refuses a reach whole (io/processes.md, 6).

use skein_conformance::{Check, reach};
use skein_conformance_real::Real;
use std::os::unix::ffi::OsStrExt;

#[test]
fn reached_spawn_is_unsupported() {
    reach::unsupported(
        &mut Real::new(super::BACKEND),
        skein_scratch::paths::binary("signal_sender_fixture").as_os_str().as_bytes(),
    )
    .check();
}
