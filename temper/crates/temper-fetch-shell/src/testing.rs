//! Cache test fixtures and native effects (tools/crates/design.md, 3.2).
use skein_lib::sha256::Sha256;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    path::{Path, PathBuf},
};

pub use crate::box_binary::testing::{Files, cached, identity, suffix};

#[must_use]
pub fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("temper-fetch-box-{name}-{}", std::process::id()));
    match fs::remove_dir_all(&path) {
        Ok(()) | Err(_) => {}
    }
    fs::DirBuilder::new().mode(0o700).create(&path).expect("private scratch directory");
    path
}
pub fn script(file: &Path, body: &[u8]) {
    fs::write(file, body).expect("write the fixture script");
    fs::set_permissions(file, fs::Permissions::from_mode(0o755)).expect("make the fixture script executable");
}
#[must_use]
pub fn digest(bytes: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(bytes);
    hash.finish()
}

#[must_use]
pub fn zstd(bytes: &[u8]) -> Vec<u8> {
    ruzstd::encoding::compress_to_vec(bytes, ruzstd::encoding::CompressionLevel::Fastest)
}

#[must_use]
pub fn site() -> temper_fetch::Origin {
    temper_fetch::Origin { host: b"site.test".as_slice().into(), port: 443 }
}
