//! Stock starts either reach the worker root or fail with their numbers;
//! the socket stays live (temper/docs/design/domain/worker.md, 4.3;
//! temper/docs/design/protocol/channel.md, 7 and 8).
use super::{fixtures, fixtures::worker_peers::LinkPeer};
use crate::{limits, settings, values::Values, workers::Workers};
use jig_domain_core as core;
use jig_domain_host as hub;
use jig_domain_tasks as tasks;
use jig_protocol_config as config;
use jig_protocol_engine_workers as kit;
use jig_protocol_worker_wire::{self as wire, Kind};
use skein_lib::{Duration, List, Queue, Time, Token, Wall};
use temper_engine_domain as root;
use temper_protocol_worker_wire::v1 as payload;

const CHANNEL: Token = Token::new(1);
const SECONDS: u64 = 120;
const SETTLE_STEPS: u16 = 1000; // The shared socket peer's finite settling bound.
const SMALL_RUN: u64 = 8;
const ASSIGNMENTS: u32 = 2; // The row's start, then the next small run.
const REFUSALS: u32 = 1; // One answer and one note for a refused start.
const WORKSPACES: u32 = 1; // Only the row's start carries workspace items.
const SPECIFICATION: &[u8] = b"Keep the worker link live.";

#[derive(Clone, Copy, Debug)]
enum Part {
    Charter,
    Window,
    Tree,
    Inbox,
    Calls,
    All,
}
#[derive(Clone, Copy, Debug)]
enum Outcome {
    Crosses,
    Past,
    CannotEncode,
}

fn settings(profile: config::Profile) -> settings::Engine {
    let mut settings = fixtures::engine();
    settings.jig.limits = config::profile(profile);
    settings.limits = limits::profile(match profile {
        config::Profile::Small => limits::Profile::Small,
        config::Profile::Standard => limits::Profile::Standard,
    });
    settings
}

#[test]
fn stock_profiles_make_both_links_and_fit_every_nonstart_frame() {
    for profile in [config::Profile::Small, config::Profile::Standard] {
        let settings = settings(profile);
        let engine = crate::workers::limits(&settings).expect("stock engine link limits");
        let mut worker = fixtures::worker_settings();
        worker.jig.limits = settings.jig.limits;
        let payload = crate::worker_settings::payload_limits(&settings.limits).unwrap();
        let worker = crate::worker_settings::link_limits(&worker.jig, &payload, settings.limits.forge.forges)
            .expect("stock worker link limits");
        assert_eq!(engine.room, settings.jig.limits.io.output);
        assert_eq!(worker.room, settings.jig.limits.io.output);
        assert!(kit::valid(&engine), "{profile:?}");
        assert!(jig_protocol_worker_link::valid(&worker), "{profile:?}");
        for rule in wire::KINDS {
            let start = match rule.body {
                Kind::Assignment | Kind::Job => true,
                Kind::Hello
                | Kind::Turn
                | Kind::Call
                | Kind::Answer
                | Kind::Fact
                | Kind::Unauthorized
                | Kind::Limited
                | Kind::AccountRefused
                | Kind::Message
                | Kind::Relayed
                | Kind::TurnKept
                | Kind::TurnBusy
                | Kind::Acknowledge
                | Kind::Cancel
                | Kind::Grant
                | Kind::Sources
                | Kind::Check
                | Kind::ReadList
                | Kind::ListRead
                | Kind::Draining
                | Kind::SignOut
                | Kind::Logins
                | Kind::SignedOut
                | Kind::JobAnswer => false,
            };
            if start {
                continue;
            }
            let (sizes, room) = match rule.direction {
                skein_channel::Direction::FromResponder => (&engine.sizes, engine.room),
                skein_channel::Direction::FromInitiator => (&worker.sizes, worker.room),
            };
            let bytes = wire::sizes::largest_body(sizes, rule.body).unwrap().checked_add(8).unwrap();
            assert!(bytes <= room, "{profile:?} {:?}: {bytes} > {room}", rule.body);
        }
    }
}

