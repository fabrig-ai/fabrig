use super::*;

fn facts() -> Facts {
    Facts {
        report: ReportRead::Read(ReportFields::empty()),
        goal_branch: true,
        close_job: Some(true),
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
fn an_unread_report_is_read_before_the_next_step() {
    for (goal_branch, closed) in [
        (true, None),
        (true, Some([9, 10])), // New work landed since the previous settlement.
        (true, Some([9, 12])),
        (false, None),
    ] {
        let mut f = facts();
        f.goal_branch = goal_branch;
        f.report = ReportRead::Unread;
        let mut closer = Closer::new();
        if let Some(tasks) = closed {
            closer.closed = Some(Box::new(tasks));
        }
        assert_eq!(step(&closer, &f).decision, Decision::ReadReport);
    }
}

#[test]
fn an_unread_report_waits_while_new_work_is_pending() {
    let mut f = facts();
    f.report = ReportRead::Unread;
    f.pending = true;
    let mut closer = Closer::new();
    closer.closed = Some(Box::new([9]));
    assert_eq!(step(&closer, &f).decision, Decision::Wait);
}

#[test]
fn a_failed_report_read_waits_and_then_asks_again() {
    let mut f = facts();
    f.report = ReportRead::Failed;
    let first = step(&Closer::new(), &f);
    assert_eq!(first.decision, Decision::Wait);
    f.report = ReportRead::Unread;
    assert_eq!(step(&first.closer, &f).decision, Decision::ReadReport);
}

#[test]
fn the_goals_answers_pass_without_holding_the_closer() {
    let mut f = facts();
    let asked = b"Asked: causes\nthe cache was cold".as_slice();
    f.report = ReportRead::Read(ReportFields { left: None, asked: Some(asked.into()) });
    let next = step(&Closer::new(), &f);
    assert_eq!(next.decision, Decision::Make(Step::Close));
    assert_eq!(next.closer.goal_asked.as_deref(), Some(asked));
    assert_eq!(next.closer.goal_left, None);
}

#[test]
fn a_report_holding_the_closer_keeps_both_parts() {
    let mut f = facts();
    let asked = b"Asked: causes\nthe cache was cold".as_slice();
    let left = b"the switch".as_slice();
    f.report = ReportRead::Read(ReportFields { left: Some(left.into()), asked: Some(asked.into()) });
    let held = step(&Closer::new(), &f);
    assert_eq!(held.decision, Decision::HoldReport);
    assert_eq!(held.closer.report_check, ReportCheck::Held);
    assert_eq!(held.closer.goal_left.as_deref(), Some(left));
    assert_eq!(held.closer.goal_asked.as_deref(), Some(asked));
}

#[test]
fn a_released_report_check_can_read_but_not_hold_again() {
    let mut f = facts();
    f.report = ReportRead::Read(ReportFields { left: Some(b"the switch".as_slice().into()), ..ReportFields::empty() });
    let held = step(&Closer::new(), &f);
    assert_eq!(held.decision, Decision::HoldReport);
    f.released = true;
    let released = step(&held.closer, &f);
    assert_eq!(released.decision, Decision::Make(Step::Close));
    f.released = false;
    f.close_job = Some(false);
    f.report = ReportRead::Unread;
    let read = step(&released.closer, &f);
    assert_eq!(read.decision, Decision::ReadReport);
    f.report =
        ReportRead::Read(ReportFields { left: Some(b"a different switch".as_slice().into()), ..ReportFields::empty() });
    let next = step(&read.closer, &f);
    assert_eq!(next.decision, Decision::Make(Step::Land));
    assert_eq!(next.closer.goal_left.as_deref(), Some(b"a different switch".as_slice()));
}

#[test]
fn close_first_where_the_project_names_a_close_job() {
    assert_eq!(step(&Closer::new(), &facts()).decision, Decision::Make(Step::Close));
}

#[test]
fn no_close_step_without_a_close_job_or_a_goal_branch() {
    let mut f = facts();
    f.close_job = Some(false);
    assert_eq!(step(&Closer::new(), &f).decision, Decision::Make(Step::Land));
    f.close_job = Some(true);
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
fn an_idle_close_reopens_or_wakes_after_work_ends() {
    const CLOSE: u64 = 20;
    const PREVIOUS_LANDINGS: [u64; 1] = [9]; // The first of the two changes in facts().
    // Pending work, new landings, held on failure, expected decision.
    for (pending, new, held, expected) in [
        (true, true, true, Decision::Wait),
        (true, true, false, Decision::Wait),
        (true, false, true, Decision::Wait),
        (true, false, false, Decision::Wait),
        (false, true, true, Decision::Supersede { change: CLOSE }),
        (false, true, false, Decision::Supersede { change: CLOSE }),
        (false, false, true, Decision::Wait),
        (false, false, false, Decision::Wake { change: CLOSE }),
    ] {
        let mut f = facts();
        f.pending = pending;
        f.child = Child::Idle { held };
        let mut c = Closer::new();
        c.closed = Some(if new { Box::new(PREVIOUS_LANDINGS) } else { f.landed.clone() });
        c.running = Some(Running { step: Step::Close, change: CLOSE });
        let s = step(&c, &f);
        assert_eq!(s.decision, expected, "pending {pending}, new landings {new}, held {held}");
        assert_eq!(s.closer.closed, c.closed);
        let running = if !pending && new { None } else { c.running };
        assert_eq!(s.closer.running, running);
    }
}

#[test]
fn an_idle_close_keeps_running_when_checking_or_skipping_its_replacement() {
    const CLOSE: u64 = 20;
    const PREVIOUS_LANDINGS: [u64; 1] = [9]; // The first of the two changes in facts().
    let mut c = Closer::new();
    c.closed = Some(Box::new(PREVIOUS_LANDINGS));
    c.running = Some(Running { step: Step::Close, change: CLOSE });
    for (close_job, expected) in [(None, Decision::CheckClose), (Some(false), Decision::SkipClose { change: CLOSE })] {
        let mut f = facts();
        f.child = Child::Idle { held: true };
        f.close_job = close_job;
        let s = step(&c, &f);
        assert_eq!(s.decision, expected);
        assert_eq!(s.closer.running, c.running, "close job {close_job:?}");
    }
}

#[test]
fn an_idle_close_cannot_bypass_the_report_hold() {
    const CLOSE: u64 = 20;
    for (closed, after_release) in
        [([9, 10], Decision::Supersede { change: CLOSE }), ([9, 12], Decision::Wake { change: CLOSE })]
    {
        let mut f = facts();
        f.child = Child::Idle { held: false };
        f.report =
            ReportRead::Read(ReportFields { left: Some(b"the switch".as_slice().into()), ..ReportFields::empty() });
        let mut c = Closer::new();
        c.closed = Some(Box::new(closed));
        c.running = Some(Running { step: Step::Close, change: CLOSE });
        let held = step(&c, &f);
        assert_eq!(held.decision, Decision::HoldReport);
        assert_eq!(held.closer.running, c.running);
        f.released = true;
        assert_eq!(step(&held.closer, &f).decision, after_release);
    }
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
