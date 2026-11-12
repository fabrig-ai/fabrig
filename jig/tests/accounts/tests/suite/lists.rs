//! The accounts child's list reads (jig/docs/design/domain/accounts.md, 3.7).
use jig_accounts_world::{
    ALICE, ALICE_PARTY, BOB, LIMITS, PROVIDER, World, account, issued, program, provider, providers, source,
};
use jig_domain_accounts::{
    Answer, Change, Event, Failure, FromComponent, Holder, Kind, ListRecord, ModelKind, ProviderList, ReadFailure,
    Request, Signing, Stored,
};
use jig_domain_link::{Form, Scheme};
use skein_lib::{Duration, Token, Wall};

const ACCOUNT: u32 = 10;
const SOURCE: u32 = 11;
const SECOND: u32 = 12;
const OTHER_ACCOUNT: u32 = 20;
const OTHER_SOURCE: u32 = 21;
const DEPLOYMENT_ACCOUNT: u32 = 30;
const DEPLOYMENT_SOURCE: u32 = 31;
const OTHER_PROVIDER: u32 = 2;
const REQUEST: Token = Token::new(50);
// accounts.md, 3.7: a day between attempts, including across starts.
const DAY: u64 = 86_400;
const HOUR: u64 = 3_600;
const VALID: Duration = Duration::from_secs(DAY * 3);
const CARRIED: u64 = 1 << 63;

fn wall(seconds: u64) -> Wall {
    Wall::from_nanos(Duration::from_secs(seconds).as_nanos())
}
fn record(
    read: Option<Wall>,
    tried: Option<Wall>,
    names: u64,
    failure: Option<ReadFailure>,
    left_out: u32,
) -> ListRecord {
    ListRecord { names, read, tried, failure, left_out }
}
fn row(number: u32, holder: Holder, list: Option<ListRecord>) -> Stored {
    let mut row = account(number, PROVIDER, holder, b"subject");
    if let Stored::Account { list: at, .. } = &mut row {
        *at = list;
    }
    row
}
fn recent() -> ListRecord {
    record(Some(Wall::EPOCH), Some(Wall::EPOCH), 1, None, 0)
}
fn ordered_source(kind: Kind) -> Vec<Stored> {
    if kind == Kind::Program {
        let mut reservation = issued(SOURCE, b"cli");
        if let Stored::Issued { holder, .. } = &mut reservation {
            *holder = ALICE;
        }
        vec![reservation, program(ACCOUNT, SOURCE, 0, b"cli")]
    } else {
        vec![source(ACCOUNT, SOURCE, 0, kind)]
    }
}
fn world(rows: &[Stored], configured: bool) -> World {
    let mut p = provider(PROVIDER, true, true);
    p.signed_in_plan = false;
    p.offers.programs = vec![b"cli".as_slice().into()].into();
    if configured {
        p.list = Some(ProviderList { keep: vec![b"*".as_slice().into()].into(), drop: Box::new([]) });
    }
    World::with_start(LIMITS, providers(vec![p, provider(OTHER_PROVIDER, false, true)]), rows)
}
fn started(kind: Kind, list: Option<ListRecord>) -> World {
    let mut world = world(&[row(ACCOUNT, ALICE, list), source(ACCOUNT, SOURCE, 0, kind)], true);
    world.step(Event::Want);
    world
}
fn change(holder: Holder, change: Change) -> Event {
    Event::Change { request: REQUEST, holder, change }
}
fn grant(source: u32) -> Event {
    Event::Component(FromComponent::Granted {
        account: source,
        generation: 7,
        valid: VALID,
        form: Form::Credential(Scheme::Bearer),
    })
}
fn reads(outputs: &[Request]) -> Vec<Request> {
    outputs.iter().filter(|r| matches!(r, Request::ReadList { .. })).cloned().collect()
}
fn read_number(outputs: &[Request]) -> u64 {
    outputs
        .iter()
        .find_map(|r| match r {
            Request::ReadList { read, .. } => Some(*read),
            _ => None,
        })
        .expect("one read asked")
}
fn again(holder: Holder) -> Event {
    change(holder, Change::ReadLists { provider: PROVIDER })
}
fn reading(count: u32) -> Request {
    Request::Answer { request: REQUEST, answer: Answer::Reading { count } }
}
fn listed(world: &mut World, read: u64) -> Vec<Request> {
    world.step(Event::Listed { read, names: 5, left_out: 2, at: world.wall })
}

