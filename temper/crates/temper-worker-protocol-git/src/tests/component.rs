use super::fixtures;
use crate::{Event, Request, Unusable, Version};
use skein_io::kernel;
use temper_worker_domain_checkout::git;
const FETCHED: &[u8] =
    b"* 0000000000000000000000000000000000000000 abababababababababababababababababababab refs/temper/7/fetched\n";

#[test]
fn every_push_fault_keeps_gits_stderr_tail() {
    use git::{Fault, Missing};
    for (stderr, fault) in [
        (b"fatal: unknown push failure\n".as_slice(), Fault::Broken),
        (b"fatal: Authentication failed for 'remote'\n".as_slice(), Fault::Refused),
        (b"fatal: unable to access 'remote': Could not resolve host\n".as_slice(), Fault::Unreachable),
        (b"fatal: repository 'remote' not found\n".as_slice(), Fault::Missing { missing: Missing::Repository }),
    ] {
        let mut component = fixtures::Started::new();
        component.down(Request::Io {
            owner: fixtures::OWNER,
            op: fixtures::push_op(None),
            deadline: fixtures::DEADLINE,
            identity: Some(fixtures::identity()),
        });
        let (child, pipes) = component.spawned();
        component.stdout(pipes[1], &[b'x'; 600]);
        component.stdout(pipes[1], b"\n");
        component.stdout(pipes[1], stderr);
        component.end(pipes[0]);
        component.end(pipes[1]);
        component.exited(child, 128);
        component.closed(child, &pipes);
        let (_, git::Done::FailedWithOutput { fault: actual, diagnostic }) = component.done().expect("terminal") else {
            panic!("push failure carries output");
        };
        assert_eq!(actual, fault);
        assert!(diagnostic.output().ends_with(stderr));
        assert!(diagnostic.cut() > 0);
        component.shutdown();
    }
}

