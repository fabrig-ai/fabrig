//! Delivery's checkout metadata and exact commit parents (protocol/git.md, 4.7).
use super::fixtures::{self, Started};
use crate::{
    Request,
    invocation::{self, Command},
};
use skein_io::Request as Io;
use temper_worker_domain_checkout::git;

fn delivery() -> Started {
    delivery_with_fresh_commit(false)
}

fn delivery_with_fresh_commit(fresh: bool) -> Started {
    let mut started = Started::new();
    let mut op = fixtures::commit_op(None);
    let git::Op::Commit { snapshot, always, .. } = &mut op else { unreachable!() };
    *always = fresh;
    *snapshot = Some(git::Snapshot { remote: Box::from(fixtures::REMOTE), account: Some(7) });
    started.down(Request::Io {
        owner: fixtures::OWNER,
        op,
        deadline: fixtures::DEADLINE,
        identity: Some(fixtures::identity()),
    });
    started
}

fn answer(started: &mut Started, command: Command, code: u8, stdout: &[u8]) {
    let request = started.below.out.io.pop().expect("next invocation");
    let Io::Spawn { spawn, .. } = &request else { panic!("spawn: {request:?}") };
    assert_eq!(spawn.args, invocation::arguments(&command));
    if let Command::SnapshotFetch { .. } = command {
        let mut credential = false;
        for word in &spawn.env {
            if word.starts_with(b"GIT_CONFIG_VALUE_0=Authorization: ") {
                credential = true;
            }
        }
        assert!(credential);
    }
    started.below.out.io.push(request);
    let (child, pipes) = started.spawned();
    let output = usize::from(pipes.len() == 3);
    started.stdout(pipes[output], stdout);
    started.end(pipes[output]);
    started.end(pipes[output.checked_add(1).expect("stderr pipe")]);
    started.exited(child, code);
    started.closed(child, &pipes);
}

fn line(commit: git::Commit) -> Box<[u8]> {
    invocation::joined(&[&invocation::hex(commit), b"\n"])
}

fn read_head(started: &mut Started, head: git::Commit, second: Option<git::Commit>) {
    let bytes = match second {
        Some(second) => {
            invocation::joined(&[&invocation::hex(head), b" ", &invocation::hex(fixtures::PARENT), b" ", &line(second)])
        }
        None => invocation::joined(&[&invocation::hex(head), b" ", &line(fixtures::PARENT)]),
    };
    answer(started, Command::SnapshotHead, 0, &bytes);
}

fn remote(
    started: &mut Started,
    head: git::Commit,
    merging: Option<git::Commit>,
    second: git::Commit,
    reachable: bool,
) {
    answer(started, Command::SnapshotFetch { remote: fixtures::REMOTE.into(), head, merging, second }, 0, b"");
    let parent_line = line(second);
    answer(
        started,
        Command::SnapshotReachable { head, merging, second },
        0,
        if reachable { &parent_line } else { b"" },
    );
}

#[test]
fn a_staged_merge_delivered_keeps_its_second_parent() {
    let mut started = delivery();
    let head = fixtures::COMMIT;
    let second = fixtures::PARENT;
    read_head(&mut started, head, None);
    answer(&mut started, Command::SnapshotMerge { head, second: None }, 0, &line(second));
    remote(&mut started, head, Some(second), second, true);
    // Staging the resolution removed the unmerged index entries; MERGE_HEAD remains.
    answer(&mut started, Command::Unmerged, 0, b"");
    answer(&mut started, Command::Add, 0, b"");
    answer(&mut started, Command::WriteTree, 0, &line(head));
    answer(&mut started, Command::CommitTree { tree: head, parent: head, merging: Some(second) }, 0, &line(second));
    answer(&mut started, Command::SnapshotReset { commit: second }, 0, b"");
    assert_eq!(started.done(), Some((fixtures::OWNER, git::Done::Committed { commit: second })));
    assert!(started.below.out.io.is_empty());
}

#[test]
fn a_local_commit_is_delivered_without_replacement() {
    for second in [None, Some(fixtures::PARENT)] {
        let mut started = delivery();
        let head = fixtures::COMMIT;
        read_head(&mut started, head, second);
        answer(&mut started, Command::SnapshotMerge { head, second }, 1, b"");
        if let Some(second) = second {
            remote(&mut started, head, None, second, true);
        }
        answer(&mut started, Command::Add, 0, b"");
        answer(&mut started, Command::DiffIndex { parent: head }, 0, b"");
        assert_eq!(started.done(), Some((fixtures::OWNER, git::Done::Committed { commit: head })));
        assert!(started.below.out.io.is_empty(), "no replacement commit or reset");
    }
}

#[test]
fn a_second_parent_only_in_the_checkout_refuses_delivery_before_staging() {
    for committed in [false, true] {
        let mut started = delivery();
        let head = fixtures::COMMIT;
        let second = fixtures::PARENT;
        let parent = committed.then_some(second);
        read_head(&mut started, head, parent);
        let parent_line = line(second);
        answer(
            &mut started,
            Command::SnapshotMerge { head, second: parent },
            u8::from(committed),
            if committed { b"" } else { &parent_line },
        );
        remote(&mut started, head, (!committed).then_some(second), second, false);
        let Some((_, git::Done::FailedWithOutput { fault, diagnostic })) = started.done() else { panic!("refused") };
        assert_eq!(fault, git::Fault::Refused);
        assert_eq!(diagnostic.output(), b"delivery second parent is not reachable from the branch's remote");
        assert!(started.below.out.io.is_empty());
    }
}

#[test]
fn a_checked_delivery_after_an_unchecked_save_makes_a_fresh_equal_tree_commit() {
    let mut started = delivery_with_fresh_commit(true);
    let saved = fixtures::COMMIT;
    let checked = fixtures::PARENT;
    read_head(&mut started, saved, None);
    answer(&mut started, Command::SnapshotMerge { head: saved, second: None }, 1, b"");
    answer(&mut started, Command::Add, 0, b"");
    answer(&mut started, Command::WriteTree, 0, &line(saved));
    answer(&mut started, Command::CommitTree { tree: saved, parent: saved, merging: None }, 0, &line(checked));
    answer(&mut started, Command::SnapshotReset { commit: checked }, 0, b"");
    assert_eq!(started.done(), Some((fixtures::OWNER, git::Done::Committed { commit: checked })));
    assert_ne!(saved, checked);
    assert!(started.below.out.io.is_empty());
}
