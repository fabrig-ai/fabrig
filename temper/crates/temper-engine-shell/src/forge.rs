//! Forge commands and slot removal (temper/docs/design/protocol/setup/box.md, 1).
mod flags;
use std::{
    ffi::OsString,
    fs,
    io::{self, Write},
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, OpenOptionsExt},
    },
    path::Path,
};
use temper_forge_setup::{machine::BoxKind, place::Place, prepare::Prepare, run, setup::Mode, spec::Which};
pub use temper_forge_setup_shell::words::Refused;
pub(crate) use temper_forge_setup_shell::words::command_error;
use temper_forge_setup_shell::{prepare, run as host, words};
use temper_service_shell::jig_directories;
use temper_shell::{
    commands::{Argument, DIRECTORIES, Row, Who},
    directories::{self, EnvironmentEntry},
};
fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
type ListenerAt = fn(skein_io::kernel::Fd) -> Result<std::net::SocketAddr, skein_shell::ListenersRefusal>;

fn inherited_listener(
    which: Which,
    environment: &[EnvironmentEntry],
    listener_at: ListenerAt,
) -> Result<Option<std::net::SocketAddr>, String> {
    let Some(count) = temper_shell::environment::LISTEN_FDS.value(environment) else { return Ok(None) };
    if which == Which::Runner {
        return Err("LISTEN_FDS: runner takes no listeners (fd 3)".into());
    }
    if count != b"1" {
        return Err("LISTEN_FDS: forgejo requires exactly 1 listener at fd 3".into());
    }
    listener_at(skein_io::kernel::Fd::new(3)).map(Some).map_err(temper_service_shell::listeners_refusal)
}
/// A binary shell supplies the checked document shape it serves (protocol/services.md, 7).
pub type Reader = fn(&[u8], &jig_protocol_config::Directories) -> Result<super::setup::Deployment, String>;

type SetupFile = temper_forge_setup::setup_file::SetupFile;
type Configuration = temper_service_shell::configuration::Read;
type Named = temper_engine_service::startup::Named;
type ReadSetup<'a> = dyn Fn(&super::Directories, u16, &str, &[OsString]) -> Result<SetupFile, String> + 'a;
type ReadConfiguration<'a> =
    dyn Fn(&super::Directories, temper_config_shell::Shape) -> Result<Configuration, String> + 'a;
type SignIn<'a> = dyn Fn(&super::Directories, &[u8], &Named, u16, &[EnvironmentEntry]) -> Result<u8, String> + 'a;

pub(super) struct Inputs<'a> {
    pub(super) reader: Reader,
    pub(super) listener_at: ListenerAt,
    pub(super) setup: &'a ReadSetup<'a>,
    pub(super) configuration: &'a ReadConfiguration<'a>,
    pub(super) sign_in: &'a SignIn<'a>,
}

type PrepareDirectory<'a> = dyn Fn(&std::path::Path, bool) -> Result<u32, String> + 'a;
type PrepareFake<'a> = dyn Fn(&std::path::Path) -> Result<(), String> + 'a;

pub(super) struct Preparation<'a> {
    pub(super) directory: &'a PrepareDirectory<'a>,
    pub(super) fake: &'a PrepareFake<'a>,
}

type PrepareOwned<'a> = dyn Fn(&skein_io::Limits, &Place, Prepare) -> Result<(), prepare::Failed> + 'a;
type RunOwned<'a> = dyn Fn(&skein_io::Limits, &run::Launch) -> host::Failed + 'a;
type Login<'a> = dyn Fn(u32) -> Result<Box<[u8]>, String> + 'a;

struct OwnedCommands<'a> {
    login: &'a Login<'a>,
    prepare: &'a PrepareOwned<'a>,
    run: &'a RunOwned<'a>,
}

pub fn start(
    directories: &super::Directories,
    arguments: &[OsString],
    ssl_cert_file: Option<std::path::PathBuf>,
    environment: &[EnvironmentEntry],
) -> Result<u8, Refused> {
    start_with_binary("temper", directories, arguments, ssl_cert_file, environment)
}
pub fn start_with_reader(
    shape: temper_config_shell::Shape,
    directories: &super::Directories,
    arguments: &[OsString],
    ssl_cert_file: Option<std::path::PathBuf>,
    environment: &[EnvironmentEntry],
    reader: Reader,
) -> Result<u8, Refused> {
    start_with_binary_reader("temper", shape, directories, arguments, ssl_cert_file, environment, reader)
}

