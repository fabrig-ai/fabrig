//! Removing or renaming a live tier (temper/docs/design/domain/after-landing-runs.md, 2).
use super::*;

const AFTER: &[u8] = b"after";
const END: &[u8] = b"Tier browser run 1: ended, its tier is no longer configured";

fn cold(saved: TierRow, tiers: Box<[Tier]>) -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    configure_tiers(&mut d, tiers);
    outputs(&mut d, Event::Restore { record: Stored::Tier(saved) });
    d
}

fn restart(d: &mut Domain) {
    let mut out = outputs(d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic }).into_vec();
    out.extend(outputs(d, Event::CatchUp));
    assert!(out.iter().any(|request| matches!(request, Request::WakeProcedure { task: TIER_TASK })));
    assert!(!d.tier_ops.follows.contains_key(&(REPO, TIER.into())));
    assert_quiet(&out);
}

fn assert_quiet(out: &[Request]) {
    for request in out {
        assert!(
            !matches!(
                request,
                Request::Call { op: api::Op::Write(_), .. }
                    | Request::Save { record: Stored::Entry { .. }, .. }
                    | Request::TierDecision { entry: Some(_), .. }
                    | Request::TierRun { .. }
                    | Request::News { .. }
                    | Request::FileRedFinding { .. }
                    | Request::FileVerdict { .. }
            ),
            "unexpected run activity: {request:?}"
        );
    }
}

fn assert_ended(d: &mut Domain, entry: u64, released: bool) {
    let out = tier_step(d, entry, released);
    assert_quiet(&out);
    assert_eq!(choice(&out), Some(&TierDecision::Done { words: END.into() }));
    let kept = row(d);
    assert!(kept.record.run.is_none());
    assert_eq!(kept.task, None);
    assert_eq!(kept.record.sequence, 1);
    assert!(d.entries.is_empty());
    assert!(
        out.iter().any(|request| matches!(request, Request::Save { record: Stored::Tier(saved), .. } if saved == kept))
    );
}

#[test]
fn a_removed_tier_restarts_without_a_follower_or_a_tree_walk() {
    let mut live = dispatched();
    seen(&mut live, Some(BASE), NEXT_TIP);
    let saved = row(&live).clone();
    assert_eq!(saved.pending.as_ref(), &[TierLanding { commit: NEXT_TIP, key: None }]);
    let mut d = cold(saved.clone(), Box::new([]));
    restart(&mut d);
    assert!(d.tier_ops.reads.is_empty());
    fresh_reads(&mut d, NEXT_TIP);
    assert_ended(&mut d, RELEASE, false);
    assert_eq!(row(&d).pending, saved.pending);
    assert_eq!(row(&d).record.judged, None);
}

#[test]
fn a_renamed_tier_reads_a_new_baseline_while_the_old_run_ends() {
    let mut after = browser_tier(Follows::Landing);
    after.name = AFTER.into();
    let mut d = cold(row(&dispatched()).clone(), Box::new([after]));
    restart(&mut d);
    let key = (REPO, AFTER.into());
    let after = d.tiers.get(&key).expect("new tier is retained");
    assert!(after.record.run.is_none());
    assert_eq!(after.baseline, None);
    assert!(d.tier_ops.reads.iter().any(|(_, read)| read.tier == key && read.baseline));
    // The repository watch can answer before the tier's own baseline read.
    for _ in 0_u8..2 {
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, api::Op::Read(api::Read::Branch { branch: b"main".as_slice().into() }));
        assert_quiet(&fixtures::answer(&mut d, call, Ok(api::Answer::Commit(NEXT_TIP))));
        if d.tiers.get(&key).unwrap().baseline.is_some() {
            break;
        }
    }
    assert_eq!(d.tiers.get(&key).unwrap().baseline, Some(TierLanding { commit: NEXT_TIP, key: None }));
    fresh_reads(&mut d, NEXT_TIP);
    assert_ended(&mut d, RELEASE, false);
}

