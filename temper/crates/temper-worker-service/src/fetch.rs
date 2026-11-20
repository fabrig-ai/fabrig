//! A figures-only opening before the worker is built (protocol/configuration/figures.md, 3).
use crate::{settings, worker_alone::Note, worker_settings};
use jig_protocol_config::{self as config, Problems, What};
use jig_protocol_worker_link as link;
use jig_protocol_worker_wire::Figures;
use skein_channel::{Lower, LowerEvent};
use skein_io::{
    self as io,
    kernel::{Complete, Fd, Submit},
};
use skein_lib::{Queue, Time, Token, Wall, stream};
use temper_service_core::{Stage, process::Process};

const OWNER: Token = Token::new(2);
/// Owns only the stock link and its process, not a worker kernel or workspace.
#[expect(missing_debug_implementations, reason = "the process owns kernel buffers")]
pub struct Fetch {
    process: Process,
    link: link::Component,
    out: Queue<link::Request>,
    io: Queue<io::Request>,
    notes: Queue<Note>,
    address: io::Addr,
    socket: Option<Token>,
    figures: Option<Figures>,
    closed: bool,
    stopping: bool,
}
impl Fetch {
    pub fn new(worker: &settings::Worker, files: &settings::Files, seed: u64) -> Result<Self, Problems> {
        let stock = config::stock();
        let own = worker_settings::Limits::shared(temper_service_core::limits::stock());
        let payload = worker_settings::payload_limits(&own).ok_or(worker_settings::problem(b"limits", What::Bound))?;
        let limits = worker_settings::link_limits_from(&worker.jig, &stock, &payload, own.peer.forges)
            .ok_or(worker_settings::problem(b"limits", What::Bound))?;
        let secret = files.contents(&worker.jig.secret).ok_or(worker_settings::problem(b"secret", What::Unreadable))?;
        let secret = secret.strip_suffix(b"\n").unwrap_or(secret);
        let mut credential = worker_settings::credential(worker, secret, &payload)
            .or(Err(worker_settings::problem(b"secret", What::Bound)))?;
        credential.opening = jig_protocol_worker_wire::Opening::Figures;
        let link = link::Component::new(&limits, jig_protocol_worker_wire::spoken(), credential, seed)
            .or(Err(worker_settings::problem(b"limits", What::Bound)))?;
        let address = match worker.jig.engine {
            config::Address::Addr(address) => address,
            config::Address::Host { .. } => return Err(worker_settings::problem(b"engine", What::Form)),
        };
        let operations =
            Process::operations_for(&stock.io, 0).ok_or(worker_settings::problem(b"limits", What::Bound))?;
        Ok(Self {
            process: Process::new(stock.io, operations, operations),
            link,
            address,
            out: Queue::uncapped(),
            io: Queue::uncapped(),
            notes: Queue::uncapped(),
            socket: None,
            figures: None,
            closed: false,
            stopping: false,
        })
    }
    pub fn completions(&mut self) -> &mut Queue<Complete> {
        self.process.completions()
    }
    pub fn submissions(&mut self) -> &mut Queue<Submit> {
        self.process.submissions()
    }
    pub fn notes(&mut self) -> &mut Queue<Note> {
        &mut self.notes
    }
    pub fn take(&mut self) -> Option<Figures> {
        self.figures.take()
    }
    #[must_use]
    pub const fn figures(&self) -> Option<&Figures> {
        self.figures.as_ref()
    }
    #[must_use]
    pub fn operations(&self) -> u32 {
        self.process.operations()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.closed && self.io.is_empty() && self.process.is_empty()
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        skein_lib::earliest(self.link.next_deadline(), self.process.io().next_deadline())
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        self.process.work_pending(now)
            || !self.io.is_empty()
            || !self.out.is_empty()
            || match self.next_deadline() {
                Some(at) => at <= now,
                None => false,
            }
            || self.closed && self.process.signals_pending()
    }
    pub fn adopt_signals(&mut self, fd: Fd) -> Result<(), Fd> {
        self.process.adopt_signals(fd)
    }
    pub fn shutdown(&mut self) {
        self.stopping = true;
    }
    pub fn check_wait_boundary(&self, now: Time) {
        assert!(
            self.is_empty()
                || self.work_pending(now)
                || self.next_deadline().is_some()
                || self.process.has_operations()
                || self.process.submission_count() != 0,
            "figures fetch would wait with nothing in flight"
        );
    }
    fn event(&mut self, now: Time, event: io::Event) {
        if let io::Event::Shutdown { .. } = event {
            self.shutdown();
            return;
        }
        if event.owner() == Some(Token::new(crate::tokens::SIGNALS)) {
            if let io::Event::Closed { .. } = event {
                self.process.signals_closed();
            }
            return;
        }
        let event = match event {
            io::Event::Connecting { socket, .. } => {
                self.socket = Some(socket);
                return;
            }
            io::Event::Connected { .. } => link::Event::Dialled { connection: OWNER },
            io::Event::Failed { .. } => link::Event::Unreachable,
            io::Event::Stream { up, .. } => link::Event::Lower { connection: OWNER, event: LowerEvent::Read(up) },
            io::Event::Output { up, .. } => link::Event::Lower { connection: OWNER, event: LowerEvent::Write(up) },
            io::Event::Closed { .. } => {
                self.socket = None;
                link::Event::Gone { connection: OWNER }
            }
            io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Usage { .. }
            | io::Event::Window { .. }
            | io::Event::Resized
            | io::Event::Shutdown { .. } => return,
        };
        self.link.down(now, event, &mut self.out);
    }
    fn requests(&mut self) {
        for _ in 0..self.out.len() {
            match self.out.pop().expect("link requests") {
                link::Request::Figures { figures } => self.figures = Some(figures),
                link::Request::Refused { reason, text } => self.notes.push(Note::OpeningRefused { reason, text }),
                link::Request::Dial => self.io.push(io::Request::Connect { owner: OWNER, addr: self.address }),
                link::Request::Lower { lower, .. } => {
                    if let Some(socket) = self.socket {
                        self.io.push(match lower {
                            Lower::Read(down) => io::Request::Stream { stream: socket, down },
                            Lower::Write(down) => io::Request::Output { stream: socket, down },
                            Lower::FinishWrite => io::Request::Stream { stream: socket, down: stream::Down::Finish },
                        });
                    }
                }
                link::Request::Close { .. } => {
                    if let Some(entity) = self.socket {
                        self.io.push(io::Request::Close { entity });
                    }
                }
                link::Request::Abort { .. } => {
                    if let Some(entity) = self.socket {
                        self.io.push(io::Request::Abort { entity });
                    }
                }
                link::Request::Closed => self.closed = true,
                link::Request::CannotSendWhole { wire, version } => {
                    self.notes.push(Note::CannotSendWhole { wire, version });
                }
                link::Request::Untranslated { kind, halt } => self.notes.push(Note::Untranslated { kind, halt }),
                link::Request::Ready
                | link::Request::Restarting { .. }
                | link::Request::RelayCancelled { .. }
                | link::Request::Resend { .. }
                | link::Request::Lost
                | link::Request::GracePassed
                | link::Request::Down { .. }
                | link::Request::Value { .. }
                | link::Request::Fact { .. } => {}
            }
        }
    }
}
pub fn iterate_stage(service: &mut Fetch, stage: Stage, now: Time, wall: Wall) {
    match stage {
        Stage::Io => {
            service.process.clock(now, wall);
            service.process.resume();
            if !service.process.io().is_ready() {
                for _ in 0..service.process.completion_count() {
                    let Some(complete) = service.process.take_completion() else { break };
                    service.process.up(complete);
                }
                service.process.fire();
            }
        }
        Stage::Route => {
            for _ in 0..service.process.told_count() {
                let event = service.process.told().pop().expect("process event count");
                service.event(now, event);
            }
        }
        Stage::Engine => {}
        Stage::Worker => {
            if service.stopping {
                service.link.down(now, link::Event::Abort, &mut service.out);
                service.stopping = false;
            }
            service.link.fire(now, &mut service.out);
            service.requests();
        }
        Stage::Out => {
            if service.closed {
                service.process.close_signals();
            }
            for _ in 0..service.io.len() {
                if service.process.room() < io::MAX_OUT_DOWN.submissions || !service.process.io().takes() {
                    break;
                }
                let request = service.io.pop().expect("fetch io requests");
                service.process.down(request);
            }
            service.process.publish();
        }
    }
}
pub fn iterate(service: &mut Fetch, now: Time, wall: Wall) {
    for stage in crate::STAGES {
        iterate_stage(service, stage, now, wall);
    }
}
