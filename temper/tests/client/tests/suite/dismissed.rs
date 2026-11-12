//! Dismissals cross the real roots (temper/docs/design/testing/README.md, 2.1;
//! temper/docs/design/draft/to-begin.md, 5).
use jig_client::{Answered, Key, contract as c};
use jig_client_kit::harness::{App, ClientId, Duty, Front};
use skein_lib::{Duration, Env, Queue, RequestEnvelope, Time, Wall};
use temper_client_domain::{
    Event, Request,
    forge::dismissed::{self, PersonFact},
    owner::Owner,
};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_client_world::{
    app::{Root, Temper},
    fixtures,
    front::Joint,
};
use temper_engine_domain::{Key as StoreKey, Record};
use temper_engine_domain_forge as forge;

const CLIENT: ClientId = ClientId(0);
// The person's other device, as in chat.rs.
const SECOND_DEVICE: ClientId = ClientId(1);
const OTHER_CLIENT: ClientId = ClientId(2);
// Person 1 and its two sign-ins take the first three people identifiers.
const OTHER_PARTY: u64 = 4;
const OTHER_SUBJECT: u64 = 8;
const PROJECT: u32 = 1;
const WHAT: PersonFact = PersonFact::ToBegin { project: PROJECT };
const FIRST: Key = Key { asker: Owner::Forge.asker(), number: 2102 };
const SECOND: Key = Key { asker: Owner::Forge.asker(), number: 2103 };
// The same restart interval as agent_recovery.rs, within the story's one second.
const OFFLINE: Duration = Duration::from_millis(30);
// A bound on immediate protocol turns, not a product timeout.
const PUMP_ROUNDS: usize = 32;

struct Probe {
    root: Root,
    out: Queue<Request>,
    sent: Option<Key>,
    attempt: Option<skein_lib::Token>,
    answers: Vec<Answered>,
}
fn probe(seed: u64, env: &Env<temper_client_domain::Limits>) -> Probe {
    let mut root = Root::Person(Box::new(temper_client_domain::unlinked(&env.limits, seed)));
    let mut out = Queue::uncapped();
    // Dismissal probes read global facts and need no page documents.
    Temper::step(&mut root, env, Temper::start(temper_client_domain::Address::Forges, Vec::new(), 0), &mut out);
    Probe { root, out, sent: None, attempt: None, answers: Vec::new() }
}
struct Story {
    front: Joint,
    probes: Vec<Probe>,
    env: Env<temper_client_domain::Limits>,
}
impl Story {
    fn new() -> Self {
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: fixtures::domain_limits() };
        let mut front = Joint::with_dismissal_people(fixtures::SEED, env.limits);
        let mut probes = Vec::new();
        for who in [CLIENT, SECOND_DEVICE] {
            front.bind(who, fixtures::PARTY);
            probes.push(probe(fixtures::SEED + u64::try_from(who.0).expect("client number"), &env));
        }
        let mut story = Self { front, probes, env };
        story.pump();
        // These observers need session facts alone; page reads, accounts and settings
        // leave the fixed engine watch table to the third device's dismissal facts.
        for probe in &mut story.probes {
            Temper::step(
                &mut probe.root,
                &story.env,
                Event::Went { place: temper_client_domain::Place::Missing }.into(),
                &mut probe.out,
            );
            for handle in
                [temper_client_domain::pages::accounts::handle(), probe.root.person().forge().settings_handle()]
            {
                Temper::step(
                    &mut probe.root,
                    &story.env,
                    Event::Client { event: jig_client::Event::Unwatch { handle } }.into(),
                    &mut probe.out,
                );
            }
        }
        story.pump();
        story
    }
    fn other_person(&mut self, member: bool) {
        self.front.bind_dismissal_person(OTHER_CLIENT, OTHER_PARTY, OTHER_SUBJECT);
        self.probes.push(probe(fixtures::SEED + OTHER_PARTY, &self.env));
        if member {
            self.front.add_dismissal_member(CLIENT, OTHER_PARTY, PROJECT);
        }
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
                                jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, request, .. } },
                        } = &request
                            && [
                                dismissed::ask(WHAT, &self.env.limits.forge).expect("fixture fits wire bounds"),
                                dismissed::ask(GOAL_FROM, &self.env.limits.forge).expect("fixture fits wire bounds"),
                                keep_asking_request(10),
                                keep_asking_request(20),
                            ]
                            .iter()
                            .any(|asked| asked == request)
                        {
                            probe.attempt = Some(*attempt);
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
                            up:
                                jig_client::Up::Answer {
                                    attempt,
                                    envelope: RequestEnvelope::Final,
                                    answer: Some(answer),
                                    ..
                                },
                        },
                }) = event.root()
                    && Some(*attempt) == probe.attempt
                    && let Some(key) = probe.sent
                {
                    assert!(probe.root.person().client().pending(key).is_some(), "answer reaches a pending request");
                    probe.answers.push(answer.clone());
                }
                Temper::step(&mut probe.root, &self.env, event, &mut probe.out);
            }
        }
        panic!("dismissal clients and engine did not settle");
    }
    fn dismiss(&mut self, who: ClientId, key: Key) {
        self.dismiss_fact(who, key, WHAT);
    }
    fn dismiss_fact(&mut self, who: ClientId, key: Key, fact: PersonFact) {
        self.dismiss_request(who, key, dismissed::ask(fact, &self.env.limits.forge).expect("dismissal request"));
    }
    fn dismiss_request(&mut self, who: ClientId, key: Key, request: jig_client::Ask) {
        let probe = &mut self.probes[who.0];
        probe.sent = Some(key);
        probe.attempt = None;
        probe.answers.clear();
        Temper::step(
            &mut probe.root,
            &self.env,
            Event::Client { event: jig_client::Event::Send { key, name: None, request } }.into(),
            &mut probe.out,
        );
        self.pump();
    }
    fn fact(&self, who: ClientId) -> Option<bool> {
        self.probes[who.0].root.person().forge().dismissed(WHAT)
    }
    fn rows(&self) -> Vec<(StoreKey, Record)> {
        self.front
            .engine
            .store
            .rows
            .iter()
            .filter_map(|(key, record)| {
                if let StoreKey::Forge(forge::Key::Dismissed { .. }) = key {
                    Some((key.clone(), record.clone()))
                } else {
                    None
                }
            })
            .collect()
    }
    fn answered_dismissed(&self, who: ClientId) {
        let [Answered::Family(record)] = self.probes[who.0].answers.as_slice() else {
            panic!("one family answer: {:?}", self.probes[who.0].answers);
        };
        assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
        let answer = w::Answer::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&record.bytes)).expect("forge answer");
        assert_eq!(answer.outcome(), &w::Outcome::Dismissed);
        let key = self.probes[who.0].sent.expect("sent dismissal");
        assert!(self.probes[who.0].root.person().client().pending(key).is_none(), "answer retires the pending");
    }
}

