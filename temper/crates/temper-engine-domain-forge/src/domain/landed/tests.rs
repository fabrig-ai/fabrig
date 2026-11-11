use crate::domain::{change, outcome_facts};
use crate::tests::{domain, env, fixtures};
use crate::{BranchHead, Domain, Key, Name, Request, Stored, What};
use skein_forge::api::Commit;
use skein_lib::{List, Queue};

const MERGE: Commit = [6; 32];
const FIRST: Commit = [10; 32];
const SECOND: Commit = [11; 32];
const THIRD: Commit = [12; 32];

fn main() -> Name {
    Name { forge: 1, repository: 2, what: What::Branch(Box::new([Box::from(b"main".as_slice())])) }
}

fn merging() -> Domain {
    let mut d = domain();
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.change.state = change::State::Landing { head: fixtures::HEAD };
    d.changes.insert(row.task, row).unwrap();
    let head = BranchHead { name: main(), commit: fixtures::BASE, owned: true };
    d.heads.insert(head.name.clone(), head).unwrap();
    d
}

fn add_change(d: &mut Domain, task: u64, settled: bool) {
    let mut row = fixtures::opening_row();
    row.task = task;
    row.change.task = task;
    if settled {
        row.ending = Some(crate::Ending::Cancelled);
        row.change.state = change::State::Ended;
    }
    d.changes.insert(task, row).unwrap();
}

fn full(d: &mut Domain, owner: u64) {
    let capacity = env().limits.landings.max(env().limits.changes);
    for number in 0..capacity {
        let byte = u8::try_from(number).unwrap().checked_add(10).unwrap();
        d.landed.insert([byte; 32], owner).unwrap();
    }
    assert_eq!(d.landed.len(), capacity);
}

fn merged(d: &mut Domain, commit: Commit) -> Queue<Request> {
    let mut out = Queue::uncapped();
    outcome_facts(
        d,
        &env(),
        fixtures::OPEN,
        fixtures::CHANGE,
        skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Merged(commit), found: false },
        &mut out,
    );
    out
}

fn erasures(out: &Queue<Request>) -> Box<[Commit]> {
    let mut commits = List::uncapped();
    for request in out {
        if let Request::Erase { key: Key::Landed(commit) } = request {
            commits.push(*commit).unwrap();
        }
    }
    commits.into_boxed()
}

fn saved(out: &Queue<Request>, record: Stored) -> bool {
    for request in out {
        if *request == (Request::Save { read_afresh: None, release: None, record: record.clone() }) {
            return true;
        }
    }
    false
}

#[test]
fn full_landed_table_still_advances_and_saves_merge() {
    let mut d = merging();
    full(&mut d, 102);
    d.landed.insert(SECOND, 100).unwrap();
    d.landed.insert(THIRD, 101).unwrap();
    let out = merged(&mut d, MERGE);
    assert!(saved(&out, Stored::BranchHead(BranchHead { name: main(), commit: MERGE, owned: true })));
    assert_eq!(d.heads.get(&main()).unwrap().commit, MERGE);
    assert!(saved(&out, Stored::Landed { commit: MERGE, task: fixtures::CHANGE }));
    assert_eq!(d.landed.get(&MERGE), Some(&fixtures::CHANGE));
    assert_eq!(erasures(&out).as_ref(), &[SECOND]);
    assert!(!d.landed.contains_key(&SECOND));
    assert_eq!(d.landed.len(), env().limits.landings.max(env().limits.changes));
}

#[test]
fn recordless_owner_goes_before_a_settled_change() {
    let mut d = merging();
    full(&mut d, 50);
    add_change(&mut d, 50, true);
    d.landed.insert(SECOND, 100).unwrap();
    let out = merged(&mut d, MERGE);
    assert_eq!(erasures(&out).as_ref(), &[SECOND]);
    assert_eq!(d.landed.get(&FIRST), Some(&50));
}

#[test]
fn lowest_settled_owner_goes_before_an_older_live_change() {
    let mut d = merging();
    full(&mut d, 50);
    add_change(&mut d, 50, false);
    add_change(&mut d, 52, true);
    add_change(&mut d, 53, true);
    assert!(d.droppable(52));
    assert!(d.droppable(53));
    d.landed.insert(SECOND, 53).unwrap();
    d.landed.insert(THIRD, 52).unwrap();
    let out = merged(&mut d, MERGE);
    assert_eq!(erasures(&out).as_ref(), &[THIRD]);
    assert_eq!(d.landed.get(&FIRST), Some(&50));
}

#[test]
fn lowest_live_owner_goes_when_no_change_is_droppable() {
    let mut d = merging();
    full(&mut d, 54);
    for task in [52, 53, 54] {
        add_change(&mut d, task, false);
        assert!(!d.droppable(task));
    }
    d.landed.insert(SECOND, 53).unwrap();
    d.landed.insert(THIRD, 52).unwrap();
    let out = merged(&mut d, MERGE);
    assert_eq!(erasures(&out).as_ref(), &[THIRD]);
    assert_eq!(d.landed.get(&FIRST), Some(&54));
}

#[test]
fn merging_changes_own_entries_go_last_but_can_make_room() {
    for own_only in [false, true] {
        let mut d = merging();
        full(&mut d, fixtures::CHANGE);
        if !own_only {
            add_change(&mut d, 52, false);
            add_change(&mut d, 53, false);
            d.landed.insert(SECOND, 53).unwrap();
            d.landed.insert(THIRD, 52).unwrap();
        }
        let out = merged(&mut d, MERGE);
        let victim = if own_only { FIRST } else { THIRD };
        assert_eq!(erasures(&out).as_ref(), &[victim], "own entries only: {own_only}");
        assert_eq!(d.landed.get(&MERGE), Some(&fixtures::CHANGE));
        assert_eq!(d.landed.len(), env().limits.landings.max(env().limits.changes));
    }
}

#[test]
fn duplicate_merge_erases_nothing_and_still_advances_head() {
    let mut d = merging();
    full(&mut d, 100);
    let out = merged(&mut d, SECOND);
    assert!(erasures(&out).is_empty());
    assert_eq!(d.landed.len(), env().limits.landings.max(env().limits.changes));
    assert_eq!(d.landed.get(&SECOND), Some(&fixtures::CHANGE));
    assert_eq!(d.heads.get(&main()).unwrap().commit, SECOND);
    assert!(saved(&out, Stored::BranchHead(BranchHead { name: main(), commit: SECOND, owned: true })));
    assert!(saved(&out, Stored::Landed { commit: SECOND, task: fixtures::CHANGE }));
}

#[test]
fn merge_with_room_keeps_existing_entries() {
    let mut d = merging();
    d.landed.insert(FIRST, 100).unwrap();
    let out = merged(&mut d, MERGE);
    assert!(erasures(&out).is_empty());
    assert_eq!(d.landed.get(&FIRST), Some(&100));
    assert_eq!(d.landed.get(&MERGE), Some(&fixtures::CHANGE));
    assert!(saved(&out, Stored::Landed { commit: MERGE, task: fixtures::CHANGE }));
}
