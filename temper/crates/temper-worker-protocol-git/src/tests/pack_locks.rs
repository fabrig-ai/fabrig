//! Store readers, an exclusive Pack and FIFO waiters (protocol/git-operations.md, 1.14).
use super::fixtures::{self, Started};
use crate::{Component, Request, invocation};
use alloc::boxed::Box;
use skein_io as io;
use skein_lib::{List, Queue, Time, Token};
use temper_worker_domain_checkout::git;

const FIRST: Token = Token::new(91);
const SECOND: Token = Token::new(92);
const THIRD: Token = Token::new(93);
const FOURTH: Token = Token::new(94);
const LATER: Time = Time::from_nanos(10000);
const FETCHED: &[u8] =
    b"* 0000000000000000000000000000000000000000 abababababababababababababababababababab refs/temper/7/fetched\n";

fn started() -> Started {
    let mut s = Started::unstarted();
    let mut limits = fixtures::limits();
    limits.operations = 4;
    let settings =
        crate::Settings { serial: fixtures::SERIAL.checked_sub(4).expect("layout names"), ..fixtures::settings() };
    s.component = Component::new(&limits, &fixtures::checkout(), settings).expect("four operations");
    s.component.start(s.now, &mut s.below.out);
    let version = spawn(&mut s, b"", b"version");
    finish(&mut s, &version, b"git version 2.47.3\n", 0);
    s.component.reclaim();
    for _ in 0_u8..15 {
        s.file_operation();
        s.complete_ok();
    }
    assert_eq!(s.below.out.events.pop(), Some(crate::Event::Started));
    s.empty_sweep();
    s
}
fn pack(s: &mut Started, owner: Token, store: Option<Token>, deadline: Time) {
    s.down(Request::Io { owner, op: git::Op::Pack { at: fixtures::at(), store }, deadline, identity: None });
}
fn fetch(s: &mut Started, owner: Token, store: Token) {
    fetch_until(s, owner, store, LATER);
}
fn fetch_until(s: &mut Started, owner: Token, store: Token, deadline: Time) {
    s.down(Request::Io {
        owner,
        op: git::Op::Fetch {
            at: fixtures::at(),
            store,
            remote: fixtures::REMOTE.into(),
            want: git::Want::Default,
            into: git::Into::Fetched,
            identity: 1,
            account: None,
        },
        deadline,
        identity: Some(fixtures::identity()),
    });
}

