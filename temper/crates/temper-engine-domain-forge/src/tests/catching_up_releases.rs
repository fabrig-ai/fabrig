//! Releases and run preparation read before judging restored facts (forge/catching-up.md, 4).
use super::*;
use crate::domain::{fresh_reads_in, from_client};
use skein_forge::api::{Answer, Commit, Error, Op, Read, State, Write};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_issues as issues;

const GOAL: u64 = 7;
const G: &[u8] = b"temper/7/g7";
const C: &[u8] = b"temper/51";
const ENTRY: u64 = 90;
const PUSH: Commit = [9; 32];

fn name(branch: &[u8]) -> Name {
    from_client(&resource(branch), &LIMITS).unwrap()
}
fn resource(branch: &[u8]) -> skein_forge_client::Resource {
    skein_forge_client::Resource { repository: REPO, what: skein_forge_client::What::Branch(branch.into()) }
}
fn hold(branch: &[u8], task: u64) -> Hold {
    Hold { name: name(branch), task, writer: None, design_writers: Box::new([]), beside: None, drift: None }
}
fn release_row(task: u64) -> ReleaseRow {
    ReleaseRow {
        task,
        root: GOAL,
        ending: ReleaseEnding::Cancelled,
        close_pull: None,
        comment: None,
        pending: None,
        failed: false,
        effects_settled: true,
        releasing: true,
        room: None,
    }
}
fn restored(branch: &[u8], records: Box<[Stored]>) -> Domain {
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::BranchHead(BranchHead { name: name(branch), commit: fixtures::HEAD, owned: true }),
        },
    );
    outputs(&mut d, Event::Restore { record: Stored::Names { task: GOAL, resources: Box::new([name(branch)]) } });
    for record in records {
        outputs(&mut d, Event::Restore { record });
    }
    d
}
fn begin(d: &mut Domain) {
    outputs(d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(d, Event::CatchUp);
    assert!(!fresh_reads_in(d), "the restart's reads are held unanswered");
}
fn answer_read(d: &mut Domain, tip: Option<Commit>) -> Box<[Request]> {
    let (call, op) = fixtures::next_call(d);
    let result = match op {
        Op::Read(Read::Branch { branch }) if branch.as_ref() == b"main" => Ok(Answer::Commit(fixtures::BASE)),
        Op::Read(Read::Branch { .. }) => match tip {
            Some(head) => Ok(Answer::Commit(head)),
            None => Err(Error::Missing),
        },
        Op::Read(Read::Pull { .. }) => Ok(Answer::Pull(fixtures::open_pull())),
        Op::Read(Read::Item { number, .. }) => Ok(Answer::Item {
            item: skein_forge::api::Summary {
                labels: Box::new([]),
                number,
                kind: skein_forge::api::Kind::Pull,
                state: State::Open,
                author: 7,
                key: None,
                title: b"Work".as_slice().into(),
                body: Box::new([]),
                updated: env().now,
            },
            comments: Box::new([]),
            more: false,
        }),
        Op::Read(Read::Items { .. }) => {
            Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now })
        }
        Op::Read(Read::Commits { .. }) => Ok(Answer::Commits { commits: Box::new([]), more: false }),
        Op::Read(_) => Err(Error::Missing),
        Op::Write(_) => panic!("no write before the fresh reads: {op:?}"),
    };
    fixtures::answer(d, call, result)
}
fn finish_reads(d: &mut Domain, tip: Option<Commit>) -> Box<[Request]> {
    let mut all = skein_lib::List::uncapped();
    for _ in 0_u32..32 {
        if fresh_reads_in(d) {
            return all.into_boxed();
        }
        for request in answer_read(d, tip) {
            all.push(request).unwrap();
        }
    }
    panic!("restart reads did not finish")
}
fn no_release_decision(out: &[Request]) {
    for request in out {
        assert!(
            !matches!(
                request,
                Request::ReleaseFailed { .. }
                    | Request::Released { .. }
                    | Request::EffectsSettled { .. }
                    | Request::Erase { .. }
                    | Request::Save { record: Stored::Entry { .. }, .. }
            ),
            "{out:?}"
        );
    }
}

