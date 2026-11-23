//! The git component: counted operations, explicit lifecycle, parent queues.
use crate::{
    Event, Identity, Limits, Outputs, Request, Settings, Unfit, Unusable, Version,
    files::{Action, Directories, SetAside, Task},
    invocation::Command,
    limits,
    operation::Operation,
    sequence::{self, Next},
    store_lock::Access,
};
use alloc::boxed::Box;
use skein_io::{self as io, kernel};
use skein_lib::{Duration, Id, List, Map, Slab, Time, Token};
use temper_worker_domain_checkout::{self as checkout, git};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Start {
    Idle,
    Checking,
    Layout,
    Started,
    Unusable,
    Aborted,
}
const SWEEP_RETRY: Duration = Duration::from_secs(60);
const SWEEP_REPORTS: u32 = 64;

#[derive(Debug)]
struct Reported {
    path: Box<[u8]>,
    error: kernel::Error,
    seen: bool,
}

/// One worker's git component; the service drives it and drains its outputs.
#[derive(Debug)]
pub struct Component {
    pub(crate) limits: Limits,
    pub(crate) checkout: checkout::Limits,
    pub(crate) settings: Settings,
    slots: Slab<Operation>,
    index: Map<Token, Id<Operation>>,
    /// Arrival-ordered store waiters; each is one admitted operation (protocol/git-operations.md, 1.14).
    store_waiters: Map<Token, Id<Operation>>,
    serial: u64,
    start: Start,
    shutting_down: bool,
    directories: Directories,
    background: Option<Task>,
    sweep_refused_removal: bool,
    sweep_ended_serial: Option<u64>,
    sweep_failures: u64,
    sweep_at: Option<Time>,
    sweep_again: bool,
    sweep_reports: List<Reported>,
    checked: Option<(Box<[u8]>, Option<Version>)>,
}
fn usable(version: Option<Version>) -> bool {
    version >= Some(Version { major: 2, minor: 41, patch: 0 })
}
fn event(out: &mut Outputs, event: Event) {
    out.events.push(event);
}
impl Component {
    /// Refuse the limit that breaks protocol/git.md section 8 before
    /// allocating state.
    pub fn new(limits: &Limits, checkout: &checkout::Limits, settings: Settings) -> Result<Component, Unfit> {
        limits::check(limits, checkout)?;
        let capacity = limits.operations.checked_add(1).ok_or(Unfit::Memory)?;
        let slots = capacity.checked_mul(2).ok_or(Unfit::Memory)?;
        assert!(settings.largest_read > 0 && settings.largest_room > 0, "io validated usable limits");
        let directories = Directories::new(settings.serial);
        Ok(Component {
            limits: *limits,
            checkout: *checkout,
            settings,
            slots: Slab::with_capacity(slots),
            index: Map::with_capacity(capacity),
            store_waiters: Map::with_capacity(limits.operations),
            serial: 0,
            start: Start::Idle,
            shutting_down: false,
            directories,
            background: None,
            sweep_refused_removal: false,
            sweep_ended_serial: None,
            sweep_failures: 0,
            sweep_at: None,
            sweep_again: false,
            sweep_reports: List::with_capacity(SWEEP_REPORTS),
            checked: None,
        })
    }
    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    /// A running or scheduled pass, the counted failures, whether the last pass met a refused removal,
    /// and the serial number that was next when that pass ended, if one has ended.
    #[must_use]
    pub fn sweep_state(&self) -> (bool, u64, bool, Option<u64>) {
        let pending = match &self.background {
            Some(task) => match &task.phase {
                crate::files::Phase::Sweep(_) => true,
                crate::files::Phase::Reds(_)
                | crate::files::Phase::Results(_)
                | crate::files::Phase::Layout(_)
                | crate::files::Phase::Make(_)
                | crate::files::Phase::Look(_)
                | crate::files::Phase::Store(_)
                | crate::files::Phase::Alternates(_)
                | crate::files::Phase::View(_)
                | crate::files::Phase::Scan(_)
                | crate::files::Phase::Index(_)
                | crate::files::Phase::Closing => false,
            },
            None => false,
        };
        let due = !self.shutting_down && (self.start == Start::Checking || self.start == Start::Layout);
        (
            pending || due || self.sweep_again || self.sweep_at.is_some(),
            self.sweep_failures,
            self.sweep_refused_removal,
            self.sweep_ended_serial,
        )
    }
    /// Take the next operation's bounds, keeping slots and work already in flight
    /// (temper/docs/design/protocol/configuration/figures.md, 3.2).
    #[must_use]
    pub fn take_limits(&mut self, limits: &Limits) -> bool {
        if limits.operations != self.limits.operations || limits::check(limits, &self.checkout).is_err() {
            return false;
        }
        self.limits = *limits;
        true
    }
    fn token(&mut self) -> Option<Token> {
        self.serial = self.serial.checked_add(1)?;
        if self.serial >= (1_u64 << 48_u32) {
            return None;
        }
        Some(Token::new(self.settings.prefix.checked_add(self.serial)?))
    }
    fn insert(&mut self, operation: Operation) -> Id<Operation> {
        let key = operation.io_owner;
        let id = self.slots.insert(operation).expect("live plus retired operations fit the slab");
        assert!(self.index.insert(key, id).expect("admitted index fits").is_none(), "unique invocation token");
        id
    }
    /// Keep workspace and worker git directories across this worker's start.
    pub fn keep_workspaces(&mut self) {
        assert_eq!(self.start, Start::Idle, "set workspace reuse before start");
        self.directories.keep = true;
    }

