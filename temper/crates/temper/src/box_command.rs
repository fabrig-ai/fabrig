//! The checked configuration's steps, supervised on one skein-io loop (protocol/setup.md, 7.1).
use skein_io::kernel::{Exit, Fd};
use skein_shell::{Clock, Config, Host, Kernel};
use std::os::unix::ffi::OsStrExt;
use temper_box::host::{Echo, Service, Settings};
use temper_forge_setup::group;

pub(crate) const ECHO_SETTINGS: skein_shell::Settings = skein_shell::Settings {
    tool: "temper",
    table: &[skein_shell::Setting {
        name: "echo",
        value: "none|steps",
        about: "Show the box's lines and a failed step's log, or every step's lines too.",
        fallback: skein_shell::Fallback::Text("none"),
    }],
};
type Arguments = Box<[Box<[u8]>]>;
pub(crate) fn echo(
    arguments: &[Box<[u8]>],
    environment: &[super::EnvironmentEntry],
) -> Result<(Echo, Arguments), String> {
    let resolved = skein_shell::resolve(&ECHO_SETTINGS, arguments, environment).map_err(|why| why.to_string())?;
    let value = resolved.value(0);
    let echo = match value.bytes.as_ref() {
        b"none" => Echo::None,
        b"steps" => Echo::Steps,
        _ => {
            let name = match value.origin {
                skein_shell::Origin::Flag => "--echo",
                skein_shell::Origin::Variable => "TEMPER_ECHO",
                skein_shell::Origin::Default | skein_shell::Origin::Base | skein_shell::Origin::Home => {
                    unreachable!("fixed echo fallback")
                }
            };
            return Err(format!("{name}: expected none or steps"));
        }
    };
    Ok((echo, resolved.rest().into()))
}

/// Checks the document before starting its steps; refusals contain unprefixed lines.
pub fn run(directories: &super::Directories, echo: Echo) -> Result<Exit, String> {
    let (bytes, held) = configuration(directories)?;
    run_configuration(directories, &bytes, held, Box::new([]), echo)
}

