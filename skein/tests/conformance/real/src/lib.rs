//! The conformance suite against both backends on the real kernel, on loopback
//! and in scratch directories (kernel.md, 8): [`Real`], the chosen kernel as the
//! suite's backend, each process of a scenario a `Kernel` of its own, each
//! root a scratch directory of its own. The tests are
//! `tests/conformance.rs`.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::path::Path;
use std::process::Command;

use skein_conformance::{Backend as ConformanceBackend, Item, Made};
use skein_io::kernel::{Complete, Done, Error, Fd, Op, Signal, Submit, Target};
use skein_lib::{Duration, Queue, Time, Token};
use skein_scratch::Scratch;
use skein_shell::{Backend, Clock, Config, Kernel, Wait, open_root};
use skein_world::{Deadline, Expired, Figures};

/// Operations in flight per process: a scenario has a few at most.
const OPERATIONS: u32 = 16;

/// Sixteen scenario operations plus one slot for cleanup's cancellation.
const KERNEL_OPERATIONS: u32 = 17;

/// The longest one wait blocks on one process's ring before the others are
/// entered: their deferred completions run only when they are.
const SLICE: Duration = Duration::from_millis(5);

/// One forced backend per process, and a scratch directory per root, removed
/// with the backend.
pub struct Real {
    backend: Backend,
    kernels: Vec<OwnedKernel>,
    clock: Clock,
    scratches: Vec<Scratch>,
    deadline: Option<Deadline>,
    figures: Option<Figures>,
}

impl Real {
    #[must_use]
    pub fn new(backend: Backend) -> Real {
        Real { backend, kernels: Vec::new(), clock: Clock::new(), scratches: Vec::new(), deadline: None, figures: None }
    }

    /// Whether the kernel checks this process's permissions on its files:
    /// not for root, whom no mode stops.
    #[must_use]
    pub fn permissions_checked() -> bool {
        let scratch = Scratch::new("owner");
        fs::metadata(scratch.path()).expect("a scratch directory has metadata").uid() != 0
    }

    fn kernel(&mut self, process: usize) -> &mut OwnedKernel {
        self.kernels.get_mut(process).expect("a process of this backend")
    }

    fn end(&mut self) {
        for kernel in &mut self.kernels {
            kernel.end();
        }
        self.kernels.clear();
        self.scratches.clear();
    }
}

impl Drop for Real {
    fn drop(&mut self) {
        self.end();
    }
}

impl ConformanceBackend for Real {
    type Process = usize;

    fn open(&mut self) -> usize {
        let kernel = match Kernel::open_on(Config { operations: KERNEL_OPERATIONS }, self.backend) {
            Ok(kernel) => kernel,
            Err(error) => panic!("{:?} is not usable here, so it cannot be tested: {error}", self.backend),
        };
        self.kernels.push(OwnedKernel {
            kernel,
            flights: BTreeMap::new(),
            descriptors: BTreeSet::new(),
            children: BTreeSet::new(),
        });
        self.kernels.len().checked_sub(1).expect("the kernel just pushed")
    }

    fn submit(&mut self, process: usize, records: &mut Queue<Submit>) {
        self.kernel(process).submit(records, Wait::No);
    }

    fn reap(&mut self, process: usize, completions: &mut Queue<Complete>) {
        self.kernel(process).reap(completions);
    }

    fn wait_begins(&mut self, what: &'static str, patience: Duration) {
        let mut deadline = Deadline::arm(what, patience);
        deadline.watch_process(std::process::id());
        self.deadline = Some(deadline);
        self.figures = None;
    }

    fn wait_unanswered(&mut self) {
        match self.deadline.take().expect("a must-complete wait began").expired() {
            Expired::Starved(starved) => {
                self.end();
                starved.pass();
            }
            Expired::Failed(figures) => self.figures = Some(figures),
        }
    }

    fn wait_figures(&self) -> Option<String> {
        self.figures.as_ref().map(ToString::to_string)
    }

    fn now(&self) -> Time {
        self.clock.now().now
    }

    fn enter(&mut self, process: usize) {
        self.kernel(process).submit(&mut Queue::with_capacity(0), Wait::No);
    }

