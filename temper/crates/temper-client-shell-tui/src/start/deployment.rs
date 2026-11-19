//! Per-origin directories and the move from the TUI's flat directory (client/tui/README.md, 2).
use super::Refused;
use std::{
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
    migrate(directory, user).map_err(|why| refused(directory, &why))?;
    let relative = relative(origin);
    let destination = directory.join(&relative);
    let made = !exists(&destination).map_err(|why| refused(&destination, &why))?;
    for component in
        [directory.join("at"), destination.parent().expect("origin host").to_path_buf(), destination.clone()]
    {
        directories::make_private(&component, user).map_err(Refused::Directory)?;
    }
    Ok((destination, made))
}

pub(super) fn theme(directory: &Path, origin: &Origin, user: u32, made: bool) -> Option<jig_client::Record> {
    if !made {
        return None;
    }
    let saved = skein_shell::read_private_file(&directory.join("address"), 300, user).ok()?;
    let saved = Origin::parse(saved.expose()).ok()?;
    if &saved == origin {
        return None;
    }
    let other = directory.join(relative(&saved));
    for component in [directory.join("at"), other.parent()?.to_path_buf(), other.clone()] {
        directories::private_root(&component, user).ok()?;
    }
    let max = super::limits::tui().native.connections.body_bytes;
    let bytes = skein_shell::read_private_file(&other.join("store"), max, user).ok()?;
    jig_client_shell_native::store_record(bytes.expose(), THEME_KEY)
}

const FILES: [&str; 3] = ["token", "store", "sign-in"];

fn refused(directory: &Path, why: &io::Error) -> Refused {
    Refused::Directory(format!("{}: {why}", directory.display()))
}

fn exists(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(why) if why.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(why) => Err(why),
    }
}

fn sync(directory: &Path) -> io::Result<()> {
    fs::File::open(directory)?.sync_all()
}

fn private(directory: &Path, user: u32) -> io::Result<()> {
    directories::make_private(directory, user).map_err(io::Error::other)?;
    sync(directory.parent().expect("directory parent"))
}

fn remove(directory: &Path) -> io::Result<()> {
    for file in FILES {
        match fs::remove_file(directory.join(file)) {
            Ok(()) => sync(directory)?,
            Err(why) if why.kind() == io::ErrorKind::NotFound => {}
            Err(why) => return Err(why),
        }
    }
    Ok(())
}

fn finish(directory: &Path, staging: &Path, destination: &Path, user: u32) -> io::Result<()> {
    directories::private_root(staging, user).map_err(io::Error::other)?;
    if exists(destination)? {
        directories::private_root(destination, user).map_err(io::Error::other)?;
        remove(staging)?;
        remove(directory)?;
        fs::remove_dir(staging)?;
    } else {
        for file in FILES {
            let source = directory.join(file);
            if exists(&source)? {
                let target = staging.join(file);
                if !exists(&target)? {
                    fs::rename(&source, &target)?;
                    sync(&target)?;
                    sync(staging)?;
                    sync(directory)?;
                }
            }
        }
        fs::rename(staging, destination)?;
        sync(destination.parent().expect("origin host"))?;
        remove(directory)?;
    }
    sync(destination.parent().expect("origin host"))
}

fn migrate(directory: &Path, user: u32) -> io::Result<()> {
    let at = directory.join("at");
    if exists(&at)? {
        directories::private_root(&at, user).map_err(io::Error::other)?;
        for host in fs::read_dir(&at)? {
            let host = host?.path();
            directories::private_root(&host, user).map_err(io::Error::other)?;
            for entry in fs::read_dir(&host)? {
                let entry = entry?;
                let name = entry.file_name();
                let Some(port) = name.to_str().and_then(|name| name.strip_suffix(".new")) else { continue };
                if port.parse::<u16>().is_ok_and(|port| port != 0) {
                    finish(directory, &entry.path(), &host.join(port), user)?;
                }
            }
        }
    }
    let saved = skein_shell::read_private_file(&directory.join("address"), 300, user)
        .ok()
        .and_then(|bytes| Origin::parse(bytes.expose()).ok());
    let legacy = match saved {
        Some(origin @ Origin::Http(_)) => {
            let note = skein_shell::read_private_file(&directory.join("sign-in"), Note::MOST, user)
                .ok()
                .and_then(|bytes| Note::read(bytes.expose(), &origin));
            note.map(|_| origin)
        }
        Some(Origin::Https { .. }) | None => None,
    };
    let Some(origin) = legacy else { return remove(directory) };
    let destination = directory.join(relative(&origin));
    if exists(&destination)? {
        return remove(directory);
    }
    private(&at, user)?;
    private(destination.parent().expect("origin host"), user)?;
    let staging = destination.with_extension("new");
    private(&staging, user)?;
    finish(directory, &staging, &destination, user)
}
