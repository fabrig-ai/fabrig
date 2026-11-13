//! The workers' host and temper's typed root handoff (jig/docs/design/protocol/service.md, 11.3).
use crate::{
    assignment::Encoding,
    settings::{Files, Forge},
    values::Values,
};
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_protocol_config::{self as config, Problems, What};
use jig_protocol_engine_workers as kit;
use jig_protocol_worker_wire::Part;
use skein_io::{self as io};
use skein_lib::{List, Map, Queue, Reader, Time, Token};
use temper_engine_domain as root;
use temper_protocol_worker_wire::v1 as wire;
/// Temper's roots and bridge, with its spoken entry from the engine's table
/// (temper/docs/design/protocol/wires.md, 4 and 10).
pub fn application(
    limits: &wire::Limits,
) -> Result<jig_protocol_worker_wire::Application, skein_codec::bridge::BridgeError> {
    let mut application = temper_protocol_worker_wire::application(limits)?;
    let mut wires = List::with_capacity(1);
    for wire in crate::wires().into_boxed() {
        if wire.name == temper_protocol_worker_wire::wire::FAMILY {
            wires.push(wire).expect("one application wire");
        }
    }
    assert!(wires.len() == 1, "the table names temper's worker wire once");
    application.spoken = wires;
    Ok(application)
}

