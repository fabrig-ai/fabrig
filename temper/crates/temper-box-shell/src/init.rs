//! Facts and file creation for `temper init` (protocol/setup/init.md, 2).
use crate::{
    directories::{Directories, make_private},
    path,
};
use skein_io::kernel::{Complete, Done, Error, Family, Fd, Op, OpenHow, Submit};
use skein_lib::{Queue, Token};
use skein_shell::{Config, Kernel, Wait};
use std::{
    num::NonZeroU16,
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, PermissionsExt},
    },
    path::{Path, PathBuf},
};

type EnvironmentEntry = (Box<[u8]>, Box<[u8]>);
/// Candidate names only: relative PATH entries are never searched.
pub(crate) fn candidates(environment: &[EnvironmentEntry], name: &[u8]) -> Vec<PathBuf> {
    let Some(value) = temper_shell::environment::PATH.value(environment) else { return Vec::new() };
    value
        .split(|byte| *byte == b':')
        .filter(|entry| entry.starts_with(b"/"))
        .map(|entry| path(entry).join(std::ffi::OsStr::from_bytes(name)))
        .collect()
}
/// Pure choice over supplied facts; the shell alone inspects candidates.
pub(crate) fn lookup(environment: &[EnvironmentEntry], name: &[u8], facts: &[(PathBuf, bool)]) -> Option<PathBuf> {
    candidates(environment, name)
        .into_iter()
        .find(|candidate| facts.iter().any(|(path, executable)| path == candidate && *executable))
}
fn find(environment: &[EnvironmentEntry], name: &[u8]) -> Option<PathBuf> {
    let entries = candidates(environment, name)
        .into_iter()
        .map(|path| {
            let executable =
                std::fs::metadata(&path).is_ok_and(|metadata| metadata.is_file() && metadata.mode() & 0o111 != 0);
            (path, executable)
        })
        .collect::<Vec<_>>();
    lookup(environment, name, &entries)
}
/// A small, synchronous startup shell over the binary's io adapter.
struct Io {
    kernel: Kernel,
    submissions: Queue<Submit>,
    completions: Queue<Complete>,
}
impl Io {
    fn open() -> Result<Self, String> {
        Ok(Self {
            kernel: Kernel::open(Config { operations: 1 }).map_err(|why| why.to_string())?,
            submissions: Queue::with_capacity(1),
            completions: Queue::with_capacity(1),
        })
    }
    fn complete(&mut self, kind: Op) -> Complete {
        self.submissions.push(Submit { op: Token::new(1), kind });
        loop {
            self.kernel.submit(&mut self.submissions, Wait::Forever);
            self.kernel.reap(&mut self.completions);
            if let Some(completion) = self.completions.pop() {
                return completion;
            }
        }
    }
    fn perform(&mut self, kind: Op) -> Result<Done, Error> {
        self.complete(kind).result
    }
    fn socket(&mut self) -> Result<Fd, String> {
        let Done::Fd(fd) = self.perform(Op::Socket { family: Family::Ipv4 }).map_err(io_error)? else {
            unreachable!("socket answers a descriptor")
        };
        Ok(fd)
    }
    fn port(&mut self, preferred: u16) -> Result<(Fd, NonZeroU16), String> {
        let (fd, address) = self.bound_port(preferred)?;
        self.listened(fd, address)
    }
    fn bound_port(&mut self, preferred: u16) -> Result<(Fd, std::net::SocketAddr), String> {
        let fd = self.socket()?;
        let preferred = ([127, 0, 0, 1], preferred).into();
        let bound = match self.perform(Op::Bind { fd, addr: preferred }) {
            Ok(bound) => Ok(bound),
            Err(_) => self.perform(Op::Bind { fd, addr: ([127, 0, 0, 1], 0).into() }).map_err(io_error),
        };
        match bound {
            Ok(Done::Bound(address)) => Ok((fd, address)),
            Ok(_) => unreachable!("bind answers an address"),
            Err(why) => {
                self.close(fd);
                Err(why)
            }
        }
    }
    /// skein sets `SO_REUSEADDR` on bind: two merely bound sockets can share a
    /// port, and listening makes a reservation exclusive. Another init that
    /// listened first between this one's bind and listen spends a try; a
    /// replacement asks the kernel for another port, as a taken port does.
    fn listened(&mut self, mut fd: Fd, mut address: std::net::SocketAddr) -> Result<(Fd, NonZeroU16), String> {
        let backlog = jig_protocol_config::profile(jig_protocol_config::Profile::Small).io.backlog;
        for attempt in 1..=crate::fake_backends::PORT_ATTEMPTS {
            match self.perform(Op::Listen { fd, backlog }) {
                Ok(_) => return Ok((fd, NonZeroU16::new(address.port()).expect("bound port"))),
                Err(Error::AddressInUse) if attempt < crate::fake_backends::PORT_ATTEMPTS => {
                    self.close(fd);
                    (fd, address) = self.bound_port(0)?;
                }
                Err(why) => {
                    self.close(fd);
                    return Err(io_error(why));
                }
            }
        }
        unreachable!("the last listen succeeds or refuses")
    }
    fn ports(&mut self, preferred: &[u16]) -> Result<Vec<(Fd, NonZeroU16)>, String> {
        let mut held = Vec::with_capacity(preferred.len());
        for port in preferred {
            match self.port(*port) {
                Ok(pair) => held.push(pair),
                Err(why) => {
                    for (fd, _) in held {
                        self.close(fd);
                    }
                    return Err(why);
                }
            }
        }
        Ok(held)
    }
    fn close(&mut self, fd: Fd) {
        let result = self.perform(Op::Close { fd });
        debug_assert!(result.is_ok(), "startup descriptor closes");
    }
    fn write(&mut self, destination: &Path, bytes: Box<[u8]>, mode: u32) -> Result<(), String> {
        let root = skein_shell::open_root(destination.parent().expect("file parent"))
            .map_err(|errno| format!("errno {errno}"))?;
        let opened = self.perform(Op::Open {
            root,
            path: destination.file_name().expect("file name").as_bytes().into(),
            how: OpenHow::Create { mode: Some(mode) },
        });
        self.close(root);
        let Done::Fd(fd) = opened.map_err(io_error)? else { unreachable!("open answers descriptor") };
        let mut bytes = bytes;
        let mut from = 0;
        let result = (|| {
            while usize::try_from(from).expect("file length fits") < bytes.len() {
                let completion = self.complete(Op::Write { fd, bytes, from, at: u64::from(from) });
                let Op::Write { bytes: returned, .. } = completion.kind else { unreachable!("write returned") };
                bytes = returned;
                let Done::Count(count) = completion.result.map_err(io_error)? else {
                    unreachable!("write answers count")
                };
                from += count;
            }
            // Create's mode is filtered by umask; init's public files have an exact mode.
            std::fs::set_permissions(destination, std::fs::Permissions::from_mode(mode))
                .map_err(|why| why.to_string())?;
            Ok(())
        })();
        self.close(fd);
        result
    }
}
fn io_error(why: Error) -> String {
    format!("errno {}", crate::errno(why))
}

