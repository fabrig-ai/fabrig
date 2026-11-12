//! Merge jobs in the whole worker world (temper/docs/design/domain/worker/jobs.md).

use jig_domain_link::{JobBound, JobKind};
use skein_lib::{Duration, Token};
use temper_fake_checkout::git::Remote;
use temper_worker_checkout_world::translate;
use temper_worker_domain::{Event, host, tooling, wire};

use super::jobs::{Report, World};

const RUN: Token = Token::new(81);
const ATTEMPT: Token = Token::new(13);
const REMOTE: &[u8] = b"org/jobs";
const SETTLED: &[u8] = b"{\"settled\":[\"code\"]}";
const LEFT: &[u8] = b"{\"settled\":[],\"left\":[\"code\"]}";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Case {
    Settled,
    Left,
    Marked,
}

fn kind() -> tooling::Kind {
    tooling::Kind {
        bound: JobBound { kind: JobKind(3), spec: 4_096, result: 512, deadline: Duration::from_secs(600) },
        verb: b"merge".as_slice().into(),
        delivers: true,
    }
}

/// A merge in progress, the scripted command, delivery, answer and teardown
/// (temper/docs/design/domain/worker/jobs.md).
#[must_use]
pub fn run(seed: u64, case: Case) -> Report {
    let mut limits = crate::limits();
    limits.checkout.conflicts = 1;
    limits.checkout.path_bytes = 64;
    let mut world = World::new(seed, kind(), limits);
    world.repository = b"fabrig".as_slice().into();
    let head = world.forge.advance(REMOTE, b"topic", b"code", b"head\n");
    let base = world.forge.advance(REMOTE, b"main", b"code", b"base\n");
    world.event(Event::Ready);
    assert_eq!(world.report.kinds.as_ref(), [kind().bound]);
    assign(&mut world, head, base);
    let owner = world.start(b"fabrig/tools/merge");
    command(&mut world, owner, head, base, case);
    let commit = world.forge.branch(REMOTE, b"topic").expect("delivery branch");
    let answer = world.report.answer.as_ref().expect("one terminal answer");
    match case {
        Case::Settled | Case::Left => {
            let host::JobEnding::Finished { result: received, work } = answer else { panic!("finished: {answer:?}") };
            assert_eq!(received.as_ref(), if case == Case::Left { LEFT } else { SETTLED });
            assert!(work.saved.is_none());
            assert_eq!(work.left.is_some(), case == Case::Settled);
        }
        Case::Marked => {
            let host::JobEnding::Failed { failure: host::JobFailure::Undelivered(_), work, .. } = answer else {
                panic!("marked file refuses delivery: {answer:?}");
            };
            assert_eq!(*work, host::Work { left: None, saved: None });
        }
    }
    if case == Case::Settled {
        assert_eq!(world.report.pushes, 1);
        assert_eq!(world.forge.parent(commit), Some(head));
        assert_eq!(world.forge.merge_parent(commit), Some(base));
    } else {
        assert_eq!(world.report.pushes, 0);
        assert_eq!(commit, head);
    }
    world.report.tree = world.forge.tree(commit);
    assert_eq!(world.domain.workspaces(), 0);
    assert_eq!(world.domain.agent().agents(), 0);
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }));
    assert_eq!(world.domain.kept_answers(), 0);
    world.event(Event::Shutdown { drain: true });
    assert_eq!(world.stops, 1);
    world.report
}

fn assign(world: &mut World, head: u64, base: u64) {
    world.event(Event::AssignJob {
        job: host::JobAssignment {
            run: RUN,
            attempt: ATTEMPT,
            kind: JobKind(3),
            spec: b"encoded spec".as_slice().into(),
            deadline: Duration::from_secs(5),
            grants: Box::new([]),
            workspace: Some(host::Workspace { workstream: 81, items: Token::new(0) }),
        },
        items: Ok(wire::Workspace {
            repositories: Box::new([wire::Repository {
                tag: 0,
                name: b"fabrig".as_slice().into(),
                remote: REMOTE.into(),
                save: None,
                git: None,
                start: wire::Start::Merge { branch: b"topic".as_slice().into(), base: translate::commit(base).raw() },
                access: wire::Access::Writable {
                    push: b"topic".as_slice().into(),
                    landing: None,
                    expected: Some(translate::commit(head).raw()),
                },
                identity: 7,
                account: None,
            }]),
        }),
        spec: Ok(wire::JobSpec {
            command: b"fabrig/tools/merge".as_slice().into(),
            request: b"request".as_slice().into(),
        }),
    });
}

fn command(world: &mut World, owner: Token, head: u64, base: u64, case: Case) {
    let workspace = world.workspace.expect("merge checkout prepared");
    let directory = translate::dir(workspace);
    let repository = [directory.as_slice(), b"/fabrig"].concat();
    let code = [repository.as_slice(), b"/code"].concat();
    let marked = world.disk.content(&code).expect("conflicting file").to_vec();
    assert!(marked.starts_with(b"<<<<<<<"));
    assert!(marked.windows(b"head\n".len()).any(|part| part == b"head\n"));
    assert!(marked.windows(b"base\n".len()).any(|part| part == b"base\n"));
    assert_eq!(
        world.disk.content(&[repository.as_slice(), b"/.git/MERGE_HEAD"].concat()),
        Some(base.to_le_bytes().as_slice())
    );
    assert_eq!(world.forge.branch(REMOTE, b"topic"), Some(head));
    if case == Case::Settled {
        // The scripted command edits the file and records its staged snapshot on the fake disk.
        world.disk.write(&code, b"settled\n");
        world.disk.write(&[repository.as_slice(), b"/.git/index"].concat(), b"code\0settled\n");
    }
    let result = if case == Case::Left { LEFT } else { SETTLED };
    world.disk.write(&[directory.as_slice(), b"/result.json"].concat(), result);
    world.event(Event::Job {
        owner,
        event: tooling::Event::Exited {
            owner,
            exit: tooling::Exit::Code(if case == Case::Left { 3 } else { 0 }),
            tail: Box::new([]),
        },
    });
    let (read_owner, op) = world.operations.pop_front().expect("read command result");
    assert_eq!(read_owner, owner);
    let tooling::Request::Read { owner: inner, path, most } = op else { panic!("read result") };
    assert_eq!(inner, owner);
    assert_eq!(path.as_ref(), format!("workspaces/{}/result.json", workspace.raw()).as_bytes());
    assert_eq!(most, 512);
    world.event(Event::Job { owner, event: tooling::Event::Read { owner, bytes: Some(result.into()) } });
    if case == Case::Settled {
        assert!(world.report.answer.is_none(), "push before answer");
        world.pushed(false);
    }
    assert!(world.report.answer.is_none(), "process gone before answer");
    world.event(Event::Job { owner, event: tooling::Event::Gone { owner } });
    assert!(world.operations.is_empty());
}
