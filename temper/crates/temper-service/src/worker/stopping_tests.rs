//! Stop integration: real root, agent channel and git component; scripted IO.
#![expect(clippy::disallowed_methods, reason = "ordinary stop integration tests inspect scripted IO and owners")]
use super::*;
use crate::tests::fixtures;
use fixtures::worker_peers::{AGENT_CHILD, AGENT_ERROR, AGENT_INPUT, AGENT_OUTPUT, AgentPeer, LinkPeer, socket_stand};
use skein_io::kernel;
use skein_lib::Duration;
use temper_worker_protocol_git_world::io::Invocation;

fn prepared() -> (fixtures::WorkerStand, AgentPeer, LinkPeer) {
    let mut stand = socket_stand();
    let mut link = LinkPeer::new();
    link.open(&mut stand.worker);
    stand.worker.link(fixtures::NOW, fixtures::application_value(fixtures::IDENTITY, 1, b"test-token"));
    for capture in [
        "clone",
        "object-format",
        "init",
        "remote-add",
        "fetch-branch",
        "update-ref",
        "fetch-missing-branch",
        "update-ref",
        "checkout-1",
        "checkout-2",
    ] {
        let mut invocation = Invocation::captured(capture);
        if capture == "fetch-branch" {
            invocation.transcript.stdout = b"* 0000000000000000000000000000000000000000 0000000000000000000000000000000000000001 refs/temper/0/fetched\n".as_slice().into();
        }
        stand.below.script.invocations.push_back(invocation);
    }
    stand.worker.link(fixtures::NOW, fixtures::repository_assignment());
    let root::Event::Assign { assignment, items: Ok(mut items) } = stand.worker.engine.pop().unwrap() else {
        panic!("decoded repository")
    };
    items.repositories[0].start = root::wire::Start::Base { branch: b"main".as_slice().into() };
    items.repositories[0].access =
        root::wire::Access::Writable { push: b"topic".as_slice().into(), expected: None, landing: None };
    stand.worker.events.push(root::Event::Assign { assignment, items: Ok(items) });
    for _ in 0_u32..2000 {
        stand.worker.iterate(fixtures::NOW, Wall::EPOCH);
        if let Some((owner, _)) = stand.worker.processes.iter().next() {
            let owner = *owner;
            let peer = AgentPeer::launch(&mut stand.worker, &fixtures::worker_agent(), owner, true);
            return (stand, peer, link);
        }
        for _ in 0_u32..16 {
            if let Some(jig_protocol_engine_workers::Request::Up { up: hub::LinkUp::Answer { answer, .. }, .. }) =
                link.exchange(&mut stand.worker)
            {
                panic!("preparation answered before starting: {answer:?}");
            }
        }
        stand.drain();
        stand.input();
    }
    panic!("the repository is prepared and its agent starts");
}

fn drain_run(stand: &mut fixtures::WorkerStand, peer: &mut AgentPeer) {
    stand.worker.drain();
    peer.settle(&mut stand.worker);
    let Some(smith_protocol_channel::Heard::Drain) = peer.events.pop() else { panic!("the peer receives Drain") };
    assert!(peer.events.is_empty(), "not cancelled during its turn");
    let agent = fixtures::worker_agent();
    let body = fixtures::worker_peers::cancellation_turn(&agent);
    let turn = smith_protocol_channel::decode_turn(&body, 1, &agent.limits.transcript, &agent.endpoints).unwrap();
    peer.component
        .send_turn(1, None, smith_domain_run::Spend::ZERO, true, &turn, peer.owner, &mut peer.events, &mut peer.out)
        .unwrap();
    peer.component
        .send_answer(
            fixtures::NOW,
            smith_domain_run::Answer::Failed {
                failure: smith_domain_run::RunFailure::Cancelled,
                spent: smith_domain_run::Spend::ZERO,
                turns: 1,
            },
            None,
            peer.owner,
            &mut peer.events,
            &mut peer.out,
        )
        .unwrap();
    peer.settle(&mut stand.worker);
    peer.finished = true;
    peer.settle(&mut stand.worker);
    let owner = *stand.worker.owners.iter().find(|(_, agent)| **agent == peer.owner).unwrap().0;
    stand.worker.up(fixtures::NOW, io::Event::Exited { owner, exit: kernel::Exit::Code(0) });
    stand.worker.up(fixtures::NOW, io::Event::Closed { owner: AGENT_OUTPUT });
    stand.worker.up(fixtures::NOW, io::Event::Stream { owner: AGENT_ERROR, up: stream::Up::End });
    for capture in ["commit-add", "commit-changed", "commit-write-tree", "commit-tree-one-parent"] {
        stand.below.script.invocations.push_back(Invocation::captured(capture));
    }
}

