//! Real-clock harness deadlines (skein/docs/design/testing/service-tiers.md, 1;
//! skein/docs/foundation/testing-strategy.md, 2.8 and 2.10).

use alloc::sync::Arc;
#[cfg(target_os = "linux")]
use core::cell::Cell;
use core::fmt;
use core::fmt::Write;
#[cfg(target_os = "linux")]
use std::path::{Path, PathBuf};

mod process;
pub(crate) mod testing;
use process::{Boot, Proc, Processes, SampleFiles};

use skein_lib::{Duration, Time};
use skein_shell::Clock;

use crate::end_to_end::Binary;

/// A harness deadline on the real clock, armed before a wait and classified
/// once it passes with the wait unanswered: starved when the thread's or the
/// tree's wait is at least half the patience, failed when neither does or
/// neither is available. Assertions, refusals, protocol errors and bad exits
/// fail whatever the load.
#[derive(Debug)]
pub struct Deadline {
    what: &'static str,
    patience: Duration,
    start: Time,
    until: Time,
    thread: Option<Sched>,
    trees: Vec<TreeWatch>,
    boot: Option<Boot>,
}

#[derive(Debug)]
enum TreeWatch {
    #[cfg(target_os = "linux")]
    Cgroup {
        path: PathBuf,
        start: Option<u64>,
        last: Cell<Option<u64>>,
    },
    Threads(ProcessWatch),
}

/// Scheduler origin for deadlines that begin together on this thread.
#[derive(Clone, Copy, Debug)]
pub struct DeadlineOrigin {
    start: Time,
    thread: Option<Sched>,
    boot: Option<Boot>,
}

impl DeadlineOrigin {
    #[must_use]
    pub fn capture() -> Self {
        Self { start: Clock::new().now().now, thread: thread_sched(), boot: Boot::read() }
    }

    /// Begins a deadline with its own patience at this origin, without reading
    /// the clock or scheduler again. Use only for waits starting here.
    #[must_use]
    pub fn arm(self, what: &'static str, patience: Duration) -> Deadline {
        let mut deadline = Deadline::at(what, patience, self.start);
        deadline.thread = self.thread;
        deadline.boot = self.boot;
        deadline
    }
}

/// One snapshot of the caller's owned process roots and their descendants.
/// Attach it only to accounts that would sample these roots at this point.
#[derive(Clone, Debug)]
pub struct ProcessWatch(Arc<Processes>);

impl ProcessWatch {
    #[must_use]
    pub fn capture(pids: &[u32]) -> Self {
        Self::capture_with(pids, &Proc::Live)
    }

    fn capture_with(pids: &[u32], proc: &Proc) -> Self {
        Self(Arc::new(Processes::arm(pids, proc)))
    }
}

/// A run's reusable procfs descriptors for arming process watches on Linux.
/// Each capture reads new figures and checks ownership and identity anew.
#[derive(Debug, Default)]
pub struct ProcessSampler {
    files: SampleFiles,
}

impl ProcessSampler {
    #[must_use]
    pub fn capture(&mut self, pids: &[u32]) -> ProcessWatch {
        self.capture_with(pids, &Proc::Live)
    }

    fn capture_with(&mut self, pids: &[u32], proc: &Proc) -> ProcessWatch {
        let files = cfg!(target_os = "linux").then_some(&mut self.files);
        ProcessWatch(Arc::new(Processes::arm_with(pids, proc, files)))
    }
}

/// A passed deadline, by its scheduler accounting.
#[derive(Debug)]
pub enum Expired {
    /// The thread or owned tree was runnable for at least half the patience.
    Starved(Starved),
    /// Neither figure reached half the patience, or accounting was unavailable.
    Failed(Figures),
}

/// Scheduler figures since arming; `None` means no usable kernel accounting.
#[derive(Debug)]
pub struct Figures {
    pub what: &'static str,
    pub patience: Duration,
    pub elapsed: Duration,
    pub thread_waited: Option<Duration>,
    pub thread_cpu: Option<Duration>,
    /// The largest runnable wait across all watched sources: each cgroup's
    /// `some` stall or owned descendant thread's delta. Waits are never summed.
    pub tree_waited: Option<Duration>,
}

