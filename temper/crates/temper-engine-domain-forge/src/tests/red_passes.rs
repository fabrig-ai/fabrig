//! Passes end reds only when a change lands (domain/forge/reds.md, 5.7).
use super::*;
use temper_engine_domain_forge_landing as landing;

fn carrying() -> Domain {
    let mut d = configured_domain(&[]);
    let name = tests::reds::name(7);
    d.holds
        .insert(name.clone(), Hold { name, task: 7, writer: None, design_writers: Box::new([]), drift: None })
        .unwrap();
    add_change(&mut d, CHANGE, b"temper/7/g7");
    d.record_red(&LIMITS, tests::reds::call(7, b"X"), &mut Queue::uncapped()).unwrap();
    d.observe_reds(CHANGE, 7, &[b"X".as_slice().into()], &[], &mut Queue::uncapped());
    d
}
fn assert_landing(d: &Domain, out: &Queue<Request>, stays: bool) {
    assert_eq!(d.reds(7).count(), usize::from(stays));
    assert_eq!(d.red_passes(CHANGE).count(), 0);
    assert_eq!(contains(out, &Request::Erase { key: Key::Red { goal: 7, key: b"X".as_slice().into() } }), !stays);
    assert!(contains(out, &Request::Erase { key: Key::RedPass { change: CHANGE, key: b"X".as_slice().into() } }));
}
fn rerecord(d: &mut Domain, again: bool) {
    if again {
        d.record_red(&LIMITS, tests::reds::call(7, b"X"), &mut Queue::uncapped()).unwrap();
    }
}
#[test]
fn a_merge_outcome_ends_passed_reds_but_not_a_later_generation_or_another_destination() {
    for again in [false, true] {
        for release in [false, true] {
            let mut d = carrying();
            rerecord(&mut d, again);
            if release {
                d.releases
                    .insert(CHANGE, release_row(CHANGE, CHANGE, ReleaseEnding::Cancelled, Some(1), true, false))
                    .unwrap();
            }
            let out = merged(&mut d, CHANGE, FIRST);
            assert_landing(&d, &out, again);
        }
    }
    let mut d = carrying();
    d.changes.get_mut(&CHANGE).unwrap().base = b"main".as_slice().into();
    let out = merged(&mut d, CHANGE, FIRST);
    assert_landing(&d, &out, true);
}
fn merged_pull() -> PendingStep {
    PendingStep {
        task: CHANGE,
        entry: OPEN,
        follow_tree: List::with_capacity(LIMITS.key_entries),
        stage: StepStage::Branch,
        heard: heard(false),
        gates: Box::new([]),
        queue_repair_active: false,
        kept: false,
        merge_job: false,
        pending: Box::new([]),
        branch: Some(HEAD),
        branch_fresh: false,
        base_tip: Some(FIRST),
        pull: Some(skein_forge::api::Pull {
            number: 1,
            head: b"temper/51".as_slice().into(),
            commit: HEAD,
            base_commit: Some(FIRST),
            base: b"temper/7/g7".as_slice().into(),
            state: skein_forge::api::State::Closed,
            merged: Some(FIRST),
            mergeable: true,
            reviewers: Box::new([]),
            ci: skein_forge::api::Ci::Passed,
            queued: None,
            dequeued: None,
            size: None,
        }),
        pull_ci: false,
        ci: change::Status::Passed,
        failed: change_ci::Failed::new(Box::new([])),
        base_ci: change::Status::Passed,
        contains_base: change::Status::Passed,
        contains_head: change::Status::Unknown,
        contains_catch_up: change::Status::Passed,
        reviews: Box::new([]),
        reviews_complete: true,
        paths: Box::new([]),
        paths_whole: false,
        woken: false,
    }
}
#[test]
fn a_pull_found_merged_ends_only_red_generations_the_change_passed() {
    for again in [false, true] {
        let mut d = carrying();
        rerecord(&mut d, again);
        let mut out = Queue::uncapped();
        decide_step(&mut d, &env(), merged_pull(), &mut out);
        assert!(landed_commit(&d.change(CHANGE).unwrap().change.state).is_some());
        assert_landing(&d, &out, again);
    }
}
#[test]
fn a_landing_read_back_ends_only_red_generations_the_change_passed() {
    for cancelled in [false, true] {
        for again in [false, true] {
            let mut d = carrying();
            rerecord(&mut d, again);
            d.changes.get_mut(&CHANGE).unwrap().change.way = change::Way::Turn;
            d.changes.get_mut(&CHANGE).unwrap().change.landing = Some(61);
            d.landing
                .rows
                .insert(
                    61,
                    LandingRow {
                        job: None,
                        landing: landing::Landing {
                            task: 61,
                            project: PROJECT,
                            phase: landing::Phase::Landed,
                            round: landing::Round::new(Box::default()),
                            last_left: None,
                            branch: b"temper/7/g7".as_slice().into(),
                            preparation: 0,
                            stale_proofs: 0,
                            members: Box::new([landing::Member {
                                round: None,
                                change: CHANGE,
                                repository: landing::Repository { forge: REPO.forge, repository: REPO.repository },
                                pull: 1,
                                head: Some(HEAD),
                                base: None,
                                candidate: Some(FIRST),
                                gates: Box::new([]),
                                asked: None,
                                landed: Some(FIRST),
                                left: None,
                                unit: 0,
                                branch: b"temper/51".as_slice().into(),
                                combined: None,
                                recorded: true,
                                told: false,
                                judging: false,
                                leave_told: false,
                            }]),
                        },
                        asks: Box::new([None]),
                        entries: Box::new([]),
                    },
                )
                .unwrap();
            let mut out = Queue::uncapped();
            if cancelled {
                crate::landing::closing(&mut d, &env(), 61, ReleaseEnding::Cancelled, &mut out);
            } else {
                step_verified(&mut d, &env(), CHANGE, FIRST, heard(false), &mut out);
            }
            assert!(landed_commit(&d.change(CHANGE).unwrap().change.state).is_some());
            assert_landing(&d, &out, again);
        }
    }
}
#[test]
fn ending_without_a_merge_and_dropping_erase_passes_without_ending_reds() {
    let mut d = carrying();
    let mut out = Queue::uncapped();
    end_change(&mut d, &env(), CHANGE, &mut out);
    assert_eq!(d.change(CHANGE).unwrap().change.state, change::State::Ended);
    assert_landing(&d, &out, true);
    // Restore a pass on a settled row to exercise the row's drop independently.
    d.red_passes
        .insert(
            (CHANGE, b"X".as_slice().into()),
            RedPass { change: CHANGE, key: b"X".as_slice().into(), generation: 1 },
        )
        .unwrap();
    let mut environment = env();
    environment.limits.changes = 1;
    let mut out = Queue::uncapped();
    step(&mut d, &environment, Event::DropChange { task: CHANGE }, &mut out);
    assert!(d.change(CHANGE).is_none());
    assert_landing(&d, &out, true);
}
