//! The immutable private setup file (temper/docs/design/protocol/setup/place.md, 3).
use jig_protocol_config::{Directories, Directory};
use std::{
    fs,
    io::{self, Write},
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
};
use temper_forge_setup::{
    machine::BoxKind,
    setup_file::{self, SetupFile},
};

#[must_use]
pub fn path(directories: &Directories, forge: u16, kind: BoxKind) -> PathBuf {
    crate::path(&directories.join(Directory::Data, &setup_file::path(forge, kind)).expect("forge setup path")).into()
}
pub fn read(directories: &Directories, forge: u16, kind: BoxKind) -> Result<SetupFile, String> {
    read_with(directories, forge, kind, |path, bound| {
        skein_shell::read_private_file(path, bound, skein_shell::effective_user())
    })
}

/// Decodes the selected private setup's bounded bytes and checks its kind.
pub fn read_with(
    directories: &Directories,
    forge: u16,
    kind: BoxKind,
    read_private: impl FnOnce(&std::path::Path, u32) -> Result<skein_lib::Secret, skein_shell::Unread>,
) -> Result<SetupFile, String> {
    let path = path(directories, forge, kind);
    let bytes = read_private(&path, 65536).map_err(|why| {
        format!(
            "{}: {why:?}; run forge prepare {forge} --listen ADDR --person LOGIN EMAIL [--runner-label LABEL]",
            path.display()
        )
    })?;
    let setup = setup_file::decode(bytes.expose(), directories)
        .map_err(|why| format!("{}: setup cannot be decoded: {why:?}", path.display()))?;
    if setup.kind() != kind {
        return Err(format!("{}: setup kind differs from its directory", path.display()));
    }
    Ok(setup)
}
/// No existing setup file is replaced, even when two writers race.
pub fn write(directories: &Directories, forge: u16, setup: &SetupFile) -> Result<(), String> {
    let path = path(directories, forge, setup.kind());
    write_once(&path, &setup_file::encode(setup)).map_err(|why| format!("{}: {why}", path.display()))
}
fn write_once(path: &std::path::Path, bytes: &[u8]) -> io::Result<()> {
    let directory = path.parent().expect("setup has a forge directory");
    temper_shell::directories::make_private(directory, skein_shell::effective_user()).map_err(io::Error::other)?;
    let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::File::open(directory)?.sync_all()
}
