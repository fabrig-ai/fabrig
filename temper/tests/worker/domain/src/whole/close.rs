//! Close jobs in the whole worker world (temper/docs/design/domain/worker/jobs.md).

use jig_domain_link::{JobBound, JobKind};
use skein_lib::{Duration, Token};
use temper_fake_checkout::git::Remote;
use temper_worker_checkout_world::translate;
use temper_worker_domain::{Event, host, tooling, wire};

use super::jobs::{Report, World};

const RUN: Token = Token::new(81);
const ATTEMPT: Token = Token::new(13);
const REMOTE: &[u8] = b"org/jobs";
const REQUEST: &[u8] = b"{\"version\":1,\"goal\":7,\"changes\":[{\"task\":9,\"landed\":true}]}";
const TITLE: &[u8] = b"Settle goal 7's status";
const SETTLED: &[u8] = b"{\"settled\":[\"docs/a.status.md\"]}";
const UNCHANGED: &[u8] = b"{\"settled\":[]}";
const CLOSE_HOOK: &[u8] = b"fabrig/.temper/hooks/close";
const MISSING: &[u8] = b"hook `.temper/hooks/close`: missing";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Case {
    Changed,
    Unchanged,
    Failed,
    MissingHook,
    NoHook,
}

fn kind() -> tooling::Kind {
    tooling::Kind {
        bound: JobBound { kind: JobKind(4), spec: 16_384, result: 512, deadline: Duration::from_secs(600) },
        verb: b"close".as_slice().into(),
        delivers: true,
    }
}

/// The goal's tip on the close branch, the command, delivery, answer and teardown
/// (temper/docs/design/domain/worker/jobs.md; temper/docs/design/protocol/tooling.md, 4.2).
pub fn run(seed: u64, case: Case) -> Result<Report, String> {
    let mut limits = crate::limits();
    limits.tooling.file_bytes = 16_384;
    if case == Case::MissingHook {
        limits.host.detail_bytes = u32::try_from(MISSING.len()).expect("room for the hook failure line");
    }
    let mut world = World::new(seed, kind(), limits);
    world.repository = b"fabrig".as_slice().into();
    world.request = REQUEST.into();
    world.title = TITLE.into();
    let head = world.forge.advance(REMOTE, b"topic", b"docs/a.status.md", b"in flight\n");
    world.forge.create_branch(REMOTE, b"goal", head).expect("goal branch");
    world.forge.advance(REMOTE, b"main", b"code", b"later main\n");
    world.event(Event::Ready)?;
    assert_eq!(world.report.kinds.as_ref(), [kind().bound]);
    let find = case == Case::NoHook;
    let program = if find || case == Case::MissingHook { CLOSE_HOOK } else { b"fabrig/tools/close" };
    assign(&mut world, head, program, find)?;
    if case == Case::MissingHook {
        missing_hook(&mut world)?;
    } else if find {
        world.absent(program, false)?;
    } else {
        let owner = world.start(program)?;
        command(&mut world, owner, head, case)?;
    }
    let commit = world.forge.branch(REMOTE, b"topic").expect("close branch");
    assert_eq!(world.forge.branch(REMOTE, b"goal"), Some(head));
    if case == Case::Changed {
        assert_eq!(world.report.pushes, 1);
        assert_eq!(world.forge.parent(commit), Some(head));
        assert_eq!(world.forge.merge_parent(commit), None);
        assert_eq!(world.forge.message(commit), TITLE);
    } else {
        assert_eq!(world.report.pushes, 0);
        assert_eq!(commit, head);
    }
    let answer = world.report.answer.as_ref().expect("one terminal answer");
    match case {
        Case::Changed | Case::Unchanged | Case::NoHook => {
            let host::JobEnding::Finished { result, work } = answer else { panic!("finished: {answer:?}") };
            let expected = match case {
                Case::Changed => SETTLED,
                Case::Unchanged => UNCHANGED,
                Case::NoHook => br#"{"settled":[],"hook":"none"}"#,
                Case::Failed | Case::MissingHook => unreachable!("the job finished"),
            };
            assert_eq!(result.as_ref(), expected);
            assert!(work.saved.is_none());
            assert_eq!(work.left.is_some(), case == Case::Changed);
        }
        Case::Failed | Case::MissingHook => {
            let host::JobEnding::Failed { failure, detail, work } = answer else { panic!("failed: {answer:?}") };
            assert_eq!(*failure, host::JobFailure::Failed);
            let expected = if case == Case::MissingHook { MISSING } else { b"exit 1: close failed" };
            assert_eq!(detail.as_ref(), expected);
            assert_eq!(*work, host::Work { left: None, saved: None });
        }
    }
    world.report.tree = world.forge.tree(commit);
    assert_eq!(world.domain.workspaces(), 0);
    assert_eq!(world.domain.agent().agents(), 0);
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }))?;
    assert_eq!(world.domain.kept_answers(), 0);
    world.event(Event::Shutdown { drain: true })?;
    assert_eq!(world.stops, 1);
    Ok(world.report)
}

