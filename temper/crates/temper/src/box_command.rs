//! The box's steps and kind slots (temper/docs/design/protocol/setup.md, 7.1).
use skein_io::kernel::{Exit, Fd};
use skein_shell::{Clock, Config, Host, Kernel};
use std::os::unix::ffi::OsStrExt;
use temper_box::host::{Echo, Service, Settings};

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
        let (steps, limits) = configuration_steps(directories, bytes, first)?;
        let listeners = open_listeners(directories, &steps, held, &inherited, &mut owned)?;
        let signals = skein_shell::open_termination_signals().map_err(|errno| format!("signals: errno {errno}"))?;
        owned.push(signals);
        let root = skein_shell::open_root(directory).map_err(|errno| format!("directory: errno {errno}"))?;
        owned.push(root);
        let settings = Settings {
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

fn open_listeners(
    directories: &super::Directories,
    steps: &temper_forge_setup::Steps,
    held: Vec<(Fd, std::net::SocketAddr)>,
    inherited: &[(Fd, std::net::SocketAddr)],
    owned: &mut Vec<Fd>,
) -> Result<Vec<(Fd, std::net::SocketAddr)>, String> {
    for (fd, address) in inherited {
        if !steps
            .always
            .iter()
            .chain(steps.asked.iter().flat_map(|group| group.steps.iter().map(|(_, step)| step)))
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
    for step in steps.always.iter().chain(steps.asked.iter().flat_map(|group| group.steps.iter().map(|(_, step)| step)))
    {
        for addr in step.listeners() {
            if listeners.iter().any(|(_, previous)| previous == addr) {
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
fn owned_notices(steps: &mut temper_forge_setup::Steps, owned: &temper_forge_setup::setup::Owned) {
    let notice = format!(
        "Forgejo: started\nForgejo: api http://{}/api/v1\nForgejo: person {}\n",
        owned.listen,
        String::from_utf8_lossy(&owned.person.login),
    );
    for step in steps.asked.iter_mut().flat_map(|group| group.steps.iter_mut().map(|(_, step)| step)) {
        if step.ready() == Some(owned.listen) {
            step.set_started_notice(notice.as_bytes().into());
        }
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

/// Reads the machine separately from the deployment's forge records (temper/docs/design/protocol/configuration.md, 3).
pub(crate) fn machine(
    bytes: &[u8],
    directories: &super::Directories,
) -> Result<temper_forge_setup::machine::Machine, String> {
    use jig_protocol_config as config;
    use temper_forge_setup::{machine, problem};
    let document = config::document(
        bytes,
        &config::DocumentLimits { bytes: super::CONFIG_BYTES, tokens: super::CONFIG_BYTES, depth: 32 },
    )
    .map_err(|why| format!("{why:?}"))?;
    let Some(application) =
        config::read::Object::new(&document, 0).map_err(|why| format!("{why:?}"))?.member(&document, b"application")
    else {
        return Err("application.machine: missing member".into());
    };
    let Some(at) = config::read::Object::new(&document, application)
        .map_err(|why| format!("{why:?}"))?
        .member(&document, b"machine")
    else {
        return Err("application.machine: missing member".into());
    };
    let mut problems = problem::Problems::empty();
    let machine = machine::read(&document, at, &temper_forge_setup::limits::small(), &directories.jig(), &mut problems);
    if !problems.is_empty() {
        return Err(String::from_utf8_lossy(&problem::lines(&problems)).into_owned());
    }
    machine.ok_or("application.machine: invalid form".into())
}

/// Builds the box's always steps and kind slots from its checked document (7.1).
pub fn configuration_steps(
    directories: &super::Directories,
    bytes: &[u8],
    first: Box<[temper_forge_setup::Step]>,
) -> Result<(temper_forge_setup::Steps, temper_box::limits::Limits), String> {
    let super::setup::Deployment { named, demo, limits, .. } = super::setup::check(bytes, &directories.jig())?;
    let mut serve = vec![super::address(&named.clients)?];
    if let Some(webhooks) = &named.webhooks {
        serve.push(super::address(&webhooks.listener)?);
    }
    let machine = machine(bytes, directories)?;
    let steps = {
        let machine = &machine;
        let mut steps = temper_forge_setup::step::kinds(
            machine,
            &directories.flags(),
            serve.into_boxed_slice(),
            cfg!(target_os = "linux"),
        );
        if let Some(fake) = &machine.fake {
            let demo = demo.as_ref().ok_or("application.machine.fake requires application.demo.llm")?;
            let mut asked = steps.asked.into_vec();
            asked.push(temper_fake::steps::fake_slot(fake, demo.llm().listen(), &directories.flags()));
            steps.asked = asked.into_boxed_slice();
        }
        let mut always = first.into_vec();
        always.extend(steps.always);
        for group in &mut steps.asked {
            group.before = u32::try_from(always.len().saturating_sub(1)).expect("bounded first steps");
        }
        steps.always = always.into_boxed_slice();
        if machine.owned.is_some() {
            let entry = temper_forge_setup::machine::entry(
                machine,
                0,
                temper_forge_setup::machine::BoxKind::Owned,
                &directories.jig(),
            )
            .expect("owned machine");
            let temper_forge_setup::setup::Mode::Owned(owned) = &entry.mode else { unreachable!("owned kind") };
            owned_notices(&mut steps, owned);
        }
        steps
    };
    Ok((steps, limits))
}
