//! Real-clock peer waits (skein/docs/foundation/testing-strategy.md, 2.10).
use skein_world::{Deadline, Expired, Starved};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use temper_forge_setup::{place::Place, spec::Spec};
pub struct Scratch {
    path: PathBuf,
}
impl Scratch {
    #[must_use]
    pub fn new(name: &str) -> Self {
        let path = skein_scratch::paths::target_tmp().join(name);
        if path.exists() {
            fs::remove_dir_all(&path).expect("prior scratch");
        }
        fs::create_dir_all(&path).expect("scratch");
        Self { path }
    }
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
    #[must_use]
    pub fn place(&self) -> Place {
        use std::os::unix::ffi::OsStrExt;
        Place { directory: self.path.as_os_str().as_bytes().into(), user: b"test-user".as_slice().into() }
    }
    pub fn remove(self) {
        fs::remove_dir_all(self.path).expect("scratch removed");
    }
}
#[must_use]
pub fn script(directory: &Path, body: &str) -> Spec {
    use std::os::unix::ffi::OsStrExt;
    let file = directory.join("program");
    fs::write(&file, format!("#!/bin/sh\n{body}\n")).expect("script");
    fs::set_permissions(&file, fs::Permissions::from_mode(0o700)).expect("executable");
    Spec {
        program: file.as_os_str().as_bytes().into(),
        directory: directory.as_os_str().as_bytes().into(),
        arguments: Box::new([]),
        environment: Box::new([b"PATH=/usr/bin:/bin".as_slice().into()]),
        output: 1024,
    }
}
#[must_use]
pub fn io() -> skein_io::Limits {
    jig_protocol_config::profile(jig_protocol_config::Profile::Small).io
}

/// Socket waits use the harness deadline, including the kernel's positive minimum.
#[must_use]
pub fn timeout(deadline: &Deadline) -> std::time::Duration {
    std::time::Duration::from_nanos(deadline.remaining().as_nanos()).max(std::time::Duration::from_millis(1))
}

/// The caller returns this verdict to the test thread after its sockets close.
#[must_use]
pub fn expired(deadline: &Deadline) -> Starved {
    match deadline.expired() {
        Expired::Starved(starved) => starved,
        Expired::Failed(figures) => panic!("the setup peer did not answer: {figures}"),
    }
}
