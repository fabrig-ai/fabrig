//! Temper's own writes (domain/forge/writes.md, sections 1–4, 6–7).
use super::{REPO, adopted, collected, env, fixtures, outputs, refusals};
use crate::writes::{Actor, Operation, Seen, Sender, WRITES, WriteState, Writes, Written};
use crate::{BranchHead, Domain, Event, Key, Name, Request, Stored};
use skein_forge::api::{Answer, Commit, Error, MergeHow, Op, Read, State, UpdateStyle, Write};
use skein_forge_client::{Condition, Entry, Made, Outcome};
use skein_lib::Queue;

const TASK: u64 = 51;
const ENTRY: u64 = 11;
const PULL: u64 = 7;
const HEAD: Commit = fixtures::HEAD;
const T: Commit = fixtures::BASE;
const C: Commit = [5; 32];
const D: Commit = [6; 32];
const M: Commit = [7; 32];
const M_PRIME: Commit = [8; 32];
const UNKNOWN: Commit = [9; 32];
const BRANCH: &[u8] = b"temper/51";
const BASE: &[u8] = b"main";
const SENDER: Sender = Sender::Entry { task: TASK, entry: ENTRY };

fn name(branch: &[u8]) -> Name {
    crate::domain::writes::branch(&env().limits, REPO, branch).expect("test branch")
}

fn repo() -> Domain {
    refusals::adopted_domain()
}

fn loading() -> Domain {
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    d
}

fn head(d: &mut Domain, name: &Name, commit: Commit) {
    outputs(d, Event::Restore { record: Stored::BranchHead(BranchHead { name: name.clone(), commit, owned: true }) });
}

fn entry(write: Write, condition: Condition) -> Entry {
    let mut entry = refusals::entry(ENTRY);
    entry.task = TASK;
    entry.effect = skein_forge_client::Effect { write, condition };
    entry
}

fn change(d: &mut Domain) {
    let mut row = fixtures::opening_row();
    row.pull = Some(PULL);
    outputs(d, Event::Change { row });
}

fn saved(out: &[Request], name: &Name) -> Writes {
    for request in out {
        if let Request::Save { record: Stored::Writes(row), .. } = request
            && row.name == *name
        {
            return row.clone();
        }
    }
    panic!("no write record for {name:?} in {out:?}");
}

fn outcome(d: &mut Domain, result: Outcome) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    crate::domain::scripted_outcome(d, &env(), ENTRY, TASK, result, &mut out);
    collected(out)
}

fn made(made: Made) -> Outcome {
    Outcome::Made { made, found: false }
}

fn kept(d: &Domain, name: &Name) -> Writes {
    d.writes.get(name).expect("record kept").clone()
}

fn no_call(out: &[Request]) {
    for request in out {
        assert!(!matches!(request, Request::Call { .. }), "unexpected forge call: {request:?}");
    }
}

fn has_flight(out: &[Request]) -> bool {
    for request in out {
        if let Request::Save { record: Stored::Flight { task: TASK, entry: ENTRY }, .. } = request {
            return true;
        }
    }
    false
}