#[test]
fn release_waits_for_the_fresh_goal_head_before_deleting() {
    let mut change = fixtures::opening_row();
    change.branch = G.into();
    change.pending = None;
    change.change.state = change::State::Landed { merge: fixtures::BASE, since: env().wall };
    change.result.head = Some(fixtures::HEAD);
    let mut d = restored(
        G,
        Box::new([
            Stored::Hold(hold(G, GOAL)),
            Stored::Change(Box::new(change)),
            Stored::Release(ReleaseRow { room: Some(1), ..release_row(GOAL) }),
        ]),
    );
    begin(&mut d);
    let before = d.releases.get(&GOAL).unwrap().clone();
    no_release_decision(&outputs(&mut d, Event::ContinueRelease { task: GOAL, entry: ENTRY }));
    assert_eq!(d.releases.get(&GOAL), Some(&before));
    assert!(
        d.effect_room(&Limits { entries: 1, ..LIMITS }),
        "waiting on facts returns its room promise without losing its place"
    );
    assert!(d.hold(&name(G)).is_some());
    let done = finish_reads(&mut d, Some(fixtures::BASE));
    assert!(done.contains(&Request::ContinueRelease { task: GOAL }), "{done:?}");
    let released = outputs(&mut d, Event::ContinueRelease { task: GOAL, entry: ENTRY });
    assert!(released.contains(&Request::Released { task: GOAL }), "{released:?}");
    assert!(d.entries.is_empty(), "the freshly unlanded goal branch is not deleted");
}

#[test]
fn release_waits_for_the_fresh_open_pull_instead_of_skipping_close() {
    let pull_name = Name { forge: REPO.forge, repository: REPO.repository, what: What::Pull(1) };
    let mut change = fixtures::opening_row();
    change.pending = None;
    change.pull = Some(1);
    change.change.state = change::State::Checking { head: fixtures::HEAD };
    let row = ReleaseRow { close_pull: Some(1), ..release_row(fixtures::CHANGE) };
    let mut d = restored(
        C,
        Box::new([
            Stored::Hold(hold(C, fixtures::CHANGE)),
            Stored::Change(Box::new(change)),
            Stored::Release(row),
            Stored::PullState(PullState {
                name: pull_name.clone(),
                head: fixtures::HEAD,
                ci: skein_forge::api::Ci::Passed,
                state: State::Closed,
            }),
            Stored::Names { task: fixtures::CHANGE, resources: Box::new([name(C), pull_name]) },
        ]),
    );
    begin(&mut d);
    no_release_decision(&outputs(&mut d, Event::ContinueRelease { task: fixtures::CHANGE, entry: ENTRY }));
    assert_eq!(d.releases.get(&fixtures::CHANGE).unwrap().close_pull, Some(1));
    let done = finish_reads(&mut d, Some(fixtures::HEAD));
    assert!(done.contains(&Request::ContinueRelease { task: fixtures::CHANGE }));
    outputs(&mut d, Event::ContinueRelease { task: fixtures::CHANGE, entry: ENTRY });
    assert_eq!(d.entries.get(&ENTRY).unwrap().entry.effect.write, Write::Close { number: 1 });
}

#[test]
fn release_waits_for_a_lost_writers_restart_read() {
    let mut lost = hold(C, fixtures::CHANGE);
    lost.writer = Some(Writer::Lost { task: fixtures::CHANGE, attempt: 1 });
    let mut d = restored(C, Box::new([Stored::Hold(lost), Stored::Release(release_row(fixtures::CHANGE))]));
    begin(&mut d);
    no_release_decision(&outputs(&mut d, Event::ContinueRelease { task: fixtures::CHANGE, entry: ENTRY }));
    assert_eq!(d.hold(&name(C)).unwrap().writer, Some(Writer::Lost { task: fixtures::CHANGE, attempt: 1 }));
    let done = finish_reads(&mut d, Some(fixtures::HEAD));
    assert!(done.contains(&Request::ContinueRelease { task: fixtures::CHANGE }));
    let released = outputs(&mut d, Event::ContinueRelease { task: fixtures::CHANGE, entry: ENTRY });
    assert!(!released.contains(&Request::ReleaseFailed { task: fixtures::CHANGE, account: None, reason: None }));
    assert_eq!(d.entries.get(&ENTRY).unwrap().entry.effect.write, Write::DeleteBranch { branch: C.into() });
}

