//! Pack's local maintenance sequence (protocol/git.md, 4.13).
use super::fixtures::{self, Started};
use crate::Request;
use alloc::boxed::Box;
use skein_lib::{List, Token};
use temper_worker_domain_checkout::git;

fn request(s: &mut Started, owner: Token, at: git::Place, store: Option<Token>) {
    s.down(Request::Io { owner, op: git::Op::Pack { at, store }, deadline: fixtures::DEADLINE, identity: None });
}

fn check_invocation(s: &Started, directory: &[u8], suffix: &[&[u8]]) {
    let mut seen = false;
    for request in &s.below.out.io {
        if let skein_io::Request::Spawn { spawn, .. } = request {
            assert_eq!(spawn.dir.as_ref(), b"");
            let mut expected = List::with_capacity(40);
            for setting in crate::invocation::SETTINGS {
                expected.push(Box::from(b"-c".as_slice())).expect("fixed argv");
                expected.push(Box::from(setting)).expect("fixed argv");
            }
            let suffix = if suffix.starts_with(&[b"-c", b"remote.origin.promisor=true"]) {
                for argument in suffix.get(..2).expect("promisor setting") {
                    expected.push(Box::from(*argument)).expect("fixed argv");
                }
                suffix.get(2..).expect("command after setting")
            } else {
                suffix
            };
            expected.push(crate::invocation::joined(&[b"--git-dir=", directory])).expect("fixed argv");
            if directory == b"gitdirs/7/app" {
                expected.push(Box::from(b"--work-tree=workspaces/7/app".as_slice())).expect("fixed argv");
            }
            for argument in suffix {
                expected.push(Box::from(*argument)).expect("fixed argv");
            }
            assert_eq!(spawn.args, expected.into_boxed());
            let mut environment = List::with_capacity(9);
            for entry in [
                b"HOME=/srv/worker/home".as_slice(),
                b"LC_ALL=C",
                b"TZ=UTC",
                b"GIT_CONFIG_NOSYSTEM=1",
                b"GIT_CONFIG_GLOBAL=/dev/null",
                b"GIT_TERMINAL_PROMPT=0",
                b"GIT_ASKPASS=",
                b"GIT_COMMITTER_NAME=temper",
                b"GIT_COMMITTER_EMAIL=temper@temper.invalid",
            ] {
                environment.push(Box::from(entry)).expect("identity-less environment");
            }
            assert_eq!(spawn.env, environment.into_boxed());
            assert!(!seen, "only one spawn at a time");
            seen = true;
        }
    }
    assert!(seen, "Pack must run the count or repack invocation");
}

fn invocation(s: &mut Started, directory: &[u8], suffix: &[&[u8]], stdout: &[u8]) {
    check_invocation(s, directory, suffix);
    let (child, pipes) = s.spawned();
    s.stdout(pipes[0], stdout);
    s.end(pipes[0]);
    s.end(pipes[1]);
    s.exited(child, 0);
    s.closed(child, &pipes);
}

const COUNT: &[&[u8]] = &[b"count-objects", b"-v"];
const REPACK: &[&[u8]] =
    &[b"-c", b"remote.origin.promisor=true", b"repack", b"-A", b"-d", b"-l", b"-q", b"--no-write-bitmap-index"];

#[test]
fn a_pack_counts_and_repacks_the_store_then_its_repository_only_above_fifty() {
    for (store_count, store_repack, repository_count, repository_repack) in [
        (b"packs: 51\n".as_slice(), true, b"packs: 51\n".as_slice(), true),
        (b"packs: 51\n".as_slice(), true, b"packs: 50\n".as_slice(), false),
        (b"packs: 50\n".as_slice(), false, b"packs: 51\n".as_slice(), true),
        (b"packs: 50\n".as_slice(), false, b"packs: 50\n".as_slice(), false),
    ] {
        let mut s = Started::new();
        request(&mut s, fixtures::OWNER, fixtures::at(), Some(Token::new(7)));
        invocation(&mut s, b"stores/7", COUNT, store_count);
        if store_repack {
            invocation(&mut s, b"stores/7", REPACK, b"");
        }
        invocation(&mut s, b"gitdirs/7/app", COUNT, repository_count);
        if repository_repack {
            invocation(&mut s, b"gitdirs/7/app", REPACK, b"");
        }
        assert_eq!(s.done(), Some((fixtures::OWNER, git::Done::Succeeded)));
        assert!(s.below.out.io.is_empty());
    }
}

const EIGHT: &[u8] =
    b"count: 0\nsize: 0\nin-pack: 27\npacks: 8\nsize-pack: 12\nprune-packable: 0\ngarbage: 0\nsize-garbage: 0\n";
const ALTERNATE: &[u8] = b"count: 0\nsize: 0\nin-pack: 27\npacks: 8\nsize-pack: 12\nprune-packable: 0\ngarbage: 0\nsize-garbage: 0\nalternate: /r/stores/7/objects\n";
const UNREADABLE: &[&[u8]] = &[
    b"count: 0\n",
    b"packs: 8\npacks: 8\n",
    b"packs: x\n",
    b"packs: -1\n",
    b"packs: +8\n",
    b"packs: 8x\n",
    b"packs: \n",
    b"packs:  8\n",
    b"packs: 4294967296\n",
    b"packs: 8",
];