#[test]
fn admitted_outbox_intents_are_saved_together_and_confirmed_by_their_answers() {
    let mut d = repo();
    change(&mut d);
    let c = name(BRANCH);
    let g = name(BASE);
    head(&mut d, &c, HEAD);
    head(&mut d, &g, T);
    let merge =
        entry(Write::Merge { number: PULL, head: HEAD, how: MergeHow::Style }, Condition::Merge { base: BASE.into() });
    let out = outputs(&mut d, Event::Enqueue { entry: merge.clone() });
    assert!(out.contains(&Request::Save {
        read_afresh: None,
        release: Some(ENTRY),
        record: Stored::Entry { entry: merge, held: false }
    }));
    no_call(&out);
    assert_eq!(
        saved(&out, &c).entries.as_ref(),
        &[Written {
            order: 1,
            operation: Operation::Merge { pull: PULL, head: HEAD },
            sender: SENDER,
            state: WriteState::Admitted,
        }]
    );
    assert_eq!(
        saved(&out, &g).entries.as_ref(),
        &[Written {
            order: 1,
            operation: Operation::MergedInto { pull: PULL, before: Some(T), after: None },
            sender: SENDER,
            state: WriteState::Admitted,
        }]
    );
    let confirmed = outcome(&mut d, made(Made::Merged(M)));
    assert_eq!(saved(&confirmed, &c).entries[0].state, WriteState::Confirmed);
    let base = saved(&confirmed, &g);
    assert_eq!((base.head, base.entries[0].state), (Some(M), WriteState::Confirmed));
    assert_eq!(base.entries[0].operation, Operation::MergedInto { pull: PULL, before: Some(T), after: Some(M) });

    let writes = [
        (
            Write::OpenPull {
                title: b"title".as_slice().into(),
                body: Box::new([]),
                head: BRANCH.into(),
                base: BASE.into(),
            },
            Condition::None,
            Operation::Open { pull: None, base: BASE.into(), head: HEAD },
            Made::Created(PULL),
            Operation::Open { pull: Some(PULL), base: BASE.into(), head: HEAD },
        ),
        (
            Write::Update { number: PULL, style: UpdateStyle::Merge },
            Condition::Update { head: HEAD, base: T },
            Operation::Update { pull: PULL, before: HEAD, base: T, after: None },
            Made::Updated(C),
            Operation::Update { pull: PULL, before: HEAD, base: T, after: Some(C) },
        ),
        (
            Write::Close { number: PULL },
            Condition::None,
            Operation::Close { pull: PULL },
            Made::Set,
            Operation::Close { pull: PULL },
        ),
        (
            Write::Reopen { number: PULL },
            Condition::None,
            Operation::Reopen { pull: PULL },
            Made::Set,
            Operation::Reopen { pull: PULL },
        ),
        (
            Write::CreateBranch { branch: BRANCH.into(), commit: HEAD },
            Condition::None,
            Operation::Create { after: HEAD },
            Made::Branch(HEAD),
            Operation::Create { after: HEAD },
        ),
        (
            Write::DeleteBranch { branch: BRANCH.into() },
            Condition::None,
            Operation::Delete { before: Some(HEAD) },
            Made::Set,
            Operation::Delete { before: Some(HEAD) },
        ),
    ];
    for (write, condition, admitted, result, confirmed) in writes {
        let mut d = repo();
        change(&mut d);
        if admitted != (Operation::Create { after: HEAD }) {
            head(&mut d, &c, HEAD);
        }
        let out = outputs(&mut d, Event::Enqueue { entry: entry(write, condition) });
        assert_eq!(saved(&out, &c).entries[0].operation, admitted);
        let out = outcome(&mut d, made(result));
        let record = saved(&out, &c);
        assert_eq!(record.entries[0].state, WriteState::Confirmed);
        assert_eq!(record.entries[0].operation, confirmed);
    }
}

#[test]
fn outbox_attempts_uncertain_answers_and_refusals_share_their_saved_decision() {
    let branch = name(BRANCH);
    let creating = || entry(Write::CreateBranch { branch: BRANCH.into(), commit: HEAD }, Condition::None);
    let mut d = repo();
    outputs(&mut d, Event::Enqueue { entry: creating() });
    outputs(&mut d, Event::Committed { entry: ENTRY });
    let (call, _) = fixtures::next_call(&mut d);
    let out = fixtures::answer(
        &mut d,
        call,
        Ok(Answer::Items { items: Box::new([]), more: false, cut: false, now: env().now }),
    );
    // The client's clock and attempt can be saved separately; the attempt
    // save and Flight are always together.
    let sent = if has_flight(&out) { out } else { super::ready(&mut d) };
    assert!(has_flight(&sent));
    assert_eq!(saved(&sent, &branch).entries[0].state, WriteState::Sent);
    for result in [Outcome::Uncertain, Outcome::Raced { made: Made::Branch(HEAD), why: Error::Stale }, Outcome::Held] {
        let mut d = repo();
        outputs(&mut d, Event::Enqueue { entry: creating() });
        assert_eq!(saved(&outcome(&mut d, result), &branch).entries[0].state, WriteState::Uncertain);
    }
    for result in [Outcome::Failed(Error::Forbidden), Outcome::Withdrawn] {
        let mut d = repo();
        outputs(&mut d, Event::Enqueue { entry: creating() });
        let settled = outcome(&mut d, result);
        assert!(saved(&settled, &branch).entries.is_empty());
        assert!(settled.contains(&Request::Erase { key: Key::Entry(ENTRY) }));
    }
    let mut d = repo();
    let post = entry(Write::Post { number: PULL, key: b"test".as_slice().into(), body: Box::new([]) }, Condition::None);
    let posted = outputs(&mut d, Event::Enqueue { entry: post.clone() });
    assert!(posted.contains(&Request::Save {
        read_afresh: None,
        release: Some(ENTRY),
        record: Stored::Entry { entry: post, held: false }
    }));
    for request in &posted {
        assert!(!matches!(request, Request::Save { record: Stored::Writes(_), .. }));
    }
}

