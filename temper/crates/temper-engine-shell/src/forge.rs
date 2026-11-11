//! Forge commands and slot removal (temper/docs/design/protocol/setup/box.md, 1).
use std::{
    ffi::OsString,
    fs,
    io::Write,
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, OpenOptionsExt},
    },
};
use temper_forge_setup::{
    place::{File, Place},
    prepare::{Exit, Failure, Prepare},
    run::{self, Refusal},
    setup::Mode,
    spec::Which,
    upgrade::Course,
};
use temper_forge_setup_shell::{command, prepare, run as host};
use temper_service_shell::jig_directories;
use temper_shell::{
    commands::{Argument, DIRECTORIES, Row, Who},
    directories::{self, EnvironmentEntry},
};
/// A command's diagnostic, without any credential contents.
#[derive(Debug)]
pub struct Refused {
    verb: &'static str,
    number: Option<u16>,
    why: String,
}
impl Refused {
    #[must_use]
    pub fn line(&self) -> String {
        self.line_with_binary("temper")
    }
    fn line_with_binary(&self, binary: &str) -> String {
        let forge = self.number.map_or_else(String::new, |n| format!("forge {n}: "));
        super::prefixed(&format!("{binary} forge {}", self.verb), &format!("{forge}{}", self.why))
    }
}
fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
fn status(exit: Exit) -> String {
    match exit {
        Exit::Code(n) => n.to_string(),
        Exit::Signal(n) => format!("signal {n}"),
    }
}
pub(crate) fn command_error(why: command::Error) -> String {
    match why {
        command::Error::Interrupted => "interrupted".into(),
        command::Error::Kernel(why) => format!("command kernel: {why}"),
        command::Error::Spawn(why) => format!("command could not start: {why:?}"),
    }
}
fn failure(binary: &str, why: prepare::Failed, place: &Place) -> String {
    match why {
        prepare::Failed::Held(file) => format!(
            "{} is running or being prepared already",
            if file == File::RunnerLock { "runner" } else { "forgejo" }
        ),
        prepare::Failed::Io(file, why) => format!("{}: {file}: {why}", text(&place.directory)),
        prepare::Failed::Command(why) => command_error(why),
        prepare::Failed::Refused(why) => match why {
            Failure::Binary { program, printed, .. } => {
                format!("{} printed {printed:?}, not the pinned release", text(&program))
            }
            Failure::Command { command, exit, tail } => {
                format!("{command:?} ended with status {}: {}", status(exit), text(&tail))
            }
            Failure::Token { exit, tail } => {
                format!("no token for temper-admin: ended with status {}: {}", status(exit), text(&tail))
            }
            Failure::PersonToken { file, exit, tail } => {
                format!(
                    "{}: no person token: ended with status {}: {}",
                    text(&place.path(file)),
                    status(exit),
                    text(&tail)
                )
            }
            Failure::Unread { command } => format!("{command:?} printed no required value"),
            Failure::State(why) => format!("{}: state cannot be decoded: {why:?}", text(&place.directory)),
            Failure::Unrecorded => format!("{}: Forgejo files have no preparation state", text(&place.directory)),
            Failure::Newer { which, recorded } => {
                let (name, pin) = pinned(which);
                format!(
                    "{name} is prepared at {}, newer than this {binary}'s pin {pin}: its migrations only go forward",
                    text(&recorded)
                )
            }
            Failure::Unreadable { which } => {
                format!("{} is prepared at a version this {binary} cannot read", pinned(which).0)
            }
        },
    }
}
fn pinned(which: Which) -> (&'static str, &'static str) {
    use skein_forge::forgejo::pin::{RUNNER_VERSION, VERSION};
    match which {
        Which::Forgejo => ("forgejo", VERSION),
        Which::Runner => ("runner", RUNNER_VERSION),
    }
}
fn prepare_hint(binary: &str, why: &str, number: u16, directories: &super::Directories) -> String {
    let flags = directories.flags().iter().map(|arg| text(arg)).collect::<Vec<_>>().join(" ");
    format!("{why}: run {binary} forge prepare {flags} {number}")
}
fn run_refusal(
    binary: &str,
    why: Refusal,
    launch: &run::Launch,
    number: u16,
    directories: &super::Directories,
) -> String {
    let prepare = |why: String| prepare_hint(binary, &why, number, directories);
    match why {
        Refusal::NoRunner => "the setup has no runner".into(),
        Refusal::NotPrepared => prepare("not prepared".into()),
        Refusal::Unfinished => prepare("its preparation did not end".into()),
        Refusal::Runner { recorded } => prepare(if recorded {
            "prepared with a runner, and the setup has none".into()
        } else {
            "prepared without a runner, and the setup has one".into()
        }),
        Refusal::Versions { course: Course::Unreadable, .. } => {
            format!("prepared at a version this {binary} cannot read: no command of this {binary} runs it")
        }
        Refusal::Versions { forgejo, runner, course } => {
            use skein_forge::forgejo::pin::{RUNNER_VERSION, VERSION};
            let recorded =
                runner.map_or_else(|| text(&forgejo), |runner| format!("{} / {}", text(&forgejo), text(&runner)));
            let pinned = if launch.runner { format!("{VERSION} / {RUNNER_VERSION}") } else { VERSION.into() };
            let reason = format!("prepared at {recorded}, this {binary} pins {pinned}");
            if course == Course::Newer {
                format!("{reason}: no command of this {binary} runs it")
            } else {
                prepare(reason)
            }
        }
        Refusal::Binary { printed } => format!("{} printed {printed:?}, not the pinned release", text(&launch.program)),
    }
}
fn run_failure(
    binary: &str,
    why: host::Failed,
    launch: &run::Launch,
    number: u16,
    directories: &super::Directories,
) -> String {
    match why {
        host::Failed::Refused(why) => run_refusal(binary, why, launch, number, directories),
        host::Failed::Locked => format!(
            "{} is running or being prepared already",
            if launch.which == Which::Forgejo { "forgejo" } else { "runner" }
        ),
        host::Failed::Lock(why) => format!("{}: lock: {why}", text(&launch.lock)),
        host::Failed::State(why) => format!("{}: state {why}", text(&launch.state)),
        host::Failed::Version(why) => command_error(why),
        host::Failed::Exec(why) => format!("{}: {why}", text(&launch.program)),
    }
}
fn inherited_listener(which: Which, environment: &[EnvironmentEntry]) -> Result<Option<std::net::SocketAddr>, String> {
    let Some(count) = temper_shell::environment::LISTEN_FDS.value(environment) else { return Ok(None) };
    if which == Which::Runner {
        return Err("LISTEN_FDS: runner takes no listeners (fd 3)".into());
    }
    if count != b"1" {
        return Err("LISTEN_FDS: forgejo requires exactly 1 listener at fd 3".into());
    }
    // Unlike inherited_listeners, listener_at preserves fd 3's exec flags.
    skein_shell::listener_at(skein_io::kernel::Fd::new(3)).map(Some).map_err(temper_service_shell::listeners_refusal)
}
pub fn start(
    directories: &super::Directories,
    arguments: &[OsString],
    ssl_cert_file: Option<std::path::PathBuf>,
    environment: &[EnvironmentEntry],
) -> Result<u8, Refused> {
    start_with_binary("temper", directories, arguments, ssl_cert_file, environment)
}
fn start_with_binary(
    binary: &str,
    directories: &super::Directories,
    arguments: &[OsString],
    ssl_cert_file: Option<std::path::PathBuf>,
    environment: &[EnvironmentEntry],
) -> Result<u8, Refused> {
    let usage = || Refused {
        verb: "prepare",
        number: None,
        why: format!(
            "usage: {binary} forge fetch [DIRECTORIES] FORGE | {binary} forge prepare [DIRECTORIES] FORGE | {binary} forge run [DIRECTORIES] FORGE forgejo|runner | {binary} forge sign-in [DIRECTORIES] FORGE | {binary} forge remove [DIRECTORIES] FORGE"
        ),
    };
    let (verb, number, which) = match arguments {
        [verb, number] if verb == "remove" => ("remove", number, None),
        [verb, number] if verb == "fetch" => ("fetch", number, None),
        [verb, number] if verb == "prepare" => ("prepare", number, None),
        [verb, number] if verb == "sign-in" => ("sign-in", number, None),
        [verb, number] if verb == "source" => ("source", number, None),
        [verb, number, which] if verb == "run" => {
            let which = match which.to_str() {
                Some("forgejo") => Which::Forgejo,
                Some("runner") => Which::Runner,
                _ => return Err(usage()),
            };
            ("run", number, Some(which))
        }
        _ => return Err(usage()),
    };
    let number = number.to_str().and_then(|n| n.parse::<u16>().ok()).ok_or_else(usage)?;
    let refused = |why| Refused { verb, number: Some(number), why };
    if verb == "remove" {
        return remove(directories, number).map(|()| 0).map_err(refused);
    }
    // Inspect before reading the document or opening the lock: neither may occupy fd 3.
    let listener = which.map(|which| inherited_listener(which, environment)).transpose().map_err(refused)?.flatten();
    let bytes = directories.read().map_err(refused)?;
    let super::setup::Deployment { named, checked, .. } =
        super::setup::check(&bytes, &jig_directories(directories)).map_err(refused)?;
    if verb == "sign-in" {
        return super::forge_sign_in::run(directories, &bytes, &named, number, environment).map_err(refused);
    }
    let entry = entry(directories, &named.machine, number).map_err(refused)?;
    let owned = match &entry.mode {
        Mode::Owned(owned) => Some(owned),
        Mode::Managed { .. } => return Err(refused("it is managed, not owned".into())),
        Mode::Fake(_) if which.is_some() || verb == "fetch" || verb == "source" => {
            return Err(refused("it is fake, not owned".into()));
        }
        Mode::Fake(_) => None,
    };
    if verb == "fetch" {
        return fetch(owned.expect("fetch requires owned"), checked.fetch_bytes, ssl_cert_file).map_err(refused);
    }
    let directory = super::path(&entry.directory);
    if which.is_none() && !directory.exists() {
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(directory)
            .map_err(|why| refused(format!("{}: {why}", directory.display())))?;
    }
    let user = skein_shell::effective_user();
    if directory.exists() {
        super::private_root(directory, user).map_err(|why| refused(format!("{}: {why}", directory.display())))?;
    }
    let Some(owned) = owned else {
        prepare_fake(&directory.join("admin_token")).map_err(refused)?;
        return Ok(0);
    };
    let place =
        Place { directory: directory.as_os_str().as_bytes().into(), user: super::login(user).map_err(refused)? };
    let limits = jig_protocol_config::profile(jig_protocol_config::Profile::Small).io;
    if let Some(which) = which {
        return run_process(binary, owned, &place, which, listener, (number, directories), &limits).map_err(refused);
    }
    if verb == "source" {
        return super::forge_sign_in::source(&bytes, &named, owned, &place, checked.limits.deadline).map_err(refused);
    }
    let owned_sign_in = named.machine.owned.as_ref().is_some_and(|owned| owned.sign_in.is_some());
    prepare::run(
        &limits,
        &place,
        Prepare::new(
            owned.clone(),
            place.clone(),
            webhook_listener(&named).map_err(refused)?.map(|socket| socket.ip()),
        )
        .with_sign_in(if owned_sign_in {
            Some(temper_forge_setup::sign_in::FirstSignIn::LinksByName)
        } else {
            named
                .sign_in
                .as_ref()
                .is_some_and(|sign_in| sign_in.forges.iter().any(|row| row.forge == number))
                .then(|| super::forge_sign_in::first_sign_in(environment))
        })
        .with_source_address(
            owned_sign_in
                .then(|| super::forge_sign_in::discovery(&bytes, &named).ok())
                .flatten()
                .map(Vec::into_boxed_slice),
            checked.limits.deadline,
        ),
    )
    .map_err(|why| refused(failure(binary, why, &place)))?;
    Ok(0)
}
fn fetch(
    owned: &temper_forge_setup::setup::Owned,
    fetch_bytes: u64,
    ssl_cert_file: Option<std::path::PathBuf>,
) -> Result<u8, String> {
    if !owned.fetch.forgejo && !owned.fetch.runner {
        return Err("neither of its binaries is \"fetched\"".into());
    }
    let pins = temper_forge_setup::fetch::pins(
        owned,
        if cfg!(target_os = "macos") {
            skein_forge::forgejo::pin::Platform::Macos
        } else {
            skein_forge::forgejo::pin::Platform::Linux
        },
        std::env::consts::ARCH,
    )
    .map_err(|why| text(&why.words()))?;
    let machine = temper_forge_setup::fetch::Fetch::new(pins, fetch_bytes);
    temper_forge_setup_shell::fetch::run(
        &jig_protocol_config::profile(jig_protocol_config::Profile::Small).io,
        &skein_shell::Trust::Machine { ssl_cert_file },
        machine,
        &mut std::io::stdout(),
    )
    .map(|()| 0)
    .map_err(|why| why.words())
}
/// Admits fd 3 and runs the process, returning its exit or why it did not start.
fn run_process(
    binary: &str,
    owned: &temper_forge_setup::setup::Owned,
    place: &Place,
    which: Which,
    listener: Option<std::net::SocketAddr>,
    (number, directories): (u16, &super::Directories),
    limits: &skein_io::Limits,
) -> Result<u8, String> {
    if let Err(why) = run::admit_listener(owned, listener) {
        return Err(text(&why.words()));
    }
    let Ok(mut launch) = run::launch(owned, place, which) else { return Err("the setup has no runner".into()) };
    if listener.is_some() {
        launch.exec = temper_forge_setup::spec::listening(launch.exec);
    }
    if let Some(address) = listener {
        return temper_forge_setup_shell::parent::run(limits, &launch, address)
            .map_err(|why| run_failure(binary, why, &launch, number, directories));
    }
    Err(run_failure(binary, host::run(limits, &launch), &launch, number, directories))
}
/// The fake shares only the private directory and token with an owned forge (protocol/setup/playground.md).
fn prepare_fake(path: &std::path::Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(_) => return Ok(()),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
        Err(why) => return Err(format!("{}: {why}", path.display())),
    }
    let seed = skein_shell::crypto_seed().map_err(|errno| format!("random token: errno {errno}"))?;
    let mut token = [0; 40];
    for (byte, digits) in seed.expose().iter().take(20).zip(token.chunks_exact_mut(2)) {
        digits[0] = b"0123456789abcdef"[usize::from(byte / 16)];
        digits[1] = b"0123456789abcdef"[usize::from(byte % 16)];
    }
    match fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path) {
        Ok(mut file) => file.write_all(&token).map_err(|why| format!("{}: {why}", path.display())),
        Err(why) if why.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(why) => Err(format!("{}: {why}", path.display())),
    }
}
#[must_use]
pub fn run(
    binary: &str,
    directories: &super::Directories,
    arguments: &[OsString],
    ssl_cert_file: Option<std::path::PathBuf>,
    environment: &[EnvironmentEntry],
) -> u8 {
    match start_with_binary(binary, directories, arguments, ssl_cert_file, environment) {
        Ok(code) => code,
        Err(why) => {
            if arguments.first().is_some_and(|verb| verb == "source") {
                let reason: String = why
                    .why
                    .bytes()
                    .take(1024)
                    .map(|c| if c.is_ascii_graphic() || c == b' ' { char::from(c) } else { ' ' })
                    .collect();
                println!("source failed {reason}");
                0
            } else {
                eprint!("{}", why.line_with_binary(binary));
                1
            }
        }
    }
}
fn webhook_listener(named: &temper_service::startup::Named) -> Result<Option<core::net::SocketAddr>, String> {
    named.webhooks.as_ref().map(|hooks| super::address(&hooks.listener)).transpose()
}