impl fmt::Display for Figures {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            out,
            "{}: patience {} ns; elapsed {} ns; thread runnable wait {}; thread CPU {}; tree runnable wait {}",
            self.what,
            self.patience.as_nanos(),
            self.elapsed.as_nanos(),
            figure(self.thread_waited),
            figure(self.thread_cpu),
            figure(self.tree_waited)
        )
    }
}

fn figure(value: Option<Duration>) -> String {
    match value {
        Some(value) => format!("{} ns", value.as_nanos()),
        None => String::from("UNAVAILABLE (ns)"),
    }
}

fn millis(value: Option<Duration>) -> String {
    match value {
        Some(value) => value.as_nanos().div_euclid(1_000_000).to_string(),
        None => String::from("UNAVAILABLE"),
    }
}

fn waited_half(patience: Duration, waited: Option<Duration>) -> bool {
    match waited {
        Some(waited) => starved(patience, waited.as_nanos()),
        None => false,
    }
}

/// A deadline whose unanswered wait was starved of CPU.
#[derive(Debug)]
pub struct Starved {
    figures: Figures,
    tree_kind: &'static str,
}

/// Whether a starved wait ends its test as a pass here: `TIERS_ONE_PROCESS` or
/// `NEXTEST` is set.
#[must_use]
#[expect(
    clippy::disallowed_methods,
    reason = "skein's round: reads TIERS_ONE_PROCESS and NEXTEST to select the starvation verdict"
)]
pub fn starved_passes() -> bool {
    std::env::var_os("TIERS_ONE_PROCESS").is_some() || std::env::var_os("NEXTEST").is_some()
}

/// Ends the running test with a `skein-world: starved:` line a helper wrote,
/// as `Starved::pass` ends it; elsewhere fails with the line. The exit skips
/// every drop, so the caller ends what it owns first.
#[expect(clippy::exit, reason = "testing/service-tiers.md, 1 ends a starved pass with exit 0")]
#[expect(clippy::print_stderr, reason = "testing/service-tiers.md, 1 requires one starvation verdict on stderr")]
pub fn pass_starved(line: &str) -> ! {
    if starved_passes() {
        eprintln!("{line}");
        std::process::exit(0);
    }
    crate::fail(line);
}

impl Starved {
    #[must_use]
    pub fn figures(&self) -> &Figures {
        &self.figures
    }

    /// Ends a test as a starved pass: exits 0 in a one-process run or under
    /// nextest, and elsewhere fails with the figures. The exit skips every
    /// drop, so the caller ends what it owns first.
    pub fn pass(self) -> ! {
        if !starved_passes() {
            crate::fail(&format!("skein-world: starved: {}", self.figures));
        }
        pass_starved(&self.line());
    }

    /// Ends a helper process whose test reads its stderr: the line and exit 0
    /// where `starved_passes()`; elsewhere fails with the figures.
    pub fn exit(self) -> ! {
        self.pass();
    }

    fn line(&self) -> String {
        let figures = &self.figures;
        let mut line = format!(
            "skein-world: starved: waited {} ms runnable, used {} ms of CPU, patience {} ms, by {}",
            millis(figures.thread_waited),
            millis(figures.thread_cpu),
            figures.patience.as_nanos().div_euclid(1_000_000),
            figures.what
        );
        if waited_half(figures.patience, figures.tree_waited) {
            write!(&mut line, "; tree waited {} ms ({})", millis(figures.tree_waited), self.tree_kind)
                .expect("String formatting");
        }
        line
    }
}

impl Deadline {
    /// Arms `patience` from now, measuring the calling thread. `what` names the wait.
    #[must_use]
    pub fn arm(what: &'static str, patience: Duration) -> Self {
        DeadlineOrigin::capture().arm(what, patience)
    }

    fn at(what: &'static str, patience: Duration, start: Time) -> Self {
        Self {
            what,
            patience,
            start,
            until: start.saturating_add(patience),
            thread: None,
            trees: Vec::new(),
            boot: None,
        }
    }

    /// Adds a binary's owned tree: its cgroup, else its processes and descendants.
    /// Earlier watches remain measured; this keeps no borrow of the binary.
    pub fn watch(&mut self, binary: &Binary) {
        if let Some(tree) = binary.keeper() {
            self.watch_tree(tree);
        }
    }