fn push(before: Commit, after: Commit) -> Operation {
    Operation::Push { before: Some(before), after }
}

#[test]
fn branch_judgement_distinguishes_current_explained_behind_drift_and_absent() {
    let mut d = repo();
    let branch = name(BRANCH);
    head(&mut d, &branch, T);
    let mut out = Queue::uncapped();
    d.admit(&branch, push(T, C), SENDER, &mut out).unwrap();
    assert_eq!(d.observe(&branch, Some(T), Actor::Unknown, &mut out), Seen::Current);
    assert_eq!(d.observe(&branch, Some(C), Actor::Other, &mut out), Seen::Drift { nearest: Some(1) });
    assert_eq!(d.observe(&branch, Some(C), Actor::Writer, &mut out), Seen::Explained { order: 1 });
    assert_eq!(kept(&d, &branch).entries[0].state, WriteState::Confirmed);
    assert_eq!(kept(&d, &branch).head, Some(C));
    d.admit(&branch, push(C, D), SENDER, &mut out).unwrap();
    assert_eq!(d.observe(&branch, Some(D), Actor::Unknown, &mut out), Seen::Explained { order: 2 });
    assert_eq!(d.observe(&branch, Some(C), Actor::Unknown, &mut out), Seen::Behind { order: 2 });
    assert_eq!(d.observe(&branch, Some(UNKNOWN), Actor::Unknown, &mut out), Seen::Drift { nearest: Some(2) });
    assert_eq!(d.observe(&branch, None, Actor::Unknown, &mut out), Seen::Drift { nearest: Some(2) });
    d.admit(&branch, Operation::Delete { before: Some(D) }, SENDER, &mut out).unwrap();
    assert_eq!(d.observe(&branch, None, Actor::Unknown, &mut out), Seen::Explained { order: 3 });
    assert_eq!(kept(&d, &branch).head, None);
}

fn contains(d: &mut Domain, tip: Commit, commit: Commit, yes: bool) -> Box<[Request]> {
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Commits { before: tip, after: commit, page: 1 }));
    let commits: Box<[_]> = if yes {
        Box::new([])
    } else {
        Box::new([skein_forge::api::CommitMessage { id: commit, parents: Box::new([]), message: Box::new([]) }])
    };
    fixtures::answer(d, call, Ok(Answer::Commits { commits, more: false }))
}

#[test]
fn an_update_intent_checks_both_pinned_commits_before_explaining_a_head() {
    let mut d = repo();
    let branch = name(BRANCH);
    head(&mut d, &branch, T);
    let mut out = Queue::uncapped();
    d.admit(&branch, Operation::Update { pull: PULL, before: T, base: C, after: None }, SENDER, &mut out).unwrap();
    assert_eq!(d.observe(&branch, Some(D), Actor::Unknown, &mut out), Seen::Drift { nearest: Some(1) });
    d.writes.get_mut(&branch).expect("update record").entries[0].state = WriteState::Sent;
    assert_eq!(d.observe(&branch, Some(D), Actor::Unknown, &mut out), Seen::Owed);
    contains(&mut d, D, T, true);
    let out = contains(&mut d, D, C, true);
    let row = saved(&out, &branch);
    assert_eq!(row.head, Some(D));
    assert_eq!(row.entries[0].state, WriteState::Confirmed);
}

