//! The store's three single-flight file drivers, adoption and close
//! (jig/docs/design/protocol/store.md, 13).
use skein_io::{
    file,
    file_layer::{self, FileIo},
    kernel::{Complete, Fd, Submit},
};
use skein_lib::{Duration, Env, Map, Queue, Time, Token};

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
#[derive(Debug)]
pub struct Submission {
    pub lane: Lane,
    pub submit: Submit,
}

/// One single-flight file driver and its owned handles.
#[derive(Debug)]
struct Driver {
    io: FileIo,
    above: Queue<file::Event>,
    handles: Map<Token, ()>,
}
impl Driver {
    fn down(&mut self, now: Time, request: file::Request, submitted: &mut Queue<Submit>) {
        if let file::Request::Close { file, .. } = &request {
            self.handles.remove(file);
        }
        file_layer::down(&mut self.io, now, request, &mut self.above, submitted);
    }
    fn complete(&mut self, complete: Complete, submitted: &mut Queue<Submit>) {
        file_layer::up(&mut self.io, complete, &mut self.above, submitted);
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

#[derive(Debug)]
pub struct Host {
    below: crate::Below,
    drivers: [Driver; 3],
    submitted: Queue<Submission>,
    adopted: bool,
    root: Option<Fd>,
    owner: Token,
    installation: [u8; 16],
    closing: bool,
    seeded: crate::Seeded,
}
impl Host {
    #[must_use]
    pub fn new(limits: &crate::Limits, value: u64) -> Self {
        let mut drivers = [driver(limits), driver(limits), driver(limits)];
        let seeded = seed(&mut drivers, value);
        Self {
            below: crate::Below {
                store: Queue::uncapped(),
                secrets: Queue::uncapped(),
                transcripts: Queue::uncapped(),
            },
            drivers,
            submitted: Queue::with_capacity(KERNEL_HANDOFF_ENTRIES),
            adopted: false,
            root: None,
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
        if self.adopted || self.closing {
            return Err(fd);
        }
        self.adopted = true;
        self.owner = owner;
        self.installation = installation;
        for index in 0..3 {
            let driver = self.drivers.get_mut(index).expect("driver index");
            driver.io = core::mem::replace(&mut driver.io, FileIo::new(1, 1, 1, Duration::from_secs(1)))
                .with_effective_user(user);
            if index == 0 {
                let file = driver.io.adopt_root(fd).expect("deployment root slot");
                driver.handles.insert(file, ()).expect("owned deployment root");
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
            file_layer::expire(&mut self.drivers.get_mut(index).expect("driver index").io, env.now, &mut submitted);
            for _ in 0..self.drivers.get(index).expect("driver index").above.len() {
                let crate::Arrived { event, fd } =
                    self.drivers.get_mut(index).expect("driver index").pop().expect("file terminal");
                if event.owner() == CLOSE {
                    continue;
                }
                if event.owner() == ADOPT {
                    if fd.is_none() {
                        if self.root.take().is_some() {
                            above
                                .push(crate::Event::OpenRefused { owner: self.owner, why: crate::OpenRefusal::Failed });
                        }
                    } else if !self.drivers.get(1).expect("driver index").handles.is_empty()
                        && !self.drivers.get(2).expect("driver index").handles.is_empty()
                        && let Some(fd) = self.root.take()
                    {
                        crate::down(
                            component,
                            env,
                            crate::Request::Open {
                                owner: self.owner,
                                root: crate::Root { token: Token::new(1), fd },
                                installation: self.installation,
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
    let io = FileIo::with_whole_limit(handles(limits), max_read(limits), 32, max_whole(limits), limits.store.deadline);
    Driver { io, above: Queue::uncapped(), handles: Map::with_capacity(handles(limits)) }
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