    /// Adds a process the caller spawned and owns, and its descendants.
    /// The test's own process ID covers its helper threads and children.
    /// Threads born after arming count from zero when their birth is known;
    /// unreadable or vanished threads contribute nothing.
    pub fn watch_process(&mut self, pid: u32) {
        self.watch_process_with(pid, &Proc::Live);
    }

    fn watch_process_with(&mut self, pid: u32, proc: &Proc) {
        let threads = self.trees.iter().filter_map(|tree| tree.processes());
        if threads.clone().any(|watch| watch.has_root(pid))
            && let Some(start) = proc.start_of(pid)
            && threads.clone().any(|watch| watch.watches(pid, start))
        {
            return;
        }
        self.watch_snapshot(&ProcessWatch::capture_with(&[pid], proc));
    }

    /// Adds owned roots sampled at this point, without reading procfs again.
    pub fn watch_snapshot(&mut self, watch: &ProcessWatch) {
        self.trees.push(TreeWatch::Threads(watch.clone()));
    }

    pub(crate) fn watch_tree(&mut self, tree: &crate::tree::Tree) {
        tree.watch(self);
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn watch_cgroup(&mut self, path: PathBuf) {
        let start = pressure(&path);
        self.trees.push(TreeWatch::Cgroup { path, start, last: Cell::new(start) });
    }

    pub(crate) fn watch_threads(&mut self, pids: &[u32]) {
        self.watch_snapshot(&ProcessWatch::capture(pids));
    }

    /// Whether the deadline has passed.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.sample_pressure();
        Clock::new().now().now >= self.until
    }

    /// Time remaining, zero once passed.
    #[must_use]
    pub fn remaining(&self) -> Duration {
        self.sample_pressure();
        self.until.saturating_since(Clock::new().now().now)
    }

    /// The deadline on skein-shell's monotonic clock, for `Wait::Until`.
    #[must_use]
    pub fn until(&self) -> Time {
        self.until
    }

    /// Classifies a passed deadline. Call only for an unanswered wait.
    #[must_use]
    pub fn expired(&self) -> Expired {
        let now = Clock::new().now().now;
        assert!(now >= self.until, "only a passed deadline is classified");
        let end = thread_sched();
        self.sample_pressure();
        self.classify_at(self.what, self.patience, now, end)
    }

    /// Classifies an unanswered wait of `bound` that has passed. The patience
    /// is at least the time since this account was armed, and never less than `bound`.
    #[must_use]
    pub fn expired_for(&self, what: &'static str, bound: Duration) -> Expired {
        let now = Clock::new().now().now;
        let end = thread_sched();
        self.sample_pressure();
        self.classify_for_at(what, bound, now, end)
    }

    fn classify_for_at(&self, what: &'static str, bound: Duration, now: Time, end: Option<Sched>) -> Expired {
        self.classify_at(what, bound.max(now.saturating_since(self.start)), now, end)
    }

    /// Classifies another unanswered deadline using this wait's time and scheduler origins.
    pub(crate) fn expired_at(&self, what: &'static str, until: Time) -> Expired {
        let now = Clock::new().now().now;
        let patience = self.referee_patience(until, now);
        let end = thread_sched();
        self.sample_pressure();
        self.classify_at(what, patience, now, end)
    }

    fn referee_patience(&self, until: Time, now: Time) -> Duration {
        assert!(now >= until, "only a passed deadline is classified");
        until.saturating_since(self.start)
    }

    fn classify_at(&self, what: &'static str, patience: Duration, now: Time, end: Option<Sched>) -> Expired {
        self.classify_with(what, patience, now, end, &Proc::Live)
    }

    fn classify_with(
        &self,
        what: &'static str,
        patience: Duration,
        now: Time,
        end: Option<Sched>,
        proc: &Proc,
    ) -> Expired {
        let sched = sched_elapsed(self.thread, end);
        let (tree_waited, kind) = self.watched_wait(proc);
        let (thread_cpu, thread_waited) = match sched {
            Some(sched) => (Some(Duration::from_nanos(sched.cpu)), sched.waited.map(Duration::from_nanos)),
            None => (None, None),
        };
        classify(
            Figures {
                what,
                patience,
                elapsed: now.saturating_since(self.start),
                thread_waited,
                thread_cpu,
                tree_waited,
            },
            kind,
        )
    }