#[test]
fn design_paths_stream_past_the_stdout_and_conflict_limits() {
    let mut component = fixtures::Started::new();
    component.down(Request::Io {
        owner: fixtures::OWNER,
        op: git::Op::DiffPaths { at: fixtures::at(), before: fixtures::PARENT, after: fixtures::COMMIT },
        deadline: fixtures::DEADLINE,
        identity: None,
    });
    let (child, pipes) = component.spawned();
    for _ in 0_u32..100 {
        component.stdout(pipes[0], b"docs/design.md\0");
    }
    component.stdout(pipes[0], b"src/lib");
    component.stdout(pipes[0], b".rs\0later.md\0");
    component.end(pipes[0]);
    component.end(pipes[1]);
    component.exited(child, 0);
    component.closed(child, &pipes);
    assert_eq!(
        component.done(),
        Some((fixtures::OWNER, git::Done::Paths { files: Box::new([Box::from(b"src/lib.rs".as_slice())]) }))
    );
    component.shutdown();
}
#[test]
fn a_fetch_runs_two_invocations_and_ends_fetched() {
    let mut component = fixtures::Started::new();
    component.down(fixtures::fetch(fixtures::OWNER, fixtures::DEADLINE));
    let (child, pipes) = component.spawned();
    component.stdout(pipes[0], FETCHED);
    component.exited(child, 0);
    component.end(pipes[0]);
    component.end(pipes[1]);
    assert_eq!(component.done(), None);
    component.closed(child, &pipes);
    let (child, pipes) = component.spawned();
    component.end(pipes[0]);
    component.end(pipes[1]);
    component.exited(child, 0);
    component.closed(child, &pipes);
    assert_eq!(component.done(), Some((fixtures::OWNER, git::Done::Fetched { commit: fixtures::COMMIT })));
    assert!(component.below.out.io.is_empty());
    component.shutdown();
    assert!(component.component.is_empty());
}
#[test]
fn a_deadline_kills_the_group_and_ends_timed_out_after_the_close() {
    let mut component = fixtures::Started::new();
    component.down(fixtures::fetch(fixtures::OWNER, fixtures::DEADLINE));
    let (child, pipes) = component.spawned();
    component.fire(fixtures::DEADLINE);
    assert_eq!(&*component.signals(), &[(child, kernel::Signal::Kill, kernel::Target::Group)]);
    assert_eq!(component.done(), None);
    component.below.out.io = skein_lib::Queue::uncapped();
    component.component.up(
        component.now,
        skein_io::Event::Stream { owner: pipes[0], up: skein_lib::stream::Up::Failed(skein_lib::stream::Fault::Other) },
        &mut component.below.out,
    );
    assert!(component.below.out.io.is_empty(), "a pipe failure after the deadline issues no second close");
    component.closed(child, &pipes);
    assert_eq!(
        component.done(),
        Some((
            fixtures::OWNER,
            git::Done::FailedWithOutput { fault: git::Fault::TimedOut, diagnostic: git::PushDiagnostic::empty() }
        ))
    );
}
#[test]
fn a_cancel_after_the_last_exit_ends_with_the_operation_s_outcome() {
    let mut component = fixtures::Started::new();
    component.down(fixtures::fetch(fixtures::OWNER, fixtures::DEADLINE));
    let (child, pipes) = component.spawned();
    component.stdout(pipes[0], FETCHED);
    component.end(pipes[0]);
    component.end(pipes[1]);
    component.exited(child, 0);
    component.closed(child, &pipes);
    let (child, pipes) = component.spawned();
    component.end(pipes[0]);
    component.end(pipes[1]);
    component.exited(child, 0);
    component.down(Request::CancelIo { owner: fixtures::OWNER });
    assert!(component.signals().is_empty());
    component.closed(child, &pipes);
    assert_eq!(component.done(), Some((fixtures::OWNER, git::Done::Fetched { commit: fixtures::COMMIT })));
}
#[test]
fn a_cancel_between_invocations_starts_no_further_one() {
    let mut component = fixtures::Started::new();
    component.down(Request::Io {
        owner: fixtures::OWNER,
        op: git::Op::CheckOut { at: fixtures::at(), commit: fixtures::COMMIT, clean: true },
        deadline: fixtures::DEADLINE,
        identity: None,
    });
    let (child, pipes) = component.spawned();
    component.end(pipes[0]);
    component.end(pipes[1]);
    component.exited(child, 0);
    component.down(Request::CancelIo { owner: fixtures::OWNER });
    component.closed(child, &pipes);
    assert!(component.below.out.io.is_empty());
    assert_eq!(
        component.done(),
        Some((
            fixtures::OWNER,
            git::Done::FailedWithOutput { fault: git::Fault::Cancelled, diagnostic: git::PushDiagnostic::empty() }
        ))
    );
}
#[test]
fn stdout_past_its_cap_is_read_dropped_and_ends_broken() {
    let mut component = fixtures::Started::new();
    component.down(fixtures::fetch(fixtures::OWNER, fixtures::DEADLINE));
    let (child, pipes) = component.spawned();
    component.stdout(pipes[0], &[b'\n'; 513]);
    assert!(!component.below.out.io.is_empty(), "reading continues past the cap");
    component.end(pipes[0]);
    component.end(pipes[1]);
    component.exited(child, 0);
    component.closed(child, &pipes);
    assert_eq!(
        component.done(),
        Some((
            fixtures::OWNER,
            git::Done::FailedWithOutput { fault: git::Fault::Broken, diagnostic: git::PushDiagnostic::empty() }
        ))
    );
}
#[test]
fn a_merges_unmerged_stdout_alone_keeps_a_prefix_past_its_cap() {
    for merging in [false, true] {
        let mut component = fixtures::Started::new();
        let op = if merging {
            fixtures::commit_op(Some(fixtures::COMMIT))
        } else {
            git::Op::Merge { at: fixtures::at(), theirs: fixtures::COMMIT }
        };
        component.down(Request::Io {
            owner: fixtures::OWNER,
            op,
            deadline: fixtures::DEADLINE,
            identity: if merging { Some(fixtures::identity()) } else { None },
        });
        if !merging {
            let (child, pipes) = component.spawned();
            component.end(pipes[0]);
            component.end(pipes[1]);
            component.exited(child, 1);
            component.closed(child, &pipes);
        }
        let (child, pipes) = component.spawned();
        let cap = fixtures::limits().stdout_bytes;
        let mut stdout = skein_lib::List::with_capacity(cap.checked_add(6).expect("room for the last path"));
        for byte in b"one\0two\0three\0four\0" {
            stdout.push(*byte).expect("named paths fit");
        }
        while stdout.len() <= cap {
            for byte in b"later\0" {
                stdout.push(*byte).expect("room through the first path past the cap");
            }
        }
        component.stdout(pipes[0], stdout.as_slice());
        assert!(!component.below.out.io.is_empty(), "reading continues past the cap");
        component.end(pipes[0]);
        component.end(pipes[1]);
        component.exited(child, 0);
        component.closed(child, &pipes);
        let done = if merging {
            git::Done::FailedWithOutput { fault: git::Fault::Broken, diagnostic: git::PushDiagnostic::empty() }
        } else {
            git::Done::Conflicted {
                files: Box::new([
                    Box::from(b"one".as_slice()),
                    Box::from(b"two".as_slice()),
                    Box::from(b"three".as_slice()),
                    Box::from(b"four".as_slice()),
                ]),
            }
        };
        assert_eq!(component.done(), Some((fixtures::OWNER, done)), "merging commit: {merging}");
    }
}