#[test]
fn a_late_merge_never_rewinds_a_design_push_and_an_unordered_one_is_compared_once() {
    for (made, holds) in [(M, false), (M_PRIME, false), (M_PRIME, true)] {
        let mut d = repo();
        change(&mut d);
        let base = name(BASE);
        head(&mut d, &base, T);
        outputs(
            &mut d,
            Event::Enqueue {
                entry: entry(
                    Write::Merge { number: PULL, head: HEAD, how: MergeHow::Style },
                    Condition::Merge { base: BASE.into() },
                ),
            },
        );
        let mut row = kept(&d, &base);
        row.head = Some(C);
        row.entries = Box::new([
            row.entries[0].clone(),
            Written {
                order: 2,
                operation: push(M, C),
                sender: Sender::Attempt { task: 10, attempt: 1 },
                state: WriteState::Confirmed,
            },
        ]);
        row.next = 3;
        outputs(&mut d, Event::Restore { record: Stored::Writes(row) });
        let ended = outcome(&mut d, Outcome::Made { made: Made::Merged(made), found: false });
        assert_eq!(saved(&ended, &base).head, Some(C));
        let mut out = Queue::uncapped();
        if made == M {
            assert_eq!(d.observe(&base, Some(M), Actor::Unknown, &mut out), Seen::Behind { order: 2 });
        } else {
            assert_eq!(d.observe(&base, Some(made), Actor::Unknown, &mut out), Seen::Owed);
            assert_eq!(d.observe(&base, Some(made), Actor::Unknown, &mut out), Seen::Owed);
            let (call, op) = fixtures::next_call(&mut d);
            assert_eq!(op, Op::Read(Read::Compare { before: C, after: made }));
            fixtures::answer(
                &mut d,
                call,
                Ok(Answer::Compare {
                    before: C,
                    after: made,
                    ahead: 1,
                    behind: u64::from(!holds),
                    files: Box::new([]),
                    changed: 0,
                }),
            );
            let expected = if holds { Seen::Current } else { Seen::Behind { order: 1 } };
            assert_eq!(d.observe(&base, Some(made), Actor::Unknown, &mut out), expected);
            no_call(&super::ready(&mut d));
        }
    }
}

#[test]
fn a_pull_is_judged_against_its_own_transitions_and_both_branch_heads() {
    let mut d = repo();
    let branch = name(BRANCH);
    let base = name(BASE);
    head(&mut d, &branch, HEAD);
    head(&mut d, &base, T);
    let mut pull = fixtures::open_pull();
    pull.number = PULL;
    let mut out = Queue::uncapped();
    d.admit(&branch, Operation::Open { pull: Some(PULL), head: HEAD, base: BASE.into() }, SENDER, &mut out).unwrap();
    pull.state = State::Closed;
    assert_eq!(d.observe_pull(&branch, PULL, &pull, &mut out), Seen::Drift { nearest: Some(1) });
    d.admit(&branch, Operation::Close { pull: PULL }, SENDER, &mut out).unwrap();
    assert_eq!(d.observe_pull(&branch, PULL, &pull, &mut out), Seen::Explained { order: 2 });
    pull.state = State::Open;
    d.admit(&branch, Operation::Reopen { pull: PULL }, SENDER, &mut out).unwrap();
    assert_eq!(d.observe_pull(&branch, PULL, &pull, &mut out), Seen::Explained { order: 3 });
    pull.base = b"another".as_slice().into();
    assert_eq!(d.observe_pull(&branch, PULL, &pull, &mut out), Seen::Drift { nearest: Some(3) });
    pull.base = BASE.into();
    d.admit(&branch, push(HEAD, C), SENDER, &mut out).unwrap();
    assert_eq!(d.observe(&branch, Some(C), Actor::Writer, &mut out), Seen::Explained { order: 4 });
    assert_eq!(d.observe_pull(&branch, PULL, &pull, &mut out), Seen::Behind { order: 4 });
    pull.commit = C;
    d.admit(&base, push(T, D), SENDER, &mut out).unwrap();
    d.observe(&base, Some(D), Actor::Writer, &mut out);
    assert_eq!(d.observe_pull(&branch, PULL, &pull, &mut out), Seen::Behind { order: 1 });
}