    pub(crate) fn thread_start(&self) -> Option<Sched> {
        self.thread
    }

    pub(crate) fn start(&self) -> Time {
        self.start
    }

    fn watched_wait(&self, proc: &Proc) -> (Option<Duration>, &'static str) {
        let mut tree_waited = None;
        let mut kind = "threads";
        for tree in &self.trees {
            let (waited, source_kind) = tree.waited(proc, self.boot);
            if waited.is_some() && (tree_waited.is_none() || waited > tree_waited) {
                tree_waited = waited;
                kind = source_kind;
            }
        }
        (tree_waited, kind)
    }

    fn sample_pressure(&self) {
        #[cfg(target_os = "linux")]
        self.sample_pressure_with(pressure);
    }

    #[cfg(target_os = "linux")]
    fn sample_pressure_with(&self, read: fn(&Path) -> Option<u64>) {
        for tree in &self.trees {
            if let TreeWatch::Cgroup { path, last, .. } = tree
                && let Some(value) = read(path)
            {
                last.set(Some(value));
            }
        }
    }
}

fn classify(figures: Figures, tree_kind: &'static str) -> Expired {
    if waited_half(figures.patience, figures.thread_waited) || waited_half(figures.patience, figures.tree_waited) {
        Expired::Starved(Starved { figures, tree_kind })
    } else {
        Expired::Failed(figures)
    }
}

pub(crate) fn starved(patience: Duration, waited: u64) -> bool {
    // Rounding up preserves the threshold without multiplication or overflow.
    waited >= patience.as_nanos().div_ceil(2)
}

impl TreeWatch {
    fn processes(&self) -> Option<&Processes> {
        match self {
            #[cfg(target_os = "linux")]
            Self::Cgroup { .. } => None,
            Self::Threads(watch) => Some(&watch.0),
        }
    }

    fn waited(&self, proc: &Proc, boot: Option<Boot>) -> (Option<Duration>, &'static str) {
        match self {
            #[cfg(target_os = "linux")]
            Self::Cgroup { start, last, .. } => (
                start.zip(last.get()).and_then(|(start, last)| last.checked_sub(start)).map(Duration::from_nanos),
                "cgroup",
            ),
            Self::Threads(watch) => (watch.0.waited(proc, boot), "threads"),
        }
    }
}

fn largest_wait(largest: Option<u64>, delta: Option<Sched>) -> Option<u64> {
    match delta.and_then(|sched| sched.waited) {
        Some(waited) => Some(largest.unwrap_or(0).max(waited)),
        None => largest,
    }
}

#[cfg(target_os = "linux")]
fn pressure(path: &Path) -> Option<u64> {
    parse_pressure(&std::fs::read_to_string(path).ok()?)
}

