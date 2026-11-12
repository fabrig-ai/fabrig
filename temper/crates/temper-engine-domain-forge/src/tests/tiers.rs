//! Tier lifecycle stories (temper/docs/design/domain/after-landing.md, 2–4 and 10).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary lifecycle stories collect and inspect requests outside the step implementation"
)]
use super::*;
use fixtures::{BASE, CHANGE, HEAD, OPEN};
use skein_forge::api;
use temper_engine_domain_forge_landing as keys;
use temper_engine_domain_forge_tiers as p;
mod goals;
mod red;

const TIER: &[u8] = b"browser";
const TIER_TASK: u64 = 61;
const NEXT_TASK: u64 = 62;
const DISPATCH: u64 = 21;
const RELEASE: u64 = 22;
const RUN: u64 = 71;
const JOB: u64 = 81;
const NEXT_TIP: api::Commit = [6; 32];
const LAST_TIP: api::Commit = [7; 32];
const READ: Token = Token::new(1_u64 << 59);
const RETRIES: u32 = 2; // The fixture's configured tier_retries.
const SMALL_RANGE: usize = 3; // 2 * one live change + 1.
pub(super) fn restoring(branch: Option<&[u8]>) -> Domain {
    restoring_following(match branch {
        Some(branch) => Follows::Branch(branch.into()),
        None => Follows::Landing,
    })
}
fn restoring_following(follows: Follows) -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    assert!(d.project_landing(
        5,
        temper_engine_domain_forge_change::Way::Queue,
        Box::new([RepositoryGates {
            repository: REPO,
            gates: Box::new([]),
            tiers: Box::new([Tier {
                name: TIER.into(),
                workflow: b"tier.yml".as_slice().into(),
                ignored: Box::new([b"docs".as_slice().into()]),
                artifact: b"report".as_slice().into(),
                format: p::Format::Lines,
                follows,
                span: None,
                description: Box::default(),
            }])
        }]),
        temper_engine_domain_forge_landing::Project::DEFAULT,
        AfterLanding {
            verdict: p::Settings { retries: RETRIES, suspects: p::Suspects::Bisect },
            bound: Duration::from_secs(7200)
        }
    ));
    d
}
fn configured(branch: Option<&[u8]>) -> Domain {
    let mut d = restoring(branch);
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    d
}
pub(super) fn row(d: &Domain) -> &TierRow {
    d.tiers.get(&(REPO, TIER.into())).expect("tier row")
}
fn main_name() -> Name {
    Name { forge: REPO.forge, repository: REPO.repository, what: What::Branch(Box::new([b"main".as_slice().into()])) }
}
fn seen(d: &mut Domain, before: Option<api::Commit>, after: api::Commit) -> Box<[Request]> {
    let name = main_name();
    d.heads.insert(name.clone(), BranchHead { name, commit: after, owned: false }).unwrap();
    let mut out = Queue::uncapped();
    crate::tiers::observed(d, &env(), REPO, b"main", before, after, &mut out);
    collect(out)
}
fn collect(mut out: Queue<Request>) -> Box<[Request]> {
    let mut values = Vec::new();
    while let Some(request) = out.pop() {
        values.push(request);
    }
    values.into_boxed_slice()
}
pub(super) fn key_trees(d: &mut Domain) -> Box<[Request]> {
    let mut all = Vec::new();
    // One branch watch, the baseline and at most the range's tips need answers.
    for _ in 0..LIMITS.changes * 2 + 3 {
        if d.tier_ops.reads.is_empty() {
            break;
        }
        let (call, op) = fixtures::next_call(d);
        let tip = match op {
            api::Op::Read(api::Read::Tree { object, .. }) => object,
            api::Op::Read(api::Read::Branch { branch }) => {
                let name = Name {
                    forge: REPO.forge,
                    repository: REPO.repository,
                    what: What::Branch(branch.split(|byte| *byte == b'/').map(Box::from).collect()),
                };
                let tip = d.heads.get(&name).map_or(BASE, |head| head.commit);
                all.extend(fixtures::answer(d, call, Ok(api::Answer::Commit(tip))));
                continue;
            }
            api::Op::Read(api::Read::Runs { .. }) if !d.tier_ops.follows.contains_key(&(REPO, TIER.into())) => {
                // A dropped run's listing can precede the new baseline's read.
                all.extend(fixtures::answer(d, call, Ok(api::Answer::Runs { runs: Box::new([]), more: false })));
                continue;
            }
            _ => panic!("key tree read: {op:?}"),
        };
        let source = if tip == HEAD { BASE } else { tip };
        let entries = Box::new([
            api::TreeEntry {
                name: b"docs".as_slice().into(),
                mode: 0o100_644,
                kind: api::TreeEntryKind::Blob,
                object: tip,
            },
            api::TreeEntry {
                name: b"src".as_slice().into(),
                mode: 0o100_644,
                kind: api::TreeEntryKind::Blob,
                object: source,
            },
        ]);
        all.extend(fixtures::answer(d, call, Ok(api::Answer::Tree { entries, more: false })));
    }
    all.into_boxed_slice()
}
fn baseline_tip(d: &mut Domain, tip: api::Commit) {
    assert!(requests(&baseline_answer(d, Ok(api::Answer::Commit(tip)))).is_empty());
}
fn baseline(d: &mut Domain) -> Box<[Request]> {
    seen(d, None, BASE);
    baseline_tip(d, BASE);
    key_trees(d)
}
fn attach(d: &mut Domain, number: u64, task: u64) {
    outputs(d, Event::TierTask { repository: REPO, tier: TIER.into(), number, task });
}
fn tier_step(d: &mut Domain, entry: u64, released: bool) -> Box<[Request]> {
    outputs(
        d,
        Event::StepTier {
            task: row(d).task.expect("tier task"),
            entry,
            heard: TierHeard { released, message_bytes: 4096 },
        },
    )
}
fn made(d: &mut Domain, entry: u64, made: skein_forge_client::Made) {
    outcome(d, entry, skein_forge_client::Outcome::Made { made, found: false });
}
fn outcome(d: &mut Domain, entry: u64, outcome: skein_forge_client::Outcome) {
    if let Some(kept) = d.entries.get_mut(&entry) {
        kept.entry.start = Some(skein_forge_client::Position { at: env().now, comment: 0, review: 0, review_page: 1 });
    }
    let mut out = Queue::uncapped();
    crate::tiers::outcome(d, entry, outcome, &mut out);
    d.entries.remove(&entry);
}
fn dispatching() -> Domain {
    let mut d = configured(None);
    baseline(&mut d);
    attach(&mut d, 1, TIER_TASK);
    tier_step(&mut d, OPEN, false);
    made(&mut d, OPEN, skein_forge_client::Made::Branch(BASE));
    tier_step(&mut d, DISPATCH, false);
    d
}
pub(super) fn dispatched() -> Domain {
    let mut d = dispatching();
    made(&mut d, DISPATCH, skein_forge_client::Made::Set);
    d
}
fn workflow(d: &Domain, conclusion: api::Conclusion, failed_step: Option<u32>) -> api::Run {
    api::Run {
        id: RUN,
        number: 1,
        workflow: b"tier.yml".as_slice().into(),
        commit: d
            .tier_ops
            .follows
            .get(&(REPO, TIER.into()))
            .map_or(row(d).record.run.as_ref().unwrap().tip, |follow| follow.commit),
        event: api::Event::Dispatch,
        actor: 7,
        state: api::RunState::Completed,
        conclusion: Some(conclusion),
        url: b"https://forge.example/run/71".as_slice().into(),
        created: env().now,
        jobs: match failed_step {
            Some(number) => Box::new([api::RunJob {
                id: JOB,
                name: b"tests".as_slice().into(),
                attempt: 1,
                state: api::RunState::Completed,
                conclusion: Some(api::Conclusion::Failure),

                steps: 3,
                step: Some(api::Step { number, name: b"step".as_slice().into() }),
            }]),
            None => Box::new([]),
        },
    }
}
fn answer(d: &mut Domain, answer: api::Answer) -> Box<[Request]> {
    d.tier_ops.reads.insert(READ, crate::tiers::Read { tier: (REPO, TIER.into()), baseline: false }).unwrap();
    let mut out = Queue::uncapped();
    crate::tiers::answered(d, &env(), READ, Ok(answer), &mut out);
    collect(out)
}
fn completed(d: &mut Domain, conclusion: api::Conclusion, failed_step: Option<u32>) {
    let run = workflow(d, conclusion, failed_step);
    answer(d, api::Answer::Runs { runs: Box::new([run.clone()]), more: false });
    answer(d, api::Answer::Run(run));
}
fn finish(d: &mut Domain) -> Box<[Request]> {
    tier_step(d, RELEASE, false);
    made(d, RELEASE, skein_forge_client::Made::Set);
    let out = tier_step(d, RELEASE + 1, false);
    // Stories without log fixtures finish the disposable read as unavailable.
    if row(d).record.run.is_some() && row(d).result.is_some() {
        let owner = *d.tier_ops.reads.iter().next().expect("red run read").0;
        let mut reads = Queue::uncapped();
        crate::tiers::answered(d, &env(), owner, Err(api::Error::Missing), &mut reads);
        tier_step(d, RELEASE + 2, false)
    } else {
        out
    }
}
fn requests(out: &[Request]) -> Vec<(u64, api::Commit)> {
    out.iter()
        .filter_map(|request| match request {
            Request::TierRun { number, tip, .. } => Some((*number, *tip)),
            _ => None,
        })
        .collect()
}
fn join_words(parts: &[&[u8]]) -> Box<[u8]> {
    parts.concat().into_boxed_slice()
}
fn choice(out: &[Request]) -> Option<&TierDecision> {
    out.iter().find_map(|request| match request {
        Request::TierDecision { decision, .. } => Some(decision),
        _ => None,
    })
}
fn baseline_answer(d: &mut Domain, result: Result<api::Answer, api::Error>) -> Box<[Request]> {
    let (owner, read) = d.tier_ops.reads.iter().next().expect("tier-owned fresh baseline read");
    assert_eq!(read.tier, (REPO, TIER.into()));
    assert!(read.baseline);
    let owner = *owner;
    let expected = match &crate::tiers::configured_all(d, REPO).first().unwrap().follows {
        Follows::Landing => b"main".as_slice().into(),
        Follows::Branch(branch) => branch.clone(),
        Follows::Goals => panic!("goal tiers have no baseline"),
    };
    let mut out = Vec::new();
    // The branch watch can be served before the tier's own fresh read.
    for _ in 0_u8..2 {
        let (call, op) = fixtures::next_call(d);
        assert_eq!(op, api::Op::Read(api::Read::Branch { branch: expected.clone() }));
        out.extend(fixtures::answer(d, call, result.clone()));
        if !d.tier_ops.reads.contains_key(&owner) {
            return out.into_boxed_slice();
        }
    }
    panic!("the tier's baseline read must be answered")
}