fn start_with_binary(
    binary: &str,
    directories: &super::Directories,
    arguments: &[OsString],
    ssl_cert_file: Option<std::path::PathBuf>,
    environment: &[EnvironmentEntry],
) -> Result<u8, Refused> {
    start_with_binary_reader(
        binary,
        temper_config_shell::Shape::EngineAlone,
        directories,
        arguments,
        ssl_cert_file,
        environment,
        super::setup::check,
    )
}

fn start_with_binary_reader(
    binary: &str,
    shape: temper_config_shell::Shape,
    directories: &super::Directories,
    arguments: &[OsString],
    ssl_cert_file: Option<std::path::PathBuf>,
    environment: &[EnvironmentEntry],
    reader: Reader,
) -> Result<u8, Refused> {
    start_with_inputs(
        binary,
        shape,
        directories,
        arguments,
        ssl_cert_file,
        environment,
        &Inputs {
            reader,
            // Unlike inherited_listeners, listener_at preserves fd 3's exec flags.
            listener_at: skein_shell::listener_at,
            setup: &setup_file,
            configuration: &temper_service_shell::configuration::read,
            sign_in: &super::forge_sign_in::run,
        },
    )
}

pub(super) fn start_with_inputs(
    binary: &str,
    shape: temper_config_shell::Shape,
    directories: &super::Directories,
    arguments: &[OsString],
    ssl_cert_file: Option<std::path::PathBuf>,
    environment: &[EnvironmentEntry],
    inputs: &Inputs<'_>,
) -> Result<u8, Refused> {
    start_with_preparation(
        binary,
        shape,
        directories,
        arguments,
        ssl_cert_file,
        environment,
        (inputs, &Preparation { directory: &prepare_directory, fake: &prepare_fake }),
    )
}

pub(super) fn start_with_preparation(
    binary: &str,
    shape: temper_config_shell::Shape,
    directories: &super::Directories,
    arguments: &[OsString],
    ssl_cert_file: Option<std::path::PathBuf>,
    environment: &[EnvironmentEntry],
    (inputs, preparation): (&Inputs<'_>, &Preparation<'_>),
) -> Result<u8, Refused> {
    start_with_effects(
        binary,
        shape,
        directories,
        arguments,
        ssl_cert_file,
        environment,
        (inputs, preparation, &OwnedCommands { login: &super::login, prepare: &prepare::run, run: &host::run }),
    )
}

