//! Owned process trees' scheduler snapshots (skein/docs/foundation/testing-strategy.md, 2.10).

use alloc::collections::{BTreeSet, VecDeque};
use core::fmt::Write;

use super::{Duration, Sched, parse_schedstat, sched_elapsed};

#[derive(Debug)]
pub(super) struct Processes {
    roots: Vec<(u32, u64)>,
    threads: Vec<Thread>,
}

#[derive(Debug)]
struct Thread {
    pid: u32,
    process_start: u64,
    tid: u32,
    sched: Sched,
    born_after_arm: bool,
}

/// Both values use Linux's boot clock, including suspended time.
#[derive(Clone, Copy, Debug)]
pub(super) struct Boot {
    upper_nanos: u64,
    ticks: u64,
}

impl Boot {
    pub(super) fn read() -> Option<Self> {
        if !cfg!(target_os = "linux") {
            return None;
        }
        let ticks = clock_ticks(&std::fs::read("/proc/self/auxv").ok()?)?;
        let uptime = std::fs::read_to_string("/proc/uptime").ok()?;
        let (seconds, hundredths) = uptime.split_whitespace().next()?.split_once('.')?;
        if hundredths.len() != 2 {
            return None;
        }
        // Uptime is truncated to centiseconds. Its upper endpoint prevents a
        // process from the arming tick being mistaken for a later birth.
        let upper_nanos = seconds
            .parse::<u64>()
            .ok()?
            .checked_mul(1_000_000_000)?
            .checked_add(hundredths.parse::<u64>().ok()?.checked_add(1)?.checked_mul(10_000_000)?)?;
        Some(Self { upper_nanos, ticks })
    }

    fn includes(self, start: u64) -> bool {
        start
            .checked_mul(1_000_000_000)
            .and_then(|nanos| nanos.checked_div(self.ticks))
            .is_some_and(|nanos| nanos >= self.upper_nanos)
    }
}

// Linux AT_CLKTCK is a native-word pair in this process's auxiliary vector.
#[expect(clippy::host_endian_bytes, reason = "proc/self/auxv uses this process's native ABI")]
fn clock_ticks(bytes: &[u8]) -> Option<u64> {
    let mut words = bytes.as_chunks::<{ size_of::<usize>() }>().0.iter();
    while let (Some(kind), Some(value)) = (words.next(), words.next()) {
        let kind = usize::from_ne_bytes(*kind);
        let value = usize::from_ne_bytes(*value);
        if kind == 17 {
            return u64::try_from(value).ok().filter(|ticks| *ticks != 0);
        }
        if kind == 0 {
            return None;
        }
    }
    None
}

#[derive(Clone, Copy)]
enum Edge {
    Root(Option<u64>),
    Child(u32),
}