#[derive(Debug)]
pub struct Workers {
    pub(crate) host: kit::host::Workers,
    pub(crate) encoding: Encoding,
    pub(crate) forges: crate::forges::Forges,
    pub(crate) bad_parts: u64,
    pub(crate) notes: Queue<crate::Note>,
    opening_limits: kit::Limits,
    worker_names: List<Box<[u8]>>,
    opening_ends: Map<Option<Box<[u8]>>, kit::Loss>,
}
impl Workers {
    pub fn new(
        config: &config::Engine,
        files: &Files,
        encoding: Encoding,
        forges: Box<[Forge]>,
        forge_limit: u32,
        limits: &kit::Limits,
    ) -> Result<Self, Problems> {
        let tokens = kit::host::Tokens {
            listeners: crate::tokens::prefix(Token::new(crate::tokens::WORKER_LISTENERS)),
            connections: crate::tokens::prefix(Token::new(crate::tokens::WORKER_CONNECTIONS)),
        };
        let application = application(&encoding.limits).or(Err(kit::host::invalid(What::Bound)))?;
        let host = kit::host::Workers::new(config, files, tokens, application, limits)?;
        let mut table = crate::forges::Forges::with_capacity(forge_limit);
        for forge in forges {
            assert!(table.insert(forge), "unique fixture forges");
        }
        let mut worker_names = List::with_capacity(limits.names);
        for worker in &config.hosts.workers {
            worker_names.push(worker.name.clone()).expect("configured worker names fit the host");
        }
        let opening_names = worker_names.len().checked_add(1).ok_or(kit::host::invalid(What::Bound))?;
        Ok(Self {
            host,
            encoding,
            forges: table,
            bad_parts: 0,
            notes: Queue::uncapped(),
            opening_limits: *limits,
            worker_names,
            opening_ends: Map::with_capacity(opening_names),
        })
    }
    pub fn down(&mut self, now: Time, event: kit::Event, events: &mut Queue<root::Event>) {
        self.host.down(now, event);
        self.pull(events);
    }
    pub fn assignment(
        &mut self,
        now: Time,
        channel: Token,
        value: root::Assignment,
        values: &Values,
        events: &mut Queue<root::Event>,
    ) {
        let (task, attempt) = (value.task, value.attempt);
        match self.encoded_assignment(now, channel, value, values) {
            Some(event) => self.down(now, event, events),
            None => events.push(root::Event::Worker(root::WorkerInput::Link {
                channel,
                link: root::Link::Up {
                    up: host::LinkUp::Answer {
                        run: Token::new(task),
                        attempt: Token::new(attempt),
                        answer: host::RunAnswer {
                            read: None,
                            refused: Box::default(),
                            turns: 0,
                            spent: 0,
                            ending: host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Start {
                                cause: host::StartCause::Other,
                                why: b"assignment cannot encode".as_slice().into(),
                            })),
                        },
                    },
                    parts: root::Parts::none(),
                },
            })),
        }
    }
    #[must_use]
    pub fn encoded_assignment(
        &self,
        now: Time,
        channel: Token,
        value: root::Assignment,
        values: &Values,
    ) -> Option<kit::Event> {
        crate::assignment::down(value, &self.forges, &self.encoding, values, channel, now)
    }
    pub fn send(
        &mut self,
        now: Time,
        channel: Token,
        down: host::LinkDown,
        values: &Values,
        events: &mut Queue<root::Event>,
    ) {
        self.down(now, delivery(now, channel, down, values), events);
    }
    pub fn up(&mut self, now: Time, event: io::Event, events: &mut Queue<root::Event>) -> bool {
        let seen = self.host.up(now, event);
        self.pull(events);
        match seen {
            kit::host::Seen::Taken => true,
            kit::host::Seen::Unknown => false,
            kit::host::Seen::ListenerClosed { ending } => ending,
        }
    }
    pub fn iterate(&mut self, now: Time, events: &mut Queue<root::Event>) {
        self.host.iterate(now);
        self.pull(events);
    }
    pub fn abort(&mut self, now: Time, events: &mut Queue<root::Event>) {
        self.host.abort(now);
        self.pull(events);
    }
    pub fn close(&mut self, now: Time, events: &mut Queue<root::Event>) {
        self.host.close(now);
        self.pull(events);
    }
    pub(crate) fn judgement(&self) -> crate::upgrade::Workers {
        crate::upgrade::Workers {
            names: self.worker_names.clone().into_boxed(),
            room: self.opening_limits.room,
            schema: kit::limits::schema(&self.opening_limits).expect("validated worker limits"),
        }
    }
    fn pull(&mut self, events: &mut Queue<root::Event>) {
        for _ in 0..self.host.pending() {
            if let Some(output) = self.host.output() {
                self.route(output, events);
            }
        }
    }
    pub(crate) fn route(&mut self, output: kit::host::Output, events: &mut Queue<root::Event>) {
        match output {
            kit::host::Output::Opened { worker, wires, statement } => {
                let name = worker_name(&worker);
                self.notes.push(crate::Note::WorkerOpened { worker, wires });
                events.push(root::Event::Worker(root::WorkerInput::Opened {
                    worker: name,
                    statement: statement_to_root(statement),
                }));
            }
            kit::host::Output::Refused { worker, reason, text } => {
                self.notes.push(crate::Note::WorkerOpeningRefused { worker, reason, text });
            }
            kit::host::Output::Hello { channel, worker, hello } => {
                let name = self.worker_names.get(worker).expect("hello's configured worker position");
                let _last = self.opening_ends.remove(&Some(name.clone()));
                events.push(root::Event::Worker(root::WorkerInput::Link {
                    channel,
                    link: root::Link::Hello { worker, hello },
                }));
            }
            kit::host::Output::Up { channel, up, parts } => self.received(channel, up, &parts, events),
            kit::host::Output::Lost { channel } => {
                events.push(root::Event::Worker(root::WorkerInput::Link { channel, link: root::Link::Lost }));
            }
            kit::host::Output::Unsent { worker, channel, unsent } => {
                match unsent.bound {
                    kit::Bound::Room(_) | kit::Bound::Channel(_) => {}
                    kit::Bound::Translation { .. } => match unsent.kind {
                        jig_protocol_worker_wire::Kind::Assignment | jig_protocol_worker_wire::Kind::Job => {}
                        jig_protocol_worker_wire::Kind::Hello
                        | jig_protocol_worker_wire::Kind::Turn
                        | jig_protocol_worker_wire::Kind::Call
                        | jig_protocol_worker_wire::Kind::Answer
                        | jig_protocol_worker_wire::Kind::Fact
                        | jig_protocol_worker_wire::Kind::Unauthorized
                        | jig_protocol_worker_wire::Kind::Limited
                        | jig_protocol_worker_wire::Kind::AccountRefused
                        | jig_protocol_worker_wire::Kind::Message
                        | jig_protocol_worker_wire::Kind::Relayed
                        | jig_protocol_worker_wire::Kind::TurnKept
                        | jig_protocol_worker_wire::Kind::TurnBusy
                        | jig_protocol_worker_wire::Kind::Acknowledge
                        | jig_protocol_worker_wire::Kind::Cancel
                        | jig_protocol_worker_wire::Kind::Grant
                        | jig_protocol_worker_wire::Kind::Sources
                        | jig_protocol_worker_wire::Kind::Check
                        | jig_protocol_worker_wire::Kind::ReadList
                        | jig_protocol_worker_wire::Kind::ListRead
                        | jig_protocol_worker_wire::Kind::Draining
                        | jig_protocol_worker_wire::Kind::SignOut
                        | jig_protocol_worker_wire::Kind::Logins
                        | jig_protocol_worker_wire::Kind::SignedOut
                        | jig_protocol_worker_wire::Kind::JobAnswer
                        | jig_protocol_worker_wire::Kind::Ask
                        | jig_protocol_worker_wire::Kind::Asked
                        | jig_protocol_worker_wire::Kind::Wires => events
                            .push(root::Event::Worker(root::WorkerInput::Link { channel, link: root::Link::Mismatch })),
                    },
                }
                self.notes.push(crate::Note::WorkerUnsent { worker, channel, unsent });
            }
            kit::host::Output::Ended { worker, channel: Some(channel), why } => {
                self.notes.push(crate::Note::WorkerLost { worker, channel, why });
            }
            kit::host::Output::Ended { worker, channel: None, why } => {
                if self.opening_ends.get(&worker) != Some(&why) {
                    self.opening_ends
                        .insert(worker.clone(), why)
                        .expect("configured worker names and unnamed openings");
                    self.notes.push(crate::Note::WorkerOpeningEnded { worker, why });
                }
            }
        }
    }
}