#[test]
fn design_claim_waits_for_fresh_ownership_and_wakes() {
    let mut d = restored(
        G,
        Box::new([
            Stored::Hold(hold(G, GOAL)),
            Stored::BranchHead(BranchHead { name: name(G), commit: fixtures::HEAD, owned: false }),
        ]),
    );
    begin(&mut d);
    let claimed = outputs(
        &mut d,
        Event::Claim {
            task: GOAL,
            attempt: 1,
            writes: Box::new([]),
            designs: Box::new([name(G)]),
            holders: Box::new([]),
        },
    );
    assert!(claimed.is_empty(), "a claim is neither permitted nor refused: {claimed:?}");
    assert!(d.hold(&name(G)).unwrap().design_writers.is_empty());
    let done = finish_reads(&mut d, Some(fixtures::HEAD));
    assert!(done.contains(&Request::WakeProcedure { task: GOAL }), "{done:?}");
    assert_eq!(
        d.hold(&name(G)).unwrap().design_writers.as_ref(),
        &[DesignWriter { task: GOAL, attempt: 1, rereading: false }]
    );
}

#[test]
fn write_claim_waits_before_judging_the_restored_writer() {
    let mut held = hold(C, fixtures::CHANGE);
    held.writer = Some(Writer::Run { task: GOAL, attempt: 1 });
    let mut d = restored(C, Box::new([Stored::Hold(held)]));
    begin(&mut d);
    let claimed = outputs(
        &mut d,
        Event::Claim {
            task: fixtures::CHANGE,
            attempt: 2,
            writes: Box::new([name(C)]),
            designs: Box::new([]),
            holders: Box::new([]),
        },
    );
    assert!(claimed.is_empty(), "a restored writer cannot refuse the claim: {claimed:?}");
    let done = finish_reads(&mut d, Some(fixtures::HEAD));
    assert!(done.contains(&Request::Taken { task: fixtures::CHANGE, resource: name(C), by: GOAL }), "{done:?}");
    assert!(done.contains(&Request::WakeProcedure { task: fixtures::CHANGE }));
}

fn seed(task: u64, parent: Option<u64>) -> items::Seed {
    items::Seed {
        project: 5,
        modify: true,
        task,
        parent,
        repository: REPO,
        run_branch: b"temper/7/r7".as_slice().into(),
        saved_branch: None,
    }
}

#[test]
fn workspace_waits_for_fresh_branch_existence() {
    let mut change = fixtures::opening_row();
    change.pending = None;
    change.delegate = Some((GOAL, change::Delegate::Produce));
    change.change.state = change::State::Producing { requested: true };
    let mut d = restored(C, Box::new([Stored::Change(Box::new(change))]));
    d.heads.remove(&name(C));
    begin(&mut d);
    assert!(d.workspace_item(seed(GOAL, Some(fixtures::CHANGE))).is_none());
    finish_reads(&mut d, Some(fixtures::HEAD));
    assert_eq!(d.workspace_item(seed(GOAL, Some(fixtures::CHANGE))).unwrap().start, items::Start::Branch(C.into()));
}