#[test]
fn a_dismissal_reaches_every_client_of_the_person() {
    let mut story = Story::new();
    story.other_person(true);
    for who in [CLIENT, SECOND_DEVICE, OTHER_CLIENT] {
        assert_eq!(story.fact(who), Some(false), "client {who:?}");
    }
    story.dismiss(CLIENT, FIRST);
    story.answered_dismissed(CLIENT);
    assert_eq!(story.fact(CLIENT), Some(true));
    assert_eq!(story.fact(SECOND_DEVICE), Some(true));
    assert_eq!(story.fact(OTHER_CLIENT), Some(false));
}

#[test]
fn a_second_dismissal_is_answered_as_the_first() {
    let mut story = Story::new();
    story.dismiss(CLIENT, FIRST);
    story.answered_dismissed(CLIENT);
    let rows = story.rows();
    assert_eq!(
        rows,
        [(
            StoreKey::Forge(forge::Key::Dismissed {
                party: fixtures::PARTY,
                what: forge::PersonFact::ToBegin { project: PROJECT },
            }),
            Record::Forge {
                row: Box::new(forge::Stored::Dismissed {
                    party: fixtures::PARTY,
                    what: forge::PersonFact::ToBegin { project: PROJECT },
                })
            }
        )]
    );
    story.dismiss(SECOND_DEVICE, SECOND);
    story.answered_dismissed(SECOND_DEVICE);
    assert_eq!(story.rows(), rows);
    for (number, at) in [(2200, 10), (2201, 10), (2202, 20)] {
        let key = Key { asker: Owner::Forge.asker(), number };
        story.dismiss_request(CLIENT, key, keep_asking_request(at));
        story.answered_dismissed(CLIENT);
        let what = forge::PersonFact::KeepAsking {
            project: PROJECT,
            subject: temper_protocol_reaches::Subject::Kind(temper_protocol_reaches::Kind::Tests),
            at,
        };
        let mut expected = rows.clone();
        expected.push((
            StoreKey::Forge(forge::Key::Dismissed { party: fixtures::PARTY, what }),
            Record::Forge { row: Box::new(forge::Stored::Dismissed { party: fixtures::PARTY, what }) },
        ));
        assert_eq!(story.rows(), expected, "only the latest KeepAsking remains");
        // The forge side's dismissal watch is handle 2; read its wire facts before the reader filters them.
        let watched = story.probes[SECOND_DEVICE.0]
            .root
            .person()
            .client()
            .document(jig_client::Handle { asker: Owner::Forge.asker(), number: 2 })
            .expect("second device's dismissal watch");
        let Some(jig_client::Latest::Family(record)) = watched.latest else {
            panic!("second device's facts delivered")
        };
        let document = wire::decode_dismissals(&record.bytes, &w::CEILINGS).expect("dismissal document");
        assert_eq!(document.dismissed().len(), 2);
        let facts = document.dismissed().iter().collect::<Vec<_>>();
        let w::PersonFact::KeepAsking(value) = facts[1] else { panic!("keep asking fact") };
        assert_eq!(
            (value.project(), value.subject(), value.at()),
            (
                PROJECT,
                &temper_protocol_reaches::subject_record(temper_protocol_reaches::Subject::Kind(
                    temper_protocol_reaches::Kind::Tests
                )),
                at
            )
        );
        assert_eq!(story.fact(SECOND_DEVICE), Some(true), "To begin still reads its fact");
    }
}