#[expect(
    clippy::too_many_lines,
    reason = "one forge command dispatches its binary-shaped document, checked setup and process lifecycle"
)]
fn start_with_effects(
    binary: &str,
    shape: temper_config_shell::Shape,
    directories: &super::Directories,
    arguments: &[OsString],
    ssl_cert_file: Option<std::path::PathBuf>,
    environment: &[EnvironmentEntry],
    (inputs, preparation, commands): (&Inputs<'_>, &Preparation<'_>, &OwnedCommands<'_>),
) -> Result<u8, Refused> {
    let usage = || {
        Refused::new(
            "prepare",
            None,
            format!(
                "usage: {binary} forge fetch [DIRECTORIES] FORGE | {binary} forge prepare [DIRECTORIES] FORGE | {binary} forge run [DIRECTORIES] FORGE forgejo|runner | {binary} forge sign-in [DIRECTORIES] FORGE | {binary} forge remove [DIRECTORIES] FORGE"
            ),
        )
    };
    let mut setup_flags = &[][..];
    let (verb, number, which) = match arguments {
        [verb, number] if verb == "remove" => ("remove", number, None),
        [verb, number] if verb == "fetch" => ("fetch", number, None),
        [verb, number, flags @ ..] if verb == "prepare" => {
            setup_flags = flags;
            ("prepare", number, None)
        }
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
    let refused = |why| Refused::new(verb, Some(number), why);

    // Inspect before reading the document or opening the lock: neither may occupy fd 3.
    let listener = which
        .map(|which| inherited_listener(which, environment, inputs.listener_at))
        .transpose()
        .map_err(refused)?
        .flatten();
    let dirs = jig_directories(directories);
    let setup = (inputs.setup)(directories, number, verb, setup_flags).map_err(refused)?;
    let entry = setup.entry(number, &dirs).ok_or_else(|| refused("invalid setup directory".into()))?;
    if verb == "remove" {
        return remove_directory(
            super::path(&directories.data),
            super::path(&entry.directory),
            super::path(&directories.cache),
        )
        .map(|()| 0)
        .map_err(refused);
    }
    let read = (inputs.configuration)(directories, shape).map_err(refused)?;
    let bytes = &read.document;
    let refused = |why: String| refused(read.problems(&why));
    let super::setup::Deployment { named, checked, .. } =
        (inputs.reader)(bytes, &jig_directories(directories)).map_err(refused)?;
    if verb == "sign-in" {
        return (inputs.sign_in)(directories, bytes, &named, number, environment).map_err(refused);
    }
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
    let user = (preparation.directory)(directory, which.is_none()).map_err(refused)?;
    let Some(owned) = owned else {
        (preparation.fake)(&directory.join("admin_token")).map_err(refused)?;
        return Ok(0);
    };
    let place =
        Place { directory: directory.as_os_str().as_bytes().into(), user: (commands.login)(user).map_err(refused)? };
    let limits = jig_protocol_config::stock().io;
    if let Some(which) = which {
        return run_process(binary, owned, &place, which, listener, (number, directories), (&limits, commands.run))
            .map_err(refused);
    }
    let client = match &setup {
        temper_forge_setup::setup_file::SetupFile::Owned(o) => o.sign_in.as_deref(),
        temper_forge_setup::setup_file::SetupFile::Fake(_) => None,
    };
    if verb == "source" {
        return super::forge_sign_in::source(bytes, &named, owned, &place, checked.limits.deadline, client)
            .map_err(refused);
    }
    let owned_sign_in = client.is_some();
    (commands.prepare)(
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
                .then(|| super::forge_sign_in::discovery(bytes, &named).ok())
                .flatten()
                .map(Vec::into_boxed_slice),
            checked.limits.deadline,
        ),
    )
    .map_err(|why| refused(words::prepare_failure(binary, why, &place)))?;
    Ok(0)
}
fn setup_file(
    directories: &super::Directories,
    number: u16,
    verb: &str,
    arguments: &[OsString],
) -> Result<temper_forge_setup::setup_file::SetupFile, String> {
    setup_file_with(
        directories,
        number,
        verb,
        arguments,
        &SetupFacts {
            group_read: &skein_shell::read_file,
            group_stat: &|path| path.symlink_metadata().map(|_| ()),
            setup_present: &Path::try_exists,
            setup_read: &temper_forge_setup_shell::setup_file::read,
            setup_write: &temper_forge_setup_shell::setup_file::write,
        },
    )
}

type GroupRead<'a> = dyn Fn(&Path, u32) -> Result<Box<[u8]>, skein_shell::Unread> + 'a;
type GroupStat<'a> = dyn Fn(&Path) -> io::Result<()> + 'a;
type SetupPresent<'a> = dyn Fn(&Path) -> io::Result<bool> + 'a;
type SetupRead<'a> = dyn Fn(&jig_protocol_config::Directories, u16, BoxKind) -> Result<SetupFile, String> + 'a;
type SetupWrite<'a> = dyn Fn(&jig_protocol_config::Directories, u16, &SetupFile) -> Result<(), String> + 'a;

struct SetupFacts<'a> {
    group_read: &'a GroupRead<'a>,
    group_stat: &'a GroupStat<'a>,
    setup_present: &'a SetupPresent<'a>,
    setup_read: &'a SetupRead<'a>,
    setup_write: &'a SetupWrite<'a>,
}

