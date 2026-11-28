//! A goal's ordered, repeatable closing steps (temper/docs/design/domain/forge/branches.md, 2).
use alloc::boxed::Box;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Step {
    Close,
    Land,
    CatchUp,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Hold {
    ClosingStep { step: Step, change: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReportCheck {
    Unread,
    Held,
    Released,
}

#[derive(Debug)]
pub enum ReportRead {
    Unread,
    Failed,
    Read(ReportFields),
}

#[derive(Clone, Debug, Hash)]
pub struct ReportFields {
    pub left: Option<Box<[u8]>>,
    pub asked: Option<Box<[u8]>>,
}

impl ReportFields {
    #[must_use]
    pub const fn empty() -> Self {
        Self { left: None, asked: None }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Running {
    pub step: Step,
    pub change: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Closer {
    pub closed: Option<Box<[u64]>>,
    pub caught_up: Option<Box<[u64]>>,
    pub running: Option<Running>,
    pub landing: Option<u64>,
    pub hold: Option<Hold>,
    pub retry: Option<Step>,
    pub selections: Box<[crate::PullSelection]>,
    pub report_check: ReportCheck,
    pub goal_left: Option<Box<[u8]>>,
    pub goal_asked: Option<Box<[u8]>>,
}

impl Closer {
    #[must_use]
    #[expect(clippy::new_without_default, reason = "step values use named constructors, not user-defined traits")]
    pub fn new() -> Self {
        Self {
            closed: None,
            caught_up: None,
            running: None,
            landing: None,
            hold: None,
            retry: None,
            selections: Box::new([]),
            report_check: ReportCheck::Unread,
            goal_left: None,
            goal_asked: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Child {
    Pending,
    Idle { held: bool },
    Done,
    Outdated,
    Ended { branch: bool },
}

#[derive(Debug)]
pub struct Facts {
    pub report: ReportRead,
    pub goal_branch: bool,
    /// None until the goal branch tip has answered the job-command read.
    pub close_job: Option<bool>,
    /// The outdated close's request, when the closer can make its catch-up.
    pub catch_up: Option<Box<[u64]>>,
    pub pending: bool,
    pub landed: Box<[u64]>,
    pub base_holds: Option<bool>,
    pub child: Child,
    pub landing_child: Child,
    pub released: bool,
    pub close_ended: u32,
    pub land_ended: u32,
    pub bound: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    ReadReport,
    HoldReport,
    Check,
    CheckClose,
    Wait,
    Make(Step),
    Supersede { change: u64 },
    SkipClose { change: u64 },
    Wake { change: u64 },
    Report,
    Hold(Hold),
}

#[derive(Debug)]
pub struct Stepped {
    pub closer: Closer,
    pub decision: Decision,
}

#[must_use]
pub fn step(closer: &Closer, facts: &Facts) -> Stepped {
    let mut closer = closer.clone();
    if closer.report_check == ReportCheck::Held {
        if !facts.released {
            return Stepped { closer, decision: Decision::HoldReport };
        }
        closer.report_check = ReportCheck::Released;
    }
    if let Some(hold) = &closer.hold {
        if !facts.released {
            let decision = Decision::Hold(hold.clone());
            return Stepped { closer, decision };
        }
        closer.hold = None;
        let Some(running) = closer.running else { unreachable!("a closing hold names the running change") };
        closer.retry = Some(running.step);
    }
    let mut stepped = closing_step(closer.clone(), facts);
    match stepped.decision {
        Decision::ReadReport | Decision::HoldReport => unreachable!("only the report gate requests a note read"),
        Decision::Wait | Decision::Hold(_) => stepped,
        // An ended landing may still need containment to judge its ending.
        Decision::Check if facts.pending => stepped,
        Decision::Check
        | Decision::CheckClose
        | Decision::Make(Step::Close | Step::Land | Step::CatchUp)
        | Decision::Supersede { .. }
        | Decision::SkipClose { .. }
        | Decision::Wake { .. }
        | Decision::Report => {
            if facts.pending {
                return Stepped { closer, decision: Decision::Wait };
            }
            match &facts.report {
                ReportRead::Unread => Stepped { closer, decision: Decision::ReadReport },
                ReportRead::Failed => Stepped { closer, decision: Decision::Wait },
                ReportRead::Read(fields) => {
                    if fields.left.is_some() && closer.report_check != ReportCheck::Released {
                        closer.report_check = ReportCheck::Held;
                        stepped = Stepped { closer, decision: Decision::HoldReport };
                    }
                    stepped.closer.goal_left.clone_from(&fields.left);
                    stepped.closer.goal_asked.clone_from(&fields.asked);
                    stepped
                }
            }
        }
    }
}

fn closing_step(mut closer: Closer, facts: &Facts) -> Stepped {
    if let Some(retry) = closer.retry {
        let decision = retry_step(&mut closer, facts, retry);
        return Stepped { closer, decision };
    }
    if let Some(running) = closer.running {
        match facts.child {
            Child::Pending => {
                if running.step == Step::Land
                    && !facts.pending
                    && facts.goal_branch
                    && closer.closed.as_deref() != Some(facts.landed.as_ref())
                {
                    match facts.close_job {
                        None => return Stepped { closer, decision: Decision::CheckClose },
                        Some(true) => {
                            closer.running = None;
                            return Stepped { closer, decision: Decision::Make(Step::Close) };
                        }
                        Some(false) => {}
                    }
                }
                return Stepped { closer, decision: Decision::Wait };
            }
            Child::Idle { held } => {
                let decision = if facts.pending || running.step != Step::Close {
                    Decision::Wait
                } else if closer.closed.as_deref() != Some(facts.landed.as_ref()) {
                    match facts.close_job {
                        None => Decision::CheckClose,
                        Some(true) => {
                            closer.running = None;
                            Decision::Supersede { change: running.change }
                        }
                        Some(false) => Decision::SkipClose { change: running.change },
                    }
                } else if held {
                    Decision::Wait
                } else {
                    Decision::Wake { change: running.change }
                };
                return Stepped { closer, decision };
            }
            Child::Outdated => {
                if facts.pending {
                    return Stepped { closer, decision: Decision::Wait };
                }
                if facts.close_job.is_none() {
                    return Stepped { closer, decision: Decision::CheckClose };
                }
                if closer.closed.as_deref() == Some(facts.landed.as_ref()) {
                    let decision = outdated(&mut closer, facts, running);
                    return Stepped { closer, decision };
                }
                closer.running = None;
            }
            Child::Ended { branch } => match running.step {
                Step::CatchUp => {
                    let decision = hold(&mut closer, running);
                    return Stepped { closer, decision };
                }
                Step::Close => {
                    let decision = ended_step(&mut closer, facts, running, branch);
                    return Stepped { closer, decision };
                }
                Step::Land => {
                    if let Some(decision) = ended_landing(&mut closer, facts, running.change, branch) {
                        return Stepped { closer, decision };
                    }
                }
            },
            // Retain the catch-up as the running step until its replacement is admitted.
            Child::Done if running.step == Step::CatchUp && facts.close_job != Some(false) => {
                return Stepped { closer, decision: close(facts) };
            }
            Child::Done => closer.running = None,
        }
    }
    let decision = if facts.pending {
        Decision::Wait
    } else if facts.goal_branch
        && facts.close_job != Some(false)
        && closer.closed.as_deref() != Some(facts.landed.as_ref())
    {
        close(facts)
    } else if let Some(change) = closer.landing {
        match facts.landing_child {
            Child::Pending | Child::Idle { .. } => Decision::Wait,
            Child::Ended { branch } => match ended_landing(&mut closer, facts, change, branch) {
                Some(decision) => decision,
                None => land(facts),
            },
            Child::Done => land(facts),
            Child::Outdated => unreachable!("only a close can end outdated"),
        }
    } else {
        land(facts)
    };
    Stepped { closer, decision }
}

fn outdated(closer: &mut Closer, facts: &Facts, running: Running) -> Decision {
    if let Some(request) = facts.catch_up.as_deref()
        && closer.caught_up.as_deref() != Some(request)
    {
        // Keep the outdated close until the catch-up is admitted, including while waiting for funds.
        Decision::Make(Step::CatchUp)
    } else {
        hold(closer, running)
    }
}

fn close(facts: &Facts) -> Decision {
    match facts.close_job {
        None => Decision::CheckClose,
        Some(true) => Decision::Make(Step::Close),
        Some(false) => land(facts),
    }
}

fn land(facts: &Facts) -> Decision {
    if !facts.goal_branch {
        return Decision::Report;
    }
    match facts.base_holds {
        None => Decision::Check,
        Some(true) => Decision::Report,
        Some(false) => Decision::Make(Step::Land),
    }
}

fn hold(closer: &mut Closer, running: Running) -> Decision {
    let hold = Hold::ClosingStep { step: running.step, change: running.change };
    closer.running = Some(running);
    closer.hold = Some(hold.clone());
    Decision::Hold(hold)
}

fn retry_step(closer: &mut Closer, facts: &Facts, kind: Step) -> Decision {
    if kind != Step::CatchUp {
        closer.running = None;
    }
    closer.retry = Some(kind);
    if facts.pending {
        return Decision::Wait;
    }
    let decision = match kind {
        Step::Close => close(facts),
        Step::Land => land(facts),
        Step::CatchUp => {
            if facts.close_job.is_none() {
                return Decision::CheckClose;
            }
            if facts.catch_up.is_none() {
                closer.retry = None;
                let running = closer.running.expect("a released catch-up still names the failed change");
                return hold(closer, running);
            }
            Decision::Make(Step::CatchUp)
        }
    };
    match decision {
        Decision::ReadReport | Decision::HoldReport | Decision::Check | Decision::CheckClose => {}
        Decision::Make(Step::Close | Step::Land | Step::CatchUp)
        | Decision::Supersede { .. }
        | Decision::SkipClose { .. }
        | Decision::Wake { .. }
        | Decision::Report
        | Decision::Wait
        | Decision::Hold(_) => {
            closer.retry = None;
        }
    }
    if decision == Decision::Report {
        closer.landing = None;
    }
    decision
}

fn ended_step(closer: &mut Closer, facts: &Facts, running: Running, branch: bool) -> Decision {
    let ended = match running.step {
        Step::Close => facts.close_ended,
        Step::Land => facts.land_ended,
        Step::CatchUp => unreachable!("a catch-up does not count attempts"),
    };
    if branch || ended >= u32::from(facts.bound) {
        hold(closer, running)
    } else {
        retry_step(closer, facts, running.step)
    }
}

fn ended_landing(closer: &mut Closer, facts: &Facts, change: u64, branch: bool) -> Option<Decision> {
    match land(facts) {
        Decision::Check => Some(Decision::Check),
        Decision::Make(Step::Land) => Some(ended_step(closer, facts, Running { step: Step::Land, change }, branch)),
        Decision::Report => {
            closer.running = None;
            closer.landing = None;
            None
        }
        Decision::ReadReport
        | Decision::HoldReport
        | Decision::CheckClose
        | Decision::Make(Step::Close | Step::CatchUp)
        | Decision::Supersede { .. }
        | Decision::SkipClose { .. }
        | Decision::Wake { .. }
        | Decision::Wait
        | Decision::Hold(_) => {
            unreachable!("containment reports, lands or asks a fresh read")
        }
    }
}

#[cfg(test)]
mod tests;
