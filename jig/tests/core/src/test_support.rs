//! Fixtures shared by the core world's binaries (jig/docs/design/domain/testing.md, 9).

pub mod bundle;
pub mod faults;
pub mod first;
pub mod person;
pub mod run_helpers;
pub mod store_peer;
pub mod switch_step;

// Direct core fixtures route connector outputs through the same bridge as their root.
use jig_domain_contract::Connector;
use jig_domain_core as core;
use skein_lib::{Env, Queue};

#[must_use]
pub fn bridge(request: core::Request) -> core::Request {
    match request {
        core::Request::Ask { connector, ask: core::Ask::Connector(ask) } => {
            core::engine::bridge_ask(Connector(connector), ask)
        }
        core::Request::Held(held) => match *held {
            core::Held::Connector(connector, released) => core::engine::bridge_released(connector, released),
            _ => core::Request::Held(held),
        },
        other => other,
    }
}

pub fn bridged_step(domain: &mut core::Core, env: &Env<core::Limits>, event: core::Event) -> Queue<core::Request> {
    let mut native = core::step(domain, env, event);
    let mut out = Queue::uncapped();
    while let Some(request) = native.pop() {
        out.push(bridge(request));
    }
    out
}
