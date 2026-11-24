//! Named-file facts for the fake's outputs and images (temper/docs/design/testing/box/README.md, 3.9–3.10).
use skein_io::kernel::{Kind, Stat};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    path::Path,
};

pub(super) const UNSAFE_OUTPUT: &str = "expected this user's private single-link regular output";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub stat: Stat,
    pub identity: (u64, u64),
}
impl Metadata {
    pub(super) fn safe(self, user: u32) -> bool {
        self.stat.kind == Kind::File
            && self.stat.owner == user
            && self.stat.mode.trailing_zeros() >= 6
            && self.stat.links == 1
    }
}
impl From<fs::Metadata> for Metadata {
    fn from(metadata: fs::Metadata) -> Self {
        let kind = if metadata.is_symlink() {
            Kind::Symlink
        } else if metadata.is_dir() {
            Kind::Directory
        } else if metadata.is_file() {
            Kind::File
        } else {
            Kind::Other
        };
        Self {
            stat: Stat {
                kind,
                size: metadata.len(),
                mode: metadata.mode() & skein_io::kernel::PERMISSIONS,
                owner: metadata.uid(),
                links: u32::try_from(metadata.nlink()).unwrap_or(u32::MAX),
            },
            identity: (metadata.dev(), metadata.ino()),
        }
    }
}

pub trait Disk {
    type File;
    fn user(&self) -> u32;
    fn metadata(&mut self, path: &Path) -> io::Result<Metadata>;
    fn open(&mut self, path: &Path) -> io::Result<Self::File>;
    fn opened_metadata(&mut self, file: &Self::File) -> io::Result<Metadata>;
    fn read(&mut self, file: Self::File, bound: u64) -> io::Result<Box<[u8]>>;
    fn create(&mut self, path: &Path, mode: u32) -> io::Result<Self::File>;
    fn write(&mut self, file: &mut Self::File, bytes: &[u8]) -> io::Result<()>;
    fn sync(&mut self, file: &Self::File) -> io::Result<()>;
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()>;
    fn create_directories(&mut self, path: &Path, mode: u32) -> io::Result<()>;
    fn remove_file(&mut self, path: &Path) -> io::Result<()>;
    fn remove_directory(&mut self, path: &Path) -> io::Result<()>;
}

pub(super) struct Native;
impl Disk for Native {
    type File = File;
    fn user(&self) -> u32 {
        skein_shell::effective_user()
    }
    fn metadata(&mut self, path: &Path) -> io::Result<Metadata> {
        fs::symlink_metadata(path).map(Metadata::from)
    }
    fn open(&mut self, path: &Path) -> io::Result<File> {
        File::open(path)
    }
    fn opened_metadata(&mut self, file: &File) -> io::Result<Metadata> {
        file.metadata().map(Metadata::from)
    }
    fn read(&mut self, file: File, bound: u64) -> io::Result<Box<[u8]>> {
        let mut bytes = Vec::new();
        file.take(bound).read_to_end(&mut bytes)?;
        Ok(bytes.into_boxed_slice())
    }
    fn create(&mut self, path: &Path, mode: u32) -> io::Result<File> {
        OpenOptions::new().write(true).create_new(true).mode(mode).open(path)
    }
    fn write(&mut self, file: &mut File, bytes: &[u8]) -> io::Result<()> {
        file.write_all(bytes)
    }
    fn sync(&mut self, file: &File) -> io::Result<()> {
        file.sync_all()
    }
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }
    fn create_directories(&mut self, path: &Path, mode: u32) -> io::Result<()> {
        fs::DirBuilder::new().recursive(true).mode(mode).create(path)
    }
    fn remove_file(&mut self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }
    fn remove_directory(&mut self, path: &Path) -> io::Result<()> {
        fs::remove_dir(path)
    }
}

/// Verify that the opened file and both looks at its name agree.
pub(super) fn opened<D: Disk>(disk: &mut D, path: &Path, first: Metadata) -> Result<(D::File, Metadata), String> {
    let file = disk.open(path).map_err(|why| why.to_string())?;
    let opened = disk.opened_metadata(&file).map_err(|why| why.to_string())?;
    let named = disk.metadata(path).map_err(|why| why.to_string())?;
    let user = disk.user();
    if !opened.safe(user) || !named.safe(user) || opened.identity != named.identity || opened.identity != first.identity
    {
        return Err("output changed while opening".into());
    }
    Ok((file, opened))
}