    /// Enters every other process's kernel to process completions and readiness,
    /// then waits on this one for a slice of `bound`.
    fn pass(&mut self, process: usize, bound: Duration) {
        let mut nothing = Queue::with_capacity(0);
        for (other, kernel) in self.kernels.iter_mut().enumerate() {
            if other != process {
                kernel.submit(&mut nothing, Wait::No);
            }
        }
        let until = self.now().saturating_add(bound.min(SLICE));
        self.kernel(process).submit(&mut nothing, Wait::Until(until));
    }

    /// Blocks the thread: no backend is entered, while the kernel's network
    /// runs on.
    fn sleep(&mut self, span: Duration) {
        std::thread::sleep(std::time::Duration::from_nanos(span.as_nanos()));
    }

    fn wait_for_connection(&mut self, process: usize, listener: Fd, bound: Duration) {
        self.wait_begins("the conformance listener's queued connection", bound);
        let remaining = self.deadline.as_ref().expect("the listener wait began").remaining();
        let readable = skein_shell::wait_readable(listener, remaining)
            .unwrap_or_else(|errno| panic!("polling listener {listener:?} of process {process} failed: errno {errno}"));
        if !readable {
            self.wait_unanswered();
        }
        assert!(
            readable,
            "no connection arrived on listener {listener:?} of process {process} within {bound:?}: {}",
            self.wait_figures().unwrap_or_default()
        );
    }

    fn assert_settled(&self, process: usize) {
        let kernel = self.kernels.get(process).expect("a process of this backend");
        assert_eq!(kernel.kernel.in_flight(), 0, "nothing in flight on process {process}");
    }

    fn append(&mut self, process: usize, root: Fd, path: &[u8], mode: u32) -> Result<Fd, Error> {
        let fd = skein_shell::open_append(root, path, mode)?;
        self.kernel(process).descriptors.insert(fd);
        Ok(fd)
    }

    /// A scratch directory holding `root`, laid out as `tree` says, and
    /// `outside` beside it, for a link to lead out to; opened as the shell
    /// opens a root.
    fn root(&mut self, process: usize, tree: &[Item]) -> Fd {
        let scratch = Scratch::new("conformance");
        let outside = scratch.path().join("outside");
        fs::create_dir(&outside).expect("a directory beside the root");
        fs::write(outside.join("secret"), b"outside").expect("a file beside the root");
        let root = scratch.path().join("root");
        fs::create_dir(&root).expect("the root");
        for item in tree {
            let path = root.join(OsStr::from_bytes(&item.path));
            match &item.made {
                Made::File(bytes) => fs::write(&path, bytes).expect("a file laid"),
                Made::Directory => fs::create_dir(&path).expect("a directory laid"),
                Made::Link(target) => symlink(OsStr::from_bytes(target), &path).expect("a symbolic link laid"),
                Made::HardLink(target) => {
                    fs::hard_link(root.join(OsStr::from_bytes(target)), &path).expect("a hard link laid");
                }
                // No FIFO without libc's mkfifo, which is unsafe: the
                // program does it.
                Made::Fifo => {
                    let made = Command::new("mkfifo").arg(&path).status().expect("mkfifo runs");
                    assert!(made.success(), "a FIFO laid");
                }
            }
        }
        // Modes last, from the deepest, so that none stops the laying.
        for item in tree.iter().rev() {
            match item.made {
                Made::File(_) | Made::Directory | Made::Fifo => {
                    mode(&root.join(OsStr::from_bytes(&item.path)), item.mode);
                }
                Made::Link(_) | Made::HardLink(_) => {}
            }
        }
        let fd = open_root(&root).expect("a scratch directory opens as a root");
        self.scratches.push(scratch);
        self.kernel(process).descriptors.insert(fd);
        fd
    }
}

fn mode(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("a mode given");
}

/// Records and resources owned by one scenario process, retained for the
/// starved exit (skein/docs/foundation/testing-strategy.md, 2.10).
struct OwnedKernel {
    kernel: Kernel,
    flights: BTreeMap<Token, Flight>,
    descriptors: BTreeSet<Fd>,
    children: BTreeSet<Fd>,
}

#[derive(Clone, Copy)]
enum Flight {
    Cancellable,
    Cancel(Token),
    Other,
}