fn assign(world: &mut World, head: u64, program: &[u8], find: bool) -> Result<(), String> {
    world.event(Event::AssignJob {
        job: host::JobAssignment {
            run: RUN,
            attempt: ATTEMPT,
            kind: JobKind(4),
            spec: b"encoded spec".as_slice().into(),
            deadline: Duration::from_secs(5),
            grants: Box::new([]),
            workspace: Some(host::Workspace { workstream: 81, items: Token::new(0) }),
        },
        items: Ok(wire::Workspace {
            rerun: None,
            hook_timeout_max_seconds: 3600,
            repositories: Box::new([wire::Repository {
                admit: false,
                tag: 0,
                name: b"fabrig".as_slice().into(),
                remote: REMOTE.into(),
                save: None,
                git: None,
                start: wire::Start::Branch { branch: b"topic".as_slice().into() },
                access: wire::Access::Writable {
                    push: b"topic".as_slice().into(),
                    landing: Some(b"goal".as_slice().into()),
                    expected: Some(translate::commit(head).raw()),
                    delivered: None,
                    pin: None,
                },
                identity: 7,
                account: None,
            }]),
        }),
        spec: Ok(wire::JobSpec {
            command: program.into(),
            request: REQUEST.into(),
            refusal: false,
            fallback: if find { Some(Box::new([])) } else { None },
        }),
    })?;
    Ok(())
}

fn missing_hook(world: &mut World) -> Result<(), String> {
    let owner = world.spawning(CLOSE_HOOK)?;
    world.event(Event::Job {
        owner,
        event: tooling::Event::Exited { owner, exit: tooling::Exit::NotRun, tail: MISSING.into() },
    })?;
    assert!(world.operations.is_empty(), "no result read after a failed check");
    assert!(world.report.answer.is_none(), "answer after both terminals");
    world.event(Event::Job { owner, event: tooling::Event::Gone { owner } })?;
    world.event(Event::Job { owner, event: tooling::Event::Gone { owner } })
}

fn command(world: &mut World, owner: Token, head: u64, case: Case) -> Result<(), String> {
    let workspace = world.workspace.expect("close checkout prepared");
    let directory = translate::dir(workspace);
    let repository = [directory.as_slice(), b"/fabrig"].concat();
    let status = [repository.as_slice(), b"/docs/a.status.md"].concat();
    assert_eq!(world.disk.content(&status), Some(b"in flight\n".as_slice()));
    assert_eq!(world.disk.content(&[repository.as_slice(), b"/code"].concat()), Some(b"original\n".as_slice()));
    assert!(!world.disk.exists(&[repository.as_slice(), b"/.git/MERGE_HEAD"].concat()));
    assert_eq!(world.forge.branch(REMOTE, b"topic"), Some(head));
    if case != Case::Unchanged {
        // The command stages its first edit, then leaves its final edit unstaged.
        world.disk.write(&status, b"staged\n");
        world.disk.write(&[repository.as_slice(), b"/.git/index"].concat(), b"docs/a.status.md\0staged\n");
        world.disk.write(&status, b"settled\n");
    }
    let result = if case == Case::Unchanged { UNCHANGED } else { SETTLED };
    world.disk.write(&[directory.as_slice(), b"/result.json"].concat(), result);
    world.event(Event::Job {
        owner,
        event: tooling::Event::Exited {
            owner,
            exit: tooling::Exit::Code(u8::from(case == Case::Failed)),
            tail: if case == Case::Failed { b"close failed".as_slice().into() } else { Box::new([]) },
        },
    })?;
    if case != Case::Failed {
        let (read_owner, op) = world.operations.pop_front().expect("read command result");
        assert_eq!(read_owner, owner);
        let tooling::Request::Read { owner: inner, path, most } = op else { panic!("read result") };
        assert_eq!(inner, owner);
        assert_eq!(path.as_ref(), format!("workspaces/{}/result.json", workspace.raw()).as_bytes());
        assert_eq!(most, 512);
        world.event(Event::Job { owner, event: tooling::Event::Read { owner, bytes: Some(result.into()) } })?;
        if world.report.pushes != 0 {
            assert!(world.report.answer.is_none(), "push before answer");
            world.pushed(false)?;
        }
    }
    assert!(world.report.answer.is_none(), "process gone before answer");
    world.event(Event::Job { owner, event: tooling::Event::Gone { owner } })?;
    assert!(world.operations.is_empty());
    Ok(())
}
