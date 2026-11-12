//! Serve's run, its host and its lines, for every binary that serves (temper/docs/design/protocol/binaries.md, 1 and temper/docs/design/protocol/components.md).
use jig_protocol_config as config;
use skein_io::kernel::{Exit, Fd};
use skein_lib::{List, Secret};
use skein_shell::{Clock, Config, Kernel};
use std::{fmt::Write, os::unix::ffi::OsStrExt, path::Path};
use temper_service_core::{self as service, settings::Files};
use temper_shell::directories::EnvironmentEntry;
use temper_shell::{
    directories::{Directories, unread},
    link_file,
};
pub mod host;
mod restart_fault;
pub use host::Host;

/// jig's form of the three directories (temper/docs/design/protocol/components.md).
#[must_use]
pub fn jig_directories(directories: &Directories) -> config::Directories {
    config::Directories::given(&directories.config, &directories.data, &directories.cache)
        .expect("resolved absolute bounded directories")
}

#[must_use]
pub fn listener_words(why: skein_shell::ListenersRefusal) -> String {
    use skein_shell::ListenersRefusal as Why;
    match why {
        Why::BadCount => "expected a decimal listener count".into(),
        Why::TooMany { bound } => format!("listener count exceeds {bound}"),
        Why::NotOpen { fd } => format!("fd {} is not open", fd.raw()),
        Why::NotSocket { fd, .. } => format!("fd {} is not a socket", fd.raw()),
        Why::NotStream { fd, .. } => format!("fd {} is not a stream socket", fd.raw()),
        Why::NotListening { fd } => format!("fd {} is not listening", fd.raw()),
        Why::NotIp { fd } => format!("fd {} is not an IP listener", fd.raw()),
        Why::Check { fd, check, errno } => format!("fd {}: {}: errno {errno}", fd.raw(), check.name()),
        Why::Kernel { fd, errno } => format!("fd {}: close-on-exec: errno {errno}", fd.raw()),
    }
}
fn path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}
fn prefixed(command: &str, why: &str) -> String {
    let mut lines = String::new();
    for line in why.lines() {
        writeln!(lines, "{command}: {line}").expect("writing into a string");
    }
    lines
}
/// The shared wait line, with the binary's payload-free detail (protocol/services.md, 7).
#[must_use]
pub fn core_wait_line(binary: &str, detail: &[u8]) -> String {
    format!("{binary} serve: the core is not answering: {}\n", String::from_utf8_lossy(detail))
}
#[must_use]
pub fn core_answers_line(binary: &str) -> String {
    format!("{binary} serve: the core answers again\n")
}
#[must_use]
pub fn problems(binary: &str, found: &service::startup::problems::SharedProblems) -> String {
    prefixed(&format!("{binary} serve"), &String::from_utf8_lossy(&service::startup::problems::lines(found)))
}
#[must_use]
pub fn file_path(at: config::Path) -> String {
    let mut found = service::startup::problems::SharedProblems::empty();
    found.push(at, service::startup::problems::What::Jig(config::What::Unreadable));
    let bytes = service::startup::problems::lines(&found);
    String::from_utf8_lossy(&bytes).trim_end().strip_suffix(": not read").expect("path and reason").into()
}
pub fn read_files(names: Box<[config::File]>, user: u32) -> Result<Files, String> {
    read_files_with_asked(names, Box::new([]), user)
}
pub fn read_files_with_asked(
    names: Box<[config::File]>,
    asked: Box<[config::File]>,
    user: u32,
) -> Result<Files, String> {
    let count = names.len().checked_add(asked.len()).expect("bounded files");
    let mut entries = List::with_capacity(u32::try_from(count).expect("bounded files"));
    let mut refused = String::new();
    for (file, optional) in
        names.into_iter().map(|file| (file, false)).chain(asked.into_iter().map(|file| (file, true)))
    {
        let resolved = path(&file.path);
        let value = if file.private {
            skein_shell::read_private_file(resolved, file.max, user)
        } else {
            skein_shell::read_file(resolved, file.max).map(Secret::new)
        };
        match value {
            Ok(value) => entries.push((file.path, value)).expect("named capacity"),
            Err(skein_shell::Unread::Absent) if optional => {}
            Err(why) => {
                writeln!(refused, "{}: {}: {}", file_path(file.at), path(&file.path).display(), unread(why))
                    .expect("writing into a string");
            }
        }
    }
    if refused.is_empty() { Ok(Files { entries }) } else { Err(refused) }
}
pub fn close(owned: Vec<Fd>) {
    for fd in owned {
        skein_shell::close_startup_fd(fd);
    }
}
#[must_use]
pub fn listeners_refusal(why: skein_shell::ListenersRefusal) -> String {
    format!("LISTEN_FDS: {}", listener_words(why))
}
#[must_use]
pub fn held_refusal(why: skein_shell::ListenersRefusal) -> String {
    format!("held listener: {}", listener_words(why))
}
/// The service's ending rendered under its binary, including output failure.
pub type Ending<S> = fn(&S, &str, bool, &Directories, &[u8]) -> (String, u8);