#[test]
fn the_provider_and_first_source_decide_a_list() {
    for (label, configured, first, after, expected) in [
        ("key", true, Kind::Key, None, true),
        ("sign-in", true, Kind::SignIn, None, true),
        ("program before key", true, Kind::Program, Some(Kind::Key), false),
        ("no provider list", false, Kind::Key, None, false),
        ("none source", true, Kind::None, None, false),
    ] {
        let mut rows = vec![row(ACCOUNT, ALICE, None)];
        rows.extend(ordered_source(first));
        if let Some(kind) = after {
            rows.push(source(ACCOUNT, SECOND, 1, kind));
        }
        let mut world = world(&rows, configured);
        world.step(Event::Want);
        assert_eq!(world.domain.has_list(ACCOUNT), expected, "{label}");
    }
}

#[test]
fn new_value_sources_ask_with_their_first_grant() {
    for kind in [Kind::Key, Kind::SignIn] {
        let mut world = world(&[], true);
        world.step(Event::Want);
        let before = match kind {
            Kind::Key => world.step(change(ALICE, Change::AddKey { provider: PROVIDER, join: None, carried: CARRIED })),
            Kind::SignIn => {
                let out =
                    world.step(Event::SignIn { request: REQUEST, holder: ALICE, by: ALICE_PARTY, provider: PROVIDER });
                let source = out
                    .iter()
                    .find_map(|r| match r {
                        Request::Signing(Signing::SignIn { source, .. }) => Some(*source),
                        _ => None,
                    })
                    .expect("sign-in started");
                world.step(Event::SignedIn {
                    said: jig_domain_accounts::Said { email: None, name: None },
                    source,
                    subject: b"new".as_slice().into(),
                    display: Box::new([]),
                    declined: false,
                })
            }
            Kind::Program | Kind::None => unreachable!(),
        };
        assert!(reads(&before).is_empty(), "{kind:?} waits for its grant");
        let number = match kind {
            Kind::Key => before
                .iter()
                .find_map(|r| match r {
                    Request::Signing(Signing::Keep { source, .. }) => Some(*source),
                    _ => None,
                })
                .expect("key being kept"),
            Kind::SignIn => {
                *world
                    .domain
                    .accounts_by_number()
                    .first()
                    .expect("new account")
                    .1
                    .sources()
                    .first()
                    .expect("first source")
                    .1
            }
            Kind::Program | Kind::None => unreachable!(),
        };
        let out = world.step(grant(number));
        let account = world.domain.source(number).expect("source").account;
        assert_eq!(
            reads(&out),
            vec![Request::ReadList {
                account,
                read: 0,
                provider: PROVIDER,
                grant: world.domain.source_grant(number, world.now).expect("source grant")
            }],
            "{kind:?}"
        );
        assert!(reads(&world.step(grant(number))).is_empty(), "an in-flight read is held once");
    }
}

