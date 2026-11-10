use super::*;

fn facts() -> Facts {
    Facts {
        goal_branch: true,
        close_job: true,
        pending: false,
        landed: Box::new([9, 12]),
        base_holds: false,
        child: Child::Pending,
        released: false,
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
    f.base_holds = true;
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
    f.base_holds = true;
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
    f.child = Child::Ended;
    let mut c = Closer::new();
    c.running = Some(Running { step: Step::Close, change: 20 });
    assert_eq!(step(&c, &f).decision, Decision::Hold(Hold::ClosingStep { step: Step::Close, change: 20 }));
}

#[test]
fn a_release_makes_the_step_change_again() {
    let mut f = facts();
    f.child = Child::Ended;
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
