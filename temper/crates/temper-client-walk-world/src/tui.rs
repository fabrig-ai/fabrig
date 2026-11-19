//! The shipped terminal on the walk's world (temper/docs/design/client/journey-walks.md, 3.3).
use crate::{facts, web::Failed};
use jig_client::{Event, Record, Request};
use jig_client_shell_native::{Engine, Transport};
use skein_io::{self as io, kernel};
use skein_lib::{Env, Queue, Secret, Time, Wall};
use skein_ui_screen::Frame;
use skein_ui_screen_terminal::Key;
use std::collections::BTreeMap;
use temper_client_shell_tui::{
    host::{Host, Report, Settings, Start},
    limits,
};
use temper_client_tui::{Note, Origin};

/// The native transport, owned and scheduled as a process of the world.
pub struct Connection {
    transport: Transport,
    io: io::Io,
    limits: io::Limits,
    inputs: Queue<Request>,
    heard: Queue<Event>,
    events: Queue<io::Event>,
    requests: Queue<io::Request>,
    submissions: Queue<kernel::Submit>,
    completions: Queue<kernel::Complete>,
    closing: bool,
    opened_streams: u64,
}
impl Connection {
    pub(crate) fn new() -> Self {
        let native = limits::tui().native;
        let operations = io::operations(&native.io).expect("native io operations");
        Self {
            transport: Transport::new(
                Engine::Plain(io::Addr::Ip(facts::SERVICE.parse().expect("fixed service address"))),
                &native.connections,
                native.io.sockets,
                0,
                jig_client_protocol_http::Mapping::with_page(b"/jig", temper_client_domain::sign_in::LINK_PAGE)
                    .expect("temper mapping"),
            ),
            io: io::Io::new(&native.io),
            limits: native.io,
            inputs: Queue::uncapped(),
            heard: Queue::uncapped(),
            events: Queue::uncapped(),
            requests: Queue::uncapped(),
            submissions: Queue::with_capacity(operations),
            completions: Queue::with_capacity(operations),
            closing: false,
            opened_streams: 0,
        }
    }
    /// The number of `OpenStream` requests carried by this connection.
    #[must_use]
    pub fn opened_streams(&self) -> u64 {
        self.opened_streams
    }
    pub(crate) fn take(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Some(event) = self.heard.pop() {
            events.push(event);
        }
        events
    }
    pub(crate) fn close(&mut self) {
        self.closing = true;
    }
}
impl skein_shell::Host for Connection {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if self.closing {
            self.transport.close(now);
        }
        while let Some(request) = self.inputs.pop() {
            if matches!(request, Request::OpenStream { .. }) {
                self.opened_streams = self.opened_streams.checked_add(1).expect("bounded stream opens");
            }
            self.transport.down(now, request);
        }
        let env = Env { now, wall, limits: self.limits };
        io::stage::up(&mut self.io, &env, &mut self.completions, &mut self.events, &mut self.submissions, now);
        while let Some(event) = self.events.pop() {
            self.transport.event(now, event);
        }
        self.transport.iterate(now, wall, &mut self.heard, &mut self.requests);
        io::stage::down(&mut self.io, &env, &mut self.requests, &mut self.submissions);
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        self.io.is_ready()
            || !self.inputs.is_empty()
            || !self.completions.is_empty()
            || !self.requests.is_empty()
            || self.transport.work_pending(now)
            || self.closing && !self.transport.is_empty()
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        skein_lib::earliest(self.io.next_deadline(), self.transport.next_deadline())
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.io.is_empty()
            && self.transport.is_empty()
            && self.inputs.is_empty()
            && self.events.is_empty()
            && self.requests.is_empty()
            && self.submissions.is_empty()
            && self.completions.is_empty()
    }
    fn operations(&self) -> u32 {
        io::operations(&self.limits).expect("native io operations")
    }
}

/// Volatile equivalents of the native shell's private files, lasting for the walk.
#[derive(Default)]
pub(crate) struct Held {
    records: BTreeMap<Box<[u8]>, Box<[u8]>>,
    credential: Option<Secret>,
    note: Option<Note>,
    address: Option<Origin>,
}