#[test]
fn baseline_reads_past_a_stale_head_at_start_and_restart() {
    for restart in [false, true] {
        let mut d = restoring(None);
        let name = main_name();
        outputs(&mut d, Event::Restore { record: Stored::BranchHead(BranchHead { name, commit: BASE, owned: false }) });
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        let mut out = Queue::uncapped();
        if restart {
            crate::tiers::restart(&mut d, &env(), &mut out);
        } else {
            crate::tiers::start_due(&mut d, &env(), REPO, &mut out);
        }
        assert!(requests(&collect(out)).is_empty());
        assert!(row(&d).baseline.is_none(), "a restored head is not a baseline");
        assert!(requests(&outputs(&mut d, Event::ReadAfresh)).is_empty());
        assert_eq!(d.tier_ops.reads.len(), 1);
        let out = baseline_answer(&mut d, Ok(api::Answer::Commit(NEXT_TIP)));
        assert!(requests(&out).is_empty());
        assert_eq!(requests(&key_trees(&mut d)), [(1, NEXT_TIP)]);
        assert!(row(&d).record.run.as_ref().unwrap().landings.is_empty());
    }
}

#[test]
fn moves_before_the_baseline_answer_are_not_landings() {
    let mut d = configured(None);
    seen(&mut d, None, BASE);
    let out = seen(&mut d, Some(BASE), HEAD);
    assert!(requests(&out).is_empty());
    assert!(row(&d).baseline.is_none(), "a move cannot choose the baseline");
    assert!(row(&d).pending.is_empty());
    baseline_answer(&mut d, Ok(api::Answer::Commit(NEXT_TIP)));
    assert_eq!(requests(&key_trees(&mut d)), [(1, NEXT_TIP)]);
    assert!(row(&d).record.run.as_ref().unwrap().landings.is_empty());
    attach(&mut d, 1, TIER_TASK);
    tier_step(&mut d, OPEN, false);
    made(&mut d, OPEN, skein_forge_client::Made::Branch(NEXT_TIP));
    tier_step(&mut d, DISPATCH, false);
    made(&mut d, DISPATCH, skein_forge_client::Made::Set);
    completed(&mut d, api::Conclusion::Success, None);
    assert!(requests(&finish(&mut d)).is_empty());
    assert!(row(&d).pending.is_empty());
    assert!(row(&d).record.run.is_none());
    assert_eq!(row(&d).record.judged.unwrap().tip, NEXT_TIP);
    seen(&mut d, Some(NEXT_TIP), LAST_TIP);
    assert_eq!(requests(&key_trees(&mut d)), [(2, LAST_TIP)]);
    assert_eq!(row(&d).record.run.as_ref().unwrap().landings.len(), 1);
}

#[test]
fn a_missing_baseline_branch_waits_for_its_next_move() {
    let mut d = restoring(None);
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let mut out = Queue::uncapped();
    crate::tiers::start_due(&mut d, &env(), REPO, &mut out);
    assert!(requests(&collect(out)).is_empty());
    assert!(requests(&outputs(&mut d, Event::ReadAfresh)).is_empty());
    let out = baseline_answer(&mut d, Err(api::Error::Missing));
    assert!(requests(&out).is_empty());
    assert!(row(&d).baseline.is_none());
    assert!(row(&d).record.run.is_none());
    assert!(d.tier_ops.reads.is_empty());
    seen(&mut d, None, BASE);
    baseline_answer(&mut d, Ok(api::Answer::Commit(BASE)));
    assert_eq!(requests(&key_trees(&mut d)), [(1, BASE)]);
    assert!(row(&d).record.run.as_ref().unwrap().landings.is_empty());
}

