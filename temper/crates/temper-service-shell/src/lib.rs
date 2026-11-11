//! Serve's run, its host and its lines, for every binary that serves (temper/docs/design/protocol/binaries.md, 1 and temper/docs/design/protocol/components.md).
use jig_protocol_config as config;
use skein_io::kernel::{Exit, Fd};
use skein_lib::{List, Secret};
use skein_shell::{Clock, Config, Kernel};
use std::{
    fmt::Write,
    os::unix::{ffi::OsStrExt, fs::FileTypeExt},
    path::Path,
};
use temper_service::{self as service, settings::Files};
use temper_shell::directories::EnvironmentEntry;
use temper_shell::{
    directories::{Directories, errno, unread},
    link_file,
};
pub mod host;
mod restart_fault;
pub use host::{Host, ending};

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
#[must_use]
pub fn problems(binary: &str, found: &service::startup::Problems) -> String {
    prefixed(&format!("{binary} serve"), &String::from_utf8_lossy(&service::startup::lines(found)))
}
#[must_use]
pub fn file_path(at: config::Path) -> String {
    let mut found = service::startup::Problems::empty();
    found.push(at, service::startup::What::Jig(config::What::Unreadable));
    let bytes = service::startup::lines(&found);
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
fn box_refusal(why: service::system::BoxRefusal) -> String {
    use service::system::BoxRefusal;
    let words = match why {
        BoxRefusal::Sockets { limit } => {
            format!("the box's pipes need two free socket slots; the service socket limit is {limit}")
        }
        BoxRefusal::Adopted => "the box's pipes were already adopted".into(),
        BoxRefusal::NoEngine => "the box's pipes need an engine".into(),
        BoxRefusal::ZeroLine => "the box's pipes need a nonzero application.limits.overrides.box.line_bytes".into(),
    };
    format!("--box: {words}")
}
/// The first links' secrets on a boxed serve's standard input (protocol/setup/box.md, 1).
fn given_secret(input: &mut impl std::io::Read) -> Result<(Option<Secret>, Option<Secret>), String> {
    use std::io::Read;
    let both = service::clients::LINK_SECRET_BYTES.checked_mul(2).expect("two secrets");
    let limit = both.checked_add(1).expect("small secret bound");
    let mut bytes = Vec::with_capacity(limit);
    input
        .take(u64::try_from(limit).expect("small secret bound"))
        .read_to_end(&mut bytes)
        .map_err(|_| "--box: stdin: expected the first link's secret".to_string())?;
    if bytes.is_empty() {
        Ok((None, None))
    } else if bytes.len() == service::clients::LINK_SECRET_BYTES {
        Ok((Some(Secret::new(bytes.into_boxed_slice())), None))
    } else if bytes.len() == both {
        let local = bytes.split_off(service::clients::LINK_SECRET_BYTES);
        Ok((Some(Secret::new(bytes.into_boxed_slice())), Some(Secret::new(local.into_boxed_slice()))))
    } else {
        Err("--box: stdin: expected the first link's secret".into())
    }
}

/// A start's service, its store's path (empty for a worker alone), and the box's line bound (0 outside a box).
pub struct Prepared {
    pub service: temper_service::Service,
    pub store: Box<[u8]>,
    pub line_bytes: u32,
}
/// The first links' secrets a boxed serve reads on its standard input (protocol/setup/box.md, 1).
pub struct Secrets {
    pub link: Option<skein_lib::Secret>,
    pub local: Option<skein_lib::Secret>,
}
/// Run serve for `binary` on the start `prepare` makes from the document's bytes, keeping the descriptors it opens in `owned`.
pub fn run(
    binary: &str,
    directories: &Directories,
    environment: &[EnvironmentEntry],
    boxed: bool,
    prepare: impl FnOnce(&[u8], &mut Vec<Fd>, Secrets) -> Result<Prepared, String>,
) -> Result<Exit, String> {
    run_inner(binary, directories, environment, boxed, prepare)
        .map_err(|why| prefixed(&format!("{binary} serve"), &why))
}
fn run_inner(
    binary: &str,
    directories: &Directories,
    environment: &[EnvironmentEntry],
    boxed: bool,
    prepare: impl FnOnce(&[u8], &mut Vec<Fd>, Secrets) -> Result<Prepared, String>,
) -> Result<Exit, String> {
    let link_file = local_link(directories, environment);
    let listeners = skein_shell::inherited_listeners().map_err(listeners_refusal)?;
    let requests = 3 + i32::try_from(listeners.len()).expect("bounded inherited listeners");
    let answers = requests + 1;
    if boxed && cfg!(target_os = "linux") {
        for fd in [requests, answers] {
            if !std::fs::metadata(format!("/proc/self/fd/{fd}")).is_ok_and(|m| m.file_type().is_fifo()) {
                return Err(format!("--box: fd {fd} is not an open pipe"));
            }
        }
    }
    let (link_secret, local_secret) = if boxed { given_secret(&mut std::io::stdin().lock())? } else { (None, None) };
    let bytes = directories.read()?;
    let mut owned = listeners.iter().map(|(fd, _)| *fd).collect::<Vec<_>>();
    let prepared = prepare(&bytes, &mut owned, Secrets { link: link_secret, local: local_secret });
    let Prepared { mut service, store, line_bytes } = match prepared {
        Ok(prepared) => prepared,
        Err(why) => {
            close(owned);
            return Err(why);
        }
    };
    for (fd, addr) in listeners {
        if let Err(fd) = service.adopt_listener(fd, addr) {
            close(owned);
            return Err(format!("listener: fd {} at {addr} refused", fd.raw()));
        }
    }
    if boxed {
        owned.extend([Fd::new(requests), Fd::new(answers)]);
        if let Err(why) = service.adopt_box(Fd::new(requests), Fd::new(answers), line_bytes) {
            close(owned);
            return Err(box_refusal(why));
        }
    }
    let opened = skein_shell::open_termination_signals()
        .map_err(|errno| format!("signals: errno {errno}"))
        .and_then(|signals| {
            owned.push(signals);
            service.adopt_signals(signals).map_err(|_| "signals refused".into())
        })
        .and_then(|()| Kernel::open(Config { operations: service.operations() }).map_err(|why| why.to_string()));
    let mut kernel = match opened {
        Ok(kernel) => kernel,
        Err(why) => {
            close(owned);
            return Err(why);
        }
    };
    let mut host = Host::with_binary(binary, service, directories.clone(), environment);
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
