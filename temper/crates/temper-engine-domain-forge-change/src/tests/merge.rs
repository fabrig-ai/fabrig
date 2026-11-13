//! Merge jobs and catch-ups (temper/docs/design/domain/forge/change/repairs.md).
use super::*;

fn catch_up(requested: bool) -> (Change, Facts) {
    let mut c = change(State::Producing { requested });
    c.last_head = None;
    let mut f = facts();
    f.branch = None;
    f.pull = Pull::Missing;
    f.catch_up = Some(BASE);
    f.contains_catch_up = Status::Failed;
    f.merge_job = true;
    (c, f)
}

/// A conflict uses the job where available under either rule (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_conflict_with_a_merge_job_goes_to_the_job_before_a_resolution() {
    for updates in [true, false] {
        for merge_job in [true, false] {
            for state in [
                State::First { head: HEAD, ready_since: at(0) },
                State::Updating { from: HEAD, base: BASE, ready_since: at(0) },
            ] {
                let mut c = change(state);
                c.owns_turn = true;
                c.clean = Box::new([MERGE]);
                c.resolutions = 1;
                let mut f = facts();
                f.mergeable = Status::Failed;
                f.merge_job = merge_job;
                f.updates = updates;
                let h = Heard { effect: EffectResult::Conflict, ..heard() };
                let next = decide(&c, &f, &h);
                let delegate = if merge_job {
                    Delegate::Merge { base: BASE, from: Some(HEAD) }
                } else {
                    Delegate::Resolve { base: BASE }
                };
                assert_eq!(next.decision, Decision::Delegate(delegate));
                assert_eq!(next.change.state, State::Resolving { requested: true, base: BASE });
                assert_eq!(next.change.resolutions, 2);
                assert_eq!(next.change.since, at(1));
                assert!(!next.change.owns_turn);
                assert!(next.change.clean.is_empty());
            }
        }
    }
}

/// A refused merge resolves against the last read base (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_refused_merge_goes_to_the_merge_job_at_once() {
    for updates in [true, false] {
        for merge_job in [true, false] {
            let mut c = change(State::Landing { head: HEAD });
            c.owns_turn = true;
            let mut f = facts();
            f.updates = updates;
            f.merge_job = merge_job;
            f.base_tip = NEW;
            let h = Heard { effect: EffectResult::Conflict, ..heard() };
            let next = decide(&c, &f, &h);
            let delegate = if merge_job {
                Delegate::Merge { base: NEW, from: Some(HEAD) }
            } else {
                Delegate::Resolve { base: NEW }
            };
            assert_eq!(next.decision, Decision::Delegate(delegate));
            assert_eq!(next.change.state, State::Resolving { requested: true, base: NEW });
            assert_eq!(next.change.resolutions, 1);
            assert!(!next.change.owns_turn);
        }
    }
}

/// A failed merge holds for a person (temper/docs/design/domain/forge/README.md, 4).
#[test]
fn a_failed_merge_effect_holds_until_released() {
    let mut c = change(State::Landing { head: HEAD });
    c.ready_since = Some(at(0));
    c.owns_turn = true;
    let f = Facts { merge_job: true, ..facts() };
    let h = Heard { effect: EffectResult::Failed, ..heard() };
    let next = decide(&c, &f, &h);
    assert_eq!(next.change.state, State::Held { was: Box::new(c.state), why: Hold::Failed });
    assert_eq!(next.decision, Decision::Hold(Hold::Failed));
    assert_eq!(next.change.resolutions, 0);
    assert!(!next.change.owns_turn);
    assert_eq!(decide(&next.change, &f, &heard()).decision, Decision::None);
}

/// A pushed merge starts checking with no clean history (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_merge_job_that_pushed_moves_the_change_on_from_its_merge() {
    let mut c = change(State::Resolving { requested: true, base: BASE });
    c.resolutions = 1;
    c.clean = Box::new([HEAD]);
    c.owns_turn = true;
    let f = Facts { branch: Some(NEW), delegate: Some(Delegate::Merge { base: BASE, from: Some(HEAD) }), ..facts() };
    let h = Heard { delegate: Status::Passed, ..heard() };
    let next = decide(&c, &f, &h);
    assert_eq!(next.decision, Decision::None);
    assert_eq!(next.change.state, State::Checking { head: NEW });
    assert_eq!(next.change.last_head, Some(NEW));
    assert_eq!(next.change.since, at(1));
    assert_eq!(next.change.resolutions, 1);
    assert!(next.change.clean.is_empty());
    assert!(!next.change.owns_turn);
}

