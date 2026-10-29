//! Owned process trees' scheduler snapshots (skein/docs/foundation/testing-strategy.md, 2.10).

use alloc::collections::BTreeSet;

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
    let mut words = bytes.chunks_exact(size_of::<usize>());
    while let (Some(kind), Some(value)) = (words.next(), words.next()) {
        let kind = usize::from_ne_bytes(kind.try_into().ok()?);
        let value = usize::from_ne_bytes(value.try_into().ok()?);
        if kind == 17 {
            return u64::try_from(value).ok().filter(|ticks| *ticks != 0);
        }
        if kind == 0 {
            return None;
        }
    }
    None
}

pub(super) enum Proc {
    Live,
    #[cfg(test)]
    Text(&'static [(&'static str, &'static str)]),
}

impl Proc {
    fn read(&self, path: &str) -> Option<String> {
        match self {
            Self::Live => std::fs::read_to_string(path).ok(),
            #[cfg(test)]
            Self::Text(files) => files.iter().find(|(name, _)| *name == path).map(|(_, text)| (*text).into()),
        }
    }

    fn tasks(&self, pid: u32) -> Vec<u32> {
        let directory = format!("/proc/{pid}/task");
        match self {
            Self::Live => match std::fs::read_dir(directory) {
                Ok(tasks) => tasks.flatten().filter_map(|task| task.file_name().to_str()?.parse().ok()).collect(),
                Err(_) => Vec::new(),
            },
            #[cfg(test)]
            Self::Text(files) => {
                let prefix = format!("{directory}/");
                files
                    .iter()
                    .filter_map(|(path, _)| path.strip_prefix(&prefix)?.split('/').next()?.parse().ok())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect()
            }
        }
    }

    fn stat(&self, pid: u32) -> Option<(u32, u64)> {
        parse_stat(&self.read(&format!("/proc/{pid}/stat"))?)
    }

    fn owner(&self, pid: &str) -> Option<u32> {
        let source = self.read(&format!("/proc/{pid}/status"))?;
        source.lines().find_map(|line| line.strip_prefix("Uid:"))?.split_whitespace().nth(1)?.parse().ok()
    }

    fn snapshot(&self, roots: &[(u32, u64)]) -> Vec<Thread> {
        let Some(owner) = self.owner("self") else { return Vec::new() };
        let mut pending: Vec<_> = roots.iter().map(|&(pid, start)| (pid, Some(start), None)).collect();
        let mut seen = BTreeSet::new();
        let mut threads = Vec::new();
        while let Some((pid, expected_start, parent)) = pending.pop() {
            if !seen.insert(pid) || self.owner(&pid.to_string()) != Some(owner) {
                continue;
            }
            let Some((ppid, process_start)) = self.stat(pid) else { continue };
            if expected_start.is_some_and(|start| start != process_start) || parent.is_some_and(|parent| parent != ppid)
            {
                continue;
            }
            let mut sampled = Vec::new();
            let mut children = Vec::new();
            for tid in self.tasks(pid) {
                let base = format!("/proc/{pid}/task/{tid}");
                if let Some(sched) = self.read(&format!("{base}/schedstat")).and_then(|text| parse_schedstat(&text)) {
                    sampled.push(Thread { pid, process_start, tid, sched });
                }
                if let Some(text) = self.read(&format!("{base}/children")) {
                    children.extend(text.split_whitespace().filter_map(|pid| pid.parse::<u32>().ok()));
                }
            }
            // A reused PID cannot lend the previous process its samples or children.
            if self.stat(pid) == Some((ppid, process_start)) && self.owner(&pid.to_string()) == Some(owner) {
                threads.extend(sampled);
                pending.extend(children.into_iter().map(|child| (child, None, Some(pid))));
            }
        }
        threads
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
        let roots = pids.iter().filter_map(|&pid| Some((pid, proc.stat(pid)?.1))).collect();
        let mut watch = Self { roots, threads: Vec::new() };
        watch.threads = proc.snapshot(&watch.roots);
        watch
    }

    pub(super) fn waited(&self, proc: &Proc, boot: Option<Boot>) -> Option<Duration> {
        let mut largest = None;
        for thread in proc.snapshot(&self.roots) {
            let start = self
                .threads
                .iter()
                .find(|start| {
                    start.pid == thread.pid && start.process_start == thread.process_start && start.tid == thread.tid
                })
                .map(|start| start.sched);
            let delta = match start {
                Some(start) => sched_elapsed(Some(start), Some(thread.sched)),
                None if boot.is_some_and(|boot| boot.includes(thread.process_start)) => Some(thread.sched),
                None => None,
            };
            largest = super::largest_wait(largest, delta);
        }
        largest.map(Duration::from_nanos)
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.threads.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{Boot, Proc, Processes, clock_ticks, parse_stat};
    use crate::deadline::{Deadline, Duration, Expired, Figures, TreeWatch, classify};

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
                let mut deadline = Deadline::arm("two sources", Duration::from_nanos(100));
                for pid in pids {
                    deadline.trees.push(TreeWatch::Threads(Processes::arm(&[pid], &START)));
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
    fn an_unseen_thread_counts_from_zero_only_in_a_process_born_after_arming() {
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
    fn proc_birth_parsing_handles_comm_delimiters_and_clock_tick_boundaries() {
        assert_eq!(parse_stat(ROOT), Some((0, 1)));
        assert_eq!(parse_stat(CHILD), Some((10, 101)));
        assert_eq!(parse_stat("10 (short) S 0"), None);
        assert_eq!(clock_ticks(&[]), None);
        assert!(!Boot { ticks: 0, ..BOOT }.includes(101));
        assert!(!BOOT.includes(u64::MAX));
    }
}
