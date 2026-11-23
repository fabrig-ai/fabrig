//! Per-origin directories and the move from the TUI's flat directory (client/tui/directory.md, 1).
use super::Refused;
use skein_lib::Secret;
use std::{
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
};
use temper_client_tui::{Note, Origin, theme::THEME_KEY};
use temper_shell::directories;

pub(super) fn relative(origin: &Origin) -> PathBuf {
    let authority = origin.authority();
    let text = std::str::from_utf8(&authority).expect("origin authority is ASCII");
    let (host, port) = text.rsplit_once(':').expect("authority includes port");
    let host = host.strip_prefix('[').and_then(|host| host.strip_suffix(']')).unwrap_or(host);
    Path::new("at").join(host).join(port)
}

pub(super) fn prepare(directory: &Path, origin: &Origin, user: u32) -> Result<(PathBuf, bool), Refused> {
    prepare_with(directory, origin, user, &mut Disk)
}

pub(super) fn prepare_with(
    directory: &Path,
    origin: &Origin,
    user: u32,
    files: &mut impl Files,
) -> Result<(PathBuf, bool), Refused> {
    migrate(directory, user, files).map_err(|why| refused(directory, &why))?;
    let relative = relative(origin);
    let destination = directory.join(&relative);
    let made = !exists(files, &destination).map_err(|why| refused(&destination, &why))?;
    for component in
        [directory.join("at"), destination.parent().expect("origin host").to_path_buf(), destination.clone()]
    {
        files.make_private(&component, user).map_err(Refused::Directory)?;
    }
    Ok((destination, made))
}

pub(super) fn theme(directory: &Path, origin: &Origin, user: u32, made: bool) -> Option<jig_client::Record> {
    theme_with(directory, origin, user, made, &mut Disk)
}

pub(super) fn theme_with(
    directory: &Path,
    origin: &Origin,
    user: u32,
    made: bool,
    files: &mut impl Files,
) -> Option<jig_client::Record> {
    if !made {
        return None;
    }
    let saved = files.read_private(&directory.join("address"), 300, user)?;
    let saved = Origin::parse(saved.expose()).ok()?;
    if &saved == origin {
        return None;
    }
    let other = directory.join(relative(&saved));
    for component in [directory.join("at"), other.parent()?.to_path_buf(), other.clone()] {
        files.private_root(&component, user).ok()?;
    }
    let max = super::limits::tui().native.connections.body_bytes;
    let bytes = files.read_private(&other.join("store"), max, user)?;
    jig_client_shell_native::store_record(bytes.expose(), THEME_KEY)
}

const FILES: [&str; 3] = ["token", "store", "sign-in"];

fn refused(directory: &Path, why: &io::Error) -> Refused {
    Refused::Directory(format!("{}: {why}", directory.display()))
}

type Entry = (OsString, PathBuf);

pub(super) trait Files {
    type Entries: Iterator<Item = io::Result<Entry>>;

    fn check_entry(&mut self, path: &Path) -> io::Result<()>;
    fn entries(&mut self, path: &Path) -> io::Result<Self::Entries>;
    fn read_private(&mut self, path: &Path, max: u32, user: u32) -> Option<Secret>;
    fn private_root(&mut self, path: &Path, user: u32) -> Result<(), String>;
    fn make_private(&mut self, path: &Path, user: u32) -> Result<(), String>;
    fn sync(&mut self, path: &Path) -> io::Result<()>;
    fn rename(&mut self, source: &Path, target: &Path) -> io::Result<()>;
    fn remove_file(&mut self, path: &Path) -> io::Result<()>;
    fn remove_dir(&mut self, path: &Path) -> io::Result<()>;
}

struct Disk;

impl Files for Disk {
    type Entries = std::iter::Map<fs::ReadDir, fn(io::Result<fs::DirEntry>) -> io::Result<Entry>>;

    fn check_entry(&mut self, path: &Path) -> io::Result<()> {
        fs::symlink_metadata(path).map(|_| ())
    }

    fn entries(&mut self, path: &Path) -> io::Result<Self::Entries> {
        Ok(fs::read_dir(path)?.map(|entry| entry.map(|entry| (entry.file_name(), entry.path()))))
    }

