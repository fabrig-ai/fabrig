//! Scratch directories and executable lookup fixtures for init.
use crate::directories::Directories;
use std::{
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::{Path, PathBuf},
};
use temper_shell::directories::EnvironmentEntry;

/// Init's retained configuration for two supplied listener ports, without reserving either.
#[must_use]
pub fn configuration(clients: std::num::NonZeroU16, webhooks: std::num::NonZeroU16) -> temper_config_shell::File {
    crate::init::configuration(clients, webhooks)
}

#[must_use]
pub fn scratch(row: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("temper-init-{}-{row}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("prior scratch");
    }
    fs::create_dir(&root).expect("scratch");
    root
}
#[must_use]
pub fn directories(root: &Path) -> Directories {
    let paths = [root.join("config"), root.join("data"), root.join("cache")];
    let args = [
        b"--config".as_slice(),
        paths[0].as_os_str().as_bytes(),
        b"--data",
        paths[1].as_os_str().as_bytes(),
        b"--cache",
        paths[2].as_os_str().as_bytes(),
    ];
    crate::directories::resolve(&args.iter().map(|v| Box::from(*v)).collect::<Vec<_>>(), &[], b"/")
        .expect("explicit directories")
        .0
}
#[must_use]
pub fn environment(root: &Path, claude: bool) -> Vec<EnvironmentEntry> {
    fs::create_dir(root.join("bin")).expect("bin");
    for name in if claude { vec!["git", "claude"] } else { vec!["git"] } {
        let path = root.join("bin").join(name);
        fs::write(&path, b"#!/bin/sh\nexit 0\n").expect("program");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("executable");
    }
    vec![(Box::from(b"PATH".as_slice()), root.join("bin").as_os_str().as_bytes().into())]
}
