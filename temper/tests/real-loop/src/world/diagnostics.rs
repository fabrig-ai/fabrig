//! Bounded overdue context (temper/docs/design/domain/testing.md, 2.4;
//! skein/docs/foundation/testing-strategy.md, 2.10).
use skein_io::kernel::{Complete, Op, Submit};
use skein_lib::{Queue, Time, Token};
use skein_shell::Host;
use std::{collections::BTreeMap, fmt::Write};

/// The diagnostic list holds at most this many rows; tracking retains all live operations.
const ROWS: usize = 32;
// The stock review charter used by case/scripts.rs and the reference case defaults.
const REVIEW_CHARTER: u32 = 4;

pub(super) struct Operations {
    pending: BTreeMap<Token, Operation>,
    pub(super) now: Time,
    pub(super) started: Option<Time>,
    timers: [Option<Timer>; 2],
    step: Option<((bool, usize), Time)>,
}
impl Default for Operations {
    fn default() -> Self {
        Self { pending: BTreeMap::new(), now: Time::ZERO, started: None, timers: [None, None], step: None }
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
    rows: BTreeMap<u64, Task>,
}
struct Task {
    parent: Option<u64>,
    phase: &'static str,
    closing: Option<jig_domain_tasks::Stage>,
    reviewer: bool,
    procedure: bool,
    live: bool,
}
impl Tasks {
    pub(super) fn observe(&mut self, engine: &temper_engine_domain::Domain) {
        if self.revision == Some(engine.revision()) {
            return;
        }
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
            self.rows.insert(
                row.number,
                Task {
                    parent,
                    phase,
                    closing,
                    // The reference case's review charter is declared by its defaults.
                    reviewer: matches!(task.executor, jig_domain_tasks::Executor::Agent { charter: REVIEW_CHARTER }),
                    procedure: matches!(task.executor, jig_domain_tasks::Executor::Procedure { .. }),
                    live: true,
                },
            );
        }
        for (number, row) in &mut self.rows {
            if !row.live && row.phase != "ended" {
                // Keep a proven ending after its dependency stub is reclaimed.
                row.phase = if tasks.stub(*number).is_some() { "ended" } else { "absent" };
            }
        }
    }
    fn below(&self, mut task: u64, ancestor: u64) -> bool {
        // Requester depth cannot exceed the number of retained tasks.
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
        let mut delegates = false;
        let mut reviewing = false;
        let mut reviewed = false;
        let mut procedure = false;
        let mut live = false;
        for (child, row) in &self.rows {
            if self.below(*child, number) {
                delegates = true;
                live |= row.live;
                reviewing |= row.reviewer && row.live;
                reviewed |= row.reviewer && row.phase == "ended";
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
        for (number, task) in &self.rows {
            rows.add(format_args!(
                "task={number} state={} phase={} closing={:?}",
                self.state(*number, task),
                task.phase,
                if task.live { task.closing } else { None }
            ));
        }
        report.push_str("task ending substates (story progress inferred from requester topology):\n");
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
mod tests {
    use super::{Operation, Operations, ROWS, Rows, Task, Tasks};
    use crate::world::{Kind, Proc};
    use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
    use skein_lib::{Duration, Queue, Time, Token, Wall};
    use skein_shell::Host;
    use skein_world::{Referee, real::World};
    use std::collections::BTreeMap;

    const START: Time = Time::from_nanos(10);
    const NOW: Time = Time::from_nanos(30);

    fn tasks() -> Tasks {
        let mut rows = BTreeMap::new();
        for (number, parent, reviewer, procedure) in [
            (crate::case::A, None, false, false),
            (crate::case::DESIGN, Some(crate::case::A), false, true),
            (crate::case::REVIEW, Some(crate::case::DESIGN), true, false),
        ] {
            rows.insert(number, Task { parent, phase: "running", closing: None, reviewer, procedure, live: true });
        }
        Tasks { revision: None, rows }
    }

    #[test]
    fn forced_overdue_names_the_story_tasks_and_bounded_pending_work() {
        let _: World<Proc, crate::world::Judge> = World::new_controlled(|controls| {
            let mut judge = crate::world::cluster_judge(controls, &[], None, crate::PATIENCE);
            judge.tasks = tasks();
            let mut caller = crate::calls::Caller::new(
                vec![crate::calls::call(crate::calls::D1, crate::argv!["task", "show", "2", "--until", "ended"])],
                std::path::PathBuf::new(),
                Fd::new(0),
            );
            caller.released = true;
            let mut proc = Proc::new(Kind::Caller(Box::new(caller)));
            proc.iterate(START, Wall::EPOCH);
            proc.drain();
            proc.diagnostics.timers(Some(Time::from_nanos(100)), None);
            // Fake pending kernel operations; no descriptors are submitted to a ring.
            for token in 1..=ROWS {
                proc.diagnostics.pending.insert(
                    Token::new(u64::try_from(token).expect("small list")),
                    Operation { kind: "sync", since: START },
                );
            }
            let terminal = crate::terminal::compose(
                std::path::Path::new("terminal-fixture"),
                temper::open::target(
                    b"http://127.0.0.1:7100/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                )
                .expect("synthetic link"),
                Fd::new(3),
                Fd::new(0),
                Fd::new(1),
                Fd::new(2),
            );
            let mut child = Proc::new(Kind::Terminal(Box::new(terminal)));
            child.diagnostics.complete(&Queue::uncapped(), START);
            let mut procs = [proc, child];
            judge.patience.force_failed();
            let panic =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| judge.capture_overdue(NOW, &mut procs)))
                    .expect_err("failed overdue still panics");
            let message = panic.downcast_ref::<String>().expect("formatted failure");
            for expected in [
                "the referee failed the real loop before patience",
                "task=2 state=reviewing phase=running closing=None",
                "wait=story-step step=0 setup=false age_ns=20",
                "call=0 kind=ctl age_ns=20 tries=1",
                "timer=host until_ns=100 observed_age_ns=20",
                "outstanding total=36 shown=32 omitted=4",
                "operation=sync token=1 age_ns=20",
                "child-run=terminal observed_age_ns=Some(20)",
                "caller step=0 call=Some(0)",
                "forced failed deadline figures",
            ] {
                assert!(message.contains(expected), "missing {expected}: {message}");
            }
            assert_eq!(
                judge.overdue(NOW),
                Some(judge.describe_pending(NOW, &mut procs) + "forced failed deadline figures\n")
            );
            judge
        });
    }

    #[test]
    fn task_progress_keeps_each_ending_substate_and_closing_obligation() {
        let mut tasks = tasks();
        let review = tasks.rows.remove(&crate::case::REVIEW).expect("review fixture");
        assert_eq!(tasks.state(crate::case::A, &tasks.rows[&crate::case::A]), "delegated");
        tasks.rows.insert(crate::case::REVIEW, review);
        assert_eq!(tasks.state(crate::case::A, &tasks.rows[&crate::case::A]), "reviewing");
        let review = tasks.rows.get_mut(&crate::case::REVIEW).expect("review");
        review.live = false;
        review.phase = "ended";
        assert_eq!(tasks.state(crate::case::A, &tasks.rows[&crate::case::A]), "landing");
        tasks.rows.get_mut(&crate::case::DESIGN).expect("design").live = false;
        assert_eq!(tasks.state(crate::case::A, &tasks.rows[&crate::case::A]), "reporting");
        let a = tasks.rows.get_mut(&crate::case::A).expect("A");
        a.phase = "closing";
        a.closing = Some(jig_domain_tasks::Stage::Run { attempt: 7 });
        let mut report = String::new();
        tasks.describe(&mut report);
        assert!(report.contains("task=2 state=reporting phase=closing closing=Some(Run { attempt: 7 })"), "{report}");
        let a = tasks.rows.get_mut(&crate::case::A).expect("A");
        a.live = false;
        a.phase = "ended";
        report.clear();
        tasks.describe(&mut report);
        assert!(report.contains("task=2 state=ended phase=ended closing=None"), "{report}");
    }

    #[test]
    fn operation_completion_and_token_reuse_reset_only_the_new_age() {
        let mut operations = Operations::default();
        let mut submissions = Queue::uncapped();
        let mut completions = Queue::uncapped();
        let token = Token::new(1);
        operations.complete(&completions, START);
        submissions.push(Submit { op: token, kind: Op::Wait { pidfd: Fd::new(4), reap: false } });
        operations.submit(&submissions);
        operations.now = Time::from_nanos(20);
        operations.submit(&submissions);
        let mut rows = Rows::default();
        operations.describe(0, NOW, false, &mut rows);
        let mut report = String::new();
        rows.finish("outstanding", &mut report);
        assert!(report.contains("operation=wait token=1 age_ns=20"), "{report}");
        completions.push(Complete {
            op: token,
            kind: Op::Wait { pidfd: Fd::new(4), reap: false },
            result: Ok(Done::Exit(crate::Exit::Code(0))),
        });
        operations.complete(&completions, NOW);
        assert!(operations.pending.is_empty());
        operations.submit(&submissions);
        assert_eq!(operations.pending[&token].since, NOW);
    }

    #[test]
    fn the_list_reports_omissions_on_both_sides_of_its_cap() {
        for (count, expected) in [(ROWS, "total=32 shown=32 omitted=0"), (ROWS + 1, "total=33 shown=32 omitted=1")] {
            let mut rows = Rows::default();
            for _ in 0..count {
                rows.add(format_args!("pending"));
            }
            let mut report = String::new();
            rows.finish("outstanding", &mut report);
            assert!(report.contains(expected), "{report}");
            assert_eq!(report.lines().count(), ROWS + 1);
        }
    }

    #[test]
    fn passive_listeners_cannot_hide_an_ending_workers_sync() {
        let caller = || {
            Proc::new(Kind::Caller(Box::new(crate::calls::Caller::new(
                Vec::new(),
                std::path::PathBuf::new(),
                Fd::new(0),
            ))))
        };
        let mut procs = [caller(), caller()];
        for token in 1..=ROWS {
            procs[0].diagnostics.pending.insert(
                Token::new(u64::try_from(token).expect("small list")),
                Operation { kind: "accept", since: START },
            );
        }
        procs[1].diagnostics.pending.insert(Token::new(1), Operation { kind: "sync", since: START });
        let mut report = String::new();
        super::outstanding(&procs, NOW, &mut report);
        assert!(
            report.starts_with("outstanding total=33 shown=32 omitted=1\nproc=1 operation=sync token=1 age_ns=20\n"),
            "{report}"
        );
        assert_eq!(report.lines().count(), ROWS + 1);
    }

    #[test]
    fn timer_ages_follow_the_first_observation_of_each_deadline() {
        let mut operations = Operations { now: START, ..Operations::default() };
        let first = Some(NOW.saturating_add(Duration::from_nanos(1)));
        operations.timers(first, None);
        operations.now = NOW;
        operations.timers(first, None);
        assert_eq!(operations.timers[0].as_ref().expect("timer").since, START);
        operations.timers(Some(NOW), first);
        assert_eq!(operations.timers[0].as_ref().expect("timer").since, NOW);
        operations.timers(None, None);
        assert!(operations.timers.iter().all(Option::is_none));
    }
}
