//! Runtime forge admission and removal on the host fixtures (forge-setup.md, 6.3).
use super::*;
use crate::{item::Item, tests::host as fixtures};
use skein_lib::{Duration, Wall};

const FIRST: u16 = 1;
const ADDED: u16 = 2;
const APP: u16 = 4; // github_settings' GitHub forge.
const RUN: Token = Token::new(1);
const PICKER: Token = Token::new(2);
const OTHER: Token = Token::new(3);
const SOCKET: Token = Token::new(10);
const NEXT_SOCKET: Token = Token::new(11);
const THREE_RUNS: u32 = 3; // A writer, its picker, and another forge's writer.
const ONE_FORGE: u32 = 1; // The smallest admitted forge table.

fn forge(number: u16) -> Forge {
    fixtures::settings(&[number]).forges.into_boxed().into_iter().next().expect("one fixture forge")
}
fn app() -> Forge {
    fixtures::github_settings().forges.into_boxed().into_iter().last().expect("GitHub fixture")
}
struct World {
    host: Host,
    env: Env<Limits>,
    up: Queue<Event>,
    io: Queue<skein_io::Request>,
}
impl World {
    fn new(limits: Limits, settings: Settings) -> Self {
        Self {
            host: Host::new(&limits, settings, fixtures::crypto()).expect("host fixtures"),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            up: Queue::uncapped(),
            io: Queue::uncapped(),
        }
    }
    fn down(&mut self, request: Request) {
        self.host.down(&self.env, request, &mut self.up, &mut self.io);
    }
    fn reconcile(&mut self, run: Token, forge: u16) {
        self.down(Request::Run { run, plan: Plan::Reconcile { forge, adopted: Box::new([]) } });
    }
    fn connecting(&mut self, run: Token, forge: u16) -> Token {
        self.reconcile(run, forge);
        let _read = self.up.pop().expect("initial admin read");
        self.down(Request::Answer {
            run,
            answer: Answer::Read(Some(crate::tests::fixtures::secret(crate::tests::fixtures::ADMIN))),
        });
        let Some(skein_io::Request::Connect { owner, .. }) = self.io.pop() else {
            panic!("expected connect");
        };
        owner
    }
    fn event(&mut self, event: skein_io::Event) {
        self.host.up(&self.env, event, &mut self.up, &mut self.io);
    }
    fn silent(&mut self) {
        self.host.fire(&self.env, &mut self.up, &mut self.io);
        assert!(self.up.is_empty());
        assert!(self.io.is_empty());
    }
}
fn removed(run: Token, forge: u16) -> Event {
    Event::Stop {
        run,
        failure: Failure { forge, step: Stage::Admin, subject: b"removed".as_slice().into(), why: Why::Down },
    }
}
fn unknown(run: Token, forge: u16) -> Event {
    Event::Stop {
        run,
        failure: Failure { forge, step: Stage::Admin, subject: b"reconcile".as_slice().into(), why: Why::Unknown },
    }
}

#[test]
fn an_added_forge_reconciles_like_a_configured_forge() {
    let mut world = World::new(fixtures::limits(), fixtures::settings(&[]));
    world.down(Request::Add { forge: forge(ADDED) });
    world.reconcile(RUN, ADDED);
    assert_eq!(world.up.pop(), Some(Event::Need { run: RUN, need: Need::Read(Item::AdminToken) }));
    world.down(Request::Answer { run: RUN, answer: Answer::Read(None) });
    assert_eq!(
        world.up.pop(),
        Some(Event::Stop {
            run: RUN,
            failure: Failure {
                forge: ADDED,
                step: Stage::Admin,
                subject: b"kit-admin".as_slice().into(),
                why: Why::Missing
            },
        })
    );
    world.silent();
    assert_eq!(world.host.next_deadline(), None);
}

#[test]
fn served_and_full_additions_leave_the_forge_unchanged() {
    for (number, why) in [(FIRST, AddRefused::Served), (ADDED, AddRefused::Full)] {
        let limits = Limits { forges: ONE_FORGE, ..fixtures::limits() };
        let mut world = World::new(limits, fixtures::settings(&[FIRST]));
        world.down(Request::Add { forge: forge(number) });
        assert_eq!(world.up.pop(), Some(Event::Refused { forge: number, why }));
        world.reconcile(RUN, FIRST);
        assert_eq!(world.up.pop(), Some(Event::Need { run: RUN, need: Need::Read(Item::AdminToken) }));
        world.down(Request::Remove { forge: FIRST });
        assert_eq!(world.up.pop(), Some(removed(RUN, FIRST)));
        assert_eq!(world.up.pop(), Some(Event::Removed { forge: FIRST }));
        world.silent();
    }
}

