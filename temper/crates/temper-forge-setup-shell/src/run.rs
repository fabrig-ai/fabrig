//! A run holds its lock across admission and hands it to the final process.
use crate::{command, process};
use std::io;
use temper_forge_setup::{
    run::{self, Launch, Refusal},
    state,
};
/// A run returns only when its process did not start.
#[derive(Debug)]
pub enum Failed {
    Refused(Refusal),
    Locked,
    Lock(io::ErrorKind),
    State(&'static str),
    Version(command::Error),
    Exec(io::Error),
}
pub(crate) fn checked(limits: &skein_io::Limits, launch: &Launch) -> Result<process::Lock, Failed> {
    let lock = process::lock(crate::path(&launch.lock)).map_err(|why| match why {
        process::Refused::Held => Failed::Locked,
        process::Refused::Io(io::ErrorKind::NotFound) => Failed::Refused(Refusal::NotPrepared),
        process::Refused::Io(why) => Failed::Lock(why),
    })?;
    let bytes = match skein_shell::read_private_file(crate::path(&launch.state), 1 << 20, skein_shell::effective_user())
    {
        Ok(bytes) => Some(bytes),
        Err(skein_shell::Unread::Absent) => None,
        Err(_) => return Err(Failed::State("cannot be read as a private file")),
    };
    let state = bytes
        .as_ref()
        .map(|bytes| state::decode(bytes.expose()))
        .transpose()
        .map_err(|_| Failed::State("cannot be decoded"))?;
    run::admit_state(launch, state.as_ref()).map_err(Failed::Refused)?;
    let ran = command::run_one(limits, &launch.version).map_err(Failed::Version)?;
    let line = ran.stdout.expose().split(|b| *b == b'\n').next().unwrap_or_default();
    run::admit_binary(launch, &String::from_utf8_lossy(line.strip_suffix(b"\r").unwrap_or(line)))
        .map_err(Failed::Refused)?;
    Ok(lock)
}
/// Locks, checks and execs, returning only a failure.
#[must_use]
pub fn run(limits: &skein_io::Limits, launch: &Launch) -> Failed {
    match checked(limits, launch) {
        Ok(lock) => Failed::Exec(process::exec(&launch.exec, lock)),
        Err(why) => why,
    }
}
