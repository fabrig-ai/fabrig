//! Parts cross the worker, and a drained attempt takes up its kept place
//! (temper/docs/design/draft/archive/worker-restarts.md, section 9, tests 1–3).
use super::{ATTEMPT, RUN, World, agent, host, protocol, wire};
use jig_domain_link::{KeptPart, Part, PartKind};
use skein_lib::{Duration, Token};
use smith_domain_session::{llm, record};
use temper_fake_checkout::git::Remote;
use temper_worker_domain::{Encodable, Encoded};

#[derive(Clone, Copy, Debug)]
pub enum InFlight {
    Answer,
    Check,
    Command,
}

fn part(number: u32, kind: record::PartKind, text: &[u8]) -> agent::channel::ToldPart {
    let mut body = protocol::session_turn(1, 0, 0, kind == record::PartKind::Answer);
    body.messages = Box::new([llm::Message {
        role: if kind == record::PartKind::Answer { llm::Role::Assistant } else { llm::Role::User },
        content: Box::new([if kind == record::PartKind::Answer {
            llm::Block::Text { text: text.into(), replay: None }
        } else {
            llm::Block::ToolResult {
                id: b"call-1".as_slice().into(),
                result: llm::Returned::Text { text: text.into(), error: true, replay: None },
            }
        }]),
    }]);
    agent::channel::ToldPart {
        number,
        turn: 1,
        thread: 0,
        call: u32::from(kind == record::PartKind::Result),
        kind,
        body: agent::TurnValue::new(body, u64::MAX).expect("bounded part"),
    }
}

fn kept(part: &Part, attempt: Token) -> KeptPart {
    KeptPart {
        attempt: attempt.raw(),
        turn: part.turn,
        thread: part.thread,
        call: part.call,
        kind: part.kind,
        body: part.body.clone(),
    }
}

fn cancel(world: &mut World) -> Result<(), String> {
    world.say(agent::Up::Answer {
        answer: agent::Answer {
            read: None,
            turns: 0,
            spent: 0,
            result: agent::RunResult::Failed { failure: agent::RunFailure::Cancelled },
        },
    })?;
    world.gone()
}

fn keep_part(world: &mut World, number: u32) -> Result<(), String> {
    world.event(super::Event::Link(host::LinkDown::PartKept { run: RUN, attempt: world.attempt, part: number }))
}

/// The second process owns fresh domain state but the same fake disk and forge.
fn restart(world: World, crashed: bool) -> World {
    if !crashed {
        assert_eq!(world.stops, 1);
        assert!(world.owed.is_empty() && world.out.is_empty());
    }
    let marks = world.marks.values().cloned().collect::<Vec<_>>().into_boxed_slice();
    for mark in &marks {
        assert!(world.disk.exists(format!("kept/{}", mark.workspace.raw()).as_bytes()));
    }
    let mut next = World::new(world.seed);
    next.forge = world.forge;
    next.disk = world.disk;
    next.marks = world.marks;
    next.first = world.first;
    next.env = world.env;
    next.domain = super::Domain::new(&next.env.limits, Box::new([]));
    next.domain.restore_workspaces(marks);
    next.attempt = Token::new(world.attempt.raw() + 1);
    next
}