#[test]
fn workspace_waits_for_the_fresh_base_head() {
    let mut change = fixtures::opening_row();
    change.pending = None;
    change.delegate = Some((GOAL, change::Delegate::Produce));
    change.change.state = change::State::Producing { requested: true };
    change.change.early = change::Early::Started(Box::new([]));
    let mut d = restored(
        C,
        Box::new([
            Stored::Change(Box::new(change)),
            Stored::BranchHead(BranchHead { name: name(b"main"), commit: fixtures::HEAD, owned: false }),
            Stored::Names { task: GOAL, resources: Box::new([name(C), name(b"main")]) },
        ]),
    );
    begin(&mut d);
    assert!(d.workspace_item(seed(GOAL, Some(fixtures::CHANGE))).is_none());
    finish_reads(&mut d, Some(fixtures::HEAD));
    assert_eq!(
        d.workspace_item(seed(GOAL, Some(fixtures::CHANGE))).unwrap().start,
        items::Start::Merge { branch: C.into(), base: fixtures::BASE }
    );
}

#[test]
fn a_saved_workspace_waits_only_when_it_needs_a_fresh_merge_base() {
    for merge in [false, true] {
        let mut change = fixtures::opening_row();
        change.pending = None;
        change.delegate = Some((GOAL, change::Delegate::Produce));
        change.change.state = change::State::Producing { requested: true };
        if merge {
            change.change.early = change::Early::Started(Box::new([]));
        }
        let mut d = restored(
            C,
            Box::new([
                Stored::Change(Box::new(change)),
                Stored::BranchHead(BranchHead { name: name(b"main"), commit: fixtures::HEAD, owned: false }),
                Stored::Names { task: GOAL, resources: Box::new([name(C), name(b"main")]) },
            ]),
        );
        begin(&mut d);
        let mut saved = seed(GOAL, Some(fixtures::CHANGE));
        saved.saved_branch = Some(b"temper/7/s7".as_slice().into());
        let before = d.workspace_item(saved.clone());
        if merge {
            assert!(before.is_none(), "a merge still needs the fresh base");
        } else {
            assert_eq!(before.unwrap().start, items::Start::Saved(b"temper/7/s7".as_slice().into()));
        }
        finish_reads(&mut d, Some(fixtures::HEAD));
        let after = d.workspace_item(saved).unwrap().start;
        assert_eq!(
            after,
            if merge {
                items::Start::Merge { branch: b"temper/7/s7".as_slice().into(), base: fixtures::BASE }
            } else {
                items::Start::Saved(b"temper/7/s7".as_slice().into())
            }
        );
    }
}

#[test]
fn design_workspace_waits_for_fresh_owned_goal_branch() {
    let mut d = restored(G, Box::new([Stored::Hold(hold(G, GOAL))]));
    begin(&mut d);
    fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
    assert!(!fresh_reads_in(&d), "the client's fresh pass still waits");
    assert!(d.design_workspace_waiting(GOAL));
    assert!(d.workspace_item(seed(GOAL, None)).is_none());
    finish_reads(&mut d, Some(fixtures::HEAD));
    assert!(!d.design_workspace_waiting(GOAL));
    assert_eq!(d.workspace_item(seed(GOAL, None)).unwrap().start, items::Start::Base(G.into()));
}

#[test]
fn a_reported_push_compares_its_existing_beside_during_catch_up() {
    let mut held = hold(C, fixtures::CHANGE);
    held.beside = Some(HeadBeside { head: fixtures::HEAD, waiting: None });
    held.writer = Some(Writer::Run { task: fixtures::CHANGE, attempt: 1 });
    let mut d = restored(C, Box::new([Stored::Hold(held)]));
    begin(&mut d);
    outputs(&mut d, Event::Answered { task: fixtures::CHANGE, attempt: 1, pushed: Box::new([(name(C), PUSH, true)]) });
    assert_eq!(d.hold(&name(C)).unwrap().beside, Some(HeadBeside { head: fixtures::HEAD, waiting: Some(PUSH) }));
    assert_eq!(d.branch_head(&name(C)), Some(fixtures::HEAD));
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Commits { before: fixtures::HEAD, after: PUSH, page: 1 }));
    fixtures::answer(&mut d, call, Ok(Answer::Commits { commits: Box::new([]), more: false }));
    assert_eq!(d.branch_head(&name(C)), Some(fixtures::HEAD), "the fresh design head already contains the late push");
    assert_eq!(d.hold(&name(C)).unwrap().beside, None);
    finish_reads(&mut d, Some(fixtures::HEAD));
}