fn close_agent(worker: &mut Worker, agent: Token) {
    for _ in 0..worker.io().len() {
        let request = worker.io().pop().unwrap();
        if let io::Request::Close { entity } | io::Request::Abort { entity } = request
            && [AGENT_CHILD, AGENT_INPUT, AGENT_OUTPUT, AGENT_ERROR].contains(&entity)
        {
            let owner = if entity == AGENT_CHILD {
                *worker.owners.iter().find(|(_, found)| **found == agent).unwrap().0
            } else {
                entity
            };
            worker.up(worker.env.now, io::Event::Closed { owner });
        } else {
            worker.io().push(request);
        }
    }
}

#[test]
fn a_drained_runs_save_and_push_complete_while_the_worker_stops() {
    let (mut stand, mut peer, mut link) = prepared();
    drain_run(&mut stand, &mut peer);
    let mut push = Invocation::captured("push-new-branch");
    push.transcript.stdout = b"To http://forge.example/app.git\n*\t0000000000000000000000000000000000000006:refs/heads/temper/saved-7\t[new branch]\nDone\n".as_slice().into();
    stand.below.script.invocations.push_back(push);
    let mut answer = None;
    for _ in 0_u32..2000 {
        stand.worker.iterate(fixtures::NOW, Wall::EPOCH);
        for _ in 0_u32..16 {
            if let Some(jig_protocol_engine_workers::Request::Up {
                up: hub::LinkUp::Answer { answer: got, .. },
                parts,
                ..
            }) = link.exchange(&mut stand.worker)
            {
                answer = Some((got, parts));
            }
        }
        if answer.is_some() {
            break;
        }
        close_agent(&mut stand.worker, peer.owner);
        stand.drain();
        stand.input();
    }
    let (answer, parts) = answer.expect("the saved run answers during shutdown");
    let hub::Ending::Failed { failure: hub::Failure::Cancelled(hub::Reason::Shutdown), .. } = answer.ending else {
        panic!("the run ends cancelled for shutdown")
    };
    let hub::Ending::Failed { work, .. } = answer.ending else { panic!("cancelled run") };
    let saved = work.saved.expect("saved-work part");
    let part = parts.iter().find(|part| part.token == saved).unwrap();
    let record = temper_protocol_worker_wire::v1::Saved::decode(
        &fixtures::wire_limits(),
        &mut skein_lib::Reader::new(&part.bytes),
    )
    .unwrap();
    let temper_protocol_worker_wire::v1::Landing::Landed(_) = record.repositories().get(0).unwrap().landing() else {
        panic!("the saved repository landed")
    };
    assert!(stand.worker.stopping());
}

fn save_push(stand: &mut fixtures::WorkerStand, peer: &AgentPeer, link: &mut LinkPeer) -> (Token, kernel::Spawn) {
    for _ in 0_u32..2000 {
        stand.worker.iterate(fixtures::NOW, Wall::EPOCH);
        close_agent(&mut stand.worker, peer.owner);
        for _ in 0_u32..16 {
            link.exchange(&mut stand.worker);
        }
        for _ in 0..stand.worker.io().len() {
            let request = stand.worker.io().pop().unwrap();
            if let io::Request::Spawn { owner, spawn } = request {
                if spawn.args.get(26).is_some_and(|arg| arg.as_ref() == b"push") {
                    return (owner, spawn);
                }
                stand.worker.io().push(io::Request::Spawn { owner, spawn });
            } else {
                stand.worker.io().push(request);
            }
        }
        stand.drain();
        stand.input();
    }
    panic!("the drained run pushes its saved work");
}

