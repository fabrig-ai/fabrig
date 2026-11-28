//! The domain-face disk faults, one translated operation at a time.
use skein_lib::Token;
use temper_fake_checkout::{Checkout, git};
use temper_worker_checkout_world::{
    forge::Forge,
    translate::{
        commit, cut, disk_snapshot as snapshot, perform, perform_with_capacity, perform_within_capacity, used,
    },
};
use temper_worker_domain_checkout::git::{Done, Fault, Into, Op, Place, Reds, Want};

const STORE: Token = Token::new(7);

fn place() -> Place {
    Place { workspace: Token::new(3), repository: Box::from(b"r".as_slice()), git: None }
}
fn fetch(into: Into) -> Op {
    Op::Fetch {
        at: place(),
        store: STORE,
        remote: Box::from(b"forge/r".as_slice()),
        want: Want::Default,
        into,
        identity: 1,
        account: None,
    }
}
fn check_out(head: u64) -> Op {
    Op::CheckOut { at: place(), commit: commit(head), clean: true, identity: 1, account: None }
}
fn merge(head: u64) -> Op {
    Op::Merge { at: place(), theirs: commit(head), identity: 1, account: None }
}
fn committing(head: u64) -> Op {
    Op::Commit {
        at: place(),
        parent: commit(head),
        merging: None,
        always: true,
        title: Box::from(b"change".as_slice()),
        body: Box::from([]),
        identity: 1,
        snapshot: None,
    }
}
fn make() -> Op {
    Op::Make { workspace: place().workspace, stores: Box::from([STORE]) }
}
fn store() -> Op {
    Op::Store {
        store: STORE,
        aside: None,
        remote: Box::from(b"forge/r".as_slice()),
        identity: 1,
        account: None,
        git: None,
    }
}
fn clone_repository() -> Op {
    Op::Clone { at: place(), remote: Box::from(b"forge/r".as_slice()), store: STORE }
}
fn fixture() -> (Forge, Checkout, u64) {
    let mut forge = Forge::new(1);
    let head = forge.repository(
        b"forge/r",
        b"main",
        [(b"a".to_vec(), b"old a".to_vec()), (b"b".to_vec(), b"old b".to_vec()), (b"c".to_vec(), b"old c".to_vec())]
            .into(),
    );
    let mut disk = Checkout::new();
    for op in [make(), store(), clone_repository()] {
        assert_eq!(perform(&mut forge, &mut disk, op), Done::Succeeded);
    }
    assert_eq!(perform(&mut forge, &mut disk, fetch(Into::Fetched)), Done::Fetched { commit: commit(head) });
    assert_eq!(perform(&mut forge, &mut disk, check_out(head)), Done::Succeeded);
    (forge, disk, head)
}
fn advanced(forge: &mut Forge) -> u64 {
    forge.advance(b"forge/r", b"main", b"a", b"new a");
    forge.advance(b"forge/r", b"main", b"b", b"new b");
    forge.advance(b"forge/r", b"main", b"c", b"new c")
}
fn pack_count(disk: &Checkout, directory: &[u8]) -> usize {
    disk.tree(&[directory, b"/objects/pack"].concat()).len()
}

#[test]
fn index_lock_refuses_checkout_merge_and_commit_without_writes() {
    let (mut forge, mut disk, head) = fixture();
    disk.write(b"ws/3/r/.git/index.lock", b"held");
    let before = snapshot(&disk);
    for op in [check_out(head), merge(head), committing(head)] {
        assert_eq!(perform(&mut forge, &mut disk, op), Done::Failed { fault: Fault::Broken });
        assert_eq!(snapshot(&disk), before);
    }
}

#[test]
fn reference_locks_refuse_both_fetch_destinations_and_point() {
    for into in [Into::Fetched, Into::Landing] {
        let reference = match into {
            Into::Fetched => b"fetched".as_slice(),
            Into::Landing => b"landing",
        };
        for directory in [b"stores/7/refs/temper/3/".as_slice(), b"ws/3/r/.git/refs/temper/"] {
            let (mut forge, mut disk, head) = fixture();
            advanced(&mut forge);
            let lock = [directory, reference, b".lock"].concat();
            disk.write(&lock, b"held");
            let before = snapshot(&disk);
            assert_eq!(perform(&mut forge, &mut disk, fetch(into)), Done::Failed { fault: Fault::Broken });
            assert_eq!(snapshot(&disk), before);
            if directory.starts_with(b"ws/") {
                assert_eq!(
                    perform(&mut forge, &mut disk, Op::Point { at: place(), into, commit: commit(head) }),
                    Done::Failed { fault: Fault::Broken }
                );
                assert_eq!(snapshot(&disk), before);
            }
        }
    }
}

