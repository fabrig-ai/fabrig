//! Durable-outcome referee (temper/docs/design/testing/engine-checks.md, 4).

use jig_domain_core as core;
use jig_domain_tasks as tasks;
use skein_lib::Time;
use temper_engine_domain::{Key, Record};

use crate::store::Store;

/// Independent bounds on simulated time and driver work. Both must be nonzero.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Bound {
    pub minutes: u64,
    pub steps: u64,
}

/// The outcome a story promises for a named task.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Expect {
    /// The task ends as its procedure prescribes, after the faults stop.
    Ends,
    /// A lasting fault holds the task with words naming that fault.
    Held(&'static str),
    /// The task ends from the moment it is named, irrespective of the faults.
    Untouched,
    /// A party holds the task or is scripted never to answer.
    Waits,
}

/// These profiles have no separate start-report bound. Delayed paged starts
/// across the five drivers measured at most 116 steps and 3.8 seconds in
/// `engine_drivers_open_within_the_shared_bound`. One minute is the smallest
/// whole minute; 1000 steps leaves room for larger restored inventories.
pub const OPENING: Bound = Bound { minutes: 1, steps: 1000 };

#[derive(Clone, Copy, Default, Hash, Debug)]
struct Moment {
    at: u64,
    step: u64,
}

#[derive(Hash, Debug)]
struct Named {
    task: u64,
    expect: Expect,
    bound: Bound,
    since: Moment,
    met: bool,
}

/// Reads only committed rows, the clock and the driver's readiness observation.
#[derive(Hash, Debug)]
pub struct Ends {
    seed: u64,
    opening: Bound,
    now: Moment,
    faults: Moment,
    last_open: Moment,
    restarting: Option<Moment>,
    named: Vec<Named>,
    finding: Option<String>,
}

impl Ends {
    #[must_use]
    pub fn new(seed: u64, opening: Bound) -> Self {
        assert!(opening.minutes != 0 && opening.steps != 0, "opening bound must be nonzero");
        Self {
            seed,
            opening,
            now: Moment::default(),
            faults: Moment::default(),
            last_open: Moment::default(),
            restarting: None,
            named: Vec::new(),
            finding: None,
        }
    }

    pub fn expect(&mut self, task: u64, expect: Expect, bound: Bound) -> Result<(), String> {
        if bound.minutes == 0 || bound.steps == 0 {
            return Err(self.format(&format!("task {task}"), "bound must have nonzero minutes and steps"));
        }
        self.named.push(Named { task, expect, bound, since: self.now, met: false });
        Ok(())
    }

    pub fn faults_stop(&mut self, at: Time) {
        self.faults = self.moment(at);
    }

    pub fn restarted(&mut self, at: Time) {
        if self.restarting.is_none() {
            self.restarting = Some(self.moment(at));
        }
    }

    /// A deliberately refused or interrupted start has no opening obligation.
    pub fn no_opening(&mut self, at: Time) {
        self.now = self.moment(at);
        self.restarting = None;
    }

    pub(crate) fn awaiting_open(&self) -> bool {
        self.restarting.is_some()
    }

    pub fn opened(&mut self, at: Time) {
        self.now = self.moment(at);
        if let Some(since) = self.restarting.take()
            && past(self.now, since, self.opening)
        {
            self.keep(self.format("opening", &opening_deadline(self.opening, self.now, since)));
        }
        self.last_open = self.now;
    }

    pub fn step(&mut self, at: Time, store: &Store) -> Result<(), String> {
        self.now.step = self.now.step.saturating_add(1);
        self.now.at = at.as_nanos();
        self.check(store, false)
    }

    pub fn close(&mut self, at: Time, store: &Store) -> Result<(), String> {
        self.now.at = at.as_nanos();
        self.check(store, true)
    }

    fn moment(&self, at: Time) -> Moment {
        Moment { at: at.as_nanos(), step: self.now.step }
    }

    fn keep(&mut self, finding: String) {
        if self.finding.is_none() {
            self.finding = Some(finding);
        }
    }

    fn format(&self, object: &str, what: &str) -> String {
        format!("seed {}: at {} ns, iteration {}: engine {object}: {what}", self.seed, self.now.at, self.now.step)
    }

