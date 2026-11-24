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
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::{Path, PathBuf},
};

use temper_shell::directories::EnvironmentEntry;
use temper_shell::lookup::find;
/// A small, synchronous startup shell over the binary's io adapter.
pub struct Io {
    kernel: Kernel,
    submissions: Queue<Submit>,
    completions: Queue<Complete>,
}
impl Io {
    pub fn open() -> Result<Self, String> {
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
    pub fn perform(&mut self, kind: Op) -> Result<Done, Error> {
        self.complete(kind).result
    }
    pub fn socket(&mut self) -> Result<Fd, String> {
        socket(self)
    }
    pub fn listened(&mut self, fd: Fd, address: std::net::SocketAddr) -> Result<(Fd, NonZeroU16), String> {
        listened(self, fd, address)
    }
    pub fn close(&mut self, fd: Fd) {
        close(self, fd);
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
trait StartupIo {
    fn perform(&mut self, kind: Op) -> Result<Done, Error>;
}

impl StartupIo for Io {
    fn perform(&mut self, kind: Op) -> Result<Done, Error> {
        Io::perform(self, kind)
    }
}

fn socket(io: &mut impl StartupIo) -> Result<Fd, String> {
    let Done::Fd(fd) = io.perform(Op::Socket { family: Family::Ipv4 }).map_err(io_error)? else {
        unreachable!("socket answers a descriptor")
    };
    Ok(fd)
}
fn port(io: &mut impl StartupIo, preferred: u16) -> Result<(Fd, NonZeroU16), String> {
    let (fd, address) = bound_port(io, preferred)?;
    listened(io, fd, address)
}
fn bound_port(io: &mut impl StartupIo, preferred: u16) -> Result<(Fd, std::net::SocketAddr), String> {
    let fd = socket(io)?;
    let preferred = ([127, 0, 0, 1], preferred).into();
    let bound = match io.perform(Op::Bind { fd, addr: preferred }) {
        Ok(bound) => Ok(bound),
        Err(_) => io.perform(Op::Bind { fd, addr: ([127, 0, 0, 1], 0).into() }).map_err(io_error),
    };
    match bound {
        Ok(Done::Bound(address)) => Ok((fd, address)),
        Ok(_) => unreachable!("bind answers an address"),
        Err(why) => {
            close(io, fd);
            Err(why)
        }
    }
}
/// skein sets `SO_REUSEADDR` on bind: two merely bound sockets can share a
/// port, and listening makes a reservation exclusive. Another init that
/// listened first between this one's bind and listen spends a try; a
/// replacement asks the kernel for another port, as a taken port does.
fn listened(
    io: &mut impl StartupIo,
    mut fd: Fd,
    mut address: std::net::SocketAddr,
) -> Result<(Fd, NonZeroU16), String> {
    let backlog = jig_protocol_config::stock().io.backlog;
    for attempt in 1..=crate::fake_backends::PORT_ATTEMPTS {
        match io.perform(Op::Listen { fd, backlog }) {
            Ok(_) => return Ok((fd, NonZeroU16::new(address.port()).expect("bound port"))),
            Err(Error::AddressInUse) if attempt < crate::fake_backends::PORT_ATTEMPTS => {
                close(io, fd);
                (fd, address) = bound_port(io, 0)?;
            }
            Err(why) => {
                close(io, fd);
                return Err(io_error(why));
            }
        }
    }
    unreachable!("the last listen succeeds or refuses")
}
fn deployment_ports(io: &mut impl StartupIo, preferred: &[u16]) -> Result<Vec<(Fd, NonZeroU16)>, String> {
    let clients = port(io, preferred[0])?;
    let next = clients.1.get().checked_add(1).unwrap_or(0);
    // Zero asks init to use listen's next port. Tests and fake-backend starts
    // may explicitly select a different webhook reservation.
    let webhooks = match port(io, if preferred[1] == 0 { next } else { preferred[1] }) {
        Ok(pair) => pair,
        Err(why) => {
            close(io, clients.0);
            return Err(why);
        }
    };
    Ok(vec![clients, webhooks])
}
fn close(io: &mut impl StartupIo, fd: Fd) {
    let result = io.perform(Op::Close { fd });
    debug_assert!(result.is_ok(), "startup descriptor closes");
}

fn io_error(why: Error) -> String {
    format!("errno {}", crate::errno(why))
}

#[derive(Debug)]
pub struct Written {
    pub paths: Vec<PathBuf>,
    pub listeners: Vec<(Fd, NonZeroU16)>,
    pub missing_git: bool,
    pub no_sign_in: Option<String>,
    pub person: temper_init::Person,
    pub login: Option<Box<[u8]>>,
}
/// Refusals are checked before directories or files are created. The held
/// listeners stay open for the caller to close or hand to the box.
pub fn write(
    directories: &Directories,
    environment: &[EnvironmentEntry],
    binary: &Path,
    preferred: [u16; 2],
) -> Result<Written, String> {
    write_with_program(directories, environment, binary, preferred, None)
}
pub(crate) fn write_with_program(
    directories: &Directories,
    environment: &[EnvironmentEntry],
    binary: &Path,
    preferred: [u16; 2],
    program: Option<&Path>,
) -> Result<Written, String> {
    write_deployment(directories, environment, binary, &preferred, program, true, ProviderKey::Generated)
}
pub fn write_bare(
    directories: &Directories,
    environment: &[EnvironmentEntry],
    binary: &Path,
    preferred: [u16; 2],
) -> Result<Written, String> {
    write_deployment(directories, environment, binary, &preferred, None, false, ProviderKey::Generated)
}

#[derive(Clone, Copy)]
enum ProviderKey {
    Generated,
    Given(&'static [u8]),
}
impl ProviderKey {
    fn make_missing(
        self,
        key: &Path,
        environment: &[EnvironmentEntry],
        startup: &mut impl DeploymentIo,
    ) -> Result<(), String> {
        if let Self::Generated = self
            && !startup.exists(key)
            && startup.find(environment, b"openssl").is_none()
        {
            return Err("openssl is not on PATH".into());
        }
        startup.make_provider_key(key, self)
    }
}

pub fn write_with_key(
    directories: &Directories,
    environment: &[EnvironmentEntry],
    binary: &Path,
    preferred: [u16; 2],
    key: &'static [u8],
) -> Result<Written, String> {
    write_deployment(directories, environment, binary, &preferred, None, true, ProviderKey::Given(key))
}
trait DeploymentIo {
    type ListenerIo: StartupIo;

    fn symlink_metadata(&mut self, path: &Path) -> std::io::Result<()>;
    fn find(&mut self, environment: &[EnvironmentEntry], name: &[u8]) -> Option<PathBuf>;
    fn open(&mut self) -> Result<Self::ListenerIo, String>;
    fn seed(&mut self) -> Result<skein_lib::Secret, i32>;
    fn open_directories(&mut self, directories: &Directories) -> Result<(), String>;
    fn login(&mut self) -> Result<Box<[u8]>, String>;
    fn make_private(&mut self, path: &Path) -> Result<(), String>;
    fn exists(&mut self, path: &Path) -> bool;
    fn write(&mut self, io: &mut Self::ListenerIo, path: &Path, bytes: Box<[u8]>, mode: u32) -> Result<(), String>;
    fn symlink(&mut self, target: &Path, path: &Path) -> Result<(), String>;
    fn make_provider_key(&mut self, path: &Path, source: ProviderKey) -> Result<(), String>;
}

struct Machine;

impl DeploymentIo for Machine {
    type ListenerIo = Io;

    fn symlink_metadata(&mut self, path: &Path) -> std::io::Result<()> {
        std::fs::symlink_metadata(path).map(|_| ())
    }
    fn find(&mut self, environment: &[EnvironmentEntry], name: &[u8]) -> Option<PathBuf> {
        find(environment, name)
    }
    fn open(&mut self) -> Result<Io, String> {
        Io::open()
    }
    fn seed(&mut self) -> Result<skein_lib::Secret, i32> {
        skein_shell::crypto_seed()
    }
    fn open_directories(&mut self, directories: &Directories) -> Result<(), String> {
        directories.open(skein_shell::effective_user())
    }
    fn login(&mut self) -> Result<Box<[u8]>, String> {
        crate::login(skein_shell::effective_user())
    }
    fn make_private(&mut self, path: &Path) -> Result<(), String> {
        make_private(path, skein_shell::effective_user())
    }
    fn exists(&mut self, path: &Path) -> bool {
        path.exists()
    }
    fn write(&mut self, io: &mut Io, path: &Path, bytes: Box<[u8]>, mode: u32) -> Result<(), String> {
        io.write(path, bytes, mode)
    }
    fn symlink(&mut self, target: &Path, path: &Path) -> Result<(), String> {
        std::os::unix::fs::symlink(target, path).map_err(|why| why.to_string())
    }
    fn make_provider_key(&mut self, path: &Path, source: ProviderKey) -> Result<(), String> {
        match source {
            ProviderKey::Generated => crate::forge_sign_in::make_missing(path, true),
            ProviderKey::Given(bytes) => crate::forge_sign_in::make_missing_given(path, bytes),
        }
    }
}

fn check_destinations(directories: &Directories, boxed: bool, startup: &mut impl DeploymentIo) -> Result<(), String> {
    let mut destinations = vec![
        path(&directories.config).join(temper_config_shell::FILE),
        path(&directories.config).join(temper_config_shell::FORMER),
        path(&directories.data).join("worker.secret"),
    ];
    if boxed {
        destinations.push(path(&directories.data).join("demo/bin/git"));
    }
    for destination in destinations {
        match startup.symlink_metadata(&destination) {
            Ok(()) => return Err(format!("{}: already exists", destination.display())),
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
    provider_key: ProviderKey,
) -> Result<Written, String> {
    write_deployment_with(directories, environment, binary, preferred, program, boxed, provider_key, &mut Machine)
}
#[expect(clippy::too_many_arguments, reason = "startup effects sit beside init's command inputs")]
fn write_deployment_with(
    directories: &Directories,
    environment: &[EnvironmentEntry],
    binary: &Path,
    preferred: &[u16],
    program: Option<&Path>,
    boxed: bool,
    provider_key: ProviderKey,
    startup: &mut impl DeploymentIo,
) -> Result<Written, String> {
    check_destinations(directories, boxed, startup)?;
    let git = startup.find(environment, b"git");
    let claude = match program {
        Some(path) => Some(path.to_path_buf()),
        None => startup.find(environment, b"claude"),
    };
    let mut io = startup.open()?;
    let listeners = deployment_ports(&mut io, preferred)?;
    let result = (|| {
        let secret = startup.seed().map_err(|errno| format!("random secret: errno {errno}"))?;
        let directories_facts = temper_init::Directories { config: &directories.config, data: &directories.data };
        let binary = binary.as_os_str().as_bytes();
        let git = git.as_ref().map(|path| path.as_os_str().as_bytes());
        let claude = claude.as_ref().map(|path| path.as_os_str().as_bytes());
        startup.open_directories(directories)?;
        let mut no_sign_in = None;
        let (files, person, login) = if boxed {
            let login = startup.login().ok();
            let answer = temper_init::files(
                &temper_init::Facts { binary, git, claude, login: login.as_deref() },
                &directories_facts,
                secret.expose().try_into().expect("32-byte seed"),
            )
            .map_err(|why| format!("{:?}: {:?}", why.fact, why.reason))?;
            (answer.files, answer.person, login)
        } else {
            let files = temper_init::bare(
                &temper_init::Bare { binary, git, claude },
                &directories_facts,
                secret.expose().try_into().expect("32-byte seed"),
            )
            .map_err(|why| format!("{:?}: {:?}", why.fact, why.reason))?;
            (files, temper_init::Person::Found, None)
        };
        startup.open_directories(directories)?;
        let data = path(&directories.data);
        if boxed {
            startup.make_private(&data.join("demo"))?;
            startup.make_private(&data.join("demo/bin"))?;
        }
        let config = path(&directories.config);
        if !startup.exists(config) {
            startup.make_private(config)?;
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
                    startup.write(&mut io, &destination, bytes, if private { 0o600 } else { 0o644 })
                }
                // skein-io at the pin has no symlink operation; like exact modes above,
                // creating the link stays in this startup shell (protocol/setup/init.md, 2).
                temper_init::Content::Link { target } => startup.symlink(path(&target), &destination),
            };
            result.map_err(|why| format!("{}: {why}", destination.display()))?;
            paths.push(destination);
            if boxed && file.name == b"worker.secret" {
                let key = data.join("provider.key");
                let existed = startup.exists(&key);
                no_sign_in = provider_key.make_missing(&key, environment, startup).err();
                if !existed && no_sign_in.is_none() {
                    paths.push(key);
                }
            }
        }
        let file = configuration(listeners[0].1, listeners[1].1);
        let destination = config.join(temper_config_shell::FILE);
        startup
            .write(
                &mut io,
                &destination,
                temper_config_shell::render(&file, temper_config_shell::Shape::Standalone),
                0o644,
            )
            .map_err(|why| format!("{}: {why}", destination.display()))?;
        paths.push(destination);
        Ok((paths, git.is_none(), person, login, no_sign_in))
    })();
    match result {
        Ok((paths, missing_git, person, login, no_sign_in)) => {
            Ok(Written { paths, listeners, missing_git, no_sign_in, person, login })
        }
        Err(why) => {
            for (fd, _) in listeners {
                close(&mut io, fd);
            }
            Err(why)
        }
    }
}
/// The explicit values init writes; all other values come from the reader.
pub(crate) fn configuration(clients: NonZeroU16, webhooks: NonZeroU16) -> temper_config_shell::File {
    let listen = std::net::SocketAddr::from(([127, 0, 0, 1], clients.get()));
    temper_config_shell::File {
        listen: Some(listen),
        webhooks: (clients.get().checked_add(1) != Some(webhooks.get()))
            .then(|| ([127, 0, 0, 1], webhooks.get()).into()),
        ..temper_config_shell::File::default()
    }
}
pub(crate) fn core(directories: &Directories, environment: &[crate::EnvironmentEntry]) -> Result<Written, String> {
    let binary = std::env::current_exe().map_err(|why| why.to_string())?;
    write(directories, environment, &binary, [8367, 0])
}
pub(crate) fn bare(directories: &Directories, environment: &[crate::EnvironmentEntry]) -> Result<Written, String> {
    let binary = std::env::current_exe().map_err(|why| why.to_string())?;
    write_bare(directories, environment, &binary, [8367, 0])
}
pub(crate) fn run(
    directories: &Directories,
    boxed: bool,
    environment: &[crate::EnvironmentEntry],
) -> Result<u8, String> {
    let written = if boxed { core(directories, environment)? } else { bare(directories, environment)? };
    Ok(finish_written(
        directories,
        boxed,
        written,
        crate::bundle::OWN.is_some(),
        skein_shell::effective_user,
        (&mut std::io::stdout(), &mut std::io::stderr()),
        crate::close,
    ))
}

fn finish_written(
    directories: &Directories,
    boxed: bool,
    written: Written,
    browser: bool,
    user: impl FnOnce() -> u32,
    (_stdout, stderr): (&mut impl std::io::Write, &mut impl std::io::Write),
    close: impl FnOnce(Vec<Fd>),
) -> u8 {
    let mut print = |line: std::fmt::Arguments<'_>| writeln!(stderr, "{line}").expect("writing to stderr");
    for path in written.paths {
        print(format_args!("init: wrote {}", path.display()));
    }
    if !browser {
        print(format_args!(
            "init: this temper has no browser (built with TEMPER_INTERNAL_BUNDLE=none): the browser is not served"
        ));
    }
    if let Some(why) = &written.no_sign_in {
        print(format_args!("init: no sign-in through temper: {why}"));
    }
    if written.missing_git {
        print(format_args!(
            "init: no git on PATH: a real forge's repositories need git 2.41 or later, set as [programs] git"
        ));
    }
    if let Some(line) = person_line(written.person, written.login.as_deref(), user()) {
        print(format_args!("{line}"));
    }
    print(format_args!("init: after starting, continue with ChatGPT or add an OpenAI API key from the accounts page"));
    let mut line =
        String::from(if boxed { "init: start it with temper box" } else { "init: start it with temper-box serve" });
    for (index, (name, value)) in
        [("config", &directories.config), ("data", &directories.data), ("cache", &directories.cache)].iter().enumerate()
    {
        if directories.origins[index] == skein_shell::Origin::Flag {
            use std::fmt::Write;
            write!(line, " --{name} {}", path(value).display()).expect("string");
        }
    }
    print(format_args!("{line}"));
    close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
    0
}
fn person_line(person: temper_init::Person, login: Option<&[u8]>, user: u32) -> Option<String> {
    match person {
        temper_init::Person::Found => None,
        temper_init::Person::FellBack => Some(match login {
            Some(login) => format!(
                "init: {} cannot be a Forgejo login: the owned Forgejo's person is person",
                String::from_utf8_lossy(login)
            ),
            None => format!("init: uid {user} has no login in /etc/passwd: the owned Forgejo's person is person"),
        }),
    }
}
#[cfg(test)]
pub(crate) mod startup;
#[cfg(test)]
mod tests;
