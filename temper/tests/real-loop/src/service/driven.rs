//! Drive either prepared service while retaining the world's joined diagnostics.
use skein_io::kernel::{Complete, Fd, Submit};
use skein_lib::{Queue, Time, Wall};
use temper_box_shell::Service;
use temper_service::{End, Note};

pub(crate) struct Driven {
    pub(super) service: Service,
    worker_end: Option<End>,
}
impl Driven {
    pub(crate) fn new(service: Service) -> Self {
        let mut driven = Self { service, worker_end: None };
        driven.sync_end();
        driven
    }
    fn sync_end(&mut self) {
        if let Service::Worker(worker) = &self.service {
            self.worker_end = worker.end().cloned().map(Into::into);
        }
    }
    pub(super) fn iterate(&mut self, now: Time, wall: Wall) {
        match &mut self.service {
            Service::Joined(service) => temper_service::iterate(service, now, wall),
            Service::Worker(service) => temper_service::worker_alone::iterate(service, now, wall),
        }
        self.sync_end();
    }
    pub(crate) fn end(&self) -> Option<&End> {
        match &self.service {
            Service::Joined(service) => service.end(),
            Service::Worker(_) => self.worker_end.as_ref(),
        }
    }
    pub(crate) fn engine_domain(&self) -> Option<&temper_engine_domain::Domain> {
        match &self.service {
            Service::Joined(service) => service.engine_domain(),
            Service::Worker(_) => None,
        }
    }
    pub(crate) fn worker_domain(&self) -> Option<&temper_worker_domain::Domain> {
        match &self.service {
            Service::Joined(service) => service.worker_domain(),
            Service::Worker(service) => Some(service.worker_domain()),
        }
    }
    pub(super) fn unsettled(&self) -> &[temper_service::Unsettled] {
        match &self.service {
            Service::Joined(service) => service.unsettled(),
            Service::Worker(_) => &[],
        }
    }
    pub(super) fn take_listening(&mut self) -> Option<Note> {
        match &mut self.service {
            Service::Joined(service) => service.listening().pop(),
            Service::Worker(service) => service.notes().pop().map(Into::into),
        }
    }
    pub(super) fn take_note(&mut self) -> Option<Note> {
        match &mut self.service {
            Service::Joined(service) => service.notes().pop(),
            Service::Worker(service) => service.notes().pop().map(Into::into),
        }
    }
    pub(super) fn take_link(&mut self) -> Option<Box<[u8]>> {
        match &mut self.service {
            Service::Joined(service) => service.links().pop(),
            Service::Worker(_) => None,
        }
    }
    pub(super) fn operations(&self) -> u32 {
        self.service.operations()
    }
    pub(crate) fn pending(&self, now: Time) -> temper_service::diagnostics::Pending {
        match &self.service {
            Service::Joined(service) => service.pending(now),
            Service::Worker(service) => service.pending(now).into(),
        }
    }
    pub(crate) fn work_pending(&self, now: Time) -> bool {
        match &self.service {
            Service::Joined(service) => service.work_pending(now),
            Service::Worker(service) => service.work_pending(now),
        }
    }
    pub(super) fn is_empty(&self) -> bool {
        match &self.service {
            Service::Joined(service) => service.is_empty(),
            Service::Worker(service) => service.is_empty(),
        }
    }
    pub(super) fn completions(&mut self) -> &mut Queue<Complete> {
        match &mut self.service {
            Service::Joined(service) => service.completions(),
            Service::Worker(service) => service.completions(),
        }
    }
    pub(super) fn submissions(&mut self) -> &mut Queue<Submit> {
        match &mut self.service {
            Service::Joined(service) => service.submissions(),
            Service::Worker(service) => service.submissions(),
        }
    }
    pub(super) fn next_deadline(&self) -> Option<Time> {
        match &self.service {
            Service::Joined(service) => service.next_deadline(),
            Service::Worker(service) => service.next_deadline(),
        }
    }
    pub(super) fn next_policy_deadline(&self) -> Option<Time> {
        match &self.service {
            Service::Joined(service) => service.next_policy_deadline(),
            Service::Worker(service) => service.next_policy_deadline(),
        }
    }
    pub(super) fn check_wait_boundary(&self, now: Time) {
        match &self.service {
            Service::Joined(service) => service.check_wait_boundary(now),
            Service::Worker(service) => service.check_wait_boundary(now),
        }
    }
    pub(super) fn adopt_signals(&mut self, fd: Fd) -> Result<(), Fd> {
        match &mut self.service {
            Service::Joined(service) => service.adopt_signals(fd),
            Service::Worker(service) => service.adopt_signals(fd),
        }
    }
    pub(super) fn adopt_listener(&mut self, fd: Fd, address: std::net::SocketAddr) -> Result<(), Fd> {
        match &mut self.service {
            Service::Joined(service) => service.adopt_listener(fd, address),
            Service::Worker(service) => service.adopt_listener(fd, address),
        }
    }
    pub(super) fn shutdown(&mut self) {
        match &mut self.service {
            Service::Joined(service) => service.shutdown(),
            Service::Worker(service) => service.shutdown(),
        }
        self.sync_end();
    }
}