#[test]
fn restored_written_comparison_can_confirm_before_the_other_fresh_reads() {
    let mut held = hold(C, fixtures::CHANGE);
    held.beside = Some(HeadBeside { head: fixtures::HEAD, waiting: Some(PUSH) });
    let mut d = restored(C, Box::new([Stored::Hold(held)]));
    begin(&mut d);
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Commits { before: fixtures::HEAD, after: PUSH, page: 1 }));
    let done = fixtures::answer(
        &mut d,
        call,
        Ok(Answer::Commits {
            commits: Box::new([skein_forge::api::CommitMessage {
                id: PUSH,
                parents: Box::new([fixtures::HEAD]),
                message: Box::new([]),
            }]),
            more: false,
        }),
    );
    assert_eq!(d.branch_head(&name(C)), Some(PUSH), "its own comparison confirms the reported push: {done:?}");
    assert!(!fresh_reads_in(&d), "the client's restart branch read is still unanswered");
    finish_reads(&mut d, Some(PUSH));
}

#[test]
fn reported_push_compares_with_the_restored_head() {
    let mut d = restored(C, Box::new([Stored::Hold(hold(C, fixtures::CHANGE))]));
    begin(&mut d);
    let mut out = Queue::uncapped();
    domain::record_written_head(&mut d, &env(), resource(C), name(C), PUSH, fixtures::CHANGE, &mut out);
    assert_eq!(d.branch_head(&name(C)), Some(fixtures::HEAD));
    assert_eq!(d.hold(&name(C)).unwrap().beside, Some(HeadBeside { head: fixtures::HEAD, waiting: Some(PUSH) }));
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Commits { before: fixtures::HEAD, after: PUSH, page: 1 }));
    let done = fixtures::answer(
        &mut d,
        call,
        Ok(Answer::Commits {
            commits: Box::new([skein_forge::api::CommitMessage {
                id: PUSH,
                parents: Box::new([fixtures::HEAD]),
                message: Box::new([]),
            }]),
            more: false,
        }),
    );
    assert_eq!(d.branch_head(&name(C)), Some(PUSH));
    let mut saved = false;
    for request in &done {
        if let Request::Save { record: Stored::BranchHead(head), .. } = request {
            saved |= head.commit == PUSH && head.owned;
        }
    }
    assert!(saved);
    finish_reads(&mut d, Some(PUSH));
}

#[test]
fn a_reported_push_comparison_survives_another_restart() {
    let mut held = hold(C, fixtures::CHANGE);
    held.writer = Some(Writer::Run { task: fixtures::CHANGE, attempt: 1 });
    let mut d = restored(C, Box::new([Stored::Hold(held)]));
    begin(&mut d);
    let reported = outputs(
        &mut d,
        Event::Answered { task: fixtures::CHANGE, attempt: 1, pushed: Box::new([(name(C), PUSH, true)]) },
    );
    let mut saved = None;
    for request in &reported {
        if let Request::Save { record: Stored::Hold(hold), .. } = request {
            saved = Some(hold.clone());
        }
    }
    let saved = saved.expect("the waiting push is durable before any read answers");
    assert_eq!(saved.beside, Some(HeadBeside { head: fixtures::HEAD, waiting: Some(PUSH) }));
    assert_eq!(d.branch_head(&name(C)), Some(fixtures::HEAD));
    let mut d = restored(C, Box::new([Stored::Hold(saved)]));
    begin(&mut d);
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Commits { before: fixtures::HEAD, after: PUSH, page: 1 }));
    let (fresh_call, fresh_op) = fixtures::next_call(&mut d);
    assert_eq!(fresh_op, Op::Read(Read::Branch { branch: C.into() }));
    let observed = fixtures::answer(&mut d, fresh_call, Ok(Answer::Commit(PUSH)));
    assert_eq!(d.hold(&name(C)).unwrap().drift, None, "the pending comparison still writes the branch: {observed:?}");
    let confirmed = fixtures::answer(
        &mut d,
        call,
        Ok(Answer::Commits {
            commits: Box::new([skein_forge::api::CommitMessage {
                id: PUSH,
                parents: Box::new([fixtures::HEAD]),
                message: Box::new([]),
            }]),
            more: false,
        }),
    );
    assert_eq!(d.branch_head(&name(C)), Some(PUSH));
    let mut pushed = false;
    for request in &confirmed {
        if let Request::Save { record: Stored::Client(skein_forge_client::Stored::Live(row)), .. } = request {
            pushed |= row.watch.resource == resource(C) && row.pushed == Some(PUSH);
        }
    }
    assert!(pushed, "the comparison sends Pushed to the client: {confirmed:?}");
    let fresh = finish_reads(&mut d, Some(PUSH));
    for request in &fresh {
        assert!(!matches!(request, Request::Drift { .. }), "{fresh:?}");
    }
    assert_eq!(d.hold(&name(C)).unwrap().drift, None);
}

