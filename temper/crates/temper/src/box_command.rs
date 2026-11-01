//! The checked configuration's steps, supervised on one skein-io loop (protocol/setup.md, 7.1).
use skein_io::kernel::{Exit, Fd};
use skein_shell::{Clock, Config, Host, Kernel};
use std::os::unix::ffi::OsStrExt;
use temper_box::host::{Echo, Service, Settings};

mod groups;
#[cfg(test)]
mod tests;

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
        let (steps, vacant) = steps(directories, &deployment, first)?;
        let super::setup::Deployment { checked, limits, .. } = deployment;
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
        let listeners = open_listeners(directories, &steps, &vacant, held, &inherited, &mut owned)?;
        let signals = skein_shell::open_termination_signals().map_err(|errno| format!("signals: errno {errno}"))?;
        owned.push(signals);
        let root = skein_shell::open_root(directory).map_err(|errno| format!("directory: errno {errno}"))?;
        owned.push(root);
        let settings = Settings {
            directories: checked
                .setup
                .entries
                .iter()
                .filter_map(|entry| match entry.mode {
                    temper_forge_setup::setup::Mode::Owned(_) | temper_forge_setup::setup::Mode::Fake(_) => {
                        Some((entry.forge, entry.directory.clone()))
                    }
                    temper_forge_setup::setup::Mode::Managed { .. } => None,
                })
                .collect(),
            vacant,
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
        let service = Service::new(&limits, steps, settings).map_err(|why| format!("box limits: {why:?}"))?;
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

type ConfiguredSteps = (temper_forge_setup::Steps, Box<[temper_forge_setup::Group]>);

fn steps(
    directories: &super::Directories,
    deployment: &super::setup::Deployment,
    first: Box<[temper_forge_setup::Step]>,
) -> Result<ConfiguredSteps, String> {
    let super::setup::Deployment { named, checked, demo, owned, .. } = deployment;
    let flags = directories.flags();
    let forgejo_listens = cfg!(target_os = "linux");
    let mut serve = vec![super::address(&named.clients)?];
    if let Some(webhooks) = &named.webhooks {
        serve.push(super::address(&webhooks.listener)?);
    }
    let mut steps =
        temper_fake::steps(&checked.setup, demo.as_ref(), &flags, serve.into_boxed_slice(), forgejo_listens, first);
    for entry in &checked.setup.entries {
        if let Some(group) = steps.asked.iter_mut().find(|group| group.forge == entry.forge) {
            for (_, step) in &mut group.steps {
                owned_notices(step, entry);
            }
        } else {
            for step in &mut steps.always {
                owned_notices(step, entry);
            }
        }
    }
    let before = u32::try_from(steps.always.len() - 1).expect("bounded steps ending in serve");
    let files = groups::files(directories, checked, demo.as_ref(), &flags, forgejo_listens)?;
    let mut asked = steps.asked.into_vec();
    for (mut group, entry) in files {
        group.before = before;
        for (_, step) in &mut group.steps {
            owned_notices(step, &entry);
        }
        asked.push(group);
    }
    steps.asked = asked.into_boxed_slice();
    let mut vacant = Vec::new();
    for (mut group, entry) in groups::kinds(owned.as_ref(), demo.as_ref(), &flags, forgejo_listens) {
        group.before = before;
        for (_, step) in &mut group.steps {
            owned_notices(step, &entry);
        }
        vacant.push(group);
    }
    Ok((steps, vacant.into_boxed_slice()))
}

fn open_listeners(
    directories: &super::Directories,
    steps: &temper_forge_setup::Steps,
    vacant: &[temper_forge_setup::Group],
    held: Vec<(Fd, std::net::SocketAddr)>,
    inherited: &[(Fd, std::net::SocketAddr)],
    owned: &mut Vec<Fd>,
) -> Result<Vec<(Fd, std::net::SocketAddr)>, String> {
    for (fd, address) in inherited {
        if !steps
            .always
            .iter()
            .chain(steps.asked.iter().chain(vacant).flat_map(|group| group.steps.iter().map(|(_, step)| step)))
            .any(|step| step.listeners().contains(address))
        {
            return Err(format!(
                "{}: inherited listener fd {} at {address}: no step names this listener",
                directories.document().display(),
                fd.raw()
            ));
        }
    }
    let mut listeners = Vec::new();
    let mut owners = Vec::new();
    for (group, step) in steps.always.iter().map(|step| (None, step)).chain(
        steps
            .asked
            .iter()
            .chain(vacant)
            .enumerate()
            .flat_map(|(index, group)| group.steps.iter().map(move |(_, step)| (Some(index), step))),
    ) {
        for addr in step.listeners() {
            if listeners.iter().any(|(_, previous)| previous == addr) {
                if group.is_some() && !owners.contains(&(*addr, group)) && !owners.contains(&(*addr, None)) {
                    owners.push((*addr, group));
                    continue;
                }
                return Err(format!(
                    "{}: port {} ({addr}): two steps name the same listener",
                    directories.document().display(),
                    addr.port()
                ));
            }
            let fd = if let Some((fd, _)) = inherited.iter().chain(&held).find(|(_, address)| address == addr) {
                *fd
            } else {
                let (fd, _) = skein_shell::open_listener(
                    *addr,
                    jig_protocol_config::profile(jig_protocol_config::Profile::Small).io.backlog,
                )
                .map_err(|why| {
                    format!(
                        "{}: port {} ({addr}): cannot listen: {why:?}",
                        directories.document().display(),
                        addr.port()
                    )
                })?;
                owned.push(fd);
                fd
            };
            listeners.push((fd, *addr));
            owners.push((*addr, group));
        }
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

/// The checked setup supplies the identity and paths; the box keeps only opaque
/// public lines on the Forgejo step, never credentials or setup configuration.
fn owned_notices(step: &mut temper_forge_setup::Step, entry: &temper_forge_setup::setup::Entry) {
    let temper_forge_setup::setup::Mode::Owned(owned) = &entry.mode else { return };
    if step.ready() == Some(owned.listen) {
        let label = format!("Forge {} (Forgejo): ", entry.forge);
        let address = owned.listen.to_string();
        let notice = [
            label.as_bytes(),
            b"started\n",
            label.as_bytes(),
            b"api http://",
            address.as_bytes(),
            b"/api/v1\n",
            label.as_bytes(),
            b"person ",
            &owned.person.login,
            b", read header ",
            &entry.directory,
            b"/person_read.header, write header ",
            &entry.directory,
            b"/person_write.header\n",
        ]
        .concat();
        step.set_started_notice(notice.into_boxed_slice());
    }
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