/// Runs a document already retained in its effective directories. Owns all supplied listeners.
pub(crate) fn run_configuration(
    directories: &super::Directories,
    bytes: &[u8],
    held: Vec<(Fd, std::num::NonZeroU16)>,
    first: Box<[temper_forge_setup::Step]>,
    echo: Echo,
) -> Result<Exit, String> {
    let mut owned = held.iter().map(|(fd, _)| *fd).collect::<Vec<_>>();
    let opened = (|| {
        let inherited = skein_shell::inherited_listeners().map_err(super::listeners_refusal)?;
        owned.extend(inherited.iter().map(|(fd, _)| *fd));
        let held = held
            .into_iter()
            .map(|(fd, _)| skein_shell::listener_at(fd).map(|address| (fd, address)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(super::listeners_refusal)?;
        let deployment = super::setup::check(bytes, &directories.jig())?;
        bridge_groups(directories, &deployment)?;
        let machine = group::Machine {
            flags: directories.flags(),
            directories: directories.jig(),
            limits: deployment.checked.limits.clone(),
            llm: deployment.demo.as_ref().map(|demo| demo.llm().listen()),
            forgejo_listens: cfg!(target_os = "linux"),
        };
        let groups = startup_groups(directories, &machine);
        let slots = GROUP_SLOTS.max(u32::try_from(groups.len()).map_err(|_| "too many group files".to_string())?);
        let machine_ports = machine_ports(&deployment);
        let super::setup::Deployment { named, limits, .. } = deployment;
        let directory = super::path(&directories.data);
        let program = std::env::current_exe().map_err(|why| why.to_string())?;
        let environment = std::env::vars_os()
            .map(|(name, value)| {
                let mut entry = name.as_bytes().to_vec();
                entry.push(b'=');
                entry.extend_from_slice(value.as_bytes());
                entry.into_boxed_slice()
            })
            .collect();
        let mut serve = vec![super::address(&named.clients)?];
        if let Some(webhooks) = &named.webhooks {
            serve.push(super::address(&webhooks.listener)?);
        }
        let mut always = first.into_vec();
        always.push(temper_forge_setup::step::serve(&directories.flags(), serve.into_boxed_slice()));
        let steps = temper_forge_setup::Steps { always: always.into_boxed_slice(), asked: Box::new([]) };
        let listeners = open_listeners(directories, &steps, &machine_ports, held, &inherited, &mut owned)?;
        let signals = skein_shell::open_termination_signals().map_err(|errno| format!("signals: errno {errno}"))?;
        owned.push(signals);
        let root = skein_shell::open_root(directory).map_err(|errno| format!("directory: errno {errno}"))?;
        owned.push(root);
        let settings = Settings {
            groups: machine,
            slots,
            program: program.as_os_str().as_bytes().into(),
            environment,
            listeners: listeners.into_boxed_slice(),
            serve: b"serve".as_slice().into(),
            root,
            echo,
            data: {
                let bytes = directory.as_os_str().as_bytes();
                let mut end = bytes.len();
                while end > 1 && bytes.get(end - 1) == Some(&b'/') {
                    end -= 1;
                }
                bytes[..end].into()
            },
            signals,
            stdout: Fd::new(1),
            stderr: Fd::new(2),
        };
        let service = Service::new(&limits, steps, groups, settings).map_err(|why| format!("box limits: {why:?}"))?;
        let kernel = Kernel::open(Config { operations: service.operations() }).map_err(|why| why.to_string())?;
        Ok((service, kernel))
    })();
    let (mut service, mut kernel) = match opened {
        Ok(opened) => opened,
        Err(why) => {
            super::close(owned);
            return Err(why);
        }
    };
    Ok(skein_shell::drive(&mut kernel, &Clock::new(), &mut service))
}

fn open_listeners(
    directories: &super::Directories,
    steps: &temper_forge_setup::Steps,
    machine_ports: &[std::net::SocketAddr],
    held: Vec<(Fd, std::net::SocketAddr)>,
    inherited: &[(Fd, std::net::SocketAddr)],
    owned: &mut Vec<Fd>,
) -> Result<Vec<(Fd, std::net::SocketAddr)>, String> {
    let mut addresses = Vec::new();
    for step in &steps.always {
        for address in step.listeners() {
            if addresses.contains(address) {
                return Err(format!(
                    "{}: port {} ({address}): two steps name the same listener",
                    directories.document().display(),
                    address.port()
                ));
            }
            addresses.push(*address);
        }
    }
    for address in machine_ports {
        if !addresses.contains(address) {
            addresses.push(*address);
        }
    }
    for (fd, address) in inherited {
        if !addresses.contains(address) {
            return Err(format!(
                "{}: inherited listener fd {} at {address}: no step names this listener",
                directories.document().display(),
                fd.raw()
            ));
        }
    }
    let mut listeners = Vec::new();
    for addr in &addresses {
        let fd = if let Some((fd, _)) = inherited.iter().chain(&held).find(|(_, address)| address == addr) {
            *fd
        } else {
            let (fd, _) = skein_shell::open_listener(
                *addr,
                jig_protocol_config::profile(jig_protocol_config::Profile::Small).io.backlog,
            )
            .map_err(|why| {
                format!("{}: port {} ({addr}): cannot listen: {why:?}", directories.document().display(), addr.port())
            })?;
            owned.push(fd);
            fd
        };
        listeners.push((fd, *addr));
    }
    // Forgejo's port must be free for Forgejo to bind (M3a, protocol/setup.md 7.1).
    for (fd, _) in held {
        if !listeners.iter().any(|(listener, _)| *listener == fd) {
            super::close(vec![fd]);
            owned.retain(|owned| *owned != fd);
        }
    }
    Ok(listeners)
}

type Configuration = (Box<[u8]>, Vec<(Fd, std::num::NonZeroU16)>);
/// Prepare a first configuration, keeping init's listeners, or read the existing one.
pub fn configuration(directories: &super::Directories) -> Result<Configuration, String> {
    let config = super::path(&directories.config);
    let empty = match std::fs::read_dir(config) {
        Ok(mut entries) => match entries.next() {
            None => true,
            Some(Ok(_)) => false,
            Some(Err(why)) => return Err(format!("{}: {why}", config.display())),
        },
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => true,
        Err(why) => return Err(format!("{}: {why}", config.display())),
    };
    let mut held = Vec::new();
    if empty {
        held = super::init::core(directories)?.listeners;
        eprintln!("First run: wrote the configuration, {}", directories.document().display());
    }
    match directories.read() {
        Ok(bytes) => Ok((bytes, held)),
        Err(why) => {
            super::close(held.into_iter().map(|(fd, _)| fd).collect());
            Err(why)
        }
    }
}

// The forge component's default number of kept forges (contract 4).
const GROUP_SLOTS: u32 = 4;

/// Machine listeners kept for groups, including groups installed after startup.
pub(crate) fn machine_ports(deployment: &super::setup::Deployment) -> Vec<std::net::SocketAddr> {
    use temper_forge_setup::setup::Mode;
    let mut ports = Vec::new();
    for entry in &deployment.checked.setup.entries {
        match &entry.mode {
            Mode::Owned(owned) if cfg!(target_os = "linux") => ports.push(owned.listen),
            Mode::Fake(fake) => ports.push(fake.listen),
            Mode::Owned(_) | Mode::Managed { .. } => {}
        }
    }
    if let Some(demo) = &deployment.demo {
        ports.push(demo.llm().listen());
    }
    ports
}

/// This bridge goes with the document's application.setup (bx-process).
/// Keep the entry's object bytes, including its relative paths and escaping.
fn bridge_groups(directories: &super::Directories, deployment: &super::setup::Deployment) -> Result<(), String> {
    use std::{fs, io::Write, os::unix::fs::OpenOptionsExt};
    let bytes = directories.read()?;
    let document = jig_protocol_config::document(
        &bytes,
        &jig_protocol_config::DocumentLimits { bytes: super::CONFIG_BYTES, tokens: super::CONFIG_BYTES, depth: 32 },
    )
    .map_err(|_| "configuration changed while starting the box".to_string())?;
    let application = super::member(&document, 0, b"application");
    let setup = super::member(&document, application, b"setup");
    let offsets = token_offsets(&bytes);
    let entries = jig_protocol_config::read::list(&document, setup, super::CONFIG_BYTES)
        .map_err(|_| "configuration changed while starting the box".to_string())?;
    let data = super::path(&directories.data);
    for (entry, at) in deployment.checked.setup.entries.iter().zip(entries.into_boxed()) {
        if matches!(entry.mode, temper_forge_setup::setup::Mode::Managed { .. }) {
            continue;
        }
        let path = data.join("box/groups").join(entry.forge.to_string());
        let create = match fs::symlink_metadata(&path) {
            Ok(_) => false,
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => true,
            Err(why) => return Err(format!("{}: {why}", path.display())),
        };
        let playground = deployment.demo.as_ref().is_some_and(|demo| demo.forge() == entry.forge);
        if create {
            super::directories::make_private(&data.join("box"), skein_shell::effective_user())?;
            super::directories::make_private(&data.join("box/groups"), skein_shell::effective_user())?;
            let end = jig_protocol_config::read::after(&document, at)
                .expect("checked entry")
                .checked_sub(1)
                .expect("object end");
            let start = offsets[usize::try_from(at).expect("token index")];
            let end = offsets[usize::try_from(end).expect("token index")].checked_add(1).expect("object closing brace");
            let content = [
                b"{\"setup\":".as_slice(),
                &bytes[start..end],
                if playground { b",\"playground\":true}" } else { b",\"playground\":false}" },
            ]
            .concat();
            if content.len() > usize::try_from(group::FILE_BYTES).expect("file bound") {
                return Err(format!("{}: group file exceeds {} bytes", path.display(), group::FILE_BYTES));
            }
            let temporary = data.join("box/groups").join(format!(".{}-{}", entry.forge, skein_shell::process_id()));
            let written = (|| {
                let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&temporary)?;
                file.write_all(&content)?;
                file.sync_all()?;
                fs::rename(&temporary, &path)
            })();
            if let Err(why) = written {
                let _ = fs::remove_file(&temporary);
                return Err(format!("{}: {why}", path.display()));
            }
        }
        if !entry.asked && !playground {
            super::directories::make_private(&data.join("box"), skein_shell::effective_user())?;
            super::directories::make_private(&data.join("box/started"), skein_shell::effective_user())?;
            let record = data.join("box/started").join(entry.forge.to_string());
            match fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&record) {
                Ok(_) => {}
                Err(why) if why.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(why) => return Err(format!("{}: {why}", record.display())),
            }
        }
    }
    Ok(())
}