/// A binary's adapter to its service (protocol/services.md, 7).
/// The shell owns scheduling and output; these plain functions supply the service's operations and words.
pub struct Served<S> {
    pub stages: &'static [service::Stage],
    pub iterate_stage: fn(&mut S, service::Stage, skein_lib::Time, skein_lib::Wall),
    pub resolve: fn(&mut S),
    pub completions: fn(&mut S) -> &mut skein_lib::Queue<skein_io::kernel::Complete>,
    pub submissions: fn(&mut S) -> &mut skein_lib::Queue<skein_io::kernel::Submit>,
    pub operations: fn(&S) -> u32,
    pub work_pending: fn(&S, skein_lib::Time) -> bool,
    pub next_mechanism_deadline: fn(&S) -> Option<skein_lib::Time>,
    pub next_policy_deadline: fn(&S) -> Option<skein_lib::Time>,
    pub is_empty: fn(&S) -> bool,
    pub shutdown: fn(&mut S),
    pub ending: Ending<S>,
    pub handover: fn(&mut S),
    pub listening: fn(&mut S, &str) -> Option<(String, bool)>,
    pub note: fn(&mut S, &str) -> Option<String>,
    pub link: fn(&mut S) -> Option<Box<[u8]>>,
    pub local_link: fn(&mut S) -> Option<Box<[u8]>>,
    pub check_wait_boundary: fn(&S, skein_lib::Time),
    pub adopt_listener: fn(&mut S, Fd, std::net::SocketAddr) -> Result<(), Fd>,
    pub adopt_signals: fn(&mut S, Fd) -> Result<(), Fd>,
}

/// A start's service, its store's path (empty for a worker alone), and the binary's adoption state.
pub struct Prepared<S, A = ()> {
    pub service: S,
    pub store: Box<[u8]>,
    pub adoption: A,
}
/// Run serve on the binary's service. Its hooks check inputs before reading the document,
/// prepare the service, then adopt binary-specific descriptors after the inherited listeners.
pub fn run<S, B, A>(
    binary: &str,
    directories: &Directories,
    environment: &[EnvironmentEntry],
    served: Served<S>,
    before: impl FnOnce(usize) -> Result<B, String>,
    prepare: impl FnOnce(&[u8], &mut Vec<Fd>, B) -> Result<Prepared<S, A>, String>,
    adopt: impl FnOnce(&mut S, &mut Vec<Fd>, A) -> Result<(), String>,
) -> Result<Exit, String> {
    run_inner(binary, directories, environment, served, before, prepare, adopt)
        .map_err(|why| prefixed(&format!("{binary} serve"), &why))
}
fn run_inner<S, B, A>(
    binary: &str,
    directories: &Directories,
    environment: &[EnvironmentEntry],
    served: Served<S>,
    before: impl FnOnce(usize) -> Result<B, String>,
    prepare: impl FnOnce(&[u8], &mut Vec<Fd>, B) -> Result<Prepared<S, A>, String>,
    adopt: impl FnOnce(&mut S, &mut Vec<Fd>, A) -> Result<(), String>,
) -> Result<Exit, String> {
    let link_file = local_link(directories, environment);
    let listeners = skein_shell::inherited_listeners().map_err(listeners_refusal)?;
    let input = before(listeners.len())?;
    let bytes = directories.read()?;
    let mut owned = listeners.iter().map(|(fd, _)| *fd).collect::<Vec<_>>();
    let Prepared { mut service, store, adoption } = match prepare(&bytes, &mut owned, input) {
        Ok(prepared) => prepared,
        Err(why) => {
            close(owned);
            return Err(why);
        }
    };
    for (fd, addr) in listeners {
        if let Err(fd) = (served.adopt_listener)(&mut service, fd, addr) {
            close(owned);
            return Err(format!("listener: fd {} at {addr} refused", fd.raw()));
        }
    }
    if let Err(why) = adopt(&mut service, &mut owned, adoption) {
        close(owned);
        return Err(why);
    }
    let opened = skein_shell::open_termination_signals()
        .map_err(|errno| format!("signals: errno {errno}"))
        .and_then(|signals| {
            owned.push(signals);
            (served.adopt_signals)(&mut service, signals).map_err(|_| "signals refused".into())
        })
        .and_then(|()| {
            Kernel::open(Config { operations: (served.operations)(&service) }).map_err(|why| why.to_string())
        });
    let mut kernel = match opened {
        Ok(kernel) => kernel,
        Err(why) => {
            close(owned);
            return Err(why);
        }
    };
    let mut host = Host::with_binary(binary, service, served, directories.clone(), environment);
    host.store = store;
    host.link_file = link_file;
    let exit = skein_shell::drive(&mut kernel, &Clock::new(), &mut host);
    if let Some(path) = &host.link_file {
        link_file::forget(path);
    }
    Ok(exit)
}

fn local_link(directories: &Directories, environment: &[EnvironmentEntry]) -> Option<std::path::PathBuf> {
    link_file::path(
        &directories.data,
        temper_shell::environment::XDG_RUNTIME_DIR.value(environment),
        cfg!(target_os = "linux"),
    )
}

#[cfg(test)]
mod tests;

/// Opt in to the service's wait-boundary check (cli/README.md, 8).
pub const WAIT_CHECK: skein_shell::Variable = skein_shell::Variable {
    name: "TEMPER_INTERNAL_WAIT_CHECK",
    set: skein_shell::Set::Internal,
    form: "SWITCH",
    line: "Check the service wait boundary.",
    absent: "Check in debug builds only.",
    notes: "",
    way: skein_shell::Way::Reads,
};
/// Cut a start at its first client listener (cli/README.md, 8).
pub const CUT_AT_LISTEN: skein_shell::Variable = skein_shell::Variable {
    name: "TEMPER_INTERNAL_CUT_AT_LISTEN",
    set: skein_shell::Set::Internal,
    form: "FILE",
    line: "Abort once at the client listener, creating this marker.",
    absent: "Do not cut the start.",
    notes: "Requires TEMPER_INTERNAL_WAIT_CHECK. An existing marker keeps the next start running.",
    way: skein_shell::Way::Reads,
};
