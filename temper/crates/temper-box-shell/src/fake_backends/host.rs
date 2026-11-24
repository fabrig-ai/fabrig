//! Native facts for the sandbox's preparation (temper/docs/design/testing/shells.md, 3.7).
use super::{Directories, Listeners, Source, locks};
use crate::fake::disk::Metadata;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener},
    num::NonZeroU16,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};
use temper_config_shell::{Facts, Read, Shape};
use temper_service::limits::Clients;

pub(super) trait Files {
    type File;
    fn user(&self) -> u32;
    fn metadata(&self, path: &Path) -> io::Result<Metadata>;
    fn exists(&self, path: &Path) -> bool;
    fn public_path(&self, path: &Path) -> Result<(), String>;
    fn read(&self, path: &Path, bound: u32, private: bool) -> Result<Box<[u8]>, skein_shell::Unread>;
    fn open_write(&self, path: &Path, mode: u32, replace: bool) -> io::Result<Self::File>;
    fn mode(&self, file: &Self::File, mode: u32) -> io::Result<()>;
    fn write(&self, file: &mut Self::File, bytes: &[u8]) -> io::Result<()>;
    fn sync(&self, file: &Self::File) -> io::Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn remove(&self, path: &Path) -> io::Result<()>;
}

pub(super) struct Initialized<L> {
    pub paths: Vec<PathBuf>,
    pub ports: Vec<NonZeroU16>,
    pub listeners: L,
}

pub(super) trait Host: Files {
    type Lock;
    type Listeners;
    type Peer;
    fn binary(&self) -> Result<PathBuf, String>;
    fn source(&self, dirs: &Directories) -> Result<Source, String>;
    fn config_facts(&self, dirs: &Directories) -> Result<Facts, temper_config_shell::Refusal>;
    fn read_config(&self, dirs: &Directories) -> Result<Read, String>;
    fn read_bundle(&self, path: &Path, limits: Clients) -> Result<(), String>;
    fn program_metadata(&self, path: &Path) -> Option<(bool, u32)>;
    fn make_private(&self, path: &Path, user: u32) -> Result<(), String>;
    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()>;
    fn lock(&self, root: &Path) -> Result<Self::Lock, String>;
    fn init(
        &self,
        dirs: &Directories,
        binary: &Path,
        preferred: [u16; 2],
        adapter: &Path,
    ) -> Result<Initialized<Self::Listeners>, String>;
    fn bind(&self, port: u16) -> io::Result<Self::Peer>;
    fn address(&self, peer: &Self::Peer) -> io::Result<SocketAddr>;
}

pub(super) struct Native<'a>(pub &'a [crate::EnvironmentEntry]);

impl Files for Native<'_> {
    type File = File;
    fn user(&self) -> u32 {
        skein_shell::effective_user()
    }
    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        fs::symlink_metadata(path).map(Metadata::from)
    }
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
    fn public_path(&self, path: &Path) -> Result<(), String> {
        super::public_path(path)
    }
    fn read(&self, path: &Path, bound: u32, private: bool) -> Result<Box<[u8]>, skein_shell::Unread> {
        if private {
            skein_shell::read_private_file(path, bound, self.user()).map(|bytes| bytes.expose().into())
        } else {
            skein_shell::read_file(path, bound)
        }
    }
    fn open_write(&self, path: &Path, mode: u32, replace: bool) -> io::Result<File> {
        let mut options = OpenOptions::new();
        options.write(true).mode(mode);
        if replace {
            options.truncate(true);
        } else {
            options.create_new(true);
        }
        options.open(path)
    }
    fn mode(&self, file: &File, mode: u32) -> io::Result<()> {
        file.set_permissions(fs::Permissions::from_mode(mode))
    }
    fn write(&self, file: &mut File, bytes: &[u8]) -> io::Result<()> {
        file.write_all(bytes)
    }
    fn sync(&self, file: &File) -> io::Result<()> {
        file.sync_all()
    }
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }
    fn remove(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }
}

impl Host for Native<'_> {
    type Lock = locks::Lock;
    type Listeners = Listeners;
    type Peer = TcpListener;
    fn binary(&self) -> Result<PathBuf, String> {
        std::env::current_exe().map_err(|why| why.to_string())
    }
    fn source(&self, dirs: &Directories) -> Result<Source, String> {
        Source::read(dirs, self.0)
    }
    fn config_facts(&self, dirs: &Directories) -> Result<Facts, temper_config_shell::Refusal> {
        temper_config_shell::facts(dirs)
    }
    fn read_config(&self, dirs: &Directories) -> Result<Read, String> {
        temper_config_shell::read(dirs, Shape::Standalone).map_err(|why| why.to_string())
    }
    fn read_bundle(&self, path: &Path, limits: Clients) -> Result<(), String> {
        crate::bundle::read(super::bytes(path).as_ref(), limits).map(|_| ())
    }
    fn program_metadata(&self, path: &Path) -> Option<(bool, u32)> {
        fs::metadata(path).ok().map(|meta| (meta.is_file(), meta.mode()))
    }
    fn make_private(&self, path: &Path, user: u32) -> Result<(), String> {
        super::make_private(path, user)
    }
    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()> {
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
    }
    fn lock(&self, root: &Path) -> Result<Self::Lock, String> {
        super::lock(root)
    }
    fn init(
        &self,
        dirs: &Directories,
        binary: &Path,
        preferred: [u16; 2],
        adapter: &Path,
    ) -> Result<Initialized<Listeners>, String> {
        let written = crate::init::write_with_program(dirs, self.0, binary, preferred, Some(adapter))?;
        Ok(Initialized {
            paths: written.paths,
            ports: written.listeners.iter().map(|(_, port)| *port).collect(),
            listeners: Listeners(written.listeners),
        })
    }
    fn bind(&self, port: u16) -> io::Result<TcpListener> {
        TcpListener::bind((Ipv4Addr::LOCALHOST, port))
    }
    fn address(&self, peer: &TcpListener) -> io::Result<SocketAddr> {
        peer.local_addr()
    }
}