pub fn relayed() -> Result<(), String> {
    let mut world = World::new(130);
    world.connect()?;
    world.assign_parts(wire::Start::Base { branch: b"main".as_slice().into() }, true, true, Box::new([]))?;
    world.say(agent::Up::Admitted)?;
    let told = part(1, record::PartKind::Answer, b"answer in flight");
    let expected = told.body.clone();
    world.say(agent::Up::Part { part: told })?;
    assert_eq!(world.part_acks, [1]);
    assert_eq!(world.parts.len(), 1);
    assert_eq!(world.parts[0].kind, PartKind::Answer);
    let decoded = smith_protocol_translate::decode_part(
        &world.parts[0].body,
        &smith_transcript::CEILINGS,
        &protocol::endpoints(),
    )
    .expect("relayed part decodes");
    assert_eq!(&decoded, expected.value());
    // A busy part and a lost link replay the same retained bytes.
    world.event(super::Event::ResendPart { run: RUN, attempt: ATTEMPT, part: 1 })?;
    assert_eq!(world.parts[0], world.parts[1]);
    world.up = false;
    world.event(super::Event::Lost)?;
    world.connect()?;
    assert_eq!(world.parts[0], world.parts[2]);
    keep_part(&mut world, 1)?;
    world.say(agent::Up::Turn { turn: protocol::turn(1, 0, 8) })?;
    world.say(agent::Up::Answer {
        answer: agent::Answer { read: None, turns: 1, spent: 0, result: protocol::report(b"done") },
    })?;
    world.gone()?;
    assert!(matches!(world.answer.as_ref().expect("answered").0.ending, host::Ending::Ended { .. }));
    let part_at = world.report.trace.iter().rposition(|line| line.contains("<- Link(Part {")).expect("part up");
    let answer_at = world.report.trace.iter().position(|line| line.contains("<- Answer {")).expect("answer up");
    assert!(part_at < answer_at, "the answer follows its parts");
    world.event(super::Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }))?;
    world.event(super::Event::Shutdown { drain: true })?;
    assert_eq!(world.stops, 0, "a turn still waits for the engine");
    world.event(super::Event::Link(host::LinkDown::TurnKept { run: RUN, attempt: ATTEMPT, turn: 1 }))?;
    assert_eq!(world.stops, 1);
    Ok(())
}

pub fn starts_and_graces() -> Result<(), String> {
    for link in [false, true] {
        for tell_parts in [false, true] {
            let mut world = World::new(131);
            world.env.limits.drain_grace = Duration::from_millis(73);
            let bodies = [part(1, record::PartKind::Answer, b"answer"), part(2, record::PartKind::Result, b"result")];
            let mut expected = Vec::new();
            let mut parts = Vec::new();
            for (index, told) in bodies.into_iter().enumerate() {
                let attempt = Token::new(7 + u64::try_from(index).expect("two parts"));
                expected.push(agent::channel::KeptPart {
                    activation: attempt.raw(),
                    turn: told.turn,
                    thread: told.thread,
                    call: told.call,
                    kind: told.kind,
                    body: told.body.clone(),
                });
                let Encoded::Part(encoded) = protocol::encode(Encodable::Part(told)) else { panic!("encoded part") };
                parts.push(kept(&encoded, attempt));
            }
            world.connect()?;
            world.assign_parts(
                wire::Start::Base { branch: b"main".as_slice().into() },
                true,
                tell_parts,
                parts.into_boxed_slice(),
            )?;
            let opening = world.opening.as_ref().expect("start");
            assert_eq!(opening.tell_parts, tell_parts);
            assert_eq!(opening.parts.as_ref(), if tell_parts { expected.as_slice() } else { &[] });
            world.say(agent::Up::Admitted)?;
            if link {
                world.event(super::Event::Link(host::LinkDown::Pause { mode: jig_domain_link::PauseMode::Drain }))?;
            } else {
                world.event(super::Event::Shutdown { drain: true })?;
            }
            assert_eq!(world.report.drains, 1);
            cancel(&mut world)?;
            world.event(super::Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }))?;
            if link {
                world.event(super::Event::Shutdown { drain: true })?;
            }
            assert_eq!(world.stops, 1);
        }
    }
    Ok(())
}

pub fn resumes(in_flight: InFlight) -> Result<(), String> {
    resume(in_flight, false)
}

#[derive(Clone, Copy)]
pub enum Fallback {
    RefusedSave,
    OtherWorkstream,
    DifferentRepositories,
}

