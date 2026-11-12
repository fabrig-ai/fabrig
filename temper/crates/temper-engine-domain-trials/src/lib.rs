//! A model's trials, a row per account (temper/docs/design/domain/trial.md, 3).
#![no_std]
#![forbid(unsafe_code)]

use skein_lib::{Duration, Env, List, Queue, Wall};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub rows: u32,
}

#[derive(Hash, Debug)]
pub struct Trials {
    rows: List<Row>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Row {
    pub account: u32,
    pub person: u64,
    pub task: Option<u64>,
    pub began: Wall,
    pub took: Option<u32>,
    pub step: Step,
    pub state: State,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Step {
    Starts,
    Answers,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum State {
    Running,
    Answered { seconds: u32 },
    Failed { why: Why },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Why {
    Ended { class: Class },
    Refused,
    Waited,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Class {
    Transient,
    Permanent,
    Run,
    Agent,
    Lost,
    Invalid,
    Refused,
    Infrastructure,
}
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
/// The person's trials document (temper/docs/design/domain/trial.md, 6).
pub const TRIALS_SUBJECT: u64 = (1 << 62) | (1 << 36);

impl Trials {
    #[must_use]
    pub fn new(limits: &Limits) -> Self {
        Self { rows: List::with_capacity(limits.rows) }
    }

    pub fn rows(&self, person: u64) -> impl Iterator<Item = &Row> {
        let mut rows = List::with_capacity(self.rows.len());
        for row in &self.rows {
            if row.person == person {
                rows.push(row).expect("no more references than rows");
            }
        }
        rows.into_boxed().into_iter()
    }

    /// The tasks of running rows, for the root's observation.
    pub fn running(&self) -> impl Iterator<Item = u64> + '_ {
        let mut tasks = List::with_capacity(self.rows.len());
        for row in &self.rows {
            if is_running(row)
                && let Some(task) = row.task
            {
                tasks.push(task).expect("no more tasks than rows");
            }
        }
        tasks.into_boxed().into_iter()
    }

    #[must_use]
    pub fn holds(&self, task: u64) -> bool {
        for row in &self.rows {
            if row.task == Some(task) {
                return true;
            }
        }
        false
    }

    #[must_use]
    pub fn next_due(&self) -> Option<Wall> {
        let mut next = None;
        for row in &self.rows {
            if is_running(row) {
                let due = due(row);
                next = Some(match next {
                    Some(previous) => due.min(previous),
                    None => due,
                });
            }
        }
        next
    }
}

/// Apply one event (temper/docs/design/domain/trial.md, 3.2).
pub fn step(trials: &mut Trials, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Try { account, person, again } => start(trials, env.wall, account, person, again, out),
        Event::AccountGone { account } => remove(trials, account, out),
        Event::Made { .. }
        | Event::Refused { .. }
        | Event::Observed { .. }
        | Event::Turn { .. }
        | Event::Finished { .. }
        | Event::RunFailed { .. }
        | Event::Due => {
            for row in trials.rows.iter_mut() {
                if is_running(row) && update(row, env.wall, event, out) {
                    out.push(Request::Changed { person: row.person });
                }
            }
        }
    }
}

fn is_running(row: &Row) -> bool {
    match row.state {
        State::Running => true,
        State::Answered { .. } | State::Failed { .. } => false,
    }
}

fn due(row: &Row) -> Wall {
    Wall::from_nanos(row.began.as_nanos().saturating_add(WAIT.as_nanos()))
}

fn seconds(began: Wall, wall: Wall) -> u32 {
    let nanos = wall.as_nanos().saturating_sub(began.as_nanos());
    let seconds = nanos.div_ceil(Duration::from_secs(1).as_nanos()).max(1);
    u32::try_from(seconds).unwrap_or(u32::MAX)
}

fn end(row: &mut Row, state: State, out: &mut Queue<Request>) {
    row.state = state;
    if let Some(task) = row.task {
        out.push(Request::Cancel { task });
    }
}

fn update(row: &mut Row, wall: Wall, event: Event, out: &mut Queue<Request>) -> bool {
    match event {
        Event::Made { account, task } => {
            if row.account != account || row.task == Some(task) {
                return false;
            }
            row.task = Some(task);
        }
        Event::Refused { account } => {
            if row.account != account {
                return false;
            }
            row.state = State::Failed { why: Why::Refused };
        }
        Event::Observed { task, past_starting, took } => {
            if row.task != Some(task) {
                return false;
            }
            let mut changed = false;
            match row.step {
                Step::Starts => {
                    if past_starting {
                        row.step = Step::Answers;
                        changed = true;
                    }
                }
                Step::Answers => {}
            }
            if row.took.is_none() && took.is_some() {
                row.took = took;
                changed = true;
            }
            return changed;
        }
        Event::Turn { task } | Event::Finished { task } => {
            if row.task != Some(task) {
                return false;
            }
            let seconds = seconds(row.began, wall);
            end(row, State::Answered { seconds }, out);
        }
        Event::RunFailed { task, class } => {
            if row.task != Some(task) {
                return false;
            }
            end(row, State::Failed { why: Why::Ended { class } }, out);
        }
        Event::Due => {
            if wall < due(row) {
                return false;
            }
            end(row, State::Failed { why: Why::Waited }, out);
        }
        Event::Try { .. } | Event::AccountGone { .. } => return false,
    }
    true
}

fn remove(trials: &mut Trials, account: u32, out: &mut Queue<Request>) {
    let mut index = 0;
    for _ in 0..trials.rows.len() {
        let row = trials.rows.get(index).expect("each original row is visited once");
        if row.account == account || row.took == Some(account) {
            let row = trials.rows.swap_remove(index).expect("visited row exists");
            if is_running(&row)
                && let Some(task) = row.task
            {
                out.push(Request::Cancel { task });
            }
            out.push(Request::Changed { person: row.person });
        } else {
            index = index.checked_add(1).expect("index is within row count");
        }
    }
}

fn start(trials: &mut Trials, wall: Wall, account: u32, person: u64, again: bool, out: &mut Queue<Request>) {
    let mut existing = None;
    let mut oldest: Option<(u32, Wall)> = None;
    for index in 0..trials.rows.len() {
        let row = trials.rows.get(index).expect("index is within row count");
        if row.account == account {
            if !again || is_running(row) {
                return;
            }
            existing = Some(index);
        }
        match row.state {
            State::Running => {}
            State::Answered { .. } | State::Failed { .. } => {
                let replace = match oldest {
                    Some((_, began)) => row.began < began,
                    None => true,
                };
                if replace {
                    oldest = Some((index, row.began));
                }
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
            if removed.person != person {
                out.push(Request::Changed { person: removed.person });
            }
        }
        trials.rows.push(row).expect("room for the admitted trial");
    }
    out.push(Request::Make { account, person });
    out.push(Request::Changed { person });
}

#[cfg(test)]
mod tests;