#[test]
fn a_baseline_waits_for_another_tier_read() {
    let mut d = restoring(None);
    d.tier_ops.reads.insert(READ, crate::tiers::Read { tier: (REPO, TIER.into()), baseline: false }).unwrap();
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    assert!(requests(&outputs(&mut d, Event::ReadAfresh)).is_empty());
    assert_eq!(d.tier_ops.reads.len(), 1, "wait for the occupied read slot");
    assert!(row(&d).baseline.is_none());
    let mut out = Queue::uncapped();
    crate::tiers::answered(&mut d, &env(), READ, Err(api::Error::Missing), &mut out);
    assert!(requests(&collect(out)).is_empty());
    assert_eq!(d.tier_ops.reads.len(), 1, "the baseline now owns the read slot");
    baseline_answer(&mut d, Ok(api::Answer::Commit(NEXT_TIP)));
    assert_eq!(requests(&key_trees(&mut d)), [(1, NEXT_TIP)]);
}

#[test]
fn a_late_baseline_answer_keeps_the_baseline_and_its_later_moves() {
    let mut d = configured(None);
    let owner = *d.tier_ops.reads.iter().next().expect("baseline read").0;
    // A baseline has started by another path while the read was in flight.
    d.tiers.get_mut(&(REPO, TIER.into())).unwrap().baseline = Some(TierLanding { commit: BASE, key: None });
    seen(&mut d, Some(BASE), LAST_TIP);
    let mut out = Queue::uncapped();
    crate::tiers::answered(&mut d, &env(), owner, Ok(api::Answer::Commit(NEXT_TIP)), &mut out);
    assert!(requests(&collect(out)).is_empty());
    assert_eq!(row(&d).baseline.unwrap().commit, BASE);
    assert_eq!(row(&d).pending.len(), 1);
    assert_eq!(row(&d).pending[0].commit, LAST_TIP);
    assert_eq!(requests(&key_trees(&mut d)), [(1, BASE)]);
    assert!(row(&d).record.run.as_ref().unwrap().landings.is_empty());
}

