//! Bounded overdue context (temper/docs/design/domain/testing.md, 2.4;
//! skein/docs/foundation/testing-strategy.md, 2.10).
mod terminals;
use skein_io::kernel::{Complete, Op, Submit};
use skein_lib::{Queue, Time, Token};
use skein_shell::Host;
use std::{collections::BTreeMap, fmt::Write};
use terminals::Terminals;

/// The diagnostic list holds at most this many rows; tracking retains all live operations.
const ROWS: usize = 32;
// The stock review charter used by case/scripts.rs and the reference case defaults.
const REVIEW_CHARTER: u32 = 4;

pub(super) struct Operations {
    pending: BTreeMap<Token, Operation>,
    pub(super) terminals: Terminals,
    pub(super) now: Time,
    pub(super) started: Option<Time>,
    timers: [Option<Timer>; 2],
    step: Option<((bool, usize), Time)>,
}
impl Default for Operations {
    fn default() -> Self {
        Self {
            pending: BTreeMap::new(),
            terminals: Terminals::default(),
            now: Time::ZERO,
            started: None,
            timers: [None, None],
            step: None,
        }
    }
}
struct Timer {
    until: Time,
    since: Time,
}
struct Operation {
    kind: &'static str,
    since: Time,
}
impl Operations {
    pub(super) fn step(&mut self, position: (bool, usize)) {
        if self.step.is_none_or(|(previous, _)| previous != position) {
            self.step = Some((position, self.now));
        }
    }
    pub(super) fn timers(&mut self, ordinary: Option<Time>, policy: Option<Time>) {
        for (kept, until) in self.timers.iter_mut().zip([ordinary, policy]) {
            if kept.as_ref().map(|timer| timer.until) != until {
                *kept = until.map(|until| Timer { until, since: self.now });
            }
        }
    }
    pub(super) fn context(&self, slot: usize, kind: &super::Kind, now: Time, out: &mut Rows) {
        use super::Kind;
        if let Kind::Caller(caller) = kind {
            if let Some(((setup, step), since)) = self.step {
                out.add(format_args!(
                    "proc={slot} wait=story-step step={step} setup={setup} age_ns={}",
                    now.saturating_since(since).as_nanos()
                ));
            }
            for (index, call) in caller.calls.iter().enumerate() {
                if call.exit.is_none() {
                    out.add(format_args!(
                        "proc={slot} call={index} kind=ctl age_ns={} tries={}",
                        now.saturating_since(call.spawned).as_nanos(),
                        call.tries
                    ));
                }
            }
        }
        let child = match kind {
            Kind::Agent(_) => Some("smith-agent"),
            Kind::Git(_) | Kind::GitRoute(_) => Some("git"),
            Kind::Claude(_) => Some("claude"),
            Kind::Relay(_) => Some("relay"),
            Kind::Ctl(_) => Some("ctl"),
            Kind::Terminal(_) => Some("terminal"),
            Kind::Llm(_) | Kind::Forge(_) | Kind::Issuer(_) | Kind::Service(_) | Kind::Caller(_) => None,
        };
        if let Some(child) = child
            && kind.exit().is_none()
        {
            let age = self.started.map(|since| now.saturating_since(since).as_nanos());
            out.add(format_args!("proc={slot} child-run={child} observed_age_ns={age:?}"));
        }
        for (timer, kind) in self.timers.iter().zip(["host", "policy"]) {
            if let Some(timer) = timer {
                out.add(format_args!(
                    "proc={slot} timer={kind} until_ns={} observed_age_ns={}",
                    timer.until.as_nanos(),
                    now.saturating_since(timer.since).as_nanos()
                ));
            }
        }
    }

    pub(super) fn complete(&mut self, completions: &Queue<Complete>, now: Time) {
        self.now = now;
        self.started.get_or_insert(now);
        for completion in completions {
            self.pending.remove(&completion.op);
        }
    }
    pub(super) fn submit(&mut self, submissions: &Queue<Submit>) {
        for submission in submissions {
            self.pending.entry(submission.op).or_insert(Operation { kind: kind(&submission.kind), since: self.now });
        }
    }
    pub(super) fn describe(&self, slot: usize, now: Time, passive: bool, out: &mut Rows) {
        for (token, operation) in &self.pending {
            if matches!(operation.kind, "accept" | "read-signal") != passive {
                continue;
            }
            out.add(format_args!(
                "proc={slot} operation={} token={} age_ns={}",
                operation.kind,
                token.raw(),
                now.saturating_since(operation.since).as_nanos()
            ));
        }
    }
}

