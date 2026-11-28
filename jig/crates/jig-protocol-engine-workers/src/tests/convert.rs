//! Written passes at the real workers host, with scripted channel peers.
use super::{fixtures, room};
use crate::{
    Event, Loss,
    host::{Output, Seen, Tokens, Workers},
};
use alloc::boxed::Box;
use jig_domain_host as hub;
use jig_protocol_worker_wire::{self as wire, Record};
use skein_channel::{Frame, Role};
use skein_codec::{
    layout::{Mode, Refusal},
    schema::parse,
};
use skein_io as io;
use skein_lib::{Intake, List, Reader, Time, Token, stream};

const APP: &str = "test-worker-wire";
const OTHER: &str = "family test-worker-wire 3\nrecord Test { value: text 16 note: u8 = 0 }\n";
const A: Token = Token::new(1);
const B: Token = Token::new(2);

struct World {
    host: Workers,
    links: List<Link>,
    out: List<Output>,
}

struct Link {
    channel: Token,
    socket: Token,
    owner: Option<Token>,
    peer: skein_fake_channel::ScriptedPeer,
    input: Intake,
    read: stream::Read,
}

impl World {
    fn new() -> Self {
        let limits = fixtures::LIMITS;
        let worker = parse(OTHER.as_bytes()).expect("different worker layout");
        let root = worker.root(b"Test", None).expect("worker's root");
        let written = wire::Written {
            wire: APP,
            declaration: "Test",
            layout: worker.layout(root, &[]).expect("layout").digest(),
        };
        let mut application = wire::Application::opaque(fixtures::application());
        application.written.push(written).expect("one written entry");
        for carried in [wire::Carried::Items, wire::Carried::Grant, wire::Carried::Left, wire::Carried::JobResult] {
            application.roots.push(wire::Root { carried, wire: APP, declaration: "Test" }).expect("one root per part");
        }
        let tokens = Tokens { listeners: 1, connections: 2 };
        let config = room::config();
        let mut host = Workers::new(&config, tokens, application, &limits).expect("workers host");
        let io::Request::Listen { owner, addr } = host.io.pop().expect("listener") else { panic!("listen") };
        let schema = crate::limits::schema(&limits).expect("channel schema");
        let peer_limits = crate::limits::channel(&limits).expect("channel limits");
        let mut links = List::with_capacity(2);
        for (channel, socket, name, secret) in [
            (A, Token::new(52), fixtures::WORKER_A, fixtures::SECRET_A),
            (B, Token::new(53), fixtures::WORKER_B, fixtures::SECRET_B),
        ] {
            let mut wires = fixtures::statement(fixtures::APPLICATION_VERSION);
            let app = wires.get_mut(4).expect("application is after jig's four");
            app.layout = worker.fingerprints().family().digest();
            app.text = OTHER.as_bytes().into();
            let credential = fixtures::credential_with(name, secret, wires);
            let mut peer =
                skein_fake_channel::ScriptedPeer::new(schema.clone(), Role::Initiator, peer_limits, credential)
                    .expect("scripted worker");
            peer.play(frame(wire::Kind::Hello, &fixtures::hello_record().encode(&wire::CEILINGS).expect("hello")));
            links
                .push(Link {
                    channel,
                    socket,
                    owner: None,
                    peer,
                    input: Intake::with_capacity(limits.room),
                    read: stream::Read::Nothing,
                })
                .ok()
                .expect("two peers");
        }
        let mut world = Self { host, links, out: List::uncapped() };
        world.down(Event::Workers(fixtures::workers()));
        for socket in [Token::new(52), Token::new(53)] {
            world.up(io::Event::Accepted { owner, socket, peer: addr });
            world.pump();
        }
        let mut opened = List::with_capacity(2);
        for output in &world.out {
            if let Output::Hello { channel, .. } = output {
                opened.push(*channel).expect("one hello per link");
            }
        }
        assert_eq!(opened.as_slice(), &[A, B]);
        assert!(world.host.converting().is_none());
        world.out.clear();
        world
    }

    fn down(&mut self, event: Event) {
        self.host.down(Time::ZERO, event);
    }

    fn up(&mut self, event: io::Event) {
        assert_eq!(self.host.up(Time::ZERO, event), Seen::Taken);
    }

    fn link(&mut self, channel: Token) -> &mut Link {
        self.links.iter_mut().find(|link| link.channel == channel).expect("test channel")
    }

