use super::*;

fn facts() -> Facts {
    Facts {
        goal_branch: true,
        close_job: true,
        pending: false,
        landed: Box::new([9, 12]),
        base_holds: Some(false),
        child: Child::Pending,
        landing_child: Child::Pending,
        released: false,
        close_ended: 1,
        land_ended: 1,
        bound: 1,
    }
}

#[test]
fn ended_steps_retry_only_below_the_bound_and_not_for_their_branch() {
    // kind, running, branch's reason, ended, pending, containment, decision
    for (kind, running, branch, ended, pending, contains, expected) in [
        (Step::Close, true, false, 2, false, Some(false), Decision::Make(Step::Close)),
        (Step::Close, true, false, 2, true, Some(false), Decision::Wait),
        (
            Step::Close,
            true,
            false,
            3,
            false,
            Some(false),
            Decision::Hold(Hold::ClosingStep { step: Step::Close, change: 20 }),
        ),
        (
            Step::Close,
            true,
            true,
            1,
            false,
            Some(false),
            Decision::Hold(Hold::ClosingStep { step: Step::Close, change: 20 }),
        ),
        (Step::Land, true, false, 2, false, None, Decision::Check),
        (Step::Land, true, false, 2, false, Some(false), Decision::Make(Step::Land)),
        (Step::Land, false, false, 2, false, Some(false), Decision::Make(Step::Land)),
        (Step::Land, true, false, 2, true, Some(false), Decision::Wait),
        (
            Step::Land,
            false,
            false,
            3,
            false,
            Some(false),
            Decision::Hold(Hold::ClosingStep { step: Step::Land, change: 20 }),
        ),
        (
            Step::Land,
            true,
            true,
            1,
            false,
            Some(false),
            Decision::Hold(Hold::ClosingStep { step: Step::Land, change: 20 }),
        ),
        (Step::Land, false, true, 1, false, Some(true), Decision::Report),
    ] {
        let mut f = facts();
        f.bound = 3;
        f.close_ended = if kind == Step::Close { ended } else { 9 };
        f.land_ended = if kind == Step::Land { ended } else { 9 };
        f.child = Child::Ended { branch };
        f.landing_child = f.child;
        f.pending = pending;
        f.base_holds = contains;
        let mut c = Closer::new();
        c.closed = Some(f.landed.clone());
        c.running = running.then_some(Running { step: kind, change: 20 });
        c.landing = (kind == Step::Land).then_some(20);
        let s = step(&c, &f);
        assert_eq!(s.decision, expected, "{kind:?}, running {running}, branch {branch}, ended {ended}");
        if s.decision == Decision::Wait && running {
            assert_eq!(s.closer.retry, Some(kind));
            f.pending = false;
            assert_eq!(step(&s.closer, &f).decision, Decision::Make(kind));
        }
        if let Decision::Hold(_) = s.decision {
            f.released = true;
            let released = step(&s.closer, &f);
            assert_eq!(released.decision, Decision::Make(kind));
            assert_eq!(released.closer.hold, None);
        }
    }
}

#[test]
fn close_first_where_the_project_names_a_close_job() {
    assert_eq!(step(&Closer::new(), &facts()).decision, Decision::Make(Step::Close));
}

#[test]
fn no_close_step_without_a_close_job_or_a_goal_branch() {
    let mut f = facts();
    f.close_job = false;
    assert_eq!(step(&Closer::new(), &f).decision, Decision::Make(Step::Land));
    f.close_job = true;
    f.goal_branch = false;
    assert_eq!(step(&Closer::new(), &f).decision, Decision::Report);
}

#[test]
fn land_after_close_unless_the_base_holds_the_branch() {
    let mut f = facts();
    let mut c = Closer::new();
    c.closed = Some(f.landed.clone());
    assert_eq!(step(&c, &f).decision, Decision::Make(Step::Land));
    f.base_holds = Some(true);
    assert_eq!(step(&c, &f).decision, Decision::Report);
}

#[test]
fn no_landing_without_a_goal_branch() {
    let mut f = facts();
    f.goal_branch = false;
    assert_eq!(step(&Closer::new(), &f).decision, Decision::Report);
}

#[test]
fn report_last() {
    let mut f = facts();
    f.base_holds = Some(true);
    let mut c = Closer::new();
    c.closed = Some(f.landed.clone());
    c.running = Some(Running { step: Step::Land, change: 20 });
    assert_eq!(step(&c, &f).decision, Decision::Wait);
    f.child = Child::Done;
    assert_eq!(step(&c, &f).decision, Decision::Report);
}

#[test]
fn a_step_change_that_ended_without_landing_holds_the_closer() {
    let mut f = facts();
    f.child = Child::Ended { branch: false };
    let mut c = Closer::new();
    c.running = Some(Running { step: Step::Close, change: 20 });
    assert_eq!(step(&c, &f).decision, Decision::Hold(Hold::ClosingStep { step: Step::Close, change: 20 }));
}

#[test]
fn a_release_makes_the_step_change_again() {
    let mut f = facts();
    f.child = Child::Ended { branch: false };
    let mut c = Closer::new();
    c.running = Some(Running { step: Step::Close, change: 20 });
    let c = step(&c, &f).closer;
    f.released = true;
    let s = step(&c, &f);
    assert_eq!(s.decision, Decision::Make(Step::Close));
    assert_eq!(s.closer.hold, None);
    assert_eq!(s.closer.running, None);
}

