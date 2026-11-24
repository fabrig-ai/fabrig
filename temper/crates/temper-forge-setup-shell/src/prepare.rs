//! The preparation's locked host; all decisions belong to its machine.
use crate::{command, files, process};
use skein_lib::{Duration, Queue, Secret};
use std::{
    io::{self, Write},
    path::Path,
};
use temper_forge_setup::{
    place::{File, Place},
    prepare::{self, Event, Exit, Failure, Found, Prepare, Ran, Request},
    spec::Spec,
};
/// Why a preparation did not finish.
#[derive(Debug)]
pub enum Failed {
    Held(File),
    Io(&'static str, io::ErrorKind),
    Command(command::Error),
    Refused(Failure),
}
/// Effects of the preparation's request/event exchange. Acquired locks are
/// held by the driver, including when it inspects or rewrites runner settings.
pub trait Host {
    type Lock;

    fn lock(&mut self, path: &Path) -> Result<Self::Lock, process::Refused>;
    fn read_private(&mut self, path: &Path, bound: u32) -> Result<Secret, skein_shell::Unread>;
    fn exists(&mut self, path: &Path) -> bool;
    fn present(&mut self, place: &Place) -> io::Result<bool>;
    fn clear(&mut self, place: &Place) -> io::Result<()>;
    fn layout(&mut self, place: &Place) -> io::Result<()>;
    fn write(&mut self, path: &Path, contents: &[u8]) -> io::Result<()>;
    fn backup_exists(&mut self, path: &Path) -> io::Result<bool>;
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()>;
    fn draw(&mut self) -> Result<Secret, i32>;
    fn capture(&mut self, spec: &Spec, within: Option<Duration>) -> Result<Ran, command::Error>;
    fn note(&mut self, line: &[u8]) -> io::Result<()>;
}

struct Machine<'a> {
    limits: &'a skein_io::Limits,
}

impl Host for Machine<'_> {
    type Lock = process::Lock;

    fn lock(&mut self, path: &Path) -> Result<Self::Lock, process::Refused> {
        process::lock(path)
    }
    fn read_private(&mut self, path: &Path, bound: u32) -> Result<Secret, skein_shell::Unread> {
        skein_shell::read_private_file(path, bound, skein_shell::effective_user())
    }
    fn exists(&mut self, path: &Path) -> bool {
        path.exists()
    }
    fn present(&mut self, place: &Place) -> io::Result<bool> {
        files::present(place)
    }
    fn clear(&mut self, place: &Place) -> io::Result<()> {
        files::clear(place)
    }
    fn layout(&mut self, place: &Place) -> io::Result<()> {
        files::layout(place)
    }
    fn write(&mut self, path: &Path, contents: &[u8]) -> io::Result<()> {
        files::write_private(path, contents)
    }
    fn backup_exists(&mut self, path: &Path) -> io::Result<bool> {
        files::exists(path)
    }
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        files::rename(from, to)
    }
    fn draw(&mut self) -> Result<Secret, i32> {
        skein_shell::crypto_seed()
    }
    fn capture(&mut self, spec: &Spec, within: Option<Duration>) -> Result<Ran, command::Error> {
        match within {
            Some(within) => {
                let until = skein_shell::Clock::new().now().now.saturating_add(within);
                command::run_until(self.limits, spec, until)
            }
            None => command::run_one(self.limits, spec),
        }
    }
    fn note(&mut self, line: &[u8]) -> io::Result<()> {
        io::stderr().write_all(line)
    }
}

fn locked<H: Host>(host: &mut H, place: &Place, file: File) -> Result<H::Lock, Failed> {
    host.lock(crate::path(&place.path(file))).map_err(|why| match why {
        process::Refused::Held => Failed::Held(file),
        process::Refused::Io(why) => Failed::Io("lock", why),
    })
}

fn private(host: &mut impl Host, path: &Path, name: &'static str) -> Result<Option<Secret>, Failed> {
    match host.read_private(path, 1 << 20) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(skein_shell::Unread::Absent) => Ok(None),
        Err(_) => Err(Failed::Io(name, io::ErrorKind::PermissionDenied)),
    }
}

