//! Shared wait wording (temper/docs/design/client/pages/waits.md, 2).
use super::fixtures as f;
use crate::words::Times;
use jig_client::contract::{
    self as c, AtBound, EscalationHolder as Holder, Unbounded, WaitKind as Kind, WaitObject as Object, waits::Bound,
};
use skein_lib::{List, Wall};
use temper_client_protocol_wire::{FORGE, waits};

const TASK: u64 = 833;
const ANSWERER: u64 = 812;
const QUESTION: u64 = 3;
const READER: u64 = 73;
const SAM: u64 = 4;
const ROLE: u32 = 7;
const PROJECT: u32 = 2;
const WORKER: u32 = 9;
const OUTBOX_ENTRY: u64 = 51;
const PULL: u64 = 33;
const RULE: u64 = 2;
const UNKNOWN_CODE: u32 = 999;
const SUBSCRIPTIONS: [u64; 2] = [5, 6];
const MINUTE: u64 = 60_000_000_000;
const SINCE: Wall = Wall::EPOCH;
const NOW: Wall = Wall::from_nanos(120 * MINUTE);
const TIMER: Wall = Wall::from_nanos(150 * MINUTE);
const NO_DEADLINE: Bound = Bound::None(Unbounded::NoDeadline);

fn wait(kind: Kind, object: Object, bound: Bound, hears: Option<Holder>) -> c::Wait {
    c::Wait { task: TASK, kind, object, since: SINCE, bound, hears }
}
fn question(at: Holder) -> Object {
    Object::Entry { kind: c::WaitedFor::Question, number: QUESTION, at }
}
fn member() -> c::ProjectMember {
    const NAME: &[u8] = b"Sam";
    let name = c::DocumentText::complete(c::Text::new(NAME.into(), u32::try_from(NAME.len()).unwrap()).unwrap());
    c::ProjectMember { party: SAM, kind: c::PartyKind::Person, role: c::HeldRole::Member, login: name.clone(), name }
}

#[test]
fn a_wait_names_what_each_kind_waits_for() {
    let words = f::words();
    let members = [member()];
    let path = List::from_boxed(Box::new([Box::from(b"slot".as_slice()), Box::from(b"one".as_slice())]));
    let resource = c::ResourceName { connector: FORGE, path };
    for (kind, object, expected) in [
        (Kind::Party, question(Holder::Person(SAM)), "question 3 at Sam"),
        (Kind::Party, question(Holder::Person(READER)), "question 3 at you"),
        (Kind::Party, question(Holder::Person(99)), "question 3 at person 99"),
        (Kind::Party, question(Holder::Role { project: PROJECT, role: ROLE }), "question 3 at role 7"),
        (Kind::Party, question(Holder::Task(ANSWERER)), "question 3 at task 812"),
        (Kind::Question, Object::Question { number: QUESTION, answerer: ANSWERER }, "question 3 to task 812"),
        (Kind::Dependency, Object::Task(ANSWERER), "task 812"),
        (
            Kind::Resource,
            Object::Resource { name: resource.clone(), holder: Some(ANSWERER) },
            "resource slot/one held by task 812",
        ),
        (Kind::Resource, Object::Resource { name: resource, holder: None }, "resource slot/one"),
        (
            Kind::Account,
            Object::Run(c::TaskRunWait::Account { account: None, source: None, why: c::TaskWaitWhy::Refreshing }),
            "Refreshing account credentials",
        ),
        (Kind::Account, Object::Run(c::TaskRunWait::Credential), "This work needs a credential from its forge."),
        (
            Kind::Host,
            Object::Run(c::TaskRunWait::Worker { worker: WORKER, until: TIMER }),
            "Waiting for worker 9 to return, until 02:30",
        ),
        (Kind::Delegate, Object::Task(ANSWERER), "task 812"),
        (
            Kind::News,
            Object::News { subscriptions: List::from_boxed(Box::new(SUBSCRIPTIONS)), timer: None },
            "news: subscriptions 5, 6",
        ),
        (Kind::News, Object::News { subscriptions: List::uncapped(), timer: Some(TIMER) }, "a timer at 02:30"),
        (Kind::News, Object::News { subscriptions: List::uncapped(), timer: None }, "nothing armed"),
        (
            Kind::Step,
            Object::Procedure {
                step: waits::CHANGE_CHECKING,
                things: List::from_boxed(Box::new([c::Thing { what: waits::THING_PULL, number: PULL, owner: None }])),
            },
            "checking: pull request #33",
        ),
        (
            Kind::Room,
            Object::Procedure {
                step: waits::LANDING_ROOM,
                things: List::from_boxed(Box::new([c::Thing {
                    what: waits::THING_ENTRY,
                    number: OUTBOX_ENTRY,
                    owner: Some(ANSWERER),
                }])),
            },
            "room: entry 51 of task 812",
        ),
        (Kind::Turn, Object::Task(ANSWERER), "task 812"),
        (
            Kind::SignOff,
            Object::Procedure {
                step: waits::CHANGE_READY,
                things: List::from_boxed(Box::new([
                    c::Thing { what: waits::THING_RULE, number: RULE, owner: None },
                    c::Thing { what: waits::THING_ROLE, number: u64::from(ROLE), owner: None },
                ])),
            },
            "ready: rule 2, role 7",
        ),
        (
            Kind::Step,
            Object::Procedure {
                step: UNKNOWN_CODE,
                things: List::from_boxed(Box::new([c::Thing {
                    what: UNKNOWN_CODE,
                    number: OUTBOX_ENTRY,
                    owner: None,
                }])),
            },
            "step 999: thing 999 51",
        ),
    ] {
        let wait = wait(kind, object, NO_DEADLINE, None);
        let said = words.wait(&wait, &mut Times::new(NOW), Some(READER), &members, false, false);
        assert_eq!(said.strip_suffix(" · waiting 2 h · no deadline".as_bytes()), Some(expected.as_bytes()));
    }
}

