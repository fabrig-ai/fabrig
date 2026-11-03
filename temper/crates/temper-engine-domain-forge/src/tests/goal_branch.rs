use super::*;
use skein_forge::api::{Answer, BranchCreation, Error, Op, Read, Write};

const GOAL: u64 = 7;
const OTHER: u64 = 8;
const TAKING: u64 = 21;
const CLOSING: u64 = 22;
const GOAL_BRANCH: &[u8] = b"temper/7/g7";
const TIP: skein_forge::api::Commit = [4; 32];

fn name(branch: &[u8]) -> Name {
    let mut parts = skein_lib::List::with_capacity(LIMITS.authority_segments);
    let mut start = 0;
    for index in 0..=branch.len() {
        if index == branch.len() || branch.get(index) == Some(&b'/') {
            parts.push(Box::from(branch.get(start..index).unwrap())).unwrap();
            start = index.checked_add(1).unwrap();
        }
    }
    Name { forge: REPO.forge, repository: REPO.repository, what: What::Branch(parts.into_boxed()) }
}

fn entry() -> skein_forge_client::Entry {
    skein_forge_client::Entry {
        number: TAKING,
        task: GOAL,
        repository: REPO,
        effect: skein_forge_client::Effect {
            write: Write::CreateBranch { branch: GOAL_BRANCH.into(), commit: TIP },
            condition: skein_forge_client::Condition::None,
        },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    }
}
fn assert_kept_making(requests: &[Request]) {
    assert!(
        requests.contains(&Request::Save {
            release: Some(TAKING),
            read_afresh: None,
            record: Stored::Entry { entry: entry(), held: false }
        }),
        "{requests:?}"
    );
}
fn assert_no_making(requests: &[Request]) {
    for request in requests {
        if let Request::Save { record: Stored::Entry { .. }, .. } | Request::MakeFailed { .. } = request {
            panic!("no making expected: {request:?}");
        }
    }
}
fn made() -> Request {
    Request::Outcome {
        entry: TAKING,
        task: GOAL,
        account: None,
        outcome: skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Branch(TIP), found: false },
    }
}

fn fresh() -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    restore_done(&mut d);
    d
}
fn restore_done(d: &mut Domain) {
    outputs(d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(d, Event::ReadAfresh);
    outputs(d, Event::SettleOutbox);
}
fn take(d: &mut Domain) {
    outputs(d, Event::Hold { entry: TAKING, task: GOAL, resource: name(GOAL_BRANCH), from: None });
}
fn missing(d: &mut Domain) -> Box<[Request]> {
    take(d);
    fixtures::answer_next(d, Err(Error::Missing));
    fixtures::answer_next(d, Ok(Answer::Commit(TIP)))
}
fn making(d: &mut Domain) -> (Token, Op) {
    missing(d);
    outputs(d, Event::Committed { entry: TAKING });
    fixtures::answer_next(d, Ok(Answer::Items { items: Box::new([]), more: false, now: env().now }));
    fixtures::next_call(d)
}

#[test]
fn goal_branch_names_are_canonical_and_distinct_from_other_own_branches() {
    for (branch, expected) in [
        (b"temper/7/g7".as_slice(), Some((7, 7))),
        (b"temper/12/g34", Some((12, 34))),
        (b"temper/0/g0", Some((0, 0))),
        (b"temper/7/c7", None),
        (b"temper/7/r7", None),
        (b"temper/7/s7", None),
        (b"temper/7/l7", None),
        (b"temper/07/g7", None),
        (b"temper/7/g07", None),
        (b"temper/7/g", None),
        (b"temper//g7", None),
        (b"temper/7/g7/extra", None),
        (b"temper/7/g7x", None),
        (b"temper/7/g-7", None),
        (b"temper/+7/g7", None),
        (b"other/7/g7", None),
        (b"temper/18446744073709551616/g7", None),
        (b"temper/7/g18446744073709551616", None),
    ] {
        assert_eq!(items::goal_branch(&name(branch), b"temper/"), expected, "{branch:?}");
    }
    assert_eq!(items::goal_branch(&name(b"team/temper/12/g34"), b"team/temper/"), Some((12, 34)));
    assert_eq!(items::goal_branch(&Name { forge: 1, repository: 2, what: What::Repository }, b"temper/"), None);
}

#[test]
fn a_goal_branch_hold_is_exclusive_and_refusing() {
    let mut d = fresh();
    take(&mut d);
    assert_eq!(d.resource_facts(&name(GOAL_BRANCH)), Some((Access::Owned, resources::GOAL_BRANCH_HOLD)));
    for branch in [b"temper/7/c7".as_slice(), b"temper/7/r7", b"temper/7/s7", b"temper/7/l7"] {
        assert_eq!(d.resource_facts(&name(branch)), Some((Access::Owned, resources::BRANCH_HOLD)));
    }
    assert_eq!(
        outputs(&mut d, Event::Hold { entry: CLOSING, task: OTHER, resource: name(GOAL_BRANCH), from: None }).as_ref(),
        &[Request::Taken { task: OTHER, resource: name(GOAL_BRANCH), by: GOAL }]
    );
}

#[test]
fn taking_a_missing_goal_branch_makes_it_at_the_default_branchs_tip() {
    let mut d = fresh();
    take(&mut d);
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
    fixtures::answer(&mut d, call, Err(Error::Missing));
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: b"main".as_slice().into() }));
    let saved = fixtures::answer(&mut d, call, Ok(Answer::Commit(TIP)));
    assert_kept_making(&saved);
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, Some(Writer::Entry(TAKING)));
    for request in ready(&mut d) {
        if let Request::Call { op: Op::Write(_), .. } = request {
            panic!("write before commit: {request:?}");
        }
    }
    outputs(&mut d, Event::Committed { entry: TAKING });
    fixtures::answer_next(&mut d, Ok(Answer::Items { items: Box::new([]), more: false, now: env().now }));
    let (_, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Write(Write::CreateBranch { branch: GOAL_BRANCH.into(), commit: TIP }));
}

