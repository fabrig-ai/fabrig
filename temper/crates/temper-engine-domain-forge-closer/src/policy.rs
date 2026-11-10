//! A goal's ordered, repeatable closing steps (temper/docs/design/domain/forge.md, 11.2).
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
    Ended,
}

#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "the root supplies independent observations to the pure policy")]
pub struct Facts {
    pub goal_branch: bool,
    pub close_job: bool,
    pub pending: bool,
    pub landed: Box<[u64]>,
    pub base_holds: bool,
    pub child: Child,
    pub landing_child: Child,
    pub released: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
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
        if facts.pending {
            return Stepped { closer, decision: Decision::Wait };
        }
        closer.retry = None;
        return Stepped { closer, decision: Decision::Make(retry) };
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
            Child::Ended => {
                let hold = Hold::ClosingStep { step: running.step, change: running.change };
                closer.hold = Some(hold.clone());
                return Stepped { closer, decision: Decision::Hold(hold) };
            }
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
            Child::Ended => {
                let hold = Hold::ClosingStep { step: Step::Land, change };
                closer.running = Some(Running { step: Step::Land, change });
                closer.hold = Some(hold.clone());
                Decision::Hold(hold)
            }
            Child::Done => {
                if facts.goal_branch && !facts.base_holds {
                    Decision::Make(Step::Land)
                } else {
                    Decision::Report
                }
            }
        }
    } else if facts.goal_branch && !facts.base_holds {
        Decision::Make(Step::Land)
    } else {
        Decision::Report
    };
    Stepped { closer, decision }
}

#[cfg(test)]
mod tests;
