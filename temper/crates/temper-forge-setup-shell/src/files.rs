//! Whole private files and the preparation's exact removable layout.
use std::{
    fs,
    io::{self, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::Path,
};
use temper_forge_setup::place::Place;

/// File effects, in the order the forge's host asks for them (temper/docs/design/testing/shells.md, 3.1).
#[doc(hidden)]
pub trait Disk {
    type File;

    fn create_new(&mut self, path: &Path, mode: u32) -> io::Result<Self::File>;
    fn open(&mut self, path: &Path) -> io::Result<Self::File>;
    fn write_all(&mut self, file: &mut Self::File, bytes: &[u8]) -> io::Result<()>;
    fn sync(&mut self, file: &Self::File) -> io::Result<()>;
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()>;
    /// No-follow metadata: true for a directory, false for any other entry.
    fn is_directory(&mut self, path: &Path) -> io::Result<bool>;
    fn remove_file(&mut self, path: &Path) -> io::Result<()>;
    fn remove_dir_all(&mut self, path: &Path) -> io::Result<()>;
    fn create_dir_all(&mut self, path: &Path) -> io::Result<()>;
    fn set_mode(&mut self, path: &Path, mode: u32) -> io::Result<()>;
}

struct Native;

impl Disk for Native {
    type File = fs::File;

    fn create_new(&mut self, path: &Path, mode: u32) -> io::Result<Self::File> {
        fs::OpenOptions::new().write(true).create_new(true).mode(mode).open(path)
    }
    fn open(&mut self, path: &Path) -> io::Result<Self::File> {
        fs::File::open(path)
    }
    fn write_all(&mut self, file: &mut Self::File, bytes: &[u8]) -> io::Result<()> {
        file.write_all(bytes)
    }
    fn sync(&mut self, file: &Self::File) -> io::Result<()> {
        file.sync_all()
    }
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }
    fn is_directory(&mut self, path: &Path) -> io::Result<bool> {
        fs::symlink_metadata(path).map(|meta| meta.is_dir())
    }
    fn remove_file(&mut self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }
    fn remove_dir_all(&mut self, path: &Path) -> io::Result<()> {
        fs::remove_dir_all(path)
    }
    fn create_dir_all(&mut self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }
    fn set_mode(&mut self, path: &Path, mode: u32) -> io::Result<()> {
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
    }
}

/// Writes beside the destination, syncs, renames and syncs its directory.
pub fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    write_private_with(&mut Native, path, bytes)
}

#[doc(hidden)]
pub fn write_private_with(disk: &mut impl Disk, path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temporary = path.with_extension("partial");
    let mut file = disk.create_new(&temporary, 0o600).or_else(|why| {
        if why.kind() != io::ErrorKind::AlreadyExists {
            return Err(why);
        }
        disk.remove_file(&temporary)?;
        disk.create_new(&temporary, 0o600)
    })?;
    disk.write_all(&mut file, bytes)?;
    disk.sync(&file)?;
    disk.rename(&temporary, path)?;
    let parent = disk.open(path.parent().expect("a forge file has a parent"))?;
    disk.sync(&parent)
}

const CLEARED: [&str; 6] =
    ["forgejo", "runner", "admin_token", "person_password", "person_read.header", "person_write.header"];

/// Whether a first preparation's files exist, including a dangling link.
pub fn present(place: &Place) -> io::Result<bool> {
    present_with(&mut Native, place)
}

#[doc(hidden)]
pub fn present_with(disk: &mut impl Disk, place: &Place) -> io::Result<bool> {
    let root = crate::path(&place.directory);
    for name in CLEARED {
        if exists_with(disk, &root.join(name))? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Clears only a recorded unfinished preparation; keeps backups and state.
pub fn clear(place: &Place) -> io::Result<()> {
    clear_with(&mut Native, place)
}

#[doc(hidden)]
pub fn clear_with(disk: &mut impl Disk, place: &Place) -> io::Result<()> {
    for name in CLEARED {
        let path = crate::path(&place.directory).join(name);
        match disk.is_directory(&path) {
            Ok(true) => disk.remove_dir_all(&path)?,
            Ok(false) => disk.remove_file(&path)?,
            Err(why) if why.kind() == io::ErrorKind::NotFound => {}
            Err(why) => return Err(why),
        }
    }
    Ok(())
}

/// Makes the owned layout private, including dump's repository directory.
pub fn layout(place: &Place) -> io::Result<()> {
    layout_with(&mut Native, place)
}

#[doc(hidden)]
pub fn layout_with(disk: &mut impl Disk, place: &Place) -> io::Result<()> {
    let root = crate::path(&place.directory);
    for name in ["home", "forgejo", "forgejo/repositories", "runner", "backups"] {
        let path = root.join(name);
        disk.create_dir_all(&path)?;
        disk.set_mode(&path, 0o700)?;
    }
    write_private_with(disk, &root.join("home/gitconfig"), b"")
}

/// Whether a backup is already present; inspection failures refuse the preparation.
pub fn exists(path: &Path) -> io::Result<bool> {
    exists_with(&mut Native, path)
}

#[doc(hidden)]
pub fn exists_with(disk: &mut impl Disk, path: &Path) -> io::Result<bool> {
    match disk.is_directory(path) {
        Ok(_) => Ok(true),
        Err(why) if why.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(why) => Err(why),
    }
}

/// Syncs a completed partial, renames it and syncs its directory.
pub fn rename(from: &Path, to: &Path) -> io::Result<()> {
    rename_with(&mut Native, from, to)
}

#[doc(hidden)]
pub fn rename_with(disk: &mut impl Disk, from: &Path, to: &Path) -> io::Result<()> {
    {
        let file = disk.open(from)?;
        disk.sync(&file)?;
    }
    disk.rename(from, to)?;
    let parent = disk.open(to.parent().expect("a backup has a parent"))?;
    disk.sync(&parent)
}
