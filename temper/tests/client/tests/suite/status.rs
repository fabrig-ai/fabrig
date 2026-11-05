//! Home's entry looks cross the real roots (temper/docs/design/domain/testing.md,
//! 2.1; temper/docs/design/domain/forge.md, 5.6; temper/docs/design/client/README.md, 3.5).
use jig_client::{Handle, Key, Latest};
use jig_client_world::harness::{App, ClientId, Duty, Front};
use skein_lib::{Duration, Env, Queue, Time, Wall};
use temper_client_domain::{
    Action, Address, Event, Request,
    forge::dismissed::{self, Dismissal},
    forge::status,
    owner::Owner,
    pages::{
        Act,
        home::{self, Facts, Status},
    },
};
use temper_client_protocol_wire::v1 as w;
use temper_client_world::{
    app::{Root, Temper},
    fixtures,
    front::Joint,
};
use temper_engine_domain::{Key as StoreKey, Record};
use temper_engine_domain_forge as forge;

const CLIENT: ClientId = ClientId(0);
const PROJECT: u32 = 1;
const PUMP_ROUNDS: usize = 32;
// Bounds immediate forge progression; no real clock or poll wait is needed.
const LANDING_ROUNDS: usize = 200;
const SECOND_DEVICE: ClientId = ClientId(1);
const OTHER_CLIENT: ClientId = ClientId(2);
// The owner and its two sign-ins occupy the first three people identifiers.
const OTHER_PARTY: u64 = 4;
const OTHER_SUBJECT: u64 = 8;
const WATCH: Handle = Handle { asker: Owner::Forge.asker(), number: 2202 };
const OFFLINE: Duration = Duration::from_millis(30);

struct Probe {
    root: Root,
    out: Queue<Request>,
}

struct Story {
    front: Joint,
    probes: Vec<Probe>,
    env: Env<temper_client_domain::Limits>,
}

impl Story {
    fn new() -> Self {
        let mut limits = fixtures::domain_limits();
        // A new page opens before the previous page's handles finish retiring.
        limits.client.domain.handles = 16;
        limits.client.domain.watches = 16;
        Self {
            front: Joint::with_status_people(fixtures::SEED, limits),
            probes: Vec::new(),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
        }
    }

    fn client(&mut self, party: u64, address: Address) -> ClientId {
        let who = ClientId(self.probes.len());
        self.front.bind(who, party);
        let root = Root::Person(Box::new(temper_client_domain::unlinked(&self.env.limits, fixtures::SEED)));
        self.probes.push(Probe { root, out: Queue::uncapped() });
        self.event(who, Temper::start(address, Vec::new(), 0));
        who
    }

    fn with_landing() -> Self {
        let mut story = Self::new();
        story.front = Joint::with_status_landing(fixtures::SEED, story.env.limits);
        story
    }

    fn land(&mut self) {
        let goal = self.front.begin_status_landing();
        self.pump();
        for _ in 0..LANDING_ROUNDS {
            self.front.approve_status_landing(goal);
            self.after(Duration::from_millis(10));
            if self.front.engine.task(goal).is_none() {
                let Record::Forge { row } = self
                    .front
                    .engine
                    .store
                    .rows
                    .get(&StoreKey::Forge(forge::Key::ProjectFacts { project: PROJECT }))
                    .expect("the landing committed project facts")
                else {
                    panic!("forge facts")
                };
                let forge::Stored::ProjectFacts(row) = row.as_ref() else { panic!("project facts row") };
                assert_eq!(row.landed, 1, "the real landing counted once");
                assert!(
                    self.front.engine.store.rows.values().any(|record| {
                        matches!(record, Record::Forge { row } if matches!(row.as_ref(),
                        forge::Stored::LandingLog(log) if log.landings.iter()
                            .any(|landing| landing.changes.iter().any(|change| change.task == goal))))
                    }),
                    "the real landing log holds the change"
                );
                return;
            }
        }
        panic!("the change did not land");
    }

    fn event(&mut self, who: ClientId, event: temper_client_world::app::Event) {
        let probe = &mut self.probes[who.0];
        Temper::step(&mut probe.root, &self.env, event, &mut probe.out);
        self.pump();
    }