#[test]
fn usability_returns_ask_even_after_a_recent_attempt() {
    let refused = Event::Refused { account: SOURCE, code: Box::new([]), reason: Box::new([]) };
    for (label, kind, hold, lift) in [
        (
            "needs sign-in",
            Kind::SignIn,
            vec![Event::Component(FromComponent::Failed { account: SOURCE, failure: Failure::SignInAgain })],
            vec![grant(SOURCE)],
        ),
        (
            "limited",
            Kind::Key,
            vec![Event::Limited { account: SOURCE, reset: None }],
            vec![change(ALICE, Change::Resume { source: SOURCE })],
        ),
        (
            "refused",
            Kind::Key,
            vec![refused],
            vec![change(ALICE, Change::ReplaceKey { source: SOURCE, carried: CARRIED }), grant(SOURCE)],
        ),
        (
            "moving",
            Kind::SignIn,
            vec![change(ALICE, Change::MoveOut { account: ACCOUNT })],
            vec![Event::Withdrawn { source: SOURCE, readable: false }, grant(SOURCE)],
        ),
    ] {
        let mut world = started(kind, Some(recent()));
        world.step(grant(SOURCE));
        for event in hold {
            assert!(reads(&world.step(event)).is_empty(), "{label} holds reads");
        }
        assert_eq!(world.step(again(ALICE)), vec![reading(0)], "{label} isn't usable");
        let outputs: Vec<_> = lift.into_iter().flat_map(|event| world.step(event)).collect();
        assert_eq!(reads(&outputs).len(), 1, "{label} lifted");
    }
}

#[test]
fn each_attempt_rearms_one_day_and_waits_for_usability() {
    for (label, failure, held, removed) in [
        ("good", None, false, false),
        ("failed", Some(ReadFailure::Deadline), false, false),
        ("held good", None, true, false),
        ("held failure", Some(ReadFailure::Unreachable), true, false),
        ("removed good", None, false, true),
        ("removed failure", Some(ReadFailure::Deadline), false, true),
    ] {
        let mut world = started(Kind::Key, None);
        let out = world.step(grant(SOURCE));
        let read = read_number(&out);
        if let Some(failure) = failure {
            world.step(Event::ListFailed { read, failure, at: world.wall });
        } else {
            listed(&mut world, read);
        }
        if removed {
            world.step(change(ALICE, Change::Remove { account: ACCOUNT }));
            assert!(world.domain.next_deadline().is_none(), "{label} leaves no alarm");
        }
        assert!(world.fire_at(DAY - 1).is_empty(), "{label} before a day");
        if held {
            world.step(Event::Limited { account: SOURCE, reset: None });
        }
        let due = world.fire_at(DAY);
        assert_eq!(reads(&due).len(), usize::from(!held && !removed), "{label} at a day");
        assert!(!due.iter().any(|r| matches!(r, Request::Save { .. })), "an alarm stores nothing");
        assert!(world.fire_at(DAY).is_empty(), "{label} fires once");
        if held {
            let out = world.step(change(ALICE, Change::Resume { source: SOURCE }));
            assert_eq!(reads(&out).len(), 1, "{label} reads when usable");
        }
    }
}

#[test]
fn restored_attempts_use_the_wall_at_want() {
    for (label, tried, age, initially, delay) in [
        ("day old", Some(Wall::EPOCH), DAY, 1, None),
        ("hour old", Some(Wall::EPOCH), HOUR, 0, Some(DAY - HOUR)),
        ("no attempt", None, HOUR, 1, None),
    ] {
        let saved = record(None, tried, 0, Some(ReadFailure::Agent), 0);
        let mut world = world(&[row(ACCOUNT, ALICE, Some(saved)), source(ACCOUNT, SOURCE, 0, Kind::Key)], true);
        world.wall = wall(age);
        assert!(reads(&world.step(Event::Want)).is_empty(), "{label} waits for restored grant");
        assert_eq!(reads(&world.step(grant(SOURCE))).len(), initially, "{label} restoration isn't becoming usable");
        if let Some(delay) = delay {
            assert!(world.fire_at(delay - 1).is_empty(), "{label} before remainder");
            assert_eq!(reads(&world.fire_at(delay)).len(), 1, "{label} after remainder");
        }
    }
}