#[test]
fn make_takes_workspace_and_its_store_locks_only() {
    let (mut forge, mut disk, _) = fixture();
    disk.write(b"ws/3/r/.git/index.lock", b"held");
    disk.write(b"stores/7/refs/temper/3/fetched.lock", b"held");
    disk.write(b"stores/7/refs/temper/4/fetched.lock", b"other workspace");
    let other = snapshot(&disk).remove(b"stores/7/refs/temper/4/fetched.lock".as_slice()).expect("other lock");
    assert_eq!(perform(&mut forge, &mut disk, make()), Done::Succeeded);
    assert!(disk.tree(b"ws/3").is_empty());
    assert!(!disk.exists(b"stores/7/refs/temper/3"));
    assert_eq!(snapshot(&disk).get(b"stores/7/refs/temper/4/fetched.lock".as_slice()), Some(&other));
    assert!(disk.exists(b"stores/7/objects"));
}

#[test]
fn cut_fetch_imports_objects_but_leaves_references_and_a_lock() {
    for into in [Into::Fetched, Into::Landing] {
        let (mut forge, mut disk, head) = fixture();
        assert_eq!(
            perform(&mut forge, &mut disk, Op::Point { at: place(), into, commit: commit(head) }),
            Done::Succeeded
        );
        let next = advanced(&mut forge);
        let before = snapshot(&disk);
        assert_eq!(cut(&mut forge, &mut disk, fetch(into)), Done::Failed { fault: Fault::TimedOut });
        assert!(git::object_present(&disk, b"stores/7", next));
        let reference = match into {
            Into::Fetched => "fetched",
            Into::Landing => "landing",
        };
        assert!(disk.exists(format!("stores/7/refs/temper/3/{reference}.lock").as_bytes()));
        for (path, value) in before {
            assert_eq!(snapshot(&disk).get(&path), Some(&value), "existing file {path:?}");
        }
        assert_eq!(pack_count(&disk, b"stores/7"), 1, "a cut leaves no successful-fetch pack");
    }
}

fn partial(merging: bool) {
    let (mut forge, mut disk, head) = fixture();
    let next = advanced(&mut forge);
    perform(&mut forge, &mut disk, fetch(Into::Fetched));
    let before = snapshot(&disk);
    let op = if merging { merge(next) } else { check_out(next) };
    assert_eq!(cut(&mut forge, &mut disk, op), Done::Failed { fault: Fault::TimedOut });
    assert_eq!(disk.content(b"ws/3/r/a"), Some(b"new a".as_slice()));
    assert_eq!(disk.content(b"ws/3/r/b"), Some(b"new b".as_slice()), "ceil(3/2) files");
    assert_eq!(disk.content(b"ws/3/r/c"), Some(b"old c".as_slice()));
    assert_eq!(disk.version(b"ws/3/r/c"), Some(before[b"ws/3/r/c".as_slice()].1));
    assert_eq!(disk.content(b"ws/3/r/.git/temper-head"), Some(head.to_le_bytes().as_slice()));
    assert!(disk.exists(b"ws/3/r/.git/index.lock"));
    assert_ne!(disk.content(b"ws/3/r/.git/index").expect("partial index"), []);
    assert!(!disk.exists(b"ws/3/r/.git/MERGE_HEAD"));
    assert!(!disk.exists(b"ws/3/r/.git/temper-conflicts"));
    assert_eq!(pack_count(&disk, b"ws/3/r/.git"), 1);
    for (path, value) in before {
        if path != b"ws/3/r/a" && path != b"ws/3/r/b" {
            assert_eq!(snapshot(&disk).get(&path), Some(&value));
        }
    }
}
#[test]
fn cut_checkout_writes_only_the_first_half_without_moving_head() {
    partial(false);
}
#[test]
fn cut_merge_writes_only_the_first_half_without_merge_metadata() {
    partial(true);
}