#[test]
fn taking_an_existing_goal_branch_makes_nothing() {
    let mut d = fresh();
    take(&mut d);
    let answered = fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
    assert_no_making(&answered);
    assert_eq!(d.branch_head(&name(GOAL_BRANCH)), Some(TIP));
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, None);
    assert!(!d.is_ready());
}

#[test]
fn a_goal_branch_making_that_fails_for_good_says_why() {
    let mut d = fresh();
    let (call, _) = making(&mut d);
    let failed = fixtures::answer(&mut d, call, Err(Error::Forbidden));
    assert!(
        failed.contains(&Request::MakeFailed {
            task: GOAL,
            resource: name(GOAL_BRANCH),
            reason: b"forge refused permission".as_slice().into()
        }),
        "{failed:?}"
    );
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, None);
}

#[test]
fn a_change_into_a_goal_branch_being_made_reads_it_once_made() {
    use temper_engine_domain_forge_change as change;
    let mut d = fresh();
    let (making_call, _) = making(&mut d);
    let mut row = fixtures::opening_row();
    row.base = GOAL_BRANCH.into();
    row.pending = None;
    row.effect = change::EffectResult::None;
    outputs(&mut d, Event::Change { row });
    fixtures::step_change(&mut d);
    let waiting = fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
    assert!(waiting.contains(&Request::ChangeDecision {
        task: fixtures::CHANGE,
        decision: change::Decision::Wait { until: None },
        entry: None,
        evidence: None
    }));
    let made = fixtures::answer(&mut d, making_call, Ok(Answer::Branch(BranchCreation::Created)));
    assert!(made.contains(&self::made()), "{made:?}");
    let wake = ready(&mut d);
    assert!(wake.contains(&Request::WakeProcedure { task: fixtures::CHANGE }), "{wake:?}");
    fixtures::step_change(&mut d);
    fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
    let (_, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
}

#[test]
fn a_closing_goal_keeps_its_branch_whatever_way_it_ended() {
    for ending in [ReleaseEnding::Done, ReleaseEnding::Failed, ReleaseEnding::Cancelled] {
        let mut d = fresh();
        take(&mut d);
        fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
        let released =
            outputs(&mut d, Event::Release { task: GOAL, root: OTHER, ending, reason: None, entry: CLOSING });
        assert!(released.contains(&Request::Released { task: GOAL }), "{released:?}");
        assert_no_making(&released);
        for request in &released {
            if let Request::Retained { .. } = request {
                panic!("a goal keeps its branch without retaining a hold: {request:?}");
            }
        }
        assert_eq!(d.hold(&name(GOAL_BRANCH)), None);
        assert_eq!(d.branch_head(&name(GOAL_BRANCH)), Some(TIP));
    }
}

#[test]
fn a_goal_branch_found_missing_after_restart_reports_the_missing_entry_allocator() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Hold(Hold { name: name(GOAL_BRANCH), task: GOAL, writer: None, drift: None }),
        },
    );
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    let missing = fixtures::answer_next(&mut d, Err(Error::Missing));
    assert!(
        missing.contains(&Request::MakeFailed {
            task: GOAL,
            resource: name(GOAL_BRANCH),
            reason: b"branch missing after restart; forge has no restored outbox entry allocator".as_slice().into()
        }),
        "{missing:?}"
    );
    assert!(missing.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }));
    assert!(d.hold(&name(GOAL_BRANCH)).is_some());
}