#[test]
fn a_stock_start_crosses_the_link_or_fails_with_its_numbers() {
    for profile in [config::Profile::Small, config::Profile::Standard] {
        let inbox = match profile {
            config::Profile::Small => Outcome::Crosses,
            // 82 messages of 262,144 bytes exceed the 1 MiB room alone.
            config::Profile::Standard => Outcome::Past,
        };
        for (part, extra, outcome) in [
            (Part::Charter, 0, Outcome::Crosses),
            (Part::Window, 0, Outcome::Crosses),
            (Part::Tree, 0, Outcome::Crosses),
            (Part::Inbox, 0, inbox),
            (Part::Calls, 0, Outcome::Past),
            (Part::All, 0, Outcome::Past),
            (Part::Charter, 1, Outcome::CannotEncode),
            (Part::Window, 1, Outcome::CannotEncode),
            (Part::Tree, 1, Outcome::CannotEncode),
            (Part::Inbox, 1, Outcome::CannotEncode),
            (Part::Calls, 1, Outcome::CannotEncode),
        ] {
            check_start(profile, part, extra, outcome);
        }
    }
}

struct Joint {
    worker: crate::Worker,
    peer: LinkPeer,
    assignments: List<hub::Assignment>,
    unsent: List<kit::Unsent>,
    answers: List<hub::RunAnswer>,
    items: List<temper_worker_domain::wire::Workspace>,
}
impl Joint {
    fn new(settings: &settings::Engine) -> Self {
        let mut worker = fixtures::worker_settings();
        worker.jig.limits = settings.jig.limits;
        worker.jig.engine = config::Address::Addr(skein_io::Addr::Ip(([127, 0, 0, 1], 7100).into()));
        let mut agent = fixtures::worker_agent();
        // The peer stops at the worker root; its agent fixture still has to
        // admit the stock root's checkout and hub when the Worker is made.
        agent.host_limits.directories = settings.limits.checkout.repositories;
        agent.host_limits.conflicts = settings.limits.checkout.conflicts;
        agent.host_limits.path_bytes = settings.limits.checkout.path_bytes;
        agent.host_limits.name_bytes = settings.limits.checkout.name_bytes;
        agent.host_limits.accounts = settings.jig.limits.hub.accounts;
        agent.host_limits.messages = settings.jig.limits.hub.inbox.max(settings.jig.limits.hub.held);
        agent.host_limits.message_bytes =
            settings.jig.limits.hub.event_bytes.checked_add(temper_worker_domain::SEPARATOR).unwrap();
        agent.host_limits.call_bytes = u64::from(settings.limits.checkout.message_bytes);
        let stand = fixtures::worker_peers::socket_stand_with_limits(&worker, &agent, &settings.limits);
        let room = settings.jig.limits.io.largest_room();
        let limits = crate::workers::limits(settings).unwrap();
        let mut joint = Self {
            worker: stand.worker,
            peer: LinkPeer::with_engine_limits(&worker, &agent, &settings.limits, &limits, room, room),
            assignments: List::with_capacity(ASSIGNMENTS),
            unsent: List::with_capacity(REFUSALS),
            answers: List::with_capacity(REFUSALS),
            items: List::with_capacity(WORKSPACES),
        };
        joint.peer.open(&mut joint.worker);
        joint.settle();
        joint
    }
    fn settle(&mut self) {
        for _ in 0..SETTLE_STEPS {
            let before = (
                self.peer.sent,
                self.peer.received,
                self.peer.engine_bytes.len(),
                self.peer.worker_bytes.len(),
                self.peer.out.len(),
                self.worker.io().len(),
            );
            if let Some(request) = self.peer.exchange(&mut self.worker) {
                match request {
                    kit::Request::Unsent { worker, channel, unsent } => {
                        assert_eq!((worker, channel), (0, CHANNEL));
                        self.unsent.push(unsent).unwrap();
                    }
                    kit::Request::Up { channel, up: hub::LinkUp::Answer { run, attempt, answer }, parts } => {
                        assert_eq!((channel, run, attempt), (CHANNEL, Token::new(7), Token::new(2)));
                        assert!(parts.is_empty());
                        self.answers.push(answer).unwrap();
                    }
                    other @ (kit::Request::Take { .. }
                    | kit::Request::TurnAway { .. }
                    | kit::Request::Lower { .. }
                    | kit::Request::Close { .. }
                    | kit::Request::Abort { .. }
                    | kit::Request::Hello { .. }
                    | kit::Request::Up { .. }
                    | kit::Request::Lost { .. }
                    | kit::Request::Ended { .. }
                    | kit::Request::Closed) => {
                        panic!("unexpected engine output; the link must stay up: {other:?}")
                    }
                }
            }
            while let Some(event) = self.worker.engine.pop() {
                match event {
                    temper_worker_domain::Event::Assign { assignment, items } => {
                        self.items.push(items.expect("workspace decodes")).unwrap();
                        self.assignments.push(assignment).unwrap();
                    }
                    temper_worker_domain::Event::Link(hub::LinkDown::Assign { assignment }) => {
                        self.assignments.push(assignment).unwrap();
                    }
                    other @ (temper_worker_domain::Event::SourceAnswered { .. }
                    | temper_worker_domain::Event::ListRead { .. }
                    | temper_worker_domain::Event::AssignJob { .. }
                    | temper_worker_domain::Event::Job { .. }
                    | temper_worker_domain::Event::Link(_)
                    | temper_worker_domain::Event::Ready
                    | temper_worker_domain::Event::Lost
                    | temper_worker_domain::Event::GracePassed
                    | temper_worker_domain::Event::Resend { .. }
                    | temper_worker_domain::Event::RelayCancelled { .. }
                    | temper_worker_domain::Event::Process(_)
                    | temper_worker_domain::Event::Live { .. }
                    | temper_worker_domain::Event::Decoded { .. }
                    | temper_worker_domain::Event::Encoded { .. }
                    | temper_worker_domain::Event::Activity { .. }
                    | temper_worker_domain::Event::Done { .. }
                    | temper_worker_domain::Event::Shutdown) => {
                        panic!("unexpected worker root event; the link must stay up: {other:?}")
                    }
                }
            }
            self.worker.iterate(self.peer.now, Wall::EPOCH);
            let after = (
                self.peer.sent,
                self.peer.received,
                self.peer.engine_bytes.len(),
                self.peer.worker_bytes.len(),
                self.peer.out.len(),
                self.worker.io().len(),
            );
            if before == after
                && self.peer.out.is_empty()
                && self.worker.io().is_empty()
                && self.peer.engine_bytes.is_empty()
                && self.peer.worker_bytes.is_empty()
            {
                return;
            }
        }
        panic!("the socket joint settles");
    }
}