#[test]
fn invalid_additions_use_the_construction_checks_without_taking_room() {
    let mut address = forge(ADDED);
    address.address = skein_io::Addr::Ip(([192, 0, 2, 1], 3000).into());
    let mut name = forge(ADDED);
    name.transport = Transport::Plaintext { name: Some(b"bad/name".as_slice().into()) };
    let mut given = forge(ADDED);
    match &mut given.kind {
        Kind::Forgejo(given) => given.bot_login = Box::new([]),
        Kind::GitHub(_) => panic!("Forgejo fixture"),
    }
    let mut reachable = forge(ADDED);
    match &mut reachable.kind {
        Kind::Forgejo(given) => given.limits.reachable = fixtures::limits().inner.checked_add(1).expect("small bound"),
        Kind::GitHub(_) => panic!("Forgejo fixture"),
    }
    let mut github = app();
    match &mut github.kind {
        Kind::GitHub(given) => given.limits.pages = u32::MAX,
        Kind::Forgejo(_) => panic!("GitHub fixture"),
    }
    for (label, forge) in
        [("address", address), ("name", name), ("given", given), ("reachable", reachable), ("GitHub", github)]
    {
        let number = forge.number();
        let mut settings = fixtures::settings(&[]);
        settings.forges = List::with_capacity(ONE_FORGE);
        settings.forges.push(forge.clone()).expect("one forge");
        assert!(Host::new(&fixtures::limits(), settings, fixtures::crypto()).is_err(), "{label}");
        let mut world = World::new(fixtures::limits(), fixtures::settings(&[FIRST]));
        world.down(Request::Add { forge });
        assert_eq!(world.up.pop(), Some(Event::Refused { forge: number, why: AddRefused::Invalid }), "{label}");
        world.down(Request::Add { forge: self::forge(ADDED) });
        world.reconcile(RUN, ADDED);
        assert_eq!(world.up.pop(), Some(Event::Need { run: RUN, need: Need::Read(Item::AdminToken) }), "{label}");
        world.down(Request::Remove { forge: ADDED });
        assert_eq!(world.up.pop(), Some(removed(RUN, ADDED)), "{label}");
        assert_eq!(world.up.pop(), Some(Event::Removed { forge: ADDED }), "{label}");
        world.silent();
    }
}

#[test]
fn removal_stops_waiting_runs_and_ignores_their_late_answers() {
    let limits = Limits { runs: THREE_RUNS, ..fixtures::limits() };
    let mut world = World::new(limits, fixtures::settings(&[FIRST, ADDED]));
    world.reconcile(RUN, FIRST);
    let _read = world.up.pop();
    world.down(Request::Run {
        run: PICKER,
        plan: Plan::Reachable { forge: FIRST, installation: None, page: core::num::NonZeroU32::MIN },
    });
    let _read = world.up.pop();
    world.reconcile(OTHER, ADDED);
    let _read = world.up.pop();
    world.down(Request::Remove { forge: FIRST });
    assert_eq!(world.up.pop(), Some(removed(RUN, FIRST)));
    assert_eq!(world.up.pop(), Some(removed(PICKER, FIRST)));
    assert_eq!(world.up.pop(), Some(Event::Removed { forge: FIRST }));
    world.down(Request::Answer { run: RUN, answer: Answer::Read(None) });
    world.down(Request::Answer { run: PICKER, answer: Answer::Written });
    world.silent();
    world.down(Request::Answer { run: OTHER, answer: Answer::Read(None) });
    assert_eq!(
        world.up.pop(),
        Some(Event::Stop {
            run: OTHER,
            failure: Failure {
                forge: ADDED,
                step: Stage::Admin,
                subject: b"kit-admin".as_slice().into(),
                why: Why::Missing
            },
        })
    );
    world.silent();
    assert_eq!(world.host.next_deadline(), None);
}

#[test]
fn a_removed_or_unknown_forge_cannot_start_a_run() {
    let mut world = World::new(fixtures::limits(), fixtures::settings(&[FIRST]));
    for number in [FIRST, ADDED] {
        world.down(Request::Remove { forge: number });
        assert_eq!(world.up.pop(), Some(Event::Removed { forge: number }));
        world.reconcile(RUN, number);
        assert_eq!(world.up.pop(), Some(unknown(RUN, number)));
        world.down(Request::Remove { forge: number });
        assert_eq!(world.up.pop(), Some(Event::Removed { forge: number }));
        world.silent();
        assert_eq!(world.host.next_deadline(), None);
    }
}

#[test]
fn removal_reuses_the_reserved_slot_for_the_same_number() {
    let limits = Limits { forges: ONE_FORGE, ..fixtures::limits() };
    let mut world = World::new(limits, fixtures::settings(&[FIRST]));
    let room = world.host.forges.as_slice().as_ptr();
    world.down(Request::Remove { forge: FIRST });
    let _removed = world.up.pop();
    world.down(Request::Add { forge: forge(FIRST) });
    world.reconcile(RUN, FIRST);
    assert_eq!(world.up.pop(), Some(Event::Need { run: RUN, need: Need::Read(Item::AdminToken) }));
    assert_eq!(world.host.forges.as_slice().as_ptr(), room, "forge storage never grows");
    assert_eq!(world.host.forges.len(), ONE_FORGE);
    world.down(Request::Remove { forge: FIRST });
    assert_eq!(world.up.pop(), Some(removed(RUN, FIRST)));
    assert_eq!(world.up.pop(), Some(Event::Removed { forge: FIRST }));
    world.silent();
}

