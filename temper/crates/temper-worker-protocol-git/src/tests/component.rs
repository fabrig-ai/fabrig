use super::fixtures;
use crate::{Event, Request, Unusable, Version};
use skein_io::kernel;
use temper_worker_domain_checkout::git;
const FETCHED: &[u8] =
    b"* 0000000000000000000000000000000000000000 abababababababababababababababababababab refs/temper/7/fetched\n";
// One byte past the component fixture's stdout limit.
const OVERLONG_PROGRESS: usize = 513;

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
fn an_unanswered_spawn_keeps_its_output_bounds() {
    let raised = crate::Limits { stdout_bytes: 1024, stderr_bytes: 1024, ..fixtures::limits() };
    let auth = b"fatal: Authentication failed for 'remote'\n";
    for long_stdout in [true, false] {
        let mut started = fixtures::Started::new();
        for (first, fault) in [(true, git::Fault::Broken), (false, git::Fault::Refused)] {
            started.down(Request::Io {
                owner: fixtures::OWNER,
                op: fixtures::push_op(None),
                deadline: fixtures::DEADLINE,
                identity: Some(fixtures::identity()),
            });
            if first {
                assert!(started.component.take_limits(&raised), "raise after issuing the spawn but before Spawned");
            }
            let (child, pipes) = started.spawned();
            started.stdout(pipes[1], auth);
            if long_stdout {
                // With truncated stdout we cannot trust the push's reply.
                started.stdout(pipes[0], &[b'x'; 600]);
                started.stdout(pipes[0], b"\n");
            } else {
                // With a short stderr tail the cause is no longer visible.
                started.stdout(pipes[1], &[b'x'; 600]);
                started.stdout(pipes[1], b"\n");
            }
            started.end(pipes[0]);
            started.end(pipes[1]);
            started.exited(child, 128);
            started.closed(child, &pipes);
            let (_, git::Done::FailedWithOutput { fault: actual, .. }) = started.done().expect("terminal") else {
                panic!("push failure carries output");
            };
            assert_eq!(
                actual,
                fault,
                "push with {} truncated by the old bound",
                if long_stdout { "stdout" } else { "stderr" }
            );
        }
        started.shutdown();
    }
}

#[test]
fn design_paths_stream_past_the_stdout_and_conflict_limits() {
    changed_paths(false);
}
#[test]
fn prepare_tree_comparison_includes_markdown_paths_in_the_stream() {
    changed_paths(true);
}
fn changed_paths(markdown: bool) {
    let mut component = fixtures::Started::new();
    component.down(Request::Io {
        owner: fixtures::OWNER,
        op: git::Op::DiffPaths { at: fixtures::at(), before: fixtures::PARENT, after: fixtures::COMMIT, markdown },
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
        Some((
            fixtures::OWNER,
            git::Done::Paths {
                files: Box::new([Box::from(if markdown { b"docs/design.md".as_slice() } else { b"src/lib.rs" })])
            }
        ))
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
    super::view::empty(&mut component);
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
    assert_eq!(
        &*component.signals(),
        &[(child, kernel::Signal::Kill, kernel::Target::Group, kernel::Reason::Deadline)]
    );
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
            git::Done::FailedWithOutput {
                fault: git::Fault::TimedOut,
                diagnostic: git::PushDiagnostic::empty().named(b"fetch")
            }
        ))
    );
}
/// temper/docs/design/protocol/git.md, 7: broken io names its kill Closed.
#[test]
fn broken_io_kills_the_group_named_closed() {
    for pipe_failure in [false, true] {
        let mut component = fixtures::Started::new();
        component.down(fixtures::fetch(fixtures::OWNER, fixtures::DEADLINE));
        let (child, pipes) = component.spawned();
        let event = if pipe_failure {
            skein_io::Event::Stream {
                owner: pipes[0],
                up: skein_lib::stream::Up::Failed(skein_lib::stream::Fault::Other),
            }
        } else {
            skein_io::Event::Failed { owner: component.below.io_owner.expect("spawned"), error: skein_io::Error::Other }
        };
        component.component.up(component.now, event, &mut component.below.out);
        assert_eq!(
            &*component.signals(),
            &[(child, kernel::Signal::Kill, kernel::Target::Group, kernel::Reason::Closed)]
        );
        assert_eq!(component.done(), None);
        component.closed(child, &pipes);
        assert_eq!(
            component.done(),
            Some((
                fixtures::OWNER,
                git::Done::FailedWithOutput {
                    fault: git::Fault::Broken,
                    diagnostic: git::PushDiagnostic::empty().named(b"fetch"),
                },
            ))
        );
    }
}