fn workers(settings: &settings::Engine) -> Workers {
    let mut encoding = fixtures::encoding(settings);
    encoding.limits = crate::translate::limits(&settings.limits).unwrap();
    let mut forges = fixtures::forges();
    // The longest remote fits the engine's remote bound with both forge names full.
    let names = usize::try_from(encoding.limits.repository_name).unwrap();
    let remote = usize::try_from(encoding.limits.repository_remote).unwrap();
    forges[0].git = filled(remote.checked_sub(names.checked_mul(2).unwrap()).unwrap().checked_sub(6).unwrap());
    forges[0].git_program = Some(filled(usize::try_from(encoding.limits.repository_git_some).unwrap()));
    let mut jig = settings.jig.clone();
    jig.hosts.workers = Box::new([jig.hosts.workers[0].clone()]);
    Workers::new(
        &jig,
        &fixtures::files(),
        encoding,
        forges,
        settings.limits.forge.forges,
        &crate::workers::limits(settings).unwrap(),
    )
    .unwrap()
}

#[expect(
    clippy::print_stderr,
    clippy::use_debug,
    clippy::disallowed_macros,
    reason = "the stock table reports measured frames for the implementation report"
)]
fn check_start(profile: config::Profile, part: Part, extra: usize, outcome: Outcome) {
    let settings = settings(profile);
    let mut workers = workers(&settings);
    let values = Values::new(settings.jig.limits.core.accounts.accounts, settings.jig.limits.token);
    let assignment = assignment(&settings, &workers, part, extra);
    let mut expected_items = None;
    let mut joint = Joint::new(&settings);
    let sent = joint.peer.sent;
    match workers.encoded_assignment(Time::ZERO, CHANNEL, assignment.clone(), &values) {
        Some(event) => {
            let kit::Event::Down { down: hub::LinkDown::Assign { assignment }, parts, .. } = &event else {
                panic!("assignment event")
            };
            if assignment.assignment.workspace.is_some() {
                expected_items = Some(
                    crate::worker_route::items(
                        assignment,
                        parts,
                        &crate::worker_settings::payload_limits(&settings.limits).unwrap(),
                    )
                    .unwrap(),
                );
            }
            joint.peer.component.down(Time::ZERO, event, &mut joint.peer.out);
        }
        None => {
            let mut events = Queue::uncapped();
            workers.assignment(Time::ZERO, CHANNEL, assignment.clone(), &values, &mut events);
            let Some(root::Event::Worker(root::WorkerInput::Link {
                channel: CHANNEL,
                link: root::Link::Up { up: hub::LinkUp::Answer { answer, .. }, .. },
            })) = events.pop()
            else {
                panic!("encoding refusal reaches the root")
            };
            assert!(events.is_empty());
            joint.answers.push(answer).unwrap();
        }
    }
    joint.settle();
    let room = settings.jig.limits.io.output;
    let bytes = joint.peer.sent.checked_sub(sent).unwrap();
    let mut pings = (joint.peer.engine_pings, joint.peer.worker_pings);
    let ping_seconds = settings.jig.limits.link.ping.as_millis().checked_div(1000).unwrap();
    for second in 1..=SECONDS {
        joint.peer.now = Time::ZERO.checked_add(Duration::from_secs(second)).unwrap();
        joint.peer.component.fire(joint.peer.now, &mut joint.peer.out);
        joint.settle();
        if second.checked_rem(ping_seconds).unwrap() == 0 {
            assert!(joint.peer.engine_pings > pings.0, "{profile:?} {part:?}+{extra}: engine pings cross at {second}s");
            assert!(joint.peer.worker_pings > pings.1, "{profile:?} {part:?}+{extra}: worker pings cross at {second}s");
            pings = (joint.peer.engine_pings, joint.peer.worker_pings);
        }
    }
    std::eprintln!("{profile:?} {part:?}+{extra}: {outcome:?}; sent={bytes}; unsent={:?}; room={room}", joint.unsent);
    match outcome {
        Outcome::Crosses => {
            assert_eq!(joint.assignments.len(), 1, "{profile:?} {part:?}+{extra}");
            let kit::Event::Down { down: hub::LinkDown::Assign { assignment: mut expected }, .. } =
                workers.encoded_assignment(Time::ZERO, CHANNEL, assignment.clone(), &values).unwrap()
            else {
                panic!("assignment")
            };
            if let Some(workspace) = &mut expected.assignment.workspace {
                workspace.items = joint.assignments.get(0).unwrap().assignment.workspace.as_ref().unwrap().items;
            }
            assert_eq!(*joint.assignments.get(0).unwrap(), expected, "the whole assignment reaches the worker root");
            assert_eq!(joint.items.get(0), expected_items.as_ref(), "the whole workspace reaches the worker root");
            assert!(joint.answers.is_empty());
            assert!(joint.unsent.is_empty());
            assert!(bytes <= usize::try_from(room).unwrap());
        }
        Outcome::Past => {
            assert!(joint.assignments.is_empty());
            assert_eq!(bytes, 0, "a past-the-room frame is never sent");
            assert_eq!(joint.unsent.len(), 1, "{profile:?} {part:?}+{extra}: one unsent");
            let unsent = *joint.unsent.get(0).unwrap();
            assert_eq!(unsent.kind, Kind::Assignment);
            assert_eq!(unsent.bound, kit::Bound::Room(room));
            assert!(unsent.bytes > room);
            assert_refusal(
                joint.answers.as_slice(),
                format!("assignment of {} bytes past the link's room of {room} bytes", unsent.bytes).as_bytes(),
            );
        }
        Outcome::CannotEncode => {
            assert!(joint.assignments.is_empty(), "{profile:?} {part:?}+{extra}");
            assert!(joint.unsent.is_empty());
            assert_eq!(bytes, 0, "an unencodable assignment sends no frame");
            assert_refusal(joint.answers.as_slice(), b"assignment cannot encode");
        }
    }
    let mut small = fixtures::assignment_with_settings(fixtures::repository(8, b"acme", b"shop"), &settings);
    small.task = SMALL_RUN;
    small.workspace.repositories = Box::new([]);
    let event = workers.encoded_assignment(joint.peer.now, CHANNEL, small, &values).unwrap();
    joint.peer.component.down(joint.peer.now, event, &mut joint.peer.out);
    let previous = joint.assignments.len();
    joint.settle();
    assert_eq!(joint.assignments.len(), previous.checked_add(1).unwrap(), "the next run crosses");
    assert_eq!(joint.assignments.last().unwrap().assignment.run, Token::new(SMALL_RUN));
}