#[test]
fn first_tip_starts_a_baseline_with_no_landings() {
    let mut d = configured(None);
    let out = baseline(&mut d);
    assert_eq!(requests(&out), [(1, BASE)]);
    assert!(row(&d).record.run.as_ref().unwrap().landings.is_empty());
    let mut again = Queue::uncapped();
    crate::tiers::start_due(&mut d, &env(), REPO, &mut again);
    assert!(requests(&collect(again)).is_empty());
}
#[test]
fn green_run_releases_its_branch_and_makes_a_receipt() {
    let mut d = dispatched();
    completed(&mut d, api::Conclusion::Success, None);
    let out = tier_step(&mut d, RELEASE, false);
    let deleted = &d.entries.get(&RELEASE).unwrap().entry;
    assert_eq!(deleted.effect.write, api::Write::DeleteBranch { branch: b"temper/tier/browser.1".as_slice().into() });
    let mut released = false;
    for request in &out {
        if let Request::Save { release: Some(number), .. } = request {
            released |= *number == RELEASE;
        }
    }
    assert!(released);
    assert!(d.describe_tier_effect(TIER_TASK, RELEASE).is_none());
    made(&mut d, RELEASE, skein_forge_client::Made::Set);
    d.wait_tier(&env(), TIER_TASK, None);
    let out = tier_step(&mut d, RELEASE + 1, false);
    assert_eq!(
        choice(&out),
        Some(&TierDecision::Done { words: b"Tier browser run 1: green, cleared 0 landings".as_slice().into() })
    );
    assert!(row(&d).record.run.is_none());
    assert_eq!(d.next_wall_deadline(), None);
    assert!(row(&d).pending.is_empty());
    assert_eq!(row(&d).record.judged.unwrap().tip, BASE);
    assert!(d.landing.receipts.iter().any(|(key, receipt)| key.check == Check::Tier(TIER.into())
        && receipt.passed
        && receipt.failures == Some(p::Failures::Tests(Box::new([])))));
}
#[test]
fn two_moves_during_a_run_start_one_at_the_newest_tip() {
    let mut d = dispatched();
    seen(&mut d, Some(BASE), NEXT_TIP);
    seen(&mut d, Some(NEXT_TIP), LAST_TIP);
    key_trees(&mut d);
    completed(&mut d, api::Conclusion::Success, None);
    let out = finish(&mut d);
    assert_eq!(requests(&out), [(2, LAST_TIP)]);
    assert_eq!(
        row(&d).record.run.as_ref().unwrap().landings.iter().map(|landing| landing.commit).collect::<Vec<_>>(),
        [NEXT_TIP, LAST_TIP]
    );
    assert!(row(&d).pending.is_empty());
}
#[test]
fn ignored_documentation_is_judged_by_receipt_without_dispatch() {
    let mut d = dispatched();
    completed(&mut d, api::Conclusion::Success, None);
    finish(&mut d);
    seen(&mut d, Some(BASE), HEAD);
    let out = key_trees(&mut d);
    assert_eq!(requests(&out), [(2, HEAD)]);
    attach(&mut d, 2, NEXT_TASK);
    let out = outputs(
        &mut d,
        Event::StepTier {
            task: NEXT_TASK,
            entry: RELEASE + 2,
            heard: TierHeard { released: false, message_bytes: 4096 },
        },
    );
    assert_eq!(
        choice(&out),
        Some(&TierDecision::Done { words: b"Tier browser run 2: green, cleared 1 landing".as_slice().into() })
    );
    assert!(d.entries.is_empty());
    assert_eq!(row(&d).record.judged.unwrap().tip, HEAD);
}
#[test]
fn outside_push_is_kept_in_the_next_range() {
    let mut d = dispatched();
    seen(&mut d, Some(BASE), NEXT_TIP);
    key_trees(&mut d);
    completed(&mut d, api::Conclusion::Success, None);
    finish(&mut d);
    let run = row(&d).record.run.as_ref().unwrap();
    assert_eq!(run.landings.len(), 1);
    assert_eq!(run.landings[0].commit, NEXT_TIP);
    assert!(!d.landed.contains_key(&NEXT_TIP));
}
#[test]
fn setup_fault_is_retried_then_green() {
    let mut d = dispatched();
    completed(&mut d, api::Conclusion::Failure, Some(0));
    let out = tier_step(&mut d, DISPATCH + 1, false);
    assert_eq!(
        d.entries.get(&(DISPATCH + 1)).unwrap().entry.effect.write,
        api::Write::Rerun { attempt: api::JobAttempt { head: BASE, run: RUN, job: JOB, attempt: 1 } }
    );
    assert_eq!(choice(&out), Some(&TierDecision::Wait { until: None }));
    made(&mut d, DISPATCH + 1, skein_forge_client::Made::Set);
    completed(&mut d, api::Conclusion::Success, None);
    let out = finish(&mut d);
    assert_eq!(
        choice(&out),
        Some(&TierDecision::Done { words: b"Tier browser run 1: green, cleared 0 landings".as_slice().into() })
    );
}
#[test]
fn failed_run_without_a_report_is_a_fault() {
    let mut d = dispatched();
    completed(&mut d, api::Conclusion::Failure, Some(1));
    let mut out = Queue::uncapped();
    d.tier_ops.reads.insert(READ, crate::tiers::Read { tier: (REPO, TIER.into()), baseline: false }).unwrap();
    crate::tiers::answered(&mut d, &env(), READ, Err(api::Error::Missing), &mut out);
    let run = row(&d).record.run.as_ref().unwrap();
    assert_eq!((run.faults, &run.part), (1, &p::Part::Whole));
    assert!(row(&d).result.is_none());
    assert!(d.landing.receipts.is_empty());
}
#[test]
fn exhausted_faults_hold_until_released_then_run_green() {
    let mut d = dispatched();
    for fault in 0..=RETRIES {
        completed(&mut d, api::Conclusion::Cancelled, None);
        if fault < RETRIES {
            let entry = DISPATCH + u64::from(fault) + 1;
            tier_step(&mut d, entry, false);
            made(&mut d, entry, skein_forge_client::Made::Set);
        }
    }
    let out = tier_step(&mut d, RELEASE + 3, false);
    assert_eq!(
        choice(&out),
        Some(&TierDecision::Held {
            words: b"Tier browser run 1 held after 3 runner faults, last: cancelled".as_slice().into()
        })
    );
    tier_step(&mut d, RELEASE + 4, true);
    assert_eq!(row(&d).record.run.as_ref().unwrap().faults, 0);
    let api::Write::Dispatch { .. } = d.entries.get(&(RELEASE + 4)).unwrap().entry.effect.write else {
        panic!("released run dispatches afresh")
    };
    made(&mut d, RELEASE + 4, skein_forge_client::Made::Set);
    completed(&mut d, api::Conclusion::Success, None);
    let out = finish(&mut d);
    assert_eq!(
        choice(&out),
        Some(&TierDecision::Done { words: b"Tier browser run 1: green, cleared 0 landings".as_slice().into() })
    );
}
#[test]
fn missing_workflow_dispatches_again_then_holds_with_its_kind_in_both_words() {
    let mut d = dispatching();
    let kind = b"workflow tier.yml not found at 0404040404040404040404040404040404040404040404040404040404040404";
    for fault in 0..=RETRIES {
        let entry = DISPATCH + u64::from(fault);
        outcome(&mut d, entry, skein_forge_client::Outcome::Failed(api::Error::Missing));
        assert_eq!(row(&d).flight.as_ref().unwrap().fault, Some(keys::Fault::MissingWorkflow));
        let out = tier_step(&mut d, entry + 1, false);
        assert_eq!(row(&d).record.run.as_ref().unwrap().faults, fault + 1);
        if fault < RETRIES {
            let api::Write::Dispatch { workflow, .. } = &d.entries.get(&(entry + 1)).unwrap().entry.effect.write else {
                panic!("a missing workflow dispatches again")
            };
            assert_eq!(workflow.as_ref(), b"tier.yml");
        } else {
            assert_eq!(
                choice(&out),
                Some(&TierDecision::Held {
                    words: join_words(&[b"Tier browser run 1 held after 3 runner faults, last: ", kind]),
                })
            );
        }
    }
    let words = d.tier_execution_words(TIER_TASK).unwrap();
    let line = join_words(&[
        b"Runner fault: Judged run: commit 0404040404040404040404040404040404040404040404040404040404040404, branch temper/tier/browser.1; ",
        kind, b"\n",
    ]);
    assert_eq!(skein_lib::bytes::count(&words, &line, 4), 3);
    tier_step(&mut d, RELEASE + 4, true);
    assert_eq!(row(&d).record.run.as_ref().unwrap().faults, 0);
    made(&mut d, RELEASE + 4, skein_forge_client::Made::Set);
    completed(&mut d, api::Conclusion::Success, None);
    assert_eq!(
        choice(&finish(&mut d)),
        Some(&TierDecision::Done { words: b"Tier browser run 1: green, cleared 0 landings".as_slice().into() })
    );
}
#[test]
fn a_missing_workflow_in_the_saved_flight_keeps_its_words_after_restart() {
    let mut d = dispatching();
    for fault in 0..RETRIES {
        let entry = DISPATCH + u64::from(fault);
        outcome(&mut d, entry, skein_forge_client::Outcome::Failed(api::Error::Missing));
        tier_step(&mut d, entry + 1, false);
    }
    outcome(&mut d, DISPATCH + u64::from(RETRIES), skein_forge_client::Outcome::Failed(api::Error::Missing));
    let saved = row(&d).clone();
    assert_eq!(saved.flight.as_ref().unwrap().fault, Some(keys::Fault::MissingWorkflow));
    assert_eq!(saved.held, None, "the third refusal is saved before the step holds the task");
    let mut cold = restoring(None);
    outputs(&mut cold, Event::Restore { record: Stored::Tier(saved) });
    outputs(&mut cold, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut cold, Event::ReadAfresh);
    let out = tier_step(&mut cold, RELEASE + 3, false);
    let kind = b"workflow tier.yml not found at 0404040404040404040404040404040404040404040404040404040404040404";
    assert_eq!(
        choice(&out),
        Some(&TierDecision::Held {
            words: join_words(&[b"Tier browser run 1 held after 3 runner faults, last: ", kind]),
        })
    );
    let words = cold.tier_execution_words(TIER_TASK).unwrap();
    assert!(skein_lib::bytes::find(&words, &join_words(&[b"; ", kind, b"\n"])).is_some());
}
#[test]
fn other_dispatch_refusals_and_missing_reruns_stay_lost() {
    for rerun in [false, true] {
        let mut d = if rerun { dispatched() } else { dispatching() };
        let entry = if rerun {
            completed(&mut d, api::Conclusion::Failure, Some(0));
            tier_step(&mut d, DISPATCH + 1, false);
            let api::Write::Rerun { .. } = d.entries.get(&(DISPATCH + 1)).unwrap().entry.effect.write else {
                panic!("retry the failed job")
            };
            DISPATCH + 1
        } else {
            DISPATCH
        };
        let error = if rerun { api::Error::Missing } else { api::Error::Refused };
        outcome(&mut d, entry, skein_forge_client::Outcome::Failed(error));
        assert_eq!(row(&d).flight.as_ref().unwrap().fault, Some(keys::Fault::Lost));
        tier_step(&mut d, entry + 1, false);
        let words = d.tier_execution_words(TIER_TASK).unwrap();
        assert!(skein_lib::bytes::find(&words, b"; lost\n").is_some());
        assert!(skein_lib::bytes::find(&words, b"not found at").is_none());
    }
}
#[test]
fn branch_no_tier_follows_starts_no_run() {
    let mut d = configured(Some(b"release/next"));
    let out = seen(&mut d, None, BASE);
    assert!(requests(&out).is_empty());
    assert!(row(&d).baseline.is_none());
    assert!(row(&d).pending.is_empty());
    assert_eq!(
        crate::tiers::watched(&d).as_slice(),
        &[Name {
            forge: REPO.forge,
            repository: REPO.repository,
            what: What::Branch(Box::new([b"release".as_slice().into(), b"next".as_slice().into()]))
        }]
    );
}
#[test]
fn cold_restart_follows_a_run_without_dispatching_it_again() {
    let mut d = dispatched();
    let run = workflow(&d, api::Conclusion::Success, None);
    answer(&mut d, api::Answer::Runs { runs: Box::new([run.clone()]), more: false });
    let saved = row(&d).clone();
    let mut cold = restoring(None);
    outputs(&mut cold, Event::Restore { record: Stored::Tier(saved.clone()) });
    outputs(&mut cold, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut cold, Event::ReadAfresh);
    answer(&mut cold, api::Answer::Run(run));
    let out = finish(&mut cold);
    assert_eq!(
        choice(&out),
        Some(&TierDecision::Done { words: b"Tier browser run 1: green, cleared 0 landings".as_slice().into() })
    );
    assert_eq!(row(&cold).record.judged.unwrap().tip, BASE);
    assert_eq!(row(&cold).record.sequence, 1);
}
#[test]
fn tier_effects_describe_the_branch_dispatch_and_rerun() {
    let mut d = configured(None);
    baseline(&mut d);
    attach(&mut d, 1, TIER_TASK);
    tier_step(&mut d, OPEN, false);
    let (repository, what, effect) = d.describe_tier_effect(TIER_TASK, OPEN).unwrap();
    assert_eq!(
        (repository, what, effect.kind),
        (
            REPO,
            What::Branch(Box::new([
                b"temper".as_slice().into(),
                b"tier".as_slice().into(),
                b"browser.1".as_slice().into()
            ])),
            9
        )
    );
    outputs(&mut d, Event::VetoTier { task: TIER_TASK, entry: OPEN });
    assert_eq!(row(&d).branch, TierBranch::Absent);
    assert!(!d.entries.contains_key(&OPEN));
    tier_step(&mut d, OPEN, false);
    made(&mut d, OPEN, skein_forge_client::Made::Branch(BASE));
    tier_step(&mut d, DISPATCH, false);
    assert_eq!(d.describe_tier_effect(TIER_TASK, DISPATCH).unwrap().2.kind, 16);
    made(&mut d, DISPATCH, skein_forge_client::Made::Set);
    completed(&mut d, api::Conclusion::Failure, Some(0));
    tier_step(&mut d, DISPATCH + 1, false);
    assert_eq!(d.describe_tier_effect(TIER_TASK, DISPATCH + 1).unwrap().2.kind, 17);
}