#[test]
fn an_operation_whose_identity_has_no_value_fails_broken_unspawned() {
    let mut component = fixtures::Started::new();
    component.down(Request::Io {
        owner: fixtures::OWNER,
        op: fixtures::push_op(None),
        deadline: fixtures::DEADLINE,
        identity: None,
    });
    assert!(component.below.out.io.is_empty());
    assert_eq!(component.done(), Some((fixtures::OWNER, git::Done::Failed { fault: git::Fault::Broken })));
}
#[test]
fn a_git_older_than_two_forty_one_makes_the_start_unusable() {
    let mut component = fixtures::Started::unstarted();
    component.component.start(component.now, &mut component.below.out);
    let (child, pipes) = component.spawned();
    component.stdout(pipes[0], b"git version 2.40.9\n");
    component.end(pipes[0]);
    component.end(pipes[1]);
    component.exited(child, 0);
    component.closed(child, &pipes);
    assert_eq!(
        component.below.out.events.pop(),
        Some(Event::Unusable { why: Unusable::Version { found: Some(Version { major: 2, minor: 40, patch: 9 }) } })
    );
    assert!(component.component.is_empty());
}
#[test]
fn an_abort_during_commit_tree_spawn_settles_without_a_done() {
    let mut component = fixtures::Started::new();
    component.down(Request::Io {
        owner: fixtures::OWNER,
        op: fixtures::commit_op(None),
        deadline: fixtures::DEADLINE,
        identity: Some(fixtures::identity()),
    });
    for (code, stdout) in [(0, b"".as_slice()), (1, b""), (0, b"abababababababababababababababababababab\n")] {
        let (child, pipes) = component.spawned();
        component.stdout(pipes[0], stdout);
        component.end(pipes[0]);
        component.end(pipes[1]);
        component.exited(child, code);
        component.closed(child, &pipes);
    }
    component.component.abort(&mut component.below.out);
    let (child, pipes) = component.spawned();
    assert_eq!(pipes.len(), 3, "commit-tree's pending stdin pipe is still known");
    assert_eq!(component.below.out.io.len(), 4, "abort child and all three pipes");
    component.closed(child, &pipes);
    assert_eq!(component.done(), None);
    for _ in 0_u8..4 {
        component.file_operation();
        component.complete_ok();
    }
    assert!(component.component.is_empty());
}