#[test]
fn capacity_refuses_writes_atomically_and_allows_them_below_the_bound() {
    for over in [0, 1] {
        let (_, _, head) = fixture();
        let operations = [
            store(),
            clone_repository(),
            fetch(Into::Fetched),
            Op::Point { at: place(), into: Into::Fetched, commit: commit(head) },
            check_out(head),
            merge(head),
            committing(head),
            Op::Reset { at: place(), commit: commit(head) },
            Op::Reds { at: place(), action: Reds::Prepare { wait_ms: 1, rerun_processes: 1 } },
            Op::Reds { at: place(), action: Reds::Write { reds: Box::from([]) } },
        ];
        for op in operations {
            let (mut forge, mut disk, _) = fixture();
            let before = snapshot(&disk);
            let capacity = used(&disk) - over;
            let done = perform_with_capacity(&mut forge, &mut disk, op.clone(), Some(capacity));
            let Done::FailedWithOutput { fault, diagnostic } = done else {
                panic!("capacity accepted {op:?}: {done:?}");
            };
            assert_eq!(fault, Fault::Broken);
            assert_eq!(diagnostic.output(), b"No space left on device");
            assert_eq!(snapshot(&disk), before);
            // One byte free admits exactly the ordinary operation, even if its writes exceed it.
            let capacity = used(&disk) + 1;
            let admitted = perform_with_capacity(&mut forge, &mut disk, op.clone(), Some(capacity));
            let (mut ordinary_forge, mut ordinary_disk, _) = fixture();
            let ordinary = perform(&mut ordinary_forge, &mut ordinary_disk, op);
            assert_eq!(admitted, ordinary);
            assert_eq!(snapshot(&disk), snapshot(&ordinary_disk));
        }
    }
}

#[test]
fn a_capacity_limited_merge_keeps_the_conflict_bound() {
    let (mut forge, mut disk, base) = fixture();
    disk.write(b"ws/3/r/a", b"our a");
    disk.write(b"ws/3/r/b", b"our b");
    git::commit(&mut forge, &mut disk, b"ws/3/r", base, b"ours").expect("parent").expect("changed");
    let theirs = advanced(&mut forge);
    perform(&mut forge, &mut disk, fetch(Into::Fetched));
    let before = snapshot(&disk);
    let capacity = used(&disk);
    let refused = perform_within_capacity(&mut forge, &mut disk, merge(theirs), 1, Some(capacity));
    assert!(matches!(refused, Done::FailedWithOutput { fault: Fault::Broken, .. }));
    assert_eq!(snapshot(&disk), before, "full disk refuses before writing conflict markers");
    let merged = perform_within_capacity(&mut forge, &mut disk, merge(theirs), 1, Some(capacity + 1));
    let Done::Conflicted { files, cut } = merged else { panic!("conflicting merge: {merged:?}") };
    assert_eq!(files.len(), 1, "the composed caller's conflict limit is retained");
    assert!(cut, "the second conflict is reported as cut");
}

#[test]
fn a_fetched_commit_s_message_is_read_whole() {
    const REMOTE: &[u8] = b"forge/r";
    const BRANCH: &[u8] = b"main";
    const MESSAGE: &[u8] = b"Change the page\n\nLook-request: 12\n";
    let (mut forge, mut disk, base) = fixture();
    let tree = git::Remote::tree(&forge, base);
    let head = git::Remote::store(&mut forge, base, None, true, tree, MESSAGE).expect("a fresh commit");
    git::Remote::push(&mut forge, REMOTE, BRANCH, head, Some(base)).expect("another party advances the branch");
    perform(&mut forge, &mut disk, fetch(Into::Fetched));
    forge.set_reachable(REMOTE, false);
    let before = snapshot(&disk);
    assert_eq!(
        perform_with_capacity(&mut forge, &mut disk, Op::Message { at: place(), commit: commit(head) }, Some(0)),
        Done::Message { message: Some(MESSAGE.into()) }
    );
    assert_eq!(snapshot(&disk), before);
}

#[test]
fn capacity_counts_regular_bytes_and_exempts_reads_make_and_pack() {
    assert_eq!(used(&Checkout::new()), 0);
    let (mut forge, mut disk, head) = fixture();
    let bytes = used(&disk);
    disk.mkdir(b"directory");
    disk.link(b"link", b"not counted");
    disk.special(b"special");
    assert_eq!(used(&disk), bytes);
    disk.write(b"extra", b"123");
    assert_eq!(used(&disk), bytes + 3);
    let reads = [
        Op::Look { at: place(), remote: Box::from(b"forge/r".as_slice()), store: STORE },
        Op::DiffPaths { at: place(), before: commit(head), after: commit(head), markdown: true },
        Op::Brings { at: place(), head: commit(head), base: commit(head), upstream: commit(head) },
        Op::IsAncestor { at: place(), ancestor: commit(head), commit: commit(head) },
        Op::Reds { at: place(), action: Reds::Read { check: 1 } },
        Op::Reds { at: place(), action: Reds::Clear },
        Op::Pack { at: place(), store: Some(STORE) },
    ];
    for op in reads {
        let before = snapshot(&disk);
        let expected = perform(&mut forge, &mut disk, op.clone());
        let actual = perform_with_capacity(&mut forge, &mut disk, op, Some(0));
        assert_eq!(actual, expected);
        assert_eq!(snapshot(&disk), before);
    }
    assert_eq!(perform_with_capacity(&mut forge, &mut disk, make(), Some(0)), Done::Succeeded);
    assert!(disk.tree(b"ws/3").is_empty());
}

