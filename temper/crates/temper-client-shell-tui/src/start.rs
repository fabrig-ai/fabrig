//! The TUI's arguments and start (temper/docs/design/client/tui/README.md, 1 and 2).
use crate::{
    limits,
    run::{Front, Run, Settings},
};
use skein_io::{
    file::Expect,
    kernel::{Exit, Fd},
};
use skein_lib::Duration;
use skein_shell::{Clock, Config, Host, Input, Kernel, Keys, Watch};
use skein_ui_screen_terminal::{Depth, LEAVE};
use std::{
    net::{SocketAddr, ToSocketAddrs},
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::{Path, PathBuf},
};
use temper_client_shell_terminal::address as engine_address;
use temper_client_tui::{Note, Origin, Scheme, Unaddressed};
use temper_shell::{directories, link_file};

mod deployment;

pub type EnvironmentEntry = (Box<[u8]>, Box<[u8]>);

#[derive(Debug, PartialEq, Eq)]
pub enum Refused {
    NotATerminal,
    NotAnAddress(Box<[u8]>),
    Link,
    NotLoopback,
    HttpName,
    HttpsAddress,
    Resolve { name: String, why: String },
    Roots(String),
    Directory(String),
}
impl Refused {
    #[must_use]
    pub fn line(&self, name: &str) -> String {
        format!("{name}: {}\n", self.reason(name))
    }

    #[must_use]
    pub fn reason(&self, name: &str) -> String {
        match self {
            Self::NotATerminal => "the TUI needs a terminal on stdin and stdout".into(),
            Self::NotAnAddress(argument) => format!("not an address: {}", String::from_utf8_lossy(argument)),
            Self::Link => format!("a sign-in link is never an argument: run {name} http://HOST:PORT and paste it"),
            Self::NotLoopback => "the engine's address must be on loopback".into(),
            Self::HttpName => "http:// reaches an engine on this machine alone: use https://NAME".into(),
            Self::HttpsAddress => "https:// needs the deployment's name".into(),
            Self::Resolve { name, why } => format!("can't find {name}: {why}"),
            Self::Roots(why) => format!("the machine's trust roots: {why}"),
            Self::Directory(why) => why.clone(),
        }
    }
}