    #[must_use]
    pub const fn keeps_workspaces(&self) -> bool {
        self.directories.keep
    }

    /// Start once with git's version check followed by the private layout.
    pub fn start(&mut self, _now: Time, out: &mut Outputs) {
        assert_eq!(self.start, Start::Idle, "start once");
        if self.settings.git.is_none() {
            self.start = Start::Layout;
            self.background(Task::layout(), out);
            return;
        }
        self.start = Start::Checking;
        let token = self.token().expect("first invocation token");
        let mut operation = Operation::new(token, token, Time::from_nanos(u64::MAX), None, None, Command::Version);
        operation.spawn(token, Command::Version, &self.settings, &self.limits, out);
        self.insert(operation);
    }
    fn owned(&self, owner: Token) -> Option<Id<Operation>> {
        for (_, id) in &self.index {
            if self.slots.get(*id).expect("live index").owner == owner {
                return Some(*id);
            }
        }
        None
    }
    /// Run an operation or cancel the operation named by its echoed owner.
    pub fn down(&mut self, now: Time, request: Request, out: &mut Outputs) {
        match request {
            Request::Io { owner, op, deadline, identity } => self.begin(now, owner, op, deadline, identity, out),
            Request::CancelIo { owner } => {
                if let Some(id) = self.owned(owner) {
                    self.slots.get_mut(id).expect("live index").cancel(git::Fault::Cancelled, &self.checkout, out);
                }
            }
            Request::Shutdown => {
                self.shutting_down = true;
                self.sweep_at = None;
                self.maintain(false, out);
            }
        }
        self.wake_stores(now, out);
    }
    /// Lock holders and waiters are operations, bounded by `Limits::operations` (protocol/git-operations.md, 1.14).
    fn store_step(&mut self, id: Id<Operation>, command: &Command) -> bool {
        let operation = self.slots.get(id).expect("live operation");
        let access = match &operation.op {
            Some(op) => Access::for_command(op, command),
            None => None,
        };
        let Some(access) = access else {
            self.slots.get_mut(id).expect("live operation").store_lock = None;
            return true;
        };
        if operation.store_lock == Some(access) && !operation.waiting() {
            return true;
        }
        let admission = operation.admission;
        let free = self.store_free(admission, access);
        self.slots.get_mut(id).expect("live operation").store_lock = Some(access);
        if !free {
            assert!(self.store_waiters.insert(admission, id).expect("one waiter per operation").is_none(), "wait once");
        }
        free
    }
    fn store_free(&self, admission: Token, access: Access) -> bool {
        for (_, id) in &self.index {
            let operation = self.slots.get(*id).expect("live index");
            if operation.admission != admission
                && !operation.waiting()
                && let Some(held) = operation.store_lock
                && access.conflicts(held)
            {
                return false;
            }
        }
        for (earlier, id) in &self.store_waiters {
            if *earlier >= admission {
                break;
            }
            let waiting = self.slots.get(*id).expect("live waiter").store_lock.expect("waiter requests a lock");
            if waiting.store == access.store {
                return false;
            }
        }
        true
    }
    fn wake_stores(&mut self, now: Time, out: &mut Outputs) {
        if self.store_waiters.is_empty() {
            return;
        }
        let mut queued = List::with_capacity(self.store_waiters.len());
        for (_, id) in &self.store_waiters {
            queued.push(*id).expect("at most one waiter per operation");
        }
        for &id in &queued {
            let operation = self.slots.get_mut(id).expect("live waiter");
            if operation.stop.is_none() && now >= operation.deadline {
                operation.stop = Some(git::Fault::TimedOut);
            }
            if let Some(fault) = operation.stop {
                self.finish(id, git::Done::Failed { fault }, out);
                continue;
            }
            let admission = operation.admission;
            let access = operation.store_lock.expect("waiter requests a lock");
            if self.store_free(admission, access) {
                self.store_waiters.remove(&admission).expect("queued waiter");
                let command = self.slots.get_mut(id).expect("live waiter").resume();
                self.advance(id, Next::Run(command), out);
            }
        }
        self.maintain(false, out);
    }
    fn begin(
        &mut self,
        now: Time,
        owner: Token,
        op: git::Op,
        deadline: Time,
        identity: Option<Identity>,
        out: &mut Outputs,
    ) {
        let needs_identity = match &op {
            git::Op::Store { .. }
            | git::Op::CheckOut { .. }
            | git::Op::Merge { .. }
            | git::Op::Fetch { .. }
            | git::Op::Counts { .. }
            | git::Op::Create { .. }
            | git::Op::Commit { .. }
            | git::Op::Push { .. } => true,
            git::Op::Reds { .. }
            | git::Op::Reset { .. }
            | git::Op::Pack { .. }
            | git::Op::Point { .. }
            | git::Op::Make { .. }
            | git::Op::Clone { .. }
            | git::Op::Look { .. }
            | git::Op::DiffPaths { .. }
            | git::Op::Brings { .. }
            | git::Op::IsAncestor { .. } => false,
        };
        if self.start != Start::Started
            || self.shutting_down
            || self.index.len() >= self.limits.operations
            || self.slots.is_full()
            || self.owned(owner).is_some()
            || (needs_identity && identity.is_none())
        {
            event(out, Event::Done { owner, done: git::Done::Failed { fault: git::Fault::Broken } });
            return;
        }
        if now >= deadline {
            event(out, Event::Done { owner, done: git::Done::Failed { fault: git::Fault::TimedOut } });
            return;
        }
        let Some(token) = self.token() else {
            event(out, Event::Done { owner, done: git::Done::Failed { fault: git::Fault::Broken } });
            return;
        };
        let mut next = sequence::first(&op);
        // Concurrent first uses check independently; only completed answers
        // are reused, and the last completed check replaces the cache.
        if let Some(program) = crate::invocation::item_program(&op)
            && op.kind() != git::Kind::Store
        {
            next = match &self.checked {
                Some((checked, answer)) if checked.as_ref() == program => {
                    if usable(*answer) {
                        next
                    } else {
                        Next::End(git::Done::Failed { fault: git::Fault::Broken })
                    }
                }
                Some(_) | None => Next::Run(Command::Version),
            };
        }
        let identity = if needs_identity { identity } else { None };
        let operation = Operation::new(owner, token, deadline, Some(op), identity, Command::Version);
        let id = self.insert(operation);
        self.advance(id, next, out);
    }
    /// Route only events accepted by the invocation or one of its pipe tokens.
    pub fn up(&mut self, now: Time, input: io::Event, out: &mut Outputs) {
        let Some(owner) = io_owner(&input) else {
            return;
        };
        let mut found = None;
        for (_, id) in &self.index {
            if self.slots.get(*id).expect("live index").accepts(owner) {
                found = Some(*id);
                break;
            }
        }
        let Some(id) = found else {
            return;
        };
        let operation = self.slots.get_mut(id).expect("live index");
        let startup = operation.op.is_none();
        let checking = operation.command == Command::Version;
        let version = operation.found_version();
        if !startup && now >= operation.deadline {
            operation.cancel(git::Fault::TimedOut, &self.checkout, out);
        }
        let next = operation.up(input, &self.settings, &self.checkout, out);
        if let Some(next) = next {
            if startup {
                let aborted = operation.aborted;
                self.retire(id);
                if aborted {
                    self.start = Start::Aborted;
                    self.maintain(false, out);
                    return;
                }
                let program = self.settings.git.as_ref().expect("startup checked a configured program");
                self.checked = Some((program.clone(), version));
                if usable(version) {
                    self.start = Start::Layout;
                    self.background(Task::layout(), out);
                } else {
                    self.start = Start::Unusable;
                    event(out, Event::Unusable { why: Unusable::Version { found: version } });
                }
            } else {
                let next = if checking {
                    if let Some(fault) = operation.stop {
                        Next::End(git::Done::Failed { fault })
                    } else {
                        let op = operation.op.as_ref().expect("item operation");
                        let program = crate::invocation::item_program(op).expect("item version check");
                        self.checked = Some((Box::from(program), version));
                        operation.ran = false;
                        if usable(version) {
                            sequence::first(op)
                        } else {
                            Next::End(git::Done::Failed { fault: git::Fault::Broken })
                        }
                    }
                } else {
                    next
                };
                let next = match operation.after_version.take() {
                    Some(command) => {
                        operation.command = *command;
                        if let Next::Store = next { Next::Run(operation.command.clone()) } else { next }
                    }
                    None => next,
                };
                self.advance(id, next, out);
                self.maintain(false, out);
            }
        }
        self.wake_stores(now, out);
    }
    fn retire(&mut self, id: Id<Operation>) {
        let operation = self.slots.get_mut(id).expect("live index");
        if operation.waiting() {
            self.store_waiters.remove(&operation.admission).expect("retiring a queued waiter");
        }
        operation.store_lock = None;
        operation.identity = None;
        operation.op = None;
        operation.command = Command::Version;
        operation.file = None;
        let key = operation.io_owner;
        self.index.remove(&key).expect("live index");
        self.slots.retire(id);
    }
    #[expect(
        clippy::too_many_lines,
        reason = "the operation dispatcher includes file and process steps for stores and alternates"
    )]
    fn advance(&mut self, id: Id<Operation>, next: Next, out: &mut Outputs) {
        let next = match next {
            Next::End(done) => {
                let operation = self.slots.get(id).expect("live operation");
                if operation.scan_index.is_some() {
                    self.finish(id, done, out);
                    return;
                }
                if let Command::Store { directory, .. } = &operation.command {
                    Next::InstallStore { directory: directory.clone(), done: Box::new(done) }
                } else if !operation.viewed {
                    sequence::ending(operation.op.as_ref().expect("operation terminal"), done)
                } else {
                    Next::End(done)
                }
            }
            Next::InitView { done, format } => {
                let operation = self.slots.get_mut(id).expect("view operation");
                operation.view_done = Some(done);
                let command = match format {
                    Some(format) => Command::Clone {
                        directory: crate::invocation::place(
                            sequence::view_place(operation.op.as_ref().expect("view operation")).expect("view place"),
                        ),
                        format,
                        bare: false,
                    },
                    None => Command::Format,
                };
                Next::Run(command)
            }
            Next::Snapshot { parent, merging } => {
                let operation = self.slots.get_mut(id).expect("live snapshot");
                let Some(git::Op::Commit { parent: target, merging: second, always, .. }) = &mut operation.op else {
                    unreachable!("snapshot sequence belongs to a commit")
                };
                if *target != parent {
                    *always = false;
                }
                *target = parent;
                *second = merging;
                let command = if merging.is_some() { Command::Unmerged } else { Command::Add };
                Next::Run(command)
            }
            other @ (Next::Results { .. }
            | Next::Reds { .. }
            | Next::Run(_)
            | Next::Make
            | Next::Look { .. }
            | Next::Store
            | Next::InstallStore { .. }
            | Next::View { .. }
            | Next::RemakeView { .. }
            | Next::Alternates { .. }
            | Next::Scan { .. }) => other,
        };
        match next {
            Next::Run(command) => {
                if command == Command::Add {
                    let operation = self.slots.get_mut(id).expect("live scan");
                    if let Some(directory) = operation.scan_index.take() {
                        let task = Task::remove_index(directory, Next::Run(command), operation.word);
                        self.file_start(id, task, out);
                        return;
                    }
                }
                if !self.store_step(id, &command) {
                    self.slots.get_mut(id).expect("live waiter").wait(command);
                    return;
                }
                let operation = self.slots.get(id).expect("live index");
                if let Command::Store { .. } = &command
                    && let Some(program) =
                        crate::invocation::item_program(operation.op.as_ref().expect("Store operation"))
                {
                    let checked = match &self.checked {
                        Some((checked, answer)) if checked.as_ref() == program => Some(usable(*answer)),
                        Some(_) | None => None,
                    };
                    match checked {
                        Some(true) => {}
                        Some(false) => {
                            self.slots.get_mut(id).expect("live operation").command = command;
                            self.advance(id, Next::End(git::Done::Failed { fault: git::Fault::Broken }), out);
                            return;
                        }
                        None => {
                            self.slots.get_mut(id).expect("live operation").after_version = Some(Box::new(command));
                            self.advance(id, Next::Run(Command::Version), out);
                            return;
                        }
                    }
                }
                if operation.program(&self.settings).is_none() {
                    let op = operation.op.as_ref().expect("no program is an operation failure");
                    let diagnostic = crate::invocation::no_program(op, &command);
                    let next = sequence::after(op, &command, 128, b"", &diagnostic, 0, &self.checkout);
                    let operation = self.slots.get_mut(id).expect("live operation");
                    operation.word = Some(crate::invocation::word(&command));
                    operation.command = command;
                    self.advance(id, next, out);
                    return;
                }
                let Some(token) = self.token() else {
                    self.finish(
                        id,
                        git::Done::FailedWithOutput {
                            fault: git::Fault::Broken,
                            diagnostic: git::PushDiagnostic::empty(),
                        },
                        out,
                    );
                    return;
                };
                let operation = self.slots.get_mut(id).expect("live index");
                let old = operation.io_owner;
                self.index.remove(&old).expect("live index");
                operation.spawn(token, command, &self.settings, &self.limits, out);
                assert!(self.index.insert(token, id).expect("same operation").is_none(), "unique token");
            }
            Next::Snapshot { .. } | Next::InitView { .. } => {
                unreachable!("continuations were resolved before dispatch")
            }
            Next::End(done) => self.finish(id, done, out),
            Next::Make => {
                let Some(git::Op::Make { workspace, stores }) = &self.slots.get(id).expect("live index").op else {
                    unreachable!("Make sequence")
                };
                self.file_start(id, Task::make(*workspace, stores.clone()), out);
            }
            Next::Look { directory } => self.file_start(id, Task::look(directory), out),
            Next::Store => {
                let Some(git::Op::Store { store, aside, remote, .. }) = &self.slots.get(id).expect("live index").op
                else {
                    unreachable!("Store sequence")
                };
                let task = Task::store(*store, *aside, remote.clone());
                self.file_start(id, task, out);
            }
            Next::InstallStore { directory, done } => {
                let operation = self.slots.get_mut(id).expect("live index");
                let Some(git::Op::Store { store, .. }) = &operation.op else { unreachable!("Store sequence") };
                // Keep a clone's failure named while its temporary directory is discarded.
                let done = Box::new(operation.named(*done));
                let task = Task::install(*store, &directory, done);
                operation.command = Command::Version;
                self.file_start(id, task, out);
            }
            Next::View { done, retry } => {
                self.slots.get_mut(id).expect("view operation").view_done = None;
                self.view_start(id, done, retry, false, None, out);
            }
            Next::RemakeView { done, format } => self.view_start(id, done, true, true, format, out),
            Next::Alternates { format } => {
                let Some(git::Op::Clone { at, store, remote } | git::Op::Look { at, store, remote }) =
                    &self.slots.get(id).expect("live index").op
                else {
                    unreachable!("Clone sequence")
                };
                self.file_start(id, Task::alternates(at, *store, remote.clone(), format), out);
            }
            Next::Reds { directory, content, marker, rerun } => {
                self.file_start(id, Task::reds(directory, content, marker, rerun), out);
            }
            Next::Results { directory } => self.file_start(id, Task::results(directory), out),
            Next::Scan { files, cut } => {
                let op = self.slots.get(id).expect("live index").op.as_ref().expect("Commit op");
                let at = match op {
                    git::Op::Commit { at, .. } => at,
                    git::Op::Reds { .. }
                    | git::Op::Reset { .. }
                    | git::Op::Pack { .. }
                    | git::Op::Point { .. }
                    | git::Op::Store { .. }
                    | git::Op::Make { .. }
                    | git::Op::Clone { .. }
                    | git::Op::Fetch { .. }
                    | git::Op::Counts { .. }
                    | git::Op::Create { .. }
                    | git::Op::CheckOut { .. }
                    | git::Op::Look { .. }
                    | git::Op::DiffPaths { .. }
                    | git::Op::Brings { .. }
                    | git::Op::IsAncestor { .. }
                    | git::Op::Merge { .. }
                    | git::Op::Push { .. } => unreachable!("scan sequence"),
                };
                let directory = crate::invocation::git_dir(at);
                let place = crate::invocation::place(at);
                let operation = self.slots.get_mut(id).expect("live scan");
                if cut && operation.scan_index.is_none() {
                    operation.scan_index = Some(directory.clone());
                    self.file_start(id, Task::copy_index(directory, Next::Scan { files, cut }), out);
                } else {
                    self.file_start(id, Task::scan(files, place, cut), out);
                }
            }
        }
    }
    fn view_start(
        &mut self,
        id: Id<Operation>,
        done: Box<git::Done>,
        retry: bool,
        remake: bool,
        format: Option<crate::invocation::Format>,
        out: &mut Outputs,
    ) {
        let operation = self.slots.get_mut(id).expect("view operation");
        operation.viewed = true;
        let at = sequence::view_place(operation.op.as_ref().expect("view operation")).expect("view place");
        let task = Task::view(at, done, retry, remake, format);
        self.file_start(id, task, out);
    }
    /// Every terminal with output names where it ended
    /// (temper/docs/design/protocol/git.md, section 6).
    fn finish(&mut self, id: Id<Operation>, done: git::Done, out: &mut Outputs) {
        let operation = self.slots.get_mut(id).expect("live index");
        if let Some(directory) = operation.scan_index.take() {
            let done = operation.named(done);
            let task = Task::remove_index(directory, Next::End(done), operation.word);
            self.file_start(id, task, out);
            return;
        }
        if !operation.aborted {
            let done = operation.named(done);
            event(out, Event::Done { owner: operation.owner, done });
        }
        self.retire(id);
    }
    fn submit(&mut self, task: &mut Task, kind: kernel::Op, out: &mut Outputs) {
        task.pending = self.token().expect("file token lifetime");
        out.kernel.push(kernel::Submit { op: task.pending, kind });
    }
    fn background(&mut self, mut task: Task, out: &mut Outputs) {
        let action = task.first(&mut self.directories, &self.settings, &self.limits);
        match action {
            Action::Run(kind) => self.background_run(task, kind, out),
            Action::Closed => {}
            Action::Layout(_) | Action::Swept { .. } | Action::Operation(_) => unreachable!("background first step"),
        }
    }
    fn background_run(&mut self, mut task: Task, kind: kernel::Op, out: &mut Outputs) {
        self.submit(&mut task, kind, out);
        self.background = Some(task);
    }
    fn background_action(&mut self, now: Time, mut task: Task, action: Action, out: &mut Outputs) {
        if let Some(entry) = task.take_set_aside() {
            // A nonempty parent of a stuck child is the same failure, not a second one.
            if entry.report {
                self.sweep_failures = self.sweep_failures.saturating_add(1);
            }
            self.sweep_report(entry, out);
        }
        match action {
            Action::Run(kind) => self.background_run(task, kind, out),
            Action::Layout(result) => {
                if self.start != Start::Aborted {
                    match result {
                        Ok(()) => {
                            self.start = Start::Started;
                            event(out, Event::Started);
                        }
                        Err(error) => {
                            self.start = Start::Unusable;
                            event(out, Event::Unusable { why: Unusable::Layout { error } });
                        }
                    }
                }
                self.maintain(result.is_ok(), out);
            }
            Action::Swept { changed, set_aside, refused_removal } => {
                self.sweep_refused_removal = refused_removal;
                self.sweep_ended_serial = Some(self.directories.serial);
                self.sweep_end(now, changed, set_aside, out);
            }
            Action::Closed => {}
            Action::Operation(_) => unreachable!("background file task"),
        }
    }
    fn sweep_report(&mut self, entry: SetAside, out: &mut Outputs) {
        let SetAside { path, error, report } = entry;
        for held in self.sweep_reports.iter_mut() {
            if held.path == path {
                held.seen = true;
                if report && held.error != error {
                    held.error = error;
                    event(out, Event::Unswept { path, error });
                }
                return;
            }
        }
        if report && self.sweep_reports.room() > 0 {
            self.sweep_reports.push(Reported { path: path.clone(), error, seen: true }).expect("report fits");
            event(out, Event::Unswept { path, error });
        }
    }
    fn sweep_end(&mut self, now: Time, changed: bool, set_aside: bool, out: &mut Outputs) {
        // Walk backwards so each swap removes an unseen entry without revisiting the tail.
        for index in (0..self.sweep_reports.len()).rev() {
            let held = self.sweep_reports.get_mut(index).expect("live report");
            if held.seen {
                held.seen = false;
            } else {
                self.sweep_reports.swap_remove(index).expect("live report");
            }
        }
        let again = core::mem::replace(&mut self.sweep_again, false) || changed;
        if !self.shutting_down && !again && set_aside {
            self.sweep_at = Some(now.saturating_add(SWEEP_RETRY));
        }
        self.maintain(again, out);
    }
    fn maintain(&mut self, sweep: bool, out: &mut Outputs) {
        let sweep = sweep && !self.shutting_down && self.start == Start::Started;
        if self.background.is_some() {
            self.sweep_again = self.sweep_again || sweep;
            return;
        }
        if (self.shutting_down || self.start == Start::Unusable) && self.index.is_empty() {
            self.background(Task::closing(), out);
        } else if sweep {
            self.sweep_at = None;
            self.background(Task::sweep(), out);
        }
    }
    fn file_start(&mut self, id: Id<Operation>, mut task: Task, out: &mut Outputs) {
        self.slots.get_mut(id).expect("live operation").word = task.word;
        let action = task.first(&mut self.directories, &self.settings, &self.limits);
        self.file_action(id, task, action, out);
    }
    fn file_action(&mut self, id: Id<Operation>, mut task: Task, action: Action, out: &mut Outputs) {
        match action {
            Action::Run(kind) => {
                self.submit(&mut task, kind, out);
                let operation = self.slots.get_mut(id).expect("live index");
                self.index.remove(&operation.io_owner).expect("live index");
                operation.io_owner = task.pending;
                assert!(self.index.insert(task.pending, id).expect("same operation").is_none(), "unique token");
                operation.file = Some(task);
            }
            Action::Operation(mut next) => {
                let operation = self.slots.get(id).expect("live index");
                if let Some(fault) = operation.stop {
                    next = if operation.ran {
                        sequence::failed(fault, b"", 0)
                    } else {
                        Next::End(git::Done::Failed { fault })
                    };
                }
                let make = operation.op.as_ref().expect("file op").kind() == git::Kind::Make;
                self.advance(id, next, out);
                self.maintain(make, out);
            }
            Action::Layout(_) | Action::Swept { .. } | Action::Closed => unreachable!("operation file task"),
        }
    }
    /// Settle owned kernel work before advancing or answering its caller.
    pub fn complete(&mut self, now: Time, complete: kernel::Complete, out: &mut Outputs) {
        let background = match &self.background {
            Some(task) => task.pending == complete.op,
            None => false,
        };
        if background {
            let mut task = self.background.take().expect("pending task");
            let action =
                task.complete(complete, &mut self.directories, &self.settings, &self.limits, self.shutting_down);
            self.background_action(now, task, action, out);
            return;
        }
        let Some(id) = self.index.get(&complete.op).copied() else {
            return;
        };
        let operation = self.slots.get_mut(id).expect("live index");
        let Some(mut task) = operation.file.take() else {
            return;
        };
        if now >= operation.deadline && operation.stop.is_none() {
            operation.stop = Some(git::Fault::TimedOut);
        }
        let action =
            task.complete(complete, &mut self.directories, &self.settings, &self.limits, operation.stop.is_some());
        self.file_action(id, task, action, out);
    }
    /// Kill at most one due child, settle due store waiters, and start a due sweep.
    pub fn fire(&mut self, now: Time, out: &mut Outputs) {
        for (_, id) in &self.index {
            let operation = self.slots.get_mut(*id).expect("live index");
            if operation.op.is_some() && operation.stop.is_none() && operation.deadline <= now {
                operation.cancel(git::Fault::TimedOut, &self.checkout, out);
                break;
            }
        }
        self.wake_stores(now, out);
        if let Some(at) = self.sweep_at
            && at <= now
        {
            self.maintain(true, out);
        }
    }
    /// Abort each child and pipe; retain a pending spawn until it can settle.
    pub fn abort(&mut self, out: &mut Outputs) {
        self.start = Start::Aborted;
        self.shutting_down = true;
        self.sweep_at = None;
        for (_, id) in &self.index {
            let operation = self.slots.get_mut(*id).expect("live index");
            operation.aborted = true;
            operation.identity = None;
            operation.cancel(git::Fault::Cancelled, &self.checkout, out);
        }
        self.wake_stores(Time::ZERO, out);
        self.maintain(false, out);
    }
    #[must_use]
    pub fn accepts(&self, event: &io::Event) -> bool {
        let Some(owner) = io_owner(event) else {
            return false;
        };
        for (_, id) in &self.index {
            if self.slots.get(*id).expect("live index").accepts(owner) {
                return true;
            }
        }
        false
    }
    #[must_use]
    pub fn owns(&self, operation: Token) -> bool {
        operation.raw().checked_sub(self.settings.prefix).unwrap_or(u64::MAX) < (1_u64 << 48_u32)
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        // Armed only between passes, while started and not shutting down.
        let mut earliest = self.sweep_at;
        for (_, id) in &self.index {
            let operation = self.slots.get(*id).expect("live index");
            if operation.op.is_some()
                && operation.stop.is_none()
                && (earliest.is_none() || Some(operation.deadline) < earliest)
            {
                earliest = Some(operation.deadline);
            }
        }
        earliest
    }
    #[must_use]
    pub fn is_due(&self, now: Time) -> bool {
        match self.next_deadline() {
            Some(deadline) => deadline <= now,
            None => false,
        }
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
            && self.background.is_none()
            && self.directories.trash.is_none()
            && self.directories.workspaces.is_none()
            && self.directories.gitdirs.is_none()
            && self.directories.stores.is_none()
            && self.directories.making.is_none()
    }
    #[must_use]
    pub fn operations(&self) -> u32 {
        limits::operations(&self.limits).expect("validated operation count")
    }
    /// The configured component bounds used to price its parent layer.
    #[must_use]
    pub fn limits(&self) -> &Limits {
        &self.limits
    }
    /// Reclaim retired state after the loop's down pass.
    pub fn reclaim(&mut self) {
        self.slots.reclaim();
    }
}
fn io_owner(event: &io::Event) -> Option<Token> {
    match event {
        io::Event::Listening { owner, .. }
        | io::Event::Accepted { owner, .. }
        | io::Event::Connecting { owner, .. }
        | io::Event::Connected { owner, .. }
        | io::Event::Stream { owner, .. }
        | io::Event::Output { owner, .. }
        | io::Event::Spawned { owner, .. }
        | io::Event::Exited { owner, .. }
        | io::Event::Usage { owner, .. }
        | io::Event::Window { owner, .. }
        | io::Event::Failed { owner, .. }
        | io::Event::Closed { owner } => Some(*owner),
        io::Event::Resized | io::Event::Shutdown { .. } => None,
    }
}
