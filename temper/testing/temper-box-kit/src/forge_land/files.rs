//! File admission and reservation (temper/docs/design/testing/box/README.md, 3.16).
use super::{LiveModel, require};
use std::{
    ffi::OsString,
    fs, io,
    os::unix::fs::{DirBuilderExt, MetadataExt},
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    File,
    Directory,
    Symlink,
    Other,
}

impl From<fs::FileType> for FileKind {
    fn from(kind: fs::FileType) -> Self {
        if kind.is_file() {
            Self::File
        } else if kind.is_dir() {
            Self::Directory
        } else if kind.is_symlink() {
            Self::Symlink
        } else {
            Self::Other
        }
    }
}

/// A no-follow stat, with only the fields the landing decisions read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileFacts {
    pub kind: FileKind,
    pub mode: u32,
    pub uid: u32,
    pub len: u64,
}

fn stat(path: &Path) -> io::Result<FileFacts> {
    let metadata = fs::symlink_metadata(path)?;
    Ok(FileFacts { kind: metadata.file_type().into(), mode: metadata.mode(), uid: metadata.uid(), len: metadata.len() })
}

/// Admit the held `ChatGPT` record a fresh store's first run moves in: a
/// regular file only its owner reads, as ctl account import's FILE must be. Its words
/// never name the path, which goes to `account import` alone.
pub fn held_account(file: &Path) -> Result<(), &'static str> {
    held_account_with(file, stat)
}

pub fn held_account_with(file: &Path, stat: impl FnOnce(&Path) -> io::Result<FileFacts>) -> Result<(), &'static str> {
    let metadata = stat(file).map_err(|_| "TEMPER_TEST_LIVE_ACCOUNT names no file")?;
    if metadata.kind != FileKind::File {
        return Err("TEMPER_TEST_LIVE_ACCOUNT is not a regular file");
    }
    if metadata.mode & 0o7777 != 0o600 {
        return Err("TEMPER_TEST_LIVE_ACCOUNT is not 0600");
    }
    Ok(())
}

/// Admit the key by metadata alone; refusal words never include its path.
/// Unset, the caller skips before reserving a store or starting a process.
pub fn anthropic_key(value: Option<OsString>) -> Result<Option<PathBuf>, &'static str> {
    anthropic_key_with(value, stat, skein_shell::effective_user)
}

pub fn anthropic_key_with(
    value: Option<OsString>,
    stat: impl FnOnce(&Path) -> io::Result<FileFacts>,
    effective_user: impl FnOnce() -> u32,
) -> Result<Option<PathBuf>, &'static str> {
    let Some(value) = value else { return Ok(None) };
    let path = PathBuf::from(value);
    let metadata = stat(&path).map_err(|_| "TEMPER_TEST_LIVE_ANTHROPIC_KEY names no file")?;
    if metadata.kind != FileKind::File {
        return Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is not a regular file or is a link");
    }
    if metadata.mode & 0o7777 != 0o600 {
        return Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is not 0600");
    }
    if metadata.uid != effective_user() {
        return Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is not owned by this user");
    }
    if metadata.len == 0 {
        return Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is empty");
    }
    Ok(Some(path))
}

fn create(path: &Path, mode: u32, recursive: bool) -> io::Result<()> {
    fs::DirBuilder::new().mode(mode).recursive(recursive).create(path)
}

fn first_entry(path: &Path) -> io::Result<bool> {
    let mut entries = fs::read_dir(path)?;
    let entry = entries.next().transpose()?;
    Ok(entry.is_some())
}

/// Make the stores' directory (0700, with its parents) when absent, then reserve
/// `stores/<story>` as `reserve_live_store` does; a held `ChatGPT` record moves into a fresh
/// store only.
pub fn story_store(stores: &Path, model: LiveModel<'_>) -> Result<(PathBuf, bool), String> {
    story_store_with(stores, model, create, stat, first_entry)
}

pub fn story_store_with(
    stores: &Path,
    model: LiveModel<'_>,
    mut create: impl FnMut(&Path, u32, bool) -> io::Result<()>,
    stat: impl FnMut(&Path) -> io::Result<FileFacts>,
    first_entry: impl FnOnce(&Path) -> io::Result<bool>,
) -> Result<(PathBuf, bool), String> {
    create(stores, 0o700, true).map_err(|error| format!("create stores directory {}: {error}", stores.display()))?;
    let store = stores.join(model.story());
    let kept = reserve_live_store_with(&store, create, stat, first_entry)?;
    let kept = account_store(kept, matches!(model, LiveModel::ChatGpt { account: Some(_) }))?;
    Ok((store, kept))
}

pub(super) fn account_store(kept: bool, account: bool) -> Result<bool, String> {
    if kept && account { Err("a held account moves into a fresh store only".to_owned()) } else { Ok(kept) }
}

/// Reserve a fresh store, or validate a kept deployment using metadata only.
/// An empty directory is fresh; an incomplete deployment is never repaired.
pub fn reserve_live_store(path: &Path) -> Result<bool, String> {
    reserve_live_store_with(path, create, stat, first_entry)
}

fn reserve_live_store_with(
    path: &Path,
    mut create: impl FnMut(&Path, u32, bool) -> io::Result<()>,
    mut stat: impl FnMut(&Path) -> io::Result<FileFacts>,
    first_entry: impl FnOnce(&Path) -> io::Result<bool>,
) -> Result<bool, String> {
    use skein_forge::forgejo::pin::{RUNNER_VERSION, VERSION};
    match create(path, 0o700, false) {
        Ok(()) => return Ok(false),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(format!("create kept store {}: {error}", path.display())),
    }
    let metadata = stat(path).map_err(|error| error.to_string())?;
    require(metadata.kind == FileKind::Directory, &format!("kept store {} is not a directory", path.display()))?;
    require(metadata.mode & 0o777 == 0o700, "kept store must have mode 0700")?;
    if !first_entry(path).map_err(|error| error.to_string())? {
        return Ok(false);
    }
    // The deployment alone: a rerun makes its own client roots, and a sitting
    // that failed before its restart never made `second`.
    for (name, kind) in [
        ("config", FileKind::Directory),
        ("data", FileKind::Directory),
        ("cache", FileKind::Directory),
        ("data/deployment", FileKind::Directory),
        ("data/forge-1/forgejo", FileKind::Directory),
        (&format!("config/{}", temper_config_shell::FILE), FileKind::File),
        ("data/box/started/1", FileKind::File),
        ("data/forge-1/forgejo/app.ini", FileKind::File),
        ("data/forge-1/forgejo/forgejo.db", FileKind::File),
        (&format!("cache/forgejo/{VERSION}/forgejo"), FileKind::File),
        (&format!("cache/forgejo-runner/{RUNNER_VERSION}/forgejo-runner"), FileKind::File),
    ] {
        let present = stat(&path.join(name)).is_ok_and(|meta| meta.kind == kind);
        require(present, &format!("kept store {} lacks {}", path.display(), name))?;
    }
    Ok(true)
}
