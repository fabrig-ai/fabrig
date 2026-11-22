//! Real agent restarts at the answer, check and command boundaries.
//! Contract: temper/docs/design/draft/worker-restarts.md, 9, focused 1–3.
use crate::{
    fixtures,
    processes::Proc,
    world::{Store, Views},
};
use jig_domain_core as core;
use jig_protocol_worker_wire::{self as wire, Record};
use skein_io::kernel;
use skein_lib::{Queue, Time, Token, Wall};
use skein_shell::Host;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Point {
    Answer,
    Check,
    Command,
}

pub(crate) const PATH: &[u8] = b"/restart-workspace";
const CHECK: &[u8] = b".smith/check";
const OUTPUT: &[u8] = b"command output before the cut\n";
const HOLD: &[u8] = b"cut-answer";
const GRACE: skein_lib::Duration = skein_lib::Duration::from_secs(1);

#[derive(Clone, Debug, Default)]
#[expect(clippy::struct_excessive_bools, reason = "independent restart observations")]
pub struct Evidence {
    pub kept: Vec<core::PartRecord>,
    pub resumed: Vec<wire::KeptPart>,
    pub retired_at_turn: bool,
    pub suspended: bool,
    pub same_worker: bool,
    pub queries: usize,
    pub outputs: Vec<String>,
    pub checks: usize,
    pub commands: usize,
    pub interrupted_output: bool,
    pub delivered: bool,
    pub committed_at_answer: bool,
    pub same_tree: bool,
    pub answer_after_drain: bool,
    pub drain_to_end: Option<skein_lib::Duration>,
    pub acknowledged_parts: usize,
}

#[derive(Default)]
pub(crate) struct State {
    pub evidence: Evidence,
    pub drain: Option<Time>,
    pub release_at: Option<Time>,
    released: bool,
    wire_seen: usize,
    saw_resumed_turn: bool,
    check_tree: Option<Vec<u8>>,
}

pub(crate) fn engine(inherited: &skein_world::Inherited) -> Proc {
    static PREPARED: std::sync::OnceLock<jig_test_service::Prepared> = std::sync::OnceLock::new();
    let prepared = PREPARED.get_or_init(|| {
        let mut document: serde_json::Value =
            serde_json::from_slice(&fixtures::engine_document()).expect("engine JSON");
        let charter = &mut document["engine"]["charters"][0];
        for permission in ["inspect", "modify", "shell"] {
            charter[permission] = true.into();
        }
        jig_test_service::prepare(&serde_json::to_vec(&document).expect("restart JSON")).expect("restart engine")
    });
    crate::processes::engine_document(inherited, prepared)
}

pub(crate) fn worker(root: kernel::Fd) -> Proc {
    static PREPARED: std::sync::OnceLock<jig_test_service::Prepared> = std::sync::OnceLock::new();
    let prepared = PREPARED.get_or_init(|| {
        let mut document: serde_json::Value = serde_json::from_slice(fixtures::WORKER_DOCUMENT).expect("worker JSON");
        document["workers"][0]["graces"]["drain_grace_ms"] = (GRACE.as_nanos() / 1_000_000).into();
        jig_test_service::prepare(&serde_json::to_vec(&document).expect("restart JSON")).expect("restart worker")
    });
    let mut proc =
        crate::processes::worker_document(root, false, false, false, crate::processes::Backend::Resume, prepared);
    let Proc::Worker(host) = &mut proc else { unreachable!("worker") };
    host.service.worker(0).expect("one worker").configure_workspace(
        Box::new([smith_host_domain::Directory {
            root: Token::new(1),
            name: b"workspace".as_slice().into(),
            host: PATH.into(),
            writable: true,
            git: false,
            conflicts: Box::default(),
            results: Some(b"/restart-workspace/results".as_slice().into()),
        }]),
        Some(smith_domain::run::outcome::ChangeSpec {
            checks_must_pass: true,
            fields: Box::new([
                smith_domain::run::outcome::FieldRule { name: b"title".as_slice().into(), max: 1024 },
                smith_domain::run::outcome::FieldRule { name: b"body".as_slice().into(), max: 1024 },
            ]),
        }),
    );
    proc
}