#[test]
fn read_again_counts_usable_accounts_of_the_admitted_holder() {
    for (label, holder, grant_own, grant_other, grant_deployment, configured, provider_number, expected) in [
        ("own and no others", ALICE, true, false, true, true, PROVIDER, 1),
        ("every own account", ALICE, true, true, true, true, PROVIDER, 2),
        ("owner deployment act", Holder::Deployment, true, true, true, true, PROVIDER, 1),
        ("member cannot read another", BOB, true, true, true, true, PROVIDER, 0),
        ("none usable", ALICE, false, false, true, true, PROVIDER, 0),
        ("no list", ALICE, true, true, true, false, PROVIDER, 0),
        ("different provider", ALICE, true, true, true, true, OTHER_PROVIDER, 0),
    ] {
        let mut world = world(
            &[
                row(ACCOUNT, ALICE, Some(recent())),
                source(ACCOUNT, SOURCE, 0, Kind::Key),
                row(OTHER_ACCOUNT, ALICE, Some(recent())),
                source(OTHER_ACCOUNT, OTHER_SOURCE, 0, Kind::SignIn),
                row(DEPLOYMENT_ACCOUNT, Holder::Deployment, Some(recent())),
                source(DEPLOYMENT_ACCOUNT, DEPLOYMENT_SOURCE, 0, Kind::Key),
            ],
            configured,
        );
        world.step(Event::Want);
        if grant_own {
            world.step(grant(SOURCE));
        }
        if grant_other {
            world.step(grant(OTHER_SOURCE));
        }
        if grant_deployment {
            world.step(grant(DEPLOYMENT_SOURCE));
        }
        let event = change(holder, Change::ReadLists { provider: provider_number });
        let out = world.step(event.clone());
        assert_eq!(out.last(), Some(&reading(expected)), "{label}");
        assert_eq!(reads(&out).len(), usize::try_from(expected).expect("small count"), "{label}");
        assert_eq!(world.step(event), vec![reading(expected)], "{label} counts an in-flight read without repeating it");
    }
}

#[test]
fn answers_keep_good_names_and_ignore_unheld_reads() {
    let old = record(Some(wall(1)), Some(wall(2)), 9, Some(ReadFailure::Agent), 3);
    for (label, before, failure, expected) in [
        ("listed", Some(old), None, record(Some(wall(3)), Some(wall(3)), 5, None, 2)),
        (
            "refused",
            Some(old),
            Some(ReadFailure::Refused { status: 403 }),
            record(old.read, Some(wall(3)), 9, Some(ReadFailure::Refused { status: 403 }), 3),
        ),
        ("room", Some(old), Some(ReadFailure::Room), record(old.read, Some(wall(3)), 9, Some(ReadFailure::Room), 3)),
        (
            "first failure",
            None,
            Some(ReadFailure::Unreachable),
            record(None, Some(wall(3)), 0, Some(ReadFailure::Unreachable), 0),
        ),
    ] {
        let mut world = started(Kind::Key, before);
        let mut out = world.step(grant(SOURCE));
        out.extend(world.step(again(ALICE)));
        let read = read_number(&out);
        world.wall = wall(3);
        let event = match failure {
            Some(failure) => Event::ListFailed { read, failure, at: world.wall },
            None => Event::Listed { read, names: 5, left_out: 2, at: world.wall },
        };
        let out = world.step(event.clone());
        assert_eq!(world.domain.list(ACCOUNT), Some(&expected), "{label}");
        assert_eq!(
            out,
            vec![Request::Save { record: row(ACCOUNT, ALICE, Some(expected)) }],
            "{label} saves the account alone"
        );
        assert!(world.step(event).is_empty(), "{label} duplicate isn't held");
    }
    for remove in [false, true] {
        let mut world = started(Kind::Key, None);
        let out = world.step(grant(SOURCE));
        let read = read_number(&out);
        if remove {
            world.step(change(ALICE, Change::Remove { account: ACCOUNT }));
        }
        for read in [read + 1, if remove { read } else { read + 2 }] {
            assert_eq!(world.step(Event::Listed { read, names: 1, left_out: 0, at: world.wall }).len(), 0);
            assert_eq!(world.step(Event::ListFailed { read, failure: ReadFailure::Agent, at: world.wall }).len(), 0);
        }
        if remove {
            assert!(world.fire_at(DAY).is_empty(), "removed account leaves no alarm");
        }
    }
}