    fn pump(&mut self) {
        for _ in 0..PUMP_ROUNDS {
            let mut worked = false;
            for (index, probe) in self.probes.iter_mut().enumerate() {
                Temper::take(&mut probe.root, &self.env, &mut probe.out);
                while let Some(request) = probe.out.pop() {
                    worked = true;
                    if let Duty::Wire(request) = Temper::duty(request) {
                        self.front.wire(ClientId(index), self.env.now, self.env.wall, request);
                    }
                }
            }
            let callbacks = self.front.poll(self.env.now, self.env.wall);
            if !worked && callbacks.is_empty() {
                return;
            }
            for (who, event) in callbacks {
                self.front.received(who, &event);
                let probe = &mut self.probes[who.0];
                Temper::step(&mut probe.root, &self.env, event, &mut probe.out);
            }
        }
        panic!("status clients and engine did not settle");
    }

    fn enter(&mut self) {
        assert_eq!(self.client(fixtures::PARTY, fixtures::inbox()), CLIENT);
        // Keep the real dismissal before this Home's first entry, so To begin is already skipped.
        self.event(
            CLIENT,
            Event::Client {
                event: jig_client::Event::Send {
                    key: Key { asker: Owner::Forge.asker(), number: 2201 },
                    name: None,
                    request: dismissed::ask(Dismissal::ToBegin { project: PROJECT }, &self.env.limits.forge)
                        .expect("skip request"),
                },
            }
            .into(),
        );
        self.go(CLIENT, Address::Home { project: PROJECT });
    }

    fn go(&mut self, who: ClientId, address: Address) {
        self.event(who, Temper::went(address));
    }

    fn after(&mut self, elapsed: Duration) {
        self.env.now = self.env.now.saturating_add(elapsed);
        self.env.wall = Wall::from_nanos(self.env.now.as_nanos());
        for probe in &mut self.probes {
            Temper::fire(&mut probe.root, &self.env, &mut probe.out);
        }
        self.pump();
    }

    fn facts(&self) -> Facts {
        let root = self.probes[CLIENT.0].root.person();
        assert_eq!(root.pages().address(), &Address::Home { project: PROJECT });
        home::facts(root, PROJECT)
    }

    fn shown(&self) -> w::ProjectStatus {
        let Status::Shown(document) = self.facts().status else {
            panic!(
                "Home's status is shown at {:?}, link {:?}, row {:?}: {:?}",
                self.env.now,
                self.probes[CLIENT.0].root.person().client().link(),
                self.looked(fixtures::PARTY),
                self.facts()
            )
        };
        *document
    }

    fn looked(&self, party: u64) -> Option<forge::Looked> {
        self.front.engine.store.rows.get(&StoreKey::Forge(forge::Key::Looked { party, project: PROJECT })).map(
            |record| {
                let Record::Forge { row } = record else { panic!("forge record") };
                let forge::Stored::Looked(row) = row.as_ref() else { panic!("looked row") };
                row.clone()
            },
        )
    }