#[test]
fn worker_attempts_settle_on_a_later_lease_or_on_fresh_containment() {
    let branch = name(BRANCH);
    let worker = Sender::Attempt { task: TASK, attempt: 1 };
    let mut d = repo();
    head(&mut d, &branch, T);
    let mut out = Queue::uncapped();
    d.admit(&branch, push(T, C), worker, &mut out).unwrap();
    d.admit(&branch, push(C, D), worker, &mut out).unwrap();
    assert_eq!(kept(&d, &branch).entries[0].state, WriteState::Confirmed);
    assert_eq!(kept(&d, &branch).head, Some(C));
    for (lost, holds) in [(false, true), (false, false), (true, true), (true, false)] {
        let mut d = repo();
        head(&mut d, &branch, T);
        let mut out = Queue::uncapped();
        d.admit(&branch, push(T, C), worker, &mut out).unwrap();
        d.finished(TASK, 1, lost, &mut out);
        let state = if lost { WriteState::Uncertain } else { WriteState::Admitted };
        assert_eq!(kept(&d, &branch).entries[0].state, state);
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Branch { branch: BRANCH.into() }));
        fixtures::answer(&mut d, call, Ok(Answer::Commit(D)));
        let out = contains(&mut d, D, C, holds);
        let row = saved(&out, &branch);
        if holds {
            assert_eq!(row.entries[0].state, WriteState::Confirmed);
            assert_eq!(row.head, Some(C));
        } else {
            assert!(row.entries.is_empty());
            assert_eq!(row.head, Some(T));
        }
    }
}

fn fill(d: &mut Domain, branch: &Name, from: u64, before: Commit, out: &mut Queue<Request>) {
    for attempt in from..=u64::try_from(WRITES).unwrap() {
        let after = [u8::try_from(100_u64.checked_add(attempt).unwrap()).unwrap(); 32];
        let sender = Sender::Attempt { task: TASK.checked_add(attempt).unwrap(), attempt };
        assert_eq!(d.admit(branch, push(before, after), sender, out), Ok(attempt));
    }
}

#[test]
fn a_full_branch_names_its_senders_and_only_evicts_an_old_confirmed_write() {
    let mut d = repo();
    let branch = name(BRANCH);
    head(&mut d, &branch, T);
    let mut out = Queue::uncapped();
    let count = u64::try_from(WRITES).unwrap();
    fill(&mut d, &branch, 1, T, &mut out);
    let full = d.admit(&branch, Operation::Delete { before: Some(T) }, SENDER, &mut out).unwrap_err();
    assert_eq!(full.name, branch);
    assert_eq!(full.senders.len(), WRITES);
    for (at, sender) in full.senders.iter().enumerate() {
        let attempt = u64::try_from(at.checked_add(1).unwrap()).unwrap();
        assert_eq!(*sender, Sender::Attempt { task: TASK.checked_add(attempt).unwrap(), attempt });
    }

    let mut d = repo();
    head(&mut d, &branch, T);
    for (before, after) in [(T, C), (C, D)] {
        d.admit(&branch, push(before, after), Sender::Attempt { task: TASK, attempt: 1 }, &mut out).unwrap();
        assert!(matches!(d.observe(&branch, Some(after), Actor::Writer, &mut out), Seen::Explained { .. }));
    }
    fill(&mut d, &branch, 3, D, &mut out);
    assert_eq!(
        d.admit(&branch, Operation::Delete { before: Some(D) }, SENDER, &mut out),
        Ok(count.checked_add(1).unwrap())
    );
    let row = kept(&d, &branch);
    assert_eq!(row.entries.len(), WRITES);
    assert_eq!(row.entries[0].order, 2);
    assert_eq!(row.entries[0].state, WriteState::Confirmed);
    assert_eq!(row.head, Some(D));
    for entry in &row.entries[1..] {
        assert_eq!(entry.state, WriteState::Admitted);
    }
}