/// Job then resolution counts the conflict once (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_merge_job_that_left_conflicts_makes_a_resolution_of_the_same_base() {
    let mut c = change(State::Updating { from: HEAD, base: BASE, ready_since: at(0) });
    c.resolutions = LIMITS.resolutions - 1;
    let mut f = Facts { merge_job: true, ..facts() };
    let conflict = Heard { effect: EffectResult::Conflict, ..heard() };
    let job = decide(&c, &f, &conflict);
    assert_eq!(job.decision, Decision::Delegate(Delegate::Merge { base: BASE, from: Some(HEAD) }));
    f.delegate = Some(Delegate::Merge { base: BASE, from: Some(HEAD) });
    f.base_tip = MERGE;
    let passed = Heard { delegate: Status::Passed, ..heard() };
    let resolve = step(&job.change, &f, &passed, at(2), &LIMITS);
    assert_eq!(resolve.decision, Decision::Delegate(Delegate::Resolve { base: BASE }));
    assert_eq!(resolve.change.state, State::Resolving { requested: true, base: BASE });
    assert_eq!(resolve.change.resolutions, LIMITS.resolutions);
    assert_eq!(resolve.change.since, at(2));
    f.delegate = Some(Delegate::Resolve { base: BASE });
    let waiting = decide(&resolve.change, &f, &heard());
    assert_eq!(waiting.decision, Decision::Wait { until: Some(at(32)) });
    f.branch = Some(NEW);
    let pushed = decide(&resolve.change, &f, &passed);
    assert_eq!(pushed.change.state, State::Checking { head: NEW });
    assert_eq!(pushed.change.resolutions, LIMITS.resolutions);
}

/// A job's terminal failure holds as a resolution's does (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_failed_merge_job_holds_the_resolution() {
    let mut c = change(State::Resolving { requested: true, base: BASE });
    c.resolutions = 1;
    let f = Facts { delegate: Some(Delegate::Merge { base: BASE, from: Some(HEAD) }), ..facts() };
    let h = Heard { delegate: Status::Failed, ..heard() };
    let next = decide(&c, &f, &h);
    assert_eq!(next.decision, Decision::Hold(Hold::Failed));
    assert_eq!(next.change.state, State::Held { was: Box::new(c.state), why: Hold::Failed });
    assert_eq!(next.change.resolutions, 1);
}

/// Unknown results and missing branch reads wait (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_merge_job_waits_for_its_result_and_a_branch_read() {
    let c = change(State::Resolving { requested: true, base: BASE });
    let mut f = Facts { delegate: Some(Delegate::Merge { base: BASE, from: Some(HEAD) }), ..facts() };
    for status in [Status::Unknown, Status::Pending] {
        let h = Heard { delegate: status, ..heard() };
        let next = decide(&c, &f, &h);
        assert_eq!(next.change, c);
        assert_eq!(next.decision, Decision::Wait { until: Some(at(30)) });
    }
    f.branch = None;
    let h = Heard { delegate: Status::Passed, ..heard() };
    assert_eq!(decide(&c, &f, &h).decision, Decision::Wait { until: Some(at(30)) });
}

/// Jobs obey the same conflict bound at every entrance (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_merge_job_past_the_resolution_bound_holds() {
    for state in [
        State::First { head: HEAD, ready_since: at(0) },
        State::Updating { from: HEAD, base: BASE, ready_since: at(0) },
        State::Landing { head: HEAD },
    ] {
        let mut c = change(state);
        c.resolutions = LIMITS.resolutions;
        let f = Facts { merge_job: true, mergeable: Status::Failed, ..facts() };
        let h = Heard { effect: EffectResult::Conflict, ..heard() };
        let next = decide(&c, &f, &h);
        assert_eq!(next.decision, Decision::Hold(Hold::Resolutions));
        assert_eq!(next.change.resolutions, LIMITS.resolutions);
    }
}

/// Asking a resolution again does not ask a job (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_resolution_asked_again_is_not_a_merge_job() {
    let c = change(State::Resolving { requested: false, base: BASE });
    let f = Facts { merge_job: true, delegate: Some(Delegate::Merge { base: BASE, from: Some(HEAD) }), ..facts() };
    let h = Heard { delegate: Status::Passed, ..heard() };
    let next = decide(&c, &f, &h);
    assert_eq!(next.decision, Decision::Delegate(Delegate::Resolve { base: BASE }));
    assert_eq!(next.change.state, State::Resolving { requested: true, base: BASE });
    assert_eq!(next.change.resolutions, 0);
}