/// Composes a forge's setup entry: its group's kind, or the machine's `owned` part with none
/// (temper/docs/design/protocol/setup/box.md, 1 and 4).
pub fn entry(
    directories: &super::Directories,
    machine: &temper_forge_setup::machine::Machine,
    number: u16,
) -> Result<temper_forge_setup::setup::Entry, String> {
    use temper_forge_setup::machine;
    let path = super::path(&directories.data).join(super::path(&machine::group_path(number)));
    let kind = if let Some(kind) = read_group_kind(&path)? {
        kind
    } else {
        if machine.owned.is_none() {
            return Err(format!("{}: absent, and application.machine names no owned Forgejo", path.display()));
        }
        machine::BoxKind::Owned
    };
    machine::entry(machine, number, kind, &jig_directories(directories)).ok_or("the machine has no entry for it".into())
}

fn group_kind(directories: &super::Directories, number: u16) -> Result<temper_forge_setup::machine::BoxKind, String> {
    let path = super::path(&directories.data).join(super::path(&temper_forge_setup::machine::group_path(number)));
    read_group_kind(&path)?.ok_or_else(|| format!("{}: absent", path.display()))
}

fn read_group_kind(path: &std::path::Path) -> Result<Option<temper_forge_setup::machine::BoxKind>, String> {
    let bytes = match skein_shell::read_file(path, 16) {
        Ok(bytes) => bytes,
        // A dangling link is an existing group path, and keeps its read refusal.
        Err(skein_shell::Unread::Absent)
            if path.symlink_metadata().is_err_and(|why| why.kind() == std::io::ErrorKind::NotFound) =>
        {
            return Ok(None);
        }
        Err(why) => return Err(format!("{}: {}", path.display(), super::unread(why))),
    };
    temper_forge_setup::machine::parse_group_file(&bytes)
        .map(Some)
        .ok_or_else(|| format!("{}: invalid group kind", path.display()))
}
/// Removes a slot's directory beneath data, keeping the cache (7.1).
fn remove(directories: &super::Directories, number: u16) -> Result<(), String> {
    let kind = group_kind(directories, number)?;
    let data = super::path(&directories.data);
    let directory = data.join(super::path(&temper_forge_setup::machine::directory(number, kind)));
    remove_directory(data, &directory, super::path(&directories.cache))
}
pub fn remove_directory(
    data: &std::path::Path,
    directory: &std::path::Path,
    cache: &std::path::Path,
) -> Result<(), String> {
    if directory == data
        || !directory.starts_with(data)
        || directory.components().any(|part| part == std::path::Component::ParentDir)
    {
        return Err(format!("{}: refusing to remove a path outside data", directory.display()));
    }
    let data = fs::canonicalize(data).map_err(|why| format!("{}: {why}", data.display()))?;
    let target = match fs::canonicalize(directory) {
        Ok(target) => target,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(why) => return Err(format!("{}: {why}", directory.display())),
    };
    let cache = fs::canonicalize(cache).unwrap_or_else(|_| cache.to_path_buf());
    if target == data || !target.starts_with(&data) || cache.starts_with(&target) || target.starts_with(&cache) {
        return Err(format!("{}: refusing to remove a path outside data or in the cache", directory.display()));
    }
    match fs::remove_dir_all(directory) {
        Ok(()) => Ok(()),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(why) => Err(format!("{}: {why}", directory.display())),
    }
}