struct Running {
    owner: Token,
    child: Token,
    pipes: Box<[Token]>,
}
// Keep other queued spawns while the fixtures deliver one invocation's events.
fn spawn(s: &mut Started, directory: &[u8], word: &[u8]) -> Running {
    let request = s.below.out.io.pop().expect("next invocation");
    match &request {
        io::Request::Spawn { spawn, .. } => {
            assert_eq!(spawn.dir.as_ref(), b"");
            let offset = invocation::SETTINGS.len().checked_mul(2).expect("fixed settings");
            let mut arguments = spawn.args.get(offset..).expect("settings precede command");
            if word == b"repack" {
                assert_eq!(arguments.first().expect("setting flag").as_ref(), b"-c");
                assert_eq!(arguments.get(1).expect("promisor setting").as_ref(), b"remote.origin.promisor=true");
                arguments = arguments.get(2..).expect("command after setting");
            }
            if !directory.is_empty() {
                assert_eq!(arguments.first().expect("git directory"), &invocation::joined(&[b"--git-dir=", directory]));
                arguments = arguments.get(1..).expect("command after git directory");
                if directory == b"gitdirs/7/app" {
                    assert_eq!(arguments.first().expect("work tree").as_ref(), b"--work-tree=workspaces/7/app");
                    arguments = arguments.get(1..).expect("command after work tree");
                }
            }
            assert_eq!(arguments.first().expect("subcommand").as_ref(), word);
        }
        other @ (io::Request::Listen { .. }
        | io::Request::Connect { .. }
        | io::Request::Bind { .. }
        | io::Request::Reject { .. }
        | io::Request::Stream { .. }
        | io::Request::Output { .. }
        | io::Request::SpawnAlone { .. }
        | io::Request::Signal { .. }
        | io::Request::Usage { .. }
        | io::Request::Window { .. }
        | io::Request::Close { .. }
        | io::Request::Abort { .. }) => panic!("expected spawn, got {other:?}"),
    }
    let saved = core::mem::replace(&mut s.below.out.io, Queue::with_capacity(20));
    s.below.out.io.push(request);
    let (child, pipes) = s.spawned();
    s.below.out.io = saved;
    Running { owner: s.below.io_owner.expect("spawn owner"), child, pipes }
}
fn finish(s: &mut Started, running: &Running, stdout: &[u8], code: u8) {
    let mut saved = core::mem::replace(&mut s.below.out.io, Queue::with_capacity(20));
    s.below.io_owner = Some(running.owner);
    s.stdout(running.pipes[0], stdout);
    s.end(running.pipes[0]);
    s.end(running.pipes[1]);
    s.exited(running.child, code);
    assert!(directories(s).is_empty(), "no lock released before the child closes");
    s.closed(running.child, &running.pipes);
    while let Some(request) = s.below.out.io.pop() {
        saved.push(request);
    }
    s.below.out.io = saved;
}
fn directories(s: &Started) -> Box<[Box<[u8]>]> {
    let mut dirs = List::with_capacity(4);
    for request in &s.below.out.io {
        if let io::Request::Spawn { spawn, .. } = request {
            dirs.push(spawn.dir.clone()).expect("four live invocations");
        }
    }
    dirs.into_boxed()
}
fn done(s: &mut Started, owner: Token, expected: git::Done) {
    assert_eq!(s.done(), Some((owner, expected)));
}
fn repository(s: &mut Started, owner: Token) {
    let running = spawn(s, b"gitdirs/7/app", b"count-objects");
    finish(s, &running, b"packs: 50\n", 0);
    done(s, owner, git::Done::Succeeded);
}
fn update(s: &mut Started, owner: Token) {
    let running = spawn(s, b"gitdirs/7/app", b"update-ref");
    finish(s, &running, b"", 0);
    assert_eq!(s.done(), None, "the fetched terminal waits for its view");
    super::view::empty(s);
    done(s, owner, git::Done::Fetched { commit: fixtures::COMMIT });
}
fn repacking(s: &mut Started) -> Running {
    pack(s, FIRST, Some(Token::new(7)), LATER);
    let count = spawn(s, b"stores/7", b"count-objects");
    finish(s, &count, b"packs: 51\n", 0);
    spawn(s, b"stores/7", b"repack")
}

#[test]
fn pack_waits_for_the_store_fetch_to_close_before_counting() {
    let mut s = started();
    fetch(&mut s, FIRST, Token::new(7));
    let fetching = spawn(&mut s, b"stores/7", b"fetch");
    pack(&mut s, SECOND, Some(Token::new(7)), LATER);
    assert!(directories(&s).is_empty(), "Pack waits behind the store fetch");
    finish(&mut s, &fetching, FETCHED, 0);
    update(&mut s, FIRST);
    let counting = spawn(&mut s, b"stores/7", b"count-objects");
    finish(&mut s, &counting, b"packs: 50\n", 0);
    repository(&mut s, SECOND);
    assert!(directories(&s).is_empty());
}

#[test]
fn fetch_waits_until_the_pack_store_part_ends_not_its_repository_part() {
    for repack in [false, true] {
        let mut s = started();
        pack(&mut s, FIRST, Some(Token::new(7)), LATER);
        let mut holding = spawn(&mut s, b"stores/7", b"count-objects");
        if repack {
            finish(&mut s, &holding, b"packs: 51\n", 0);
            holding = spawn(&mut s, b"stores/7", b"repack");
        }
        fetch(&mut s, SECOND, Token::new(7));
        assert!(directories(&s).is_empty(), "Fetch waits for Pack's exclusive store part");
        finish(&mut s, &holding, if repack { b"" } else { b"packs: 50\n" }, 0);
        let counting_repository = spawn(&mut s, b"gitdirs/7/app", b"count-objects");
        let fetching = spawn(&mut s, b"stores/7", b"fetch");
        finish(&mut s, &fetching, FETCHED, 0);
        update(&mut s, SECOND);
        finish(&mut s, &counting_repository, b"packs: 50\n", 0);
        done(&mut s, FIRST, git::Done::Succeeded);
    }
}