#[test]
fn full_range_compacts_adjacent_outside_landings_preserving_changes() {
    let mut d = dispatched();
    let mut change = fixtures::opening_row();
    change.task = CHANGE;
    d.changes.insert(CHANGE, change).unwrap();
    d.landed.insert(HEAD, CHANGE).unwrap();
    d.tiers.get_mut(&(REPO, TIER.into())).unwrap().pending = Box::new([
        TierLanding { commit: [10; 32], key: Some([10; 32]) },
        TierLanding { commit: [11; 32], key: Some([11; 32]) },
        TierLanding { commit: HEAD, key: Some(HEAD) },
    ]);
    let mut small = env();
    small.limits.changes = 1;
    let mut out = Queue::uncapped();
    crate::tiers::observed(&mut d, &small, REPO, b"main", Some(HEAD), NEXT_TIP, &mut out);
    assert_eq!(row(&d).pending.len(), SMALL_RANGE);
    assert_eq!(
        row(&d).pending.as_ref(),
        &[
            TierLanding { commit: [11; 32], key: Some([11; 32]) },
            TierLanding { commit: HEAD, key: Some(HEAD) },
            TierLanding { commit: NEXT_TIP, key: None },
        ]
    );
}

#[test]
fn a_dropped_changes_landing_is_outside_the_tiers_own_range() {
    for drop in [false, true] {
        let mut d = dispatched();
        let mut change = fixtures::opening_row();
        change.task = CHANGE;
        change.change.task = CHANGE;
        change.change.state = temper_engine_domain_forge_change::State::Ended;
        change.ending = Some(Ending::Cancelled);
        d.changes.insert(CHANGE, change).unwrap();
        d.landed.insert(HEAD, CHANGE).unwrap();
        let mut small = env();
        small.limits.changes = 1;
        if drop {
            step(&mut d, &small, Event::DropChange { task: CHANGE }, &mut Queue::uncapped());
            assert!(d.change(CHANGE).is_none());
        }
        d.tiers.get_mut(&(REPO, TIER.into())).unwrap().pending = Box::new([
            TierLanding { commit: HEAD, key: Some(HEAD) },
            TierLanding { commit: [10; 32], key: Some([10; 32]) },
            TierLanding { commit: [11; 32], key: Some([11; 32]) },
        ]);
        crate::tiers::observed(&mut d, &small, REPO, b"main", Some(HEAD), NEXT_TIP, &mut Queue::uncapped());
        let first = if drop { [10; 32] } else { HEAD };
        assert_eq!(
            row(&d).pending.as_ref(),
            &[
                TierLanding { commit: first, key: Some(first) },
                TierLanding { commit: [11; 32], key: Some([11; 32]) },
                TierLanding { commit: NEXT_TIP, key: None },
            ],
            "dropped {drop}"
        );
    }
}

#[test]
fn a_green_whole_run_clears_its_nonempty_range() {
    let mut d = dispatched();
    seen(&mut d, Some(BASE), NEXT_TIP);
    key_trees(&mut d);
    completed(&mut d, api::Conclusion::Success, None);
    finish(&mut d);
    attach(&mut d, 2, NEXT_TASK);
    tier_step(&mut d, OPEN, false);
    made(&mut d, OPEN, skein_forge_client::Made::Branch(NEXT_TIP));
    tier_step(&mut d, DISPATCH, false);
    made(&mut d, DISPATCH, skein_forge_client::Made::Set);
    completed(&mut d, api::Conclusion::Success, None);
    let out = finish(&mut d);
    assert_eq!(
        choice(&out),
        Some(&TierDecision::Done { words: b"Tier browser run 2: green, cleared 1 landing".as_slice().into() })
    );
    assert!(row(&d).pending.is_empty());
    assert!(row(&d).record.run.is_none());
    assert_eq!(d.landing.receipts.len(), 2);
}

