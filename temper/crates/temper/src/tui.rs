//! The TUI's arguments and startup (temper/docs/design/client/README.md, 6.11 to 6.16).
use crate::{commands, directories, open};
use skein_io::{
    file::Expect,
    kernel::{Exit, Fd},
};
use skein_lib::Duration;
use skein_shell::{Clock, Config, Host, Input, Kernel, Keys, Watch};
use skein_ui_screen_terminal::{Depth, LEAVE};
use std::{
    net::SocketAddr,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::{Path, PathBuf},
};
use temper_client_shell_tui::{
    limits,
    run::{Run, Settings},
};
use temper_client_tui::Scheme;

type EnvironmentEntry = (Box<[u8]>, Box<[u8]>);

#[derive(Debug, PartialEq, Eq)]
pub enum Refused {
    NotATerminal,
    NotAnAddress(Box<[u8]>),
    Link,
    NotLoopback,
    Directory(String),
}
impl Refused {
    #[must_use]
    pub fn line(&self) -> String {
        let why = match self {
            Self::NotATerminal => "the TUI needs a terminal on stdin and stdout".into(),
            Self::NotAnAddress(argument) => format!("not an address: {}", String::from_utf8_lossy(argument)),
            Self::Link => "a sign-in link is never an argument: run temper http://HOST:PORT and paste it".into(),
            Self::NotLoopback => "the engine's address must be on loopback".into(),
            Self::Directory(why) => why.clone(),
        };
        format!("temper: {why}\n")
    }
}

pub fn address(argument: &[u8]) -> Result<SocketAddr, Refused> {
    match open::target(argument) {
        Ok(open::Target::Address { engine }) => Ok(engine),
        Ok(open::Target::Link { .. }) => Err(Refused::Link),
        Err(open::Refused::NotLoopback) => Err(Refused::NotLoopback),
        Err(
            open::Refused::Usage
            | open::Refused::NotALink(_)
            | open::Refused::Directory(_)
            | open::Refused::NotSignedIn
            | open::Refused::LinkRefused,
        ) => Err(Refused::NotAnAddress(argument.into())),
    }
}

pub fn directory(environment: &[EnvironmentEntry]) -> Result<PathBuf, Refused> {
    let absolute = |name| lookup(environment, name).filter(|value| value.starts_with(b"/"));
    let base = if let Some(state) = absolute(b"XDG_STATE_HOME") {
        crate::path(state).to_path_buf()
    } else if let Some(home) = absolute(b"HOME") {
        crate::path(home).join(".local/state")
    } else {
        return Err(Refused::Directory("TUI directory: XDG_STATE_HOME or HOME must be an absolute path".into()));
    };
    Ok(base.join("temper/tui"))
}

#[must_use]
pub fn kept(bytes: &[u8]) -> Option<SocketAddr> {
    address(bytes).ok()
}

fn lookup<'a>(environment: &'a [EnvironmentEntry], name: &[u8]) -> Option<&'a [u8]> {
    environment.iter().find(|(key, _)| key.as_ref() == name).map(|(_, value)| value.as_ref())
}

#[must_use]
pub fn opener(
    environment: &[EnvironmentEntry],
    linux: bool,
    executable: &dyn Fn(&Path) -> bool,
) -> Option<temper_client_shell_tui::opener::Opener> {
    if lookup(environment, b"SSH_CONNECTION").is_some()
        || lookup(environment, b"SSH_TTY").is_some()
        || (linux && lookup(environment, b"DISPLAY").is_none() && lookup(environment, b"WAYLAND_DISPLAY").is_none())
    {
        return None;
    }
    let program = if linux { "xdg-open" } else { "open" };
    for directory in lookup(environment, b"PATH")?.split(|byte| *byte == b':') {
        if !directory.starts_with(b"/") {
            continue;
        }
        let path = crate::path(directory).join(program);
        if executable(&path) {
            let env = environment
                .iter()
                .map(|(name, value)| {
                    let mut entry = name.to_vec();
                    entry.push(b'=');
                    entry.extend_from_slice(value);
                    entry.into_boxed_slice()
                })
                .collect();
            return Some(temper_client_shell_tui::opener::Opener { program: path.as_os_str().as_bytes().into(), env });
        }
    }
    None
}

#[must_use]
pub fn scheme(environment: &[EnvironmentEntry]) -> Scheme {
    match lookup(environment, b"COLORFGBG").and_then(|value| value.rsplit(|byte| *byte == b';').next()) {
        Some(b"7" | b"15") => Scheme::Light,
        _ => Scheme::Dark,
    }
}

#[must_use]
pub fn depth(environment: &[EnvironmentEntry]) -> Depth {
    match lookup(environment, b"COLORTERM") {
        Some(b"truecolor" | b"24bit") => Depth::True,
        _ => Depth::Indexed,
    }
}

