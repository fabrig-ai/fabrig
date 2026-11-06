//! A model's trials, a row per account (temper/docs/design/domain/trial.md, 3).
#![no_std]
#![forbid(unsafe_code)]

use skein_lib::{Duration, Env, List, Queue, Wall};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub rows: u32,
}

#[derive(Debug)]
pub struct Trials {
    rows: List<Row>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Row {
    pub account: u32,
    pub person: u64,
    pub task: Option<u64>,
    pub began: Wall,
    pub took: Option<u32>,
    pub step: Step,
    pub state: State,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step { Starts, Answers }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State { Running, Answered { seconds: u32 }, Failed { why: Why } }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Why { Ended { class: Class }, Refused, Waited }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class { Transient, Permanent, Run, Agent, Lost, Invalid, Refused }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Event {
    Try { account: u32, person: u64, again: bool },
    Made { account: u32, task: u64 },
    Refused { account: u32 },
    Observed { task: u64, past_starting: bool, took: Option<u32> },
    Turn { task: u64 },
    Finished { task: u64 },
    RunFailed { task: u64, class: Class },
    Due,
    AccountGone { account: u32 },
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Request {
    Make { account: u32, person: u64 },
    Cancel { task: u64 },
    Changed { person: u64 },
}

/// The minute from the trial's beginning (temper/docs/design/domain/trial.md, 3.2).
pub const WAIT: Duration = Duration::from_secs(60);
/// The person's trials document (temper/docs/plan/model-check/notes/contracts.md).
pub const TRIALS_SUBJECT: u64 = (1 << 62) | (1 << 36);

impl Trials {
    #[must_use]
    pub fn new(limits: &Limits) -> Self {
        Self { rows: List::with_capacity(limits.rows) }
    }

    pub fn rows(&self, _person: u64) -> impl Iterator<Item = &Row> {
        let mut rows = List::with_capacity(self.rows.len());
        for row in &self.rows {
            rows.push(row).expect("no more references than rows");
        }
        rows.into_boxed().into_iter()
    }

    /// The tasks of running rows, for the root's observation.
    pub fn running(&self) -> impl Iterator<Item = u64> + '_ {
        List::<u64>::with_capacity(0).into_boxed().into_iter()
    }

    #[must_use]
    pub fn holds(&self, _task: u64) -> bool { false }

    #[must_use]
    pub fn next_due(&self) -> Option<Wall> { None }
}

/// Apply one event (temper/docs/design/domain/trial.md, 3.2).
pub fn step(trials: &mut Trials, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Try { account, person, again: _ } => start(trials, env.wall, account, person, out),
        Event::Made { .. } | Event::Refused { .. } | Event::Observed { .. } | Event::Turn { .. }
        | Event::Finished { .. } | Event::RunFailed { .. } | Event::Due | Event::AccountGone { .. } => {}
    }
}

fn start(trials: &mut Trials, wall: Wall, account: u32, person: u64, out: &mut Queue<Request>) {
    let mut existing = None;
    let mut oldest: Option<(u32, Wall)> = None;
    for index in 0..trials.rows.len() {
        let row = trials.rows.get(index).expect("index is within row count");
        if row.account == account { existing = Some(index); }
        match row.state {
            State::Running => {},
            State::Answered { .. } | State::Failed { .. } => {
                let replace = match oldest { Some((_, began)) => row.began < began, None => true };
                if replace { oldest = Some((index, row.began)); }
            }
        }
    }
    let row = Row { account, person, task: None, began: wall, took: None, step: Step::Starts, state: State::Running };
    if let Some(index) = existing {
        *trials.rows.get_mut(index).expect("the account's row exists") = row;
    } else {
        if trials.rows.room() == 0 {
            let Some((index, _)) = oldest else { return };
            let removed = trials.rows.swap_remove(index).expect("oldest ended row exists");
            if removed.person != person { out.push(Request::Changed { person: removed.person }); }
        }
        trials.rows.push(row).expect("room for the admitted trial");
    }
    out.push(Request::Make { account, person });
    out.push(Request::Changed { person });
}

#[cfg(test)]
mod tests;