pub const FORGE: Argument = Argument { shown: "FORGE", line: "A forge's number in the configuration." };

pub const FETCH: Row = Row {
    words: &["forge", "fetch"],
    who: Who::Deployment,
    forms: &["[DIRECTORIES] FORGE"],
    line: "Download an owned forge's Forgejo and runner binaries into the cache.",
    arguments: &[DIRECTORIES, FORGE],
    settings: &[&directories::SETTINGS],
    notes: "Fetches only the binaries the forge's setup gives as \"fetched\", at temper's pinned versions, for amd64 or arm64; another machine is refused, naming its architecture. It says how far each binary has gone on its stdout, a line at each percent. A forge with no group's file takes `application.machine.owned`, in `<data>/forge-N`, and writes nothing under `box/`.",
};

pub const PREPARE: Row = Row {
    words: &["forge", "prepare"],
    who: Who::Deployment,
    forms: &["[DIRECTORIES] FORGE"],
    line: "Prepare an owned forge before its Forgejo first runs.",
    arguments: &[DIRECTORIES, FORGE],
    settings: &[&directories::SETTINGS],
    notes: "A forge with no group's file takes `application.machine.owned`, in `<data>/forge-N`, and writes nothing under `box/`.",
};

pub const RUN: Row = Row {
    words: &["forge", "run"],
    who: Who::Deployment,
    forms: &["[DIRECTORIES] FORGE forgejo|runner"],
    line: "Run an owned forge's Forgejo or its runner.",
    arguments: &[DIRECTORIES, FORGE],
    settings: &[&directories::SETTINGS],
    notes: "Checks the binary and the preparation, and takes the forge's lock, first. Run alone, it replaces itself by the program, whose exit status is then its own; given the forge's listener, as the box gives it, it runs the program as its child. A forge with no group's file takes `application.machine.owned`, in `<data>/forge-N`, and writes nothing under `box/`.",
};