/// Keep passive listeners from hiding an ending worker's file or child operations.
pub(super) fn outstanding(procs: &[super::Proc], now: Time, report: &mut String) {
    let mut rows = Rows::default();
    for (slot, proc) in procs.iter().enumerate() {
        proc.diagnostics.context(slot, &proc.kind, now, &mut rows);
    }
    for passive in [false, true] {
        for (slot, proc) in procs.iter().enumerate() {
            proc.diagnostics.describe(slot, now, passive, &mut rows);
        }
    }
    rows.finish("outstanding", report);
}

#[derive(Default)]
pub(super) struct Rows {
    text: String,
    total: usize,
}
impl Rows {
    pub(super) fn add(&mut self, row: std::fmt::Arguments<'_>) {
        if self.total < ROWS {
            writeln!(self.text, "{row}").expect("string writer");
        }
        self.total += 1;
    }
    pub(super) fn finish(self, label: &str, report: &mut String) {
        writeln!(
            report,
            "{label} total={} shown={} omitted={}",
            self.total,
            self.total.min(ROWS),
            self.total.saturating_sub(ROWS)
        )
        .expect("string writer");
        report.push_str(&self.text);
    }
}

fn kind(op: &Op) -> &'static str {
    match op {
        Op::Socket { .. } => "socket",
        Op::Bind { .. } => "bind",
        Op::Listen { .. } => "listen",
        Op::Accept { .. } => "accept",
        Op::Connect { .. } => "connect",
        Op::Recv { .. } => "recv",
        Op::Send { .. } => "send",
        Op::Shutdown { .. } => "shutdown",
        Op::Close { .. } => "close",
        Op::Open { .. } => "open",
        Op::Read { .. } => "read",
        Op::Write { .. } => "write",
        Op::Append { .. } => "append",
        Op::Truncate { .. } => "truncate",
        Op::Sync { .. } => "sync",
        Op::Lock { .. } => "lock",
        Op::Stat { .. } => "stat",
        Op::Rename { .. } => "rename",
        Op::Remove { .. } => "remove",
        Op::MakeDirectory { .. } => "make-directory",
        Op::List { .. } => "list",
        Op::Spawn { .. } => "spawn",
        Op::Wait { .. } => "wait",
        Op::Signal { .. } => "signal",
        Op::Usage => "usage",
        Op::Window { .. } => "window",
        Op::ReadSignal { .. } => "read-signal",
        Op::PipeRead { .. } => "pipe-read",
        Op::PipeWrite { .. } => "pipe-write",
        Op::Cancel { .. } => "cancel",
    }
}