/// A turn's prepare conflict is its queue's business (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_turn_prepare_conflict_still_goes_to_a_resolution() {
    let mut c = change(State::Preparing { from: HEAD, main: BASE, ready_since: at(0) });
    c.way = crate::Way::Turn;
    let f = Facts { merge_job: true, ..facts() };
    let h = Heard { effect: EffectResult::Conflict, ..heard() };
    let next = decide(&c, &f, &h);
    assert_eq!(next.decision, Decision::Delegate(Delegate::Resolve { base: BASE }));
    assert_eq!(next.change.resolutions, 1);
}

/// Containment ends a catch-up with its commit in words (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_catch_up_already_contained_ends_with_no_change() {
    let sha1 = [
        0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd,
        0xef, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    for (commit, words) in [
        (sha1, b"The landing branch already contains commit abcdef0123456789abcdef0123456789abcdef01.".as_slice()),
        ([0xab; 32], b"The landing branch already contains commit abababababababababababababababababababababababababababababababab.".as_slice()),
    ] {
        let (c, mut f) = catch_up(false);
        f.catch_up = Some(commit);
        f.contains_catch_up = Status::Passed;
        let next = decide(&c, &f, &heard());
        assert_eq!(next.decision, Decision::NoChange);
        assert_eq!(next.change.state, State::NoChange { words: words.into() });
        assert_eq!(next.change.resolutions, 0);
        assert_eq!(decide(&next.change, &f, &heard()).decision, Decision::NoChange);
    }
}

/// A contained pushed head ends only when no work or hold stands (temper/docs/design/domain/forge/change/README.md, 2).
#[test]
#[expect(clippy::large_stack_arrays, reason = "the phase table carries facts with bounded Copy repair context")]
fn a_pushed_head_already_contained_ends_with_no_change() {
    let mut sha1 = [0; 32];
    sha1.get_mut(..20).expect("SHA-1 width").fill(0xab);
    for (head, words) in [
        (sha1, b"The landing branch already contains its head abababababababababababababababababababab.".as_slice()),
        ([0xab; 32], b"The landing branch already contains its head abababababababababababababababababababababababababababababababab.".as_slice()),
    ] {
        let opening = State::Opening { requested: false };
        let f = Facts { branch: Some(head), contains_head: Status::Passed, ..facts() };
        for (state, facts, heard, ends) in [
            (opening.clone(), f.clone(), heard(), true),
            (State::Checking { head }, f.clone(), heard(), true),
            (State::Gating { head, asked: Box::new([]) }, f.clone(), heard(), true),
            (State::Queued { head, ready_since: at(0) }, f.clone(), heard(), true),
            (State::First { head, ready_since: at(0) }, f.clone(), heard(), true),
            (opening.clone(), Facts { contains_head: Status::Failed, ..f.clone() }, heard(), false),
            (opening.clone(), Facts { contains_head: Status::Unknown, ..f.clone() }, heard(), false),
            (opening.clone(), Facts { branch: None, ..f.clone() }, heard(), false),
            (State::Landing { head }, f.clone(), heard(), false),
            (State::Updating { from: head, base: BASE, ready_since: at(0) }, f.clone(), heard(), false),
            (State::Preparing { from: head, main: BASE, ready_since: at(0) }, f.clone(), heard(), false),
            (opening.clone(), f.clone(), Heard { delegate_live: true, ..heard() }, false),
            (opening.clone(), Facts { writer_taken: true, ..f.clone() }, heard(), false),
            (opening.clone(), Facts { drift: Some(Hold::BranchMoved), ..f.clone() }, heard(), false),
            (State::Held { was: Box::new(opening.clone()), why: Hold::Failed }, f.clone(), heard(), false),
            (opening.clone(), Facts { pull: Pull::Merged { commit: MERGE }, ..f.clone() }, heard(), false),
            (opening, f.clone(), Heard { cancelled: true, ..heard() }, false),
        ] {
            let mut c = change(state);
            c.owns_turn = true;
            let next = decide(&c, &facts, &heard);
            if ends {
                assert_eq!(next.decision, Decision::NoChange, "{:?}", c.state);
                assert_eq!(next.change.state, State::NoChange { words: words.into() });
                assert!(!next.change.owns_turn);
            } else {
                let baseline = decide(&c, &Facts { contains_head: Status::Unknown, ..facts }, &heard);
                assert_eq!(next, baseline, "{:?}", c.state);
                assert_ne!(next.decision, Decision::NoChange);
            }
        }
    }
}

