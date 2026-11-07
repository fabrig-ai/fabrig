//! The switch and decisions cross both roots (temper/docs/design/client/README.md,
//! 3.5, Away, and 4.6; temper/docs/design/domain/forge.md, 5.8 and 5.9).
use jig_client_kit::harness::{App, ClientId, Duty, Front};
use skein_lib::{Duration, Env, Queue, RequestEnvelope, Time, Wall};
use temper_client_domain::{
    Action, Address, Event, Request,
    forge::away::AwayRow,
    pages::{Act, away, home},
};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_client_world::{
    app::{Root, Temper},
    fixtures,
    front::{HostAnswer, Joint},
};
use temper_engine_domain::{Key as StoreKey, Record};
use temper_engine_domain_forge as forge;

const CLIENT: ClientId = ClientId(0);
const SECOND_DEVICE: ClientId = ClientId(1);
const PROJECT: u32 = 1;
const PUMP_ROUNDS: usize = 32;
const OFFLINE: Duration = Duration::from_millis(30);

struct Probe {
    root: Root,
    out: Queue<Request>,
    answers: Vec<jig_client::Answered>,
    decided_watch: Option<skein_lib::Token>,
    decisions: Option<w::Decisions>,
}
struct Story {
    front: Joint,
    probes: Vec<Probe>,
    env: Env<temper_client_domain::Limits>,
}
impl Story {
    fn new() -> Self {
        let mut limits = fixtures::domain_limits();
        // Home may open while the previous page's handles retire.
        // The session also holds trials and a running trial task.
        limits.client.domain.handles = 18;
        limits.client.domain.watches = 18;
        let mut story = Self {
            front: Joint::with_away_people(fixtures::SEED, limits),
            probes: Vec::new(),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
        };
        assert_eq!(story.client(fixtures::PARTY), CLIENT);
        story
    }
    fn client(&mut self, party: u64) -> ClientId {
        let who = ClientId(self.probes.len());
        self.front.bind(who, party);
        self.start_client(who);
        who
    }
    fn start_client(&mut self, who: ClientId) {
        assert_eq!(who.0, self.probes.len());
        self.probes.push(Probe {
            root: Root::Person(Box::new(temper_client_domain::unlinked(&self.env.limits, fixtures::SEED))),
            out: Queue::uncapped(),
            answers: Vec::new(),
            decided_watch: None,
            decisions: None,
        });
        self.event(
            who,
            Temper::start(Address::Welcome { step: temper_client_domain::address::Step::Welcome }, Vec::new(), 0),
        );
        // The story needs Away and Home, reserving the fixed engine table for those watches.
        for handle in [
            temper_client_domain::pages::accounts::handle(),
            self.probes[who.0].root.person().forge().settings_handle(),
        ] {
            self.event(who, Event::Client { event: jig_client::Event::Unwatch { handle } }.into());
        }
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
                        if let Request::Unlinked {
                            request:
                                jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject } },
                        } = &request
                            && Some(subject)
                                == temper_client_domain::forge::decided::watch(PROJECT, &self.env.limits.forge).as_ref()
                        {
                            probe.decided_watch = Some(*watch);
                        }
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
                if let Some(Event::Client {
                    event:
                        jig_client::Event::Up {
                            up: jig_client::Up::Answer { envelope: RequestEnvelope::Final, answer: Some(answer), .. },
                        },
                }) = event.root()
                {
                    probe.answers.push(answer.clone());
                }
                if let Some(Event::Client {
                    event: jig_client::Event::Up { up: jig_client::Up::FamilyDelivered { watch, record } },
                }) = event.root()
                    && Some(*watch) == probe.decided_watch
                {
                    probe.decisions = Some(
                        temper_client_domain::forge::decided::document(record, self.env.limits.forge)
                            .expect("decisions document"),
                    );
                }
                Temper::step(&mut probe.root, &self.env, event, &mut probe.out);
            }
        }
        panic!("Away clients and engine did not settle");
    }
    fn act(&mut self, act: away::Act) {
        self.event(CLIENT, Event::Act { action: Action::Pages(Act::Away(act)) }.into());
    }
    fn toggle(&mut self) {
        self.act(away::Act::Toggle { project: PROJECT });
    }
    fn after(&mut self, elapsed: Duration) {
        self.env.now = self.env.now.saturating_add(elapsed);
        self.env.wall = Wall::from_nanos(self.env.now.as_nanos());
        for probe in &mut self.probes {
            Temper::fire(&mut probe.root, &self.env, &mut probe.out);
        }
        self.pump();
    }
    fn row(&self, who: ClientId) -> Option<AwayRow> {
        self.probes[who.0].root.person().forge().away(PROJECT)
    }
    fn stored(&self) -> forge::Away {
        let Record::Forge { row } = self
            .front
            .engine
            .store
            .rows
            .get(&StoreKey::Forge(forge::Key::Away { party: fixtures::PARTY, project: PROJECT }))
            .expect("committed Away")
        else {
            panic!("forge record")
        };
        let forge::Stored::Away(row) = row.as_ref() else { panic!("Away row") };
        *row
    }
    fn home(&mut self, who: ClientId) {
        self.event(who, Temper::went(Address::Home { project: PROJECT }));
        if self.facts(who).to_begin() {
            self.event(who, Event::Act { action: Action::Pages(Act::SkipToBegin) }.into());
        }
        assert!(!self.facts(who).to_begin());
    }
    fn facts(&self, who: ClientId) -> home::Facts {
        home::facts(self.probes[who.0].root.person(), PROJECT, self.env.wall)
    }
    fn decide(&mut self, goal: u64, completion: u32) -> String {
        let HostAnswer::Tool(answer) = self.front.host_tool(
            goal,
            completion,
            temper_engine_domain::Tool::Decided(forge::DecidedCall {
                question: b"Which implementation?".as_slice().into(),
                options: Box::new([b"Small".as_slice().into(), b"Broad".as_slice().into()]),
                took: 0,
                why: b"Fits the design".as_slice().into(),
                kind: forge::DecisionKind::Implementation,
            }),
        ) else {
            panic!("tool answer")
        };
        let rendered = temper_engine_smith::answer_for_tool(&answer, b"decided", 1024);
        let text = String::from_utf8(rendered.text().to_vec()).expect("tool words");
        self.pump();
        text
    }
    fn decisions(&self) -> Vec<forge::Decided> {
        self.front
            .engine
            .store
            .rows
            .values()
            .filter_map(|record| {
                if let Record::Forge { row } = record
                    && let forge::Stored::Decided(row) = row.as_ref()
                {
                    Some(row.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

#[test]
fn on_crosses_the_roots_and_reaches_every_device() {
    let mut story = Story::new();
    assert_eq!(story.client(fixtures::PARTY), SECOND_DEVICE);
    assert_eq!(story.row(CLIENT), None);
    assert_eq!(story.row(SECOND_DEVICE), None);
    story.toggle();
    let row = story.row(CLIENT).expect("the switch delivered its row");
    assert_eq!(row.since, story.env.wall);
    assert_eq!(row.until, Some(Wall::from_nanos(28_800_000_000_000)));
    assert_eq!((row.kinds, row.landed, row.off), (7, 0, None));
    assert_eq!(story.row(SECOND_DEVICE), Some(row), "the other device hears the party's watch");
    let stored = story.stored();
    assert_eq!(
        stored,
        forge::Away {
            party: fixtures::PARTY,
            project: PROJECT,
            since: row.since,
            until: row.until,
            kinds: row.kinds,
            landed: row.landed,
            off: row.off,
        }
    );
    story.front.restart_dismissal_people(OFFLINE);
    assert_eq!(story.stored(), stored, "restart restores the committed row");
    story.front.reload(SECOND_DEVICE, story.env.now);
    story.probes[SECOND_DEVICE.0] = Probe {
        root: Root::Person(Box::new(temper_client_domain::unlinked(&story.env.limits, fixtures::SEED))),
        out: Queue::uncapped(),
        answers: Vec::new(),
        decided_watch: None,
        decisions: None,
    };
    story.event(SECOND_DEVICE, Temper::start(fixtures::inbox(), Vec::new(), 0));
    story.after(OFFLINE);
    assert_eq!(story.row(SECOND_DEVICE), Some(row), "a fresh client reads the restored row");
}

#[test]
fn off_keeps_the_choices() {
    let mut story = Story::new();
    story.toggle();
    story.act(away::Act::Kinds { project: PROJECT, kinds: 9 });
    let first = story.stored();
    story.after(Duration::from_secs(60));
    story.toggle();
    let off = story.stored();
    assert_eq!((off.kinds, off.until), (9, first.until), "off retains both choices");
    assert_eq!((off.since, off.landed, off.off), (first.since, first.landed, Some(story.env.wall)));
    assert!(!story.probes[CLIENT.0].root.person().away_on(PROJECT, story.env.wall));
    // Go beyond the old 08:00: the retained choice means the next 08:00 on re-entry.
    story.after(Duration::from_secs(8 * 60 * 60));
    story.toggle();
    let on = story.stored();
    assert_eq!(on.since, story.env.wall, "on again starts a new spell");
    assert!(on.since > first.since);
    assert_eq!((on.kinds, on.off), (9, None));
    assert_eq!(on.until, Some(Wall::from_nanos(115_200_000_000_000)));
    assert!(story.probes[CLIENT.0].root.person().away_on(PROJECT, story.env.wall));
}

#[test]
fn a_goal_is_answered_by_the_switch() {
    let mut story = Story::new();
    let goal = story.front.begin_away_goal();
    story.pump();
    story.home(CLIENT);
    assert!(story.facts(CLIENT).away.is_none());
    story.toggle();
    assert_eq!(story.decide(goal, 1), "Decided: carry on with Small.");
    let decisions = story.decisions();
    let [row] = decisions.as_slice() else { panic!("one committed decision: {decisions:?}") };
    assert_eq!((row.project, row.party, row.task, row.number), (PROJECT, fixtures::PARTY, goal, 1));
    assert_eq!(row.at, story.env.wall);
    assert_eq!(row.question.as_ref(), b"Which implementation?");
    assert_eq!(row.options.as_ref(), &[Box::from(b"Small".as_slice()), Box::from(b"Broad".as_slice())]);
    assert_eq!(row.took, 0);
    assert_eq!(row.why.as_ref(), b"Fits the design");
    assert!(!row.overturned);
    let form = story.facts(CLIENT).away.expect("Home hears the decision on its open watch");
    assert_eq!(form.span, Some((story.stored().since, None)));
    assert_eq!(form.landed, Some(0));
    assert_eq!(form.showing, 1);
    let [line] = form.rows.as_ref() else { panic!("one decision on Home") };
    assert_eq!((line.number, line.task, line.at), (row.number, goal, row.at));
    assert_eq!(line.question, row.question);
    assert_eq!(line.took.as_ref(), b"Small");
    assert_eq!(line.why, row.why);
    assert!(!line.overturned);
    story.after(Duration::from_secs(1));
    story.toggle();
    assert_eq!(
        story.decide(goal, 2),
        "Ask: your person is here or this kind waits for them. Ask with message, as a question; if your brief says [Requested by person N], with delegate, a person task addressed to person N."
    );
    assert_eq!(story.decisions(), decisions, "asking saves no new decision");
    assert_eq!(
        story.facts(CLIENT).away.expect("decisions remain after off").span,
        Some((story.stored().since, Some(story.env.wall)))
    );
}

#[test]
fn past_until_the_client_draws_off_and_the_root_says_ask() {
    let mut story = Story::new();
    let goal = story.front.begin_away_goal();
    story.pump();
    story.home(CLIENT);
    story.toggle();
    let before = story.stored();
    let until = before.until.expect("08:00 choice");
    assert_eq!(story.decide(goal, 1), "Decided: carry on with Small.");
    let decisions = story.decisions();
    story.after(Duration::from_nanos(until.as_nanos() - 1));
    assert!(story.probes[CLIENT.0].root.person().away_on(PROJECT, story.env.wall));
    story.after(Duration::from_nanos(2));
    assert!(story.env.wall > until);
    assert!(!story.probes[CLIENT.0].root.person().away_on(PROJECT, story.env.wall));
    assert_eq!(story.row(CLIENT).expect("expired row remains").ended(story.env.wall), Some(until));
    assert_eq!(story.stored(), before, "expiry does not write the Away row");
    assert_eq!(
        story.decide(goal, 2),
        "Ask: your person is here or this kind waits for them. Ask with message, as a question; if your brief says [Requested by person N], with delegate, a person task addressed to person N."
    );
    assert_eq!(story.decisions(), decisions, "expired Away saves no new decision");
    assert_eq!(story.stored(), before, "the call does not write the expired Away row");
    assert_eq!(story.facts(CLIENT).away.expect("the earlier decision remains").span, Some((before.since, Some(until))));
}

#[test]
fn another_partys_home_shows_nothing() {
    const OTHER_PARTY: u64 = 4;
    const OTHER_SUBJECT: u64 = 8;
    let mut story = Story::new();
    let goal = story.front.begin_away_goal();
    story.pump();
    story.home(CLIENT);
    story.toggle();
    assert_eq!(story.decide(goal, 1), "Decided: carry on with Small.");
    assert_eq!(story.facts(CLIENT).away.expect("owner's decision form").showing, 1);
    let decisions = story.decisions();
    let other = SECOND_DEVICE;
    // The people take turns on Home, leaving the fixed watch table room for the restart oracle.
    story.event(CLIENT, Temper::went(Address::Forges));
    story.front.bind_dismissal_person(other, OTHER_PARTY, OTHER_SUBJECT);
    story.front.add_dismissal_member(CLIENT, OTHER_PARTY, PROJECT);
    story.start_client(other);
    story.home(other);
    assert!(story.probes[other.0].decisions.as_ref().expect("the member's Home watch delivered").rows().is_empty());
    assert_eq!(story.row(other), None, "the switch belongs to its person");
    assert!(story.facts(other).away.is_none(), "another member sees no decision form");
    story.probes[other.0].answers.clear();
    let bounds = &w::CEILINGS;
    let overturn = w::Overturn::new(bounds, w::OverturnParts { project: PROJECT, number: 1 }).expect("overturn");
    let requesting =
        w::Requesting::new(bounds, w::RequestingParts { request: w::Request::Overturn(overturn) }).expect("request");
    story.event(
        other,
        Event::Client {
            event: jig_client::Event::Send {
                key: jig_client::Key { asker: temper_client_domain::owner::Owner::Forge.asker(), number: 2302 },
                name: None,
                request: jig_client::Ask::Family {
                    scope: jig_client::contract::Scope::Project(PROJECT),
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::requesting(&requesting),
                    },
                },
            },
        }
        .into(),
    );
    let [jig_client::Answered::Family(record)] = story.probes[other.0].answers.as_slice() else {
        panic!(
            "one overturn answer: {:?}; pending {:?}; link {:?}",
            story.probes[other.0].answers,
            story.probes[other.0]
                .root
                .person()
                .client()
                .pending(jig_client::Key { asker: temper_client_domain::owner::Owner::Forge.asker(), number: 2302 }),
            story.probes[other.0].root.person().client().link()
        )
    };
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let answer = w::Answer::decode(bounds, &mut reader).expect("forge answer");
    assert!(reader.is_empty());
    let w::Outcome::OverturnRefused(reason) = answer.outcome() else { panic!("overturn refused: {answer:?}") };
    assert_eq!(reason.reason(), &w::OverturnRefusal::Unknown);
    assert_eq!(story.decisions(), decisions, "the other person changed no decision");
    assert!(story.facts(other).away.is_none());
    story.event(other, Temper::went(Address::Forges));
    story.home(CLIENT);
    assert_eq!(story.facts(CLIENT).away.expect("owner still sees the decision").showing, 1);
}