/// Token offsets pair skein's checked structural tokens with their source bytes.
fn token_offsets(input: &[u8]) -> Vec<usize> {
    use skein_json::tokenizer::{self, Event, Request, Tokenizer};
    use skein_lib::{
        Env, Intake, Queue, Time, Wall,
        stream::{Down, Up},
    };
    let length = u32::try_from(input.len()).expect("checked document length");
    let limits = tokenizer::Limits { depth: 32, string: length, number: length, chunk: length.max(1), length };
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut machine = Tokenizer::new(&limits);
    let mut intake = Intake::with_capacity(length.max(tokenizer::largest_demand(&limits)));
    intake.append(input).expect("bounded document");
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    let mut demanded = None;
    let mut offsets = Vec::new();
    for _ in 0..input.len().saturating_mul(4).saturating_add(8) {
        match demanded.take() {
            Some(read) => {
                let up = match intake.meet(read) {
                    Some(bytes) => Up::Bytes(bytes),
                    None => Up::End,
                };
                tokenizer::up(&mut machine, &env, up, &mut above, &mut below);
            }
            None => tokenizer::down(&mut machine, &env, Request::Next, &mut above, &mut below),
        }
        if let Some(Down::Demand { read, .. }) = below.pop() {
            demanded = Some(read);
        }
        match above.pop() {
            Some(Event::Token(_)) => offsets.push(usize::try_from(machine.position().offset).expect("document offset")),
            Some(Event::Done) => return offsets,
            Some(_) => unreachable!("checked JSON document"),
            None => {}
        }
    }
    unreachable!("bounded tokenizer pump")
}

