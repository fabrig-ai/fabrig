//! The store's three single-flight file drivers, adoption and close
//! (jig/docs/design/protocol/store/README.md, 8; jig/docs/design/protocol/service.md, 11.5).
use crate::Below;
use skein_io::{
    file,
    file_layer::{FileIo, Limits as FileLimits, step},
    kernel::{Complete, Fd, Submit},
};
use skein_lib::{Duration, Env, Map, Queue, Time, Token, Wall};
mod wait;
pub use wait::{Wait, WaitLane};

/// Whether a get the store can't answer yet is answered in the pass (protocol/service.md, 11.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotReady {
    /// `Hosted::secrets` returns it, as any answer to a get.
    InPass,
    /// It stays in `above`, for the owner's dispatch.
    Above,
}

#[expect(clippy::partial_pub_fields, reason = "the owner dispatches terminals and reads the component")]
pub struct Hosted {
    pub component: crate::Store,
    /// The store's terminals, in order, for the owner's dispatch.
    pub above: Queue<crate::Event>,
    /// Terminals the owner's dispatch hands its clients' side.
    pub secrets: Queue<crate::Event>,
    /// The answers to the accounts' records' loads and writes (protocol/store/secrets.md).
    pub sources: Queue<crate::Event>,
    env: Env<crate::Limits>,
    drivers: Drivers,
    not_ready: NotReady,
    closing: bool,
}
// A terminal can hold a kept digest, so Debug counts the queues without formatting their values.
impl core::fmt::Debug for Hosted {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Hosted")
            .field("above", &self.above.len())
            .field("secrets", &self.secrets.len())
            .field("sources", &self.sources.len())
            .field("closing", &self.closing)
            .finish_non_exhaustive()
    }
}
/// Operation issued once its roots are adopted (protocol/store/opening.md, 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Adoption {
    Open,
    Rehearse { scratch: Fd },
    Back,
}

impl Hosted {
    #[must_use]
    pub fn new(
        limits: &crate::Limits,
        application: crate::sizes::Application,
        models: &[jig_domain_core_vocabulary::Model],
        seed: u64,
        not_ready: NotReady,
    ) -> Self {
        let drivers = Drivers::new(limits, seed);
        Self {
            component: crate::Store::on_directory(limits, application, models, drivers.seeded()),
            above: Queue::uncapped(),
            secrets: Queue::uncapped(),
            sources: Queue::uncapped(),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
            drivers,
            not_ready,
            closing: false,
        }
    }
    #[must_use]
    pub fn limits(&self) -> &crate::Limits {
        &self.env.limits
    }
    #[must_use]
    pub fn mark_durable(&self) -> bool {
        self.component.mark_durable()
    }
    pub fn clock(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
    }
    pub fn adopt(&mut self, fd: Fd, user: u32, installation: [u8; 16]) -> Result<(), Fd> {
        self.drivers.adopt(fd, user, Token::new(1), installation)
    }
    pub fn adopt_mode(&mut self, fd: Fd, user: u32, installation: [u8; 16], mode: Adoption) -> Result<(), Fd> {
        self.drivers.adopt_mode(fd, user, Token::new(1), installation, mode)
    }
    /// Whether adoption has issued its operation, rather than failed to clone a root.
    #[must_use]
    pub fn started(&self) -> bool {
        self.drivers.started
    }
    pub fn ask(&mut self, request: crate::Request) {
        match request {
            crate::Request::Secrets { owner, request } => {
                if let Some(event) = self.secrets(owner, request) {
                    self.above.push(event);
                }
            }
            request @ (crate::Request::Switch { .. }
            | crate::Request::Rehearse { .. }
            | crate::Request::Back { .. }
            | crate::Request::Close { .. }
            | crate::Request::Open { .. }
            | crate::Request::Site { .. }
            | crate::Request::Commit { .. }
            | crate::Request::Restore { .. }
            | crate::Request::Load { .. }
            | crate::Request::Read { .. }
            | crate::Request::Get { .. }
            | crate::Request::Transcript { .. }) => {
                self.drivers.down(&mut self.component, &self.env, request, &mut self.above);
            }
        }
    }
    /// Get answers stay in this pass; secret writes' terminals are handed to their owner.
    pub fn secrets(&mut self, owner: Token, request: crate::Secrets) -> Option<crate::Event> {
        let immediate = match &request {
            crate::Secrets::Get { .. } | crate::Secrets::Load { .. } => true,
            crate::Secrets::Keep { .. }
            | crate::Secrets::Adopt { .. }
            | crate::Secrets::AdoptValue { .. }
            | crate::Secrets::Replace { .. }
            | crate::Secrets::Erase { .. }
            | crate::Secrets::Hold { .. }
            | crate::Secrets::Carry { .. } => false,
        };
        self.drivers.down(&mut self.component, &self.env, crate::Request::Secrets { owner, request }, &mut self.above);
        let mut answer = None;
        if immediate {
            for _ in 0..self.above.len() {
                let event = self.above.pop().expect("store terminals");
                if self.not_ready == NotReady::Above
                    && let crate::Event::SecretsNotReady { .. } = &event
                {
                    self.above.push(event);
                    continue;
                }
                if let crate::Event::Secret { owner: found, .. }
                | crate::Event::SecretsLoaded { owner: found, .. }
                | crate::Event::SecretsNotReady { owner: found } = &event
                    && *found == owner
                {
                    answer = Some(event);
                } else {
                    self.above.push(event);
                }
            }
        }
        answer
    }
    pub fn iterate(&mut self) {
        self.drivers.iterate(&mut self.component, &self.env, &mut self.above);
    }
    pub fn complete(&mut self, lane: Lane, complete: Complete) {
        self.drivers.complete(lane, complete);
    }
    #[must_use]
    pub fn submissions(&self) -> u32 {
        self.drivers.submissions()
    }
    pub fn pop_submission(&mut self) -> Option<Submission> {
        self.drivers.pop_submission()
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.drivers.next_deadline()
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        !self.secrets.is_empty()
            || !self.sources.is_empty()
            || !self.above.is_empty()
            || !self.closing && crate::patching::phase::ready(&self.component)
            || self.drivers.work_pending(now)
    }
    #[must_use]
    pub fn closing(&self) -> bool {
        self.closing
    }
    #[must_use]
    pub fn idle(&self) -> bool {
        self.secrets.is_empty() && self.sources.is_empty() && self.above.is_empty() && self.drivers.idle()
    }
    #[must_use]
    pub fn closed(&self) -> bool {
        self.drivers.closed()
    }
    pub fn close(&mut self) {
        self.closing = true;
        self.drivers.close();
        self.above = Queue::uncapped();
        self.secrets = Queue::uncapped();
        self.sources = Queue::uncapped();
    }
}

