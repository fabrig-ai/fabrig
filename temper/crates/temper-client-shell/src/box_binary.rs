//! The client's box lookup and exec (temper/docs/design/client/box-binary.md, 2, 3, 4, 5 and 6).
use crate::{CLIENT, Capture, SITE};
use skein_shell::{Argument, Command, Reach, Set, Settings};
use std::{
    ffi::OsStr,
    io::{self, Write},
    os::unix::{ffi::OsStrExt, process::CommandExt},
    path::{Path, PathBuf},
};
use temper_fetch::Origin;
use temper_fetch_shell::box_binary::{Find, find};
use temper_shell::{directories, directories::EnvironmentEntry, environment, table};

pub const BOX: Command = Command {
    reach: Reach::Words(&["box"]),
    set: Set::Public,
    forms: &["[ARGUMENT...]"],
    line: "Run the TUI in front of this build's temper-box, fetching it if needed.",
    arguments: &[Argument { shown: "ARGUMENT...", line: "Arguments for temper-box, passed on unchanged." }],
    settings: &[&directories::SETTINGS],
    switches: &[],
    variables: &[],
    notes: "Finds temper-box beside this executable, then in the cache, then at the download site. A checkout build downloads none: build it with cargo build -p temper-box. In a terminal, runs the TUI in front of the box. Without a terminal, or with --echo steps, runs the box in this process's place.",
};

const CACHE: Settings = Settings {
    tool: directories::SETTINGS.tool,
    about: directories::SETTINGS.about,
    table: &[directories::SETTINGS.table[2]],
};
pub const FETCH: Command = Command {
    reach: Reach::Words(&["internal", "fetch-box"]),
    set: Set::Internal,
    forms: &["[--cache DIR]"],
    line: "Find this build's temper-box, fetching it if needed, for temper box.",
    arguments: &[],
    settings: &[&CACHE],
    switches: &[],
    variables: &[],
    notes: "Writes progress lines, then found PATH on stdout; failures on stderr, exit 1. Reads no terminal.",
};

/// The directories and echo that a terminal front end reads among the box's arguments.
pub struct BoxArguments {
    pub directories: directories::Directories,
    pub echo_steps: bool,
}

/// Everything else stays the box's to take or refuse; the caller retains its whole argv.
pub fn resolve(
    arguments: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    working: &[u8],
) -> Result<BoxArguments, (u8, String)> {
    let (directories, rest) = directories::resolve(arguments, environment, working)
        .map_err(|why| (2, table::refused(&CLIENT, &BOX, &why.to_string())))?;
    let mut echo_steps = false;
    let mut rest = rest.iter();
    while let Some(argument) = rest.next() {
        if argument.as_ref() == b"--" {
            break;
        }
        if argument.as_ref() == b"--echo" {
            echo_steps = rest.next().is_some_and(|value| value.as_ref() == b"steps");
        }
    }
    Ok(BoxArguments { directories, echo_steps })
}

fn failure(why: impl std::fmt::Display) -> (u8, String) {
    (1, format!("temper: {why}\n"))
}

fn site(environment: &[EnvironmentEntry]) -> Result<Origin, (u8, String)> {
    let Some(value) = SITE.value(environment) else {
        return Ok(Origin { host: b"fabrig.ai".as_slice().into(), port: 443 });
    };
    let refused = || {
        failure(
            "TEMPER_INTERNAL_SITE: expected https://127.0.0.1:PORT, https://[::1]:PORT or https://localhost:PORT, with nothing after the port",
        )
    };
    let value = std::str::from_utf8(value).map_err(|_| refused())?;
    let (host, port) = value.strip_prefix("https://").and_then(|value| value.rsplit_once(':')).ok_or_else(refused)?;
    if !["127.0.0.1", "[::1]", "localhost"].contains(&host)
        || port.is_empty()
        || !port.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(refused());
    }
    let port = port.parse::<u16>().ok().filter(|port| *port != 0).ok_or_else(refused)?;
    Ok(Origin { host: host.as_bytes().into(), port })
}

struct Inputs {
    beside: PathBuf,
    cache: PathBuf,
    site: Origin,
    trust: skein_shell::Trust,
}
impl Inputs {
    fn new(cache: PathBuf, captured: &Capture) -> Result<Self, (u8, String)> {
        let executable = captured.executable.as_ref().map_err(failure)?;
        let beside = executable.parent().ok_or_else(|| failure("executable has no parent"))?.to_path_buf();
        Ok(Self {
            beside,
            cache,
            site: site(&captured.environment)?,
            trust: skein_shell::Trust::Machine {
                ssl_cert_file: environment::SSL_CERT_FILE.value(&captured.environment).map(|value| path(value).into()),
            },
        })
    }
    fn find(&self) -> Find<'_> {
        Find {
            beside: &self.beside,
            cache: &self.cache,
            site: &self.site,
            trust: &self.trust,
            version: temper_version::VERSION,
            commit: temper_version::COMMIT,
        }
    }
}
fn path(bytes: &[u8]) -> &Path {
    Path::new(OsStr::from_bytes(bytes))
}