#[test]
fn a_second_pack_waits_and_a_repository_only_pack_takes_no_store_lock() {
    let mut s = started();
    let holding = repacking(&mut s);
    pack(&mut s, SECOND, Some(Token::new(7)), LATER);
    assert!(directories(&s).is_empty(), "a second Pack waits for the same store");
    pack(&mut s, THIRD, None, LATER);
    repository(&mut s, THIRD);
    finish(&mut s, &holding, b"", 0);
    let first_repository = spawn(&mut s, b"gitdirs/7/app", b"count-objects");
    let second_count = spawn(&mut s, b"stores/7", b"count-objects");
    finish(&mut s, &second_count, b"packs: 50\n", 0);
    repository(&mut s, SECOND);
    finish(&mut s, &first_repository, b"packs: 50\n", 0);
    done(&mut s, FIRST, git::Done::Succeeded);
}

#[test]
fn store_fetches_share_the_lock_and_another_store_is_never_held() {
    let mut s = started();
    fetch(&mut s, FIRST, Token::new(7));
    let first = spawn(&mut s, b"stores/7", b"fetch");
    fetch(&mut s, SECOND, Token::new(7));
    let second = spawn(&mut s, b"stores/7", b"fetch");
    pack(&mut s, THIRD, Some(Token::new(7)), LATER);
    assert!(directories(&s).is_empty());
    fetch(&mut s, FOURTH, Token::new(8));
    let other = spawn(&mut s, b"stores/8", b"fetch");
    finish(&mut s, &first, FETCHED, 0);
    update(&mut s, FIRST);
    assert!(directories(&s).is_empty(), "Pack still waits for the second reader");
    finish(&mut s, &second, FETCHED, 0);
    update(&mut s, SECOND);
    let count = spawn(&mut s, b"stores/7", b"count-objects");
    finish(&mut s, &count, b"packs: 51\n", 0);
    let repack = spawn(&mut s, b"stores/7", b"repack");
    finish(&mut s, &other, FETCHED, 0);
    update(&mut s, FOURTH);
    s.component.reclaim();
    fetch(&mut s, FOURTH, Token::new(8));
    let other = spawn(&mut s, b"stores/8", b"fetch");
    finish(&mut s, &other, FETCHED, 0);
    update(&mut s, FOURTH);
    finish(&mut s, &repack, b"", 0);
    repository(&mut s, THIRD);
}

#[test]
fn a_fetch_arriving_after_a_waiting_pack_stays_behind_it() {
    let mut s = started();
    fetch(&mut s, FIRST, Token::new(7));
    let first = spawn(&mut s, b"stores/7", b"fetch");
    pack(&mut s, SECOND, Some(Token::new(7)), LATER);
    fetch(&mut s, THIRD, Token::new(7));
    fetch(&mut s, FOURTH, Token::new(7));
    assert!(directories(&s).is_empty(), "a later reader cannot pass a waiting Pack");
    finish(&mut s, &first, FETCHED, 0);
    update(&mut s, FIRST);
    let counting = spawn(&mut s, b"stores/7", b"count-objects");
    assert!(directories(&s).is_empty(), "the later reader waits through the store count");
    finish(&mut s, &counting, b"packs: 50\n", 0);
    let repository_count = spawn(&mut s, b"gitdirs/7/app", b"count-objects");
    let later = spawn(&mut s, b"stores/7", b"fetch");
    let last = spawn(&mut s, b"stores/7", b"fetch");
    finish(&mut s, &later, FETCHED, 0);
    update(&mut s, THIRD);
    finish(&mut s, &last, FETCHED, 0);
    update(&mut s, FOURTH);
    finish(&mut s, &repository_count, b"packs: 50\n", 0);
    done(&mut s, SECOND, git::Done::Succeeded);
}

#[test]
fn a_waiters_deadline_or_cancel_ends_it_unspawned_and_unblocks_those_behind() {
    for deadline in [true, false] {
        let mut s = started();
        fetch(&mut s, FIRST, Token::new(7));
        let first = spawn(&mut s, b"stores/7", b"fetch");
        pack(&mut s, SECOND, Some(Token::new(7)), fixtures::DEADLINE);
        fetch(&mut s, THIRD, Token::new(7));
        assert!(directories(&s).is_empty());
        if deadline {
            s.fire(fixtures::DEADLINE);
        } else {
            s.down(Request::CancelIo { owner: SECOND });
        }
        assert!(s.signals().is_empty(), "waiting has no child to kill");
        done(
            &mut s,
            SECOND,
            git::Done::Failed { fault: if deadline { git::Fault::TimedOut } else { git::Fault::Cancelled } },
        );
        let later = spawn(&mut s, b"stores/7", b"fetch");
        finish(&mut s, &later, FETCHED, 0);
        update(&mut s, THIRD);
        finish(&mut s, &first, FETCHED, 0);
        update(&mut s, FIRST);
        assert_eq!(s.done(), None, "each waiter ends once");
    }
}