fn keep_asking_request(at: u64) -> jig_client::Ask {
    let what = w::PersonFact::KeepAsking(
        w::KeepAsking::new(
            &w::CEILINGS,
            w::KeepAskingParts {
                project: PROJECT,
                subject: temper_protocol_reaches::subject_record(temper_protocol_reaches::Subject::Kind(
                    temper_protocol_reaches::Kind::Tests,
                )),
                at,
            },
        )
        .expect("fixture fits wire bounds"),
    );
    let record = w::Requesting::new(
        &w::CEILINGS,
        w::RequestingParts {
            request: w::Request::Dismiss(
                w::Dismiss::new(&w::CEILINGS, w::DismissParts { what }).expect("fixture fits wire bounds"),
            ),
        },
    )
    .expect("fixture fits wire bounds");
    jig_client::Ask::Family {
        scope: c::Scope::Project(PROJECT),
        record: jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::requesting(&record),
        },
    }
}

#[test]
fn a_dismissal_outlives_a_restart() {
    let mut story = Story::new();
    story.dismiss(CLIENT, FIRST);
    story.answered_dismissed(CLIENT);
    story.front.restart_dismissal_people(OFFLINE);
    story.front.reload(SECOND_DEVICE, story.env.now);
    story.probes[SECOND_DEVICE.0] = probe(fixtures::SEED, &story.env);
    story.pump();
    assert_eq!(story.fact(SECOND_DEVICE), None, "fresh client has no dismissal document yet");
    story.env.now = story.env.now.saturating_add(OFFLINE);
    story.env.wall = Wall::from_nanos(story.env.now.as_nanos());
    for probe in &mut story.probes {
        Temper::fire(&mut probe.root, &story.env, &mut probe.out);
    }
    story.pump();
    assert_eq!(story.fact(SECOND_DEVICE), Some(true));
}

#[test]
fn a_party_with_no_role_is_refused() {
    let mut story = Story::new();
    story.other_person(false);
    story.dismiss(OTHER_CLIENT, FIRST);
    assert_eq!(
        story.probes[OTHER_CLIENT.0].answers,
        [Answered::Jig(c::Answer::Refused { refusal: c::Refusal::Role { held: None }, at: None })]
    );
    let [Answered::Jig(c::Answer::Refused { refusal, .. })] = story.probes[OTHER_CLIENT.0].answers.as_slice() else {
        panic!("jig's refusal reached the pending dismissal");
    };
    assert_eq!(
        fixtures::phrasebooks(&story.env.limits).say_word(jig_client::words::Word::refusal(refusal), &[]).as_ref(),
        b"The held role does not permit this action"
    );
    assert!(story.probes[OTHER_CLIENT.0].root.person().client().pending(FIRST).is_none());
    assert_eq!(story.rows().len(), 0);
}

const GOAL_FROM: PersonFact = PersonFact::GoalFrom { project: PROJECT, chat: 7, goal: 11, at: 0 };
#[test]
fn a_goal_from_reaches_another_client_of_the_party() {
    let mut story = Story::new();
    story.other_person(true);
    story.dismiss_fact(CLIENT, FIRST, GOAL_FROM);
    story.answered_dismissed(CLIENT);
    assert_eq!(
        story.probes[SECOND_DEVICE.0].root.person().forge().goals_from(PROJECT, 7).collect::<Vec<_>>(),
        [(11, 0)]
    );
    assert_eq!(story.probes[OTHER_CLIENT.0].root.person().forge().goals_from(PROJECT, 7).collect::<Vec<_>>(), []);
}