/// temper/docs/design/protocol/git-operations.md, 1.5, and temper/docs/design/protocol/git.md, 6: one deadline, distinct invocations.
#[test]
fn checkout_and_clean_deadlines_name_the_invocation_in_flight() {
    for (during_clean, word) in [(false, b"checkout".as_slice()), (true, b"clean".as_slice())] {
        let mut component = fixtures::Started::new();
        component.down(Request::Io {
            owner: fixtures::OWNER,
            op: git::Op::CheckOut {
                at: fixtures::at(),
                commit: fixtures::COMMIT,
                clean: true,
                identity: 1,
                account: None,
            },
            deadline: fixtures::DEADLINE,
            identity: Some(fixtures::identity()),
        });
        if during_clean {
            let (child, pipes) = component.spawned();
            component.end(*pipes.first().expect("stdout"));
            component.end(*pipes.get(1).expect("stderr"));
            component.exited(child, 0);
            component.closed(child, &pipes);
            assert_eq!(component.done(), None);
        }
        let (child, pipes) = component.spawned();
        component.fire(fixtures::DEADLINE);
        assert_eq!(
            &*component.signals(),
            &[(child, kernel::Signal::Kill, kernel::Target::Group, kernel::Reason::Deadline)]
        );
        assert_eq!(component.done(), None);
        component.closed(child, &pipes);
        let Some((fixtures::OWNER, git::Done::FailedWithOutput { fault: git::Fault::TimedOut, diagnostic })) =
            component.done()
        else {
            panic!("deadline ends with output after close");
        };
        assert_eq!(diagnostic.command(), Some(word));
    }
}

/// temper/docs/design/protocol/git.md, section 6: queued spawns name the next invocation.
#[test]
fn a_cancel_during_a_queued_clean_spawn_names_clean() {
    let mut component = fixtures::Started::new();
    component.down(Request::Io {
        owner: fixtures::OWNER,
        op: git::Op::CheckOut { at: fixtures::at(), commit: fixtures::COMMIT, clean: true, identity: 1, account: None },
        deadline: fixtures::DEADLINE,
        identity: Some(fixtures::identity()),
    });
    let (child, pipes) = component.spawned();
    component.end(*pipes.first().expect("stdout"));
    component.end(*pipes.get(1).expect("stderr"));
    component.exited(child, 0);
    component.closed(child, &pipes);
    component.down(Request::CancelIo { owner: fixtures::OWNER });
    assert_eq!(component.done(), None);
    let (child, pipes) = component.spawned();
    assert_eq!(
        &*component.signals(),
        &[(child, kernel::Signal::Kill, kernel::Target::Group, kernel::Reason::Cancelled)]
    );
    component.closed(child, &pipes);
    assert_eq!(
        component.done(),
        Some((
            fixtures::OWNER,
            git::Done::FailedWithOutput {
                fault: git::Fault::Cancelled,
                diagnostic: git::PushDiagnostic::empty().named(b"clean"),
            }
        ))
    );
}