fn observe_branch(d: &mut Domain, expected: &[u8]) {
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, api::Op::Read(api::Read::Branch { branch: expected.into() }));
    fixtures::answer(d, call, Ok(api::Answer::Commit(BASE)));
    if row(d).baseline.is_none() {
        baseline_tip(d, BASE);
    }
}
#[test]
fn first_observation_of_each_followed_branch_starts_its_baseline() {
    for branch in [None, Some(b"release/next".as_slice())] {
        let mut d = configured(branch);
        observe_branch(&mut d, branch.unwrap_or(b"main"));
        let out = key_trees(&mut d);
        assert_eq!(requests(&out), [(1, BASE)]);
        assert!(row(&d).record.run.as_ref().unwrap().landings.is_empty());
    }
}
#[test]
fn naming_then_unnaming_a_task_keeps_the_tier_watch() {
    let mut d = configured(None);
    outputs(&mut d, Event::Names { task: CHANGE, resources: Box::new([main_name()]) });
    outputs(&mut d, Event::Unnamed { task: CHANGE });
    observe_branch(&mut d, b"main");
    assert_eq!(requests(&key_trees(&mut d)), [(1, BASE)]);
}
#[test]
fn retained_tier_receipts_protect_the_followed_tips_key() {
    let mut d = dispatched();
    completed(&mut d, api::Conclusion::Success, None);
    finish(&mut d);
    let original = d.landing.receipts.iter().next().unwrap().0.clone();
    let receipt = d.landing.receipts.get(&original).unwrap().clone();
    let mut small = env();
    small.limits.receipts_per_gate = 2;
    for key in [[8; 32], [9; 32]] {
        let mut out = Queue::uncapped();
        landing::put_receipt(&mut d, &small, ReceiptKey { key, ..original.clone() }, receipt.clone(), &mut out);
    }
    assert!(d.landing.receipts.contains_key(&original));
    assert_eq!(d.landing.receipts.len(), 2);
}

#[test]
fn a_vetoed_dispatch_is_a_fault_on_the_next_step() {
    let mut d = configured(None);
    baseline(&mut d);
    attach(&mut d, 1, TIER_TASK);
    tier_step(&mut d, OPEN, false);
    made(&mut d, OPEN, skein_forge_client::Made::Branch(BASE));
    tier_step(&mut d, DISPATCH, false);
    outputs(&mut d, Event::VetoTier { task: TIER_TASK, entry: DISPATCH });
    tier_step(&mut d, DISPATCH + 1, false);
    assert_eq!(row(&d).record.run.as_ref().unwrap().faults, 1);
    let api::Write::Dispatch { .. } = d.entries.get(&(DISPATCH + 1)).unwrap().entry.effect.write else {
        panic!("vetoed dispatch is retried")
    };
}

#[test]
fn tier_flights_and_walks_use_hearing_deadlines() {
    skein_rows::check(
        [
            ("quiet", (false, 60, 0, false, 5)),
            ("hearing", (true, 60, 0, false, 60)),
            ("poll is later", (true, 60, 58, false, 5)),
            ("expired", (true, 60, 60, false, 5)),
            ("disabled", (true, 0, 0, false, 5)),
            ("another repository", (true, 60, 0, true, 5)),
        ],
        tier_hearing_row,
    );
}
fn tier_hearing_row((hint, fallback, elapsed, other, delay): (bool, u64, u64, bool, u64)) {
    for walking in [false, true] {
        let mut d = if walking { configured(None) } else { dispatched() };
        let running = if walking {
            seen(&mut d, None, BASE);
            baseline_tip(&mut d, BASE);
            None
        } else {
            let mut run = workflow(&d, api::Conclusion::Success, None);
            run.state = api::RunState::Running;
            run.conclusion = None;
            answer(&mut d, api::Answer::Runs { runs: Box::new([run.clone()]), more: false });
            Some(run)
        };
        let mut clock = env();
        clock.limits.client.fallback = Duration::from_secs(fallback);
        if hint {
            fixtures::repository_heard(
                &mut d,
                &clock,
                if other { api::Repository { repository: 3, ..REPO } } else { REPO },
            );
        }
        clock.now = clock.now.saturating_add(Duration::from_secs(elapsed));
        let owner = if walking {
            *d.tier_ops.reads.iter().next().unwrap().0
        } else {
            d.tier_ops.reads.insert(READ, crate::tiers::Read { tier: (REPO, TIER.into()), baseline: false }).unwrap();
            READ
        };
        let result = match running {
            Some(run) => Ok(api::Answer::Run(run)),
            None => Err(api::Error::Busy),
        };
        let mut out = Queue::uncapped();
        crate::tiers::answered(&mut d, &clock, owner, result, &mut out);
        let expected = Some(clock.now.saturating_add(Duration::from_secs(delay)));
        if walking {
            assert_eq!(crate::tiers::deadline(&d), expected, "walk retry");
        } else {
            assert_eq!(row(&d).flight.as_ref().unwrap().next, expected, "flight poll");
            let mut out = Queue::uncapped();
            crate::tiers::hinted(
                &d,
                &api::Hint { repository: REPO, change: api::Change::Commit(BASE), key: None },
                &mut out,
            );
            assert!(
                collect(out).contains(&Request::WakeProcedure { task: TIER_TASK }),
                "job delivery wakes immediately"
            );
        }
    }
}

#[test]
fn waiting_tier_task_wakes_at_its_wall_deadline() {
    for until in [Some(Wall::from_nanos(env().wall.as_nanos() + Duration::from_secs(2).as_nanos())), None] {
        let mut d = configured(None);
        baseline(&mut d);
        attach(&mut d, 1, TIER_TASK);
        let deadline = until.unwrap_or(Wall::from_nanos(env().wall.as_nanos() + LIMITS.client.poll.as_nanos()));
        d.wait_tier(&env(), TIER_TASK, until);
        assert_eq!(d.next_wall_deadline(), Some(deadline));
        let mut clock = env();
        clock.wall = Wall::from_nanos(deadline.as_nanos() - 1);
        let mut out = Queue::uncapped();
        fire(&mut d, &clock, &mut out);
        assert!(!collect(out).contains(&Request::WakeProcedure { task: TIER_TASK }));
        assert_eq!(d.next_wall_deadline(), Some(deadline));

        clock.wall = deadline;
        let mut out = Queue::uncapped();
        fire(&mut d, &clock, &mut out);
        assert_eq!(
            collect(out).iter().filter(|request| **request == Request::WakeProcedure { task: TIER_TASK }).count(),
            1
        );
        assert_eq!(d.next_wall_deadline(), None);
        let mut out = Queue::uncapped();
        fire(&mut d, &clock, &mut out);
        assert!(!collect(out).contains(&Request::WakeProcedure { task: TIER_TASK }));
    }
}

#[test]
fn offered_tier_spends_only_its_own_wait() {
    let mut d = configured(None);
    baseline(&mut d);
    attach(&mut d, 1, TIER_TASK);
    assert_eq!(d.tier_ops.due.capacity(), LIMITS.repositories * LIMITS.tiers.tiers);
    d.wait_tier(&env(), TIER_TASK, Some(env().wall));
    d.wait_landing(&env(), NEXT_TASK, None);
    d.offered_tier(TIER_TASK);
    let poll = Wall::from_nanos(env().wall.as_nanos() + LIMITS.client.poll.as_nanos());
    assert_eq!(d.next_wall_deadline(), Some(poll));
    d.offered_landing(NEXT_TASK);
    assert_eq!(d.next_wall_deadline(), None);
    let mut clock = env();
    clock.wall = poll;
    let mut out = Queue::uncapped();
    fire(&mut d, &clock, &mut out);
    assert!(!collect(out).contains(&Request::WakeProcedure { task: TIER_TASK }));
}

fn close_tier(d: &mut Domain, task: u64, ending: ReleaseEnding) -> Box<[Request]> {
    outputs(d, Event::SettleEffects { task, root: task, ending })
}