#[test]
fn allows_uses_good_reads_and_the_models_origin() {
    for (label, saved, number, expected) in [
        ("none", None, 1, [true, false, true]),
        ("only failure", Some(record(None, Some(Wall::EPOCH), 1, Some(ReadFailure::Agent), 0)), 1, [true, false, true]),
        ("good member", Some(recent()), 1, [true, true, true]),
        ("good absent", Some(recent()), 2, [false, false, true]),
        (
            "failure retains good",
            Some(record(Some(Wall::EPOCH), Some(wall(1)), 1, Some(ReadFailure::Room), 0)),
            1,
            [true, true, true],
        ),
        ("last bit", Some(record(Some(Wall::EPOCH), Some(Wall::EPOCH), 1 << 63, None, 0)), 64, [true, true, true]),
        ("zero", Some(recent()), 0, [false, false, true]),
        ("past bit bound", Some(recent()), 65, [false, false, true]),
    ] {
        let world = started(Kind::Key, saved);
        for (kind, expected) in [ModelKind::Stock, ModelKind::List, ModelKind::Person].into_iter().zip(expected) {
            assert_eq!(world.domain.allows(ACCOUNT, number, kind), expected, "{label} {kind:?}");
        }
    }
}

#[test]
fn other_good_reads_union_only_the_providers_names() {
    for (label, other, other_provider, expected) in [
        ("good", Some(record(Some(Wall::EPOCH), None, 6, None, 0)), PROVIDER, 14),
        ("failed good", Some(record(Some(Wall::EPOCH), None, 6, Some(ReadFailure::Agent), 0)), PROVIDER, 14),
        ("failed only", Some(record(None, None, 6, Some(ReadFailure::Agent), 0)), PROVIDER, 8),
        ("different provider", Some(recent()), OTHER_PROVIDER, 8),
        ("never read", None, PROVIDER, 8),
    ] {
        let mut other = row(OTHER_ACCOUNT, BOB, other);
        if let Stored::Account { provider, .. } = &mut other {
            *provider = other_provider;
        }
        let world = world(
            &[
                row(ACCOUNT, ALICE, Some(recent())),
                other,
                row(DEPLOYMENT_ACCOUNT, Holder::Deployment, Some(record(Some(Wall::EPOCH), None, 8, None, 0))),
            ],
            true,
        );
        assert_eq!(world.domain.listed_names(PROVIDER, ACCOUNT), expected, "{label}");
    }
}

#[test]
fn reordering_to_a_usable_value_first_asks() {
    for (label, initial, target, expected, has_list) in [
        ("program to key", Kind::Program, SECOND, 1, true),
        ("program stays first", Kind::Program, SOURCE, 0, false),
        ("key to another key", Kind::Key, SECOND, 1, true),
        ("key stays first", Kind::Key, SOURCE, 0, true),
    ] {
        let mut rows = vec![row(ACCOUNT, ALICE, Some(recent())), source(ACCOUNT, SECOND, 1, Kind::Key)];
        rows.extend(ordered_source(initial));
        let mut world = world(&rows, true);
        world.step(Event::Want);
        world.step(grant(SOURCE));
        world.step(grant(SECOND));
        let out = world.step(change(ALICE, Change::MoveSource { account: ACCOUNT, source: target, place: 0 }));
        assert_eq!(reads(&out).len(), expected, "{label}");
        for request in reads(&out) {
            if let Request::ReadList { grant, .. } = request {
                assert_eq!(grant.account, target, "{label} takes the new first source's grant");
            }
        }
        assert_eq!(world.domain.has_list(ACCOUNT), has_list, "{label}");
    }
}