pub(super) enum Proc {
    Live,
    #[cfg(test)]
    Text(&'static [(&'static str, &'static str)]),
}

impl Proc {
    fn read<'a>(&self, path: &str, text: &'a mut String) -> Option<&'a str> {
        text.clear();
        match self {
            Self::Live => {
                use std::io::Read;
                // Proc exposes these records with a size of zero. Use streaming
                // reads without File::read_to_string's metadata lookup.
                std::fs::File::open(path).ok()?.take(u64::MAX).read_to_string(text).ok()?;
            }
            #[cfg(test)]
            Self::Text(files) => {
                tests::opened(false);
                let (_, source) = files.iter().find(|(name, _)| *name == path)?;
                text.push_str(source);
            }
        }
        Some(text)
    }

    fn file<'a>(&self, path: &mut String, leaf: &str, text: &'a mut String) -> Option<&'a str> {
        let base = path.len();
        path.push_str(leaf);
        let source = self.read(path, text);
        path.truncate(base);
        source
    }

    fn tasks(&self, directory: &str, tids: &mut Vec<u32>) {
        tids.clear();
        match self {
            Self::Live => {
                if let Ok(tasks) = std::fs::read_dir(directory) {
                    tids.extend(tasks.flatten().filter_map(|task| task.file_name().to_str()?.parse::<u32>().ok()));
                }
            }
            #[cfg(test)]
            Self::Text(files) => {
                tests::opened(true);
                tids.extend(files.iter().filter_map(|(path, _)| {
                    path.strip_prefix(directory)?.strip_prefix('/')?.split('/').next()?.parse::<u32>().ok()
                }));
                tids.sort_unstable();
                tids.dedup();
            }
        }
    }

    fn stat(&self, path: &mut String, text: &mut String) -> Option<(u32, u64)> {
        parse_stat(self.file(path, "/stat", text)?)
    }

    pub(super) fn start_of(&self, pid: u32) -> Option<u64> {
        self.stat(&mut format!("/proc/{pid}"), &mut String::new()).map(|(_, start)| start)
    }

    fn owner(&self, path: &mut String, text: &mut String) -> Option<u32> {
        let source = self.file(path, "/status", text)?;
        source.lines().find_map(|line| line.strip_prefix("Uid:"))?.split_whitespace().nth(1)?.parse().ok()
    }

    fn snapshot(&self, mut pending: VecDeque<(u32, Edge)>, boot: Option<Boot>) -> Processes {
        let mut path = String::from("/proc/self");
        let mut text = String::new();
        let owner = self.owner(&mut path, &mut text);
        let mut tids = Vec::new();
        let mut seen = BTreeSet::new();
        let mut roots = Vec::new();
        let mut threads = Vec::new();
        while let Some((pid, edge)) = pending.pop_front() {
            if seen.contains(&pid) {
                continue;
            }
            path.truncate("/proc/".len());
            write!(&mut path, "{pid}").expect("String formatting");
            let Some((ppid, process_start)) = self.stat(&mut path, &mut text) else { continue };
            match edge {
                // Keep the root's identity even if its accounting is unavailable here.
                Edge::Root(None) => roots.push((pid, process_start)),
                Edge::Root(Some(start)) if start != process_start => continue,
                Edge::Child(parent) if parent != ppid => continue,
                Edge::Root(Some(_)) | Edge::Child(_) => {}
            }
            if owner.is_none() || self.owner(&mut path, &mut text) != owner {
                continue;
            }
            let process_path = path.len();
            path.push_str("/task");
            self.tasks(&path, &mut tids);
            let base = path.len();
            let first_thread = threads.len();
            let first_child = pending.len();
            let process_born = boot.is_some_and(|boot| boot.includes(process_start));
            for &tid in &tids {
                write!(&mut path, "/{tid}").expect("String formatting");
                // Thread birth is needed only at expiry and only if the process's
                // birth cannot already prove it. The leader has its process's birth.
                let thread_start =
                    if boot.is_some() && !process_born && tid != pid { self.stat(&mut path, &mut text) } else { None };
                if let Some(sched) = self.file(&mut path, "/schedstat", &mut text).and_then(parse_schedstat)
                    && (thread_start.is_none() || self.stat(&mut path, &mut text) == thread_start)
                {
                    let born_after_arm =
                        process_born || boot.zip(thread_start).is_some_and(|(boot, (_, start))| boot.includes(start));
                    threads.push(Thread { pid, process_start, tid, sched, born_after_arm });
                }
                if let Some(children) = self.file(&mut path, "/children", &mut text) {
                    pending.extend(
                        children.split_whitespace().filter_map(|child| Some((child.parse().ok()?, Edge::Child(pid)))),
                    );
                }
                path.truncate(base);
            }
            path.truncate(process_path);
            // A reused PID cannot lend the previous process its samples or children.
            if self.stat(&mut path, &mut text) == Some((ppid, process_start)) {
                seen.insert(pid);
            } else {
                threads.truncate(first_thread);
                pending.truncate(first_child);
            }
        }
        roots.sort_unstable();
        roots.dedup();
        Processes { roots, threads }
    }
}

fn parse_stat(source: &str) -> Option<(u32, u64)> {
    // comm may contain spaces and closing parentheses; fields resume after its last ')'.
    let (_, tail) = source.rsplit_once(") ")?;
    let mut fields = tail.split_whitespace();
    let parent = fields.nth(1)?.parse().ok()?; // field 4, after state
    let start = fields.nth(17)?.parse().ok()?; // field 22
    Some((parent, start))
}

