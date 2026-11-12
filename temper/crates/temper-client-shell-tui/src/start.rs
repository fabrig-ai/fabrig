//! The TUI's arguments and start (temper/docs/design/client/tui/README.md, 1 and 2).
use crate::{
    limits,
    run::{Run, Settings},
};
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
use temper_client_shell_terminal::address as engine_address;
use temper_client_tui::{Note, Scheme};
use temper_shell::{directories, link_file};

pub type EnvironmentEntry = (Box<[u8]>, Box<[u8]>);

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
    pub fn line(&self, name: &str) -> String {
        let why = match self {
            Self::NotATerminal => "the TUI needs a terminal on stdin and stdout".into(),
            Self::NotAnAddress(argument) => format!("not an address: {}", String::from_utf8_lossy(argument)),
            Self::Link => format!("a sign-in link is never an argument: run {name} http://HOST:PORT and paste it"),
            Self::NotLoopback => "the engine's address must be on loopback".into(),
            Self::Directory(why) => why.clone(),
        };
        format!("{name}: {why}\n")
    }
}

pub fn address(argument: &[u8]) -> Result<SocketAddr, Refused> {
    match engine_address::target(argument) {
        Ok(engine_address::Target::Address { engine }) => Ok(engine),
        Ok(engine_address::Target::Link { .. }) => Err(Refused::Link),
        Err(engine_address::Refused::NotLoopback) => Err(Refused::NotLoopback),
        Err(engine_address::Refused::NotALink(_)) => Err(Refused::NotAnAddress(argument.into())),
    }
}

pub fn directory(environment: &[EnvironmentEntry]) -> Result<PathBuf, Refused> {
    let absolute =
        |variable: &skein_shell::Variable| variable.value(environment).filter(|value| value.starts_with(b"/"));
    let base = if let Some(state) = absolute(skein_shell::Base::State.variable()) {
        path(state).to_path_buf()
    } else if let Some(home) = absolute(&skein_shell::HOME) {
        path(home).join(".local/state")
    } else {
        return Err(Refused::Directory("TUI directory: XDG_STATE_HOME or HOME must be an absolute path".into()));
    };
    Ok(base.join("temper/tui"))
}

/// Whether the captured environment requests recording (temper/docs/design/cli/README.md, 8).
#[must_use]
pub fn recording(environment: &[EnvironmentEntry]) -> bool {
    temper_shell::environment::INTERNAL_RECORD.value(environment).is_some()
}

#[must_use]
pub fn kept(bytes: &[u8]) -> Option<SocketAddr> {
    address(bytes).ok()
}

#[must_use]
pub fn selected_address(given: Option<SocketAddr>, link: Option<&[u8]>, saved: Option<SocketAddr>) -> SocketAddr {
    given
        .or_else(|| match engine_address::target(link?) {
            Ok(engine_address::Target::Link { engine, .. }) => Some(engine),
            Ok(engine_address::Target::Address { .. }) | Err(_) => None,
        })
        .or(saved)
        .unwrap_or(SocketAddr::from(([127, 0, 0, 1], 8367)))
}

#[must_use]
pub fn on_box(link: Option<&[u8]>, engine: SocketAddr) -> bool {
    match link.map(engine_address::target) {
        Some(Ok(engine_address::Target::Link { engine: linked, .. })) => linked == engine,
        Some(Ok(engine_address::Target::Address { .. }) | Err(_)) | None => false,
    }
}

#[must_use]
pub fn link_path(environment: &[EnvironmentEntry], working: &[u8]) -> Option<PathBuf> {
    let data = link_file::deployment(environment, working)?;
    link_file::path(&data, temper_shell::environment::XDG_RUNTIME_DIR.value(environment), cfg!(target_os = "linux"))
}

#[must_use]
pub fn read_note(directory: &Path, engine: SocketAddr, user: u32) -> Option<Note> {
    let bytes = skein_shell::read_private_file(&directory.join("sign-in"), Note::MOST, user).ok()?;
    Note::read(bytes.expose(), engine.to_string().as_bytes())
}

#[must_use]
pub fn opener(
    environment: &[EnvironmentEntry],
    linux: bool,
    executable: &dyn Fn(&Path) -> bool,
) -> Option<crate::opener::Opener> {
    if temper_shell::environment::SSH_CONNECTION.value(environment).is_some()
        || temper_shell::environment::SSH_TTY.value(environment).is_some()
        || (linux
            && temper_shell::environment::DISPLAY.value(environment).is_none()
            && temper_shell::environment::WAYLAND_DISPLAY.value(environment).is_none())
    {
        return None;
    }
    let program = if linux { "xdg-open" } else { "open" };
    for directory in temper_shell::environment::PATH.value(environment)?.split(|byte| *byte == b':') {
        if !directory.starts_with(b"/") {
            continue;
        }
        let path = path(directory).join(program);
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
            return Some(crate::opener::Opener { program: path.as_os_str().as_bytes().into(), env });
        }
    }
    None
}

