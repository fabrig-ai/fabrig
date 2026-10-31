//! Connector-owned keyed requests and their synchronous party parts
//! (`domain/connectors.md`, section 5.5; `domain/core.md`, section 5).

use crate::routing::Work;
use crate::{Ask, Core, Event, Family, Held, Limits, Request, fresh};
use alloc::boxed::Box;
use jig_domain_people as people;
use skein_lib::{Env, Queue, ReplyTo, Token};

#[derive(Debug)]
pub(crate) struct Flight {
    pub connector: u16,
    pub token: Token,
    pub part: bool,
    /// The number of the value the request carried, unused if the
    /// connector cannot decide this time.
    pub carried: Option<u64>,
}

/// The request in admission: its connector, the owner retaining its
/// value, and the number of the value it carried.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Input {
    pub connector: u16,
    pub owner: Token,
    pub carried: Option<u64>,
}

#[expect(clippy::too_many_arguments, reason = "metadata envelope without an application value")]
pub(super) fn start(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    reply_to: ReplyTo,
    sign_in: u64,
    key: [u8; 16],
    scope: people::ApplicationScope,
    digest: [u8; 32],
    input: Input,
) -> Queue<Request> {
    assert!(core.application_input.is_none(), "one admission at a time");
    core.application_input = Some(input);
    let connector = input.connector;
    let routed = super::people_routes::route_people(
        core,
        env,
        work,
        people::Event::Ask {
            models: Box::new([]),
            reply_to,
            sign_in,
            key,
            ask: people::Ask::Application { scope, connector, digest },
        },
    );
    // Still here only when people did not admit it: refused, answered
    // from a retained answer, or joined to the flight of its key.
    let unused = match core.application_input.take() {
        Some(input) => input.carried,
        None => None,
    };
    let mut out = first(unused, routed);
    if core.connectors.contains(&connector) {
        out.push(Request::Ask { connector, ask: Ask::DropApplication { owner: input.owner } });
    }
    out
}

/// The number of a carried value, which is all the core keeps of it.
#[expect(clippy::manual_map, reason = "step code has no closures (programming-model.md, section 10)")]
pub(crate) fn number(carried: Option<crate::Carried>) -> Option<u64> {
    match carried {
        Some(value) => Some(value.number),
        None => None,
    }
}

/// The output naming a carried value the core left unused.
pub(crate) fn unused(number: u64) -> Request {
    Request::Held(Box::new(Held::CarriedUnused { number }))
}

/// Puts the carried value's unused output before every other output, so
/// the clients' component hears of it before the request's answer. The
/// remaining outputs retain their order.
pub(crate) fn first(unused: Option<u64>, requests: Queue<Request>) -> Queue<Request> {
    let Some(number) = unused else { return requests };
    let mut rest = requests;
    let mut out = Queue::uncapped();
    out.push(self::unused(number));
    for _ in 0..rest.len() {
        out.push(rest.pop().expect("route output count"));
    }
    out
}

#[expect(clippy::too_many_arguments, reason = "the admitted party envelope is passed unchanged")]
pub(super) fn admitted(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    out: &mut Queue<Request>,
    request: Token,
    connector: u16,
    person: u64,
    scope: people::ApplicationScope,
    role: Option<people::Role>,
) {
    let input = core.application_input.take();
    let carried = match input {
        Some(input) => input.carried,
        None => None,
    };
    let owner = match input {
        Some(input) if input.connector == connector && core.connectors.contains(&connector) => input.owner,
        Some(_) | None => {
            if let Some(number) = carried {
                out.push(unused(number));
            }
            work.push(Work::Event(Event::People(people::Event::Decided {
                request,
                outcome: people::Outcome::Refused(people::Refusal::NotOffered),
            })));
            return;
        }
    };
    let Some(number) = fresh(&mut core.counters, Family::Application) else {
        if let Some(number) = carried {
            out.push(unused(number));
        }
        work.push(Work::Event(Event::People(people::Event::Decided {
            request,
            outcome: people::Outcome::Refused(people::Refusal::Limit { which: people::LimitKind::Arithmetic }),
        })));
        return;
    };
    let token = Token::new(number);
    assert!(
        core.applications.insert(request, Flight { connector, token, part: false, carried }).is_ok(),
        "one application per people flight"
    );
    out.push(Request::Ask {
        connector,
        ask: Ask::Application {
            request,
            token,
            owner,
            person,
            scope,
            role,
            retention: env.limits.people.request_retention,
        },
    });
}

pub(super) fn part(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    request: Token,
    ask: people::Ask,
) -> Queue<Request> {
    let Some(flight) = core.applications.get_mut(&request) else { return empty() };
    assert!(!flight.part, "one synchronous core part at a time");
    let connector = flight.connector;
    let admitted = people::application_part(
        &mut core.people,
        &Env { now: env.now, wall: env.wall, limits: env.limits.people },
        request,
        ask,
    );
    match admitted {
        Ok(route) => {
            flight.part = true;
            let mut child = Queue::uncapped();
            child.push(route);
            super::people_routes::tag_people(core, env, child, work)
        }
        Err(outcome) => answer(connector, request, outcome),
    }
}

pub(super) fn answer(connector: u16, request: Token, outcome: people::Outcome) -> Queue<Request> {
    let mut out = Queue::uncapped();
    out.push(Request::Ask { connector, ask: Ask::ApplicationPartAnswered { request, outcome } });
    out
}

pub(super) fn finished(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    request: Token,
    outcome: Result<Token, people::Again>,
) -> Queue<Request> {
    let Some(flight) = core.applications.remove(&request) else { return empty() };
    assert!(!flight.part, "core part settled before whole outcome");
    let (outcome, unused) = match outcome {
        Ok(token) => {
            assert!(token == flight.token, "the owner kept the allocated token");
            (people::Outcome::Applied { connector: flight.connector, token }, None)
        }
        Err(why) => (people::Outcome::Again(why), flight.carried),
    };
    first(unused, super::people_routes::route_people(core, env, work, people::Event::Decided { request, outcome }))
}

fn empty() -> Queue<Request> {
    Queue::uncapped()
}