#[test]
fn the_stop_bound_aborts_a_save_push_that_never_answers_and_ends_the_worker() {
    let (mut stand, mut peer, mut link) = prepared();
    stand.worker.env.limits.stop_bound = Duration::from_secs(2);
    drain_run(&mut stand, &mut peer);
    let (owner, spawn) = save_push(&mut stand, &peer, &mut link);
    // The invocation exists but no output or exit reaches the worker.
    stand.worker.up(
        fixtures::NOW,
        io::Event::Spawned { owner, child: Token::new(901), pipes: Box::new([Token::new(902), Token::new(903)]) },
    );
    drop(spawn);
    let at = Time::ZERO.saturating_add(stand.worker.env.limits.stop_bound);
    stand.worker.iterate(Time::from_nanos(at.as_nanos() - 1), Wall::EPOCH);
    assert!(stand.worker.aborted.is_none(), "not before the stop bound");
    stand.worker.iterate(at, Wall::EPOCH);
    assert!(stand.worker.aborted.is_some(), "the bound abandons the hung push");
    assert_eq!(stand.worker.end(), Some(&crate::worker_alone::End::Aborted));
    let mut bound_notes = 0_u32;
    while let Some(note) = stand.worker.notes().pop() {
        if let crate::worker_alone::Note::StopBoundPassed { abandoned } = note {
            assert_eq!(abandoned, 1, "its last turn was not acknowledged");
            bound_notes += 1;
        }
    }
    assert_eq!(bound_notes, 1);
    let mut killed = false;
    let mut child_closed = false;
    for _ in 0_u32..1000 {
        for _ in 0..stand.worker.io().len() {
            let request = stand.worker.io().pop().unwrap();
            match request {
                io::Request::Signal { child, signal: kernel::Signal::Kill, to: kernel::Target::Group }
                    if child == Token::new(901) =>
                {
                    killed = true;
                }
                io::Request::Abort { entity } | io::Request::Close { entity }
                    if [Token::new(901), Token::new(902), Token::new(903)].contains(&entity) =>
                {
                    if entity == Token::new(901) {
                        killed = true; // IO's Abort kills the child's group.
                    } else {
                        stand.worker.up(at, io::Event::Closed { owner: entity });
                    }
                }
                io::Request::Close { .. } | io::Request::Abort { .. } => {
                    stand.worker.up(at, io::Event::Closed { owner: link.owner });
                }
                io::Request::Stream { stream, .. } | io::Request::Output { stream, .. }
                    if [Token::new(902), Token::new(903), Token::new(44)].contains(&stream) => {}
                other @ (io::Request::Listen { .. }
                | io::Request::Connect { .. }
                | io::Request::Bind { .. }
                | io::Request::Reject { .. }
                | io::Request::Stream { .. }
                | io::Request::Output { .. }
                | io::Request::Spawn { .. }
                | io::Request::SpawnAlone { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. }) => stand.below.out.io.push(other),
            }
        }
        if killed && !child_closed {
            child_closed = true;
            stand.worker.up(at, io::Event::Exited { owner, exit: kernel::Exit::Signal(9) });
            stand.worker.up(at, io::Event::Closed { owner });
        }
        stand.drain();
        stand.input();
        stand.worker.iterate(at, Wall::EPOCH);
        if stand.worker.is_empty() {
            break;
        }
    }
    assert!(killed, "the push's group is killed");
    assert!(stand.worker.is_empty(), "all children and the worker root have closed");
    // No second stop-bound line on another iteration.
    link.now = at;
    stand.worker.iterate(at, Wall::EPOCH);
    assert!(stand.worker.notes().is_empty());
}