fn assert_refusal(answers: &[hub::RunAnswer], words: &[u8]) {
    assert_eq!(answers.len(), 1, "one refused answer, never silence");
    let answer = &answers[0];
    assert_eq!(
        answer.ending,
        hub::Ending::Refused(hub::Refusal::Invalid(hub::Invalid::Start {
            cause: hub::StartCause::Other,
            why: words.into(),
        }))
    );
    assert_eq!((answer.turns, answer.spent, answer.read), (0, 0, None));
    assert!(answer.refused.is_empty());
}

fn filled(bytes: usize) -> Box<[u8]> {
    let mut body = skein_lib::bytes::zeroed(bytes);
    body.fill(b'x');
    body
}

fn assignment(settings: &settings::Engine, workers: &Workers, part: Part, extra: usize) -> root::Assignment {
    let mut run = fixtures::assignment_with_settings(fixtures::repository(7, b"acme", b"shop"), settings);
    run.workspace.repositories = Box::new([]);
    match part {
        Part::Charter => charter(&mut run, workers, extra),
        Part::Window => window(&mut run, settings, extra),
        Part::Tree => tree(&mut run, &workers.encoding.limits, extra),
        Part::Inbox => inbox(&mut run, settings, extra),
        Part::Calls => calls(&mut run, settings, extra),
        Part::All => {
            charter(&mut run, workers, extra);
            window(&mut run, settings, extra);
            tree(&mut run, &workers.encoding.limits, extra);
            inbox(&mut run, settings, extra);
            calls(&mut run, settings, extra);
        }
    }
    run
}

