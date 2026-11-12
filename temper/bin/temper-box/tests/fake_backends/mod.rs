//! Child reads and retention, on the existing box/ctl real-loop harness.
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

const SOURCE_PAGE: &[u8] = b"<!doctype html><html><head></head><body>source provisioned fixture</body></html>";

fn private(path: &Path) {
    fs::create_dir_all(path).expect("fixture directory");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("private fixture directory");
}
fn write(path: &Path, content: &[u8]) {
    fs::write(path, content).expect("fixture bytes");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private fixture file");
}
fn bundle(path: &Path, page: &[u8]) {
    private(path);
    private(&path.join("0000000000000001"));
    write(&path.join("index.html"), page);
    write(&path.join("0000000000000001/main.js"), b"// process fixture\n");
}
fn flags(config: &Path, data: &Path, cache: &Path, bundle: Option<&Path>) -> Vec<String> {
    let mut flags = vec![
        "--config".into(),
        config.display().to_string(),
        "--data".into(),
        data.display().to_string(),
        "--cache".into(),
        cache.display().to_string(),
        "--fake-backends".into(),
    ];
    if let Some(bundle) = bundle {
        flags.extend(["--bundle".into(), bundle.display().to_string()]);
    }
    flags
}

#[cfg(target_os = "linux")]
mod newer_build;
mod walk;