#[test]
fn a_push_without_a_held_recorded_head_is_taken_at_once() {
    for has_hold in [false, true] {
        let mut d = restored(C, Box::new([]));
        if has_hold {
            outputs(&mut d, Event::Restore { record: Stored::Hold(hold(C, fixtures::CHANGE)) });
            d.heads.remove(&name(C));
        }
        begin(&mut d);
        domain::record_written_head(
            &mut d,
            &env(),
            resource(C),
            name(C),
            PUSH,
            fixtures::CHANGE,
            &mut Queue::uncapped(),
        );
        assert_eq!(d.branch_head(&name(C)), Some(PUSH));
        if let Some(hold) = d.hold(&name(C)) {
            assert_eq!(hold.beside, None);
        }
    }
}

#[test]
fn a_failed_close_waits_for_fresh_pull_state_before_failing_release() {
    let mut change = fixtures::opening_row();
    change.pending = None;
    change.pull = Some(1);
    change.change.state = change::State::Checking { head: fixtures::HEAD };
    let row = ReleaseRow { close_pull: Some(1), pending: Some((ENTRY, None)), ..release_row(fixtures::CHANGE) };
    let mut entry = status(ENTRY, fixtures::CHANGE);
    entry.effect.write = Write::Close { number: 1 };
    let mut d = restored(
        C,
        Box::new([Stored::Change(Box::new(change)), Stored::Release(row), Stored::Entry { entry, held: true }]),
    );
    begin(&mut d);
    let mut out = Queue::uncapped();
    domain::scripted_outcome(
        &mut d,
        &env(),
        ENTRY,
        fixtures::CHANGE,
        skein_forge_client::Outcome::Failed(Error::Refused),
        &mut out,
    );
    for request in &out {
        assert!(!matches!(request, Request::ReleaseFailed { .. }), "{out:?}");
    }
    assert_eq!(d.releases.get(&fixtures::CHANGE).unwrap().pending, Some((ENTRY, None)));
    assert!(d.entries.contains_key(&ENTRY), "the durable pending release retains its entry until judgement");
    let done = finish_reads(&mut d, Some(fixtures::HEAD));
    let mut failed = false;
    for request in &done {
        if let Request::ReleaseFailed { task, .. } = request {
            failed |= *task == fixtures::CHANGE;
        }
    }
    assert!(failed, "{done:?}");
    assert!(!d.entries.contains_key(&ENTRY));
}

#[test]
fn release_effect_settlement_waits_for_fresh_reads() {
    let row = ReleaseRow { effects_settled: false, releasing: false, ..release_row(GOAL) };
    let mut d = restored(C, Box::new([Stored::Release(row)]));
    begin(&mut d);
    no_release_decision(&outputs(&mut d, Event::ContinueRelease { task: GOAL, entry: ENTRY }));
    assert!(!d.releases.get(&GOAL).unwrap().effects_settled);
    let done = finish_reads(&mut d, Some(fixtures::HEAD));
    assert!(done.contains(&Request::ContinueRelease { task: GOAL }));
    assert!(
        outputs(&mut d, Event::ContinueRelease { task: GOAL, entry: ENTRY })
            .contains(&Request::EffectsSettled { task: GOAL })
    );
}