/// A catch-up asks a merge job without counting a conflict (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_catch_ups_producer_is_the_merge_job() {
    let (mut c, f) = catch_up(false);
    c.resolutions = LIMITS.resolutions;
    let next = decide(&c, &f, &heard());
    assert_eq!(next.decision, Decision::Delegate(Delegate::Merge { base: BASE, from: None }));
    assert_eq!(next.change.state, State::Producing { requested: true });
    assert_eq!(next.change.resolutions, LIMITS.resolutions);
    let f = Facts { delegate: Some(Delegate::Merge { base: BASE, from: None }), ..f };
    assert_eq!(decide(&next.change, &f, &heard()).decision, Decision::Wait { until: Some(at(30)) });
}

/// A catch-up waits for its containment read (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_catch_up_waits_until_containment_is_known() {
    let (c, mut f) = catch_up(false);
    for status in [Status::Unknown, Status::Pending] {
        f.contains_catch_up = status;
        let next = decide(&c, &f, &heard());
        assert_eq!(next.change, c);
        assert_eq!(next.decision, Decision::Wait { until: Some(at(30)) });
    }
}

/// A catch-up's pushed branch goes to opening (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_catch_up_opens_the_branch_its_job_or_resolution_pushed() {
    for delegate in [Delegate::Merge { base: BASE, from: None }, Delegate::Resolve { base: BASE }] {
        let (c, mut f) = catch_up(true);
        f.delegate = Some(delegate);
        f.branch = Some(NEW);
        let h = Heard { delegate: Status::Passed, ..heard() };
        let next = decide(&c, &f, &h);
        assert_eq!(next.decision, Decision::None);
        assert_eq!(next.change.state, State::Opening { requested: false });
        assert_eq!(next.change.last_head, Some(NEW));
        assert_eq!(next.change.since, at(1));
    }
}

/// A catch-up's unresolved job asks one counted resolution (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_catch_up_job_that_left_conflicts_makes_one_resolution() {
    let (c, mut f) = catch_up(true);
    f.delegate = Some(Delegate::Merge { base: BASE, from: None });
    f.base_tip = MERGE;
    let h = Heard { delegate: Status::Passed, ..heard() };
    let next = decide(&c, &f, &h);
    assert_eq!(next.decision, Decision::Delegate(Delegate::Resolve { base: BASE }));
    assert_eq!(next.change.state, State::Producing { requested: true });
    assert_eq!(next.change.resolutions, 1);
    assert_eq!(next.change.since, at(1));
    f.delegate = Some(Delegate::Resolve { base: BASE });
    let waiting = decide(&next.change, &f, &heard());
    assert_eq!(waiting.change.resolutions, 1);
    assert_eq!(waiting.decision, Decision::Wait { until: Some(at(31)) });
    let no_push = decide(&next.change, &f, &h);
    assert_eq!(no_push.decision, Decision::Wait { until: Some(at(31)) });
    assert_eq!(no_push.change.resolutions, 1);
}

/// The catch-up's resolution is bounded too (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_catch_up_resolution_obeys_the_resolution_bound() {
    let (mut c, mut f) = catch_up(true);
    f.delegate = Some(Delegate::Merge { base: BASE, from: None });
    let h = Heard { delegate: Status::Passed, ..heard() };
    c.resolutions = LIMITS.resolutions - 1;
    let admitted = decide(&c, &f, &h);
    assert_eq!(admitted.decision, Decision::Delegate(Delegate::Resolve { base: BASE }));
    assert_eq!(admitted.change.resolutions, LIMITS.resolutions);
    c.resolutions = LIMITS.resolutions;
    let held = decide(&c, &f, &h);
    assert_eq!(held.decision, Decision::Hold(Hold::Resolutions));
    assert_eq!(held.change.state, State::Held { was: Box::new(c.state), why: Hold::Resolutions });
}

/// A failed catch-up job holds as a failed producer does (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_failed_catch_up_job_holds_the_producer() {
    let (c, mut f) = catch_up(true);
    f.delegate = Some(Delegate::Merge { base: BASE, from: None });
    let h = Heard { delegate: Status::Failed, ..heard() };
    let next = decide(&c, &f, &h);
    assert_eq!(next.decision, Decision::Hold(Hold::Failed));
    assert_eq!(next.change.state, State::Held { was: Box::new(c.state), why: Hold::Failed });
    assert_eq!(next.change.resolutions, 0);
}
