//! Real agent restarts at the answer, check and command boundaries.
//! Contract: protocol/channel/records.md, 5; smith's domain/turn-parts.md, 7.
use crate::{
    fixtures,
    processes::Proc,
    world::{Store, Views},
};
use jig_domain_core as core;
use jig_protocol_worker_wire::{self as wire, Record};
use skein_io::kernel;
use skein_lib::{Duration, Queue, Time, Token, Wall};
use skein_shell::Host;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Point {
    Answer,
    Check,
    Command,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashScope {
    Unit,
    Worker,
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
    pub suspension_cause: Option<jig_domain_tasks::SuspendWhy>,
    pub fresh_hello_at: Option<Time>,
    pub suspended_at: Option<Time>,
    pub unknown_time: bool,
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
    pub orphan_exit: Option<OrphanExit>,
}

#[derive(Clone, Copy, Debug)]
pub struct OrphanExit {
    pub cut_at: Time,
    pub exited_at: Time,
    pub exit: kernel::Exit,
    pub bound: Duration,
}

struct Orphan {
    process: usize,
    output: kernel::Fd,
    bound: Duration,
    wrote_to_host: bool,
}

#[derive(Default)]
pub struct State {
    pub evidence: Evidence,
    pub drain: Option<Time>,
    pub release_at: Option<Time>,
    released: bool,
    wire_seen: usize,
    assignments_seen: usize,
    assignment_frames_seen: usize,
    observations_seen: usize,
    saw_resumed_turn: bool,
    check_tree: Option<Vec<u8>>,
    pub(crate) check_marked: bool,
    pub(crate) killed_at: Option<Time>,
    pub(crate) killed_agents: usize,
    orphan: Option<Orphan>,
    pub(crate) saved_wire: Option<crate::processes::Wire>,
}

impl State {
    pub(crate) fn orphan_deadline(&self) -> Option<Time> {
        let orphan = self.orphan.as_ref()?;
        self.evidence.orphan_exit.is_none().then(|| self.killed_at.expect("worker cut").saturating_add(orphan.bound))
    }

    // With the host gone, a receiver could not distinguish a suppressed answer
    // from a failed write. Watch the orphan's stdout submissions instead.
    pub(crate) fn observe_orphan_submissions(&mut self, process: usize, submissions: &Queue<kernel::Submit>) {
        if let Some(orphan) = &mut self.orphan
            && orphan.process == process
        {
            orphan.wrote_to_host |= submissions.iter().any(|submit| {
                matches!(&submit.kind, kernel::Op::Write { fd, .. } | kernel::Op::PipeWrite { fd, .. } if *fd == orphan.output)
            });
        }
    }

    fn observe_orphan(&mut self, now: Time, procs: &[Proc]) -> Result<(), String> {
        let Some(orphan) = &self.orphan else { return Ok(()) };
        if orphan.wrote_to_host {
            return Err("the orphan wrote to its host after the worker was killed".into());
        }
        if self.evidence.orphan_exit.is_none() {
            let Some(Proc::Agent { host, .. }) = procs.get(orphan.process) else {
                return Err("the orphan was removed before exiting on its own".into());
            };
            if let Some(exit) = host.exit() {
                self.evidence.orphan_exit = Some(OrphanExit {
                    cut_at: self.killed_at.expect("worker cut"),
                    exited_at: now,
                    exit,
                    bound: orphan.bound,
                });
            }
        }
        Ok(())
    }

    /// Worker frames are append-only, including across the worker's replacement.
    /// Decode each assignment once, retaining the second one's carried parts.
    pub fn observe_assignments(&mut self, frames: &[(Time, u16, Box<[u8]>)]) -> Result<(), String> {
        for (_, kind, bytes) in &frames[self.assignment_frames_seen..] {
            if *kind == 3 {
                return Err(format!("restart worker refused: {bytes:?}"));
            }
            if *kind == wire::Kind::Assignment.number() {
                let assignment =
                    wire::Assignment::decode(&wire::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("assignment");
                self.assignments_seen += 1;
                if self.assignments_seen == 2 {
                    self.evidence.resumed = wire::parts::decode(&assignment.parts, &wire::CEILINGS)
                        .expect("assignment parts")
                        .into_boxed()
                        .into_vec();
                }
            }
        }
        self.assignment_frames_seen = frames.len();
        Ok(())
    }

    /// The fake provider keeps its observations after either worker exits.
    /// Only newly decoded requests can contribute new outputs.
    pub fn observe_queries(&mut self, observations: &[skein_fake_peers::llm::Observation]) -> Result<(), String> {
        for observation in &observations[self.observations_seen..] {
            if let skein_fake_peers::llm::Observation::Query { query, .. } = observation {
                self.evidence.queries += 1;
                for part in query.messages.iter().flat_map(|message| &message.parts) {
                    if let skein_fake_llm_domain::api::Part::ToolOutput { output, .. } = part {
                        let text = String::from_utf8_lossy(output);
                        if !self.evidence.outputs.iter().any(|seen| seen == text.as_ref()) {
                            self.evidence.outputs.push(text.to_string());
                        }
                        self.evidence.interrupted_output |= text.contains("interrupted by a worker restart after")
                            && text.contains(std::str::from_utf8(OUTPUT).expect("fixture text").trim());
                        self.evidence.unknown_time |=
                            text.contains("interrupted by a worker restart after an unknown time");
                        self.evidence.delivered |= text.contains("delivered") || text.contains("Delivered");
                    }
                }
            }
        }
        self.observations_seen = observations.len();
        if self.killed_at.is_none() && self.evidence.queries >= 3 && self.evidence.outputs.is_empty() {
            return Err(
                "the resumed provider request replanned the cut answer instead of carrying its call result".into()
            );
        }
        Ok(())
    }
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
    host.restart_root = inherited.roots.iter().find(|(name, _)| name.as_ref() == b"replacement").map(|(_, fd)| *fd);
    proc
}

fn agent_configuration() -> smith_agent_service::Config {
    let mut config = fixtures::agent_configuration();
    config.limits.domain.run.check_timeout = Duration::from_secs(4);
    config
}

fn orphan_exit_bound() -> Duration {
    let config = agent_configuration();
    let limits =
        smith_agent_profile::host_limits(&smith_agent_process_world::profile(), &config.limits, &config.domain.limits)
            .expect("the restart agent's host limits derive");
    smith_agent_profile::orphan_exit(&limits).expect("the restart agent's orphan exit derives")
}

pub(crate) fn agent(_: &kernel::Spawn, inherited: &skein_world::Inherited) -> Proc {
    let config = agent_configuration();
    let fd = inherited.pipes.iter().find(|(child, _)| *child == 2).expect("stderr").1;
    let (errors, capture) = smith_agent_process_world::AgentErrors::pipe(fd);
    let agent = smith_agent_process_world::configured(config, fixtures::SEED, inherited, errors.clone());
    Proc::Agent {
        host: Box::new(capture.host(agent)),
        errors,
        output: inherited.pipes.iter().find(|(child, _)| *child == 1).expect("stdout").1,
    }
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
pub(crate) fn files(first_check_passes: bool) -> Vec<skein_fake_machine::Item> {
    vec![
        skein_fake_machine::Item::directory(b".smith"),
        skein_fake_machine::Item::file(CHECK, b"#!/bin/sh\n").mode(0o700),
        skein_fake_machine::Item::file(b"tree.txt", b"same tree across the restart\n"),
        skein_fake_machine::Item::directory(b"results"),
        skein_fake_machine::Item::file(b"results/check-started", if first_check_passes { b"1" } else { b"0" }),
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
    let ready = ready(point, procs, &views.restart);
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

fn ready(point: Point, procs: &[Proc], state: &State) -> bool {
    match point {
        Point::Answer => procs.iter().any(|proc| matches!(proc, Proc::Peer(peer) if peer.is_held(HOLD))),
        Point::Check => state.evidence.checks == 1,
        Point::Command => state.evidence.commands == 1,
    }
}

/// Kill without a drain or settle. Only the worker starts again through
/// its registered factory, whether or not its unit was killed with it.
pub(crate) fn crash(
    point: Point,
    scope: CrashScope,
    now: Time,
    procs: &mut [Proc],
    state: &mut State,
    controls: &skein_world::Controls,
) {
    if let Some(previous) = state.saved_wire.take() {
        let host = procs
            .iter_mut()
            .find_map(|proc| if let Proc::Worker(host) = proc { Some(host) } else { None })
            .expect("worker");
        host.wire.preceded_by(previous);
    }
    if state.killed_at.is_some() {
        // The provider scripts by conversation position, so the abandoned turn
        // is asked again. Release only after the resumed query has arrived
        // and the orphan (if any) has cancelled its own completion.
        if point == Point::Answer && state.evidence.queries >= 3 && state.orphan_deadline().is_none() {
            for proc in procs {
                if let Proc::Peer(peer) = proc
                    && peer.is_held(HOLD)
                {
                    peer.release(HOLD).expect("release resumed answer");
                }
            }
        }
        return;
    }
    if !ready(point, procs, state) || point == Point::Check && !state.check_marked {
        return;
    }
    let worker_at = procs.iter().position(|proc| matches!(proc, Proc::Worker(_))).expect("worker");
    let Proc::Worker(host) = &mut procs[worker_at] else { unreachable!("worker") };
    let sent = host.wire.frames.iter().filter(|(_, kind, _)| *kind == wire::Kind::Part.number()).count();
    if sent != state.evidence.acknowledged_parts
        || point == Point::Check && state.evidence.kept.len() != 1
        || point == Point::Command && state.evidence.kept.len() != 2
    {
        return;
    }
    state.killed_at = Some(now);
    state.saved_wire = Some(std::mem::take(&mut host.wire));
    match scope {
        CrashScope::Unit => {
            state.killed_agents = procs.iter().filter(|proc| matches!(proc, Proc::Agent { .. })).count();
            controls.kill_unit(worker_at);
        }
        CrashScope::Worker => {
            let mut agents = procs.iter().enumerate().filter_map(|(process, proc)| match proc {
                Proc::Agent { host, output, .. } if host.exit().is_none() => Some((process, *output)),
                _ => None,
            });
            let (process, output) = agents.next().expect("the worker's agent is still running at the cut");
            assert!(agents.next().is_none(), "there is just one agent to orphan");
            state.orphan = Some(Orphan { process, output, bound: orphan_exit_bound(), wrote_to_host: false });
            controls.cut(worker_at, skein_world::Cut::Kill);
        }
    }
}

pub(crate) fn observe(now: Time, procs: &[Proc], store: &Store, state: &mut State) -> Result<(), String> {
    state.observe_orphan(now, procs)?;
    let worker = crate::world::check_worker_wire(procs)?;
    state.observe_assignments(&worker.frames)?;
    for part in store.parts() {
        if part.key.attempt == 1 && part.turn == 2 && !state.evidence.kept.iter().any(|known| known.key == part.key) {
            state.evidence.kept.push(part.clone());
        }
    }
    for end in store.terminals() {
        if end.attempt == 1
            && let jig_domain_tasks::End::Suspended(suspension) = end.end
            && suspension.worker == 1
        {
            state.evidence.suspended |= suspension.why
                == if state.killed_at.is_some() {
                    jig_domain_tasks::SuspendWhy::Restarted
                } else {
                    jig_domain_tasks::SuspendWhy::Stopped
                };
            state.evidence.suspension_cause = Some(suspension.why);
            state.evidence.suspended_at.get_or_insert(now);
        }
    }
    if state.killed_at.is_some() && state.evidence.fresh_hello_at.is_none() {
        let engine = procs
            .iter()
            .find_map(|proc| if let Proc::Engine(host) = proc { Some(host) } else { None })
            .expect("engine");
        for (at, kind, bytes) in &engine.wire.frames {
            if *kind == wire::Kind::Hello.number() && Some(*at) >= state.killed_at {
                let hello =
                    wire::Hello::decode(&wire::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("worker hello");
                if hello.fresh {
                    state.evidence.fresh_hello_at = Some(*at);
                    break;
                }
            }
        }
    }
    if state.assignments_seen >= 2 {
        state.evidence.same_worker |=
            store.core_rows(core::CoreKey::RunProof(0), core::CoreKey::RunProof(u64::MAX)).any(
                |row| matches!(row, core::CoreRecord::RunProof(proof) if proof.attempt == 2 && proof.worker == Some(1)),
            );
    }
    for turn in store.turns() {
        if turn.attempt == 2 && turn.turn == 1 && !state.saw_resumed_turn {
            state.saw_resumed_turn = true;
            state.evidence.retired_at_turn = !store.parts().any(|part| part.key.attempt == 1);
            let record = smith_transcript::Turn::decode(
                &smith_transcript::CEILINGS,
                &mut skein_lib::Reader::new(&turn.transcript),
            )
            .expect("committed turn");
            let answer = state.evidence.kept.iter().find(|part| part.kind == jig_domain_link::PartKind::Answer);
            let Some(answer) = answer else {
                if state.killed_at.is_some() && state.evidence.kept.is_empty() {
                    continue;
                }
                return Err("the drained cut answer was kept".into());
            };
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
            state.observe_queries(peer.observations())?;
        }
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
            } else if !store.turns().any(|turn| {
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