    fn pump(&mut self) {
        for _ in 0_u32..2048 {
            if self.host.pending() > 0 {
                if let Some(output) = self.host.output() {
                    self.out.push(output).expect("captured outputs");
                }
                continue;
            }
            if let Some(request) = self.host.io.pop() {
                match request {
                    io::Request::Bind { socket, owner } => self.socket(socket).owner = Some(owner),
                    io::Request::Stream { stream, down: stream::Down::Demand { read, .. } } => {
                        self.socket(stream).read = read;
                    }
                    io::Request::Stream { down: stream::Down::Finish, .. } => {}
                    io::Request::Output { stream, down: stream::OutputDown::Room { right, .. } } => {
                        let owner = self.socket(stream).owner.expect("bound stream");
                        self.up(io::Event::Output {
                            owner,
                            up: stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted },
                        });
                    }
                    io::Request::Output { stream, down: stream::OutputDown::Send { bytes, .. } } => {
                        self.socket(stream).peer.feed(&bytes).expect("valid engine frame");
                    }
                    io::Request::Abort { entity } | io::Request::Close { entity } => {
                        let channel = self.socket(entity).channel;
                        self.gone(channel);
                    }
                    request => panic!("unexpected IO: {request:?}"),
                }
                continue;
            }
            let mut progress = false;
            for link in self.links.iter_mut() {
                let Some(owner) = link.owner else { continue };
                if let Some(bytes) = link.peer.pop_output() {
                    link.input.append(&bytes).expect("channel input room");
                    progress = true;
                }
                if let Some(bytes) = link.input.meet(link.read) {
                    link.read = stream::Read::Nothing;
                    assert_eq!(
                        self.host.up(Time::ZERO, io::Event::Stream { owner, up: stream::Up::Bytes(bytes) }),
                        Seen::Taken
                    );
                    progress = true;
                }
            }
            if !progress {
                return;
            }
        }
        panic!("channel steps settle")
    }

    fn socket(&mut self, socket: Token) -> &mut Link {
        self.links.iter_mut().find(|link| link.socket == socket).expect("test socket")
    }

    fn gone(&mut self, channel: Token) {
        let owner = self.link(channel).owner.take().expect("bound stream");
        self.up(io::Event::Closed { owner });
    }

    fn answer(&mut self, bytes: &[u8]) {
        self.host.converted(Time::ZERO, Ok(bytes.into()));
        self.pump();
    }

    fn ask(&self, mode: Mode, expected: &[u8], bound: u32) {
        let given = self.host.converting().expect("one value waits");
        assert_eq!((given.wire, given.mode, given.value, given.bound), (APP, mode, expected, bound));
        let (from, to) = match mode {
            Mode::Keep => (fixtures::application().get(0).expect("engine wire").text, OTHER),
            Mode::Read => (OTHER, fixtures::application().get(0).expect("engine wire").text),
        };
        for (side, text) in [(given.from, from), (given.to, to)] {
            assert_eq!(side.text.text(), text.as_bytes());
            assert_eq!(side.text.root(b"Test", None), Some(side.root));
            assert!(side.held.is_empty());
        }
    }

    fn ping_for_task(&mut self, channel: Token, number: u64) {
        self.down(Event::Down {
            channel,
            down: hub::LinkDown::Acknowledge { run: Token::new(number), attempt: A },
            parts: Box::new([]),
            values: Box::new([]),
        });
    }

    fn received(&mut self, channel: Token, kind: wire::Kind, body: &[u8]) {
        self.link(channel).peer.play(frame(kind, body));
        self.pump();
    }

    fn job_answer(&mut self, channel: Token, left: Option<&[u8]>, result: &[u8]) {
        let record = wire::JobAnswer {
            task: channel.raw(),
            attempt: 1,
            ending: wire::JobEnding::Finished(wire::JobFinished {
                result: result.into(),
                work: wire::Work { left: left.map(Box::from), saved: None },
            }),
        };
        self.received(channel, wire::Kind::JobAnswer, &record.encode(&wire::CEILINGS).expect("job answer"));
    }
}

fn frame(kind: wire::Kind, bytes: &[u8]) -> Frame {
    let mut frame =
        skein_channel::frame_writer(kind.number(), u32::try_from(bytes.len()).expect("test body")).expect("frame");
    frame.put(bytes).expect("measured body");
    frame.finish().expect("measured frame")
}

fn refusal() -> Refusal {
    Refusal { declaration: "Child".into(), part: "n".into(), why: "invalid decimal".into() }
}