pub(crate) fn terminal_refusal(given: bool) -> String {
    if given { Refused::NotATerminal.line() } else { commands::unknown(&[]) }
}

#[expect(clippy::needless_pass_by_value, reason = "the command transfers its owned address to the TUI entry point")]
pub(crate) fn run(argument: Option<Box<[u8]>>, clock: &Clock) -> u8 {
    if skein_shell::terminal_modes(Fd::new(0)).is_err() || skein_shell::terminal_modes(Fd::new(1)).is_err() {
        eprint!("{}", terminal_refusal(argument.is_some()));
        return 1;
    }
    match start(argument.as_deref(), clock) {
        Ok(code) => code,
        Err(why) => {
            eprint!("{}", why.line());
            1
        }
    }
}

fn start(argument: Option<&[u8]>, clock: &Clock) -> Result<u8, Refused> {
    let given = argument.map(address).transpose()?;
    let environment = skein_shell::environment();
    let directory = directory(&environment)?;
    let user = skein_shell::effective_user();
    directories::make_private(&directory, user).map_err(Refused::Directory)?;
    let saved = skein_shell::read_private_file(&directory.join("address"), 64, user)
        .ok()
        .and_then(|bytes| kept(bytes.expose()));
    let engine = given.or(saved).unwrap_or(SocketAddr::from(([127, 0, 0, 1], 8367)));
    let scheme = scheme(&environment);
    let depth = depth(&environment);
    let Ok(Input::Terminal { kept, size }) = skein_shell::raw_input(Fd::new(0), Keys::Pass) else {
        return Err(Refused::NotATerminal);
    };
    skein_fail_stop::before_report(Some(kept.on_abort(Fd::new(1), LEAVE)));
    let parent = directory.parent().expect("TUI directory has a parent");
    let result = (|| {
        let signals = skein_shell::open_signals(Watch::TerminationAndWindow).expect("TUI signals");
        let root = skein_shell::open_root(parent).map_err(|number| {
            skein_shell::close_startup_fd(signals);
            Refused::Directory(format!("{}: cannot open its parent: errno {number}", directory.display()))
        })?;
        #[cfg(feature = "record")]
        let record = Some(skein_shell::open_root(parent).map_err(|number| {
            skein_shell::close_startup_fd(root);
            skein_shell::close_startup_fd(signals);
            Refused::Directory(format!("{}: cannot open recording parent: errno {number}", directory.display()))
        })?);
        #[cfg(not(feature = "record"))]
        let record = None;
        let settings = Settings {
            record,
            engine: skein_io::Addr::Ip(engine),
            directory: Box::from(b"tui".as_slice()),
            address: engine.to_string().into_bytes().into_boxed_slice(),
            scheme,
            depth,
            size: skein_ui_screen::Size { rows: size.rows, columns: size.columns },
            limits: limits::tui(),
            opener: opener(&environment, cfg!(target_os = "linux"), &|path| {
                std::fs::metadata(path)
                    .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
            }),
        };
        let seed = skein_shell::seed().expect("kernel seed");
        let mut run = Run::new(settings, root, user, seed, Fd::new(0), Fd::new(1), signals).expect("TUI descriptors");
        let mut kernel = Kernel::open(Config { operations: run.operations() }).expect("TUI kernel");
        let exit = skein_shell::drive(&mut kernel, clock, &mut run);
        Ok((run, exit, seed))
    })();
    skein_fail_stop::before_report(None);
    kept.restore().expect("restore terminal modes");
    let (run, exit, seed) = result?;
    if run.refused() {
        return Err(Refused::Directory(format!("{}: cannot open its files", directory.display())));
    }
    if let Some(signed_in) = run.signed_in() {
        let mut bytes = b"http://".to_vec();
        bytes.extend_from_slice(signed_in);
        if crate::tui::kept(&bytes) != saved {
            keep_address(parent, bytes.into_boxed_slice(), user, seed, clock);
        }
    }
    Ok(match exit {
        Exit::Code(code) => code,
        Exit::Signal(_) => 1,
    })
}

fn keep_address(parent: &std::path::Path, bytes: Box<[u8]>, user: u32, seed: u64, clock: &Clock) {
    let Ok(mut kernel) = Kernel::open(Config { operations: skein_shell::STORE_OPERATIONS }) else { return };
    let Ok(root) = skein_shell::open_root(parent) else { return };
    let store = skein_shell::StartupStore {
        root,
        directory: skein_shell::StoreDirectory::Private { name: b"tui", user },
        name: b"address",
        bytes,
        expected: Expect::Any,
        timeout: Duration::from_secs(5),
        seed,
    };
    let _ = skein_shell::store_file(&mut kernel, clock, store);
}