#[derive(Debug)]
pub(crate) struct Written {
    pub paths: Vec<PathBuf>,
    pub listeners: Vec<(Fd, NonZeroU16)>,
    pub missing_git: bool,
    pub no_sign_in: Option<String>,
    pub person: temper_init::Person,
    pub login: Option<Box<[u8]>>,
}
/// Refusals are checked before directories or files are created. The held
/// listeners stay open for the caller to close or hand to the box.
pub(crate) fn write(
    directories: &Directories,
    environment: &[EnvironmentEntry],
    binary: &Path,
    preferred: [u16; 5],
) -> Result<Written, String> {
    write_with_program(directories, environment, binary, preferred, None)
}
pub(crate) fn write_with_program(
    directories: &Directories,
    environment: &[EnvironmentEntry],
    binary: &Path,
    preferred: [u16; 5],
    program: Option<&Path>,
) -> Result<Written, String> {
    write_deployment(directories, environment, binary, &preferred, program, true)
}
pub(crate) fn write_bare(
    directories: &Directories,
    environment: &[EnvironmentEntry],
    binary: &Path,
    preferred: [u16; 2],
) -> Result<Written, String> {
    write_deployment(directories, environment, binary, &preferred, None, false)
}
fn check_destinations(directories: &Directories, boxed: bool) -> Result<(), String> {
    let mut destinations = vec![
        directories.document(),
        path(&directories.data).join("worker.secret"),
        path(&directories.data).join("worker.digest"),
    ];
    if boxed {
        destinations
            .extend([path(&directories.data).join("demo/llm.key"), path(&directories.data).join("demo/bin/git")]);
    }
    for destination in destinations {
        match std::fs::symlink_metadata(&destination) {
            Ok(_) => return Err(format!("{}: already exists", destination.display())),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
            Err(why) => return Err(format!("{}: {why}", destination.display())),
        }
    }
    Ok(())
}
fn write_deployment(
    directories: &Directories,
    environment: &[EnvironmentEntry],
    binary: &Path,
    preferred: &[u16],
    program: Option<&Path>,
    boxed: bool,
) -> Result<Written, String> {
    check_destinations(directories, boxed)?;
    let git = find(environment, b"git");
    let claude = match program {
        Some(path) => Some(path.to_path_buf()),
        None => find(environment, b"claude"),
    };
    let mut io = Io::open()?;
    let listeners = io.ports(preferred)?;
    let result = (|| {
        let secret = skein_shell::crypto_seed().map_err(|errno| format!("random secret: errno {errno}"))?;
        let directories_facts = temper_init::Directories { config: &directories.config, data: &directories.data };
        let binary = binary.as_os_str().as_bytes();
        let git = git.as_ref().map(|path| path.as_os_str().as_bytes());
        let claude = claude.as_ref().map(|path| path.as_os_str().as_bytes());
        directories.open(skein_shell::effective_user())?;
        let no_sign_in = if boxed {
            let key = path(&directories.data).join("provider.key");
            let result = if !key.exists() && find(environment, b"openssl").is_none() {
                Err("openssl is not on PATH".into())
            } else {
                crate::forge_sign_in::make_missing(&key, true)
            };
            result.err()
        } else {
            None
        };
        let (files, person, login) = if boxed {
            let demo_key = skein_shell::crypto_seed().map_err(|errno| format!("random demo key: errno {errno}"))?;
            let login = crate::login(skein_shell::effective_user()).ok();
            let answer = temper_init::files(
                &temper_init::Facts {
                    binary,
                    git,
                    claude,
                    login: login.as_deref(),
                    sign_in: no_sign_in.is_none(),
                    ports: temper_init::Ports {
                        clients: listeners[0].1,
                        owned: listeners[1].1,
                        forge: listeners[2].1,
                        llm: listeners[3].1,
                        webhooks: listeners[4].1,
                    },
                },
                &directories_facts,
                secret.expose().try_into().expect("32-byte seed"),
                demo_key.expose().try_into().expect("32-byte seed"),
            )
            .map_err(|why| format!("{:?}: {:?}", why.fact, why.reason))?;
            (answer.files, answer.person, login)
        } else {
            let files = temper_init::bare(
                &temper_init::Bare { binary, git, claude, clients: listeners[0].1, webhooks: listeners[1].1 },
                &directories_facts,
                secret.expose().try_into().expect("32-byte seed"),
            )
            .map_err(|why| format!("{:?}: {:?}", why.fact, why.reason))?;
            (files, temper_init::Person::Found, None)
        };
        directories.open(skein_shell::effective_user())?;
        let data = path(&directories.data);
        if boxed {
            make_private(&data.join("demo"), skein_shell::effective_user())?;
            make_private(&data.join("demo/bin"), skein_shell::effective_user())?;
        }
        let config = path(&directories.config);
        if !config.exists() {
            make_private(config, skein_shell::effective_user())?;
        }
        let mut paths = Vec::with_capacity(files.len());
        for file in files {
            let root = match file.directory {
                temper_init::Directory::Config => config,
                temper_init::Directory::Data => path(&directories.data),
            };
            let destination = root.join(path(file.name));
            let result = match file.content {
                temper_init::Content::Bytes { bytes, private } => {
                    io.write(&destination, bytes, if private { 0o600 } else { 0o644 })
                }
                // skein-io at the pin has no symlink operation; like exact modes above,
                // creating the link stays in this startup shell (protocol/setup/init.md, 2).
                temper_init::Content::Link { target } => {
                    std::os::unix::fs::symlink(path(&target), &destination).map_err(|why| why.to_string())
                }
            };
            result.map_err(|why| format!("{}: {why}", destination.display()))?;
            paths.push(destination);
        }
        Ok((paths, git.is_none(), person, login, no_sign_in))
    })();
    match result {
        Ok((paths, missing_git, person, login, no_sign_in)) => {
            Ok(Written { paths, listeners, missing_git, no_sign_in, person, login })
        }
        Err(why) => {
            for (fd, _) in listeners {
                io.close(fd);
            }
            Err(why)
        }
    }
}
pub(crate) fn core(directories: &Directories, environment: &[crate::EnvironmentEntry]) -> Result<Written, String> {
    let binary = std::env::current_exe().map_err(|why| why.to_string())?;
    write(directories, environment, &binary, [8367, 8368, 8369, 8370, 8371])
}
pub(crate) fn bare(directories: &Directories, environment: &[crate::EnvironmentEntry]) -> Result<Written, String> {
    let binary = std::env::current_exe().map_err(|why| why.to_string())?;
    write_bare(directories, environment, &binary, [8367, 8371])
}
pub(crate) fn run(
    directories: &Directories,
    boxed: bool,
    environment: &[crate::EnvironmentEntry],
) -> Result<u8, String> {
    let written = if boxed { core(directories, environment)? } else { bare(directories, environment)? };
    for path in written.paths {
        eprintln!("init: wrote {}", path.display());
    }
    if crate::bundle::OWN.is_none() {
        eprintln!(
            "init: this temper has no browser (built with TEMPER_INTERNAL_BUNDLE=none): the browser is not served"
        );
    }
    if let Some(why) = &written.no_sign_in {
        eprintln!("init: no sign-in through temper: {why}");
    }
    if written.missing_git {
        eprintln!("init: no git on PATH: a real forge's repositories need git 2.41 or later, set as workers[].git");
    }
    if let Some(line) = person_line(written.person, written.login.as_deref(), skein_shell::effective_user()) {
        eprintln!("{line}");
    }
    eprintln!("init: after starting, continue with ChatGPT or add an OpenAI API key from the accounts page");
    let mut line =
        String::from(if boxed { "init: start it with temper box" } else { "init: start it with temper serve" });
    for (index, (name, value)) in
        [("config", &directories.config), ("data", &directories.data), ("cache", &directories.cache)].iter().enumerate()
    {
        if directories.origins[index] == skein_shell::Origin::Flag {
            use std::fmt::Write;
            write!(line, " --{name} {}", path(value).display()).expect("string");
        }
    }
    eprintln!("{line}");
    crate::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
    Ok(0)
}
fn person_line(person: temper_init::Person, login: Option<&[u8]>, user: u32) -> Option<String> {
    match person {
        temper_init::Person::Found => None,
        temper_init::Person::FellBack => Some(match login {
            Some(login) => format!(
                "init: {} cannot be a Forgejo login: forge 3's person is person",
                String::from_utf8_lossy(login)
            ),
            None => format!("init: uid {user} has no login in /etc/passwd: forge 3's person is person"),
        }),
    }
}
#[cfg(test)]
mod tests;