pub(crate) fn worker_restarting(inherited: &skein_world::Inherited) -> Proc {
    let root = inherited.roots.iter().find(|(name, _)| name.as_ref() == b"worker").expect("worker root").1;
    let mut proc = worker(root);
    let Proc::Worker(host) = &mut proc else { unreachable!("worker") };
    host.service.adopt_signals(inherited.signal).expect("worker signal");
    host.restart_root =
        Some(inherited.roots.iter().find(|(name, _)| name.as_ref() == b"replacement").expect("replacement").1);
    proc
}

pub(crate) fn agent(_: &kernel::Spawn, inherited: &skein_world::Inherited) -> Proc {
    let mut config = fixtures::agent_configuration();
    config.limits.domain.run.check_timeout = skein_lib::Duration::from_secs(4);
    let fd = inherited.pipes.iter().find(|(child, _)| *child == 2).expect("stderr").1;
    let (errors, capture) = smith_agent_process_world::AgentErrors::pipe(fd);
    let agent = smith_agent_process_world::configured(config, fixtures::SEED, inherited, errors.clone());
    Proc::Agent { host: Box::new(capture.host(agent)), errors }
}

pub(crate) fn roots(_: &kernel::Spawn) -> Vec<skein_world::StartupRoot> {
    vec![skein_world::StartupRoot { name: b"workspace".as_slice().into(), path: PATH.into() }]
}
pub(crate) fn command_roots(spawn: &kernel::Spawn) -> Vec<skein_world::StartupRoot> {
    vec![skein_world::StartupRoot {
        name: b"command".as_slice().into(),
        path: if is_check(spawn) { b"restart-check".as_slice().into() } else { b"restart-command".as_slice().into() },
    }]
}
fn is_check(spawn: &kernel::Spawn) -> bool {
    spawn.args.iter().any(|arg| arg.windows(CHECK.len()).any(|piece| piece == CHECK))
}
pub(crate) fn files() -> Vec<skein_fake_machine::Item> {
    vec![
        skein_fake_machine::Item::directory(b".smith"),
        skein_fake_machine::Item::file(CHECK, b"#!/bin/sh\n").mode(0o700),
        skein_fake_machine::Item::file(b"tree.txt", b"same tree across the restart\n"),
        skein_fake_machine::Item::directory(b"results"),
        skein_fake_machine::Item::file(b"results/check-started", b"0"),
    ]
}

