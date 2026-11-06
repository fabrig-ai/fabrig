//! The builder's two passes and reclaim point (skein/docs/foundation/programming-model.md, 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
pub use hearth_domain_builder as domain;
pub use hearth_protocol_builder as protocol;
pub mod limits;
use domain::{
    Builder,
    vocabulary::{Call, Event, Request},
};
pub use limits::{Files, Limits};
use protocol::{Asked, Protocol, Told};
use skein_io::{
    self as io, Io, file,
    file_layer::{self, FileIo},
    kernel::{Complete, Fd, Submit},
};
use skein_lib::{Env, Queue, Time, Wall};
/// Whether these limits are representable: every edge bound they size is
/// finite (skein/docs/foundation/programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    protocol::limits::valid(&limits.protocol) && domain::valid(&limits.domain)
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Configuration {
    pub proxy: Option<Box<[u8]>>,
    pub store: Box<[u8]>,
    pub limits: Limits,
    pub uid: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Roots {
    pub store: Fd,
    pub cache: Fd,
    pub resolutions: Fd,
    pub staging: Fd,
}
#[derive(Debug)]
pub struct Service {
    io: Io,
    files: FileIo,
    file_told: Queue<file::Event>,
    io_submissions: Queue<Submit>,
    file_submissions: Queue<Submit>,
    io_completions: Queue<Complete>,
    io_asked: Queue<io::Request>,
    io_env: Env<io::Limits>,
    protocol: Protocol,
    protocol_env: Env<protocol::limits::Limits>,
    domain: Builder,
    domain_env: Env<domain::Limits>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    told: Queue<io::Event>,
    queue_limit: u32,
    asked: Queue<Asked>,
    calls: Queue<Event>,
    answers: Queue<Request>,
}
impl Service {
    /// The domain state, excluding protocol machinery and environments.
    #[must_use]
    pub const fn domain(&self) -> &Builder {
        &self.domain
    }
    #[must_use]
    pub fn new(configuration: &Configuration, roots: Roots, call: Call, form: protocol::Form, seed: u64) -> Self {
        let limits = &configuration.limits;
        assert!(limits.io.is_usable() && limits.queue >= 3, "usable io and queues holding every MAX_OUT");
        assert!(valid(limits), "representable limits");
        let mut files = FileIo::with_whole_limit(
            limits.files.files,
            limits.files.read,
            limits.files.entries,
            limits.files.whole,
            limits.files.timeout,
        );
        files.seed_randomness(seed);
        let roots = protocol::Roots {
            store: protocol::Root { fd: roots.store, token: files.adopt_root(roots.store).expect("store root slot") },
            cache: protocol::Root { fd: roots.cache, token: files.adopt_root(roots.cache).expect("cache root slot") },
            resolutions: protocol::Root {
                fd: roots.resolutions,
                token: files.adopt_root(roots.resolutions).expect("resolutions root slot"),
            },
            staging: protocol::Root {
                fd: roots.staging,
                token: files.adopt_root(roots.staging).expect("staging root slot"),
            },
        };
        Self {
            io: Io::new(&limits.io),
            files,
            file_told: Queue::uncapped(),
            io_submissions: Queue::uncapped(),
            file_submissions: Queue::uncapped(),
            io_completions: Queue::uncapped(),
            io_asked: Queue::uncapped(),
            io_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.io },
            protocol: Protocol::new(&limits.protocol, call, form, roots, &configuration.store, seed)
                .with_proxy(configured_proxy(configuration))
                .with_file_read(limits.files.read),
            protocol_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.protocol },
            domain: Builder::new(&limits.domain, configuration.uid),
            domain_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.domain },
            completions: Queue::with_capacity(limits.queue),
            submissions: Queue::with_capacity(limits.queue),
            told: Queue::uncapped(),
            queue_limit: limits.queue,
            asked: Queue::with_capacity(limits.queue),
            calls: Queue::with_capacity(limits.queue),
            answers: Queue::with_capacity(limits.queue),
        }
    }
    pub fn adopt_output_pipe(&mut self, fd: Fd) -> Result<(), Fd> {
        if self.protocol.output_bound() {
            return Err(fd);
        }
        let output = self.io.adopt_write_pipe(fd, None)?;
        self.protocol.bind_output(output);
        Ok(())
    }
    pub fn adopt_output_file(&mut self, fd: Fd) -> Result<(), Fd> {
        if self.protocol.output_bound() {
            return Err(fd);
        }
        let output = self.io.adopt_append(fd, self.io_env.limits.close_timeout)?;
        self.protocol.bind_output(output);
        Ok(())
    }
    pub fn adopt_error_pipe(&mut self, fd: Fd) -> Result<(), Fd> {
        if self.protocol.error_bound() {
            return Err(fd);
        }
        let output = self.io.adopt_write_pipe(fd, None)?;
        self.protocol.bind_error(output);
        Ok(())
    }
    pub fn adopt_error_file(&mut self, fd: Fd) -> Result<(), Fd> {
        if self.protocol.error_bound() {
            return Err(fd);
        }
        let output = self.io.adopt_append(fd, self.io_env.limits.close_timeout)?;
        self.protocol.bind_error(output);
        Ok(())
    }
    pub fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    pub fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    #[must_use]
    pub fn take_startup(&mut self) -> Option<Box<[u8]>> {
        self.protocol.take_startup()
    }
    #[must_use]
    pub fn code(&self) -> Option<u8> {
        if self.is_empty() { self.protocol.code() } else { None }
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.io.is_empty()
            && self.files.takes()
            && self.files.open_files() == 4
            && self.file_told.is_empty()
            && self.file_submissions.is_empty()
            && self.io_submissions.is_empty()
            && self.io_completions.is_empty()
            && self.io_asked.is_empty()
            && self.protocol.is_empty()
            && self.domain.is_empty()
            && self.completions.is_empty()
            && self.submissions.is_empty()
            && self.told.is_empty()
            && self.asked.is_empty()
            && self.calls.is_empty()
            && self.answers.is_empty()
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        let protocol_ready = self.protocol.work_pending();
        let domain_ready = self.domain.work_pending();
        self.io.is_ready()
            || self.io.is_due(now)
            || self.files.is_due(now)
            || ((!self.file_submissions.is_empty() || !self.io_submissions.is_empty()) && self.submissions.room() > 0)
            || !self.io_completions.is_empty()
            || (!self.io_asked.is_empty() && self.io.takes())
            || (protocol_ready && protocol_room(self, protocol::MAX_OUT_RESUME))
            || (protocol_room(self, protocol::MAX_OUT_RESUME)
                && match protocol::next_deadline(&self.protocol) {
                    Some(at) => at <= now,
                    None => false,
                })
            || (!protocol_ready
                && protocol_room(self, protocol::MAX_OUT_UP)
                && (!self.file_told.is_empty() || !self.told.is_empty()))
            || (domain_ready && self.answers.room() >= domain::MAX_OUT_RESUME)
            || (!domain_ready
                && self.answers.room() >= domain::MAX_OUT
                && (!self.calls.is_empty()
                    || match domain::next_deadline(&self.domain) {
                        Some(at) => at <= now,
                        None => false,
                    }))
            || !self.completions.is_empty()
            || (!self.asked.is_empty() && self.io_asked.is_empty() && self.files.takes())
            || (!self.answers.is_empty() && self.protocol.takes() && protocol_room(self, protocol::MAX_OUT_DOWN))
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        [
            self.io.next_deadline(),
            self.files.next_deadline(),
            domain::next_deadline(&self.domain),
            protocol::next_deadline(&self.protocol),
        ]
        .into_iter()
        .flatten()
        .min()
    }
}
#[must_use]
pub fn operations(limits: &Limits) -> Option<u32> {
    io::operations(&limits.io)?.checked_add(2)
}
pub fn iterate(service: &mut Service, now: Time, wall: Wall) {
    service.io_env.now = now;
    service.io_env.wall = wall;
    service.protocol_env.now = now;
    service.protocol_env.wall = wall;
    service.domain_env.now = now;
    service.domain_env.wall = wall;
    io_up(service);
    protocol_up(service);
    domain_stage(service);
    protocol_down(service);
    io_down(service);
    service.io.reclaim();
    for _ in 0..service.submissions.capacity() {
        if service.submissions.room() == 0 {
            break;
        }
        if let Some(submit) = service.io_submissions.pop() {
            service.submissions.push(submit);
        } else if let Some(submit) = service.file_submissions.pop() {
            service.submissions.push(submit);
        } else {
            break;
        }
    }
}
fn io_up(service: &mut Service) {
    for _ in 0..service.completions.len() {
        let Some(complete) = service.completions.pop() else {
            break;
        };
        if file_layer::owns(complete.op) {
            file_layer::up(&mut service.files, complete, &mut service.file_told, &mut service.file_submissions);
        } else {
            service.io_completions.push(complete);
        }
    }
    io::stage::up(
        &mut service.io,
        &service.io_env,
        &mut service.io_completions,
        &mut service.told,
        &mut service.io_submissions,
        service.io_env.now,
    );
    if service.files.is_due(service.io_env.now) {
        file_layer::expire(&mut service.files, service.io_env.now, &mut service.file_submissions);
    }
}
fn protocol_up(service: &mut Service) {
    if protocol_room(service, protocol::MAX_OUT_RESUME) {
        protocol::fire(&mut service.protocol, &service.protocol_env, &mut service.calls, &mut service.asked);
    }
    for _ in 0..service.queue_limit {
        if !service.protocol.work_pending() || !protocol_room(service, protocol::MAX_OUT_RESUME) {
            break;
        }
        let marks = (service.calls.len(), service.asked.len());
        protocol::resume(&mut service.protocol, &service.protocol_env, &mut service.calls, &mut service.asked);
        protocol_within(service, marks, protocol::MAX_OUT_RESUME);
    }
    if service.protocol.work_pending() {
        return;
    }
    for _ in 0..service.file_told.len() {
        if service.protocol.work_pending() || !protocol_room(service, protocol::MAX_OUT_UP) {
            break;
        }
        let Some(event) = service.file_told.pop() else {
            break;
        };
        protocol::up(
            &mut service.protocol,
            &service.protocol_env,
            Told::File(event),
            &mut service.calls,
            &mut service.asked,
        );
    }
    for _ in 0..service.queue_limit {
        if service.protocol.work_pending() || !protocol_room(service, protocol::MAX_OUT_UP) {
            break;
        }
        let Some(event) = service.told.pop() else {
            break;
        };
        let marks = (service.calls.len(), service.asked.len());
        protocol::up(
            &mut service.protocol,
            &service.protocol_env,
            Told::Io(event),
            &mut service.calls,
            &mut service.asked,
        );
        protocol_within(service, marks, protocol::MAX_OUT_UP);
    }
}
fn domain_stage(service: &mut Service) {
    for _ in 0..service.calls.capacity() {
        if !service.domain.work_pending() || service.answers.room() < domain::MAX_OUT_RESUME {
            break;
        }
        let before = service.answers.len();
        domain::resume(&mut service.domain, &service.domain_env, &mut service.answers);
        within(before, service.answers.len(), domain::MAX_OUT_RESUME);
    }
    if service.domain.work_pending() {
        return;
    }
    for _ in 0..service.calls.capacity() {
        if service.answers.room() < domain::MAX_OUT {
            break;
        }
        let Some(event) = service.calls.pop() else {
            break;
        };
        let before = service.answers.len();
        domain::step(&mut service.domain, &service.domain_env, event, &mut service.answers);
        within(before, service.answers.len(), domain::MAX_OUT);
    }
    if service.calls.is_empty() && service.answers.room() >= domain::MAX_OUT {
        domain::fire(&mut service.domain, &service.domain_env, &mut service.answers);
    }
}
fn protocol_down(service: &mut Service) {
    for _ in 0..service.answers.capacity() {
        if !service.protocol.takes() || !protocol_room(service, protocol::MAX_OUT_DOWN) {
            break;
        }
        let Some(request) = service.answers.pop() else {
            break;
        };
        let marks = (service.calls.len(), service.asked.len());
        protocol::down(&mut service.protocol, &service.protocol_env, request, &mut service.calls, &mut service.asked);
        protocol_within(service, marks, protocol::MAX_OUT_DOWN);
    }
}
fn io_down(service: &mut Service) {
    io::stage::down(&mut service.io, &service.io_env, &mut service.io_asked, &mut service.io_submissions);
    for _ in 0..service.asked.len() {
        if !service.io_asked.is_empty() || !service.files.takes() {
            break;
        }
        let Some(request) = service.asked.pop() else {
            break;
        };
        match request {
            Asked::Io(request) => {
                service.io_asked.push(request);
                io::stage::down(&mut service.io, &service.io_env, &mut service.io_asked, &mut service.io_submissions);
            }
            Asked::Beneath(request) => {
                let request = beneath(&service.files, request);
                file_layer::down(
                    &mut service.files,
                    service.io_env.now,
                    request,
                    &mut service.file_told,
                    &mut service.file_submissions,
                );
            }
            Asked::File(request) => {
                file_layer::down(
                    &mut service.files,
                    service.io_env.now,
                    request,
                    &mut service.file_told,
                    &mut service.file_submissions,
                );
            }
        }
    }
}
fn protocol_room(service: &Service, max: protocol::MaxOut) -> bool {
    service.calls.room() >= max.above && service.asked.room() >= max.below
}
fn protocol_within(service: &Service, (up, down): (u32, u32), max: protocol::MaxOut) {
    within(up, service.calls.len(), max.above);
    within(down, service.asked.len(), max.below);
}
fn within(before: u32, after: u32, max: u32) {
    assert!(after.checked_sub(before).expect("steps add to outputs") <= max, "an entry point keeps its MAX_OUT");
}
#[cfg(test)]
mod tests;

fn beneath(files: &FileIo, request: protocol::Beneath) -> file::Request {
    match request {
        protocol::Beneath::Rename { owner, from_root, from, to_root, to } => file::Request::Rename {
            owner,
            from_root: files.descriptor(from_root).expect("lists directory token"),
            from,
            to_root,
            to,
        },
        protocol::Beneath::OpenDirectory { owner, root, name, no_follow } => file::Request::OpenDirectory {
            owner,
            root: files.descriptor(root).expect("walk directory token"),
            name,
            no_follow,
        },
        protocol::Beneath::Remove { owner, root, name, directory } => file::Request::Remove {
            owner,
            root: files.descriptor(root).expect("walk directory token"),
            name,
            directory,
        },
    }
}

fn configured_proxy(configuration: &Configuration) -> Option<protocol::Proxy> {
    Some(
        protocol::Proxy::new(configuration.proxy.as_deref()?, &configuration.limits.protocol.mmdebstrap)
            .expect("validated proxy"),
    )
}
