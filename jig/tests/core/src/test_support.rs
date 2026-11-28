//! Fixtures shared by the core world's binaries (jig/docs/design/domain/testing.md, 9).

pub mod bundle;
pub mod faults;
pub mod first;
pub mod person;
pub mod run_helpers;
pub mod store_peer;
pub mod switch_step;

use jig_domain_contract::{Connector, Up};
use jig_domain_core::{
    self as core,
    engine::{Engine, In, Out, Peer},
};
use skein_lib::{Env, Queue};

/// Answer a connector's ask and collect its continuations alongside direct core requests.
pub fn answer_connector(engine: &mut Engine, env: &Env<core::Limits>, connector: u16, up: Up) -> Queue<core::Request> {
    let mut outputs = Queue::uncapped();
    core::engine::step(engine, env, In::Connector(Connector(connector), up), &mut outputs);
    let mut requests = Queue::uncapped();
    while let Some(output) = outputs.pop() {
        let request = match output {
            Out::Ask(Connector(connector), ask) => core::Request::Ask { connector, ask: core::Ask::Connector(ask) },
            Out::Held(Peer::Connector(connector, released)) => {
                core::Request::Held(Box::new(core::Held::Connector(connector, released)))
            }
            Out::Held(Peer::Store(load)) => core::Request::Held(Box::new(core::Held::Store(load))),
            Out::Core(request) => request,
            Out::Write(write) => core::Request::Write(write),
            Out::Now(_) | Out::Held(Peer::Application(_) | Peer::Resume(_)) => {
                unreachable!("the world drives its own start")
            }
        };
        requests.push(request);
    }
    requests
}
