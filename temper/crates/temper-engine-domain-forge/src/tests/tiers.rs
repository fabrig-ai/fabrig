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
use temper_engine_domain_forge_tiers as p;

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
fn restoring(branch: Option<&[u8]>) -> Domain {
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
                branch: branch.map(Box::from),
            }])
        }]),
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
fn row(d: &Domain) -> &TierRow {
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
fn key_trees(d: &mut Domain) -> Box<[Request]> {
    let mut all = Vec::new();
    // A fixture has at most the baseline plus the range's tips to key.
    for _ in 0..LIMITS.changes * 2 + 2 {
        let Some((owner, _)) = d.tier_ops.reads.iter().next() else { break };
        let owner = *owner;
        let mut out = Queue::uncapped();
        let tip = row(d)
            .baseline
            .into_iter()
            .chain(row(d).pending.iter().copied())
            .find(|tip| tip.key.is_none())
            .expect("unkeyed tip")
            .commit;
        let source = if tip == HEAD { BASE } else { tip };
        let entries = Box::new([
            api::TreeEntry {
                name: b"src".as_slice().into(),
                mode: 0o100_644,
                kind: api::TreeEntryKind::Blob,
                object: source,
            },
            api::TreeEntry {
                name: b"docs".as_slice().into(),
                mode: 0o100_644,
                kind: api::TreeEntryKind::Blob,
                object: tip,
            },
        ]);
        crate::tiers::answered(d, &env(), owner, Ok(api::Answer::Tree { entries, more: false }), &mut out);
        all.extend(collect(out));
    }
    all.into_boxed_slice()
}
fn baseline(d: &mut Domain) -> Box<[Request]> {
    seen(d, None, BASE);
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
            heard: TierHeard { released, cancelled: false },
        },
    )
}
fn made(d: &mut Domain, entry: u64, made: skein_forge_client::Made) {
    if let Some(kept) = d.entries.get_mut(&entry) {
        kept.entry.start = Some(skein_forge_client::Position { at: env().now, comment: 0, review: 0, review_page: 1 });
    }
    let mut out = Queue::uncapped();
    crate::tiers::outcome(d, entry, skein_forge_client::Outcome::Made { made, found: false }, &mut out);
    d.entries.remove(&entry);
}
fn dispatched() -> Domain {
    let mut d = configured(None);
    baseline(&mut d);
    attach(&mut d, 1, TIER_TASK);
    tier_step(&mut d, OPEN, false);
    made(&mut d, OPEN, skein_forge_client::Made::Branch(BASE));
    tier_step(&mut d, DISPATCH, false);
    made(&mut d, DISPATCH, skein_forge_client::Made::Set);
    d
}
fn workflow(d: &Domain, conclusion: api::Conclusion, failed_step: Option<u32>) -> api::Run {
    api::Run {
        id: RUN,
        number: 1,
        workflow: b"tier.yml".as_slice().into(),
        commit: row(d).record.run.as_ref().unwrap().tip,
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
    d.tier_ops.reads.insert(READ, (REPO, TIER.into())).unwrap();
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
    tier_step(d, RELEASE + 1, false)
}
fn requests(out: &[Request]) -> Vec<(u64, api::Commit)> {
    out.iter()
        .filter_map(|request| match request {
            Request::TierRun { number, tip, .. } => Some((*number, *tip)),
            _ => None,
        })
        .collect()
}
fn choice(out: &[Request]) -> Option<&TierDecision> {
    out.iter().find_map(|request| match request {
        Request::TierDecision { decision, .. } => Some(decision),
        _ => None,
    })
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
    let out = tier_step(&mut d, RELEASE + 1, false);
    assert_eq!(
        choice(&out),
        Some(&TierDecision::Done { words: b"Tier browser run 1: green, cleared 0 landings".as_slice().into() })
    );
    assert!(row(&d).record.run.is_none());
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
        Event::StepTier { task: NEXT_TASK, entry: RELEASE + 2, heard: TierHeard { released: false, cancelled: false } },
    );
    assert_eq!(
        choice(&out),
        Some(&TierDecision::Done { words: b"Tier browser run 2: green, cleared 1 landings".as_slice().into() })
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
fn branch_no_tier_follows_starts_no_run() {
    let mut d = configured(Some(b"release/next"));
    let out = seen(&mut d, None, BASE);
    assert!(requests(&out).is_empty());
    assert!(d.tiers.is_empty());
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
        Some(&TierDecision::Done { words: b"Tier browser run 2: green, cleared 1 landings".as_slice().into() })
    );
    assert!(row(&d).pending.is_empty());
    assert!(row(&d).record.run.is_none());
    assert_eq!(d.landing.receipts.len(), 2);
}

fn observe_branch(d: &mut Domain, expected: &[u8]) {
    for _ in 0..LIMITS.client.pending {
        let (call, op) = fixtures::next_call(d);
        match op {
            api::Op::Read(api::Read::Items { .. }) => {
                fixtures::answer(d, call, Ok(api::Answer::Items { items: Box::new([]), more: false, now: env().now }));
            }
            api::Op::Read(api::Read::Branch { branch }) => {
                assert_eq!(branch.as_ref(), expected);
                fixtures::answer(d, call, Ok(api::Answer::Commit(BASE)));
                return;
            }
            _ => panic!("unexpected watch call: {op:?}"),
        }
    }
    panic!("tier branch was not watched");
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