fn found(host: &mut impl Host, place: &Place) -> Result<Found, Failed> {
    let state = private(host, crate::path(&place.path(File::State)), "state")?;
    let runner_configuration =
        private(host, crate::path(&place.path(File::RunnerConfiguration)), "runner configuration")?;
    Ok(Found {
        state,
        runner_configuration,
        runner_secret: host.exists(crate::path(&place.path(File::RunnerSecret))),
        admin_token: host.exists(crate::path(&place.path(File::AdminToken))),
        person_headers: [
            host.exists(crate::path(&place.path(File::PersonReadHeader))),
            host.exists(crate::path(&place.path(File::PersonWriteHeader))),
        ],
        files: host.present(place).map_err(|why| Failed::Io("directory", why.kind()))?,
    })
}
fn write<H: Host>(
    host: &mut H,
    place: &Place,
    file: File,
    contents: &[u8],
    locks: &mut Vec<(File, H::Lock)>,
) -> Result<(), Failed> {
    let path = place.path(file);
    if file == File::RunnerConfiguration && !locks.iter().any(|(file, _)| *file == File::RunnerLock) {
        if private(host, crate::path(&path), "runner configuration")?.is_some_and(|before| before.expose() == contents)
        {
            return Ok(());
        }
        locks.push((File::RunnerLock, locked(host, place, File::RunnerLock)?));
    }
    host.write(crate::path(&path), contents).map_err(|why| Failed::Io("file", why.kind()))
}
/// Holds every acquired lock until the terminal answer.
pub fn run(limits: &skein_io::Limits, place: &Place, machine: Prepare) -> Result<(), Failed> {
    run_with(&mut Machine { limits }, place, machine)
}

pub fn run_with(host: &mut impl Host, place: &Place, mut machine: Prepare) -> Result<(), Failed> {
    let mut locks = vec![(File::ForgejoLock, locked(host, place, File::ForgejoLock)?)];
    let mut event = Event::Found(found(host, place)?);
    let mut out = Queue::uncapped();
    loop {
        prepare::step(&mut machine, event, &mut out);
        assert_eq!(out.len(), 1, "one preparation request per event");
        event = match out.pop().expect("one preparation request") {
            Request::Version { spec, .. } => {
                let ran = host.capture(&spec, None).map_err(Failed::Command)?;
                Event::Printed(command::first_line(&ran).into_owned().into_boxed_str())
            }
            Request::Run { spec, .. } | Request::Token { spec } | Request::PersonToken { spec, .. } => {
                Event::Ran(host.capture(&spec, None).map_err(Failed::Command)?)
            }
            Request::Source { spec, deadline } => {
                Event::Ran(source_capture(host.capture(&spec, Some(deadline)), spec.output)?)
            }
            Request::Note { line } => {
                host.note(&line).map_err(|why| Failed::Io("stderr", why.kind()))?;
                Event::Done
            }
            Request::Lock(file) => {
                locks.push((file, locked(host, place, file)?));
                Event::Done
            }
            Request::Draw => Event::Drew(host.draw().map_err(|_| Failed::Io("entropy", io::ErrorKind::Other))?),
            Request::Clear => {
                host.clear(place).map_err(|why| Failed::Io("directory", why.kind()))?;
                Event::Done
            }
            Request::Layout => {
                host.layout(place).map_err(|why| Failed::Io("layout", why.kind()))?;
                Event::Done
            }
            Request::Write { file, contents } => {
                write(host, place, file, contents.expose(), &mut locks)?;
                Event::Done
            }
            Request::Exists { path } => {
                Event::Exists(host.backup_exists(crate::path(&path)).map_err(|why| Failed::Io("backup", why.kind()))?)
            }
            Request::Rename { from, to } => {
                host.rename(crate::path(&from), crate::path(&to)).map_err(|why| Failed::Io("backup", why.kind()))?;
                Event::Done
            }
            Request::End(result) => return result.map_err(Failed::Refused),
        };
    }
}

fn source_capture(result: Result<Ran, command::Error>, output: u32) -> Result<Ran, Failed> {
    let words = match result {
        Ok(ran) => return Ok(ran),
        Err(command::Error::Interrupted) => return Err(Failed::Command(command::Error::Interrupted)),
        Err(command::Error::Kernel(words)) => words,
        Err(command::Error::Spawn(why)) => format!("spawn: {why:?}"),
    };
    let bytes = words.as_bytes();
    let stderr = bytes.get(..bytes.len().min(usize::try_from(output).expect("output bound"))).expect("bounded error");
    Ok(Ran { exit: Exit::Code(1), stdout: Secret::new(Box::new([])), stderr: Secret::new(stderr.into()) })
}