#[test]
fn standalone_ancestry_needs_no_identity_and_cancels_after_its_child_closes() {
    let mut component = fixtures::Started::new();
    component.down(Request::Io {
        owner: fixtures::OWNER,
        op: git::Op::IsAncestor { at: fixtures::at(), ancestor: fixtures::PARENT, commit: fixtures::COMMIT },
        deadline: fixtures::DEADLINE,
        identity: None,
    });
    let (child, pipes) = component.spawned();
    component.down(Request::CancelIo { owner: fixtures::OWNER });
    assert_eq!(&*component.signals(), &[(child, kernel::Signal::Kill, kernel::Target::Group)]);
    assert_eq!(component.done(), None);
    component.closed(child, &pipes);
    assert_eq!(
        component.done(),
        Some((
            fixtures::OWNER,
            git::Done::FailedWithOutput { fault: git::Fault::Cancelled, diagnostic: git::PushDiagnostic::empty() }
        ))
    );
}

#[test]
fn a_look_spawns_at_the_worker_root_without_an_identity() {
    let mut component = fixtures::Started::new();
    component.down(Request::Io {
        owner: fixtures::OWNER,
        op: git::Op::Look { at: fixtures::at(), remote: Box::from(fixtures::REMOTE) },
        deadline: fixtures::DEADLINE,
        identity: None,
    });
    let Some(skein_io::Request::Spawn { spawn, .. }) = component.below.out.io.pop() else { panic!("look spawn") };
    assert_eq!(&*spawn.dir, b"");
    assert_eq!(
        spawn.args,
        crate::invocation::arguments(&crate::invocation::Command::Origin {
            directory: Box::from(&b"workspaces/7/app"[..])
        })
    );
    assert_eq!(component.done(), None);
}

#[test]
fn a_point_spawns_one_local_update_ref_without_an_identity() {
    for into in [git::Into::Fetched, git::Into::Landing] {
        let mut component = fixtures::Started::new();
        component.down(Request::Io {
            owner: fixtures::OWNER,
            op: git::Op::Point { at: fixtures::at(), into, commit: fixtures::COMMIT },
            deadline: fixtures::DEADLINE,
            identity: None,
        });
        let Some(skein_io::Request::Spawn { spawn, .. }) = component.below.out.io.pop() else { panic!("point spawn") };
        assert_eq!(&*spawn.dir, b"workspaces/7/app");
        let mut expected = skein_lib::List::uncapped();
        for setting in crate::invocation::SETTINGS {
            expected.push(Box::from(b"-c".as_slice())).expect("uncapped args");
            expected.push(Box::from(setting)).expect("uncapped args");
        }
        let reference = match into {
            git::Into::Fetched => b"refs/temper/fetched".as_slice(),
            git::Into::Landing => b"refs/temper/landing".as_slice(),
        };
        for word in [b"update-ref".as_slice(), reference, b"abababababababababababababababababababab"] {
            expected.push(Box::from(word)).expect("uncapped args");
        }
        assert_eq!(spawn.args.as_ref(), expected.as_slice());
        assert_eq!(spawn.env, crate::invocation::environment(fixtures::HOME, None, None));
        assert!(component.below.out.io.is_empty(), "one invocation");
        assert_eq!(component.done(), None);
    }
}

/// temper/docs/design/protocol/git.md, 4.6: merge progress is not a parsed result.
#[test]
fn a_merges_progress_stdout_does_not_refuse_its_conflict_listing() {
    for code in [0, 1] {
        let mut component = fixtures::Started::new();
        component.down(Request::Io {
            owner: fixtures::OWNER,
            op: git::Op::Merge { at: fixtures::at(), theirs: fixtures::COMMIT },
            deadline: fixtures::DEADLINE,
            identity: None,
        });
        let (child, pipes) = component.spawned();
        component.stdout(pipes[0], &[b'x'; 513]);
        component.end(pipes[0]);
        component.end(pipes[1]);
        component.exited(child, code);
        component.closed(child, &pipes);
        let expected = if code == 0 {
            git::Done::Merged
        } else {
            assert_eq!(component.done(), None, "the conflict listing still runs");
            let (child, pipes) = component.spawned();
            component.stdout(pipes[0], b"baseline\0");
            component.end(pipes[0]);
            component.end(pipes[1]);
            component.exited(child, 0);
            component.closed(child, &pipes);
            git::Done::Conflicted { files: Box::new([b"baseline".as_slice().into()]) }
        };
        assert_eq!(component.done(), Some((fixtures::OWNER, expected)));
    }
}