#[test]
fn the_outbox_line_waits_for_a_branch_record_then_asks_its_write_again() {
    use jig_domain_core_vocabulary::waits::{WaitKind, WaitObject};
    use temper_client_protocol_wire::waits::{THING_WRITE_ENTRY, THING_WRITE_RECORD};
    let mut d = refusals::adopted_domain();
    let branch = name(BRANCH);
    head(&mut d, &branch, HEAD);
    let mut out = Queue::uncapped();
    fill(&mut d, &branch, 1, HEAD, &mut out);
    outputs(&mut d, Event::Change { row: super::outbox_room::row(TASK) });
    let waited = super::outbox_room::open(&mut d, TASK, ENTRY);
    assert!(waited.contains(&Request::ChangeDecision {
        task: TASK,
        decision: temper_engine_domain_forge_change::Decision::Wait { until: None },
        entry: None,
        evidence: None,
        close_at: None,
        close_refusal: None,
    }));
    assert!(d.entries.is_empty());
    let report = crate::room::waited(&d, &env().limits, TASK, 1).expect("a branch's room is reported");
    assert_eq!(report.kind, WaitKind::Room);
    let WaitObject::Procedure { things, .. } = report.object else { panic!("procedure wait") };
    assert_eq!(things.len(), 4);
    let mut things = things.as_slice().iter();
    let record = things.next().unwrap();
    assert_eq!((record.what, record.number, record.owner), (THING_WRITE_RECORD, u64::try_from(WRITES).unwrap(), None));
    for (order, thing) in (1_u64..=3).zip(things) {
        assert_eq!(
            (thing.what, thing.number, thing.owner),
            (THING_WRITE_ENTRY, order, Some(TASK.checked_add(order).unwrap()))
        );
    }
    let words = crate::domain::writes::room_account(&d, &env().limits, &branch);
    assert!(words.starts_with(b"waiting for room in the write record of temper/51 (32 entries), held by task 52 attempt 1, task 53 attempt 2,"));
    assert!(words.ends_with(b"task 72 attempt 21, and 11 more"));
    assert!(words.len() <= usize::try_from(crate::EFFECT_ACCOUNT_BYTES).unwrap());
    // A worker answered without landing. The fresh read settles and frees
    // its entry even though the outbox already had room all along.
    d.finished(TASK + 1, 1, false, &mut out);
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: BRANCH.into() }));
    fixtures::answer(&mut d, call, Ok(Answer::Commit(HEAD)));
    let settled = contains(&mut d, HEAD, [101; 32], false);
    let next = super::ready(&mut d);
    let wake = Request::WakeProcedure { task: TASK };
    assert!(settled.contains(&wake) || next.contains(&wake));
    let sent = super::outbox_room::open(&mut d, TASK, ENTRY + 1);
    assert_eq!(
        saved(&sent, &branch).entries.last().unwrap().order,
        u64::try_from(WRITES).unwrap().checked_add(1).unwrap()
    );
}

#[test]
fn restored_uncertain_records_wait_for_fresh_reads_and_outlive_their_heads() {
    let mut d = loading();
    let branch = name(BRANCH);
    head(&mut d, &branch, T);
    let row = Writes {
        name: branch.clone(),
        head: Some(T),
        next: 2,
        entries: Box::new([Written {
            order: 1,
            operation: push(T, C),
            sender: Sender::Attempt { task: TASK, attempt: 1 },
            state: WriteState::Uncertain,
        }]),
    };
    outputs(&mut d, Event::Restore { record: Stored::Writes(row.clone()) });
    assert_eq!(kept(&d, &branch), row);
    let mut out = Queue::uncapped();
    crate::domain::heads::free(&mut d, &env(), &mut out);
    assert!(collected(out).contains(&Request::Erase { key: Key::BranchHead(branch.clone()) }));
    assert_eq!(kept(&d, &branch), row);
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    no_call(&super::ready(&mut d));
    outputs(&mut d, Event::CatchUp);
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: BRANCH.into() }));
    let seen = fixtures::answer(&mut d, call, Ok(Answer::Commit(C)));
    assert_eq!(saved(&seen, &branch).entries[0].state, WriteState::Confirmed);
    assert!(seen.contains(&Request::Erase { key: Key::Writes(branch) }));
}

