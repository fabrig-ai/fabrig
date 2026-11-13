//! System configuration shared with the startup (protocol/services.md, 4.2).
use crate::settings;
use alloc::boxed::Box;

#[must_use]
pub fn playground_config(
    demo: Option<&settings::Demo>,
    welcome: bool,
    deadline: skein_lib::Duration,
) -> Option<temper_engine_domain_forge::PlaygroundConfig> {
    if !welcome {
        return None;
    }
    match demo {
        Some(demo) => Some(temper_engine_domain_forge::PlaygroundConfig { provider: demo.provider, deadline }),
        None => None,
    }
}

/// github.com's remotes' base (temper/docs/design/protocol/configuration/engine.md).
pub const GITHUB_REMOTE: &[u8] = b"https://github.com";

/// Machine places passed to the root (temper/docs/design/protocol/configuration/engine.md).
#[must_use]
pub fn machine_config(machine: &temper_forge_setup::machine::Machine) -> temper_engine_domain_forge::Machine {
    use temper_engine_domain_forge::{Address, Machine, Place};
    let github = match &machine.github {
        Some(github) => {
            let (host, port) = match &github.api {
                jig_protocol_config::Address::Host { name, port } => (name.clone(), *port),
                jig_protocol_config::Address::Addr(skein_io::Addr::Ip(address)) => {
                    (address_host(*address), address.port())
                }
                jig_protocol_config::Address::Addr(skein_io::Addr::InProcess(_)) => {
                    unreachable!("checked machine socket")
                }
            };
            Place {
                api: Address { tls: if github.tls { Some(host.clone()) } else { None }, host, port },
                git: github.remote.clone(),
                web: Some(github.web.clone()),
            }
        }
        None => Place {
            api: Address {
                host: skein_lib::bytes::copy_of(b"api.github.com"),
                port: 443,
                tls: Some(skein_lib::bytes::copy_of(b"api.github.com")),
            },
            git: skein_lib::bytes::copy_of(GITHUB_REMOTE),
            web: Some(skein_lib::bytes::copy_of(b"https://github.com")),
        },
    };
    Machine {
        owned_person: match &machine.owned {
            Some(owned) => owned.person.login.clone(),
            None => Box::new([]),
        },
        owned: match &machine.owned {
            Some(owned) => Some(local_place(owned.listen, &owned.remote)),
            None => None,
        },
        fake: match &machine.fake {
            Some(fake) => Some(local_place(fake.listen, &fake.remote)),
            None => None,
        },
        github,
        github_git_program: match &machine.github {
            Some(github) => github.git.clone(),
            None => None,
        },
    }
}
/// Render a checked IP address for system configuration and the component.
#[expect(clippy::disallowed_macros, reason = "startup renders a checked IP address for the domain place")]
#[must_use]
pub fn address_host(address: core::net::SocketAddr) -> Box<[u8]> {
    alloc::format!("{}", address.ip()).into_bytes().into_boxed_slice()
}
fn local_place(address: core::net::SocketAddr, remote: &[u8]) -> temper_engine_domain_forge::Place {
    temper_engine_domain_forge::Place {
        api: temper_engine_domain_forge::Address { host: address_host(address), port: address.port(), tls: None },
        git: skein_lib::bytes::copy_of(remote),
        web: None,
    }
}
