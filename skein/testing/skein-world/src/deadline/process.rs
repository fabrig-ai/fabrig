//! Owned process trees' scheduler snapshots (skein/docs/foundation/testing-strategy.md, 2.10).

use alloc::borrow::Cow;
use alloc::collections::{BTreeSet, VecDeque};
use core::fmt::Write;
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::{DirEntryExt, MetadataExt};
use std::path::Path;

mod files;
pub(super) use files::SampleFiles;
use files::{TaskFiles, TaskIdentity, read_fresh};

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
        let ticks = clock_ticks(&fs::read("/proc/self/auxv").ok()?)?;
        let uptime = fs::read_to_string("/proc/uptime").ok()?;
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
    #[cfg(test)]
    Directory(std::path::PathBuf),
}

#[derive(Clone, Copy)]
struct Task {
    tid: u32,
    inode: u64,
}

impl Proc {
    fn path<'a>(&self, path: &'a str) -> Cow<'a, Path> {
        match self {
            Self::Live => Cow::Borrowed(Path::new(path)),
            #[cfg(test)]
            Self::Text(_) => Cow::Borrowed(Path::new(path)),
            #[cfg(test)]
            Self::Directory(root) => Cow::Owned(root.join(path.trim_start_matches('/'))),
        }
    }

    fn open(&self, path: &str) -> Option<File> {
        #[cfg(test)]
        tests::opened(false);
        File::open(self.path(path)).ok()
    }

    fn read<'a>(&self, path: &str, text: &'a mut String) -> Option<&'a str> {
        text.clear();
        match self {
            #[cfg(test)]
            Self::Text(files) => {
                tests::opened(false);
                let (_, source) = files.iter().find(|(name, _)| *name == path)?;
                text.push_str(source);
            }
            Self::Live => {
                // Proc exposes these records with a size of zero. Use streaming
                // reads without File::read_to_string's metadata lookup.
                self.open(path)?.take(u64::MAX).read_to_string(text).ok()?;
            }
            #[cfg(test)]
            Self::Directory(_) => {
                self.open(path)?.take(u64::MAX).read_to_string(text).ok()?;
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

    fn task_file<'a>(
        &self,
        path: &mut String,
        leaf: &str,
        text: &'a mut String,
        slot: &mut Option<File>,
    ) -> Option<&'a str> {
        text.clear();
        if let Some(file) = slot {
            if read_fresh(file, text).is_ok() {
                return Some(text);
            }
            *slot = None;
            return self.file(path, leaf, text);
        }
        let base = path.len();
        path.push_str(leaf);
        let file = self.open(path);
        path.truncate(base);
        // This is also the ordinary read when a descriptor cannot be reused.
        text.clear();
        let file = file?;
        (&file).take(u64::MAX).read_to_string(text).ok()?;
        *slot = Some(file);
        Some(text)
    }

    fn tasks(&self, directory: &str, tasks: &mut Vec<Task>) {
        tasks.clear();
        #[cfg(test)]
        tests::opened(true);
        match self {
            #[cfg(test)]
            Self::Text(files) => {
                tasks.extend(files.iter().filter_map(|(path, _)| {
                    let tid = path.strip_prefix(directory)?.strip_prefix('/')?.split('/').next()?.parse().ok()?;
                    Some(Task { tid, inode: 0 })
                }));
                tasks.sort_unstable_by_key(|task| task.tid);
                tasks.dedup_by_key(|task| task.tid);
            }
            _ => {
                if let Ok(directory) = fs::read_dir(self.path(directory)) {
                    tasks.extend(directory.flatten().filter_map(|entry| {
                        let tid = entry.file_name().to_str()?.parse().ok()?;
                        Some(Task { tid, inode: entry.ino() })
                    }));
                }
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

    fn snapshot(
        &self,
        mut pending: VecDeque<(u32, Edge)>,
        boot: Option<Boot>,
        mut files: Option<&mut SampleFiles>,
    ) -> Processes {
        let mut path = String::from("/proc/self");
        let mut text = String::new();
        let owner = self.owner(&mut path, &mut text);
        let mut tasks = Vec::new();
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
            self.tasks(&path, &mut tasks);
            let base = path.len();
            let first_thread = threads.len();
            let first_child = pending.len();
            let process_born = boot.is_some_and(|boot| boot.includes(process_start));
            for &Task { tid, inode } in &tasks {
                write!(&mut path, "/{tid}").expect("String formatting");
                let mut task_files = files
                    .as_deref_mut()
                    .filter(|_| inode != 0)
                    .and_then(|files| files.task(tid, TaskIdentity { pid, process_start, inode }));
                let filling = task_files.as_deref().is_some_and(TaskFiles::filling);
                // Thread birth is needed only at expiry and only if the process's
                // birth cannot already prove it. The leader has its process's birth.
                let thread_start =
                    if boot.is_some() && !process_born && tid != pid { self.stat(&mut path, &mut text) } else { None };
                let sched = match &mut task_files {
                    Some(files) => self.task_file(&mut path, "/schedstat", &mut text, files.schedstat()),
                    None => self.file(&mut path, "/schedstat", &mut text),
                };
                if let Some(sched) = sched.and_then(parse_schedstat)
                    && (thread_start.is_none() || self.stat(&mut path, &mut text) == thread_start)
                {
                    let born_after_arm =
                        process_born || boot.zip(thread_start).is_some_and(|(boot, (_, start))| boot.includes(start));
                    threads.push(Thread { pid, process_start, tid, sched, born_after_arm });
                }
                let children = match &mut task_files {
                    Some(files) => self.task_file(&mut path, "/children", &mut text, files.children()),
                    None => self.file(&mut path, "/children", &mut text),
                };
                if let Some(children) = children {
                    pending.extend(
                        children.split_whitespace().filter_map(|child| Some((child.parse().ok()?, Edge::Child(pid)))),
                    );
                }
                // Open record descriptors pin their containing proc directory.
                // Only keep newly opened records if that directory still has
                // the inode reported by this scan's task enumeration.
                if filling
                    && let Some(files) = task_files
                    && !fs::metadata(self.path(&path)).is_ok_and(|entry| entry.ino() == inode)
                {
                    files.clear();
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
        if let Some(files) = files {
            files.drop_unseen();
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
        Self::arm_with(pids, proc, None)
    }

    pub(super) fn arm_with(pids: &[u32], proc: &Proc, files: Option<&mut SampleFiles>) -> Self {
        proc.snapshot(pids.iter().map(|&pid| (pid, Edge::Root(None))).collect(), None, files)
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
        for thread in proc.snapshot(roots, boot, None).threads {
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

    #[cfg(target_os = "linux")]
    mod reuse {
        use super::{CHILD, OWNED_TREE, Opens, Proc, UID, count};
        use crate::deadline::{ProcessSampler, Sched};
        use skein_scratch::Scratch;
        use std::{fs, path::Path};

        const ROOT: u32 = 10;
        const PARENT_THREAD: u32 = 11;
        const OWNED_PROCESSES: usize = 2;
        const PARENT_SCHEDSTAT: &str = "proc/10/task/11/schedstat";
        const PARENT_CHILDREN: &str = "proc/10/task/11/children";
        const LATER_CPU: u64 = 7;
        const LATER_WAIT: u64 = 80;

        fn write(root: &Path, name: &str, text: &str) {
            let path = root.join(name.trim_start_matches('/'));
            fs::create_dir_all(path.parent().expect("fixture file's directory")).expect("fixture directory");
            fs::write(path, text).expect("fixture record");
        }

        fn tree(scratch: &Scratch) -> Proc {
            let Proc::Text(tree) = OWNED_TREE else { unreachable!("text fixture") };
            for (path, text) in tree {
                write(scratch.path(), path, text);
            }
            for path in ["proc/10/task/10/children", "proc/30/task/30/children"] {
                write(scratch.path(), path, "");
            }
            Proc::Directory(scratch.path().to_path_buf())
        }

        fn warmed(label: &str) -> (Scratch, Proc, ProcessSampler) {
            let scratch = Scratch::new(label);
            let proc = tree(&scratch);
            let mut sampler = ProcessSampler::default();
            drop(sampler.capture_with(&[ROOT], &proc));
            (scratch, proc, sampler)
        }

        #[test]
        fn warm_arming_reuses_task_opens_without_reusing_figures() {
            let (scratch, proc, mut sampler) = warmed("deadline-warm");
            write(scratch.path(), PARENT_SCHEDSTAT, "7 80 1\n");
            let (watch, opens) = count(|| sampler.capture_with(&[ROOT], &proc));
            // One caller owner, and two identity reads and one owner per process.
            assert_eq!(opens, Opens { files: 1 + 3 * OWNED_PROCESSES, directories: OWNED_PROCESSES });
            let parent = watch.0.threads.iter().find(|thread| thread.tid == PARENT_THREAD).expect("child's parent");
            assert_eq!(parent.sched, Sched { cpu: LATER_CPU, waited: Some(LATER_WAIT) });
        }

        #[test]
        fn a_reused_tid_does_not_reuse_open_task_records() {
            let (scratch, proc, mut sampler) = warmed("deadline-replaced-task");
            fs::rename(scratch.path().join("proc/10/task/11"), scratch.path().join("dead-task"))
                .expect("replace the task without reusing its inode");
            write(scratch.path(), PARENT_SCHEDSTAT, "7 80 1\n");
            write(scratch.path(), PARENT_CHILDREN, "");
            let watch = sampler.capture_with(&[ROOT], &proc);
            let mut threads: Vec<_> = watch.0.threads.iter().map(|thread| thread.tid).collect();
            threads.sort_unstable();
            assert_eq!(threads, [ROOT, PARENT_THREAD]);
        }

        #[test]
        fn warm_arming_rediscovers_children_of_stable_threads() {
            const NEW_CHILD: u32 = 40;
            let (scratch, proc, mut sampler) = warmed("deadline-warm-children");
            write(scratch.path(), PARENT_CHILDREN, "30 40 ");
            write(scratch.path(), "proc/40/stat", CHILD);
            write(scratch.path(), "proc/40/status", UID);
            write(scratch.path(), "proc/40/task/40/schedstat", "7 80 1\n");
            write(scratch.path(), "proc/40/task/40/children", "");
            let watch = sampler.capture_with(&[ROOT], &proc);
            assert!(watch.0.threads.iter().any(|thread| thread.pid == NEW_CHILD));
        }
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