pub fn fallback(reason: Fallback) -> Result<(), String> {
    let mut world = World::new(133);
    world.start()?;
    let path = temper_fake_checkout::layout::path(world.workspace.expect("prepared"), b"app");
    world.disk.write(&[path.as_slice(), b"/code"].concat(), b"unsaved edit\n");
    world.disk.write(&[path.as_slice(), b"/.git/target/output"].concat(), b"old build");
    if matches!(reason, Fallback::DifferentRepositories) {
        world.marks.values_mut().next().expect("held mark").repositories[0].remote =
            b"another/repository".as_slice().into();
    }
    let mut next = restart(world, true);
    if matches!(reason, Fallback::OtherWorkstream) {
        next.run = Token::new(82);
    }
    next.faults.refuse_push = matches!(reason, Fallback::RefusedSave);
    next.connect()?;
    assert_eq!(next.workstreams.as_ref(), &[super::WORKSTREAM]);
    next.assign(wire::Start::Branch { branch: b"topic".as_slice().into() })?;
    next.say(agent::Up::Admitted)?;
    let path = temper_fake_checkout::layout::path(next.workspace.expect("prepared"), b"app");
    assert_eq!(next.disk.tree(&path).get(b"code".as_slice()).expect("assigned tree"), b"original\n");
    assert!(!next.disk.exists(&[path.as_slice(), b"/.git/target/output"].concat()));
    assert!(next.report.trace.iter().any(|line| line.contains("op: Make {")));
    assert!(next.report.trace.iter().any(|line| line.contains("op: CheckOut {")));
    assert_eq!(next.report.saves, u32::from(matches!(reason, Fallback::RefusedSave)));
    assert_eq!(next.forge.branch(super::REMOTE, b"saved"), Some(next.first));
    next.finish()?;
    next.gone()?;
    Ok(())
}

pub fn crashes(in_flight: InFlight) -> Result<(), String> {
    resume(in_flight, true)
}

fn resume(in_flight: InFlight, crashed: bool) -> Result<(), String> {
    let mut world = World::new(132);
    // The answer and its interrupted call can be unacknowledged together.
    world.env.limits.host.turns = 2;
    world.env.limits.host.turn_bytes = 1024;
    world.env.limits.host.turn_queue_bytes = 2048;
    world.env.limits.agent.turns = 2;
    world.env.limits.agent.turn_bytes = 1024;
    world.env.limits.agent.unacknowledged_bytes = 2048;
    world.env.limits.bounds.unacknowledged = 2;
    assert!(super::valid(&world.env.limits));
    world.domain = super::Domain::new(&world.env.limits, Box::new([]));
    world.connect()?;
    world.assign_parts(wire::Start::Base { branch: b"main".as_slice().into() }, true, true, Box::new([]))?;
    world.say(agent::Up::Admitted)?;
    let directory = &world.opening.as_ref().expect("start").directories[0];
    let path = temper_fake_checkout::layout::path(directory.root, &directory.name);
    let results = [path.as_slice(), b"/.git/smith-check/passed"].concat();
    let output = [path.as_slice(), b"/.git/target/build-output"].concat();
    world.disk.write(&results, b"first check passed");
    world.disk.write(&output, b"untracked build");
    world.disk.write(&[path.as_slice(), b"/code"].concat(), b"saved edit\n");
    if !matches!(in_flight, InFlight::Answer) || crashed {
        world.say(agent::Up::Part { part: part(1, record::PartKind::Answer, b"answer in flight") })?;
    }
    if crashed && !matches!(in_flight, InFlight::Answer) {
        world.say(agent::Up::Part { part: part(2, record::PartKind::Result, b"previous result") })?;
    }
    let operation = match in_flight {
        InFlight::Answer => agent::Operation::Completion,
        InFlight::Check => agent::Operation::Checks,
        InFlight::Command => agent::Operation::Command,
    };
    world.say(agent::Up::Long { operation, span: Duration::from_secs(60) })?;
    if crashed {
        assert_eq!(world.report.saves, 0, "a crash does not settle");
        assert_eq!(world.stops, 0);
        let mark = world.marks.values().next().expect("held workspace");
        assert_eq!(mark.state, temper_worker_domain::checkout::git::KeptState::Held);
        assert_eq!(mark.repositories[0].commit, super::translate::commit(world.first));
    } else {
        drain(&mut world, in_flight)?;
    }
    let expected: Vec<_> = world.parts.iter().map(|part| kept(part, ATTEMPT)).collect();
    resumed(world, crashed, &path, &expected)
}