/// temper/docs/design/protocol/git.md, section 6: classification retains its invocation.
#[test]
fn a_classified_fetch_failure_names_fetch() {
    let mut component = fixtures::Started::new();
    component.down(fixtures::fetch(fixtures::OWNER, fixtures::DEADLINE));
    let (child, pipes) = component.spawned();
    let stderr = b"fatal: unable to access 'remote': Could not resolve host\n";
    component.stdout(*pipes.get(1).expect("stderr"), stderr);
    component.end(*pipes.first().expect("stdout"));
    component.end(*pipes.get(1).expect("stderr"));
    component.exited(child, 128);
    component.closed(child, &pipes);
    let Some((fixtures::OWNER, git::Done::FailedWithOutput { fault: git::Fault::Unreachable, diagnostic })) =
        component.done()
    else {
        panic!("fetch failure is classified from stderr");
    };
    assert_eq!(diagnostic.command(), Some(b"fetch".as_slice()));
    assert_eq!(diagnostic.output(), stderr);
}

/// temper/docs/design/protocol/git.md, section 6: an entrance failure has no output.
#[test]
fn an_operation_received_past_its_deadline_fails_without_output() {
    let mut component = fixtures::Started::new();
    component.now = fixtures::DEADLINE;
    component.down(fixtures::fetch(fixtures::OWNER, fixtures::DEADLINE));
    assert!(component.below.out.io.is_empty());
    assert_eq!(component.done(), Some((fixtures::OWNER, git::Done::Failed { fault: git::Fault::TimedOut })));
}

