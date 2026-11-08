//! The engine's link at rest (temper/docs/design/client/README.md, 6.12).
use std::{
    fmt::Write as _,
    fs,
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

pub const MOST: u32 = 4096;

/// The deployment's link: Linux's absolute runtime, or macOS's data directory.
#[must_use]
pub fn path(data: &[u8], runtime: Option<&[u8]>, linux: bool) -> Option<PathBuf> {
    if !linux {
        return Some(crate::path(data).join("printed-link"));
    }
    let runtime = crate::path(runtime?);
    if !runtime.is_absolute() {
        return None;
    }
    let digest = skein_lib::sha256::digest(data);
    let mut key = String::with_capacity(16);
    for byte in digest.iter().take(8) {
        write!(&mut key, "{byte:02x}").expect("hexadecimal key");
    }
    Some(runtime.join("temper").join(format!("printed-link-{key}")))
}

/// The deployment that `temper box` resolves without flags.
#[must_use]
#[expect(clippy::type_complexity, reason = "the shared link-file contract takes environment entries as byte pairs")]
pub fn deployment(environment: &[(Box<[u8]>, Box<[u8]>)], working: &[u8]) -> Option<Box<[u8]>> {
    crate::directories::resolve(&[], environment, working).ok().map(|(directories, _)| directories.data)
}

/// Replace the complete link through a synced private temporary in its directory.
pub fn keep(path: &Path, url: &[u8], user: u32) -> Result<(), String> {
    let result = (|| {
        let parent = path.parent().ok_or_else(|| "no parent directory".to_string())?;
        crate::directories::make_private(parent, user)?;
        let seed = skein_shell::crypto_seed().map_err(|errno| format!("system randomness: errno {errno}"))?;
        let mut suffix = String::with_capacity(64);
        for byte in seed.expose() {
            write!(&mut suffix, "{byte:02x}").expect("hexadecimal temporary name");
        }
        let temporary = parent.join(format!(".printed-link-{suffix}.partial"));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)
            .map_err(|why| why.to_string())?;
        let result = file.write_all(url).and_then(|()| file.write_all(b"\n")).and_then(|()| file.sync_all());
        drop(file);
        let result = result.and_then(|()| fs::rename(&temporary, path));
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result.map_err(|why| why.to_string())
    })();
    result.map_err(|why| format!("{}: {why}", path.display()))
}

/// Absence and removal errors are quiet.
pub fn forget(path: &Path) {
    let _ = fs::remove_file(path);
}

/// Refusals read as none; only one final line feed is removed.
#[must_use]
pub fn read(path: &Path, user: u32) -> Option<skein_lib::Secret> {
    let value = skein_shell::read_private_file(path, MOST, user).ok()?;
    let bytes = value.expose();
    Some(skein_lib::Secret::new(bytes.strip_suffix(b"\n").unwrap_or(bytes).into()))
}

#[cfg(test)]
mod tests;