fn charter(run: &mut root::Assignment, workers: &Workers, extra: usize) {
    run.run.policy.instructions = Box::new([]);
    run.sections = Box::new([root::BriefSection {
        kind: root::BriefKind::Core(jig_domain_brief::Core::Task),
        body: root::BriefBody::Text(SPECIFICATION.into()),
    }]);
    let charter = temper_engine_smith::charter(run);
    let bytes =
        smith_protocol_translate::encode_charter(&charter, &smith_charter::CEILINGS, &workers.encoding.endpoints)
            .unwrap();
    let fixed = charter.owned_bytes().unwrap().max(u64::try_from(bytes.len()).unwrap());
    let room = usize::try_from(workers.encoding.charter_bytes.checked_sub(fixed).unwrap()).unwrap();
    run.run.policy.instructions = filled(room.checked_add(extra).unwrap());
}
fn window(run: &mut root::Assignment, settings: &settings::Engine, extra: usize) {
    run.transcript = Box::new([filled(
        usize::try_from(settings.jig.limits.hub.transcript_bytes)
            .unwrap()
            .checked_sub(4)
            .unwrap()
            .checked_add(extra)
            .unwrap(),
    )]);
}
fn tree(run: &mut root::Assignment, limits: &payload::Limits, extra: usize) {
    run.run.policy.resume = true;
    let mut repositories = List::with_capacity(limits.items_repositories);
    for index in 0..limits.items_repositories {
        let mut repository = fixtures::repository(
            index.checked_add(1).unwrap(),
            &filled(usize::try_from(limits.repository_name).unwrap()),
            &filled(
                usize::try_from(limits.repository_name)
                    .unwrap()
                    .checked_add(if index == 0 { extra } else { 0 })
                    .unwrap(),
            ),
        );
        let mut heads = List::with_capacity(limits.merged_heads);
        for _ in 0..limits.merged_heads {
            heads.push([7; 32]).unwrap();
        }
        repository.start = root::ForgeStart::Merged {
            branch: filled(usize::try_from(limits.merged_branch).unwrap()),
            heads: heads.into_boxed(),
        };
        repository.push = Some(filled(usize::try_from(limits.writable_push).unwrap()));
        repository.landing = Some(filled(usize::try_from(limits.writable_landing_some).unwrap()));
        repository.save = Some(filled(usize::try_from(limits.repository_save_some).unwrap()));
        repositories.push(repository).unwrap();
    }
    run.workspace.repositories = repositories.into_boxed();
}
fn inbox(run: &mut root::Assignment, settings: &settings::Engine, extra: usize) {
    let mut messages = List::with_capacity(settings.jig.limits.core.tasks.run_inbox);
    for number in 0..settings.jig.limits.core.tasks.run_inbox {
        messages
            .push(tasks::Word {
                number: u64::from(number).checked_add(1).unwrap(),
                from: tasks::Party::Person(1),
                kind: tasks::MessageKind::Words,
                words: filled(
                    usize::try_from(settings.jig.limits.core.tasks.message_bytes)
                        .unwrap()
                        .checked_add(if number == 0 { extra } else { 0 })
                        .unwrap(),
                ),
                at: Wall::EPOCH,
                hits: 1,
                eligible: true,
            })
            .unwrap();
    }
    run.messages = messages.into_boxed();
}
fn calls(run: &mut root::Assignment, settings: &settings::Engine, extra: usize) {
    let mut calls = List::with_capacity(settings.jig.limits.core.call_records);
    for number in 0..settings.jig.limits.core.call_records {
        calls
            .push(core::Settled {
                key: core::CallKey {
                    task: run.task,
                    attempt: run.attempt,
                    completion: number.checked_add(1).unwrap(),
                    position: 0,
                },
                call: core::SettledCall {
                    serial: u64::from(number).checked_add(1).unwrap(),
                    tool: b"see".as_slice().into(),
                    answer: core::SettledAnswer::Host {
                        error: false,
                        body: filled(
                            usize::try_from(settings.jig.limits.core.call_answer_bytes)
                                .unwrap()
                                .checked_add(if number == 0 { extra } else { 0 })
                                .unwrap(),
                        ),
                    },
                },
            })
            .unwrap();
    }
    run.settled = calls.into_boxed();
}