pub(crate) fn fetch(arguments: &[Box<[u8]>], captured: &Capture) -> Result<u8, (u8, String)> {
    let refused = |why: &str| (2, table::refused(&CLIENT, &FETCH, why));
    let resolved =
        skein_shell::resolve(&CACHE, arguments, &captured.environment).map_err(|why| refused(&why.to_string()))?;
    if !resolved.rest().is_empty() {
        return Err(refused("invalid arguments"));
    }
    let working = captured.working.as_ref().map_err(failure)?;
    let cache = working.join(path(&resolved.value(0).bytes));
    if cache.as_os_str().as_bytes().len() > 4096 {
        return Err(refused("--cache DIR: absolute path is past 4096 bytes"));
    }
    let inputs = Inputs::new(cache, captured)?;
    let mut stdout = io::stdout().lock();
    let result = find(&IO, &inputs.find(), &mut stdout).map_err(|why| why.words());
    found(result, &mut stdout)
}

fn found(result: Result<PathBuf, String>, stdout: &mut dyn Write) -> Result<u8, (u8, String)> {
    let file = result.map_err(fetch_failure)?;
    stdout
        .write_all(b"found ")
        .and_then(|()| stdout.write_all(file.as_os_str().as_bytes()))
        .and_then(|()| stdout.write_all(b"\n"))
        .map_err(fetch_failure)?;
    Ok(0)
}

fn fetch_failure(why: impl std::fmt::Display) -> (u8, String) {
    let (code, mut line) = failure(why);
    // The subprocess protocol bounds stderr, including the prefix and final line feed.
    if line.len() > 8192 {
        let mut end = 8191;
        while !line.is_char_boundary(end) {
            end -= 1;
        }
        line.truncate(end);
        line.push('\n');
    }
    (code, line)
}

/// A silent lookup stays silent; only its first progress write announces a fetch.
struct Fetching<'a> {
    out: &'a mut dyn Write,
    started: bool,
}
impl Write for Fetching<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if !bytes.is_empty() && !self.started {
            self.started = true;
            self.out.write_all(b"fetching temper-box\n")?;
        }
        self.out.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}

fn terminal_front(resolved: &BoxArguments, input: &impl io::IsTerminal, output: &impl io::IsTerminal) -> bool {
    !resolved.echo_steps && input.is_terminal() && output.is_terminal()
}

fn exec_command(file: &Path, arguments: &[Box<[u8]>], environment: &[EnvironmentEntry]) -> std::process::Command {
    let mut command = std::process::Command::new(file);
    command
        .args(arguments.iter().map(|word| OsStr::from_bytes(word)))
        .env_clear()
        .envs(environment.iter().map(|(key, value)| (OsStr::from_bytes(key), OsStr::from_bytes(value))));
    command
}

pub(crate) fn run(arguments: &[Box<[u8]>], captured: &Capture) -> Result<u8, (u8, String)> {
    let working = captured.working.as_ref().map_err(failure)?;
    let resolved = resolve(arguments, &captured.environment, working.as_os_str().as_bytes())?;
    if terminal_front(&resolved, &io::stdin(), &io::stdout()) {
        let executable = captured.executable.as_ref().map_err(failure)?;
        return Ok(temper_client_shell_tui::start::run_box(
            executable.as_os_str().as_bytes(), &resolved.directories.cache, arguments, &resolved.directories.data,
            &captured.environment, working.as_os_str().as_bytes(), &skein_shell::Clock::new(),
        ));
    }
    let inputs = Inputs::new(path(&resolved.directories.cache).into(), captured)?;
    let file = {
        let mut stderr = io::stderr().lock();
        let mut progress = Fetching { out: &mut stderr, started: false };
        find(&IO, &inputs.find(), &mut progress).map_err(|why| failure(why.words()))?
    };
    let why = exec_command(&file, arguments, &captured.environment).exec();
    Err(failure(format!("{}: {why}", file.display())))
}

const IO: skein_io::Limits = skein_io::Limits {
    sockets: 16,
    names: 8,
    refusals: 8,
    intake: 1_048_576,
    receive: 16_384,
    output: 1_048_576,
    sends: 8,
    accepts: 4,
    backlog: 16,
    close_timeout: skein_lib::Duration::from_secs(30),
    retry: skein_lib::Duration::from_millis(250),
};

#[cfg(test)]
mod tests;