#[cfg(any(target_os = "linux", test))]
fn parse_pressure(source: &str) -> Option<u64> {
    for line in source.lines() {
        let mut fields = line.split_whitespace();
        if fields.next() == Some("some") {
            for field in fields {
                if let Some(value) = field.strip_prefix("total=") {
                    let micros = value.parse::<u64>().ok()?;
                    return micros.checked_mul(1000);
                }
            }
        }
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Sched {
    pub(crate) cpu: u64,
    pub(crate) waited: Option<u64>,
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn thread_sched() -> Option<Sched> {
    skein_shell::keeper::thread_cpu().map(|cpu| Sched { cpu: cpu.as_nanos(), waited: None })
}

#[cfg(target_os = "linux")]
pub(crate) fn thread_sched() -> Option<Sched> {
    // Linux scheduler/sched-stats documents runtime and runnable wait in nanoseconds;
    // thread-self selects this thread's scheduler accounting.
    let source = std::fs::read_to_string("/proc/thread-self/schedstat").ok()?;
    parse_schedstat(&source)
}

fn parse_schedstat(source: &str) -> Option<Sched> {
    let mut fields = source.split_whitespace();
    let runtime = fields.next()?.parse::<u64>().ok()?;
    let waited = fields.next()?.parse::<u64>().ok()?;
    let _slices = fields.next()?.parse::<u64>().ok()?;
    // Disabled accounting emits zeros; a saturated counter has no usable delta.
    if fields.next().is_some() || runtime == 0 || runtime == u64::MAX || waited == u64::MAX {
        return None;
    }
    Some(Sched { cpu: runtime, waited: Some(waited) })
}

pub(crate) fn sched_elapsed(start: Option<Sched>, end: Option<Sched>) -> Option<Sched> {
    let (start, end) = start.zip(end)?;
    let waited = match start.waited.zip(end.waited) {
        Some((start, end)) => Some(end.checked_sub(start)?),
        None => None,
    };
    Some(Sched { cpu: end.cpu.checked_sub(start.cpu)?, waited })
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    use super::{Cell, PathBuf, TreeWatch};
    use super::{
        Deadline, Duration, Expired, Figures, Sched, Time, classify, largest_wait, parse_pressure, parse_schedstat,
        sched_elapsed, starved,
    };

    #[test]
    fn step_expiry_uses_the_longer_of_its_bound_and_account_age() {
        let start = Time::from_nanos(1);
        let mut deadline = Deadline::at("browser step", Duration::from_secs(40), start);
        deadline.thread = Some(Sched { cpu: 0, waited: Some(0) });
        for (age, bound, waited_ms, starved) in [
            (10, 4, Some(6000), true),
            (10, 4, Some(4000), false),
            (2, 10, Some(6000), true),
            (2, 10, Some(4900), false),
            (2, 10, None, false),
        ] {
            let now = start.saturating_add(Duration::from_secs(age));
            let end = waited_ms.map(|ms| Sched { cpu: 0, waited: Some(Duration::from_millis(ms).as_nanos()) });
            let expired = deadline.classify_for_at("step expiry", Duration::from_secs(bound), now, end);
            let (is_starved, figures) = match &expired {
                Expired::Starved(verdict) => (true, verdict.figures()),
                Expired::Failed(figures) => (false, figures),
            };
            assert_eq!(is_starved, starved, "age={age}, bound={bound}, waited_ms={waited_ms:?}");
            assert_eq!(figures.patience, Duration::from_secs(10));
        }
    }

    #[test]
    fn referee_patience_is_its_deadline_since_the_runs_start() {
        let deadline = Deadline::at("the real loop", Duration::from_secs(1), Time::from_nanos(7));
        let now = Time::from_nanos(207);
        let patience = deadline.referee_patience(Time::from_nanos(107), now);
        let Expired::Failed(figures) = deadline.classify_at("the referee's expectation", patience, now, None) else {
            crate::fail("unavailable accounting fails");
        };
        assert_eq!(figures.what, "the referee's expectation");
        assert_eq!(figures.patience, Duration::from_nanos(100));
        assert!(figures.elapsed >= figures.patience);
        assert_eq!(deadline.start(), Time::from_nanos(7));
    }

    #[test]
    fn referee_classification_uses_the_runs_scheduler_origin_and_half_its_own_patience() {
        let deadline = Deadline {
            what: "the real loop",
            patience: Duration::from_nanos(1000),
            start: Time::from_nanos(7),
            until: Time::from_nanos(1007),
            thread: Some(Sched { cpu: 100, waited: Some(200) }),
            trees: Vec::new(),
            boot: None,
        };
        for (waited, expected) in [(49, false), (50, true), (51, true)] {
            let expired = deadline.classify_at(
                "the referee's expectation",
                Duration::from_nanos(100),
                Time::from_nanos(207),
                Some(Sched { cpu: 130, waited: Some(200_u64.checked_add(waited).expect("tiny counters")) }),
            );
            let (is_starved, figures) = match &expired {
                Expired::Starved(starved) => (true, starved.figures()),
                Expired::Failed(figures) => (false, figures),
            };
            assert_eq!(is_starved, expected, "waited={waited}");
            assert_eq!(figures.patience, Duration::from_nanos(100));
            assert_eq!(figures.elapsed, Duration::from_nanos(200));
            assert_eq!(figures.thread_cpu, Some(Duration::from_nanos(30)));
            assert_eq!(figures.thread_waited, Some(Duration::from_nanos(waited)));
        }
    }

    #[test]
    fn thread_clock_cpu_reaches_both_findings_but_cannot_prove_starvation() {
        let deadline = Deadline {
            what: "the real loop",
            patience: Duration::from_nanos(100),
            start: Time::from_nanos(7),
            until: Time::from_nanos(107),
            thread: Some(Sched { cpu: 10, waited: None }),
            trees: Vec::new(),
            boot: None,
        };
        let end = Some(Sched { cpu: 1010, waited: None });
        for what in ["the real loop", "the referee's expectation"] {
            let Expired::Failed(figures) =
                deadline.classify_at(what, Duration::from_nanos(100), Time::from_nanos(207), end)
            else {
                crate::fail("CPU alone cannot prove starvation");
            };
            assert_eq!(figures.thread_cpu, Some(Duration::from_nanos(1000)));
            assert_eq!(figures.thread_waited, None);
            assert_eq!(figures.tree_waited, None);
            assert!(figures.to_string().contains("thread CPU 1000 ns"));
        }
        assert_eq!(sched_elapsed(deadline.thread, None), None);
        assert_eq!(sched_elapsed(None, end), None);
        assert_eq!(sched_elapsed(end, deadline.thread), None);
    }

    #[test]
    fn starvation_includes_exactly_half_the_patience_waiting() {
        let patience = Duration::from_nanos(100);
        assert!(!starved(patience, 49));
        assert!(starved(patience, 50));
    }

    #[test]
    fn starvation_includes_a_run_with_much_cpu_and_half_the_patience_waiting() {
        let patience = Duration::from_nanos(100);
        let elapsed =
            sched_elapsed(Some(Sched { cpu: 1, waited: Some(1) }), Some(Sched { cpu: 1001, waited: Some(51) }))
                .expect("increasing scheduler counters");
        assert_eq!(elapsed.cpu, 1000);
        assert!(starved(patience, elapsed.waited.expect("schedstat includes runnable wait")));
    }

    #[test]
    fn starvation_threshold_preserves_fractional_nanoseconds() {
        let patience = Duration::from_nanos(101);
        assert!(!starved(patience, 50));
        assert!(starved(patience, 51));
    }

    #[test]
    fn starvation_handles_zero_and_maximum_counters_without_overflow() {
        assert!(starved(Duration::ZERO, 0));
        assert!(!starved(Duration::from_nanos(100), 0));
        let patience = Duration::from_nanos(u64::MAX);
        assert!(starved(patience, u64::MAX));
        assert!(starved(patience, u64::MAX.div_ceil(2)));
        assert!(!starved(patience, u64::MAX.div_euclid(2)));
    }

    #[test]
    fn starvation_requires_accounting_at_both_ends() {
        let start = Sched { cpu: 1, waited: Some(1) };
        let end = Sched { cpu: 2, waited: Some(101) };
        for (start, end) in [(None, Some(end)), (Some(start), None), (None, None)] {
            let elapsed = sched_elapsed(start, end);
            assert_eq!(elapsed, None);
            assert!(!elapsed.is_some_and(|sched| starved(
                Duration::from_nanos(100),
                sched.waited.expect("schedstat includes runnable wait")
            )));
        }
        assert_eq!(sched_elapsed(Some(start), Some(end)), Some(Sched { cpu: 1, waited: Some(100) }));
        assert_eq!(sched_elapsed(Some(end), Some(start)), None);
        assert_eq!(sched_elapsed(Some(start), Some(Sched { cpu: 2, waited: Some(0) })), None);
    }

    #[test]
    fn classification_uses_either_wait_and_never_adds_them() {
        for (thread, tree, expected) in [
            (Some(49), None, false),
            (Some(50), None, true),
            (Some(51), None, true),
            (None, Some(49), false),
            (None, Some(50), true),
            (None, Some(51), true),
            (Some(49), Some(49), false),
            (Some(50), Some(50), true),
            (Some(50), Some(1), true),
            (Some(1), Some(50), true),
            (Some(0), Some(0), false),
            (None, None, false),
        ] {
            let figures = Figures {
                what: "the classifier",
                patience: Duration::from_nanos(100),
                elapsed: Duration::from_nanos(100),
                thread_waited: thread.map(Duration::from_nanos),
                thread_cpu: Some(Duration::from_nanos(1000)),
                tree_waited: tree.map(Duration::from_nanos),
            };
            let actual = match classify(figures, "threads") {
                Expired::Starved(_) => true,
                Expired::Failed(_) => false,
            };
            assert_eq!(actual, expected, "thread={thread:?}, tree={tree:?}");
        }
    }

    #[test]
    fn schedstat_parsing_keeps_zero_wait_but_refuses_unusable_counters() {
        assert_eq!(parse_schedstat("123 456 7\n"), Some(Sched { cpu: 123, waited: Some(456) }));
        assert_eq!(parse_schedstat("1 0 1"), Some(Sched { cpu: 1, waited: Some(0) }));
        for source in [
            "0 0 0",
            "0 7 1",
            "1 2",
            "",
            "one 2 3",
            "1 2 three",
            "1 2 3 4",
            "18446744073709551615 1 2",
            "1 18446744073709551615 2",
            "18446744073709551616 1 2",
        ] {
            assert_eq!(parse_schedstat(source), None, "{source}");
        }
    }

    #[test]
    fn pressure_parsing_uses_some_microseconds_and_preserves_a_zero_baseline() {
        assert_eq!(
            parse_pressure("some avg10=0.00 avg60=0.00 avg300=0.00 total=123\nfull avg10=0.00 total=999\n"),
            Some(123_000)
        );
        assert_eq!(parse_pressure("some avg10=0.00 total=0"), Some(0));
        for source in [
            "",
            "full total=123",
            "some avg10=0.00",
            "some total=bad",
            "some total=-1",
            "some total=18446744073709551615",
            "some total=18446744073709551616",
            "some total=18446744073709552",
        ] {
            assert_eq!(parse_pressure(source), None, "{source}");
        }
    }

    #[test]
    fn owned_threads_use_the_largest_usable_delta_not_a_sum() {
        let start = Some(Sched { cpu: 1, waited: Some(10) });
        let mut largest = None;
        for end in [Some(Sched { cpu: 2, waited: Some(40) }), None, Some(Sched { cpu: 2, waited: Some(30) })] {
            largest = largest_wait(largest, sched_elapsed(start, end));
        }
        assert_eq!(largest, Some(30));
        assert_eq!(largest_wait(None, sched_elapsed(start, None)), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_gone_cgroup_keeps_its_last_pressure_figure() {
        let now = Time::ZERO;
        let mut deadline = Deadline::at("the gone cgroup", Duration::ZERO, now);
        deadline.trees.push(TreeWatch::Cgroup {
            path: PathBuf::from("missing/cpu.pressure"),
            start: Some(1000),
            last: Cell::new(Some(6000)),
        });
        deadline.sample_pressure_with(|_| None);
        let Expired::Starved(starved) = deadline.classify_at(deadline.what, deadline.patience, now, None) else {
            crate::fail("the saved cgroup figure classifies")
        };
        assert_eq!(starved.figures().tree_waited, Some(Duration::from_nanos(5000)));
        assert!(starved.line().ends_with("; tree waited 0 ms (cgroup)"));
        if let Some(TreeWatch::Cgroup { last, .. }) = deadline.trees.first() {
            last.set(None);
        }
        deadline.sample_pressure_with(|_| None);
        let Expired::Failed(figures) = deadline.classify_at(deadline.what, deadline.patience, now, None) else {
            crate::fail("no usable figures fail even at zero patience")
        };
        assert_eq!(figures.tree_waited, None);
    }

    #[test]
    fn starvation_lines_name_the_wait_and_only_a_deciding_tree() {
        let figures = Figures {
            what: "the binary's start",
            patience: Duration::from_millis(100),
            elapsed: Duration::from_millis(100),
            thread_waited: Some(Duration::from_millis(1)),
            thread_cpu: Some(Duration::from_millis(2)),
            tree_waited: Some(Duration::from_millis(50)),
        };
        let Expired::Starved(starved) = classify(figures, "threads") else { crate::fail("tree reaches half") };
        assert_eq!(
            starved.line(),
            "skein-world: starved: waited 1 ms runnable, used 2 ms of CPU, patience 100 ms, by the binary's start; tree waited 50 ms (threads)"
        );
        let figures = Figures {
            tree_waited: Some(Duration::from_millis(49)),
            thread_waited: Some(Duration::from_millis(50)),
            ..starved.figures
        };
        let Expired::Starved(starved) = classify(figures, "threads") else { crate::fail("thread reaches half") };
        assert!(starved.line().ends_with(", by the binary's start"));
    }
}
