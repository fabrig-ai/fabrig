//! The engine's link at rest (temper/docs/design/client/tui/directory.md, 1).
use std::{
    fmt::Write as _,
    fs,
    io::{self, Write},
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
    keep_with(path, url, user, &mut Native)
}

/// Link publication's effects, in the startup shell (temper/docs/design/testing/shells.md, 3.12).
trait Publication {
    type File;

    fn prepare(&mut self, path: &Path, user: u32) -> Result<(), String>;
    fn seed(&mut self) -> Result<skein_lib::Secret, i32>;
    fn create_new(&mut self, path: &Path, mode: u32) -> io::Result<Self::File>;
    fn write_all(&mut self, file: &mut Self::File, bytes: &[u8]) -> io::Result<()>;
    fn sync(&mut self, file: &Self::File) -> io::Result<()>;
    fn close(&mut self, file: Self::File);
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()>;
    fn remove(&mut self, path: &Path) -> io::Result<()>;
}

struct Native;

impl Publication for Native {
    type File = fs::File;

    fn prepare(&mut self, path: &Path, user: u32) -> Result<(), String> {
        crate::directories::make_private(path, user)
    }
    fn seed(&mut self) -> Result<skein_lib::Secret, i32> {
        skein_shell::crypto_seed()
    }
    fn create_new(&mut self, path: &Path, mode: u32) -> io::Result<Self::File> {
        fs::OpenOptions::new().write(true).create_new(true).mode(mode).open(path)
    }
    fn write_all(&mut self, file: &mut Self::File, bytes: &[u8]) -> io::Result<()> {
        file.write_all(bytes)
    }
    fn sync(&mut self, file: &Self::File) -> io::Result<()> {
        file.sync_all()
    }
    fn close(&mut self, file: Self::File) {
        drop(file);
    }
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }
    fn remove(&mut self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }
}

fn keep_with(path: &Path, url: &[u8], user: u32, disk: &mut impl Publication) -> Result<(), String> {
    let result = (|| {
        let parent = path.parent().ok_or_else(|| "no parent directory".to_string())?;
        disk.prepare(parent, user)?;
        let seed = disk.seed().map_err(|errno| format!("system randomness: errno {errno}"))?;
        let mut suffix = String::with_capacity(64);
        for byte in seed.expose() {
            write!(&mut suffix, "{byte:02x}").expect("hexadecimal temporary name");
        }
        let temporary = parent.join(format!(".printed-link-{suffix}.partial"));
        let mut file = disk.create_new(&temporary, 0o600).map_err(|why| why.to_string())?;
        let result = disk
            .write_all(&mut file, url)
            .and_then(|()| disk.write_all(&mut file, b"\n"))
            .and_then(|()| disk.sync(&file));
        disk.close(file);
        let result = result.and_then(|()| disk.rename(&temporary, path));
        if result.is_err() {
            let _ = disk.remove(&temporary);
        }
        result.map_err(|why| why.to_string())
    })();
    result.map_err(|why| format!("{}: {why}", path.display()))
}

/// Absence and removal errors are quiet.
pub fn forget(path: &Path) {
    forget_with(path, &mut Native);
}

fn forget_with(path: &Path, disk: &mut impl Publication) {
    let _ = disk.remove(path);
}

/// Refusals read as none; only one final line feed is removed.
#[must_use]
pub fn read(path: &Path, user: u32) -> Option<skein_lib::Secret> {
    read_with(path, user, skein_shell::read_private_file)
}

fn read_with(
    path: &Path,
    user: u32,
    read: impl FnOnce(&Path, u32, u32) -> Result<skein_lib::Secret, skein_shell::Unread>,
) -> Option<skein_lib::Secret> {
    let value = read(path, MOST, user).ok()?;
    let bytes = value.expose();
    Some(skein_lib::Secret::new(bytes.strip_suffix(b"\n").unwrap_or(bytes).into()))
}

#[cfg(test)]
mod tests;