#[test]
fn successful_fetch_checkout_and_merge_leave_numbered_packs_and_pack_uses_fifty() {
    let (mut forge, mut disk, head) = fixture();
    perform(&mut forge, &mut disk, fetch(Into::Fetched));
    perform(&mut forge, &mut disk, check_out(head));
    perform(&mut forge, &mut disk, merge(head));
    assert_eq!(disk.content(b"stores/7/objects/pack/pack-2.pack"), Some(b"pack\n".as_slice()));
    assert_eq!(disk.content(b"ws/3/r/.git/objects/pack/pack-3.pack"), Some(b"pack\n".as_slice()));
    for n in 1..=50 {
        disk.write(format!("stores/7/objects/pack/pack-{n}.pack").as_bytes(), b"pack\n");
        disk.write(format!("ws/3/r/.git/objects/pack/pack-{n}.pack").as_bytes(), b"pack\n");
    }
    let before = snapshot(&disk);
    let pack = Op::Pack { at: place(), store: Some(STORE) };
    assert_eq!(perform(&mut forge, &mut disk, pack.clone()), Done::Succeeded);
    assert_eq!(snapshot(&disk), before, "fifty needs no packing");
    disk.write(b"stores/7/objects/pack/pack-51.pack", b"pack\n");
    disk.write(b"ws/3/r/.git/objects/pack/pack-51.pack", b"pack\n");
    assert_eq!(
        perform_with_capacity(&mut forge, &mut disk, Op::Pack { at: place(), store: None }, Some(0)),
        Done::Succeeded
    );
    assert_eq!(pack_count(&disk, b"stores/7"), 51, "unnamed store stays");
    assert_eq!(pack_count(&disk, b"ws/3/r/.git"), 1);
    assert_eq!(perform_with_capacity(&mut forge, &mut disk, pack, Some(0)), Done::Succeeded);
    assert_eq!(pack_count(&disk, b"stores/7"), 1);
    assert_eq!(pack_count(&disk, b"ws/3/r/.git"), 1);
    assert_eq!(disk.content(b"stores/7/objects/pack/pack-52.pack"), Some(b"pack\n".as_slice()));
    assert!(git::object_present(&disk, b"stores/7", head));
    assert_eq!(perform(&mut forge, &mut disk, check_out(head)), Done::Succeeded, "objects still usable");
    assert!(disk.exists(b"ws/3/r/.git/objects/pack/pack-53.pack"));
}

#[test]
fn counts_fetches_the_whole_published_commit_then_removes_the_stale_ref() {
    const LOCAL: &[u8] = b"ws/3/r/.git/refs/temper/counts";
    const STORE_REF: &[u8] = b"stores/7/refs/temper/3/counts";
    const REMOTE: &[u8] = b"forge/r";
    const FILE: &[u8] = b"{\"checks\":7}\n";
    let (mut forge, mut disk, head) = fixture();
    let op = Op::Counts { at: place(), store: STORE, remote: REMOTE.into(), identity: 1, account: None };
    assert_eq!(perform(&mut forge, &mut disk, op.clone()), Done::Succeeded);
    assert!(disk.content(LOCAL).is_none());
    let counts = forge.publish_counts(REMOTE, FILE);
    assert_eq!(forge.branches(REMOTE), [b"main".to_vec()], "a publication isn't a branch");
    assert_eq!(forge.branch(REMOTE, b"main"), Some(head));
    assert_eq!(perform(&mut forge, &mut disk, op.clone()), Done::Fetched { commit: commit(counts) });
    let tip = counts.to_le_bytes();
    for reference in [LOCAL, STORE_REF] {
        assert_eq!(disk.content(reference), Some(tip.as_slice()));
    }
    let contents = format!("stores/7/objects/trees/{counts}/counts.json");
    assert_eq!(disk.content(contents.as_bytes()), Some(FILE));
    assert!(git::object_present(&disk, b"ws/3/r", counts));

    forge.remove_counts(REMOTE);
    forge.set_reachable(REMOTE, false);
    assert_eq!(perform(&mut forge, &mut disk, op.clone()), Done::Failed { fault: Fault::Unreachable });
    assert_eq!(disk.content(LOCAL), Some(tip.as_slice()), "a failure doesn't delete the local ref");
    forge.set_reachable(REMOTE, true);
    assert_eq!(perform(&mut forge, &mut disk, op), Done::Succeeded);
    assert!(!disk.exists(LOCAL));
    assert_eq!(disk.content(STORE_REF), Some(tip.as_slice()), "the store still retains its fetched objects");
}
