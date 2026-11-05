//! Dismissals cross the real roots (temper/docs/design/domain/testing.md, 2.1;
//! temper/docs/design/draft/to-begin.md, 5).
use jig_client::{Answered, Key, contract as c};
use jig_client_world::harness::{App, ClientId, Duty, Front};
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
const INBOX_CLIENT: ClientId = ClientId(1);
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
    let (mut root, _) = Temper::new(seed);
    let mut out = Queue::uncapped();
    Temper::step(&mut root, env, Temper::start(fixtures::inbox(), Vec::new(), 0), &mut out);
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
        for who in [CLIENT, INBOX_CLIENT] {
            front.bind(who, fixtures::PARTY);
            probes.push(probe(fixtures::SEED + u64::try_from(who.0).expect("client number"), &env));
        }
        let mut story = Self { front, probes, env };
        story.pump();
        // Reserve the fixed engine watch table for a third device's dismissal facts.
        // These observers use neither accounts nor settings.
        for probe in &mut story.probes {
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
                            && [WHAT, GOAL_FROM]
                                .iter()
                                .any(|fact| Some(request) == dismissed::ask(*fact, &self.env.limits.forge).as_ref())
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
        let probe = &mut self.probes[who.0];
        probe.sent = Some(key);
        Temper::step(
            &mut probe.root,
            &self.env,
            Event::Client {
                event: jig_client::Event::Send {
                    key,
                    name: None,
                    request: dismissed::ask(fact, &self.env.limits.forge).expect("dismissal request"),
                },
            }
            .into(),
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
    for who in [CLIENT, INBOX_CLIENT, OTHER_CLIENT] {
        assert_eq!(story.fact(who), Some(false), "client {who:?}");
    }
    story.dismiss(CLIENT, FIRST);
    story.answered_dismissed(CLIENT);
    assert_eq!(story.fact(CLIENT), Some(true));
    assert_eq!(story.fact(INBOX_CLIENT), Some(true));
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
    story.dismiss(INBOX_CLIENT, SECOND);
    story.answered_dismissed(INBOX_CLIENT);
    assert_eq!(story.rows(), rows);
}

#[test]
fn a_dismissal_outlives_a_restart() {
    let mut story = Story::new();
    story.dismiss(CLIENT, FIRST);
    story.answered_dismissed(CLIENT);
    story.front.restart_dismissal_people(OFFLINE);
    story.front.reload(INBOX_CLIENT, story.env.now);
    story.probes[INBOX_CLIENT.0] = probe(fixtures::SEED, &story.env);
    story.pump();
    assert_eq!(story.fact(INBOX_CLIENT), None, "fresh client has no dismissal document yet");
    story.env.now = story.env.now.saturating_add(OFFLINE);
    story.env.wall = Wall::from_nanos(story.env.now.as_nanos());
    for probe in &mut story.probes {
        Temper::fire(&mut probe.root, &story.env, &mut probe.out);
    }
    story.pump();
    assert_eq!(story.fact(INBOX_CLIENT), Some(true));
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
    assert!(story.rows().is_empty());
}

const GOAL_FROM: PersonFact = PersonFact::GoalFrom { project: PROJECT, chat: 7, goal: 11, at: 0 };
#[test]
fn a_goal_from_reaches_another_client_of_the_party() {
    let mut story = Story::new();
    story.other_person(true);
    story.dismiss_fact(CLIENT, FIRST, GOAL_FROM);
    story.answered_dismissed(CLIENT);
    assert_eq!(
        story.probes[INBOX_CLIENT.0].root.person().forge().goals_from(PROJECT, 7).collect::<Vec<_>>(),
        [(11, 0)]
    );
    assert_eq!(story.probes[OTHER_CLIENT.0].root.person().forge().goals_from(PROJECT, 7).collect::<Vec<_>>(), []);
}
