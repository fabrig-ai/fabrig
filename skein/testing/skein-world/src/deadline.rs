//! Real-clock harness deadlines (skein/docs/design/testing.md, 5;
//! skein/docs/foundation/testing-strategy.md, 2.8 and 2.10).

#[cfg(target_os = "linux")]
use core::cell::Cell;
use core::fmt;
use core::fmt::Write;
#[cfg(target_os = "linux")]
use std::path::PathBuf;

mod process;
use process::{Boot, Proc, Processes};

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
    Threads(Processes),
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

impl Starved {
    #[must_use]
    pub fn figures(&self) -> &Figures {
        &self.figures
    }

    /// Ends a nextest test as a starved pass, writing one line to stderr.
    /// Elsewhere fails with the figures. The caller ends what it owns first:
    /// exiting skips destructors.
    #[expect(clippy::print_stderr, reason = "testing.md, 5 requires one starvation verdict on stderr")]
    pub fn pass(self) -> ! {
        if std::env::var_os("NEXTEST").is_none() {
            crate::fail(&format!("skein-world: starved: {}", self.figures));
        }
        eprintln!("{}", self.line());
        std::process::exit(0);
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
        let start = Clock::new().now().now;
        Self {
            what,
            patience,
            start,
            until: start.saturating_add(patience),
            thread: thread_sched(),
            trees: Vec::new(),
            boot: Boot::read(),
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
    /// Threads first seen at expiry count from zero only when their process
    /// started after arming; unreadable or vanished threads contribute nothing.
    pub fn watch_process(&mut self, pid: u32) {
        self.watch_threads(&[pid]);
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
        self.trees.push(TreeWatch::Threads(Processes::arm(pids, &Proc::Live)));
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
        self.classify_at(self.what, self.patience, now, thread_sched())
    }

    /// Classifies another unanswered deadline using this wait's time and scheduler origins.
    pub(crate) fn expired_at(&self, what: &'static str, until: Time) -> Expired {
        let now = Clock::new().now().now;
        assert!(now >= until, "only a passed deadline is classified");
        self.classify_at(what, until.saturating_since(self.start), now, thread_sched())
    }

    fn classify_at(&self, what: &'static str, patience: Duration, now: Time, end: Option<Sched>) -> Expired {
        let sched = sched_elapsed(self.thread, end);
        self.sample_pressure();
        let (tree_waited, kind) = self.watched_wait(&Proc::Live);
        let (thread_cpu, thread_waited) = match sched {
            Some(sched) => (Some(Duration::from_nanos(sched.cpu)), Some(Duration::from_nanos(sched.waited))),
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
        for tree in &self.trees {
            if let TreeWatch::Cgroup { path, last, .. } = tree
                && let Some(value) = pressure(path)
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
    fn waited(&self, proc: &Proc, boot: Option<Boot>) -> (Option<Duration>, &'static str) {
        match self {
            #[cfg(target_os = "linux")]
            Self::Cgroup { start, last, .. } => (
                start.zip(last.get()).and_then(|(start, last)| last.checked_sub(start)).map(Duration::from_nanos),
                "cgroup",
            ),
            Self::Threads(processes) => (processes.waited(proc, boot), "threads"),
        }
    }
}

fn largest_wait(largest: Option<u64>, delta: Option<Sched>) -> Option<u64> {
    match delta {
        Some(delta) => Some(largest.unwrap_or(0).max(delta.waited)),
        None => largest,
    }
}

#[cfg(target_os = "linux")]
fn pressure(path: &std::path::Path) -> Option<u64> {
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
    pub(crate) waited: u64,
}

pub(crate) fn thread_sched() -> Option<Sched> {
    if !cfg!(target_os = "linux") {
        return None;
    }
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
    Some(Sched { cpu: runtime, waited })
}

pub(crate) fn sched_elapsed(start: Option<Sched>, end: Option<Sched>) -> Option<Sched> {
    let (start, end) = start.zip(end)?;
    Some(Sched { cpu: end.cpu.checked_sub(start.cpu)?, waited: end.waited.checked_sub(start.waited)? })
}

#[cfg(test)]
mod tests {
    use super::{
        Boot, Deadline, Duration, Expired, Figures, Sched, Time, TreeWatch, classify, largest_wait, parse_pressure,
        parse_schedstat, sched_elapsed, starved,
    };
    #[cfg(target_os = "linux")]
    use super::{Cell, PathBuf};

    #[test]
    fn referee_patience_is_its_deadline_since_the_runs_start() {
        let deadline = Deadline {
            what: "the real loop",
            patience: Duration::from_secs(1),
            start: Time::from_nanos(7),
            until: Time::from_nanos(1_000_000_007),
            thread: None,
            trees: Vec::new(),
            boot: Boot::read(),
        };
        let Expired::Failed(figures) = deadline.expired_at("the referee's expectation", Time::from_nanos(107)) else {
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
            thread: Some(Sched { cpu: 100, waited: 200 }),
            trees: Vec::new(),
            boot: Boot::read(),
        };
        for (waited, expected) in [(49, false), (50, true), (51, true)] {
            let expired = deadline.classify_at(
                "the referee's expectation",
                Duration::from_nanos(100),
                Time::from_nanos(207),
                Some(Sched { cpu: 130, waited: 200_u64.checked_add(waited).expect("tiny counters") }),
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
    fn starvation_includes_exactly_half_the_patience_waiting() {
        let patience = Duration::from_nanos(100);
        assert!(!starved(patience, 49));
        assert!(starved(patience, 50));
    }

    #[test]
    fn starvation_includes_a_run_with_much_cpu_and_half_the_patience_waiting() {
        let patience = Duration::from_nanos(100);
        let elapsed = sched_elapsed(Some(Sched { cpu: 1, waited: 1 }), Some(Sched { cpu: 1001, waited: 51 }))
            .expect("increasing scheduler counters");
        assert_eq!(elapsed.cpu, 1000);
        assert!(starved(patience, elapsed.waited));
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
        let start = Sched { cpu: 1, waited: 1 };
        let end = Sched { cpu: 2, waited: 101 };
        for (start, end) in [(None, Some(end)), (Some(start), None), (None, None)] {
            let elapsed = sched_elapsed(start, end);
            assert_eq!(elapsed, None);
            assert!(!elapsed.is_some_and(|sched| starved(Duration::from_nanos(100), sched.waited)));
        }
        assert_eq!(sched_elapsed(Some(start), Some(end)), Some(Sched { cpu: 1, waited: 100 }));
        assert_eq!(sched_elapsed(Some(end), Some(start)), None);
        assert_eq!(sched_elapsed(Some(start), Some(Sched { cpu: 2, waited: 0 })), None);
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
        assert_eq!(parse_schedstat("123 456 7\n"), Some(Sched { cpu: 123, waited: 456 }));
        assert_eq!(parse_schedstat("1 0 1"), Some(Sched { cpu: 1, waited: 0 }));
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
        let start = Some(Sched { cpu: 1, waited: 10 });
        let mut largest = None;
        for end in [Some(Sched { cpu: 2, waited: 40 }), None, Some(Sched { cpu: 2, waited: 30 })] {
            largest = largest_wait(largest, sched_elapsed(start, end));
        }
        assert_eq!(largest, Some(30));
        assert_eq!(largest_wait(None, sched_elapsed(start, None)), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_gone_cgroup_keeps_its_last_pressure_figure() {
        let mut deadline = Deadline::arm("the gone cgroup", Duration::ZERO);
        deadline.thread = None;
        // A missing file in this process's proc directory supplies an unavailable read.
        deadline.trees.push(TreeWatch::Cgroup {
            path: PathBuf::from("/proc/thread-self/skein-missing-pressure"),
            start: Some(1000),
            last: Cell::new(Some(6000)),
        });
        let Expired::Starved(starved) = deadline.expired() else { crate::fail("the saved cgroup figure classifies") };
        assert_eq!(starved.figures().tree_waited, Some(Duration::from_nanos(5000)));
        assert!(starved.line().ends_with("; tree waited 0 ms (cgroup)"));
        if let Some(TreeWatch::Cgroup { last, .. }) = deadline.trees.first() {
            last.set(None);
        }
        let Expired::Failed(figures) = deadline.expired() else {
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
    #[cfg(target_os = "linux")]
    #[test]
    #[expect(
        clippy::print_stderr,
        reason = "the task report records the keeper's accounting observed in a passing run"
    )]
    fn a_binary_watch_chooses_the_keepers_cgroup_else_owned_threads() {
        use crate::end_to_end::{Binary, Command, Mode};
        let command = Command {
            program: "/bin/sh".into(),
            arguments: vec!["-c".into(), "read answer".into()],
            environment: vec![],
            directory: "/tmp".into(),
        };
        let binary = Binary::start(command, Mode::Pipes, 1024).expect("start a waiting owned binary");
        let has_cgroup = binary.keeper().and_then(crate::tree::Tree::cgroup).is_some();
        let mut deadline = Deadline::arm("the accounting observation", Duration::from_millis(20));
        deadline.watch(&binary);
        let (is_cgroup, source) = match deadline.trees.first().expect("a watch adds a source") {
            TreeWatch::Cgroup { start, .. } => {
                (true, format!("cgroup pressure baseline={}", super::figure(start.map(Duration::from_nanos))))
            }
            TreeWatch::Threads(threads) => (false, format!("cgroup absent; {} readable owned threads", threads.len())),
        };
        assert_eq!(is_cgroup, has_cgroup);
        deadline.watch_process(std::process::id());
        deadline.watch(&binary);
        assert_eq!(deadline.trees.len(), 3, "binary and process watches accumulate");
        std::thread::sleep(core::time::Duration::from_nanos(deadline.remaining().as_nanos()));
        let expired = deadline.expired();
        drop(binary);
        match expired {
            Expired::Failed(figures) => {
                let baseline = match deadline.thread {
                    Some(sched) => format!("CPU {} ns, runnable wait {} ns", sched.cpu, sched.waited),
                    None => String::from("UNAVAILABLE"),
                };
                eprintln!("deadline accounting observation: {source}; own baseline={baseline}; {figures}");
            }
            Expired::Starved(starved) => starved.pass(),
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[expect(clippy::print_stderr, reason = "the report records both live sources and the deadline figures")]
    fn watching_our_process_and_an_idle_child_keeps_both_sources() {
        let mut child = std::process::Command::new("/bin/sleep").arg("10").spawn().expect("spawn idle child");
        // Let the child enter its idle wait before taking the scheduler baseline.
        let startup = Deadline::arm("the idle child's startup", Duration::from_millis(20));
        std::thread::sleep(core::time::Duration::from_nanos(startup.remaining().as_nanos()));
        let mut deadline = Deadline::arm("our process and idle child", Duration::from_millis(20));
        assert!(deadline.boot.is_some(), "Linux exposes the shared boot-clock origin");
        deadline.watch_process(std::process::id());
        deadline.watch_process(child.id());
        std::thread::sleep(core::time::Duration::from_nanos(deadline.remaining().as_nanos()));
        assert_eq!(deadline.trees.len(), 2, "each watch keeps a source");
        let sources: Vec<_> =
            deadline.trees.iter().map(|tree| tree.waited(&super::Proc::Live, deadline.boot).0).collect();
        let expired = deadline.expired();
        child.kill().expect("end idle child");
        child.wait().expect("reap idle child");
        let figures = match expired {
            Expired::Failed(figures) => figures,
            Expired::Starved(starved) => starved.pass(),
        };
        for source in &sources {
            assert!(source.is_some(), "each owned process has readable accounting: {sources:?}; {:?}", deadline.trees);
            let waited = source.expect("checked readable accounting");
            assert!(waited.as_nanos() < figures.patience.as_nanos().div_ceil(2));
        }
        let waited = figures.tree_waited.expect("both owned processes have readable accounting");
        assert!(waited.as_nanos() < figures.patience.as_nanos().div_ceil(2));
        let sources = sources.iter().copied().map(super::figure).collect::<Vec<_>>().join(", ");
        eprintln!("deadline accounting: own process and child: sources=[{sources}]; {figures}");
    }
}