const ADOPT: Token = Token::new(u64::MAX - 1);
const CLOSE: Token = Token::new(u64::MAX);
/// Three drivers, each with at most four kernel records per transition.
pub const KERNEL_HANDOFF_ENTRIES: u32 = 3 * 4;

/// Independent file-operation lanes; token namespaces belong to the owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lane {
    Store,
    Secrets,
    Transcripts,
}
impl Lane {
    pub const ALL: [Self; 3] = [Self::Store, Self::Secrets, Self::Transcripts];
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Store => 0,
            Self::Secrets => 1,
            Self::Transcripts => 2,
        }
    }
    fn at(index: usize) -> Self {
        *Self::ALL.get(index).expect("driver index")
    }
}
/// An untagged kernel record and the driver that owns it.
pub struct Submission {
    pub lane: Lane,
    pub submit: Submit,
}

impl core::fmt::Debug for Submission {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Submission").field("lane", &self.lane).field("op", &self.submit.op).finish_non_exhaustive()
    }
}
/// One single-flight file driver and its owned handles.
#[derive(Debug)]
struct Driver {
    io: FileIo,
    file_limits: FileLimits,
    above: Queue<file::Event>,
    handles: Map<Token, ()>,
}
impl Driver {
    fn down(&mut self, now: Time, request: file::Request, submitted: &mut Queue<Submit>) {
        if let file::Request::Close { file, .. } = &request {
            self.handles.remove(file);
        }
        let env = Env { now, wall: Wall::EPOCH, limits: self.file_limits };
        step::down(&mut self.io, &env, request, &mut self.above, submitted);
    }
    fn complete(&mut self, complete: Complete, submitted: &mut Queue<Submit>) {
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: self.file_limits };
        step::up(&mut self.io, &env, complete, &mut self.above, submitted);
    }
    fn pop(&mut self) -> Option<crate::Arrived> {
        let event = self.above.pop()?;
        let fd = match &event {
            file::Event::Opened { file, .. } => {
                self.handles.insert(*file, ()).expect("bounded open file handles");
                self.io.descriptor(*file)
            }
            file::Event::Written { .. }
            | file::Event::Read { .. }
            | file::Event::Loaded { .. }
            | file::Event::Stored { .. }
            | file::Event::Scanned { .. }
            | file::Event::Cut { .. }
            | file::Event::Synced { .. }
            | file::Event::Closed { .. }
            | file::Event::Removed { .. }
            | file::Event::Linked { .. }
            | file::Event::Renamed { .. }
            | file::Event::Made { .. }
            | file::Event::Failed { .. }
            | file::Event::Stated { .. }
            | file::Event::Listed { .. }
            | file::Event::Locked { .. }
            | file::Event::Busy { .. }
            | file::Event::Refused { .. }
            | file::Event::Conflict { .. }
            | file::Event::TooLarge { .. }
            | file::Event::Cancelled { .. } => None,
        };
        Some(crate::Arrived { event, fd })
    }
}
/// Seed the same three drivers used for directory generation creation.
fn seed(drivers: &mut [Driver; 3], value: u64) -> crate::Seeded {
    let [store, secrets, transcripts] = drivers;
    crate::seed([&mut store.io, &mut secrets.io, &mut transcripts.io], value)
}