#[test]
fn a_wait_says_its_age_and_own_bound() {
    const SOON: Wall = Wall::from_nanos(NOW.as_nanos() + 20 * MINUTE);
    const LATER: Wall = Wall::from_nanos(NOW.as_nanos() + 240 * MINUTE);
    const IN_TWO_HOURS: Wall = Wall::from_nanos(NOW.as_nanos() + 120 * MINUTE);
    const IN_FIVE_MINUTES: Wall = Wall::from_nanos(NOW.as_nanos() + 5 * MINUTE);
    const NEXT_MINUTE: Wall = Wall::from_nanos(NOW.as_nanos() + MINUTE);
    const JUST_AFTER_NOW: Wall = Wall::from_nanos(NOW.as_nanos() + 1);
    let words = f::words();
    for (bound, expected, next) in [
        (Bound::At { at: SOON, then: AtBound::PassUp }, "passes up in 20 min", JUST_AFTER_NOW),
        (Bound::At { at: LATER, then: AtBound::Hold }, "holds in 4 h", JUST_AFTER_NOW),
        (Bound::At { at: IN_TWO_HOURS, then: AtBound::Overdue }, "is overdue in 2 h", JUST_AFTER_NOW),
        (Bound::At { at: IN_FIVE_MINUTES, then: AtBound::GiveUp }, "gives up in 5 min", JUST_AFTER_NOW),
        (Bound::At { at: SINCE, then: AtBound::Overdue }, "overdue for 2 h", NEXT_MINUTE),
        (Bound::At { at: SINCE, then: AtBound::PassUp }, "due now", NEXT_MINUTE),
        (Bound::At { at: SINCE, then: AtBound::Hold }, "due now", NEXT_MINUTE),
        (Bound::At { at: SINCE, then: AtBound::GiveUp }, "due now", NEXT_MINUTE),
        (Bound::At { at: NOW, then: AtBound::PassUp }, "due now", NEXT_MINUTE),
        (Bound::At { at: NOW, then: AtBound::Overdue }, "overdue for under 1 min", NEXT_MINUTE),
        (NO_DEADLINE, "no deadline", NEXT_MINUTE),
        (Bound::None(Unbounded::Standing), "standing", NEXT_MINUTE),
        (Bound::None(Unbounded::Behind(ANSWERER)), "behind task 812", NEXT_MINUTE),
        (Bound::None(Unbounded::OperatorHold), "an operator's hold", NEXT_MINUTE),
    ] {
        let wait = wait(Kind::Dependency, Object::Task(ANSWERER), bound, None);
        let mut times = Times::new(NOW);
        let said = words.wait(&wait, &mut times, None, &[], false, false);
        assert_eq!(said.strip_prefix("task 812 · waiting 2 h · ".as_bytes()), Some(expected.as_bytes()));
        assert_eq!(times.next, Some(next));
    }
}

#[test]
fn a_wait_includes_its_hearer_only_where_asked() {
    let words = f::words();
    let members = [member()];
    for (hears, include, expected) in [
        (Some(Holder::Person(READER)), true, " · then you hear"),
        (Some(Holder::Person(SAM)), true, " · then Sam hears"),
        (Some(Holder::Role { project: PROJECT, role: ROLE }), true, " · then role 7 hears"),
        (Some(Holder::Task(ANSWERER)), true, " · then task 812 hears"),
        (Some(Holder::Task(ANSWERER)), false, ""),
        (None, true, ""),
    ] {
        let wait = wait(Kind::Party, question(Holder::Person(SAM)), NO_DEADLINE, hears);
        let said = words.wait(&wait, &mut Times::new(NOW), Some(READER), &members, false, include);
        assert_eq!(
            said.strip_prefix("question 3 at Sam · waiting 2 h · no deadline".as_bytes()),
            Some(expected.as_bytes())
        );
    }
}