#[test]
fn removal_forgets_only_that_forges_one_time_states() {
    let mut world = World::new(fixtures::limits(), fixtures::github_settings());
    world.down(Request::Run {
        run: RUN,
        plan: Plan::Manifest { forge: APP, target: crate::github::manifest::Target::Personal },
    });
    let Some(Event::Done { outcome: reconcile::Outcome::Manifest { state, .. }, .. }) = world.up.pop() else {
        panic!("manifest result");
    };
    let other = world
        .host
        .states
        .issue(FIRST, states::Purpose::Accept, &[1; 16], Time::ZERO, Duration::from_secs(600), b"install")
        .expect("other state");
    world.down(Request::Remove { forge: APP });
    assert_eq!(world.up.pop(), Some(Event::Removed { forge: APP }));
    world.down(Request::Add { forge: app() });
    world.down(Request::Run {
        run: RUN,
        plan: Plan::Convert { forge: APP, code: crate::tests::fixtures::secret(b"code"), state },
    });
    assert_eq!(
        world.up.pop(),
        Some(Event::Stop {
            run: RUN,
            failure: Failure {
                forge: APP,
                step: Stage::State,
                subject: b"convert".as_slice().into(),
                why: Why::Unknown
            }
        })
    );
    assert_eq!(world.host.states.take(FIRST, states::Purpose::Accept, &other, Time::ZERO), Ok(()));
    world.silent();
}

#[test]
fn removal_aborts_a_connect_even_before_its_socket_arrives() {
    let mut world = World::new(fixtures::limits(), fixtures::settings(&[FIRST]));
    let owner = world.connecting(RUN, FIRST);
    world.down(Request::Remove { forge: FIRST });
    assert_eq!(world.up.pop(), Some(removed(RUN, FIRST)));
    assert_eq!(world.up.pop(), Some(Event::Removed { forge: FIRST }));
    world.event(skein_io::Event::Connecting { owner, socket: SOCKET });
    assert_eq!(world.io.pop(), Some(skein_io::Request::Abort { entity: SOCKET }));
    world.down(Request::Add { forge: forge(FIRST) });
    world.reconcile(OTHER, FIRST);
    assert_eq!(world.up.pop(), Some(Event::Need { run: OTHER, need: Need::Read(Item::AdminToken) }));
    world.event(skein_io::Event::Connected { owner });
    world.event(skein_io::Event::Closed { owner });
    world.down(Request::Remove { forge: FIRST });
    assert_eq!(world.up.pop(), Some(removed(OTHER, FIRST)));
    assert_eq!(world.up.pop(), Some(Event::Removed { forge: FIRST }));
    world.silent();
    assert_eq!(world.host.next_deadline(), None);
}

#[test]
fn removal_aborts_live_and_settling_connections_only_for_its_forge() {
    for live in [true, false] {
        let mut world = World::new(fixtures::limits(), fixtures::settings(&[FIRST, ADDED]));
        let owner = world.connecting(RUN, FIRST);
        world.event(skein_io::Event::Connecting { owner, socket: SOCKET });
        let other = world.connecting(OTHER, ADDED);
        world.event(skein_io::Event::Connecting { owner: other, socket: NEXT_SOCKET });
        if !live {
            // A completed run leaves its connection held until io's Closed.
            world.host.runs.get_mut(&RUN).expect("run").reply(Reply::Status(403), &mut world.host.plan_out);
            world.host.fire(&world.env, &mut world.up, &mut world.io);
            let _stop = world.up.pop().expect("finished run");
        }
        world.down(Request::Remove { forge: FIRST });
        if live {
            assert_eq!(world.up.pop(), Some(removed(RUN, FIRST)));
        }
        assert_eq!(world.up.pop(), Some(Event::Removed { forge: FIRST }));
        assert_eq!(world.io.pop(), Some(skein_io::Request::Abort { entity: SOCKET }));
        assert!(world.io.is_empty(), "the other forge's connection stays open");
        world.event(skein_io::Event::Closed { owner });
        world.down(Request::Remove { forge: ADDED });
        assert_eq!(world.up.pop(), Some(removed(OTHER, ADDED)));
        assert_eq!(world.up.pop(), Some(Event::Removed { forge: ADDED }));
        assert_eq!(world.io.pop(), Some(skein_io::Request::Abort { entity: NEXT_SOCKET }));
        world.event(skein_io::Event::Closed { owner: other });
        world.silent();
        assert_eq!(world.host.next_deadline(), None);
    }
}