#[expect(clippy::struct_field_names, reason = "the drivers part holds its three file drivers")]
pub struct Drivers {
    below: Below,
    drivers: [Driver; 3],
    submitted: Queue<Submission>,
    adopted: bool,
    started: bool,
    root: Option<Fd>,
    scratch: Option<crate::Root>,
    adoption: Adoption,
    owner: Token,
    installation: [u8; 16],
    closing: bool,
    seeded: crate::Seeded,
}
impl core::fmt::Debug for Drivers {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Drivers").field(&self.submitted.len()).finish()
    }
}
impl Drivers {
    #[must_use]
    pub fn new(limits: &crate::Limits, value: u64) -> Self {
        let mut drivers = [driver(limits), driver(limits), driver(limits)];
        let seeded = seed(&mut drivers, value);
        Self {
            below: Below { store: Queue::uncapped(), secrets: Queue::uncapped(), transcripts: Queue::uncapped() },
            drivers,
            submitted: Queue::with_capacity(KERNEL_HANDOFF_ENTRIES),
            adopted: false,
            started: false,
            root: None,
            scratch: None,
            adoption: Adoption::Open,
            owner: Token::new(0),
            installation: [0; 16],
            closing: false,
            seeded,
        }
    }
    /// Kernel records waiting for the owner's token namespace and submission queue.
    #[must_use]
    pub fn submissions(&self) -> u32 {
        self.submitted.len()
    }
    pub fn pop_submission(&mut self) -> Option<Submission> {
        self.submitted.pop()
    }
    #[must_use]
    pub fn seeded(&self) -> crate::Seeded {
        self.seeded
    }
    pub fn adopt(&mut self, fd: Fd, user: u32, owner: Token, installation: [u8; 16]) -> Result<(), Fd> {
        self.adopt_mode(fd, user, owner, installation, Adoption::Open)
    }
    pub fn adopt_mode(
        &mut self,
        fd: Fd,
        user: u32,
        owner: Token,
        installation: [u8; 16],
        mode: Adoption,
    ) -> Result<(), Fd> {
        if self.adopted || self.closing {
            return Err(fd);
        }
        self.adopted = true;
        self.adoption = mode;
        self.owner = owner;
        self.installation = installation;
        for index in 0..3 {
            let driver = self.drivers.get_mut(index).expect("driver index");
            let placeholder = FileLimits { files: 1, read: 1, entries: 1, whole: 1, timeout: Duration::from_secs(1) };
            driver.io = core::mem::replace(&mut driver.io, FileIo::with_limits(&placeholder)).with_effective_user(user);
            if index == 0 {
                let file = driver.io.adopt_root(fd).expect("deployment root slot");
                driver.handles.insert(file, ()).expect("owned deployment root");
                if let Adoption::Rehearse { scratch } = mode {
                    let token = driver.io.adopt_root(scratch).expect("scratch root slot");
                    driver.handles.insert(token, ()).expect("owned scratch root");
                    self.scratch = Some(crate::Root { token, fd: scratch });
                }
            }
        }
        self.root = Some(fd);
        let clone =
            file::Request::OpenDirectory { owner: ADOPT, root: fd, name: b".".as_slice().into(), no_follow: true };
        self.below.secrets.push(clone);
        self.below.transcripts.push(file::Request::OpenDirectory {
            owner: ADOPT,
            root: fd,
            name: b".".as_slice().into(),
            no_follow: true,
        });
        if let Adoption::Rehearse { scratch } = mode {
            self.below.secrets.push(file::Request::OpenDirectory {
                owner: ADOPT,
                root: scratch,
                name: b".".as_slice().into(),
                no_follow: true,
            });
            self.below.transcripts.push(file::Request::OpenDirectory {
                owner: ADOPT,
                root: scratch,
                name: b".".as_slice().into(),
                no_follow: true,
            });
        }
        Ok(())
    }
    /// Typed store requests, with file effects retained in the host.
    pub fn down(
        &mut self,
        component: &mut crate::Store,
        env: &Env<crate::Limits>,
        request: crate::Request,
        above: &mut Queue<crate::Event>,
    ) {
        crate::down(component, env, request, above, &mut self.below);
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        let mut next: Option<Time> = None;
        for driver in &self.drivers {
            next = match (next, driver.io.next_deadline()) {
                (Some(left), Some(right)) => Some(left.min(right)),
                (left, None) => left,
                (None, right) => right,
            };
        }
        next
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        if !self.submitted.is_empty() {
            return true;
        }
        for index in 0..3 {
            let driver = self.drivers.get(index).expect("driver index");
            if driver.io.is_due(now)
                || !self.drivers.get(index).expect("driver index").above.is_empty()
                || driver.io.takes() && (!self.queue(index).is_empty() || self.can_close(index))
            {
                return true;
            }
        }
        false
    }
    fn root_count(&self) -> u32 {
        match self.adoption {
            Adoption::Open | Adoption::Back => 1,
            Adoption::Rehearse { .. } => 2,
        }
    }
    fn can_close(&self, index: usize) -> bool {
        // Root closure follows the other drivers’ handles and terminals.
        self.closing
            && !self.drivers.get(index).expect("driver index").handles.is_empty()
            && (index != 0
                || self.drivers.get(1).expect("driver index").handles.is_empty()
                    && self.drivers.get(2).expect("driver index").handles.is_empty()
                    && self.drivers.get(1).expect("driver index").io.takes()
                    && self.drivers.get(2).expect("driver index").io.takes()
                    && self.drivers.get(1).expect("driver index").above.is_empty()
                    && self.drivers.get(2).expect("driver index").above.is_empty())
    }
    #[must_use]
    pub fn closed(&self) -> bool {
        self.closing && self.idle() && empty_handles(&self.drivers)
    }
    #[must_use]
    pub fn idle(&self) -> bool {
        self.submitted.is_empty()
            && self.below.store.is_empty()
            && self.below.secrets.is_empty()
            && self.below.transcripts.is_empty()
            && idle_drivers(&self.drivers)
    }
    pub fn close(&mut self) {
        self.closing = true;
        self.root = None;
        for index in 0..3 {
            for _ in 0..self.queue(index).len() {
                let _request = self.pop(index);
            }
        }
    }
    /// Complete an operation in the driver's local token namespace.
    pub fn complete(&mut self, lane: Lane, complete: Complete) {
        let mut submitted = Queue::with_capacity(4);
        let driver = self.drivers.get_mut(lane.index()).expect("driver index");
        driver.complete(complete, &mut submitted);
        self.publish(lane.index(), &mut submitted);
    }
    fn publish(&mut self, index: usize, queue: &mut Queue<Submit>) {
        let lane = Lane::at(index);
        for _ in 0..queue.len() {
            let submit = queue.pop().expect("file submissions");
            self.submitted.push(Submission { lane, submit });
        }
    }
    fn queue(&self, index: usize) -> &Queue<file::Request> {
        if index == 0 {
            &self.below.store
        } else if index == 1 {
            &self.below.secrets
        } else {
            &self.below.transcripts
        }
    }
    fn pop(&mut self, index: usize) -> Option<file::Request> {
        if index == 0 {
            self.below.store.pop()
        } else if index == 1 {
            self.below.secrets.pop()
        } else {
            self.below.transcripts.pop()
        }
    }
    fn close_request(&mut self, index: usize) -> Option<file::Request> {
        if self.can_close(index) {
            let handles = &mut self.drivers.get_mut(index).expect("driver index").handles;
            if let Some((token, ())) = handles.iter().last() {
                let file = *token;
                handles.remove(&file);
                return Some(close(file));
            }
        }
        None
    }
    pub fn iterate(&mut self, component: &mut crate::Store, env: &Env<crate::Limits>, above: &mut Queue<crate::Event>) {
        if self.adopted && !self.closing {
            crate::fire(component, env, above, &mut self.below);
        }
        for index in 0..3 {
            let mut submitted = Queue::with_capacity(4);
            let driver = self.drivers.get_mut(index).expect("driver index");
            step::expire(&mut driver.io, &env.with(driver.file_limits), &mut submitted);
            for _ in 0..self.drivers.get(index).expect("driver index").above.len() {
                let crate::Arrived { event, fd } =
                    self.drivers.get_mut(index).expect("driver index").pop().expect("file terminal");
                if event.owner() == CLOSE {
                    continue;
                }
                if event.owner() == ADOPT {
                    if fd.is_none() {
                        if self.root.take().is_some() {
                            above.push(match self.adoption {
                                Adoption::Open | Adoption::Rehearse { .. } => {
                                    crate::Event::OpenRefused { owner: self.owner, why: crate::OpenRefusal::Failed }
                                }
                                Adoption::Back => {
                                    crate::Event::BackRefused { owner: self.owner, why: crate::BackRefusal::Unreadable }
                                }
                            });
                        }
                    } else if self.drivers.get(1).expect("driver index").handles.len() >= self.root_count()
                        && self.drivers.get(2).expect("driver index").handles.len() >= self.root_count()
                        && let Some(fd) = self.root.take()
                    {
                        self.started = true;
                        crate::down(
                            component,
                            env,
                            match self.adoption {
                                Adoption::Open => crate::Request::Open {
                                    owner: self.owner,
                                    root: crate::Root { token: Token::new(1), fd },
                                    installation: self.installation,
                                },
                                Adoption::Rehearse { .. } => crate::Request::Rehearse {
                                    owner: self.owner,
                                    directory: crate::Root { token: Token::new(1), fd },
                                    scratch: self.scratch.expect("adopted scratch"),
                                    installation: self.installation,
                                },
                                Adoption::Back => crate::Request::Back {
                                    owner: self.owner,
                                    root: crate::Root { token: Token::new(1), fd },
                                },
                            },
                            above,
                            &mut self.below,
                        );
                    }
                    continue;
                }
                let arrived = crate::Arrived { event, fd };
                let event = if index == 0 {
                    crate::Io::Store(arrived)
                } else if index == 1 {
                    crate::Io::Secrets(arrived)
                } else {
                    crate::Io::Transcripts(arrived)
                };
                if !self.closing {
                    crate::up(component, env, event, above, &mut self.below);
                }
            }
            if self.drivers.get(index).expect("driver index").io.takes() {
                let request = self.pop(index);
                let request = match request {
                    Some(request) => Some(request),
                    None => self.close_request(index),
                };
                if let Some(request) = request {
                    let driver = self.drivers.get_mut(index).expect("driver index");
                    driver.down(env.now, request, &mut submitted);
                }
            }
            self.publish(index, &mut submitted);
        }
    }
}
fn handles(limits: &crate::Limits) -> u32 {
    limits.files.checked_add(4).expect("store file handles fit")
}
/// Largest positioned read admitted by the store.
#[must_use]
pub fn max_read(limits: &crate::Limits) -> u32 {
    limits
        .store
        .chunk
        .max(limits.core.resume_bytes)
        .max(u32::try_from(limits.core.load_bytes.min(u64::from(u32::MAX))).expect("file read length"))
}
/// Whole-file room includes the manifest even when positioned reads are smaller.
#[must_use]
pub fn max_whole(limits: &crate::Limits) -> u32 {
    max_read(limits).max(crate::sizes::MANIFEST_BYTES)
}
/// Reject a page larger than every positioned-read bound.
#[must_use]
pub fn check_read_page(limits: &crate::Limits) -> Option<()> {
    if max_read(limits) < limits.store.page.bytes {
        return None;
    }
    Some(())
}
fn driver(limits: &crate::Limits) -> Driver {
    let files = handles(limits);
    let read = max_read(limits);
    let whole = max_whole(limits);
    // Each open adds a log segment, and a store that seldom snapshots keeps them all: 32 entries stopped a store
    // with few writes after its 32nd start.
    let file_limits = FileLimits { files, read, entries: 1024, whole, timeout: limits.store.deadline };
    let io = FileIo::with_limits(&file_limits);
    Driver { io, file_limits, above: Queue::uncapped(), handles: Map::with_capacity(handles(limits)) }
}
fn close(file: Token) -> file::Request {
    file::Request::Close { owner: CLOSE, file }
}

fn empty_handles(drivers: &[Driver; 3]) -> bool {
    for driver in drivers {
        if !driver.handles.is_empty() {
            return false;
        }
    }
    true
}
fn idle_drivers(drivers: &[Driver; 3]) -> bool {
    for driver in drivers {
        if !driver.io.takes() || !driver.above.is_empty() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests;