    fn check(&mut self, store: &Store, closing: bool) -> Result<(), String> {
        if let Some(since) = self.restarting
            && (closing || reached(self.now, since, self.opening))
        {
            let what = if closing {
                "still unopened at close".into()
            } else {
                opening_deadline(self.opening, self.now, since)
            };
            self.keep(self.format("opening", &what));
        }
        let baseline =
            Moment { at: self.faults.at.max(self.last_open.at), step: self.faults.step.max(self.last_open.step) };
        for index in 0..self.named.len() {
            let named = &self.named[index];
            if named.met {
                continue;
            }
            let since = if named.expect == Expect::Untouched { named.since } else { baseline };
            let result = outcome(store, named.task, named.expect);
            let what = match result {
                Ok(true) if !past(self.now, since, named.bound) || named.expect == Expect::Waits => {
                    self.named[index].met = true;
                    continue;
                }
                Err(what) => Some(what),
                Ok(_) if closing => Some("expectation still unmet at close".into()),
                Ok(_)
                    if (self.restarting.is_none() || named.expect == Expect::Untouched)
                        && reached(self.now, since, named.bound) =>
                {
                    Some(deadline(named.bound))
                }
                Ok(_) => None,
            };
            if let Some(what) = what {
                self.keep(self.format(&format!("task {}", self.named[index].task), &what));
            }
        }
        self.finding.clone().map_or(Ok(()), Err)
    }
}

fn reached(now: Moment, since: Moment, bound: Bound) -> bool {
    now.at.saturating_sub(since.at) >= bound.minutes.saturating_mul(60_000_000_000)
        || now.step.saturating_sub(since.step) >= bound.steps
}

fn past(now: Moment, since: Moment, bound: Bound) -> bool {
    now.at.saturating_sub(since.at) > bound.minutes.saturating_mul(60_000_000_000)
        || now.step.saturating_sub(since.step) > bound.steps
}

fn opening_deadline(bound: Bound, now: Moment, since: Moment) -> String {
    format!(
        "did not open within {} minutes and {} steps (elapsed {} ns and {} steps)",
        bound.minutes,
        bound.steps,
        now.at.saturating_sub(since.at),
        now.step.saturating_sub(since.step)
    )
}

fn deadline(bound: Bound) -> String {
    format!("neither ended nor held within {} minutes and {} steps", bound.minutes, bound.steps)
}

/// The goal can be idle while its producer is waiting for its person. Read the
/// requester chain, not `root`: a goal may itself be beneath a chat or gathering.
fn descendant_waits(store: &Store, ancestor: u64) -> bool {
    store.rows.values().any(|record| {
        let Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) = record else { return false };
        row.number != ancestor
            && (matches!(row.phase, tasks::Phase::Held { .. }) || !row.questions.is_empty())
            && beneath(store, row.number, ancestor)
    })
}

fn beneath(store: &Store, mut number: u64, ancestor: u64) -> bool {
    // A requester chain has at most one edge per retained task. A malformed
    // cycle cannot turn an unrelated task into a descendant of this goal.
    for _ in 0..store.rows.len() {
        let row = [tasks::Key::Live(number), tasks::Key::Ended(number)].into_iter().find_map(|key| {
            match store.rows.get(&Key::Core(core::Key::Tasks(key))) {
                Some(Record::Core(core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))) => {
                    Some(row)
                }
                _ => None,
            }
        });
        let Some(row) = row else { return false };
        let tasks::Party::Task(parent) = row.requester else { return false };
        if parent == ancestor {
            return true;
        }
        number = parent;
    }
    false
}

fn outcome(store: &Store, task: u64, expect: Expect) -> Result<bool, String> {
    let ended = Key::Core(core::Key::Tasks(tasks::Key::Ended(task)));
    if let Some(Record::Core(core::Record::Tasks(tasks::Stored::Ended(row)))) = store.rows.get(&ended)
        && matches!(row.phase, tasks::Phase::Ended(_))
    {
        return match expect {
            Expect::Ends | Expect::Untouched => Ok(true),
            Expect::Held(_) | Expect::Waits => Err("ended instead of remaining held or waiting".into()),
        };
    }
    let live = Key::Core(core::Key::Tasks(tasks::Key::Live(task)));
    let Some(Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) = store.rows.get(&live) else {
        return Ok(false);
    };
    match (&row.phase, expect) {
        (tasks::Phase::Held { why, source, .. }, Expect::Held(words)) => {
            let held_words = match source.as_deref() {
                Some(tasks::HoldSource::Preparation { words } | tasks::HoldSource::Account { words, .. }) => {
                    Some(words.as_ref())
                }
                _ => None,
            }
            .or_else(|| {
                let tasks::Hold::Placed { hold } = why else { return None };
                store.rows.values().find_map(|row| match row {
                    Record::Core(core::Record::Tasks(tasks::Stored::PlacedHold(row))) if row.number == *hold => {
                        Some(row.reason.as_ref())
                    }
                    _ => None,
                })
            });
            if held_words.is_some_and(|held| String::from_utf8_lossy(held).contains(words)) {
                Ok(true)
            } else {
                Err(format!("held without expected words {words:?}"))
            }
        }
        (tasks::Phase::Held { .. }, Expect::Ends | Expect::Untouched) => Err("held instead of ended".into()),
        (tasks::Phase::Held { .. } | tasks::Phase::Waiting, Expect::Waits) => Ok(true),
        (tasks::Phase::Active(_), Expect::Waits) => Ok(descendant_waits(store, task)),
        _ => Ok(false),
    }
}