    fn rows(&self) -> Vec<(StoreKey, Record)> {
        self.front
            .engine
            .store
            .rows
            .iter()
            .filter_map(|(key, record)| {
                if matches!(key, StoreKey::Forge(forge::Key::ProjectFacts { .. } | forge::Key::Looked { .. })) {
                    Some((key.clone(), record.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    fn watch(&mut self, who: ClientId) {
        // Observers exercise status alone; their unused account and settings
        // subscriptions leave room for Home within the engine's fixed view bound.
        let settings = self.probes[who.0].root.person().forge().settings_handle();
        for handle in [temper_client_domain::pages::accounts::handle(), settings] {
            self.event(who, Event::Client { event: jig_client::Event::Unwatch { handle } }.into());
        }
        self.event(
            who,
            Event::Client {
                event: jig_client::Event::Watch {
                    handle: WATCH,
                    subject: status::watch(PROJECT, &self.env.limits.forge).expect("status watch"),
                },
            }
            .into(),
        );
    }

    fn watched(&self, who: ClientId) -> w::ProjectStatus {
        let client = self.probes[who.0].root.person().client();
        let document = client.document(WATCH).expect("status handle");
        assert_eq!(document.state, jig_client::State::Live, "status watch stays live");
        let Some(Latest::Family(record)) = document.latest else { panic!("status document delivered to {who:?}") };
        status::document(record, self.env.limits.forge).expect("decoded status")
    }

    fn look(&mut self, who: ClientId, number: u64) {
        self.event(
            who,
            Event::Client {
                event: jig_client::Event::Send {
                    key: Key { asker: Owner::Forge.asker(), number },
                    name: None,
                    request: status::ask(PROJECT, &self.env.limits.forge).expect("look request"),
                },
            }
            .into(),
        );
    }
}

#[test]
fn entering_home_looks_once_and_shows_status() {
    let mut story = Story::new();
    story.enter();
    let row = story.looked(fixtures::PARTY).expect("committed look");
    assert_eq!(row.previous, None);
    assert_eq!(row.latest, story.env.wall);
    assert_eq!((row.seen, row.next_seen), (0, 0));
    assert_eq!(story.shown().previous(), &None);
    assert_eq!(story.facts().landed_since(), 0);
    assert_eq!(story.facts().brief(), None);
    assert_eq!(story.front.status_looks(CLIENT), 1);
    story.pump();
    assert_eq!(story.front.status_looks(CLIENT), 1, "the settled Home sends no second look");
}

#[test]
fn a_second_entry_within_the_sitting_keeps_its_last_looked() {
    let mut story = Story::new();
    story.enter();
    let first = story.looked(fixtures::PARTY).expect("first look");
    story.go(CLIENT, fixtures::inbox());
    story.after(Duration::from_secs(31 * 60));
    story.go(CLIENT, Address::Home { project: PROJECT });
    assert_eq!(*story.shown().previous(), Some(first.latest.as_nanos()));
    let before = story.facts();
    let sitting = story.looked(fixtures::PARTY).expect("new sitting");
    story.go(CLIENT, fixtures::inbox());
    story.after(Duration::from_secs(29 * 60));
    story.go(CLIENT, Address::Home { project: PROJECT });
    let after = story.looked(fixtures::PARTY).expect("same sitting");
    assert_eq!(after.previous, sitting.previous, "29 minutes keeps the previous sitting's last look");
    assert_eq!(after.seen, sitting.seen);
    assert_eq!(after.latest, story.env.wall, "this sitting's last look advances");
    assert_eq!(story.shown().previous(), &Some(first.latest.as_nanos()));
    assert_eq!(story.facts().landed_since(), before.landed_since());
    assert_eq!(story.front.status_looks(CLIENT), 3, "one look per entry");
}

#[test]
fn coming_back_after_a_landing_says_what_landed() {
    let mut story = Story::with_landing();
    story.enter();
    let first = story.looked(fixtures::PARTY).expect("first sitting");
    assert_eq!(story.facts().landed_since(), 0);
    story.go(CLIENT, fixtures::inbox());
    story.after(Duration::from_secs(60));
    story.land();
    assert_eq!(story.looked(fixtures::PARTY), Some(first.clone()), "landing while away does not look");
    story.after(Duration::from_secs(31 * 60));
    story.go(CLIENT, Address::Home { project: PROJECT });
    let document = story.shown();
    assert_eq!(document.previous(), &Some(first.latest.as_nanos()));
    assert_eq!((document.landed(), document.seen()), (1, 0));
    assert_eq!(story.facts().landed_since(), 1, "seen is the count at the previous sitting's last look");
    let second = story.looked(fixtures::PARTY).expect("second sitting");
    assert_eq!((second.seen, second.next_seen), (0, 1));
    assert_eq!(story.front.status_looks(CLIENT), 2);
}

#[test]
fn no_look_goes_while_to_begin_shows() {
    let mut story = Story::new();
    story.client(fixtures::PARTY, Address::Home { project: PROJECT });
    assert!(story.facts().to_begin());
    assert_eq!(story.facts().status, Status::Unknown);
    assert_eq!(story.looked(fixtures::PARTY), None);
    assert_eq!(story.front.status_looks(CLIENT), 0);
    story.after(Duration::from_secs(31 * 60));
    assert!(story.facts().to_begin(), "To begin remains until done or skipped");
    assert_eq!(story.facts().status, Status::Unknown);
    assert_eq!(story.looked(fixtures::PARTY), None);
    assert_eq!(story.front.status_looks(CLIENT), 0, "time passing sends no look");
}

#[test]
fn a_skip_sends_it() {
    let mut story = Story::new();
    story.client(fixtures::PARTY, Address::Home { project: PROJECT });
    assert!(story.facts().to_begin());
    assert_eq!(story.front.status_looks(CLIENT), 0);
    story.event(CLIENT, Event::Act { action: Action::Pages(Act::SkipToBegin) }.into());
    assert!(!story.facts().to_begin());
    assert_eq!(story.shown().previous(), &None);
    assert_eq!(story.facts().landed_since(), 0);
    let row = story.looked(fixtures::PARTY).expect("Skip sends the entry's look");
    assert_eq!(row.latest, story.env.wall);
    assert_eq!(story.front.status_looks(CLIENT), 1);
    story.pump();
    assert_eq!(story.front.status_looks(CLIENT), 1, "Skip sends this entry's look only once");
}

#[test]
fn a_look_reaches_another_device_of_the_party() {
    let mut story = Story::new();
    story.enter();
    let first = story.looked(fixtures::PARTY).expect("owner's first look");
    story.go(CLIENT, Address::Forges);
    // Observers need no project page or its standing needs watch.
    let observing = Address::Welcome { step: temper_client_domain::address::Step::Welcome };
    assert_eq!(story.client(fixtures::PARTY, observing.clone()), SECOND_DEVICE);
    story.watch(SECOND_DEVICE);
    story.front.bind_dismissal_person(OTHER_CLIENT, OTHER_PARTY, OTHER_SUBJECT);
    story.front.add_dismissal_member(CLIENT, OTHER_PARTY, PROJECT);
    assert_eq!(story.client(OTHER_PARTY, observing), OTHER_CLIENT);
    story.watch(OTHER_CLIENT);
    story.after(Duration::from_secs(60));
    story.look(OTHER_CLIENT, 2203);
    let other_first = story.looked(OTHER_PARTY).expect("member's first look");
    story.after(Duration::from_secs(31 * 60));
    story.look(OTHER_CLIENT, 2204);
    let other = story.looked(OTHER_PARTY).expect("member's second sitting");
    assert_eq!(story.watched(OTHER_CLIENT).previous(), &Some(other_first.latest.as_nanos()));
    assert_eq!(story.watched(SECOND_DEVICE).previous(), &None, "the owner's sitting has not moved");
    story.go(CLIENT, Address::Home { project: PROJECT });
    assert_eq!(story.shown().previous(), &Some(first.latest.as_nanos()));
    assert_eq!(
        story.watched(SECOND_DEVICE).previous(),
        &Some(first.latest.as_nanos()),
        "the first device's look marks the second device's open watch"
    );
    assert_eq!(
        story.watched(OTHER_CLIENT).previous(),
        &Some(other_first.latest.as_nanos()),
        "another party's watch keeps that party's previous sitting"
    );
    assert_eq!(story.looked(OTHER_PARTY), Some(other), "the owner's look changes no other party's row");
    assert_eq!(story.front.status_looks(SECOND_DEVICE), 0, "the watching device sent no look");
}

#[test]
fn a_restart_keeps_the_rows() {
    let mut story = Story::with_landing();
    story.enter();
    story.go(CLIENT, fixtures::inbox());
    story.land();
    story.after(Duration::from_secs(31 * 60));
    story.go(CLIENT, Address::Home { project: PROJECT });
    assert_eq!(story.facts().landed_since(), 1);
    let document = story.shown();
    let looked = story.looked(fixtures::PARTY).expect("look before restart");
    let rows = story.rows();
    assert_eq!(rows.len(), 2, "one ProjectFacts and one Looked row");
    story.front.restart_dismissal_people(OFFLINE);
    assert_eq!(story.rows(), rows, "the committed rows survive the engine restart");
    story.front.reload(CLIENT, story.env.now);
    story.probes[CLIENT.0] = Probe {
        root: Root::Person(Box::new(temper_client_domain::unlinked(&story.env.limits, fixtures::SEED))),
        out: Queue::uncapped(),
    };
    story.event(CLIENT, Temper::start(fixtures::inbox(), Vec::new(), 0));
    story.after(OFFLINE);
    story.go(CLIENT, Address::Home { project: PROJECT });
    assert_eq!(story.shown(), document, "a fresh Home reads the restored rows");
    assert_eq!(story.facts().landed_since(), 1);
    let after = story.looked(fixtures::PARTY).expect("look after restart");
    assert_eq!(after.previous, looked.previous);
    assert_eq!((after.seen, after.next_seen), (looked.seen, looked.next_seen));
    assert_eq!(after.latest, story.env.wall);
}
