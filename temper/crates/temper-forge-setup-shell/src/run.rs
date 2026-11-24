//! A run holds its lock across admission and hands it to the final process.
use crate::{command, process};
use skein_lib::Secret;
use std::{io, path::Path};
use temper_forge_setup::{
    prepare::Ran,
    run::{self, Launch, Refusal},
    spec::Spec,
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
    checked_with(
        launch,
        process::lock,
        |path, bound| skein_shell::read_private_file(path, bound, skein_shell::effective_user()),
        |spec| command::run_one(limits, spec),
    )
}

/// Admits a run in lock, record, then binary order without starting its process.
pub fn checked_with<L>(
    launch: &Launch,
    lock: impl FnOnce(&Path) -> Result<L, process::Refused>,
    read_private: impl FnOnce(&Path, u32) -> Result<Secret, skein_shell::Unread>,
    capture: impl FnOnce(&Spec) -> Result<Ran, command::Error>,
) -> Result<L, Failed> {
    let lock = lock(crate::path(&launch.lock)).map_err(|why| match why {
        process::Refused::Held => Failed::Locked,
        process::Refused::Io(io::ErrorKind::NotFound) => Failed::Refused(Refusal::NotPrepared),
        process::Refused::Io(why) => Failed::Lock(why),
    })?;
    let bytes = match read_private(crate::path(&launch.state), 1 << 20) {
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
    let ran = capture(&launch.version).map_err(Failed::Version)?;
    run::admit_binary(launch, &command::first_line(&ran)).map_err(Failed::Refused)?;
    Ok(lock)
}
/// Locks, checks and execs, returning only a failure.
#[must_use]
pub fn run(limits: &skein_io::Limits, launch: &Launch) -> Failed {
    run_with(|| checked(limits, launch), |lock| process::exec(&launch.exec, lock))
}

/// Keeps the admitted lock in the final exec's hands; a refused admission never execs.
pub fn run_with<L>(checked: impl FnOnce() -> Result<L, Failed>, exec: impl FnOnce(L) -> io::Error) -> Failed {
    match checked() {
        Ok(lock) => Failed::Exec(exec(lock)),
        Err(why) => why,
    }
}