pub const REMOVE: Row = Row {
    words: &["forge", "remove"],
    who: Who::Deployment,
    forms: &["[DIRECTORIES] FORGE"],
    line: "Remove a stopped box slot's forge directory, keeping the cache.",
    arguments: &[DIRECTORIES, FORGE],
    settings: &[&directories::SETTINGS],
    notes: "Reads the slot's kind from box/groups/FORGE and refuses a directory outside data or in the cache.",
};

pub const SOURCE: Row = Row {
    words: &["forge", "source"],
    who: Who::Deployment,
    forms: &["[DIRECTORIES] FORGE"],
    line: "Add the box’s public temper sign-in source to its owned Forgejo.",
    arguments: &[DIRECTORIES, FORGE],
    settings: &[&directories::SETTINGS],
    notes: "Lists the sources and adds temper when absent, under the setup deadline. Prints source added or source failed WHY and exits 0 either way: a failure keeps the box running with plain links.",
};

pub const SIGN_IN: Row = Row {
    words: &["forge", "sign-in"],
    who: Who::Deployment,
    forms: &["[DIRECTORIES] FORGE"],
    line: "Make what people need to sign in to temper through a Forgejo, and print the Forgejo command that registers temper.",
    arguments: &[DIRECTORIES, FORGE],
    settings: &[&directories::SETTINGS],
    notes: "For a Forgejo of application.sign_in.forges, on temper's host, before temper serve. Writes the provider's key and the forge's client secret where they are missing, private, and leaves any already there; prints Forgejo's add-oauth command, never running it.",
};