#[test]
fn rebase_restart_reads_the_branch_instead_of_the_cached_tip() {
    let mut row = fixtures::opening_row();
    row.change.state = change::State::Preparing { from: fixtures::HEAD, main: fixtures::BASE, ready_since: env().wall };
    row.pending = Some(ENTRY);
    let mut entry = status(ENTRY, fixtures::CHANGE);
    entry.effect = skein_forge_client::Effect {
        write: Write::Update { number: 1, style: skein_forge::api::UpdateStyle::Rebase },
        condition: skein_forge_client::Condition::Update { head: fixtures::HEAD, base: fixtures::BASE },
    };
    let mut d = restored(
        C,
        Box::new([
            Stored::Hold(hold(C, fixtures::CHANGE)),
            Stored::Change(Box::new(row)),
            Stored::Entry { entry, held: false },
        ]),
    );
    // Only the branch read may explain a rewritten tip before the entry's outcome.
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);
    assert_eq!(d.change(fixtures::CHANGE).unwrap().drift, None);
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: C.into() }));
    fixtures::answer(&mut d, call, Ok(Answer::Commit(PUSH)));
    finish_reads(&mut d, Some(PUSH));
    // An unattributed rewrite waits for its entry's exact outcome, then judges this read's head.
    domain::change_outcome(
        &mut d,
        &env(),
        ENTRY,
        skein_forge_client::Outcome::Failed(Error::Refused),
        &mut Queue::uncapped(),
    );
    assert_eq!(d.change(fixtures::CHANGE).unwrap().drift, Some(change::Hold::BranchMoved));
}

#[test]
fn landing_criteria_wait_until_rebuild_and_the_fresh_reads() {
    use temper_engine_domain_forge_landing as policy;
    let change = fixtures::opening_row();
    let evidence = ChangeEvidence {
        touched: Touched::UNKNOWN,
        size: None,
        prior: change.change.clone(),
        head: Some(fixtures::HEAD),
        base_tip: fixtures::BASE,
        contains_base: change::Status::Passed,
        ci: change::Status::Passed,
        gates: Box::new([]),
        reviews: Box::new([]),
        reviews_complete: true,
    };
    let row = LandingRow {
        read_hold: None,
        last_event: None,
        turn_since: None,
        waiting: None,
        follows: None,
        flake_reruns: Box::new([]),
        room: None,
        job: None,
        landing: policy::Landing {
            last_left: None,
            searched: None,
            round: policy::Round::new(b"temper/7/l61".as_slice().into()),
            task: 61,
            project: 5,
            phase: policy::Phase::Preparing,
            members: Box::new([policy::Member {
                round: None,
                unit: 0,
                branch: b"temper/7/l61".as_slice().into(),
                combined: None,
                recorded: false,
                told: false,
                judging: false,
                leave_told: false,
                left: None,
                change: fixtures::CHANGE,
                repository: policy::Repository { forge: REPO.forge, repository: REPO.repository },
                pull: 1,
                head: Some(fixtures::HEAD),
                base: Some(fixtures::BASE),
                candidate: Some(fixtures::HEAD),
                gates: Box::new([]),
                asked: None,
                landed: None,
            }]),
            preparation: 0,
            stale_proofs: 0,
            last_prepare: None,
            branch: b"main".as_slice().into(),
        },
        asks: Box::new([None]),
        entries: Box::new([]),
    };
    let mut d = restored(C, Box::new([Stored::Change(Box::new(change)), Stored::Landing(row.clone())]));
    assert_eq!(landing::judge(&d, &row, Criterion::Ci, fixtures::HEAD, &evidence, &[]), Some(JudgeVerdict::Wait));
    begin(&mut d);
    assert_eq!(landing::judge(&d, &row, Criterion::Ci, fixtures::HEAD, &evidence, &[]), Some(JudgeVerdict::Wait));
    let done = finish_reads(&mut d, Some(fixtures::HEAD));
    assert!(done.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }));
    assert_eq!(landing::judge(&d, &row, Criterion::Ci, fixtures::HEAD, &evidence, &[]), Some(JudgeVerdict::Met));
}