#[test]
fn a_cancel_after_the_last_git_exit_still_preempts_its_view() {
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
    assert!(component.signals().is_empty(), "the child is already closing");
    component.closed(child, &pipes);
    assert_eq!(
        component.done(),
        Some((
            fixtures::OWNER,
            git::Done::FailedWithOutput {
                fault: git::Fault::Cancelled,
                diagnostic: git::PushDiagnostic::empty().named(b"update-ref")
            }
        ))
    );
}
#[test]
fn a_cancel_between_invocations_starts_no_further_one() {
    let mut component = fixtures::Started::new();
    component.down(Request::Io {
        owner: fixtures::OWNER,
        op: git::Op::CheckOut { at: fixtures::at(), commit: fixtures::COMMIT, clean: true, identity: 1, account: None },
        deadline: fixtures::DEADLINE,
        identity: Some(fixtures::identity()),
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
            git::Done::FailedWithOutput {
                fault: git::Fault::Cancelled,
                diagnostic: git::PushDiagnostic::empty().named(b"checkout")
            }
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
            git::Done::FailedWithOutput {
                fault: git::Fault::Broken,
                diagnostic: git::PushDiagnostic::empty().named(b"fetch")
            }
        ))
    );
}
#[test]
fn a_merges_unmerged_stdout_alone_keeps_a_prefix_past_its_cap() {
    let mut component = fixtures::Started::new();
    let op = git::Op::Merge { at: fixtures::at(), theirs: fixtures::COMMIT, identity: 1, account: None };
    component.down(Request::Io {
        owner: fixtures::OWNER,
        op,
        deadline: fixtures::DEADLINE,
        identity: Some(fixtures::identity()),
    });
    let (child, pipes) = component.spawned();
    component.end(pipes[0]);
    component.end(pipes[1]);
    component.exited(child, 1);
    component.closed(child, &pipes);
    super::view::invocation(&mut component, b"", 0);
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
    let done = git::Done::Conflicted {
        files: Box::new([
            Box::from(b"one".as_slice()),
            Box::from(b"two".as_slice()),
            Box::from(b"three".as_slice()),
            Box::from(b"four".as_slice()),
        ]),
        cut: true,
    };
    super::view::empty(&mut component);
    assert_eq!(component.done(), Some((fixtures::OWNER, done)));
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
    for _ in 0_u8..6 {
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
    assert_eq!(
        &*component.signals(),
        &[(child, kernel::Signal::Kill, kernel::Target::Group, kernel::Reason::Cancelled)]
    );
    assert_eq!(component.done(), None);
    component.closed(child, &pipes);
    assert_eq!(
        component.done(),
        Some((
            fixtures::OWNER,
            git::Done::FailedWithOutput {
                fault: git::Fault::Cancelled,
                diagnostic: git::PushDiagnostic::empty().named(b"merge-base")
            }
        ))
    );
}

#[test]
fn a_look_spawns_at_the_worker_root_without_an_identity() {
    let mut component = fixtures::Started::new();
    component.down(Request::Io {
        owner: fixtures::OWNER,
        op: git::Op::Look { at: fixtures::at(), remote: Box::from(fixtures::REMOTE), store: skein_lib::Token::new(9) },
        deadline: fixtures::DEADLINE,
        identity: None,
    });
    let Some(skein_io::Request::Spawn { spawn, .. }) = component.below.out.io.pop() else { panic!("look spawn") };
    assert_eq!(&*spawn.dir, b"");
    assert_eq!(
        spawn.args,
        crate::invocation::arguments(&crate::invocation::Command::Origin {
            directory: Box::from(&b"gitdirs/7/app"[..])
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
        assert_eq!(&*spawn.dir, b"");
        let mut expected = skein_lib::List::uncapped();
        for setting in crate::invocation::SETTINGS {
            expected.push(Box::from(b"-c".as_slice())).expect("uncapped args");
            expected.push(Box::from(setting)).expect("uncapped args");
        }
        let reference = match into {
            git::Into::Fetched => b"refs/temper/fetched".as_slice(),
            git::Into::Landing => b"refs/temper/landing".as_slice(),
        };
        for word in [
            b"--git-dir=gitdirs/7/app".as_slice(),
            b"--work-tree=workspaces/7/app",
            b"update-ref",
            reference,
            b"abababababababababababababababababababab",
        ] {
            expected.push(Box::from(word)).expect("uncapped args");
        }
        assert_eq!(spawn.args.as_ref(), expected.as_slice());
        assert_eq!(spawn.env, crate::invocation::environment(fixtures::HOME, None, None));
        assert!(component.below.out.io.is_empty(), "one invocation");
        assert_eq!(component.done(), None);
    }
}

/// temper/docs/design/protocol/git-operations.md, 1.6: neither command's stdout is a parsed result.
#[test]
fn a_merges_progress_stdout_does_not_refuse_its_conflict_listing() {
    for code in [0, 1] {
        let mut component = start_with_identity(merge());
        super::view::invocation(&mut component, &[b'x'; OVERLONG_PROGRESS], code);
        let expected = if code == 0 {
            git::Done::Merged
        } else {
            assert_eq!(component.done(), None, "the conflict listing still runs");
            super::view::invocation(&mut component, b"", 0);
            super::view::invocation(&mut component, b"baseline\0", 0);
            git::Done::Conflicted { files: Box::new([b"baseline".as_slice().into()]), cut: false }
        };
        super::view::empty(&mut component);
        assert_eq!(component.done(), Some((fixtures::OWNER, expected)));
    }
}

#[test]
fn overlong_unterminated_history_stdout_does_not_refuse_conflicts() {
    let mut component = start_with_identity(merge());
    super::view::invocation(&mut component, b"", 1);
    super::view::invocation(&mut component, &[b'x'; OVERLONG_PROGRESS], 0);
    assert_eq!(component.done(), None, "only the listing can finish the conflicted merge");
    super::view::invocation(&mut component, b"baseline\0", 0);
    super::view::empty(&mut component);
    assert_eq!(
        component.done(),
        Some((fixtures::OWNER, git::Done::Conflicted { files: Box::new([b"baseline".as_slice().into()]), cut: false }))
    );
}

#[test]
fn history_fetch_failure_names_log_and_keeps_the_object() {
    for (stderr, expected) in [
        (
            b"fatal: unable to access 'https://forge.invalid/o/r.git/': \
              Failed to connect to forge.invalid port 443 after 0 ms: Could not connect to server\n\
              fatal: could not fetch 1545e581943f2dd5f6250aa315b8c67fd118b3de from promisor remote\n"
                .as_slice(),
            git::Fault::Unreachable,
        ),
        (b"fatal: unable to read 4cb29ea38f70d7c61b2a3a25b02e3bdf44905402\n", git::Fault::Broken),
    ] {
        let mut component = start_with_identity(merge());
        super::view::invocation(&mut component, b"", 1);
        let (child, pipes) = component.spawned();
        component.stdout(pipes[1], stderr);
        component.end(pipes[0]);
        component.end(pipes[1]);
        component.exited(child, 128);
        component.closed(child, &pipes);
        let Some((fixtures::OWNER, git::Done::FailedWithOutput { fault, diagnostic })) = component.done() else {
            panic!("history fetch failure with output")
        };
        assert_eq!(fault, expected);
        assert_eq!(diagnostic.command(), Some(b"log".as_slice()));
        assert_eq!(diagnostic.output(), stderr);
        assert_eq!(diagnostic.cut(), 0);
        assert!(component.below.out.io.is_empty() && component.below.out.kernel.is_empty());
    }
}

#[test]
fn only_the_promisor_invocations_receive_lazy_fetch_credentials() {
    let checkout =
        git::Op::CheckOut { at: fixtures::at(), commit: fixtures::COMMIT, clean: true, identity: 1, account: None };
    for (op, invocations) in [
        (checkout, [(0, true, b"checkout".as_slice()), (0, false, b"clean")].as_slice()),
        (merge(), &[(1, true, b"merge".as_slice()), (0, true, b"log"), (0, false, b"diff-files")]),
    ] {
        let mut component = start_with_identity(op);
        for &(exit, remote, word) in invocations {
            assert_spawn_credentials(&component, word, remote);
            super::view::invocation(&mut component, b"", exit);
        }
    }
}

fn merge() -> git::Op {
    git::Op::Merge { at: fixtures::at(), theirs: fixtures::COMMIT, identity: 1, account: None }
}

fn start_with_identity(op: git::Op) -> fixtures::Started {
    let mut component = fixtures::Started::new();
    component.down(Request::Io {
        owner: fixtures::OWNER,
        op,
        deadline: fixtures::DEADLINE,
        identity: Some(fixtures::identity()),
    });
    component
}

fn assert_spawn_credentials(component: &fixtures::Started, word: &[u8], remote: bool) {
    let mut spawned = false;
    for request in &component.below.out.io {
        if let skein_io::Request::Spawn { spawn, .. } = request {
            assert!(!spawned, "only one spawn");
            spawned = true;
            assert!(spawn.args.contains(&Box::from(word)));
            assert_eq!(spawn.env.contains(&Box::from(b"GIT_CONFIG_COUNT=1".as_slice())), remote);
            if !remote {
                for entry in &spawn.env {
                    assert!(!entry.starts_with(b"GIT_CONFIG_COUNT="));
                    assert!(!entry.starts_with(b"GIT_CONFIG_KEY_"));
                    assert!(!entry.starts_with(b"GIT_CONFIG_VALUE_"));
                }
            }
            assert_eq!(spawn.args.contains(&Box::from(b"remote.origin.promisor=true".as_slice())), remote);
            if remote {
                assert!(spawn.env.contains(&Box::from(b"GIT_COMMITTER_NAME=Temper Bot".as_slice())));
                assert!(spawn.env.contains(&Box::from(b"GIT_CONFIG_KEY_0=http.extraHeader".as_slice())));
                assert!(spawn.env.contains(&Box::from(
                    b"GIT_CONFIG_VALUE_0=Authorization: Basic dGVtcGVyLWJvdDp0b2stMQ==".as_slice()
                )));
            }
        }
    }
    assert!(spawned);
}