pub const FORGE_CHILD: Row = Row {
    words: &["internal", "forge-child"],
    who: Who::Spawned,
    forms: &["--parent PID --signal kill|terminate -- PROGRAM ARGS..."],
    line: "Run an owned forge's program, sent SIGNAL when its parent PID ends.",
    arguments: &[
        Argument { shown: "PID", line: "The process whose end ends the program." },
        Argument { shown: "PROGRAM", line: "The program to run, with its ARGS." },
    ],
    settings: &[],
    notes: "",
};

/// Whether the forge command checks the deployment document (protocol/binaries.md, 2).
#[must_use]
pub fn reads_document(row: &Row) -> bool {
    row.words.first() == Some(&"forge") && row.words != REMOVE.words
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Forge {
        directories: super::Directories,
        arguments: Vec<OsString>,
        ssl_cert_file: Option<std::path::PathBuf>,
        environment: Box<[EnvironmentEntry]>,
    },
    ForgeChild {
        parent: u32,
        signal: skein_io::kernel::Signal,
        program: OsString,
        arguments: Vec<OsString>,
    },
}

pub fn configure(
    row: &Row,
    arguments: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    working: &[u8],
) -> Result<Command, String> {
    let invalid = || "invalid arguments".to_string();
    let os =
        |args: &[Box<[u8]>]| args.iter().map(|arg| std::ffi::OsStr::from_bytes(arg).to_owned()).collect::<Vec<_>>();
    if row.words == FORGE_CHILD.words {
        let [flag, parent, signal_flag, signal, separator, program, rest @ ..] = arguments else {
            return Err(invalid());
        };
        if flag.as_ref() != b"--parent"
            || signal_flag.as_ref() != b"--signal"
            || separator.as_ref() != b"--"
            || program.is_empty()
        {
            return Err(invalid());
        }
        let parent = std::str::from_utf8(parent).ok().and_then(|pid| pid.parse::<u32>().ok()).ok_or_else(invalid)?;
        let signal = match signal.as_ref() {
            b"kill" => skein_io::kernel::Signal::Kill,
            b"terminate" => skein_io::kernel::Signal::Terminate,
            _ => return Err(invalid()),
        };
        return Ok(Command::ForgeChild {
            parent,
            signal,
            program: std::ffi::OsStr::from_bytes(program).into(),
            arguments: os(rest),
        });
    }
    let ["forge", verb] = row.words else {
        return Err(invalid());
    };
    let (directories, rest) = directories::resolve(arguments, environment, working).map_err(|why| why.to_string())?;
    let rest = os(&rest);
    let valid =
        if *verb == "run" { rest.len() == 2 && (rest[1] == "forgejo" || rest[1] == "runner") } else { rest.len() == 1 };
    if !valid || rest[0].to_str().and_then(|n| n.parse::<u16>().ok()).is_none() {
        return Err(invalid());
    }
    let mut arguments = vec![OsString::from(verb)];
    arguments.extend(rest);
    let ssl_cert_file =
        temper_shell::environment::SSL_CERT_FILE.value(environment).map(|value| super::path(value).to_path_buf());
    Ok(Command::Forge { directories, arguments, ssl_cert_file, environment: environment.into() })
}

pub fn command(binary: &str, command: Command) -> Result<u8, String> {
    match command {
        Command::Forge { directories, arguments, ssl_cert_file, environment } => {
            Ok(run(binary, &directories, &arguments, ssl_cert_file, &environment))
        }
        Command::ForgeChild { parent, signal, program, arguments } => {
            let why = temper_forge_setup_shell::process::child(parent, signal, &program, &arguments);
            Err(super::prefixed(&format!("{binary} internal forge-child"), &why.to_string()))
        }
    }
}

#[cfg(test)]
mod tests;