#[expect(clippy::disallowed_methods, reason = "configuration validates worker names as UTF-8")]
fn worker_name(name: &[u8]) -> Box<str> {
    core::str::from_utf8(name).expect("configured worker name is UTF-8").into()
}

fn statement_to_root(statement: kit::Statement) -> root::WorkerStatement {
    let mut wires = List::with_capacity(statement.wires.len());
    for wire in statement.wires.into_boxed() {
        wires
            .push(root::OpenedWire { name: wire.name, lowest: wire.lowest, layouts: wire.layouts.into_boxed() })
            .expect("one root wire per stated wire");
    }
    let mut terms = List::with_capacity(statement.terms.len());
    for term in statement.terms.into_boxed() {
        terms.push(root::OpenedTerm { kind: term.kind, largest: term.largest }).expect("one root term per stated term");
    }
    root::WorkerStatement { wires: wires.into_boxed(), output: statement.output, terms: terms.into_boxed() }
}

fn delivery(now: Time, channel: Token, down: host::LinkDown, values: &Values) -> kit::Event {
    let values = values.grants(kit::host::grants(&down), now);
    kit::Event::Down { channel, down, parts: Box::default(), values }
}

impl Workers {
    /// Decode application records in this step, discarding and counting a bad set.
    pub fn received(&mut self, channel: Token, up: host::LinkUp, records: &[Part], events: &mut Queue<root::Event>) {
        let parts = match parts(&up, records, &self.encoding.limits) {
            Some(parts) => parts,
            None => {
                self.bad_parts = self.bad_parts.saturating_add(1);
                root::Parts::none()
            }
        };
        events.push(root::Event::Worker(root::WorkerInput::Link { channel, link: root::Link::Up { up, parts } }));
    }
}
/// Decode the answer's named parts; any missing or bad part discards all parts.
#[must_use]
pub fn parts(up: &host::LinkUp, parts: &[Part], limits: &wire::Limits) -> Option<root::Parts> {
    let (work, preparation) = match up {
        host::LinkUp::JobAnswer { answer, .. } => match answer {
            host::JobEnding::Finished { work, .. } => (Some(work), None),
            host::JobEnding::Failed { work, failure, .. } => (
                Some(work),
                match failure {
                    host::JobFailure::Unprepared(host::Preparation::Permanent { resource }) => *resource,
                    host::JobFailure::Unprepared(host::Preparation::Transient)
                    | host::JobFailure::Failed
                    | host::JobFailure::Deadline
                    | host::JobFailure::Result
                    | host::JobFailure::Unstarted
                    | host::JobFailure::Undelivered(_)
                    | host::JobFailure::Cancelled(_) => None,
                },
            ),
            host::JobEnding::Refused(_) => (None, None),
        },
        host::LinkUp::Answer { answer, .. } => match &answer.ending {
            host::Ending::Ended { work, .. } | host::Ending::Parked { work } => (Some(work), None),
            host::Ending::Failed { work, failure, .. } => (
                Some(work),
                match failure {
                    host::Failure::Unprepared(host::Preparation::Permanent { resource }) => *resource,
                    host::Failure::Unprepared(host::Preparation::Transient)
                    | host::Failure::Run(_)
                    | host::Failure::Agent(_)
                    | host::Failure::Cancelled(_) => None,
                },
            ),
            host::Ending::Refused(_) => (None, None),
        },
        host::LinkUp::ListRead { .. }
        | host::LinkUp::Draining
        | host::LinkUp::Logins { .. }
        | host::LinkUp::SignedOut { .. }
        | host::LinkUp::Piece { .. }
        | host::LinkUp::Activity { .. }
        | host::LinkUp::Unauthorized { .. }
        | host::LinkUp::Limited { .. }
        | host::LinkUp::AccountRefused { .. }
        | host::LinkUp::Turn { .. }
        | host::LinkUp::Ask { .. }
        | host::LinkUp::Call { .. } => (None, None),
    };
    let left = match work {
        Some(work) => left(work.left, parts, limits).ok()?,
        None => None,
    };
    let saved = match work {
        Some(work) => saved(work.saved, parts, limits).ok()?,
        None => None,
    };
    let preparation = preparation_part(preparation, parts, limits).ok()?;
    Some(crate::translate::parts(left, saved, preparation))
}
fn bytes(token: Token, parts: &[Part]) -> Option<&[u8]> {
    for part in parts {
        if part.token == token {
            return Some(&part.bytes);
        }
    }
    None
}
fn left(token: Option<Token>, parts: &[Part], limits: &wire::Limits) -> Result<Option<wire::Left>, ()> {
    let Some(token) = token else {
        return Ok(None);
    };
    let mut reader = Reader::new(bytes(token, parts).ok_or(())?);
    let value = wire::Left::decode(limits, &mut reader).or(Err(()))?;
    if !reader.is_empty() {
        return Err(());
    }
    Ok(Some(value))
}
fn saved(token: Option<Token>, parts: &[Part], limits: &wire::Limits) -> Result<Option<wire::Saved>, ()> {
    let Some(token) = token else {
        return Ok(None);
    };
    let mut reader = Reader::new(bytes(token, parts).ok_or(())?);
    let value = wire::Saved::decode(limits, &mut reader).or(Err(()))?;
    if !reader.is_empty() {
        return Err(());
    }
    Ok(Some(value))
}
fn preparation_part(
    token: Option<Token>,
    parts: &[Part],
    limits: &wire::Limits,
) -> Result<Option<wire::Preparation>, ()> {
    let Some(token) = token else {
        return Ok(None);
    };
    let mut reader = Reader::new(bytes(token, parts).ok_or(())?);
    let value = wire::Preparation::decode(limits, &mut reader).or(Err(()))?;
    if !reader.is_empty() {
        return Err(());
    }
    Ok(Some(value))
}
/// Channel bounds, with application parts measured from temper's codecs.
#[must_use]
pub fn limits(settings: &crate::settings::Engine) -> Option<kit::Limits> {
    let jig = &settings.jig.limits;
    let wire = crate::translate::limits(&settings.limits)?;
    let core = jig.core;
    let hub = jig.hub;
    let link = jig.link;
    let ceilings = jig_protocol_worker_wire::CEILINGS;
    let sizes = jig_protocol_worker_wire::sizes::Sizes {
        application_ask: hub.application_ask_bytes,
        application_answer: hub.application_answer_bytes,
        program_sources: core.fleet.program_sources,
        piece: hub.piece_bytes,
        charter: u32::try_from(hub.charter_bytes).ok()?,
        transcript: u32::try_from(hub.transcript_bytes).ok()?,
        turn: u32::try_from(hub.turn_bytes).ok()?,
        message: core.tasks.message_bytes,
        sender: ceilings.message_sender,
        call_name: u32::try_from(core.fleet.call_name_bytes).ok()?,
        tool_name: ceilings.call_tool,
        call_input: core.call_answer_bytes,
        answer: core.call_answer_bytes,
        result: u32::try_from(hub.outcome_bytes).ok()?,
        detail: hub.detail_bytes,
        grant: crate::values::grant_bytes(
            crate::values::credential_bytes(&settings.jig),
            u32::try_from(wire::Identity::encoded_bound(&wire)?).ok()?,
        )?,
        name: link.name,
        secret: link.secret,
        items: u32::try_from(wire::Items::encoded_bound(&wire)?).ok()?,
        left: u32::try_from(wire::Left::encoded_bound(&wire)?).ok()?,
        saved: u32::try_from(wire::Saved::encoded_bound(&wire)?).ok()?,
        evidence: u32::try_from(hub.delivery_evidence_bytes).ok()?,
        resource: u32::try_from(wire::Preparation::encoded_bound(&wire)?).ok()?,
        slots: core.fleet.slots,
        calls: hub.run_calls,
        turns: hub.turns,
        inbox: core.tasks.run_inbox,
        answered: core.call_records,
        models: ceilings.bounds_endpoints,
        workstreams: core.fleet.workstreams,
        accounts: core.accounts.accounts,
        credentials: settings.limits.forge.forges,
        facts: link.facts,
    };
    let limits = kit::Limits {
        room: jig.io.largest_room(),
        sizes,
        names: link.names,
        workers: core.fleet.workers,
        opening: link.opening,
        chunk: link.chunk,
        handshake: link.handshake,
        hello: link.hello,
        ping: link.ping,
        silence: link.silence,
        stall: link.stall,
    };
    if !valid(&limits) {
        return None;
    }
    Some(limits)
}

