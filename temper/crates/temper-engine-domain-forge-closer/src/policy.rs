//! A goal's ordered, repeatable closing steps (temper/docs/design/domain/forge/branches.md, 2).
use alloc::boxed::Box;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Step {
    Close,
    Land,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Hold {
    ClosingStep { step: Step, change: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Running {
    pub step: Step,
    pub change: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Closer {
    pub closed: Option<Box<[u64]>>,
    pub running: Option<Running>,
    pub landing: Option<u64>,
    pub hold: Option<Hold>,
    pub retry: Option<Step>,
    pub selections: Box<[crate::PullSelection]>,
}

impl Closer {
    #[must_use]
    #[expect(clippy::new_without_default, reason = "step values use named constructors, not user-defined traits")]
    pub fn new() -> Self {
        Self { closed: None, running: None, landing: None, hold: None, retry: None, selections: Box::new([]) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Child {
    Pending,
    Done,
    Ended { branch: bool },
}

#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "the root supplies independent observations to the pure policy")]
pub struct Facts {
    pub goal_branch: bool,
    pub close_job: bool,
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
    Check,
    Wait,
    Make(Step),
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
    if let Some(hold) = &closer.hold {
        if !facts.released {
            let decision = Decision::Hold(hold.clone());
            return Stepped { closer, decision };
        }
        closer.hold = None;
        let Some(running) = closer.running.take() else { unreachable!("a closing hold names the running change") };
        closer.retry = Some(running.step);
    }
    if let Some(retry) = closer.retry {
        let decision = retry_step(&mut closer, facts, retry);
        return Stepped { closer, decision };
    }
    if let Some(running) = closer.running {
        match facts.child {
            Child::Pending => {
                if running.step == Step::Land
                    && !facts.pending
                    && facts.close_job
                    && closer.closed.as_deref() != Some(facts.landed.as_ref())
                {
                    closer.running = None;
                    return Stepped { closer, decision: Decision::Make(Step::Close) };
                }
                return Stepped { closer, decision: Decision::Wait };
            }
            Child::Ended { branch } => match running.step {
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
            Child::Done => closer.running = None,
        }
    }
    let decision = if facts.pending {
        Decision::Wait
    } else if facts.goal_branch && facts.close_job && closer.closed.as_deref() != Some(facts.landed.as_ref()) {
        Decision::Make(Step::Close)
    } else if let Some(change) = closer.landing {
        match facts.landing_child {
            Child::Pending => Decision::Wait,
            Child::Ended { branch } => match ended_landing(&mut closer, facts, change, branch) {
                Some(decision) => decision,
                None => land(facts),
            },
            Child::Done => land(facts),
        }
    } else {
        land(facts)
    };
    Stepped { closer, decision }
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
    closer.running = None;
    closer.retry = Some(kind);
    if facts.pending {
        return Decision::Wait;
    }
    let decision = match kind {
        Step::Close => Decision::Make(Step::Close),
        Step::Land => land(facts),
    };
    match decision {
        Decision::Check => {}
        Decision::Make(Step::Close | Step::Land) | Decision::Report | Decision::Wait | Decision::Hold(_) => {
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
        Decision::Make(Step::Close) | Decision::Wait | Decision::Hold(_) => {
            unreachable!("containment reports, lands or asks a fresh read")
        }
    }
}

#[cfg(test)]
mod tests;
