//! The worker's service and vocabulary (temper/docs/design/protocol/services.md, 5).
use crate::{Worker, WorkerRuntime, settings, worker_settings};
use jig_protocol_config::{Problems, What};
use skein_io::{
    self as io,
    kernel::{Complete, Fd, Submit},
};
use skein_lib::{List, Queue, Time, Token, Wall};
use temper_service_core::process::Process;
pub use temper_service_core::{STAGES, Stage};

pub use crate::worker_vocabulary::{End, Note, Stop};

#[expect(missing_debug_implementations, reason = "the worker owns credential records")]
pub struct Service {
    pub(crate) process: Process,
    pub(crate) worker: Worker,
    pub(crate) held_listeners: List<Token>,
    pub(crate) listeners_closing: u32,
}
// The stock small profile's Prepared::room: twice its worker root output
// bound of 5,099, independent of the watermark.
const WORKER_ALONE_QUEUE: u32 = 10_198;
impl Service {
    pub(crate) fn new(
        settings: &settings::Worker,
        limits: &worker_settings::Limits,
        agent: &settings::Agent,
        files: &settings::Files,
        runtime: WorkerRuntime,
        seed: u64,
    ) -> Result<Self, Problems> {
        let io_limits = settings.jig.limits.io;
        worker_settings::prepare(settings, agent, limits)?;
        let worker_operations = worker_settings::configured_operations(&limits.git_component)
            .ok_or(worker_settings::problem(b"limits", What::Bound))?;
        let operations = Process::operations_for(&io_limits, worker_operations)
            .ok_or(worker_settings::problem(b"limits", What::Bound))?;
        Ok(Self {
            worker: Worker::new(settings, agent, limits, files, runtime, seed)?,
            process: Process::new(io_limits, operations, WORKER_ALONE_QUEUE),
            held_listeners: List::with_capacity(io_limits.sockets),
            listeners_closing: 0,
        })
    }
    pub const fn completions(&mut self) -> &mut Queue<Complete> {
        self.process.completions()
    }
    pub const fn submissions(&mut self) -> &mut Queue<Submit> {
        self.process.submissions()
    }
    #[must_use]
    pub fn operations(&self) -> u32 {
        self.process.operations()
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        self.worker.work_pending(now)
            || self.process.work_pending(now)
            || self.end().is_some()
                && (self.listeners_closing < self.held_listeners.len() || self.process.signals_pending())
    }
    #[must_use]
    pub fn next_policy_deadline(&self) -> Option<Time> {
        self.worker.next_deadline()
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        skein_lib::earliest(self.next_policy_deadline(), self.process.io().next_deadline())
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.end().is_some() && self.worker.is_empty() && self.process.is_empty()
    }
    /// A first call drains; another aborts, keeping a failure's reason.
    pub fn shutdown(&mut self) {
        if self.worker.stopping() {
            self.worker.abort();
            self.worker.set_end(End::Aborted);
        } else {
            self.worker.stop();
        }
    }
    pub fn notes(&mut self) -> &mut Queue<Note> {
        self.worker.notes()
    }
    #[must_use]
    pub fn end(&self) -> Option<&End> {
        self.worker.end()
    }
    pub fn check_wait_boundary(&self, now: Time) {
        assert!(
            self.is_empty()
                || self.work_pending(now)
                || self.next_deadline().is_some()
                || self.process.has_operations()
                || self.process.submission_count() != 0,
            "service would wait forever with nothing in flight: end={:?}; state={:?}; worker={:?}",
            self.end(),
            self.wait_state(),
            self.worker.pending(now)
        );
    }
    /// Readiness without advancing the worker or exposing payloads.
    #[must_use]
    pub fn pending(&self, now: Time) -> crate::diagnostics::Pending {
        crate::diagnostics::Pending {
            work_pending: self.work_pending(now),
            worker_runs: true,
            io_ready: self.process.io().is_ready(),
            io_due: self.process.io().is_due(now),
            completions: self.process.completion_count(),
            told: self.process.told_count(),
            raw_submissions: self.process.raw_submission_count(),
            ended: self.end().is_some(),
            signals: self.process.has_signals(),
            signals_closing: self.process.signals_closing(),
            box_pipe_pending: false,
            engine: None,
            worker: Some(self.worker.pending(now)),
        }
    }
    /// The worker domain, for a world to inspect before requesting shutdown.
    #[must_use]
    pub const fn worker_domain(&self) -> &temper_worker_domain::Domain {
        &self.worker.root
    }
    #[must_use]
    pub fn wait_state(&self) -> crate::diagnostics::Wait {
        crate::diagnostics::Wait {
            engine: None,
            worker_empty: self.worker.is_empty(),
            held_listeners: self.held_listeners.len(),
            listener_closes: self.listeners_closing,
            signals_closing: self.process.signals_closing(),
            operations: self.process.has_operations(),
            submissions: self.process.submission_count(),
        }
    }
    pub fn adopt_signals(&mut self, descriptor: Fd) -> Result<(), Fd> {
        self.process.adopt_signals(descriptor)
    }
    /// Keep an inherited listener until a matching Listen claims it or the service ends.
    pub fn adopt_listener(&mut self, fd: Fd, address: core::net::SocketAddr) -> Result<(), Fd> {
        if self.held_listeners.room() == 0 {
            return Err(fd);
        }
        match self.process.io_mut().adopt_listener(fd, address) {
            Ok(entity) => {
                self.held_listeners.push(entity).expect("one handle per admitted listener");
                Ok(())
            }
            Err(fd) => Err(fd),
        }
    }
    pub fn reuse_workspaces(&mut self) {
        self.worker.reuse_workspaces();
    }
    pub fn dump_llm(&mut self, directory: &[u8]) {
        self.worker.dump_llm(directory);
    }
    pub fn configure_program_sources(
        &mut self,
        resources: crate::program_sources::Resources,
    ) -> Result<(), crate::program_sources::Resources> {
        self.worker.configure_program_sources(resources)
    }
    fn route_events(&mut self, now: Time) {
        for _ in 0..self.process.told_count() {
            let event = self.process.told().pop().expect("io event count");
            if let io::Event::Shutdown { .. } = event {
                self.shutdown();
                continue;
            }
            if event.owner() == Some(Token::new(crate::tokens::SIGNALS)) {
                if let io::Event::Closed { .. } = event {
                    self.process.signals_closed();
                }
                continue;
            }
            if self.worker.accepts(&event) {
                self.worker.up(now, event);
            }
        }
    }
    fn close_sources(&mut self) {
        if self.end().is_none() {
            return;
        }
        for _ in self.listeners_closing..self.held_listeners.len() {
            if self.process.room() < io::MAX_OUT_DOWN.submissions {
                break;
            }
            let entity = *self.held_listeners.get(self.listeners_closing).expect("held listener");
            self.listeners_closing = self.listeners_closing.checked_add(1).expect("one close per held listener");
            self.process.down(io::Request::Close { entity });
        }
        self.process.close_signals();
    }
    fn route_requests(&mut self) {
        for _ in 0..self.worker.submitted().len() {
            if self.process.room() == 0 {
                break;
            }
            let submit = self.worker.submitted().pop().expect("worker kernel submissions");
            self.process.raw_submissions().push(submit);
        }
        for _ in 0..self.worker.io().len() {
            if self.process.room() < io::MAX_OUT_DOWN.submissions || !self.process.io().takes() {
                break;
            }
            let request = self.worker.io().pop().expect("worker io requests");
            self.process.down(request);
        }
    }
}
/// Drive the worker and its process in the shared turn's order.
pub fn iterate(service: &mut Service, now: Time, wall: Wall) {
    for stage in STAGES {
        iterate_stage(service, stage, now, wall);
    }
}
pub fn iterate_stage(service: &mut Service, stage: Stage, now: Time, wall: Wall) {
    match stage {
        Stage::Io => {
            service.process.clock(now, wall);
            service.process.resume();
            if !service.process.io().is_ready() {
                for _ in 0..service.process.completion_count() {
                    let Some(complete) = service.process.take_completion() else { break };
                    if crate::tokens::prefix(complete.op) == 3 || crate::tokens::prefix(complete.op) == 6 {
                        service.worker.complete(now, complete).expect("worker operation owner");
                    } else {
                        service.process.up(complete);
                    }
                }
                service.process.fire();
            }
        }
        Stage::Route => service.route_events(now),
        Stage::Engine => {}
        Stage::Worker => {
            service.worker.iterate(now, wall);
            if service.worker.end().is_none()
                && let Some(problems) = service.worker.refused()
            {
                service.worker.set_end(End::Stopped(Stop::Start(problems.clone())));
                service.worker.abort();
            }
            if service.worker.is_empty() && service.worker.end().is_none() {
                service.worker.set_end(End::Drained);
            }
        }
        Stage::Out => {
            service.close_sources();
            service.route_requests();
            service.process.publish();
        }
    }
}
