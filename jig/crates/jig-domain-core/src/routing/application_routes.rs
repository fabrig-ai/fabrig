//! Connector-owned keyed requests and their synchronous party parts
//! (`domain/connectors.md`, section 5.5; `domain/core.md`, section 5).
//! Closed at once (jig/docs/design/domain/people.md, 5.2).

use crate::{
    Ask, Core, Event, Family, Held, Limits, Request, fresh,
    routing::{
        Work,
        party::dispatch::{Party, Routed},
    },
};
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
pub(crate) fn start(
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
    let closed = match scope {
        people::ApplicationScope::Project(project) => !core.open_project(project),
        people::ApplicationScope::Own | people::ApplicationScope::Deployment => false,
    };
    if closed && core.people.person(sign_in, env.now, env.wall).is_some() {
        let mut out = Queue::uncapped();
        out.push(Request::Held(Box::new(Held::PeopleReply {
            to: reply_to,
            reply: people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role { held: None })),
        })));
        if core.connectors.contains(&input.connector) {
            out.push(Request::Ask { connector: input.connector, ask: Ask::DropApplication { owner: input.owner } });
        }
        return first(input.carried, out);
    }
    assert!(core.application_input.is_none(), "one admission at a time");
    core.application_input = Some(input);
    let connector = input.connector;
    let routed = super::party::people::route_people(
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

pub(crate) fn admitted(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    connector: u16,
    scope: people::ApplicationScope,
) {
    let Party { request, person, role, .. } = party;

    let input = core.application_input.take();
    let carried = match input {
        Some(input) => input.carried,
        None => None,
    };
    let owner = match input {
        Some(input) if input.connector == connector && core.connectors.contains(&connector) => input.owner,
        Some(_) | None => {
            if let Some(number) = carried {
                routed.out.push(unused(number));
            }
            routed.work.push(Work::Event(Event::People(people::Event::Decided {
                request,
                outcome: people::Outcome::Refused(people::Refusal::NotOffered),
            })));
            return;
        }
    };
    let Some(number) = fresh(&mut core.counters, Family::Application) else {
        if let Some(number) = carried {
            routed.out.push(unused(number));
        }
        routed.work.push(Work::Event(Event::People(people::Event::Decided {
            request,
            outcome: people::Outcome::Refused(people::Refusal::Limit { which: people::LimitKind::Arithmetic }),
        })));
        return;
    };
    let token = Token::new(number);
    let request = core
        .fly(super::flight::Flight::Application { request, route: Flight { connector, token, part: false, carried } });
    routed.out.push(Request::Ask {
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

pub(crate) fn part(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    request: Token,
    ask: people::Ask,
) -> Queue<Request> {
    let Some((original, flight)) = core.application_mut(request) else { return empty() };
    assert!(!flight.part, "one synchronous core part at a time");
    let connector = flight.connector;
    let original = core.people_request(original).expect("application party route");
    let admitted = people::application_part(&mut core.people, &env.with(env.limits.people), original, ask);
    match admitted {
        Ok(route) => {
            core.application_mut(request).expect("application part owner").1.part = true;
            let mut child = Queue::uncapped();
            child.push(route);
            super::party::people::tag_people_part(core, env, child, work, Some((request, original, connector)))
        }
        Err(outcome) => answer(connector, request, outcome),
    }
}

pub(crate) fn answer(connector: u16, request: Token, outcome: people::Outcome) -> Queue<Request> {
    let mut out = Queue::uncapped();
    out.push(Request::Ask { connector, ask: Ask::ApplicationPartAnswered { request, outcome } });
    out
}

pub(crate) fn finished(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    request: Token,
    outcome: Result<Token, people::Again>,
) -> Queue<Request> {
    let Some((request, flight)) = core.take_application(request) else { return empty() };
    assert!(!flight.part, "core part settled before whole outcome");
    let (outcome, unused) = match outcome {
        Ok(token) => {
            assert!(token == flight.token, "the owner kept the allocated token");
            (people::Outcome::Applied { connector: flight.connector, token }, None)
        }
        Err(why) => (people::Outcome::Again(why), flight.carried),
    };
    first(unused, super::party::people::route_people(core, env, work, people::Event::Decided { request, outcome }))
}

fn empty() -> Queue<Request> {
    Queue::uncapped()
}