pub(crate) fn turns(point: Point, result: Box<[u8]>) -> Box<[skein_fake_llm_domain::api::Turn]> {
    use skein_fake_llm_domain::api::{Finish, Line, Turn};
    let (name, arguments): (&[u8], &[u8]) = if point == Point::Command {
        (b"shell", br#"{"command":"long command","timeout_seconds":30}"#)
    } else {
        (b"deliver", br#"{"fields":{"title":"cut answer","body":"keep this tree"}}"#)
    };
    let mut lines = Vec::new();
    if point == Point::Answer {
        lines.push(Line::Pieces {
            pieces: Box::new([skein_fake_llm_domain::api::Piece {
                text: b"Held answer".as_slice().into(),
                hold: Some(HOLD.into()),
            }]),
        });
    }
    lines.push(Line::Call { name: name.into(), arguments: arguments.into() });
    Box::new([
        Turn {
            lines: Box::new([Line::Text { text: b"Before the cut".as_slice().into() }]),
            finish: Finish::Stop,
            tokens: 1,
        },
        Turn { lines: lines.into_boxed_slice(), finish: Finish::ToolCalls, tokens: 1 },
        Turn {
            lines: Box::new([Line::Call { name: b"finish".as_slice().into(), arguments: result }]),
            finish: Finish::ToolCalls,
            tokens: 1,
        },
    ])
}

/// Observe a check opening the same fake tree; its marker stays on that tree.
pub(crate) fn check_opened(
    state: &mut State,
    files: &mut skein_fake_machine::Machine,
    root: skein_fake_machine::Opened,
) {
    let file = files.open(root, b"tree.txt", skein_fake_machine::How::ReadNoFollow).expect("check's tree");
    let tree = files.read(file, 0, 64).expect("check reads tree");
    files.close(file);
    state.evidence.checks += 1;
    if let Some(previous) = &state.check_tree {
        state.evidence.same_tree = previous == &tree;
    } else {
        state.check_tree = Some(tree);
    }
}

pub(crate) fn act(point: Point, now: Time, procs: &mut [Proc], views: &mut Views) {
    if point == Point::Answer && views.restart.drain.is_some() && !views.restart.released {
        let draining = crate::world::check_worker_wire(procs)
            .expect("worker wire")
            .frames
            .iter()
            .any(|(_, kind, _)| *kind == wire::Kind::Draining.number());
        if draining && views.restart.release_at.is_none() {
            views.restart.release_at = Some(now.saturating_add(skein_lib::Duration::from_millis(10)));
        }
        if views.restart.release_at.is_some_and(|at| at <= now) {
            for proc in procs.iter_mut() {
                if let Proc::Peer(peer) = proc {
                    peer.release(HOLD).expect("release held answer");
                }
            }
            views.restart.released = true;
            views.restart.release_at = None;
        }
    }
    if views.restart.drain.is_some() {
        return;
    }
    let ready = match point {
        Point::Answer => procs.iter().any(|proc| matches!(proc, Proc::Peer(peer) if peer.is_held(HOLD))),
        Point::Check => views.restart.evidence.checks == 1,
        Point::Command => views.restart.evidence.commands == 1,
    };
    if ready {
        views.restart.drain = Some(now);
        for proc in procs {
            if let Proc::Worker(host) = proc {
                host.service.shutdown();
                host.stopping = true;
            }
        }
    }
}

pub(crate) fn observe(now: Time, procs: &[Proc], store: &Store, state: &mut State) -> Result<(), String> {
    let worker = crate::world::check_worker_wire(procs)?;
    if let Some((_, _, bytes)) = worker.frames.iter().find(|(_, kind, _)| *kind == 3) {
        return Err(format!("restart worker refused: {bytes:?}"));
    }
    for row in store.rows.values() {
        if let jig_test_domain::Record::Core(core::Record::Core(core::CoreRecord::Part(part))) = row
            && part.key.attempt == 1
            && part.turn == 2
            && !state.evidence.kept.iter().any(|known| known.key == part.key)
        {
            state.evidence.kept.push(part.clone());
        }
    }
    state.evidence.suspended |= store.rows.values().any(|row| matches!(row,
        jig_test_domain::Record::Core(core::Record::Core(core::CoreRecord::Terminal(end)))
        if end.attempt == 1 && matches!(end.end, jig_domain_tasks::End::Suspended(jig_domain_tasks::Suspended {worker: 1, why: jig_domain_tasks::SuspendWhy::Stopped, ..}))));
    let assignments = worker
        .frames
        .iter()
        .filter(|(_, kind, _)| *kind == wire::Kind::Assignment.number())
        .map(|(_, _, bytes)| {
            wire::Assignment::decode(&wire::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("assignment")
        })
        .collect::<Vec<_>>();
    if let Some(resumed) = assignments.get(1) {
        state.evidence.resumed =
            wire::parts::decode(&resumed.parts, &wire::CEILINGS).expect("assignment parts").into_boxed().into_vec();
        state.evidence.same_worker |= store.rows.values().any(|row| {
            matches!(row,
            jig_test_domain::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof)))
            if proof.attempt == 2 && proof.worker == Some(1))
        });
    }
    for turn in crate::world::turns(store) {
        if turn.attempt == 2 && turn.turn == 1 && !state.saw_resumed_turn {
            state.saw_resumed_turn = true;
            state.evidence.retired_at_turn = !store.rows.values().any(|row| matches!(row, jig_test_domain::Record::Core(core::Record::Core(core::CoreRecord::Part(part))) if part.key.attempt == 1));
            let record = smith_transcript::Turn::decode(
                &smith_transcript::CEILINGS,
                &mut skein_lib::Reader::new(&turn.transcript),
            )
            .expect("committed turn");
            let answer = state
                .evidence
                .kept
                .iter()
                .find(|part| part.kind == jig_domain_link::PartKind::Answer)
                .expect("cut answer kept");
            let cut =
                smith_transcript::Turn::decode(&smith_transcript::CEILINGS, &mut skein_lib::Reader::new(&answer.body))
                    .expect("answer part");
            let calls: Vec<_> = cut
                .messages
                .iter()
                .flat_map(|message| &message.blocks)
                .filter_map(|block| if let smith_transcript::Block::Call(call) = block { Some(&call.id) } else { None })
                .collect();
            let results: Vec<_> =
                record
                    .messages
                    .iter()
                    .flat_map(|message| &message.blocks)
                    .filter_map(|block| {
                        if let smith_transcript::Block::Result(result) = block { Some(&result.id) } else { None }
                    })
                    .collect();
            state.evidence.committed_at_answer = record.place == cut.place
                && record.messages.as_slice().starts_with(cut.messages.as_slice())
                && !calls.is_empty()
                && calls == results;
        }
    }
    observe_wire(now, procs, store, state)?;
    for proc in procs {
        if let Proc::Peer(peer) = proc {
            state.evidence.queries = smith_agent_process_world::fake::queries(peer).count();
            for query in smith_agent_process_world::fake::queries(peer) {
                for part in query.messages.iter().flat_map(|message| &message.parts) {
                    if let skein_fake_llm_domain::api::Part::ToolOutput { output, .. } = part {
                        let text = String::from_utf8_lossy(output);
                        if !state.evidence.outputs.iter().any(|seen| seen == text.as_ref()) {
                            state.evidence.outputs.push(text.to_string());
                        }
                        state.evidence.interrupted_output |= text.contains("interrupted by a worker restart after")
                            && text.contains(std::str::from_utf8(OUTPUT).expect("fixture text").trim());
                        state.evidence.delivered |= text.contains("delivered") || text.contains("Delivered");
                    }
                }
            }
        }
    }

    if state.evidence.queries >= 3 && state.evidence.outputs.is_empty() {
        return Err("the resumed provider request replanned the cut answer instead of carrying its call result".into());
    }
    Ok(())
}