#[test]
fn holds_each_written_part_until_its_owner_answers_then_sends_the_frame() {
    let mut world = World::new();
    world.down(fixtures::assignment_down(A));
    world.pump();
    let limits = *world.host.component.codec();
    let [Output::Convert] = world.out.as_slice() else { panic!("unexpected outputs: {:?}", world.out) };
    world.ask(Mode::Keep, b"items", limits.workspace_items);
    assert_eq!(world.link(A).peer.observed(), []);
    world.answer(b"translated items");
    let [Output::Convert, Output::Convert] = world.out.as_slice() else {
        panic!("unexpected outputs: {:?}", world.out)
    };
    let values = fixtures::values();
    let Some(crate::GrantValue::Application { secret }) = values.get(1).expect("second grant") else {
        panic!("application grant")
    };
    world.ask(Mode::Keep, secret.expose(), limits.application_value_secret);
    assert_eq!(world.link(A).peer.observed(), []);
    world.answer(b"translated grant");
    assert!(world.host.converting().is_none());
    let [sent] = world.link(A).peer.observed() else { panic!("one assignment was sent") };
    assert_eq!(sent.kind, wire::Kind::Assignment.number());
    let record = wire::Assignment::decode(&limits, &mut Reader::new(&sent.body)).expect("converted assignment");
    assert_eq!(&*record.workspace.expect("workspace").items, b"translated items");
    let wire::GrantValue::Application(value) = &record.grants.get(1).expect("second grant").value else {
        panic!("application grant")
    };
    assert_eq!(value.secret.expose(), b"translated grant");
}

#[test]
fn written_reads_stop_their_channel_and_resume_oldest_first_while_other_work_goes_on() {
    let mut world = World::new();
    let limits = *world.host.component.codec();
    world.job_answer(A, Some(b"left"), b"first result");
    world.ask(Mode::Read, b"left", limits.work_left_some);
    let [Output::Convert] = world.out.as_slice() else { panic!("unexpected outputs: {:?}", world.out) };
    let fact = fixtures::fact_record().encode(&wire::CEILINGS).expect("fact");
    world.received(A, wire::Kind::Fact, &fact);
    assert_eq!(world.link(A).read, stream::Read::Nothing);
    world.ping_for_task(A, 31);
    world.ping_for_task(A, 32);
    world.ping_for_task(B, 33);
    world.pump();
    assert_eq!(world.link(A).peer.observed(), []);
    assert_eq!(world.link(B).peer.observed().len(), 1, "another channel sends while A is held");
    world.job_answer(B, None, b"second result");
    world.ask(Mode::Read, b"left", limits.work_left_some);
    let [Output::Convert] = world.out.as_slice() else { panic!("unexpected outputs: {:?}", world.out) };
    world.answer(b"engine left");
    world.ask(Mode::Read, b"second result", limits.job_finished_result);
    world.answer(b"other result");
    world.ask(Mode::Read, b"first result", limits.job_finished_result);
    assert_eq!(world.link(A).peer.observed(), []);
    world.answer(b"engine result");
    let frame = |index: usize| &world.links.get(0).expect("first link").peer.observed()[index];
    assert_eq!(world.links.get(0).expect("first link").peer.observed().len(), 2);
    for (index, task) in [(0_usize, 31_u64), (1, 32)] {
        assert_eq!(frame(index).kind, wire::Kind::Acknowledge.number());
        let read = wire::Acknowledge::decode(&limits, &mut Reader::new(&frame(index).body)).expect("acknowledge");
        assert_eq!(read.task, task, "queued sends remain in order");
    }
    assert!(world.host.converting().is_none());
    let mut ups = world.out.iter().filter_map(|entry| match entry {
        Output::Up { channel, up, parts } => Some((*channel, up, parts)),
        _ => None,
    });
    let (B, hub::LinkUp::JobAnswer { answer: hub::JobEnding::Finished { result, .. }, .. }, _) =
        ups.next().expect("second channel answers")
    else {
        panic!("B's answer")
    };
    assert_eq!(&**result, b"other result");
    let (A, hub::LinkUp::JobAnswer { answer: hub::JobEnding::Finished { result, work }, .. }, parts) =
        ups.next().expect("first channel answers")
    else {
        panic!("A's answer")
    };
    assert_eq!(&**result, b"engine result");
    let [left] = parts.as_ref() else { panic!("one work part") };
    assert_eq!(work.left, Some(left.token));
    assert_eq!(&*left.bytes, b"engine left");
    let Some((A, hub::LinkUp::Activity { .. }, _)) = ups.next() else { panic!("reading resumes after A's frame") };
    assert!(ups.next().is_none());
}