fn setup_file_with(
    directories: &super::Directories,
    number: u16,
    verb: &str,
    arguments: &[OsString],
    facts: &SetupFacts<'_>,
) -> Result<SetupFile, String> {
    let flags = flags::Flags::parse(arguments)?;
    let dirs = jig_directories(directories);
    let kind = read_group_kind_with(&group_path(directories, number), facts.group_read, facts.group_stat)?
        .unwrap_or(BoxKind::Owned);
    let path = temper_forge_setup_shell::setup_file::path(&dirs, number, kind);
    if verb == "prepare" && !(facts.setup_present)(&path).map_err(|why| format!("{}: {why}", path.display()))? {
        let setup = flags.make(&dirs)?;
        (facts.setup_write)(&dirs, number, &setup)?;
        Ok(setup)
    } else {
        let setup = (facts.setup_read)(&dirs, number, kind)?;
        flags.compare(&setup, &dirs)?;
        Ok(setup)
    }
}

pub(crate) mod testing {
    pub fn setup_file(
        directories: &super::super::Directories,
        number: u16,
        verb: &str,
        flags: &[std::ffi::OsString],
    ) -> Result<temper_forge_setup::setup_file::SetupFile, String> {
        super::setup_file(directories, number, verb, flags)
    }
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
        &jig_protocol_config::stock().io,
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
    (limits, run_owned): (&skein_io::Limits, &RunOwned<'_>),
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
            .map_err(|why| words::run_failure(binary, why, &launch, number, directories));
    }
    Err(words::run_failure(binary, run_owned(limits, &launch), &launch, number, directories))
}
fn prepare_directory(directory: &Path, create: bool) -> Result<u32, String> {
    prepare_directory_with(
        directory,
        create,
        Path::exists,
        |path, mode| fs::DirBuilder::new().recursive(true).mode(mode).create(path),
        skein_shell::effective_user,
        skein_shell::symlink_stat,
    )
}

fn prepare_directory_with(
    directory: &Path,
    create: bool,
    mut present: impl FnMut(&Path) -> bool,
    mkdir: impl FnOnce(&Path, u32) -> io::Result<()>,
    effective_user: impl FnOnce() -> u32,
    stat: impl FnOnce(&Path) -> io::Result<skein_io::kernel::Stat>,
) -> Result<u32, String> {
    let named = |why: &str| format!("{}: {why}", directory.display());
    if create && !present(directory) {
        mkdir(directory, 0o700).map_err(|why| named(&why.to_string()))?;
    }
    let user = effective_user();
    if present(directory) {
        directories::private_root_with(user, stat(directory)).map_err(|why| named(&why))?;
    }
    Ok(user)
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
    for (byte, digits) in seed.expose().iter().take(20).zip(token.as_chunks_mut::<2>().0) {
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
    run_with_shape_reader(
        binary,
        temper_config_shell::Shape::EngineAlone,
        directories,
        arguments,
        ssl_cert_file,
        environment,
        super::setup::check,
    )
}

#[must_use]
pub fn run_with_shape_reader(
    binary: &str,
    shape: temper_config_shell::Shape,
    directories: &super::Directories,
    arguments: &[OsString],
    ssl_cert_file: Option<std::path::PathBuf>,
    environment: &[EnvironmentEntry],
    reader: Reader,
) -> u8 {
    let result = start_with_binary_reader(binary, shape, directories, arguments, ssl_cert_file, environment, reader);
    report_exit(binary, arguments, result, |line| println!("{line}"), |line| eprint!("{line}"))
}

pub(super) fn report_exit(
    binary: &str,
    arguments: &[OsString],
    result: Result<u8, Refused>,
    stdout: impl FnOnce(String),
    stderr: impl FnOnce(String),
) -> u8 {
    words::report(binary, result, arguments.first().is_some_and(|verb| verb == "source"), stdout, stderr)
}
fn webhook_listener(named: &temper_engine_service::startup::Named) -> Result<Option<core::net::SocketAddr>, String> {
    named.webhooks.as_ref().map(|hooks| super::address(&hooks.listener)).transpose()
}

/// Composes a forge's setup entry: its group's kind, or the machine's `owned` part with none
/// (temper/docs/design/protocol/setup/box.md, 1 and 4).
pub fn entry(directories: &super::Directories, number: u16) -> Result<temper_forge_setup::setup::Entry, String> {
    let kind = read_group_kind_with(&group_path(directories, number), &skein_shell::read_file, &|path| {
        path.symlink_metadata().map(|_| ())
    })?
    .unwrap_or(BoxKind::Owned);
    let dirs = jig_directories(directories);
    temper_forge_setup_shell::setup_file::read(&dirs, number, kind)?
        .entry(number, &dirs)
        .ok_or("invalid setup directory".into())
}

fn group_path(directories: &super::Directories, number: u16) -> std::path::PathBuf {
    super::path(&directories.data).join(super::path(&temper_forge_setup::machine::group_path(number)))
}

fn read_group_kind_with(
    path: &Path,
    read: &GroupRead<'_>,
    no_follow: &GroupStat<'_>,
) -> Result<Option<BoxKind>, String> {
    let bytes = match read(path, 16) {
        Ok(bytes) => bytes,
        // A dangling link is an existing group path, and keeps its read refusal.
        Err(skein_shell::Unread::Absent) if no_follow(path).is_err_and(|why| why.kind() == io::ErrorKind::NotFound) => {
            return Ok(None);
        }
        Err(why) => return Err(format!("{}: {}", path.display(), super::unread(why))),
    };
    temper_forge_setup::machine::parse_group_file(&bytes)
        .map(Some)
        .ok_or_else(|| format!("{}: invalid group kind", path.display()))
}
/// Removes a slot's directory beneath data, keeping the cache (7.1).
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
    notes: "Fetches only the binaries the forge's setup gives as \"fetched\", at temper's pinned versions, for amd64 or arm64; another machine is refused, naming its architecture. It says how far each binary has gone on its stdout, a line at each percent. Reads the forge's immutable setup file in its data directory.",
};