fn observe_wire(now: Time, procs: &[Proc], store: &Store, state: &mut State) -> Result<(), String> {
    let worker = crate::world::check_worker_wire(procs)?;
    let engine = procs
        .iter()
        .find_map(|proc| if let Proc::Engine(host) = proc { Some(&host.wire) } else { None })
        .ok_or("engine wire")?;
    for (at, kind, bytes) in worker.frames.iter().skip(state.wire_seen) {
        if *kind == wire::Kind::Part.number() {
            let part =
                wire::v22::Part::decode(&wire::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("part frame");
            if part.attempt == 1 && part.turn == 2 && part.kind == wire::PartKind::Answer {
                state.evidence.answer_after_drain = state.drain.is_some_and(|drain| *at > drain);
            }
        } else if *kind == wire::Kind::PartKept.number() {
            let kept = wire::PartKept::decode(&wire::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("part kept");
            let part = engine
                .frames
                .iter()
                .filter(|(_, kind, _)| *kind == wire::Kind::Part.number())
                .find_map(|(_, _, bytes)| {
                    let part = wire::v22::Part::decode(&wire::CEILINGS, &mut skein_lib::Reader::new(bytes))
                        .expect("received part");
                    ((part.task, part.attempt, part.number) == (kept.task, kept.attempt, kept.number)).then_some(part)
                })
                .ok_or("part kept follows a received part")?;
            let key = core::PartKey { task: kept.task, attempt: kept.attempt, number: kept.number };
            let retained = store.rows.get(&jig_test_domain::Key::Core(core::Key::Core(core::CoreKey::Part(key))));
            if let Some(jig_test_domain::Record::Core(core::Record::Core(core::CoreRecord::Part(retained)))) = retained
            {
                let kind = match part.kind {
                    wire::PartKind::Answer => jig_domain_link::PartKind::Answer,
                    wire::PartKind::Result => jig_domain_link::PartKind::Result,
                    wire::PartKind::Window => jig_domain_link::PartKind::Window,
                };
                if retained.body != part.body
                    || retained.turn != part.turn
                    || retained.thread != part.thread
                    || retained.call != part.call
                    || retained.kind != kind
                {
                    return Err("acknowledged part is kept unchanged".into());
                }
            } else if !crate::world::turns(store).iter().any(|turn| {
                turn.task == kept.task
                    && (turn.attempt > kept.attempt || turn.attempt == kept.attempt && turn.turn >= part.turn)
            }) {
                return Err(format!("part acknowledged before durable storage at {now:?}: {key:?}"));
            }
            state.evidence.acknowledged_parts += 1;
        } else if *kind == wire::Kind::Answer.number() {
            let answer = wire::Answer::decode(&wire::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("answer");
            if answer.attempt == 1 {
                state.evidence.drain_to_end = state.drain.map(|drain| at.saturating_since(drain));
            }
        }
    }
    state.wire_seen = worker.frames.len();
    Ok(())
}

/// A long fake command writes once; a check marks its tree and passes on its second start.
pub(crate) fn command(spawn: &kernel::Spawn, inherited: &skein_world::Inherited) -> Proc {
    let output = inherited.pipes.iter().find(|(child, _)| *child == 1).expect("stdout").1;
    let mut host = Command {
        submissions: Queue::uncapped(),
        completions: Queue::uncapped(),
        descriptors: inherited
            .pipes
            .iter()
            .map(|(_, fd)| *fd)
            .chain(inherited.roots.iter().map(|(_, fd)| *fd))
            .chain([inherited.signal])
            .collect(),
        signal: inherited.signal,
        closing: false,
        left: 0,
    };
    if is_check(spawn) {
        assert!(
            spawn.env.iter().any(|entry| entry.as_ref() == b"SMITH_CHECK_RESULTS=/restart-workspace/results"),
            "checks reuse their results directory"
        );
        host.submit(
            1,
            kernel::Op::Open {
                root: inherited.roots[0].1,
                path: b"results/check-started".as_slice().into(),
                how: kernel::OpenHow::WriteNoFollow,
            },
        );
    } else {
        host.submit(1, kernel::Op::PipeWrite { fd: output, bytes: OUTPUT.into(), from: 0 });
        host.listen();
    }
    Proc::Auth(Box::new(host))
}
struct Command {
    submissions: Queue<kernel::Submit>,
    completions: Queue<kernel::Complete>,
    descriptors: Vec<kernel::Fd>,
    signal: kernel::Fd,
    closing: bool,
    left: usize,
}
impl Command {
    fn submit(&mut self, number: u64, kind: kernel::Op) {
        self.submissions.push(kernel::Submit { op: Token::new(number), kind });
    }
    fn listen(&mut self) {
        self.submit(2, kernel::Op::ReadSignal { fd: self.signal });
    }
    fn close(&mut self) {
        self.closing = true;
        self.left = self.descriptors.len();
        for (at, fd) in self.descriptors.clone().into_iter().enumerate() {
            self.submit(10 + u64::try_from(at).expect("descriptor"), kernel::Op::Close { fd });
        }
    }
}
impl Host for Command {
    fn iterate(&mut self, _: Time, _: Wall) {
        while let Some(complete) = self.completions.pop() {
            match complete.kind {
                kernel::Op::Open { .. } => {
                    let Ok(kernel::Done::Fd(fd)) = complete.result else { panic!("check marker opens") };
                    self.descriptors.push(fd);
                    self.submit(3, kernel::Op::Read { fd, buf: vec![0; 1].into_boxed_slice(), at: 0 });
                }
                kernel::Op::Read { fd, buf, .. } => {
                    assert_eq!(complete.result, Ok(kernel::Done::Count(1)));
                    if buf[0] == b'1' {
                        self.close();
                    } else {
                        self.submit(4, kernel::Op::Write { fd, bytes: b"1".as_slice().into(), from: 0, at: 0 });
                    }
                }
                kernel::Op::Write { .. } => {
                    assert_eq!(complete.result, Ok(kernel::Done::Count(1)));
                    self.listen();
                }
                kernel::Op::PipeWrite { fd, bytes, from } => {
                    if let Ok(kernel::Done::Count(count)) = complete.result {
                        let from = from + count;
                        if usize::try_from(from).expect("short output") < bytes.len() {
                            self.submit(1, kernel::Op::PipeWrite { fd, bytes, from });
                        }
                    }
                }
                kernel::Op::ReadSignal { .. } => self.close(),
                kernel::Op::Close { .. } => self.left -= 1,
                _ => panic!("command completion"),
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _: Time) -> bool {
        !self.completions.is_empty() || !self.submissions.is_empty()
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.closing && self.left == 0 && self.submissions.is_empty()
    }
    fn operations(&self) -> u32 {
        8
    }
}