/// Check the channel's limits independently of the service's containers.
#[must_use]
pub fn valid(limits: &kit::Limits) -> bool {
    // jig's valid() replaces this at the set's repin
    kit::valid(limits)
}

#[cfg(test)]
#[test]
fn channel_limits_accept_stock_and_refuse_zero_handshake() {
    let mut limits = limits(&crate::tests::fixtures::engine()).unwrap();
    assert!(valid(&limits));
    limits.handshake = skein_lib::Duration::ZERO;
    assert!(!valid(&limits));
}

impl Workers {
    /// Construct checked channels from the service's typed settings.
    pub fn configured(settings: &mut crate::settings::Engine, deployment: [u8; 16]) -> Result<Self, Problems> {
        let worker_limits = limits(settings).ok_or(kit::host::invalid(What::Bound))?;
        let mut endpoints =
            List::with_capacity(u32::try_from(settings.jig.llm.endpoints.len()).expect("endpoint count"));
        for endpoint in &settings.jig.llm.endpoints {
            endpoints
                .push(smith_protocol_translate::Endpoint {
                    name: endpoint.name.clone(),
                    number: endpoint.number,
                    dialect: endpoint.dialect,
                })
                .expect("endpoint count");
        }
        let workers = Self::new(
            &settings.jig,
            &settings.files,
            Encoding {
                limits: crate::translate::limits(&settings.limits).ok_or(kit::host::invalid(What::Bound))?,
                endpoints: smith_protocol_translate::Endpoints::new(endpoints),
                deployment,
                charter_bytes: crate::assignment::charter_bound(
                    settings.jig.limits.hub.charter_bytes,
                    settings.agent_charter_bytes,
                ),
            },
            Box::new([]),
            settings.limits.forge.forges,
            &worker_limits,
        )?;
        Ok(workers)
    }
}

impl Workers {
    #[must_use]
    pub fn encoding(&self) -> &Encoding {
        &self.encoding
    }
    pub fn configured_forges(&mut self, forges: crate::forges::Forges) {
        self.forges = forges;
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hello_forwards_a_nonzero_configured_worker_position() {
        let mut service = crate::tests::fixtures::started();
        let channel = Token::new(7);
        let hello = jig_domain_link::Hello {
            jobs: Box::default(),
            slots: 1,
            stop_bound: skein_lib::Duration::from_millis(100),
            workstreams: Box::new([]),
            hosting: Box::new([]),
            bounds: crate::tests::fixtures::worker_agent().bounds,
        };
        let mut events = Queue::uncapped();
        service
            .engine
            .as_mut()
            .expect("engine")
            .workers
            .route(kit::host::Output::Hello { channel, worker: 1, hello }, &mut events);
        let Some(root::Event::Worker(root::WorkerInput::Link {
            channel: forwarded_channel,
            link: root::Link::Hello { worker, .. },
        })) = events.pop()
        else {
            panic!("hello reaches the root");
        };
        assert_eq!((forwarded_channel, worker), (channel, 1));
    }
}