#[test]
fn projection_and_its_due_wait_for_fresh_reads() {
    for requested in [false, true] {
        let view = projection::goal_view();
        let goal = view.goal;
        let mut d = restored(
            C,
            Box::new([Stored::Issue(IssueRow {
                goal,
                repository: REPO,
                state: issues::Projected::default(),
                number: None,
                pending: None,
                before: None,
                failed: false,
                desired: Some(view.clone()),
            })]),
        );
        begin(&mut d);
        if requested {
            let out = outputs(&mut d, Event::Project { entry: ENTRY, repository: REPO, view });
            for request in &out {
                assert!(!matches!(request, Request::ProjectionEffect { .. }), "{out:?}");
            }
        }
        let mut fired = Queue::uncapped();
        fire(&mut d, &env(), &mut fired);
        for request in &fired {
            assert!(!matches!(request, Request::ProjectAfter { goal: g, when: None } if *g == goal), "{fired:?}");
        }
        let done = finish_reads(&mut d, Some(fixtures::HEAD));
        assert!(done.contains(&Request::ProjectAfter { goal, when: None }), "{done:?}");
        let after = outputs(&mut d, Event::ProjectDesired { entry: ENTRY, goal });
        let mut effect = false;
        for request in &after {
            effect |= matches!(request, Request::ProjectionEffect { .. });
        }
        assert!(effect, "{after:?}");
    }
}

#[test]
fn a_release_waiting_for_fresh_observations_fails_at_its_step_figure() {
    let mut d = restored(G, Box::new([Stored::Hold(hold(G, GOAL)), Stored::Release(release_row(GOAL))]));
    begin(&mut d);
    outputs(&mut d, Event::ContinueRelease { task: GOAL, entry: ENTRY });
    let clock = env();
    let bound = advanced_wall(Wall::EPOCH, clock.limits.waits.step);
    let before = fire_at(&mut d, &clock, Wall::from_nanos(bound.as_nanos().saturating_sub(1)));
    no_release_decision(&before);
    let failed = fire_at(&mut d, &clock, bound);
    let expected = Request::ReleaseFailed { task: GOAL, account: None, reason: Some(b"no progress at release for 30 minutes: waiting for fresh observations from the forge; the last answer: no answer".as_slice().into()) };
    assert!(failed.contains(&expected), "{failed:?}");
    assert!(d.releases.get(&GOAL).expect("held release").failed);
}

#[test]
fn fresh_reads_do_not_fail_a_release_with_a_cleanup_write_in_flight() {
    const PULL: u64 = 1;
    const KEY: &[u8] = b"cleanup-key";
    const REASON: &[u8] = b"cancel reason";
    for (write, resource, pull, comment) in [
        (Write::Post { number: PULL, key: KEY.into(), body: REASON.into() }, None, Some(PULL), Some(Box::from(REASON))),
        (Write::Close { number: PULL }, None, Some(PULL), None),
        (Write::DeleteBranch { branch: G.into() }, Some(name(G)), None, None),
    ] {
        let pending = Some((ENTRY, resource));
        let row = ReleaseRow { close_pull: pull, pending: pending.clone(), comment, ..release_row(GOAL) };
        let mut entry = status(ENTRY, GOAL);
        entry.effect.write = write;
        let mut d = restored(
            G,
            Box::new([Stored::Hold(hold(G, GOAL)), Stored::Release(row), Stored::Entry { entry, held: false }]),
        );
        begin(&mut d);
        outputs(&mut d, Event::ContinueRelease { task: GOAL, entry: ENTRY });
        let clock = env();
        let out = fire_at(&mut d, &clock, advanced_wall(Wall::EPOCH, clock.limits.waits.step));
        no_release_decision(&out);
        assert_eq!(d.releases.get(&GOAL).expect("pending release").pending, pending);
    }
}