impl OwnedKernel {
    fn submit(&mut self, records: &mut Queue<Submit>, wait: Wait) {
        for record in &*records {
            let flight = match record.kind {
                Op::Accept { .. }
                | Op::Connect { .. }
                | Op::Recv { .. }
                | Op::Send { .. }
                | Op::Open { .. }
                | Op::Read { .. }
                | Op::Write { .. }
                | Op::Append { .. }
                | Op::Sync { .. }
                | Op::Wait { .. }
                | Op::ReadSignal { .. }
                | Op::PipeRead { .. }
                | Op::PipeWrite { .. } => Flight::Cancellable,
                Op::Cancel { target } => Flight::Cancel(target),
                Op::Socket { .. }
                | Op::Bind { .. }
                | Op::Listen { .. }
                | Op::Shutdown { .. }
                | Op::Close { .. }
                | Op::Truncate { .. }
                | Op::Lock { .. }
                | Op::Stat { .. }
                | Op::Rename { .. }
                | Op::Remove { .. }
                | Op::MakeDirectory { .. }
                | Op::List { .. }
                | Op::Spawn { .. }
                | Op::Signal { .. }
                | Op::Usage
                | Op::Window { .. } => Flight::Other,
            };
            self.flights.insert(record.op, flight);
        }
        self.kernel.submit(records, wait);
        // A scenario submits no more than its kernel can take.
        for record in &*records {
            self.flights.remove(&record.op);
        }
    }

    fn reap(&mut self, out: &mut Queue<Complete>) {
        let before = usize::try_from(out.len()).expect("the completion count fits usize");
        self.kernel.reap(out);
        for complete in out.iter().skip(before) {
            self.flights.remove(&complete.op);
            match (&complete.kind, &complete.result) {
                (Op::Close { fd }, _) => {
                    self.descriptors.remove(fd);
                    self.children.remove(fd);
                }
                (Op::Wait { pidfd, reap: true }, Ok(Done::Exit(_))) => {
                    self.children.remove(pidfd);
                }
                (_, Ok(Done::Fd(fd) | Done::Accepted { fd, .. })) => {
                    self.descriptors.insert(*fd);
                }
                (Op::Spawn { spawn }, Ok(Done::Spawned { pidfd, .. })) => {
                    self.descriptors.insert(*pidfd);
                    self.children.insert(*pidfd);
                    for pipe in &spawn.pipes {
                        self.descriptors.insert(pipe.parent.expect("a spawned pipe has its parent end"));
                    }
                }
                _ => {}
            }
        }
    }

    fn end(&mut self) {
        let mut completions = Queue::with_capacity(OPERATIONS);
        let mut records = Queue::with_capacity(1);
        let mut next = u64::MAX;
        loop {
            self.reap(&mut completions);
            while completions.pop().is_some() {}
            for child in &self.children {
                let _sent = skein_shell::signal_kept_child(*child, Signal::Kill, Target::Group);
            }
            if self.flights.is_empty() {
                break;
            }
            let cancelling: BTreeSet<Token> = self
                .flights
                .values()
                .filter_map(|flight| match flight {
                    Flight::Cancel(target) => Some(*target),
                    Flight::Cancellable | Flight::Other => None,
                })
                .collect();
            let targets: Vec<Token> = self
                .flights
                .iter()
                .filter_map(|(token, flight)| {
                    if matches!(flight, Flight::Cancellable) && !cancelling.contains(token) {
                        Some(*token)
                    } else {
                        None
                    }
                })
                .collect();
            for target in targets {
                if self.kernel.room() == 0 {
                    break;
                }
                while self.flights.contains_key(&Token::new(next)) {
                    next = next.checked_sub(1).expect("cleanup tokens in reach");
                }
                records.push(Submit { op: Token::new(next), kind: Op::Cancel { target } });
                next = next.checked_sub(1).expect("cleanup tokens in reach");
                self.submit(&mut records, Wait::No);
            }
            let until = Clock::new().now().now.saturating_add(SLICE);
            self.submit(&mut records, Wait::Until(until));
        }
        for child in &self.children {
            skein_shell::abandon_binary(Some(*child), &[]);
        }
        self.children.clear();
        for descriptor in &self.descriptors {
            skein_shell::close_keeper_fd(*descriptor);
        }
        self.descriptors.clear();
    }
}