#[test]
fn a_goal_branch_making_uses_the_known_default_tip() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    // This head is already part of the connector's working set before the taking.
    outputs(
        &mut d,
        Event::Restore { record: Stored::BranchHead(BranchHead { name: name(b"main"), commit: TIP, owned: false }) },
    );
    restore_done(&mut d);
    take(&mut d);
    let made = fixtures::answer_next(&mut d, Err(Error::Missing));
    assert_kept_making(&made);
    assert!(!d.is_ready(), "the effect still waits for its durable commit");
}

#[test]
fn a_goal_branch_read_failure_names_the_resource_and_forge_answer() {
    for read_default in [false, true] {
        let mut d = fresh();
        take(&mut d);
        if read_default {
            fixtures::answer_next(&mut d, Err(Error::Missing));
        }
        let failed = fixtures::answer_next(&mut d, Err(Error::Forbidden));
        assert!(
            failed.contains(&Request::MakeFailed {
                task: GOAL,
                resource: name(GOAL_BRANCH),
                reason: b"forge refused permission".as_slice().into()
            }),
            "{failed:?}"
        );
        assert!(d.hold(&name(GOAL_BRANCH)).is_some());
    }
}

#[test]
fn a_goal_branch_taking_blocks_a_writer_during_its_reads() {
    let mut d = fresh();
    take(&mut d);
    assert_eq!(
        outputs(
            &mut d,
            Event::Claim { task: OTHER, attempt: 1, writes: Box::new([name(GOAL_BRANCH)]), holders: Box::new([GOAL]) }
        )
        .as_ref(),
        &[Request::Taken { task: OTHER, resource: name(GOAL_BRANCH), by: GOAL }]
    );
    fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
    outputs(
        &mut d,
        Event::Claim { task: OTHER, attempt: 1, writes: Box::new([name(GOAL_BRANCH)]), holders: Box::new([GOAL]) },
    );
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, Some(Writer::Run { task: OTHER, attempt: 1 }));
}

#[test]
fn a_goal_branch_taking_waits_for_its_forge_to_connect() {
    let mut d = fresh();
    take(&mut d);
    fixtures::answer_next(&mut d, Err(Error::NotConnected));
    assert!(!d.is_ready());
    outputs(
        &mut d,
        Event::Connection {
            forge: REPO.forge,
            state: Connection::Connected { id: GOAL, login: b"bot".as_slice().into() },
        },
    );
    ready(&mut d);
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
    let found = fixtures::answer(&mut d, call, Ok(Answer::Commit(TIP)));
    assert_no_making(&found);
}

#[test]
fn a_goal_branch_making_restored_before_sending_keeps_its_taking_entry() {
    let mut before = fresh();
    let saved = missing(&mut before);
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    for request in saved {
        if let Request::Save { record, .. } = request {
            outputs(&mut d, Event::Restore { record });
        }
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    let missing = fixtures::answer_next(&mut d, Err(Error::Missing));
    assert!(missing.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }));
    assert_no_making(&missing);
    outputs(&mut d, Event::SettleOutbox);
    fixtures::answer_next(&mut d, Ok(Answer::Items { items: Box::new([]), more: false, now: env().now }));
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Write(Write::CreateBranch { branch: GOAL_BRANCH.into(), commit: TIP }));
    let made = fixtures::answer(&mut d, call, Ok(Answer::Branch(BranchCreation::Created)));
    assert!(made.contains(&self::made()), "{made:?}");
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, None);
}

const MERGING: u64 = 23;
const MERGED: skein_forge::api::Commit = [9; 32];
const MERGE_CALLS: u32 = 8; // Inbox, pull and branch reads, merge, then its proof.