impl Processes {
    pub(super) fn arm(pids: &[u32], proc: &Proc) -> Self {
        proc.snapshot(pids.iter().map(|&pid| (pid, Edge::Root(None))).collect(), None)
    }

    pub(super) fn has_root(&self, pid: u32) -> bool {
        self.roots.iter().any(|&(root, _)| root == pid)
    }

    pub(super) fn watches(&self, pid: u32, start: u64) -> bool {
        self.roots.contains(&(pid, start))
    }

    pub(super) fn waited(&self, proc: &Proc, boot: Option<Boot>) -> Option<Duration> {
        let roots = self.roots.iter().map(|&(pid, start)| (pid, Edge::Root(Some(start)))).collect();
        let mut largest = None;
        for thread in proc.snapshot(roots, boot).threads {
            let delta = if thread.born_after_arm {
                Some(thread.sched)
            } else {
                let start = self
                    .threads
                    .iter()
                    .find(|start| {
                        start.pid == thread.pid
                            && start.process_start == thread.process_start
                            && start.tid == thread.tid
                    })
                    .map(|start| start.sched);
                sched_elapsed(start, Some(thread.sched))
            };
            largest = super::largest_wait(largest, delta);
        }
        largest.map(Duration::from_nanos)
    }

    #[cfg(target_os = "linux")]
    pub(super) fn len(&self) -> usize {
        self.threads.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{Boot, Proc, Processes, clock_ticks, parse_stat};
    use crate::deadline::{Deadline, DeadlineOrigin, Duration, Expired, Figures, ProcessWatch, Time, classify};
    use core::cell::Cell;

    #[derive(Default, Debug, PartialEq, Eq)]
    struct Opens {
        files: usize,
        directories: usize,
    }

    std::thread_local! {
        static OPENS: Cell<Option<Opens>> = const { Cell::new(None) };
    }

    pub(super) fn opened(directory: bool) {
        if let Some(mut counts) = OPENS.take() {
            if directory {
                counts.directories = counts.directories.saturating_add(1);
            } else {
                counts.files = counts.files.saturating_add(1);
            }
            OPENS.set(Some(counts));
        }
    }

    fn count<T>(sample: impl FnOnce() -> T) -> (T, Opens) {
        OPENS.set(Some(Opens::default()));
        let value = sample();
        (value, OPENS.take().expect("procfs open counts"))
    }

    const ROOT: &str = "10 (root ) worker) S 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1";
    const CHILD: &str = "30 (child) S 10 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 101";
    const UID: &str = "Name:\thelper\nUid:\t1000\t1000\t1000\t1000\n";
    const BOOT: Boot = Boot { upper_nanos: 1_010_000_000, ticks: 100 };
    const START: Proc = Proc::Text(&[
        ("/proc/self/status", UID),
        ("/proc/10/status", UID),
        ("/proc/10/stat", ROOT),
        ("/proc/10/task/10/schedstat", "1 10 1"),
        ("/proc/20/status", UID),
        ("/proc/20/stat", ROOT),
        ("/proc/20/task/20/schedstat", "1 10 1"),
    ]);

    const OWNED_TREE: Proc = Proc::Text(&[
        ("/proc/self/status", UID),
        ("/proc/10/status", UID),
        ("/proc/10/stat", ROOT),
        ("/proc/10/task/10/schedstat", "1 10 1"),
        ("/proc/10/task/11/schedstat", "1 10 1"),
        ("/proc/10/task/11/children", "30"),
        ("/proc/30/status", UID),
        ("/proc/30/stat", CHILD),
        ("/proc/30/task/30/schedstat", "1 10 1"),
    ]);

    #[test]
    fn stable_arming_opens_one_plus_three_per_process_and_two_per_thread() {
        // OWNED_TREE: root and child; the root's non-leader owns the child.
        const PROCESSES: usize = 2;
        const THREADS: usize = 3;
        let (snapshot, opens) = count(|| Processes::arm(&[10], &OWNED_TREE));
        assert_eq!(opens, Opens { files: 1 + 3 * PROCESSES + 2 * THREADS, directories: PROCESSES });
        assert_eq!(snapshot.threads.len(), THREADS);
    }

    #[test]
    fn coarmed_accounts_share_reads_but_keep_their_own_verdicts() {
        const SHORT: Duration = Duration::from_nanos(100);
        const LONG: Duration = Duration::from_nanos(200);
        const ROOTS: [u32; 2] = [10, 20];
        let origin = DeadlineOrigin { start: Time::ZERO, thread: None, boot: None };
        let end = Proc::Text(&[
            ("/proc/self/status", UID),
            ("/proc/10/status", UID),
            ("/proc/10/stat", ROOT),
            ("/proc/10/task/10/schedstat", "2 70 2"),
            ("/proc/20/status", UID),
            ("/proc/20/stat", ROOT),
            ("/proc/20/task/20/schedstat", "2 10 2"),
        ]);
        let (_, single) = count(|| ProcessWatch::capture_with(&ROOTS, &START));
        let (shared, opens) = count(|| {
            let watch = ProcessWatch::capture_with(&ROOTS, &START);
            [("step", SHORT), ("command", LONG)].map(|(what, patience)| {
                let mut deadline = origin.arm(what, patience);
                deadline.watch_snapshot(&watch);
                deadline
            })
        });
        assert_eq!(opens, single);
        for (shared, expected_starved) in shared.into_iter().zip([true, false]) {
            let mut separate = origin.arm(shared.what, shared.patience);
            for pid in ROOTS {
                separate.watch_process_with(pid, &START);
            }
            for deadline in [&shared, &separate] {
                assert_eq!(
                    matches!(
                        deadline.classify_with(deadline.what, deadline.patience, deadline.until(), None, &end),
                        Expired::Starved(_)
                    ),
                    expected_starved,
                );
            }
        }
    }

    #[test]
    fn another_watch_of_the_same_pid_and_birth_only_checks_identity() {
        const PID: u32 = 10;
        let mut deadline = Deadline::at("command readiness", Duration::from_nanos(100), Time::ZERO);
        deadline.watch_process_with(PID, &OWNED_TREE);
        let ((), opens) = count(|| deadline.watch_process_with(PID, &OWNED_TREE));
        assert_eq!(opens, Opens { files: 1, directories: 0 });
    }

    #[test]
    fn two_sources_keep_the_larger_delta_and_never_sum() {
        let large = Proc::Text(&[
            ("/proc/self/status", UID),
            ("/proc/10/status", UID),
            ("/proc/10/stat", ROOT),
            ("/proc/10/task/10/schedstat", "2 40 2"),
            ("/proc/20/status", UID),
            ("/proc/20/stat", ROOT),
            ("/proc/20/task/20/schedstat", "2 70 2"),
        ]);
        let small = Proc::Text(&[
            ("/proc/self/status", UID),
            ("/proc/10/status", UID),
            ("/proc/10/stat", ROOT),
            ("/proc/10/task/10/schedstat", "2 40 2"),
            ("/proc/20/status", UID),
            ("/proc/20/stat", ROOT),
            ("/proc/20/task/20/schedstat", "2 50 2"),
        ]);
        for (end, expected, starved) in [(large, 60, true), (small, 40, false)] {
            for pids in [[10, 20], [20, 10]] {
                let mut deadline = Deadline::at("two sources", Duration::from_nanos(100), Time::ZERO);
                for pid in pids {
                    deadline.watch_snapshot(&ProcessWatch::capture_with(&[pid], &START));
                }
                let (waited, kind) = deadline.watched_wait(&end);
                assert_eq!(waited, Some(Duration::from_nanos(expected)));
                let figures = Figures {
                    what: "two sources",
                    patience: deadline.patience,
                    elapsed: deadline.patience,
                    thread_waited: None,
                    thread_cpu: None,
                    tree_waited: waited,
                };
                assert_eq!(matches!(classify(figures, kind), Expired::Starved(_)), starved);
            }
        }
    }

    #[test]
    fn an_expiry_descendant_decides_even_when_born_after_arming() {
        let end = Proc::Text(&[
            ("/proc/self/status", UID),
            ("/proc/10/status", UID),
            ("/proc/10/stat", ROOT),
            ("/proc/10/task/10/schedstat", "2 10 2"),
            ("/proc/10/task/11/children", "30"),
            ("/proc/30/status", UID),
            ("/proc/30/stat", CHILD),
            ("/proc/30/task/30/schedstat", "7 80 1"),
        ]);
        let watch = Processes::arm(&[10], &START);
        let waited = watch.waited(&end, Some(BOOT));
        assert_eq!(waited, Some(Duration::from_nanos(80)));
        assert!(super::super::waited_half(Duration::from_nanos(100), waited));
    }

    #[test]
    fn an_unseen_thread_without_its_own_birth_needs_a_new_process() {
        let old = Proc::Text(&[
            ("/proc/self/status", UID),
            ("/proc/30/status", UID),
            ("/proc/30/stat", ROOT),
            ("/proc/30/task/31/schedstat", "7 80 1"),
        ]);
        let new = Proc::Text(&[
            ("/proc/self/status", UID),
            ("/proc/30/status", UID),
            ("/proc/30/stat", CHILD),
            ("/proc/30/task/31/schedstat", "7 80 1"),
        ]);
        for (proc, expected) in [(old, None), (new, Some(Duration::from_nanos(80)))] {
            let mut watch = Processes::arm(&[30], &proc);
            watch.threads.clear(); // A thread not readable in the arming snapshot.
            assert_eq!(watch.waited(&proc, Some(BOOT)), expected);
            assert_eq!(watch.waited(&proc, None), None, "unknown arming time cannot prove a birth");
        }
        assert!(!BOOT.includes(100), "the ambiguous arming tick cannot prove a later birth");
        assert!(BOOT.includes(101));
    }

    #[test]
    fn a_new_thread_in_an_older_process_counts_from_birth() {
        let later = Proc::Text(&[
            ("/proc/self/status", UID),
            ("/proc/10/status", UID),
            ("/proc/10/stat", ROOT),
            ("/proc/10/task/10/schedstat", "2 10 2"),
            ("/proc/10/task/11/stat", "11 (thread) S 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 101"),
            ("/proc/10/task/11/schedstat", "7 80 1"),
        ]);
        let watch = Processes::arm(&[10], &START);
        assert_eq!(watch.waited(&later, Some(BOOT)), Some(Duration::from_nanos(80)));
    }

    #[test]
    fn unavailable_gone_unowned_and_reused_processes_supply_no_delta() {
        let watch = Processes::arm(&[10], &START);
        for proc in [
            Proc::Text(&[]),
            Proc::Text(&[("/proc/self/status", UID), ("/proc/10/status", UID), ("/proc/10/stat", ROOT)]),
            Proc::Text(&[
                ("/proc/self/status", UID),
                ("/proc/10/status", UID),
                ("/proc/10/stat", ROOT),
                ("/proc/10/task/10/schedstat", "0 0 0"),
            ]),
            Proc::Text(&[
                ("/proc/self/status", UID),
                ("/proc/10/status", "Uid: 2000 2000 2000 2000"),
                ("/proc/10/stat", ROOT),
                ("/proc/10/task/10/schedstat", "7 8000 1"),
            ]),
            Proc::Text(&[
                ("/proc/self/status", UID),
                ("/proc/10/status", UID),
                ("/proc/10/stat", CHILD),
                ("/proc/10/task/10/schedstat", "7 8000 1"),
            ]),
        ] {
            assert_eq!(watch.waited(&proc, Some(BOOT)), None);
        }
    }

    #[test]
    #[expect(clippy::host_endian_bytes, reason = "the auxv fixture uses this process's native ABI")]
    fn proc_birth_parsing_handles_comm_delimiters_and_clock_tick_boundaries() {
        assert_eq!(parse_stat(ROOT), Some((0, 1)));
        assert_eq!(parse_stat(CHILD), Some((10, 101)));
        assert_eq!(parse_stat("10 (short) S 0"), None);
        assert_eq!(clock_ticks(&[]), None);
        let auxv =
            [6_usize.to_ne_bytes(), 4096_usize.to_ne_bytes(), 17_usize.to_ne_bytes(), 100_usize.to_ne_bytes()].concat();
        assert_eq!(clock_ticks(&auxv), Some(100));
        assert!(!Boot { ticks: 0, ..BOOT }.includes(101));
        assert!(!BOOT.includes(u64::MAX));
    }
}