#[test]
fn pack_counts_require_one_complete_decimal_packs_line() {
    for stdout in [EIGHT, ALTERNATE] {
        assert_eq!(crate::output::packs(stdout), Some(8));
    }
    assert_eq!(crate::output::packs(b"packs: 0\n"), Some(0));
    assert_eq!(crate::output::packs(b"packs: 4294967295\n"), Some(u32::MAX));
    for stdout in UNREADABLE {
        assert_eq!(crate::output::packs(stdout), None, "{stdout:?}");
    }
}

#[test]
fn a_pack_without_a_store_counts_only_its_repository() {
    for (stdout, repack) in [(b"packs: 51\n".as_slice(), true), (b"packs: 50\n".as_slice(), false)] {
        let mut s = Started::new();
        let op = git::Op::Pack { at: fixtures::at(), store: None };
        assert!(!op.is_remote(), "Pack uses the local deadline");
        assert_eq!(op.kind(), git::Kind::Pack);
        s.down(Request::Io { owner: fixtures::OWNER, op, deadline: fixtures::DEADLINE, identity: None });
        invocation(&mut s, b"gitdirs/7/app", COUNT, stdout);
        if repack {
            invocation(&mut s, b"gitdirs/7/app", REPACK, b"");
        }
        assert_eq!(s.done(), Some((fixtures::OWNER, git::Done::Succeeded)));
        assert!(s.below.out.io.is_empty());
    }
}

#[test]
fn a_pack_reads_counts_in_both_places_and_stops_at_an_unreadable_count() {
    for stdout in [EIGHT, ALTERNATE] {
        let mut s = Started::new();
        request(&mut s, fixtures::OWNER, fixtures::at(), Some(Token::new(7)));
        invocation(&mut s, b"stores/7", COUNT, stdout);
        invocation(&mut s, b"gitdirs/7/app", COUNT, stdout);
        assert_eq!(s.done(), Some((fixtures::OWNER, git::Done::Succeeded)));
        assert!(s.below.out.io.is_empty());
    }
    for in_store in [true, false] {
        for stdout in UNREADABLE {
            let mut s = Started::new();
            request(&mut s, fixtures::OWNER, fixtures::at(), Some(Token::new(7)));
            if !in_store {
                invocation(&mut s, b"stores/7", COUNT, b"packs: 50\n");
            }
            let directory = if in_store { b"stores/7".as_slice() } else { b"gitdirs/7/app".as_slice() };
            invocation(&mut s, directory, COUNT, stdout);
            assert_eq!(
                s.done(),
                Some((
                    fixtures::OWNER,
                    git::Done::FailedWithOutput {
                        fault: git::Fault::Broken,
                        diagnostic: git::PushDiagnostic::empty().named(b"count-objects"),
                    }
                ))
            );
            assert!(s.below.out.io.is_empty(), "no invocation after an unreadable count");
        }
    }
}

#[test]
fn a_pack_stops_on_each_failed_local_invocation_and_keeps_its_diagnostic() {
    for in_store in [true, false] {
        for repack in [false, true] {
            let mut s = Started::new();
            request(&mut s, fixtures::OWNER, fixtures::at(), Some(Token::new(7)));
            if !in_store {
                invocation(&mut s, b"stores/7", COUNT, b"packs: 51\n");
                invocation(&mut s, b"stores/7", REPACK, b"");
            }
            let directory = if in_store { b"stores/7".as_slice() } else { b"gitdirs/7/app".as_slice() };
            if repack {
                invocation(&mut s, directory, COUNT, b"packs: 51\n");
            }
            check_invocation(&s, directory, if repack { REPACK } else { COUNT });
            let (child, pipes) = s.spawned();
            let stderr = b"fatal: Authentication failed for 'local'\n";
            s.stdout(pipes[0], if repack { b"" } else { b"packs: 51\n" });
            s.stdout(pipes[1], stderr);
            s.end(pipes[0]);
            s.end(pipes[1]);
            s.exited(child, 128);
            assert_eq!(s.done(), None, "wait for close");
            s.closed(child, &pipes);
            assert_eq!(
                s.done(),
                Some((
                    fixtures::OWNER,
                    git::Done::FailedWithOutput {
                        fault: git::Fault::Broken,
                        diagnostic: git::PushDiagnostic::new(stderr, 0).named(if repack {
                            b"repack"
                        } else {
                            b"count-objects"
                        }),
                    }
                ))
            );
            assert!(s.below.out.io.is_empty(), "nothing starts after the failure");
        }
    }
}

#[test]
fn a_deadline_or_cancel_kills_a_running_repack_before_ending() {
    use skein_io::kernel::{Reason, Signal, Target};
    for (deadline, fault, reason) in
        [(true, git::Fault::TimedOut, Reason::Deadline), (false, git::Fault::Cancelled, Reason::Cancelled)]
    {
        let mut s = Started::new();
        request(&mut s, fixtures::OWNER, fixtures::at(), Some(Token::new(7)));
        invocation(&mut s, b"stores/7", COUNT, b"packs: 51\n");
        check_invocation(&s, b"stores/7", REPACK);
        let (child, pipes) = s.spawned();
        if deadline {
            s.fire(fixtures::DEADLINE);
        } else {
            s.down(Request::CancelIo { owner: fixtures::OWNER });
        }
        assert_eq!(s.signals().as_ref(), &[(child, Signal::Kill, Target::Group, reason)]);
        assert_eq!(s.done(), None);
        s.closed(child, &pipes);
        assert_eq!(
            s.done(),
            Some((
                fixtures::OWNER,
                git::Done::FailedWithOutput { fault, diagnostic: git::PushDiagnostic::empty().named(b"repack") }
            ))
        );
        assert!(s.below.out.io.is_empty(), "no repository invocation after stopping the store");
    }
}