fn merge_into_goal(d: &mut Domain) -> Box<[Request]> {
    let mut row = fixtures::opening_row();
    row.base = GOAL_BRANCH.into();
    row.pull = Some(1);
    row.pending = Some(MERGING);
    row.change.state = temper_engine_domain_forge_change::State::Landing { head: fixtures::HEAD };
    d.changes.insert(row.task, row).expect("fixture change");
    let mut entry = entry();
    entry.number = MERGING;
    entry.task = fixtures::CHANGE;
    entry.effect = skein_forge_client::Effect {
        write: Write::Merge { number: 1, head: fixtures::HEAD, how: skein_forge::api::MergeHow::Style },
        condition: skein_forge_client::Condition::Merge { base: GOAL_BRANCH.into() },
    };
    outputs(d, Event::Enqueue { entry });
    outputs(d, Event::Committed { entry: MERGING });
    let mut merged = false;
    for _ in 0..MERGE_CALLS {
        let (call, op) = fixtures::next_call(d);
        let answer = match op {
            Op::Read(Read::Items { .. }) => Answer::Items { items: Box::new([]), more: false, now: env().now },
            Op::Read(Read::Branch { branch }) if branch.as_ref() == GOAL_BRANCH => Answer::Commit(TIP),
            Op::Read(Read::Branch { branch }) if branch.as_ref() == b"temper/51" => Answer::Commit(fixtures::HEAD),
            Op::Read(Read::Pull { number: 1 }) => {
                let mut pull = fixtures::open_pull();
                pull.base = GOAL_BRANCH.into();
                if merged {
                    pull.state = skein_forge::api::State::Closed;
                    pull.merged = Some(MERGED);
                }
                Answer::Pull(pull)
            }
            Op::Write(Write::Merge { head, .. }) => {
                assert_eq!(head, fixtures::HEAD);
                merged = true;
                Answer::Merged(MERGED)
            }
            other @ (Op::Read(_) | Op::Write(_)) => panic!("unexpected goal merge call: {other:?}"),
        };
        let requests = fixtures::answer(d, call, Ok(answer));
        if requests.contains(&Request::Outcome {
            entry: MERGING,
            task: fixtures::CHANGE,
            account: None,
            outcome: skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Merged(MERGED), found: false },
        }) {
            assert!(merged, "outcome follows the merge write");
            return requests;
        }
    }
    panic!("goal merge did not settle")
}

#[test]
fn a_merge_into_a_held_goal_branch_saves_only_an_owned_heads_advance() {
    for owned in [Some(true), Some(false), None] {
        let mut d = fresh();
        take(&mut d);
        fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
        if let Some(owned) = owned {
            d.heads.get_mut(&name(GOAL_BRANCH)).expect("goal head").owned = owned;
        } else {
            d.heads.remove(&name(GOAL_BRANCH));
        }
        let requests = merge_into_goal(&mut d);
        let advanced = BranchHead { name: name(GOAL_BRANCH), commit: MERGED, owned: true };
        assert_eq!(
            requests.contains(&Request::Save {
                read_afresh: None,
                release: None,
                record: Stored::BranchHead(advanced.clone())
            }),
            owned == Some(true),
            "owned={owned:?}: {requests:?}"
        );
        match owned {
            Some(owned) => {
                let expected = BranchHead { name: name(GOAL_BRANCH), commit: if owned { MERGED } else { TIP }, owned };
                assert_eq!(d.heads.get(&name(GOAL_BRANCH)), Some(&expected), "owned={owned}");
            }
            None => assert_eq!(d.heads.get(&name(GOAL_BRANCH)), None),
        }
        for request in &requests {
            if let Request::Save { record: Stored::BranchHead(head), .. } = request {
                assert_eq!(owned, Some(true), "an unowned or absent head is read afresh");
                assert_eq!(head, &advanced);
            }
        }
    }
}

#[test]
fn a_merge_into_a_watched_goal_branch_saves_the_clients_advanced_tip() {
    let mut d = fresh();
    take(&mut d);
    fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
    outputs(&mut d, Event::Names { task: GOAL, resources: Box::new([name(GOAL_BRANCH)]) });
    fixtures::answer_next(&mut d, Ok(Answer::Items { items: Box::new([]), more: false, now: env().now }));
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
    let watched = fixtures::answer(&mut d, call, Ok(Answer::Commit(TIP)));
    assert_saved_goal_tip(&watched, TIP);
    let requests = merge_into_goal(&mut d);
    assert_saved_goal_tip(&requests, MERGED);
}

fn assert_saved_goal_tip(requests: &[Request], tip: skein_forge::api::Commit) {
    let resource =
        skein_forge_client::Resource { repository: REPO, what: skein_forge_client::What::Branch(GOAL_BRANCH.into()) };
    let mut saved = false;
    for request in requests {
        if let Request::Save { record: Stored::Client(skein_forge_client::Stored::Live(row)), .. } = request
            && row.watch.resource == resource
        {
            assert_eq!(row.cached.tip, Some(tip));
            saved = true;
        }
    }
    assert!(saved, "the client's advanced tip is durable: {requests:?}");
}