    fn read_private(&mut self, path: &Path, max: u32, user: u32) -> Option<Secret> {
        skein_shell::read_private_file(path, max, user).ok()
    }

    fn private_root(&mut self, path: &Path, user: u32) -> Result<(), String> {
        directories::private_root(path, user)
    }

    fn make_private(&mut self, path: &Path, user: u32) -> Result<(), String> {
        directories::make_private(path, user)
    }

    fn sync(&mut self, path: &Path) -> io::Result<()> {
        fs::File::open(path)?.sync_all()
    }

    fn rename(&mut self, source: &Path, target: &Path) -> io::Result<()> {
        fs::rename(source, target)
    }

    fn remove_file(&mut self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }

    fn remove_dir(&mut self, path: &Path) -> io::Result<()> {
        fs::remove_dir(path)
    }
}

fn exists(files: &mut impl Files, path: &Path) -> io::Result<bool> {
    match files.check_entry(path) {
        Ok(()) => Ok(true),
        Err(why) if why.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(why) => Err(why),
    }
}

fn private(files: &mut impl Files, directory: &Path, user: u32) -> io::Result<()> {
    files.make_private(directory, user).map_err(io::Error::other)?;
    files.sync(directory.parent().expect("directory parent"))
}

fn remove(files: &mut impl Files, directory: &Path) -> io::Result<()> {
    for file in FILES {
        match files.remove_file(&directory.join(file)) {
            Ok(()) => files.sync(directory)?,
            Err(why) if why.kind() == io::ErrorKind::NotFound => {}
            Err(why) => return Err(why),
        }
    }
    Ok(())
}

fn finish(files: &mut impl Files, directory: &Path, staging: &Path, destination: &Path, user: u32) -> io::Result<()> {
    files.private_root(staging, user).map_err(io::Error::other)?;
    if exists(files, destination)? {
        files.private_root(destination, user).map_err(io::Error::other)?;
        remove(files, staging)?;
        remove(files, directory)?;
        files.remove_dir(staging)?;
    } else {
        for file in FILES {
            let source = directory.join(file);
            if exists(files, &source)? {
                let target = staging.join(file);
                if !exists(files, &target)? {
                    files.rename(&source, &target)?;
                    files.sync(&target)?;
                    files.sync(staging)?;
                    files.sync(directory)?;
                }
            }
        }
        files.rename(staging, destination)?;
        files.sync(destination.parent().expect("origin host"))?;
        remove(files, directory)?;
    }
    files.sync(destination.parent().expect("origin host"))
}

fn migrate(directory: &Path, user: u32, files: &mut impl Files) -> io::Result<()> {
    let at = directory.join("at");
    if exists(files, &at)? {
        files.private_root(&at, user).map_err(io::Error::other)?;
        for host in files.entries(&at)? {
            let (_, host) = host?;
            files.private_root(&host, user).map_err(io::Error::other)?;
            for entry in files.entries(&host)? {
                let (name, path) = entry?;
                let Some(port) = name.to_str().and_then(|name| name.strip_suffix(".new")) else { continue };
                if port.parse::<u16>().is_ok_and(|port| port != 0) {
                    finish(files, directory, &path, &host.join(port), user)?;
                }
            }
        }
    }
    let saved =
        files.read_private(&directory.join("address"), 300, user).and_then(|bytes| Origin::parse(bytes.expose()).ok());
    let legacy = match saved {
        Some(origin @ Origin::Http(_)) => {
            let note = files
                .read_private(&directory.join("sign-in"), Note::MOST, user)
                .and_then(|bytes| Note::read(bytes.expose(), &origin));
            note.map(|_| origin)
        }
        Some(Origin::Https { .. }) | None => None,
    };
    let Some(origin) = legacy else { return remove(files, directory) };
    let destination = directory.join(relative(&origin));
    if exists(files, &destination)? {
        return remove(files, directory);
    }
    private(files, &at, user)?;
    private(files, destination.parent().expect("origin host"), user)?;
    let staging = destination.with_extension("new");
    private(files, &staging, user)?;
    finish(files, directory, &staging, &destination, user)
}
