//! A list through the real worker and hosted agent, without run accounting
//! (jig/docs/design/domain/engine.md, 17.12).
use crate::{
    referee::{Store, client::Received},
    world::{HarnessProcess, Proc},
};
use jig_domain_core as core;
use skein_fake_peers::llm::Observation;
use std::cell::RefCell;
use temper_engine_domain as root;

#[derive(Debug)]
pub struct Outcome {
    pub requests: Vec<Option<u32>>,
    pub agents: usize,
    pub queries: usize,
    pub new_in_me: bool,
    pub accounting_unchanged: bool,
}

#[derive(Debug, PartialEq, Eq)]
struct Accounting {
    attempts: u64,
    tasks: Vec<jig_domain_tasks::Stored>,
}
thread_local! {
    static BEFORE: RefCell<Option<Accounting>> = const { RefCell::new(None) };
}
fn accounting(store: &Store) -> Accounting {
    let mut attempts = 0;
    let mut tasks = Vec::new();
    for row in store.rows.values() {
        match row {
            root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(deployment))) => {
                attempts = deployment.runs;
            }
            root::Record::Core(core::Record::Tasks(row)) => tasks.push(row.clone()),
            _ => {}
        }
    }
    Accounting { attempts, tasks }
}
pub(crate) fn before(store: &Store) {
    BEFORE.with(|before| {
        assert!(before.borrow().is_none(), "accounting captured once before adding the key");
        *before.borrow_mut() = Some(accounting(store));
    });
}
pub(crate) fn outcome(procs: &[Proc], store: &Store) -> Outcome {
    let mut result =
        Outcome { requests: Vec::new(), agents: 0, queries: 0, new_in_me: false, accounting_unchanged: false };
    for proc in procs {
        match proc {
            Proc::List(_) => result.agents += 1,
            Proc::Peer(peer) => {
                for observation in peer.observations() {
                    match observation {
                        Observation::List { index, .. } => result.requests.push(*index),
                        Observation::Query { .. } => result.queries += 1,
                        _ => {}
                    }
                }
            }
            Proc::Other(HarnessProcess::Client(person)) => {
                result.new_in_me |= person.wire.received.iter().any(|received| {
                    matches!(received, Received::Delivery(jig_client::contract::Delivery::Me(me))
                        if me.projects.iter().any(|project| project.models.iter().any(|model|
                            model.name.bytes() == b"gpt-future" && !model.picker)))
                });
            }
            _ => {}
        }
    }
    let after = accounting(store);
    result.accounting_unchanged =
        BEFORE.with(|before| before.borrow_mut().take().is_some_and(|before| before.attempts == 0 && before == after));
    result
}

// Runs map the channel's endpoint 1 to backend 0. A list speaks the runtime's
// endpoint names directly, so this fixture gives it the engine's endpoint 1.
pub(crate) fn agent(inherited: &skein_world::Inherited) -> Proc {
    let config = temper_playground::documents::agent_configuration();
    let (destinations, _) = config.llm_endpoints.into_parts();
    let destination = destinations.into_boxed().into_vec().pop().expect("one fake endpoint");
    let endpoints = smith_protocol_llm::Endpoints::new(
        Box::new([smith_protocol_llm::ConfiguredEndpoint {
            name: smith_domain::llm::Endpoint(1),
            destination,
            connect: skein_lib::Duration::from_secs(1),
            handshake: skein_lib::Duration::from_secs(1),
            oversized_reasoning: smith_protocol_llm::OversizedReasoning::Fail,
        }]),
        1,
    )
    .expect("the engine's endpoint");
    let pipe = |number| inherited.pipes.iter().find(|(child, _)| *child == number).expect("list pipe").1;
    Proc::List(Box::new(smith_agent_shell::list::Process::with_endpoints(
        endpoints,
        smith_agent_shell::list::Faces {
            stdin: pipe(0),
            stdout: pipe(1),
            stderr: Some(pipe(2)),
            signal: inherited.signal,
        },
    )))
}