#[derive(Default)]
pub(super) struct Tasks {
    revision: Option<u64>,
    terminal_revision: u64,
    rows: BTreeMap<u64, Task>,
    ended: u64,
    reclaimed: u64,
}
#[derive(Default)]
struct Summary {
    completed: u64,
    reviewers: u64,
}
struct Task {
    parent: Option<u64>,
    phase: &'static str,
    closing: Option<jig_domain_tasks::Stage>,
    reviewer: bool,
    procedure: bool,
    live: bool,
    completed: Summary,
}
impl Tasks {
    pub(super) fn observe(&mut self, engine: &temper_engine_domain::Domain, terminals: &Terminals) {
        if self.revision != Some(engine.revision()) {
            self.revision = Some(engine.revision());
            for task in self.rows.values_mut() {
                task.live = false;
            }
            let tasks = engine.tasks();
            for row in &tasks.view_tasks() {
                let task = tasks.task(row.number).expect("live view row");
                let parent = match task.requester {
                    jig_domain_tasks::Party::Task(parent) => Some(parent),
                    jig_domain_tasks::Party::Person(_) | jig_domain_tasks::Party::Deployment { .. } => None,
                };
                let (phase, closing) = phase(&task.phase);
                let kept = self.rows.entry(row.number).or_insert(Task {
                    parent,
                    phase,
                    closing,
                    reviewer: false,
                    procedure: false,
                    live: true,
                    completed: Summary::default(),
                });
                kept.parent = parent;
                kept.phase = phase;
                kept.closing = closing;
                kept.reviewer = matches!(task.executor, jig_domain_tasks::Executor::Agent { charter: REVIEW_CHARTER });
                kept.procedure = matches!(task.executor, jig_domain_tasks::Executor::Procedure { .. });
                kept.live = true;
            }
            for row in self.rows.values_mut() {
                if !row.live && row.phase != "ended" {
                    row.phase = "absent";
                }
            }
        }
        self.observe_terminals(terminals);
    }
    fn observe_terminals(&mut self, terminals: &Terminals) {
        if self.terminal_revision != terminals.revision {
            self.terminal_revision = terminals.revision;
            for number in &terminals.ended {
                self.ended = self.ended.saturating_add(1);
                if let Some(row) = self.rows.get_mut(number) {
                    row.phase = "ended";
                    row.closing = None;
                    row.live = false;
                }
            }
            self.reclaim();
        }
    }
    fn reclaim(&mut self) {
        // Each pass removes at least one ended leaf, or stops; depth is at most
        // the retained task count. Summaries replace completed subtrees.
        for _ in 0..self.rows.len() {
            let leaves: Vec<_> = self
                .rows
                .iter()
                .filter_map(|(number, row)| {
                    (row.phase == "ended" && !self.rows.values().any(|child| child.parent == Some(*number)))
                        .then_some(*number)
                })
                .collect();
            if leaves.is_empty() {
                break;
            }
            for number in leaves {
                let row = self.rows.remove(&number).expect("ended leaf");
                self.reclaimed = self.reclaimed.saturating_add(1);
                if let Some(parent) = row.parent.and_then(|parent| self.rows.get_mut(&parent)) {
                    parent.completed.completed =
                        parent.completed.completed.saturating_add(row.completed.completed).saturating_add(1);
                    parent.completed.reviewers = parent
                        .completed
                        .reviewers
                        .saturating_add(row.completed.reviewers)
                        .saturating_add(u64::from(row.reviewer));
                }
            }
        }
    }
    fn below(&self, mut task: u64, ancestor: u64) -> bool {
        for _ in 0..self.rows.len() {
            let Some(parent) = self.rows.get(&task).and_then(|row| row.parent) else { return false };
            if parent == ancestor {
                return true;
            }
            task = parent;
        }
        false
    }
    fn state(&self, number: u64, task: &Task) -> &'static str {
        if !task.live {
            return task.phase;
        }
        let mut delegates = task.completed.completed != 0;
        let mut reviewing = false;
        let mut reviewed = task.completed.reviewers != 0;
        let mut procedure = false;
        let mut live = false;
        for (child, row) in &self.rows {
            if self.below(*child, number) {
                delegates = true;
                live |= row.live;
                reviewing |= row.reviewer && row.live;
                reviewed |= row.completed.reviewers != 0 || row.reviewer && row.phase == "ended";
                procedure |= row.procedure && row.live;
            }
        }
        if reviewing {
            "reviewing"
        } else if reviewed && procedure {
            "landing"
        } else if live {
            "delegated"
        } else if delegates {
            "reporting"
        } else {
            task.phase
        }
    }
    pub(super) fn describe(&self, report: &mut String) {
        let mut rows = Rows::default();
        // Live blockers precede missing or ending rows, regardless of task number.
        for live in [true, false] {
            for (number, task) in &self.rows {
                if task.live == live {
                    rows.add(format_args!(
                        "task={number} state={} phase={} closing={:?} completed_below={} reviewed_below={}",
                        self.state(*number, task),
                        task.phase,
                        if task.live { task.closing } else { None },
                        task.completed.completed,
                        task.completed.reviewers
                    ));
                }
            }
        }
        report.push_str("task ending substates (story progress inferred from requester topology):\n");
        writeln!(
            report,
            "task history state=ended count={} reclaimed={} retained={}",
            self.ended,
            self.reclaimed,
            self.rows.len()
        )
        .expect("string writer");
        rows.finish("task substates", report);
    }
}
fn phase(phase: &jig_domain_tasks::Phase) -> (&'static str, Option<jig_domain_tasks::Stage>) {
    use jig_domain_tasks::{Phase, Was};
    match phase {
        Phase::Waiting => ("waiting", None),
        Phase::Active(active) => (active_phase(*active), None),
        Phase::Closing(closing) => ("closing", Some(closing.stage)),
        Phase::Held { was, .. } => (
            "held",
            match was {
                Was::Closing(closing) => Some(closing.stage),
                Was::Waiting | Was::Active(_) => None,
            },
        ),
        Phase::Ended(_) => ("ended", None),
    }
}
fn active_phase(active: jig_domain_tasks::Active) -> &'static str {
    use jig_domain_tasks::Active;
    match active {
        Active::Idle => "idle",
        Active::Due => "due",
        Active::Preparing => "preparing",
        Active::Claimed { .. } => "claimed",
        Active::Running { .. } => "running",
        Active::BackingOff { .. } => "backing-off",
    }
}

#[cfg(test)]
mod tests;
