//! Derived bookkeeping agrees with nodes (hearth/docs/design/testing/README.md, 4.1).
use crate::linux::{At, Handle, How, Ids, Item, Machine, Op, Pid, Refusal, Reply, Sizes};

fn checked(machine: &mut Machine, pid: Pid, op: Op) -> Result<Reply, Refusal> {
    let (_, before, mut changed) = machine.bookkeeping(pid);
    let reply = machine.call(pid, op);
    let (used, after, observed) = machine.bookkeeping(pid);
    for (node, facts) in &after {
        if before.get(node).map(|facts| facts.changes) != Some(facts.changes) {
            changed.insert(*node);
        }
    }
    for node in before.keys() {
        if !after.contains_key(node) {
            changed.insert(*node);
        }
    }
    assert_eq!(observed, changed, "the nodes changed by each call");
    assert_eq!(machine.counts(pid).changed, u64::try_from(changed.len()).unwrap());
    assert_eq!(used, after.values().map(|facts| facts.size).sum::<u64>(), "held bytes");
    reply
}

fn open(machine: &mut Machine, pid: Pid, path: &[u8], how: How) -> Handle {
    let reply = checked(machine, pid, Op::Open { at: At::Root, path: path.into(), how });
    let Ok(Reply::Opened(handle)) = reply else { panic!("expected an open handle, got {reply:?}") };
    handle
}

#[test]
fn held_bytes_and_changed_nodes_follow_creates_writes_truncations_and_removals() {
    const OWNER: Ids = Ids { user: 1000, group: 1000 };
    let mut machine = Machine::new(26, Sizes { bytes: 10, ..Sizes::tiny() });
    machine.lay(&[
        Item::directory(b"/srv", 0o755).owner(OWNER),
        Item::file(b"/srv/laid", 0o644, b"ab"),
        Item::link(b"/srv/link", b"laid"),
        Item::hard_link(b"/srv/alias", b"/srv/laid"),
    ]);
    let pid = machine.login(OWNER, b"/srv");
    assert_eq!(machine.bookkeeping(pid).0, 6, "hard links hold their bytes once");
    let dir = open(&mut machine, pid, b"/srv", How::Directory);
    let file = open(&mut machine, pid, b"/srv/new", How::Create { mode: None });
    for op in [
        Op::MakeDirectory { dir, path: b"child".as_slice().into(), mode: 0o755 },
        Op::Remove { dir, path: b"child".as_slice().into(), directory: true },
        Op::Write { file, at: 1, bytes: b"abc".as_slice().into() },
        Op::Write { file, at: 0, bytes: b"x".as_slice().into() },
        Op::Truncate { file, length: 1 },
        Op::Truncate { file, length: 3 },
        Op::Remove { dir, path: b"laid".as_slice().into(), directory: false },
        Op::Remove { dir, path: b"alias".as_slice().into(), directory: false },
        Op::Remove { dir, path: b"new".as_slice().into(), directory: false },
    ] {
        assert_eq!(checked(&mut machine, pid, op), Ok(Reply::Done));
    }
    assert_eq!(machine.bookkeeping(pid).0, 7, "an unlinked open file still holds its bytes");
    assert_eq!(checked(&mut machine, pid, Op::Close { handle: file }), Ok(Reply::Done));
    assert_eq!(machine.bookkeeping(pid).0, 4, "closing the last handle frees the unlinked file");
    assert_eq!(
        checked(&mut machine, pid, Op::Remove { dir, path: b"link".as_slice().into(), directory: false }),
        Ok(Reply::Done)
    );
    assert_eq!(machine.bookkeeping(pid).0, 0, "removing the link frees its target bytes");
    assert_eq!(
        checked(&mut machine, pid, Op::Open { at: At::Root, path: b"/missing".as_slice().into(), how: How::Read }),
        Err(Refusal::NotFound)
    );
    let file = open(&mut machine, pid, b"/srv/full", How::Create { mode: None });
    assert_eq!(checked(&mut machine, pid, Op::Truncate { file, length: 10 }), Ok(Reply::Done));
    assert_eq!(machine.bookkeeping(pid).0, 10, "the exact truncate uses the scenario byte bound");
    let reader = open(&mut machine, pid, b"/srv/full", How::Read);
    assert_eq!(
        checked(&mut machine, pid, Op::Read { file: reader, at: 0, most: 10 }),
        Ok(Reply::Read(vec![0; 10].into()))
    );
}