#[test]
fn an_uncertain_queue_merge_owes_both_reads_even_after_restart() {
    let mut d = repo();
    change(&mut d);
    let branch = name(BRANCH);
    let base = name(BASE);
    head(&mut d, &branch, HEAD);
    head(&mut d, &base, T);
    let merge =
        entry(Write::Merge { number: PULL, head: HEAD, how: MergeHow::Style }, Condition::Merge { base: BASE.into() });
    outputs(&mut d, Event::Enqueue { entry: merge });
    let lost = outcome(&mut d, Outcome::Uncertain);
    let head_record = saved(&lost, &branch);
    let base_record = saved(&lost, &base);
    assert_eq!(head_record.entries[0].operation, Operation::Merge { pull: PULL, head: HEAD });
    assert_eq!(base_record.entries[0].operation, Operation::MergedInto { pull: PULL, before: Some(T), after: None });
    for row in [&head_record, &base_record] {
        assert_eq!(row.entries[0].state, WriteState::Uncertain);
        assert!(d.inspection_write_readback(&row.name, row.entries[0].order), "no read owed for {:?}", row.name);
    }

    let mut restarted = loading();
    for row in [head_record, base_record] {
        head(&mut restarted, &row.name, row.head.unwrap());
        outputs(&mut restarted, Event::Restore { record: Stored::Writes(row) });
    }
    outputs(&mut restarted, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut restarted, Event::CatchUp);
    for name in [&branch, &base] {
        let row = kept(&restarted, name);
        assert_eq!(row.entries[0].state, WriteState::Uncertain);
        assert!(restarted.inspection_write_readback(name, row.entries[0].order));
    }

    let mut merged = fixtures::open_pull();
    merged.number = PULL;
    merged.state = State::Closed;
    merged.merged = Some(M);
    for _ in 0_u8..2 {
        let (call, op) = fixtures::next_call(&mut restarted);
        assert_eq!(op, Op::Read(Read::Pull { number: PULL, ci: false }));
        fixtures::answer(&mut restarted, call, Ok(Answer::Pull(merged.clone())));
    }
    assert_eq!(kept(&restarted, &branch).entries[0].state, WriteState::Confirmed);
    assert_eq!(kept(&restarted, &base).entries[0].state, WriteState::Confirmed);
    assert_eq!(kept(&restarted, &base).head, Some(M));
}

#[test]
fn recording_an_earlier_members_merge_never_writes_the_rounds_final_base() {
    let mut d = repo();
    change(&mut d);
    let branch = name(BRANCH);
    let base = name(BASE);
    head(&mut d, &base, M);
    head(&mut d, &branch, HEAD);
    let mut out = Queue::uncapped();
    d.admit(&base, push(M, C), Sender::Attempt { task: TASK, attempt: 2 }, &mut out).unwrap();
    assert_eq!(d.observe(&base, Some(C), Actor::Writer, &mut out), Seen::Explained { order: 1 });
    let saved_base = kept(&d, &base);

    let admitted = outputs(
        &mut d,
        Event::Enqueue {
            entry: entry(
                Write::Merge { number: PULL, head: HEAD, how: MergeHow::Recorded { commit: M } },
                Condition::Merge { base: BASE.into() },
            ),
        },
    );
    assert_eq!(saved(&admitted, &branch).entries[0].operation, Operation::Merge { pull: PULL, head: HEAD });
    assert_eq!(kept(&d, &base), saved_base, "the recording adds no base entry");
    let confirmed = outcome(&mut d, made(Made::Merged(M)));
    assert_eq!(saved(&confirmed, &branch).entries[0].state, WriteState::Confirmed);
    assert_eq!(kept(&d, &base), saved_base, "the base keeps the last member's tip");
}