#[test]
fn closing_a_tier_task_saves_its_drop_in_the_same_decision() {
    for ending in [ReleaseEnding::Cancelled, ReleaseEnding::Failed, ReleaseEnding::Done] {
        let mut d = dispatched();
        d.wait_tier(&env(), TIER_TASK, None);
        let out = close_tier(&mut d, TIER_TASK, ending);
        assert!(out.iter().any(|request| matches!(request,
            Request::Save { record: Stored::Tier(saved), .. }
                if saved.task.is_none() && saved.record.run.is_none() && saved.record.sequence == 1)));
        assert_eq!(row(&d).record.sequence, 1);
        assert!(row(&d).record.run.is_none());
        assert!(row(&d).task.is_none());
        assert!(row(&d).flight.is_none());
        assert!(row(&d).observations.is_empty());
        assert!(row(&d).held.is_none());
        assert_eq!(row(&d).branch, TierBranch::Absent);
        assert!(!d.tier_ops.follows.contains_key(&(REPO, TIER.into())));
        assert_eq!(d.next_wall_deadline(), None);
        assert!(choice(&out).is_none());
        let out = key_trees(&mut d);
        assert_eq!(requests(&out), [(2, BASE)]);
    }
}

#[test]
fn a_dropped_runs_landings_still_verify_in_the_next_numbered_run() {
    let mut d = dispatched();
    seen(&mut d, Some(BASE), NEXT_TIP);
    seen(&mut d, Some(NEXT_TIP), LAST_TIP);
    key_trees(&mut d);
    completed(&mut d, api::Conclusion::Success, None);
    finish(&mut d);
    attach(&mut d, 2, NEXT_TASK);
    let landings = row(&d).record.run.as_ref().unwrap().landings.clone();
    assert_eq!(landings.len(), 2);
    for landing in &landings {
        outputs(
            &mut d,
            Event::Restore {
                record: Stored::Verification(Verification {
                    repository: REPO,
                    commit: landing.commit,
                    changes: Box::new([CHANGE]),
                    tiers: Box::new([TierVerification { tier: TIER.into(), verdict: None, repaired: None }]),
                }),
            },
        );
    }
    let out = close_tier(&mut d, NEXT_TASK, ReleaseEnding::Cancelled);
    assert_eq!(row(&d).record.sequence, 2);
    assert_eq!(requests(&out), [(3, LAST_TIP)]);
    let run = row(&d).record.run.as_ref().unwrap();
    assert_eq!(run.landings, landings);
    assert!(d.verifications.iter().all(|(_, verification)| verification.tiers[0].verdict.is_none()));
    attach(&mut d, 3, TIER_TASK + 2);
    tier_step(&mut d, OPEN, false);
    assert_eq!(
        d.entries.get(&OPEN).unwrap().entry.effect.write,
        api::Write::CreateBranch { branch: b"temper/tier/browser.3".as_slice().into(), commit: LAST_TIP }
    );
}

#[test]
fn dropping_a_full_range_compacts_only_adjacent_outside_landings() {
    let mut d = dispatched();
    let owned = [12; 32];
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Verification(Verification {
                repository: REPO,
                commit: owned,
                changes: Box::new([CHANGE]),
                tiers: Box::new([TierVerification { tier: TIER.into(), verdict: None, repaired: None }]),
            }),
        },
    );
    let kept = d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
    kept.record.judged = Some(p::Judged { tip: BASE, key: BASE });
    kept.record.run.as_mut().unwrap().landings = Box::new([
        p::Landing { commit: [10; 32], key: [20; 32] },
        p::Landing { commit: [11; 32], key: [21; 32] },
        p::Landing { commit: owned, key: [22; 32] },
    ]);
    kept.pending = Box::new([
        TierLanding { commit: [13; 32], key: Some([23; 32]) },
        TierLanding { commit: [14; 32], key: Some([24; 32]) },
        TierLanding { commit: [15; 32], key: Some([25; 32]) },
    ]);
    let mut small = env();
    small.limits.changes = 1;
    let mut out = Queue::uncapped();
    step(
        &mut d,
        &small,
        Event::SettleEffects { task: TIER_TASK, root: TIER_TASK, ending: ReleaseEnding::Cancelled },
        &mut out,
    );
    let out = collect(out);
    let expected = [
        p::Landing { commit: [11; 32], key: [21; 32] },
        p::Landing { commit: owned, key: [22; 32] },
        p::Landing { commit: [15; 32], key: [25; 32] },
    ];
    assert_eq!(row(&d).record.run.as_ref().unwrap().landings.as_ref(), &expected);
    assert!(out.iter().all(|request| match request {
        Request::Save { record: Stored::Tier(row), .. } => row.pending.len() <= SMALL_RANGE,
        _ => true,
    }));
}

#[test]
fn late_dispatch_and_read_answers_cannot_revive_a_closed_tier_task() {
    let mut d = dispatched();
    // Keep one dispatch and its listing read in flight at the close.
    tier_step(&mut d, DISPATCH + 1, false);
    let key = (REPO, TIER.into());
    let owner = *d.tier_ops.reads.iter().next().expect("listing read").0;
    d.tiers.get_mut(&key).unwrap().flight.as_mut().unwrap().waiting = true;
    close_tier(&mut d, TIER_TASK, ReleaseEnding::Cancelled);
    assert!(!d.tier_ops.reads.contains_key(&owner), "close forgets the old run's read");
    key_trees(&mut d);
    attach(&mut d, 2, NEXT_TASK);
    let before = row(&d).clone();
    let mut late = Queue::uncapped();
    crate::tiers::outcome(
        &mut d,
        DISPATCH,
        skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Set, found: false },
        &mut late,
    );
    crate::tiers::veto(&mut d, TIER_TASK, DISPATCH, &mut late);
    crate::tiers::answered(&mut d, &env(), owner, Ok(api::Answer::Runs { runs: Box::new([]), more: false }), &mut late);
    assert!(collect(late).is_empty());
    assert_eq!(row(&d), &before);
}

#[test]
fn a_late_baseline_read_cannot_revive_a_closed_tier_task() {
    let mut d = configured(None);
    let (owner, read) = d.tier_ops.reads.iter().next().expect("tier-owned fresh baseline read");
    assert_eq!(read.tier, (REPO, TIER.into()));
    assert!(read.baseline);
    let owner = *owner;
    // A run was restored while the fresh baseline read was in flight.
    outputs(&mut d, Event::Restore { record: Stored::Tier(row(&dispatched()).clone()) });
    close_tier(&mut d, TIER_TASK, ReleaseEnding::Cancelled);
    assert!(!d.tier_ops.reads.contains_key(&owner), "close forgets the old baseline read");
    assert!(row(&d).record.run.is_none());
    assert!(row(&d).task.is_none());
    let before = row(&d).clone();
    let mut late = Queue::uncapped();
    crate::tiers::answered(&mut d, &env(), owner, Ok(api::Answer::Commit(NEXT_TIP)), &mut late);
    assert!(collect(late).is_empty(), "a late baseline answer cannot dispatch");
    assert_eq!(row(&d), &before);
}

#[test]
fn cancelling_a_tier_task_drops_its_wait() {
    let mut d = dispatched();
    d.wait_tier(&env(), TIER_TASK, None);
    let out = close_tier(&mut d, TIER_TASK, ReleaseEnding::Cancelled);
    assert!(choice(&out).is_none());
    assert!(row(&d).task.is_none());
    assert_eq!(d.next_wall_deadline(), None);
}