pub const PREPARE: Row = Row {
    words: &["forge", "prepare"],
    who: Who::Deployment,
    forms: &["[DIRECTORIES] FORGE [--listen ADDR --person LOGIN EMAIL] [SETUP FLAGS]"],
    line: "Prepare an owned forge before its Forgejo first runs.",
    arguments: &[DIRECTORIES, FORGE],
    settings: &[&directories::SETTINGS],
    notes: "The first prepare writes the immutable setup file. Required: --listen ADDR --person LOGIN EMAIL. Optional: --remote URL, --public ORIGIN, --forgejo PATH (default fetched), --runner-label LABEL (repeatable; enables a runner), --runner-binary PATH (default fetched), --runner-capacity N (default 1), --runner-path PATH (default /usr/local/bin:/usr/bin:/bin), --sign-in CLIENT. Later flags must equal the file; with none it is read.",
};

pub const RUN: Row = Row {
    words: &["forge", "run"],
    who: Who::Deployment,
    forms: &["[DIRECTORIES] FORGE forgejo|runner"],
    line: "Run an owned forge's Forgejo or its runner.",
    arguments: &[DIRECTORIES, FORGE],
    settings: &[&directories::SETTINGS],
    notes: "Checks the binary and the preparation, and takes the forge's lock, first. Run alone, it replaces itself by the program, whose exit status is then its own; given the forge's listener, as the box gives it, it runs the program as its child. Reads the forge's immutable setup file in its data directory.",
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
    notes: "For a Forgejo of [[sign_in.forges]], on temper's host, before temper-engine serve. Writes the provider's key and the forge's client secret where they are missing, private, and leaves any already there; prints Forgejo's add-oauth command, never running it.",
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
    let valid = if *verb == "run" {
        rest.len() == 2 && (rest[1] == "forgejo" || rest[1] == "runner")
    } else if *verb == "prepare" {
        !rest.is_empty()
    } else {
        rest.len() == 1
    };
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
    command_with_reader(binary, temper_config_shell::Shape::EngineAlone, command, super::setup::check)
}

pub fn command_with_reader(
    binary: &str,
    shape: temper_config_shell::Shape,
    command: Command,
    reader: Reader,
) -> Result<u8, String> {
    match command {
        Command::Forge { directories, arguments, ssl_cert_file, environment } => {
            Ok(run_with_shape_reader(binary, shape, &directories, &arguments, ssl_cert_file, &environment, reader))
        }
        Command::ForgeChild { parent, signal, program, arguments } => {
            let why = temper_forge_setup_shell::process::child(parent, signal, &program, &arguments);
            Err(super::prefixed(&format!("{binary} internal forge-child"), &why.to_string()))
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod owned_tests;
