//! The setup owner checks its document before commands read any files.
use jig_protocol_config as config;
use temper_forge_setup::{
    check::{self, Checked},
    problem,
};
use temper_service::startup::Named;

#[derive(Debug)]
pub(crate) struct Deployment {
    pub named: Named,
    pub checked: Checked,
    pub limits: temper_box::limits::Limits,
    pub demo: Option<temper_fake::demo::Demo>,
    pub ports: Box<[std::net::SocketAddr]>,
}

pub(crate) fn check_with(
    bytes: &[u8],
    directories: &config::Directories,
    given: &temper_service::stock::Given,
) -> Result<Deployment, String> {
    let named = temper_service::startup::named_with(bytes, directories, given)
        .map_err(|why| String::from_utf8_lossy(&temper_service::startup::lines(&why)).into_owned())?;
    let document = config::document(
        bytes,
        &config::DocumentLimits { bytes: super::CONFIG_BYTES, tokens: super::CONFIG_BYTES, depth: 32 },
    )
    .expect("named checked document");
    let mut listeners = match &named.clients {
        config::Address::Addr(a) => a.socket().into_iter().collect::<Vec<_>>(),
        config::Address::Host { .. } => Vec::new(),
    };
    if let Some(webhooks) = &named.webhooks
        && let config::Address::Addr(a) = &webhooks.listener
        && let Some(socket) = a.socket()
    {
        listeners.push(socket);
    }
    let application = super::member(&document, 0, b"application");
    let setup_overrides: Vec<_> =
        named.overrides.iter().filter(|(name, _)| !name.starts_with(b"box.")).cloned().collect();
    let checked =
        check::checked(&setup_overrides).map_err(|why| String::from_utf8_lossy(&problem::lines(&why)).into_owned())?;
    let mut problems = temper_service::startup::Problems::empty();
    let mut ports = listeners;
    if let Some(owned) = &named.machine.owned {
        let host = std::str::from_utf8(&owned.api.host).expect("named machine IP");
        let ip = host.parse().expect("named machine IP");
        let listen = std::net::SocketAddr::new(ip, owned.api.port);
        if ports.contains(&listen) {
            return Err("application.machine.owned.mode.owned.listen: must be unique".into());
        }
        ports.push(listen);
        if named.redirects.iter().any(|redirect| redirect.port == listen.port()) {
            return Err(
                "application.machine.owned.mode.owned.listen: must differ from LLM sign-in redirect ports".into()
            );
        }
    }
    let facts = temper_fake::demo::Facts { llm: &named.llm, listeners: &ports, directories };
    let demo = temper_fake::demo::read(&document, super::member(&document, application, b"demo"), &facts)
        .map_err(|why| String::from_utf8_lossy(&why.line()).into_owned())?;
    if let Some(demo) = &demo {
        if named.redirects.iter().any(|redirect| redirect.port == demo.fake().listen.port()) {
            return Err("application.demo.forge.mode.fake.listen: must differ from LLM sign-in redirect ports".into());
        }
        for listen in [demo.fake().listen, demo.llm().listen()] {
            if !ports.contains(&listen) {
                ports.push(listen);
            }
        }
        temper_service::startup::redirect_ports(
            &named.redirects,
            Some(demo.llm().listen().port()),
            temper_service::startup::Listener::Demo,
            &mut problems,
        );
    }
    if !problems.is_empty() {
        return Err(String::from_utf8_lossy(&temper_service::startup::lines(&problems)).into_owned());
    }
    let limits = box_limits(&named.overrides)?;
    Ok(Deployment { named, checked, limits, demo, ports: ports.into_boxed_slice() })
}
pub(crate) fn check(bytes: &[u8], directories: &config::Directories) -> Result<Deployment, String> {
    check_with(bytes, directories, &temper_service::stock::Given::Stock)
}

fn box_limits(overrides: &[(Box<[u8]>, u64)]) -> Result<temper_box::limits::Limits, String> {
    use temper_box::limits::{self, Refused};
    let mut limits = limits::small();
    let mut problems = Vec::new();
    let prefix = "application.limits.overrides.";
    for (name, value) in overrides.iter().filter(|(name, _)| name.starts_with(b"box.")) {
        if let Err(why) = limits::apply(&mut limits, name, *value) {
            let reason = match why {
                Refused::Unknown => "no limit of temper's has this name",
                Refused::Range => "outside its type",
            };
            problems.push(format!("{prefix}{}: {reason}", String::from_utf8_lossy(name)));
        }
    }
    if let Err(why) = limits::check(&limits) {
        let names = why.names();
        let name = String::from_utf8_lossy(names[0]);
        let reason = match names.get(1) {
            Some(other) => format!("must agree with {}", String::from_utf8_lossy(other)),
            None => "past its bound".into(),
        };
        problems.push(format!("{prefix}{name}: {reason}"));
    }
    if problems.is_empty() { Ok(limits) } else { Err(problems.join("\n")) }
}