#[must_use]
pub fn scheme(environment: &[EnvironmentEntry]) -> Option<Scheme> {
    let background = temper_shell::environment::COLORFGBG.value(environment)?.rsplit(|byte| *byte == b';').next()?;
    if background.is_empty() || !background.iter().all(u8::is_ascii_digit) {
        return None;
    }
    Some(match std::str::from_utf8(background).ok()?.trim_start_matches('0') {
        "7" | "15" => Scheme::Light,
        _ => Scheme::Dark,
    })
}

#[must_use]
pub fn depth(environment: &[EnvironmentEntry]) -> Depth {
    match temper_shell::environment::COLORTERM.value(environment) {
        Some(b"truecolor" | b"24bit") => Depth::True,
        _ => Depth::Indexed,
    }
}

#[must_use]
pub fn terminal_refusal(name: &str, given: bool) -> String {
    if given { Refused::NotATerminal.line(name) } else { format!("{name}: no command; {name} --help lists them\n") }
}

#[must_use]
pub fn run(name: &str, argument: Option<&[u8]>, clock: &Clock) -> u8 {
    if skein_shell::terminal_modes(Fd::new(0)).is_err() || skein_shell::terminal_modes(Fd::new(1)).is_err() {
        eprint!("{}", terminal_refusal(name, argument.is_some()));
        return 1;
    }
    match start(argument, clock) {
        Ok(code) => code,
        Err(why) => {
            eprint!("{}", why.line(name));
            1
        }
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "cli C3: the client's environment selects directories, links and terminal settings; temper's table will pass it"
)]
fn start(argument: Option<&[u8]>, clock: &Clock) -> Result<u8, Refused> {
    let given = argument.map(address).transpose()?;
    let environment = skein_shell::environment();
    let directory = directory(&environment)?;
    let user = skein_shell::effective_user();
    directories::make_private(&directory, user).map_err(Refused::Directory)?;
    let saved = skein_shell::read_private_file(&directory.join("address"), 64, user)
        .ok()
        .and_then(|bytes| kept(bytes.expose()));
    let link_file =
        std::env::current_dir().ok().and_then(|working| link_path(&environment, working.as_os_str().as_bytes()));
    let (engine, boxed) = {
        let link = link_file.as_deref().and_then(|path| link_file::read(path, user));
        let bytes = link.as_ref().map(skein_lib::Secret::expose);
        let engine = selected_address(given, if given.is_none() { bytes } else { None }, saved);
        (engine, on_box(bytes, engine))
    };
    let note = read_note(&directory, engine, user);
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
        let files = skein_shell::open_root(parent).map_err(|number| {
            skein_shell::close_startup_fd(root);
            skein_shell::close_startup_fd(signals);
            Refused::Directory(format!("{}: cannot open sign-in parent: errno {number}", directory.display()))
        })?;
        let record = if recording(&environment) {
            Some(skein_shell::open_root(parent).map_err(|number| {
                skein_shell::close_startup_fd(files);
                skein_shell::close_startup_fd(root);
                skein_shell::close_startup_fd(signals);
                Refused::Directory(format!("{}: cannot open recording parent: errno {number}", directory.display()))
            })?)
        } else {
            None
        };
        let settings = Settings {
            record,
            front: None,
            engine: skein_io::Addr::Ip(engine),
            directory: Box::from(b"tui".as_slice()),
            address: engine.to_string().into_bytes().into_boxed_slice(),
            scheme,
            boxed,
            link_file: link_file.map(|path| path.as_os_str().as_bytes().into()),
            note,
            files: Some(files),
            depth,
            size: skein_ui_screen::Size { rows: size.rows, columns: size.columns },
            limits: limits::tui(),
            opener: opener(&environment, cfg!(target_os = "linux"), &|path| {
                std::fs::metadata(path)
                    .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
            }),
        };
        let seed = skein_shell::seed().expect("kernel seed");
        let mut run = local_run(settings, root, user, seed, signals);
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
        if self::kept(&bytes) != saved {
            keep_address(parent, bytes.into_boxed_slice(), user, seed, clock);
        }
    }
    Ok(match exit {
        Exit::Code(code) => code,
        Exit::Signal(_) => 1,
    })
}

#[must_use]
pub fn local_run(mut settings: Settings, root: Fd, user: u32, seed: u64, signals: Fd) -> Run {
    let link = settings.link_file.take();
    let run = Run::new(settings, root, user, seed, Fd::new(0), Fd::new(1), signals).expect("TUI descriptors");
    let Some(path) = link else { return run };
    let Some(parent) = self::path(&path).parent() else { return run };
    let Ok(root) = skein_shell::open_root(parent) else { return run };
    run.with_local_link(root, &path)
}

pub fn keep_address(parent: &std::path::Path, bytes: Box<[u8]>, user: u32, seed: u64, clock: &Clock) {
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

fn path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}

#[cfg(test)]
mod tests;
