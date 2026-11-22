//! Scratch configurations and preparation for fake-backend box tests.
use crate::{
    directories::{Directories, make_private},
    fake_backends::{Chosen, Options, Prepared, bytes, configuration, prepare, validate, write},
    path,
};
use std::{
    fs,
    net::TcpListener,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use temper_config_shell::{FILE, Shape};

pub(crate) const PAGE: &[u8] = b"<!doctype html><html><head></head><body>effective fixture</body></html>";
pub struct Scratch(pub PathBuf);
impl Scratch {
    #[must_use]
    pub fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("temper-fake-backends-{}-{name}", std::process::id()));
        fs::create_dir(&path).expect("new private scratch");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("private scratch");
        Self(path)
    }
    #[must_use]
    pub fn dirs(&self) -> Directories {
        Directories {
            config: bytes(&self.0.join("source-config")),
            data: bytes(&self.0.join("source-data")),
            cache: bytes(&self.0.join("cache")),
            origins: [skein_shell::Origin::Flag; 3],
        }
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove private fixture");
    }
}
#[must_use]
pub fn config_path(dirs: &Directories) -> PathBuf {
    path(&dirs.config).join(FILE)
}
#[must_use]
pub fn parsed(path: &Path) -> temper_config_shell::Read {
    let config = path.parent().expect("config root");
    let data = config.with_file_name(if config.ends_with("source-config") { "source-data" } else { "data" });
    let dirs = Directories {
        config: bytes(config),
        data: bytes(&data),
        cache: bytes(&config.with_file_name("cache")),
        origins: [skein_shell::Origin::Flag; 3],
    };
    configuration(&dirs, &fs::read(path).expect("file bytes")).expect("test configuration")
}
pub fn store_file(path: &Path, file: &temper_config_shell::File) {
    write(path, &temper_config_shell::render(file, Shape::Standalone), true).expect("test configuration");
}
#[must_use]
pub fn read_json(path: &Path) -> serde_json::Value {
    let bytes = if path.file_name().is_some_and(|name| name == FILE) {
        parsed(path).document
    } else {
        fs::read(path).expect("fixture bytes").into()
    };
    serde_json::from_slice(&bytes).expect("fixture JSON")
}
pub(crate) fn initialized_source(scratch: &Scratch) -> Directories {
    let source = scratch.dirs();
    let binary = std::env::current_exe().expect("fixture binary");
    // Optional executables are explicitly absent, independent of the test runner's PATH.
    let initialized = crate::init::write(&source, &[], &binary, [0; 2]).expect("source init without PATH");
    crate::close(initialized.listeners.into_iter().map(|(fd, _)| fd).collect());
    for path in initialized.paths {
        if fs::symlink_metadata(&path).expect("fixture metadata").is_file() {
            fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private fixture file");
        }
    }
    source
}
// Each scratch provisions a fake-claude sibling beside its source directories.
pub fn start(source: &Directories, bundle: Option<&Path>) -> Result<Prepared, String> {
    start_chosen(source, bundle, None)
}
pub fn start_chosen(source: &Directories, bundle: Option<&Path>, chosen: Option<&Chosen>) -> Result<Prepared, String> {
    start_options(source, &Options { bundle: bundle.map(Path::to_path_buf), ..Options::default() }, chosen)
}
pub fn start_options(source: &Directories, options: &Options, chosen: Option<&Chosen>) -> Result<Prepared, String> {
    let adapter = path(&source.config).parent().expect("scratch root").join("fake-claude");
    if !adapter.exists() {
        write(&adapter, b"#!/bin/sh\nexit 1\n", false).expect("fixture adapter");
        fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).expect("executable fixture adapter");
    }
    // Use the source fixture's explicit absence of optional executables.
    prepare(source, options, &adapter, chosen, &[])
}
#[must_use]
pub fn provision_bundle(path: &Path) -> PathBuf {
    make_private(&path.join("0000000000000001"), skein_shell::effective_user()).expect("private fixture bundle");
    write(&path.join("index.html"), PAGE, false).expect("fixture page");
    write(&path.join("0000000000000001/main.js"), b"// fixture\n", false).expect("fixture script");
    path.to_path_buf()
}
#[must_use]
pub fn provisioned_source(scratch: &Scratch) -> Directories {
    let source = initialized_source(scratch);
    let bundle = provision_bundle(&scratch.0.join("bundle-loopback"));
    let mut file = parsed(&config_path(&source)).file;
    file.internal.bundle = Some(bundle);
    store_file(&config_path(&source), &file);
    source
}

#[must_use]
pub fn prepare_empty(scratch: &Scratch, source: &Directories) -> Prepared {
    // Init holds clients and webhooks; the four welcome peers each add one.
    const INIT_LISTENERS: usize = 6;
    let supplied = provision_bundle(&scratch.0.join("explicit-bundle"));
    let prepared = start(source, Some(&supplied)).expect("init directly into effective directories");
    let tree = read_json(&config_path(&prepared.directories));
    assert_eq!(tree["application"]["clients"]["bundle"], supplied.to_str().expect("fixture bundle path"));
    assert!(path(&prepared.directories.data).join("worker.secret").is_file());
    assert!(!config_path(source).exists());
    assert_eq!(prepared.listeners.0.len(), INIT_LISTENERS);
    for (_, port) in &prepared.listeners.0 {
        assert!(TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port.get())).is_err(), "listener stays held");
    }
    validate(
        &prepared.directories,
        &fs::read(config_path(&prepared.directories)).expect("effective configuration bytes"),
        &scratch.0.join("fake-claude"),
    )
    .expect("ordinary effective config and agent checks");
    prepared
}

#[must_use]
pub fn prepared_source(scratch: &Scratch) -> (Directories, Prepared) {
    let source = provisioned_source(scratch);
    let prepared = start(&source, None).expect("fresh sandbox");
    (source, prepared)
}