#[test]
fn a_failed_store_part_releases_its_lock_for_the_next_operation() {
    for (repack, stop) in
        [(false, None), (true, None), (true, Some(git::Fault::TimedOut)), (true, Some(git::Fault::Cancelled))]
    {
        let mut s = started();
        pack(&mut s, FIRST, Some(Token::new(7)), LATER);
        let mut holding = spawn(&mut s, b"stores/7", b"count-objects");
        if repack {
            finish(&mut s, &holding, b"packs: 51\n", 0);
            holding = spawn(&mut s, b"stores/7", b"repack");
        }
        fetch_until(&mut s, SECOND, Token::new(7), Time::from_nanos(20000));
        assert!(directories(&s).is_empty());
        if let Some(fault) = stop {
            if fault == git::Fault::TimedOut {
                s.fire(LATER);
            } else {
                s.down(Request::CancelIo { owner: FIRST });
            }
            assert_eq!(s.done(), None, "the lock remains held until close");
            assert!(directories(&s).is_empty());
            s.below.io_owner = Some(holding.owner);
            s.closed(holding.child, &holding.pipes);
        } else {
            finish(&mut s, &holding, b"", 128);
        }
        done(
            &mut s,
            FIRST,
            git::Done::FailedWithOutput {
                fault: stop.unwrap_or(git::Fault::Broken),
                diagnostic: git::PushDiagnostic::empty().named(if repack { b"repack" } else { b"count-objects" }),
            },
        );
        let fetching = spawn(&mut s, b"stores/7", b"fetch");
        finish(&mut s, &fetching, FETCHED, 0);
        update(&mut s, SECOND);
    }
}

#[test]
fn a_waiting_fetch_ends_without_a_child_and_leaves_the_queue_moving() {
    for deadline in [true, false] {
        let mut s = started();
        let holding = repacking(&mut s);
        fetch_until(&mut s, SECOND, Token::new(7), fixtures::DEADLINE);
        pack(&mut s, THIRD, Some(Token::new(7)), LATER);
        assert!(directories(&s).is_empty());
        if deadline {
            s.fire(fixtures::DEADLINE);
        } else {
            s.down(Request::CancelIo { owner: SECOND });
        }
        assert!(s.signals().is_empty(), "a waiting fetch has no child");
        done(
            &mut s,
            SECOND,
            git::Done::Failed { fault: if deadline { git::Fault::TimedOut } else { git::Fault::Cancelled } },
        );
        assert!(directories(&s).is_empty(), "the holder still owns the store");
        finish(&mut s, &holding, b"", 0);
        repository(&mut s, FIRST);
        let next = spawn(&mut s, b"stores/7", b"count-objects");
        finish(&mut s, &next, b"packs: 50\n", 0);
        repository(&mut s, THIRD);
        assert_eq!(s.done(), None);
    }
}

#[test]
fn shutdown_closes_its_directories_when_the_last_store_waiter_expires() {
    let mut s = started();
    let holding = repacking(&mut s);
    fetch_until(&mut s, SECOND, Token::new(7), fixtures::DEADLINE);
    s.down(Request::Shutdown);
    s.below.io_owner = Some(holding.owner);
    s.end(holding.pipes[0]);
    s.end(holding.pipes[1]);
    s.exited(holding.child, 128);
    for pipe in &holding.pipes {
        s.component.up(s.now, io::Event::Closed { owner: *pipe }, &mut s.below.out);
    }
    s.now = fixtures::DEADLINE;
    s.component.up(s.now, io::Event::Closed { owner: holding.owner }, &mut s.below.out);
    done(
        &mut s,
        FIRST,
        git::Done::FailedWithOutput {
            fault: git::Fault::Broken,
            diagnostic: git::PushDiagnostic::empty().named(b"repack"),
        },
    );
    done(&mut s, SECOND, git::Done::Failed { fault: git::Fault::TimedOut });
    assert!(!s.below.out.kernel.is_empty(), "shutdown must close directories after its last waiter ends");
    for _ in 0_u8..5 {
        s.file_operation();
        s.complete_ok();
    }
    assert!(s.component.is_empty());
}