#[test]
fn restart_spends_tier_waits_and_wakes_their_tasks() {
    let saved = row(&dispatched()).clone();
    let mut d = restoring(None);
    outputs(&mut d, Event::Restore { record: Stored::Tier(saved) });
    d.wait_tier(&env(), TIER_TASK, None);
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let out = outputs(&mut d, Event::ReadAfresh);
    assert!(out.contains(&Request::WakeProcedure { task: TIER_TASK }));
    assert_eq!(d.next_wall_deadline(), None);
}

mod reports;

#[test]
fn pending_execution_words_survive_restart_until_the_task_keeps_them() {
    let mut d = dispatched();
    let mut run = workflow(&d, api::Conclusion::Success, None);
    run.state = api::RunState::Running;
    run.conclusion = None;
    answer(&mut d, api::Answer::Runs { runs: Box::new([run]), more: false });
    let saved = row(&d).clone();
    assert!(!saved.observations.is_empty());
    let mut cold = restoring(None);
    outputs(&mut cold, Event::Restore { record: Stored::Tier(saved.clone()) });
    outputs(&mut cold, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    assert_eq!(row(&cold).observations, saved.observations);
    outputs(&mut cold, Event::TierWords { task: TIER_TASK, words: b"Tier browser run 1".as_slice().into() });
    assert_eq!(row(&cold).observations, saved.observations);
    let words = cold.tier_execution_words(TIER_TASK).expect("restored execution words");
    assert!(skein_lib::bytes::find(&words, b"forge run 1 https://forge.example/run/71").is_some());
    let out = outputs(&mut cold, Event::TierWords { task: TIER_TASK, words });
    assert!(row(&cold).observations.is_empty());
    assert!(out.iter().any(|request| match request {
        Request::Save { record: Stored::Tier(row), .. } => row.observations.is_empty(),
        _ => false,
    }));
}

#[test]
fn tier_definitions_pin_landing_and_explicit_branch_bytes() {
    let d = configured(None);
    let tier = crate::tiers::configured_all(&d, REPO)[0].clone();
    assert_eq!(crate::tiers::definition(&tier).as_ref(), b"\x00\x08tier.yml\x00\x01\x00\x04docs\x00\x06report\x01\x00");
    let tier = Tier { follows: Follows::Branch(b"release/next".as_slice().into()), ..tier };
    assert_eq!(
        crate::tiers::definition(&tier).as_ref(),
        b"\x00\x08tier.yml\x00\x01\x00\x04docs\x00\x06report\x01\x01\x00\x0crelease/next"
    );
}

#[test]
fn goal_tiers_start_no_run_without_a_held_goal_branch() {
    let mut d = restoring_following(Follows::Goals);
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let out = outputs(&mut d, Event::ReadAfresh);
    assert!(requests(&out).is_empty());
    assert!(crate::tiers::watched(&d).is_empty());
    for branch in [b"main".as_slice(), b"temper/1/g1"] {
        let mut out = Queue::uncapped();
        crate::tiers::observed(&mut d, &env(), REPO, branch, None, BASE, &mut out);
        crate::tiers::observed(&mut d, &env(), REPO, branch, Some(BASE), HEAD, &mut out);
        crate::tiers::start_due(&mut d, &env(), REPO, &mut out);
        assert!(requests(&collect(out)).is_empty());
        assert!(d.tiers.is_empty());
        assert!(d.tier_ops.reads.is_empty());
    }
}

fn definition_key(tier: &Tier) -> keys::Keyed {
    keys::key(
        keys::Repository { forge: REPO.forge, repository: REPO.repository },
        &tier.name,
        &crate::tiers::definition(tier),
        &tier.ignored,
        BASE,
        &[keys::Tree { object: BASE, entries: Box::new([]) }],
        None,
        &keys::KeyLimits { entries: LIMITS.key_entries, path_bytes: LIMITS.key_path_bytes },
    )
}
#[test]
fn tier_keys_distinguish_follows_and_ignore_the_span() {
    let d = configured(None);
    let tier = crate::tiers::configured_all(&d, REPO)[0].clone();
    let landing = definition_key(&tier);
    assert_eq!(
        landing,
        keys::Keyed::Key([
            65, 183, 146, 65, 162, 229, 1, 83, 240, 199, 229, 238, 167, 53, 146, 217, 234, 128, 179, 203, 64, 103, 79,
            205, 187, 211, 89, 224, 4, 58, 81, 50
        ])
    );
    let branch = Tier { follows: Follows::Branch(b"release/next".as_slice().into()), ..tier.clone() };
    let branch_key = definition_key(&branch);
    assert_eq!(
        branch_key,
        keys::Keyed::Key([
            148, 25, 25, 98, 233, 154, 23, 238, 201, 141, 4, 166, 70, 20, 6, 128, 94, 166, 198, 205, 251, 183, 59, 228,
            6, 186, 224, 54, 40, 160, 176, 103
        ])
    );
    let goals = Tier { follows: Follows::Goals, ..tier.clone() };
    assert_eq!(
        crate::tiers::definition(&goals).as_ref(),
        b"\x00\x08tier.yml\x00\x01\x00\x04docs\x00\x06report\x01\x02"
    );
    let goals_key = definition_key(&goals);
    let keys::Keyed::Key(_) = goals_key else { panic!("complete goal definition key") };
    assert_ne!(goals_key, landing);
    assert_ne!(goals_key, branch_key);
    for tier in [tier, branch, goals] {
        let definition = crate::tiers::definition(&tier);
        let key = definition_key(&tier);
        let spaced = Tier { span: Some(Duration::from_millis(1_200_000)), ..tier };
        assert_eq!(crate::tiers::definition(&spaced), definition);
        assert_eq!(definition_key(&spaced), key);
    }
}

#[test]
fn a_head_sweep_keeps_a_tiers_live_run_and_retained_probe() {
    let mut d = configured(None);
    seen(&mut d, None, BASE);
    baseline_answer(&mut d, Ok(api::Answer::Commit(BASE)));
    key_trees(&mut d);
    let row = d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
    assert_eq!(row.record.run.as_ref().unwrap().number, 1);
    row.branch = TierBranch::Made;
    row.probe = Some(TierProbeBranch { landing: 0, branch: TierBranch::Made });
    let run = crate::tiers::name(REPO, b"temper/tier/browser.1");
    let probe = crate::tiers::name(REPO, b"temper/tier/browser.1.0");
    for name in [&run, &probe] {
        d.heads.insert(name.clone(), BranchHead { name: name.clone(), commit: HEAD, owned: true }).unwrap();
    }
    let mut out = Queue::uncapped();
    domain::heads::free(&mut d, &env(), &mut out);
    assert!(d.heads.contains_key(&run));
    assert!(d.heads.contains_key(&probe));
    assert!(out.is_empty());
    let row = d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
    row.record.run = None;
    row.branch = TierBranch::Absent;
    row.probe = None;
    domain::heads::free(&mut d, &env(), &mut out);
    assert!(!d.heads.contains_key(&run));
    assert!(!d.heads.contains_key(&probe));
    assert_eq!(out.len(), 2);
}