#[test]
fn a_run_answer_after_removal_does_not_judge_the_run() {
    let mut d = dispatched();
    let run = workflow(&d, api::Conclusion::Success, None);
    answer(&mut d, api::Answer::Runs { runs: Box::new([run.clone()]), more: false });
    assert!(d.tier_ops.follows.contains_key(&(REPO, TIER.into())));
    assert_quiet(&tier_step(&mut d, RELEASE, false));
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, api::Op::Read(api::Read::Run { run: RUN }));

    configure_tiers(&mut d, Box::new([]));
    let out = fixtures::answer(&mut d, call, Ok(api::Answer::Run(run)));
    assert_quiet(&out);
    assert!(row(&d).result.is_none());
    assert!(d.landing.receipts.is_empty());
    assert_ended(&mut d, RELEASE + 1, false);
}

#[test]
fn releasing_a_held_run_after_removal_ends_it_without_dispatching_again() {
    let mut d = dispatched();
    d.tiers.get_mut(&(REPO, TIER.into())).unwrap().record.run.as_mut().unwrap().faults = RETRIES;
    completed(&mut d, api::Conclusion::Cancelled, None);
    assert!(matches!(choice(&settled_step(&mut d, RELEASE, false)), Some(TierDecision::Held { .. })));
    hold::act(&mut d, TierAct::Hold { reason: b"maintenance".as_slice().into() });
    let held = row(&d).hold.clone();

    configure_tiers(&mut d, Box::new([]));
    assert_ended(&mut d, RELEASE + 1, true);
    assert!(row(&d).held.is_none());
    assert_eq!(row(&d).hold, held);
}

#[test]
fn a_removed_tier_waits_for_its_saved_dispatch_to_settle() {
    let mut live = dispatching();
    let (repository, what, described) = live.describe_tier_effect(TIER_TASK, DISPATCH).unwrap();
    let entry = live.entries.remove(&DISPATCH).unwrap().entry;
    let mut d = cold(row(&live).clone(), Box::new([]));
    outputs(&mut d, Event::Restore { record: Stored::Entry { entry, held: false } });
    restart(&mut d);
    let (got_repository, got_what, got) = d.describe_tier_effect(TIER_TASK, DISPATCH).expect("kept dispatch");
    assert_eq!((got_repository, got_what, got.kind, got.state), (repository, what, described.kind, described.state));
    assert_eq!(got.effect, described.effect);

    let out = tier_step(&mut d, RELEASE, false);
    assert_quiet(&out);
    assert_eq!(choice(&out), Some(&TierDecision::Wait { until: None }));
    outcome(&mut d, DISPATCH, skein_forge_client::Outcome::Failed(api::Error::Missing));
    assert!(!row(&d).flight.as_ref().unwrap().waiting);
    assert!(d.tier_ops.follows.is_empty());
    assert_ended(&mut d, RELEASE + 1, false);
}

#[test]
fn a_removed_tier_can_settle_its_issued_comment() {
    let mut d = post::ready(true);
    let status = RELEASE + 2;
    let comment = status + 1;
    tier_step(&mut d, status, false);
    made(&mut d, status, skein_forge_client::Made::Set);
    tier_step(&mut d, comment, false);
    let (_, what, before) = d.describe_tier_effect(TIER_TASK, comment).expect("issued verdict comment");
    assert!(matches!(before.effect.write, api::Write::Post { .. }));
    let judged = row(&d).record.judged;

    configure_tiers(&mut d, Box::new([]));
    let (_, got_what, after) = d.describe_tier_effect(TIER_TASK, comment).expect("kept verdict comment");
    assert_eq!(got_what, what);
    assert_eq!(after.effect, before.effect);
    let out = tier_step(&mut d, comment + 1, false);
    assert_quiet(&out);
    assert_eq!(choice(&out), Some(&TierDecision::Wait { until: None }));
    made(&mut d, comment, skein_forge_client::Made::Commented(1));
    assert_ended(&mut d, comment + 2, false);
    assert_eq!(row(&d).record.judged, judged, "the verdict was committed before removal");
    assert!(row(&d).posting.is_none());
}