/// A terminal host and the state its run carries, with no terminal or opener.
pub struct Terminal {
    pub(crate) host: Host,
    pub(crate) at: usize,
    held: Held,
    record_bytes: usize,
    browsed: Option<Box<[u8]>>,
    pub(crate) closed: bool,
}
impl Terminal {
    pub(crate) fn new(at: usize, held: Held, seed: u64, now: Time, wall: Wall) -> Self {
        let limits = limits::tui();
        let record_bytes = skein_lib::usize_of(limits.native.connections.body_bytes);
        Self {
            host: Host::new(&Settings { limits }, seed, now, wall),
            at,
            held,
            record_bytes,
            browsed: None,
            closed: false,
        }
    }
    pub(crate) fn start(&mut self, connection: &mut Connection) -> Result<bool, Failed> {
        self.report(
            Report::Start(Start {
                size: facts::SIZE,
                scheme: None,
                background: None,
                plain: false,
                version: facts::VERSION.into(),
                address: Origin::Http(facts::SERVICE.parse().expect("fixed service address")),
                local: false,
                boxed: false,
                kept: self
                    .held
                    .records
                    .iter()
                    .map(|(key, bytes)| Record { key: key.clone(), bytes: bytes.clone() })
                    .collect(),
                credential: self.held.credential.clone(),
                note: self.held.note,
                front: false,
                set_up: false,
                box_log: Box::default(),
            }),
            connection,
        )
    }
    pub fn frame(&mut self) -> &Frame {
        self.host.frame()
    }
    #[must_use]
    pub fn browsed(&self) -> Option<&[u8]> {
        self.browsed.as_deref()
    }
    #[must_use]
    pub fn event_of(&self, key: &Key) -> Option<temper_client_tui::Event> {
        self.host.event_of(key)
    }
    pub(crate) fn held(self) -> Held {
        self.held
    }
    pub(crate) fn check(&self) -> Result<(), Failed> {
        if self.closed { Err(Failed::Closed) } else { Ok(()) }
    }
    pub(crate) fn report(&mut self, report: Report, connection: &mut Connection) -> Result<bool, Failed> {
        let mut reports = Queue::uncapped();
        reports.push(report);
        let mut pending = false;
        while let Some(report) = reports.pop() {
            let duties = self.host.report(report);
            for request in duties.asked {
                match request {
                    Request::Save { records } => {
                        let mut written = true;
                        for record in records.into_boxed() {
                            let bytes: usize = self
                                .held
                                .records
                                .iter()
                                .filter(|(key, _)| **key != record.key)
                                .map(|(key, value)| key.len() + value.len() + 8)
                                .sum();
                            if bytes + record.key.len() + record.bytes.len() + 12 > self.record_bytes {
                                written = false;
                            } else {
                                self.held.records.insert(record.key, record.bytes);
                            }
                        }
                        reports.push(Report::Heard(if written { Event::Stored } else { Event::NotStored }));
                    }
                    Request::Erase { records } => {
                        for key in records.into_boxed() {
                            self.held.records.remove(&key);
                        }
                        reports.push(Report::Heard(Event::Stored));
                    }
                    Request::Told { .. }
                    | Request::AskedLink(_)
                    | Request::Approval(_)
                    | Request::Authorized(_)
                    | Request::ServiceCredential { .. } => {}
                    Request::Exchange { .. } | Request::OpenStream { .. } | Request::CloseStream => {
                        connection.inputs.push(request);
                        pending = true;
                    }
                }
            }
            if let Some(credential) = duties.keep {
                self.held.credential = credential;
                reports.push(Report::Kept { written: true });
            }
            if let Some(note) = duties.note {
                self.held.note = Some(note);
            }
            if let Some(address) = duties.address {
                self.held.address = Some(address);
            }
            if let Some(address) = duties.open {
                self.browsed = Some(address);
                reports.push(Report::Opened { opened: true });
            }
            if duties.local {
                reports.push(Report::Local { link: None });
            }
            if duties.close {
                self.closed = true;
                connection.close();
            }
            if duties.stop.is_some() {
                return Err(Failed::Asked("stop"));
            }
            if duties.lend.is_some() {
                return Err(Failed::Asked("lend"));
            }
            if duties.unlend {
                return Err(Failed::Asked("unlend"));
            }
        }
        Ok(pending)
    }
}