#[test]
fn an_ended_landing_reports_when_the_base_holds_the_branch() {
    for running in [true, false] {
        let mut f = facts();
        f.child = Child::Ended { branch: false };
        f.landing_child = Child::Ended { branch: false };
        f.base_holds = Some(true);
        let mut c = Closer::new();
        c.closed = Some(f.landed.clone());
        c.landing = Some(20);
        c.running = running.then_some(Running { step: Step::Land, change: 20 });
        let s = step(&c, &f);
        assert_eq!(s.decision, Decision::Report);
        assert_eq!(s.closer.landing, None, "the report names no landing");
        assert_eq!(s.closer.hold, None);
    }
}

#[test]
fn an_ended_landing_holds_when_the_base_does_not_hold_the_branch() {
    for running in [true, false] {
        let mut f = facts();
        f.child = Child::Ended { branch: false };
        f.landing_child = Child::Ended { branch: false };
        let mut c = Closer::new();
        c.closed = Some(f.landed.clone());
        c.landing = Some(20);
        c.running = running.then_some(Running { step: Step::Land, change: 20 });
        assert_eq!(step(&c, &f).decision, Decision::Hold(Hold::ClosingStep { step: Step::Land, change: 20 }));
    }
}

#[test]
fn a_released_landing_reports_or_makes_a_new_landing() {
    for base_holds in [true, false] {
        let mut f = facts();
        f.child = Child::Ended { branch: false };
        f.landing_child = Child::Ended { branch: false };
        f.base_holds = Some(base_holds);
        f.released = true;
        let mut c = Closer::new();
        c.closed = Some(f.landed.clone());
        c.landing = Some(20);
        c.running = Some(Running { step: Step::Land, change: 20 });
        c.hold = Some(Hold::ClosingStep { step: Step::Land, change: 20 });
        let s = step(&c, &f);
        assert_eq!(s.decision, if base_holds { Decision::Report } else { Decision::Make(Step::Land) });
        assert_eq!(s.closer.hold, None);
        assert_eq!(s.closer.running, None);
    }
}

#[test]
fn an_ended_landing_reads_before_deciding_and_keeps_its_release() {
    let mut f = facts();
    f.child = Child::Ended { branch: false };
    f.landing_child = Child::Ended { branch: false };
    f.base_holds = None;
    let mut c = Closer::new();
    c.closed = Some(f.landed.clone());
    c.landing = Some(20);
    c.running = Some(Running { step: Step::Land, change: 20 });
    let s = step(&c, &f);
    assert_eq!(s.decision, Decision::Check);
    assert_eq!(s.closer.hold, None);
    c.hold = Some(Hold::ClosingStep { step: Step::Land, change: 20 });
    f.released = true;
    let s = step(&c, &f);
    assert_eq!(s.decision, Decision::Check);
    assert_eq!(s.closer.hold, None);
    assert_eq!(s.closer.retry, Some(Step::Land));
    f.released = false;
    for base_holds in [false, true] {
        f.base_holds = Some(base_holds);
        let s = step(&s.closer, &f);
        assert_eq!(s.decision, if base_holds { Decision::Report } else { Decision::Make(Step::Land) });
        assert_eq!(s.closer.hold, None, "a release never holds again on the ended landing");
    }
}

#[test]
fn a_change_landed_into_the_branch_since_the_close_step_closes_again() {
    let f = facts();
    let mut c = Closer::new();
    c.closed = Some(Box::new([9]));
    assert_eq!(step(&c, &f).decision, Decision::Make(Step::Close));
}

#[test]
fn a_branch_moved_past_its_landing_lands_again() {
    let mut f = facts();
    f.child = Child::Done;
    f.landing_child = Child::Done;
    let mut c = Closer::new();
    c.closed = Some(f.landed.clone());
    c.running = Some(Running { step: Step::Land, change: 20 });
    c.landing = Some(20);
    assert_eq!(step(&c, &f).decision, Decision::Make(Step::Land));
}

#[test]
fn a_kept_closer_makes_no_new_step() {
    let mut f = facts();
    f.pending = true;
    let c = Closer::new();
    assert_eq!(step(&c, &f).decision, Decision::Wait);
    let mut c = c;
    c.running = Some(Running { step: Step::Close, change: 20 });
    let s = step(&c, &f);
    assert_eq!(s.decision, Decision::Wait);
    assert_eq!(s.closer.running, c.running);
}

#[test]
fn new_work_closes_again_while_the_landing_is_live() {
    let mut f = facts();
    let mut c = Closer::new();
    c.closed = Some(Box::new([9]));
    c.landing = Some(20);
    c.running = Some(Running { step: Step::Land, change: 20 });
    assert_eq!(step(&c, &f).decision, Decision::Make(Step::Close));
    c.running = Some(Running { step: Step::Close, change: 21 });
    c.closed = Some(f.landed.clone());
    f.child = Child::Done;
    assert_eq!(step(&c, &f).decision, Decision::Wait);
}
#[test]
fn a_released_retry_waits_for_dependencies() {
    let mut f = facts();
    f.released = true;
    f.pending = true;
    let mut c = Closer::new();
    c.closed = Some(f.landed.clone());
    c.running = Some(Running { step: Step::Close, change: 20 });
    c.hold = Some(Hold::ClosingStep { step: Step::Close, change: 20 });
    let s = step(&c, &f);
    assert_eq!(s.decision, Decision::Wait);
    assert_eq!(s.closer.hold, None);
    assert_eq!(s.closer.retry, Some(Step::Close));
    f.released = false;
    f.pending = false;
    assert_eq!(step(&s.closer, &f).decision, Decision::Make(Step::Close));
}