fn drain(world: &mut World, in_flight: InFlight) -> Result<(), String> {
    world.event(super::Event::Shutdown { drain: true })?;
    if matches!(in_flight, InFlight::Answer) {
        world.say(agent::Up::Part { part: part(1, record::PartKind::Answer, b"answer in flight") })?;
    } else {
        let text = match in_flight {
            InFlight::Answer => unreachable!(),
            InFlight::Check => b"delivery check interrupted by a worker restart".as_slice(),
            InFlight::Command => b"output so far; interrupted by a worker restart".as_slice(),
        };
        world.say(agent::Up::Part { part: part(2, record::PartKind::Result, text) })?;
    }
    let expected: Vec<_> = world.parts.iter().map(|part| kept(part, ATTEMPT)).collect();
    assert_eq!(world.part_acks.len(), expected.len());
    cancel(world)?;
    let (answer, work, _) = world.answer.take().expect("saved answer");
    assert!(matches!(
        answer.ending,
        host::Ending::Failed { failure: host::Failure::Cancelled(host::Reason::Shutdown), .. }
    ));
    assert!(work.saved.is_some());
    assert_eq!(world.report.kept.len(), 1);
    assert_eq!(world.report.saves, 1);
    assert_eq!(world.stops, 0, "unacknowledged parts and answer hold the worker");
    world.event(super::Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }))?;
    assert_eq!(world.stops, 0, "answer acknowledgement does not keep its parts");
    for number in 1..=u32::try_from(expected.len()).expect("part count") {
        keep_part(world, number)?;
    }
    assert_eq!(world.stops, 1);
    Ok(())
}

fn resumed(world: World, crashed: bool, path: &[u8], expected: &[KeptPart]) -> Result<(), String> {
    let mut next = restart(world, crashed);
    next.connect()?;
    assert_eq!(next.workstreams.as_ref(), &[super::WORKSTREAM]);
    next.assign_parts(
        if crashed {
            wire::Start::Branch { branch: b"topic".as_slice().into() }
        } else {
            wire::Start::Saved { branch: b"saved".as_slice().into() }
        },
        true,
        true,
        expected.to_vec().into_boxed_slice(),
    )?;
    assert!(next.reading, "resumed start: {:?}", next.report.trace);
    next.say(agent::Up::Admitted)?;
    let opening = next.opening.as_ref().expect("resumed start");
    assert!(opening.tell_parts);
    assert_eq!(opening.directories[0].root, next.workspace.expect("restored place"));
    assert_eq!(temper_fake_checkout::layout::path(opening.directories[0].root, &opening.directories[0].name), path);
    for (actual, expected) in opening.parts.iter().zip(expected) {
        assert_eq!(actual.activation, ATTEMPT.raw());
        let bytes = smith_protocol_translate::encode_part(
            actual.body.value(),
            &smith_transcript::CEILINGS,
            &protocol::endpoints(),
        )
        .expect("part");
        assert_eq!(bytes, expected.body);
    }
    assert_eq!(opening.parts.len(), expected.len());
    let tree = next.disk.tree(path);
    assert_eq!(tree.get(b".git/target/build-output".as_slice()).expect("build output"), b"untracked build");
    assert_eq!(tree.get(b".git/smith-check/passed".as_slice()).expect("check results"), b"first check passed");
    assert!(
        !next
            .report
            .trace
            .iter()
            .any(|line| line.contains("op: Make {") || line.contains("op: CheckOut {") || line.contains("op: Clone {")),
        "taking up the kept tree does not rebuild it"
    );
    let saved_head = next.forge.branch(super::REMOTE, b"saved").expect("saved branch");
    if crashed {
        assert_eq!(next.report.saves, 1, "one save before the agent starts");
        assert_eq!(next.pushes, 1);
        assert_eq!(next.forge.parent(saved_head), Some(next.first));
        assert_eq!(next.forge.tree(saved_head).get(b"code".as_slice()).expect("saved code"), b"saved edit\n");
    }
    next.finish()?;
    next.gone()?;
    if crashed {
        let (_, work, _) = next.answer.as_ref().expect("resumed answer");
        let [saved] = work.saved.as_deref().expect("take-up save in answer") else { panic!("one save") };
        assert_eq!(saved.landing, wire::Landing::Landed { commit: super::translate::commit(saved_head).raw() });
    }
    next.event(super::Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: next.attempt }))?;
    next.event(super::Event::Shutdown { drain: true })?;
    assert_eq!(next.stops, 1);
    Ok(())
}