#[test]
fn while_a_send_waits_no_more_read_chunks_are_demanded() {
    let mut world = World::new();
    world.down(fixtures::assignment_down(A));
    world.pump();
    // One already issued read can complete while the send is held. The rest
    // of this frame must not be demanded yet.
    let fact = fixtures::fact_record().encode(&wire::CEILINGS).expect("fact");
    world.received(A, wire::Kind::Fact, &fact);
    let [Output::Convert] = world.out.as_slice() else { panic!("unexpected outputs: {:?}", world.out) };
    assert_eq!(world.link(A).read, stream::Read::Nothing);
    world.answer(b"items");
    let [Output::Convert, Output::Convert] = world.out.as_slice() else {
        panic!("unexpected outputs: {:?}", world.out)
    };
    world.answer(b"grant");
    let [Output::Convert, Output::Convert, Output::Up { channel: A, .. }] = world.out.as_slice() else {
        panic!("unexpected outputs: {:?}", world.out)
    };
}

#[test]
fn one_past_the_held_output_frames_is_full_and_a_late_answer_does_nothing() {
    let mut world = World::new();
    world.job_answer(A, None, b"gone");
    let [Output::Convert] = world.out.as_slice() else { panic!("unexpected outputs: {:?}", world.out) };
    let cap = crate::limits::channel(&fixtures::LIMITS).expect("limits").output_frames;
    for task in 1..=cap {
        world.ping_for_task(A, u64::from(task));
    }
    world.pump();
    let [Output::Convert] = world.out.as_slice() else { panic!("all output slots can wait: {:?}", world.out) };
    world.job_answer(B, None, b"still waiting");
    world.ping_for_task(A, u64::from(cap).checked_add(1).expect("one over cap"));
    world.pump();
    let [Output::Convert, Output::Ended { channel: Some(A), why: Loss::Full, .. }, Output::Lost { channel: A }] =
        world.out.as_slice()
    else {
        panic!("one past the waiting slots loses the channel: {:?}", world.out);
    };
    assert!(world.host.converting().is_none(), "the lost channel no longer lends a value");
    assert_eq!(world.link(A).peer.observed(), []);
    world.out.clear();
    world.answer(b"discarded");
    let [Output::Convert] = world.out.as_slice() else { panic!("the answer to A was not used for B: {:?}", world.out) };
    world.answer(b"other lives");
    let [Output::Convert, Output::Up { channel: B, .. }] = world.out.as_slice() else {
        panic!("unexpected outputs: {:?}", world.out)
    };
}

#[test]
fn written_refusals_take_the_start_running_and_read_refusal_routes() {
    let mut world = World::new();
    world.down(fixtures::assignment_down(A));
    world.pump();
    world.host.converted(Time::ZERO, Err(refusal()));
    world.pump();
    let words = b"the worker's layout cannot keep test-worker-wire Test Child.n: invalid decimal";
    let [Output::Convert, Output::Unsent { unsent, .. }, Output::Up { up: hub::LinkUp::Answer { answer, .. }, .. }] =
        world.out.as_slice()
    else {
        panic!("untranslated start: {:?}", world.out)
    };
    assert!(!unsent.mismatches());
    assert_eq!(answer.ending, hub::Ending::Refused(hub::Refusal::Untranslated(words.as_slice().into())));
    assert_eq!(world.link(A).peer.observed(), []);
    world.out.clear();
    world.down(Event::Down {
        channel: A,
        down: hub::LinkDown::Grant { run: A, attempt: A, grant: fixtures::grant(jig_domain_link::Form::Application) },
        parts: Box::new([]),
        values: Box::new([Some(crate::GrantValue::Application {
            secret: skein_lib::Secret::new(b"renewed".as_slice().into()),
        })]),
    });
    world.pump();
    world.host.converted(Time::ZERO, Err(refusal()));
    world.pump();
    let [Output::Convert, Output::Unsent { unsent, .. }] = world.out.as_slice() else {
        panic!("refused renewal: {:?}", world.out)
    };
    assert!(unsent.mismatches());
    let crate::Bound::Translation(problem) = &unsent.bound else { panic!("converter refusal") };
    assert_eq!(&*problem.kept(), words);
    assert_eq!(world.link(A).peer.observed(), []);
    world.out.clear();
    world.job_answer(B, None, b"unreadable");
    world.host.converted(Time::ZERO, Err(refusal()));
    world.pump();
    let [
        Output::Convert,
        Output::Unread { kind: wire::Kind::JobAnswer, problem, .. },
        Output::Ended { channel: Some(B), .. },
        Output::Lost { channel: B },
    ] = world.out.as_slice()
    else {
        panic!("undecodable body: {:?}", world.out)
    };
    assert_eq!(&*problem.read(), b"test-worker-wire Test Child.n: invalid decimal");
}
