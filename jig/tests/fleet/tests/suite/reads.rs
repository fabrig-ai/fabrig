//! Provider reads in the fleet's world (jig/docs/design/domain/engine/settings.md, 12).
use jig_domain_fleet::{self as fleet, Event, Request};
use jig_domain_link::{EndpointBound, Grant, Hello, ListFailure, ListOutcome, Login, LoginState, SourceName};
use jig_fleet_world::fixtures::{ANALYST, domain, needs};
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};

const FIRST: Token = Token::new(101);
const SECOND: Token = Token::new(102);
const RETURNED: Token = Token::new(103);
const ENDPOINT: u32 = 7;
const RENEWED: Grant = Grant { generation: 2, ..ANALYST };
const NEXT: Grant = Grant { account: 2, ..ANALYST };

fn hello(channel: Token, slots: u32, endpoint: u32) -> Event {
    hello_endpoints(channel, slots, &[endpoint])
}
fn hello_endpoints(channel: Token, slots: u32, endpoints: &[u32]) -> Event {
    Event::Hello {
        channel,
        worker: u32::from(channel == SECOND),
        hello: Hello {
            slots,
            bounds: jig_domain_link::Bounds {
                endpoints: endpoints.iter().map(|&endpoint| EndpointBound { endpoint, window: 0, output: 0 }).collect(),
                ..jig_fleet_world::fixtures::bounds()
            },
            stop_bound: Duration::from_secs(1),
            jobs: Box::new([]),
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    }
}
fn sources(channel: Token) -> Vec<Request> {
    vec![Request::Sources { channel, sources: Box::new([]) }]
}
fn ready(channel: Token) -> Event {
    Event::Logins { channel, logins: Box::new([]) }
}
fn read(number: u64, grant: Grant) -> Event {
    Event::ReadList { read: number, endpoint: ENDPOINT, grant }
}
fn sent(channel: Token, number: u64, grant: Grant) -> Vec<Request> {
    vec![Request::ReadList { channel, read: number, endpoint: ENDPOINT, grant }]
}
fn outcome() -> ListOutcome {
    ListOutcome::Failed(ListFailure::Deadline)
}
fn listed() -> ListOutcome {
    ListOutcome::Listed(Box::new([jig_domain_link::ListedModel {
        name: b"model".as_slice().into(),
        shown: Some(b"Model".as_slice().into()),
        window: Some(4096),
        output: Some(512),
        efforts: Box::new([b"high".as_slice().into()]),
    }]))
}
fn answer(channel: Token, read: u64) -> Event {
    Event::ListRead { channel, read, outcome: outcome() }
}
fn answered(read: u64) -> Vec<Request> {
    vec![Request::ListRead { read, outcome: outcome() }]
}
fn start() -> Event {
    Event::Start {
        reply_to: ReplyTo::new(Token::new(12)),
        run: Token::new(10),
        attempt: Token::new(11),
        workstream: 1,
        kinds: fleet::Kinds::Workers,
        version: fleet::Version { charter: 1, revision: 1 },
        placement: Some(fleet::PlacementConstraint { channel: FIRST, kind: fleet::HostKind::Worker }),
        assignment: fleet::Assignment { turns: Token::new(0), answered: Token::new(0) },
        grants: Box::new([]),
    }
}
fn assigned() -> Vec<Request> {
    vec![
        Request::Assign {
            channel: FIRST,
            kind: fleet::HostKind::Worker,
            activation: 11,
            run: Token::new(10),
            attempt: Token::new(11),
            assignment: fleet::Assignment { turns: Token::new(0), answered: Token::new(0) },
        },
        Request::Placed { run: Token::new(10), attempt: Token::new(11) },
    ]
}

struct World {
    fleet: fleet::Domain,
    env: Env<fleet::Limits>,
}
impl World {
    fn new() -> Self {
        let limits = fleet::Limits { reads: 2, engine_slots: 3, ..jig_fleet_world::LIMITS };
        Self { fleet: domain(&limits, needs()), env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits } }
    }
    fn snapshot(&self) -> String {
        // These are the fleet-owned inputs to the deployment's document and capacity history.
        format!(
            "{:?}",
            (
                self.fleet.attempts(),
                self.fleet.waiting(),
                self.fleet.calls(),
                self.fleet.turns(),
                self.fleet.host_views(),
                self.fleet.attempt_views(),
                self.fleet.hello_refusals(),
                self.fleet.host_history_lost(),
                self.fleet.hello_history_lost(),
                self.fleet.next_deadline(),
                self.fleet.facts_lost(),
            )
        )
    }
    fn send(&mut self, event: Event) -> Vec<Request> {
        let unchanged = matches!(&event, Event::ReadList { .. } | Event::ListRead { .. } | Event::Grant { .. });
        let before = self.snapshot();
        let mut out = Queue::uncapped();
        fleet::step(&mut self.fleet, &self.env, event, &mut out);
        let mut resumes = 0;
        while self.fleet.is_ready() {
            assert!(resumes < 32, "read placement must settle");
            fleet::resume(&mut self.fleet, &self.env, true, &mut out);
            resumes += 1;
        }
        self.fleet.reclaim();
        if unchanged {
            assert_eq!(self.snapshot(), before, "reads leave attempts, slots, documents and capacity unchanged");
        }
        let mut requests = Vec::new();
        while let Some(request) = out.pop() {
            requests.push(request);
        }
        requests
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table holds the fleet read placement stories")]
fn reads_wait_place_retry_and_answer_without_attempts() {
    let name = SourceName::new(b"provider").expect("fixture source name");
    let rows = [
        (
            "disconnected then ready",
            vec![
                (read(1, ANALYST), vec![]),
                (hello(FIRST, 1, ENDPOINT), sources(FIRST)),
                (ready(FIRST), sent(FIRST, 1, ANALYST)),
                (answer(FIRST, 1), answered(1)),
            ],
        ),
        (
            "a kept login still permits a credential read",
            vec![
                (Event::Sources { sources: Box::new([name]) }, vec![]),
                (hello(FIRST, 1, ENDPOINT), vec![Request::Sources { channel: FIRST, sources: Box::new([name]) }]),
                (read(1, ANALYST), vec![]),
                (
                    Event::Logins {
                        channel: FIRST,
                        logins: Box::new([Login { name, state: LoginState::Usable, email: None }]),
                    },
                    sent(FIRST, 1, ANALYST),
                ),
            ],
        ),
        (
            "most free slots",
            vec![
                (hello(FIRST, 1, ENDPOINT), sources(FIRST)),
                (ready(FIRST), vec![]),
                (hello(SECOND, 2, ENDPOINT), sources(SECOND)),
                (ready(SECOND), vec![]),
                (read(1, ANALYST), sent(SECOND, 1, ANALYST)),
                (read(2, NEXT), sent(FIRST, 2, NEXT)),
            ],
        ),
        (
            "declared slots are not free slots",
            vec![
                (hello(FIRST, 2, ENDPOINT), sources(FIRST)),
                (ready(FIRST), vec![]),
                (start(), assigned()),
                (hello(SECOND, 2, ENDPOINT), sources(SECOND)),
                (ready(SECOND), vec![]),
                (read(1, ANALYST), sent(SECOND, 1, ANALYST)),
            ],
        ),
        (
            "no endpoint then a supporting hello",
            vec![
                (hello(FIRST, 3, 8), sources(FIRST)),
                (ready(FIRST), vec![]),
                (read(1, ANALYST), vec![]),
                (hello(SECOND, 1, ENDPOINT), sources(SECOND)),
                (ready(SECOND), sent(SECOND, 1, ANALYST)),
            ],
        ),
        (
            "one worker serializes reads",
            vec![
                (hello(FIRST, 1, ENDPOINT), sources(FIRST)),
                (ready(FIRST), vec![]),
                (read(1, ANALYST), sent(FIRST, 1, ANALYST)),
                (read(2, NEXT), vec![]),
                (answer(FIRST, 1), answered(1).into_iter().chain(sent(FIRST, 2, NEXT)).collect()),
                (answer(FIRST, 2), answered(2)),
            ],
        ),
        (
            "loss retries at the head and fences stale channels",
            vec![
                (hello(FIRST, 1, ENDPOINT), sources(FIRST)),
                (ready(FIRST), vec![]),
                (read(1, ANALYST), sent(FIRST, 1, ANALYST)),
                (read(2, NEXT), vec![]),
                (Event::Lost { channel: FIRST }, vec![]),
                (answer(FIRST, 1), vec![]),
                (hello(RETURNED, 1, ENDPOINT), sources(RETURNED)),
                (ready(RETURNED), sent(RETURNED, 1, ANALYST)),
                (answer(FIRST, 1), vec![]),
                (answer(RETURNED, 1), answered(1).into_iter().chain(sent(RETURNED, 2, NEXT)).collect()),
            ],
        ),
        (
            "loss puts a read ahead of an older blocked endpoint",
            vec![
                (hello(FIRST, 1, ENDPOINT), sources(FIRST)),
                (ready(FIRST), vec![]),
                (Event::ReadList { read: 1, endpoint: 8, grant: ANALYST }, vec![]),
                (read(2, NEXT), sent(FIRST, 2, NEXT)),
                (Event::Lost { channel: FIRST }, vec![]),
                (hello_endpoints(RETURNED, 1, &[ENDPOINT, 8]), sources(RETURNED)),
                (ready(RETURNED), sent(RETURNED, 2, NEXT)),
                (
                    answer(RETURNED, 2),
                    vec![
                        Request::ListRead { read: 2, outcome: outcome() },
                        Request::ReadList { channel: RETURNED, read: 1, endpoint: 8, grant: ANALYST },
                    ],
                ),
            ],
        ),
        (
            "listed rows reach the parent intact",
            vec![
                (hello(FIRST, 1, ENDPOINT), sources(FIRST)),
                (ready(FIRST), vec![]),
                (read(1, ANALYST), sent(FIRST, 1, ANALYST)),
                (
                    Event::ListRead { channel: FIRST, read: 1, outcome: listed() },
                    vec![Request::ListRead { read: 1, outcome: listed() }],
                ),
            ],
        ),
        (
            "stale answers do not release a worker",
            vec![
                (hello(FIRST, 1, ENDPOINT), sources(FIRST)),
                (ready(FIRST), vec![]),
                (hello(SECOND, 1, ENDPOINT), sources(SECOND)),
                (ready(SECOND), vec![]),
                (read(1, ANALYST), sent(FIRST, 1, ANALYST)),
                (answer(SECOND, 1), vec![]),
                (answer(FIRST, 99), vec![]),
                (answer(FIRST, 1), answered(1)),
                (answer(FIRST, 1), vec![]),
            ],
        ),
        (
            "renewal while queued",
            vec![
                (read(1, ANALYST), vec![]),
                (Event::Grant { grant: Grant { account: 2, ..RENEWED } }, vec![]),
                (
                    Event::Grant {
                        grant: Grant { form: jig_domain_link::Form::Application, generation: 3, ..ANALYST },
                    },
                    vec![],
                ),
                (Event::Grant { grant: RENEWED }, vec![]),
                (Event::Grant { grant: ANALYST }, vec![]),
                (hello(FIRST, 1, ENDPOINT), sources(FIRST)),
                (ready(FIRST), sent(FIRST, 1, RENEWED)),
            ],
        ),
        (
            "renewal never cancels an in flight read",
            vec![
                (hello(FIRST, 1, ENDPOINT), sources(FIRST)),
                (ready(FIRST), vec![]),
                (read(1, ANALYST), sent(FIRST, 1, ANALYST)),
                (Event::Grant { grant: RENEWED }, vec![]),
                (answer(FIRST, 1), answered(1)),
            ],
        ),
        (
            "no slots still takes a read",
            vec![
                (hello(FIRST, 0, ENDPOINT), sources(FIRST)),
                (ready(FIRST), vec![]),
                (read(1, ANALYST), sent(FIRST, 1, ANALYST)),
                (answer(FIRST, 1), answered(1)),
            ],
        ),
        (
            "occupied slot and document facts stay unchanged",
            vec![
                (hello(FIRST, 1, ENDPOINT), sources(FIRST)),
                (ready(FIRST), vec![]),
                (start(), assigned()),
                (read(1, ANALYST), sent(FIRST, 1, ANALYST)),
                (answer(FIRST, 1), answered(1)),
            ],
        ),
        (
            "a draining worker holds its read through the answer",
            vec![
                (hello(FIRST, 1, ENDPOINT), sources(FIRST)),
                (ready(FIRST), vec![]),
                (start(), assigned()),
                (read(1, ANALYST), sent(FIRST, 1, ANALYST)),
                (
                    Event::Answer {
                        channel: FIRST,
                        run: Token::new(10),
                        attempt: Token::new(11),
                        answer: fleet::Answer::Busy,
                        payload: Token::new(0),
                    },
                    vec![
                        Request::Drop { payload: Token::new(0) },
                        Request::Refused {
                            to: ReplyTo::new(Token::new(12)),
                            run: Token::new(10),
                            attempt: Token::new(11),
                            refusal: fleet::Refusal::Recheck,
                        },
                    ],
                ),
                (answer(FIRST, 1), answered(1)),
                (read(2, NEXT), vec![]),
            ],
        ),
        (
            "engine endpoint is never chosen",
            vec![(hello(Token::new(0), 3, ENDPOINT), vec![]), (read(1, ANALYST), vec![])],
        ),
    ];
    for (name, steps) in rows {
        let mut world = World::new();
        for (index, (event, expected)) in steps.into_iter().enumerate() {
            assert_eq!(world.send(event), expected, "{name}, step {index}");
        }
    }
    // Capacity counts both queued and in-flight reads, since either may need requeuing.
    for connected in [false, true] {
        let mut world = World::new();
        if connected {
            world.send(hello(FIRST, 1, ENDPOINT));
            world.send(ready(FIRST));
        }
        world.send(read(1, ANALYST));
        world.send(read(2, NEXT));
    }
}