pub fn address(argument: &[u8]) -> Result<Origin, Refused> {
    // A link is never an argument, even when its secret is malformed (client/tui/README.md, 1).
    if let Some(rest) = std::str::from_utf8(argument).ok().and_then(|text| text.strip_prefix("http://"))
        && let Some((host, path)) = rest.split_once('/')
        && host.parse::<SocketAddr>().is_ok()
        && let Some((path, _)) = path.split_once("?secret=")
        && (path == "jig/v1/sign-in/link"
            || Some(path.as_bytes()) == temper_client_domain::sign_in::LINK_PAGE.strip_prefix(b"/"))
    {
        return Err(Refused::Link);
    }
    Origin::parse(argument).map_err(|why| match why {
        Unaddressed::NotAnAddress => Refused::NotAnAddress(argument.into()),
        Unaddressed::HttpName => Refused::HttpName,
        Unaddressed::HttpsAddress => Refused::HttpsAddress,
        Unaddressed::OffLoopback => Refused::NotLoopback,
    })
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
pub fn kept(bytes: &[u8]) -> Option<Origin> {
    if bytes.len() > 300 {
        return None;
    }
    Origin::parse(bytes).ok()
}

#[must_use]
pub fn selected_address(given: Option<Origin>, link: Option<&[u8]>, saved: Option<Origin>) -> Origin {
    given
        .or_else(|| match engine_address::target(link?) {
            Ok(engine_address::Target::Link { engine, .. }) => Some(Origin::Http(engine)),
            Ok(engine_address::Target::Address { .. }) | Err(_) => None,
        })
        .or(saved)
        .unwrap_or(Origin::Http(SocketAddr::from(([127, 0, 0, 1], 8367))))
}

#[must_use]
pub fn on_box(link: Option<&[u8]>, engine: &Origin) -> bool {
    match link.map(engine_address::target) {
        Some(Ok(engine_address::Target::Link { engine: linked, .. })) => *engine == Origin::Http(linked),
        Some(Ok(engine_address::Target::Address { .. }) | Err(_)) | None => false,
    }
}

// An explicit deployment never reads a local engine's link, including for box detection.
fn select(
    given: Option<Origin>,
    saved: Option<Origin>,
    link_file: Option<PathBuf>,
    read: &dyn Fn(&Path) -> Option<skein_lib::Secret>,
) -> (Origin, Option<PathBuf>, bool) {
    let link = if let Some(Origin::Https { .. }) = &given { None } else { link_file.as_deref().and_then(read) };
    let bytes = link.as_ref().map(skein_lib::Secret::expose);
    let origin = selected_address(given, bytes, saved);
    let boxed = on_box(bytes, &origin);
    let link_file = match &origin {
        Origin::Http(_) => link_file,
        Origin::Https { .. } => None,
    };
    (origin, link_file, boxed)
}

fn engine(
    origin: &Origin,
    environment: &[EnvironmentEntry],
    resolve: &dyn Fn(&str, u16) -> std::io::Result<Vec<SocketAddr>>,
) -> Result<jig_client_shell_native::Engine, Refused> {
    use jig_client_shell_native::{Engine, Tls};
    match origin {
        Origin::Http(socket) => Ok(Engine::Plain(skein_io::Addr::Ip(*socket))),
        Origin::Https { name, port } => {
            let text = std::str::from_utf8(name).expect("parsed origin has an ASCII name");
            let name = skein_tls::Name::new(text).ok_or_else(|| Refused::NotAnAddress(origin.print()))?;
            let addresses =
                resolve(text, *port).map_err(|why| Refused::Resolve { name: text.into(), why: why.to_string() })?;
            if addresses.is_empty() {
                return Err(Refused::Resolve { name: text.into(), why: "no address".into() });
            }
            let trust = skein_shell::Trust::Machine {
                ssl_cert_file: temper_shell::environment::SSL_CERT_FILE
                    .value(environment)
                    .map(|bytes| path(bytes).into()),
            };
            let roots = skein_shell::read_trust_roots(
                &trust,
                &skein_shell::RootBounds { bundle_bytes: 1 << 20, certificates: 1024, certificate_bytes: 65_536 },
            )
            .map_err(|why| {
                Refused::Roots(match why {
                    skein_shell::RootsRefusal::Unread { path, why } => format!("{}: {why:?}", path.display()),
                    other @ (skein_shell::RootsRefusal::NoBundle
                    | skein_shell::RootsRefusal::TooMany { .. }
                    | skein_shell::RootsRefusal::TooLong { .. }) => format!("{other:?}"),
                })
            })?;
            let (config, _) = skein_tls::Config::from_der(&roots.certificates, &[b"http/1.1"])
                .map_err(|why| Refused::Roots(format!("{why:?}")))?;
            Ok(Engine::Tls(Tls {
                name,
                host: origin.print().strip_prefix(b"https://").expect("HTTPS origin").into(),
                addresses: addresses.into(),
                config,
            }))
        }
    }
}

#[must_use]
pub fn link_path(environment: &[EnvironmentEntry], working: &[u8]) -> Option<PathBuf> {
    let data = link_file::deployment(environment, working)?;
    link_file::path(&data, temper_shell::environment::XDG_RUNTIME_DIR.value(environment), cfg!(target_os = "linux"))
}

#[must_use]
pub fn read_note(directory: &Path, engine: &Origin, user: u32) -> Option<Note> {
    let bytes = skein_shell::read_private_file(&directory.join("sign-in"), Note::MOST, user).ok()?;
    Note::read(bytes.expose(), engine)
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
pub fn plain(environment: &[EnvironmentEntry]) -> bool {
    temper_shell::environment::TERM_PROGRAM.value(environment) == Some(b"Apple_Terminal".as_slice())
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
pub fn run(
    name: &str,
    argument: Option<&[u8]>,
    environment: &[EnvironmentEntry],
    working: &[u8],
    version: &[u8],
    clock: &Clock,
) -> u8 {
    if skein_shell::terminal_modes(Fd::new(0)).is_err() || skein_shell::terminal_modes(Fd::new(1)).is_err() {
        eprint!("{}", terminal_refusal(name, argument.is_some()));
        return 1;
    }
    match start(argument, environment, working, version, clock, None) {
        Ok(code) => code,
        Err(why) => {
            eprint!("{}", why.line(name));
            1
        }
    }
}

struct BoxStart {
    executable: Box<[u8]>,
    directories: directories::Directories,
    arguments: Box<[Box<[u8]>]>,
    link: Option<PathBuf>,
    log: Box<[u8]>,
}

impl BoxStart {
    fn settings(self, environment: &[EnvironmentEntry], working: &[u8]) -> Result<Front, Refused> {
        let directory = skein_shell::open_root(path(working))
            .map_err(|number| Refused::Directory(format!("cannot open working directory: errno {number}")))?;
        Ok(Front {
            executable: self.executable,
            set_up: box_set_up(&self.directories.config, &|directory| {
                std::fs::read_dir(directory).map(|mut entries| entries.next().is_some())
            }),
            directories: self.directories,
            arguments: self.arguments,
            directory,
            log: self.log,
            environment: environment
                .iter()
                .map(|(name, value)| {
                    let mut entry = name.to_vec();
                    entry.push(b'=');
                    entry.extend_from_slice(value);
                    entry.into()
                })
                .collect(),
        })
    }
}

/// The one directory read beside the captured environment, before the fetch.
fn box_set_up(config: &[u8], read: &dyn Fn(&Path) -> std::io::Result<bool>) -> bool {
    match read(path(config)) {
        Ok(has_entry) => has_entry,
        Err(error) => error.kind() != std::io::ErrorKind::NotFound,
    }
}

fn box_log(data: &[u8]) -> Box<[u8]> {
    path(data).join("box/logs/box.log").as_os_str().as_bytes().into()
}

/// `temper box`'s terminal case; its arguments remain the box's to read.
#[must_use]
pub fn run_box(
    executable: &[u8],
    directories: &directories::Directories,
    arguments: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    working: &[u8],
    version: &[u8],
    clock: &Clock,
) -> u8 {
    let front = BoxStart {
        executable: executable.into(),
        directories: directories.clone(),
        arguments: arguments.into(),
        link: link_file::path(
            &directories.data,
            temper_shell::environment::XDG_RUNTIME_DIR.value(environment),
            cfg!(target_os = "linux"),
        ),
        log: box_log(&directories.data),
    };
    match start(None, environment, working, version, clock, Some(front)) {
        Ok(code) => code,
        Err(why) => {
            eprint!("{}", why.line("temper"));
            1
        }
    }
}

#[expect(clippy::too_many_lines, reason = "startup transfers kept modes into the run and restores them on every exit")]
fn start(
    argument: Option<&[u8]>,
    environment: &[EnvironmentEntry],
    working: &[u8],
    version: &[u8],
    clock: &Clock,
    front: Option<BoxStart>,
) -> Result<u8, Refused> {
    let given = argument.map(address).transpose()?;
    let directory = directory(environment)?;
    let user = skein_shell::effective_user();
    directories::make_private(&directory, user).map_err(Refused::Directory)?;
    let saved = skein_shell::read_private_file(&directory.join("address"), 300, user)
        .ok()
        .and_then(|bytes| kept(bytes.expose()));
    let (origin, link_file, boxed) =
        select(given, saved.clone(), link_path(environment, working), &|path| link_file::read(path, user));
    let engine = engine(&origin, environment, &|name, port| (name, port).to_socket_addrs().map(Iterator::collect))?;
    let (origin_directory, made) = deployment::prepare(&directory, &origin, user)?;
    let theme = deployment::theme(&directory, &origin, user, made);
    let note = read_note(&origin_directory, &origin, user);
    let link_file = front.as_ref().map_or(link_file, |front| front.link.clone());
    let scheme = scheme(environment);
    let depth = depth(environment);
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
        let record = if recording(environment) {
            Some(skein_shell::open_root(parent).map_err(|number| {
                skein_shell::close_startup_fd(files);
                skein_shell::close_startup_fd(root);
                skein_shell::close_startup_fd(signals);
                Refused::Directory(format!("{}: cannot open recording parent: errno {number}", directory.display()))
            })?)
        } else {
            None
        };
        let front = front.map(|front| front.settings(environment, working)).transpose().inspect_err(|_| {
            for fd in record.into_iter().chain([files, root, signals]) {
                skein_shell::close_startup_fd(fd);
            }
        })?;
        let in_front = front.is_some();
        let settings = Settings {
            record,
            front,
            engine,
            directory: Path::new("tui").join(deployment::relative(&origin)).as_os_str().as_bytes().into(),
            theme,
            address: origin,
            scheme,
            plain: plain(environment),
            version: version.into(),
            boxed,
            link_file: link_file.map(|path| path.as_os_str().as_bytes().into()),
            note,
            files: Some(files),
            depth,
            size: skein_ui_screen::Size { rows: size.rows, columns: size.columns },
            limits: limits::tui(),
            opener: opener(environment, cfg!(target_os = "linux"), &|path| {
                std::fs::metadata(path)
                    .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
            }),
        };
        let seed = skein_shell::seed().expect("kernel seed");
        let run = local_run(settings, root, user, seed, signals);
        let kernel = Kernel::open(Config { operations: run.operations() }).expect("TUI kernel");
        Ok((run, kernel, seed, in_front))
    })();
    let (run, mut kernel, seed, in_front) = match result {
        Ok(started) => started,
        Err(why) => {
            skein_fail_stop::before_report(None);
            kept.restore().expect("restore terminal modes");
            return Err(why);
        }
    };
    let mut run = run.with_kept(kept);
    let exit = if in_front {
        drive_front(&mut kernel, clock, &mut run)
    } else {
        skein_shell::drive(&mut kernel, clock, &mut run)
    };
    skein_fail_stop::before_report(None);
    run.restore_terminal();
    say_recording(&directory, run.recording_save(), &mut std::io::stderr().lock());
    if run.refused() {
        return Err(Refused::Directory(format!("{}: cannot open its files", directory.display())));
    }
    if run.terminated() {
        return Ok(1);
    }
    if let Some(signed_in) = run.signed_in() {
        let bytes = signed_in.print();
        if self::kept(&bytes) != saved {
            keep_address(parent, bytes, user, seed, clock);
        }
    }
    Ok(exit_code(&run, exit))
}

// The terminal has left before these words, and the box's closing words follow them.
fn say_recording(directory: &Path, saved: Option<(&[u8], bool)>, stderr: &mut impl std::io::Write) {
    if let Some((name, stored)) = saved {
        let path = directory.join(std::ffi::OsStr::from_bytes(name));
        let words = if stored { "recorded to" } else { "couldn't save the recording to" };
        let _ = writeln!(stderr, "temper: {words} {}", path.display());
    }
}

fn exit_code(run: &Run, exit: Exit) -> u8 {
    if let Some((code, words)) = run.box_exit() {
        use std::io::Write;
        let _ = std::io::stderr().lock().write_all(&words);
        return code;
    }
    match exit {
        Exit::Code(code) => code,
        Exit::Signal(_) => 1,
    }
}

// A termination signal is the process's shutdown path: the kernel may still own the box's
// wait and pipe reads. Its buffers stay owned by Kernel's shutdown path until process exit.
fn drive_front(kernel: &mut Kernel, clock: &Clock, run: &mut Run) -> Exit {
    loop {
        kernel.reap(run.completions());
        let skein_shell::Now { now, wall } = clock.now();
        run.iterate(now, wall);
        if run.terminated() {
            return Exit::Code(1);
        }
        if run.is_empty() {
            assert_eq!(kernel.in_flight(), 0, "a drained run has no kernel operation in flight");
            return run.exit().expect("a drained run's exit");
        }
        let deadline = clock.until(run.next_deadline());
        let wait = if run.work_pending(now) {
            skein_shell::Wait::No
        } else {
            deadline.map_or(skein_shell::Wait::Forever, skein_shell::Wait::Until)
        };
        kernel.submit(run.submissions(), wait);
    }
}

#[must_use]
pub fn local_run(mut settings: Settings, root: Fd, user: u32, seed: u64, signals: Fd) -> Run {
    let link = settings.link_file.take();
    let front = settings.front.is_some();
    let run = Run::new(settings, root, user, seed, Fd::new(0), Fd::new(1), signals).expect("TUI descriptors");
    let Some(path) = link else { return run };
    if front {
        // The box may not have made its data directory yet. Open beneath / on later local duties.
        let Ok(root) = skein_shell::open_root(Path::new("/")) else { return run };
        return run.with_local_path(root, path.strip_prefix(b"/").expect("absolute link path"));
    }
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