fn startup_groups(directories: &super::Directories, machine: &group::Machine) -> Box<[group::Built]> {
    let directory = super::path(&directories.data).join("box/groups");
    let entries = match std::fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Box::new([]),
        Err(why) => {
            eprintln!("temper box: box/groups: {why}");
            return Box::new([]);
        }
    };
    let mut names = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => {
                let name = entry.file_name();
                if let Some(forge) = canonical_forge(name.as_bytes()) {
                    names.push(forge);
                }
            }
            Err(why) => eprintln!("temper box: box/groups: {why}"),
        }
    }
    names.sort_unstable();
    let mut groups = Vec::new();
    for forge in names {
        match startup_group(&directory.join(forge.to_string()), forge, machine) {
            Ok(group) => groups.push(group),
            Err(why) => eprintln!("temper box: box/groups/{forge}: {why}"),
        }
    }
    groups.into_boxed_slice()
}
fn canonical_forge(name: &[u8]) -> Option<u16> {
    let number = std::str::from_utf8(name).ok()?.parse::<u16>().ok()?;
    (number.to_string().as_bytes() == name).then_some(number)
}
fn startup_group(path: &std::path::Path, forge: u16, machine: &group::Machine) -> Result<group::Built, String> {
    let bytes = skein_shell::read_private_file(path, group::FILE_BYTES, skein_shell::effective_user())
        .map_err(super::unread)?;
    let file = group::read(bytes.expose(), forge, machine).map_err(|why| {
        let prefix = format!("box/groups/{forge}: ");
        String::from_utf8_lossy(why.strip_prefix(prefix.as_bytes()).unwrap_or(&why)).into_owned()
    })?;
    group::build(&file, machine).map_err(|why| String::from_utf8_lossy(&why).into_owned())
}
#[cfg(test)]
mod tests;
