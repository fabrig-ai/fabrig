//! Requests keep their own attempts; reads and watches can share an exchange (protocol/client/wire.md, 2.3).
//! A family's admission refusal uses jig's wire (client/domain/contract/requests.md, 3).
//! Transport admission counts unfinished exchanges (jig/docs/design/protocol/client/README.md, 10).
//! The link gathers transport work until settling (protocol/client/wire.md, 2.3).
//! Stream bounds and order refusals differ from unreadable records (protocol/client/README.md, 6).
use crate::{Event, Fact, Limits, Message, Request, Streamed, Unusable, translate};
use alloc::boxed::Box;
use jig_client_domain::{Answered, Ask, Family, Readable, Readout, Retry, Watchable};
use jig_client_protocol_wire as w;
use jig_client_protocol_wire as wire;
use jig_client_protocol_wire::Record as _;
use skein_lib::{Map, Queue, Reader, RequestEnvelope, RequestKey, Secret, Time, Token, Wall, Writer};
#[derive(Debug)]
pub(crate) struct Attempt {
    pub(super) token: Token,
    pub(super) owner: w::Owner,
    pub(super) version: u32,
    pub(super) kind: ExchangeKind,
    pub(super) held: Option<Secret>,
    pub(super) parts: skein_lib::List<jig_client_domain::contract::PartAsk>,
}
impl Attempt {
    pub(super) fn carrying(&self) -> bool {
        match self.kind {
            ExchangeKind::Attempt { carrying, .. } => carrying,
            ExchangeKind::LinkAsk
            | ExchangeKind::Approval(_)
            | ExchangeKind::SignInApproval
            | ExchangeKind::Authorize
            | ExchangeKind::SignIn
            | ExchangeKind::SignOut { .. }
            | ExchangeKind::Read { .. }
            | ExchangeKind::WatchOpen { .. }
            | ExchangeKind::WatchClose { .. } => false,
        }
    }
}
#[derive(Debug)]
pub(super) enum ExchangeKind {
    LinkAsk,
    Authorize,
    Approval(crate::approval::ApprovalKind),
    SignInApproval,
    SignIn,
    SignOut { generation: u64 },
    // `carrying`: an exchange carrying a value either way, sent once and never held
    // behind the store (protocol/client/README.md, 3 and 7).
    Attempt { carrying: bool, key: RequestKey },
    Read { turn: Option<(u64, u64, u32)>, retry: Option<Readable> },
    WatchOpen { name: u64 },
    WatchClose { name: u64 },
}
#[derive(Debug)]
enum StreamRefusal {
    Record(crate::Undecodable),
    Order,
}
#[derive(Debug)]
pub(super) enum Gathered {
    Exchange { exchange: Token, credential: Option<Secret>, message: Message },
    OpenStream { credential: Option<Secret> },
    CloseStream,
}
#[derive(Debug)]
pub struct Link {
    pub(super) limits: Limits,
    pub(super) attempts: Map<Token, Attempt>,
    // Cancellation ends the logical attempt; transport credit ends with its HTTP terminal.
    pub(super) transport: Map<Token, ()>,
    pub(super) gathered: Queue<Gathered>,
    pub(super) batches: Map<Token, Map<u16, Token>>,
    pub(super) next: u64,
    pub(super) credential: Option<Secret>,
    pub(super) expires: Option<Wall>,
    pub(super) generation: u64,
    pub(super) stream: Option<crate::silence::Silence>,
    pub(super) retired: Map<u64, Token>,
    pub(super) predecessor: Option<[u8; 16]>,
    watches: crate::watches::Watches,
    pub(super) store: crate::store::Store,
}
impl Link {
    pub fn new(limits: &Limits) -> Result<Self, Unusable> {
        Ok(Self {
            limits: *limits,
            store: crate::store::Store::new(),
            attempts: Map::uncapped(),
            transport: Map::uncapped(),
            gathered: Queue::uncapped(),
            next: 1,
            credential: None,
            expires: None,
            generation: 0,
            stream: None,
            retired: Map::uncapped(),
            predecessor: None,
            batches: Map::uncapped(),
            watches: crate::watches::Watches::new(limits),
        })
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        let Some(stream) = &self.stream else { return None };
        Some(stream.silence())
    }
    /// Whether a read can own a logical attempt, independently of transport credit.
    #[must_use]
    pub fn has_exchange_room(&self) -> bool {
        let mut reads = 0_u32;
        for (_, attempt) in &self.attempts {
            match attempt.kind {
                ExchangeKind::Read { .. } => reads = reads.saturating_add(1),
                ExchangeKind::LinkAsk
                | ExchangeKind::Authorize
                | ExchangeKind::Approval(_)
                | ExchangeKind::SignInApproval
                | ExchangeKind::SignIn
                | ExchangeKind::SignOut { .. }
                | ExchangeKind::Attempt { .. }
                | ExchangeKind::WatchOpen { .. }
                | ExchangeKind::WatchClose { .. } => {}
            }
        }
        reads < self.limits.client().reads
    }
    /// Unfinished transport exchanges; a batch counts once until its terminal.
    #[must_use]
    pub fn in_flight(&self) -> u32 {
        self.transport.len()
    }
    pub(super) fn emit_exchange(&mut self, exchange: Token, credential: Option<Secret>, message: Message) {
        self.gathered.push(Gathered::Exchange { exchange, credential, message });
    }
    pub fn step(&mut self, now: Time, event: Event, out: &mut Queue<Request>) {
        match event {
            Event::SignIn { with } => self.sign_in(with, out),
            Event::SignOut => self.sign_out(out),
            Event::OpenStream => self.open_stream(now, out),
            Event::Authorize { query } => self.authorize(query, out),
            Event::AskLink => self.ask_link(out),
            Event::AskApproval { port } => self.ask_approval(port, out),
            Event::ReadApproval { code } => self.read_approval(code, out),
            Event::DecideApproval { code, approve } => self.decide_approval(code, approve, out),
            Event::Start { kept } => self.start_store(kept, out),
            Event::Save { record } => self.save(record, out),
            Event::Erase { key } => self.erase(key, out),
            Event::Stored => self.written(),
            Event::NotStored => self.write_failed(out),
            Event::Open { watch, subject } => self.open(watch, subject, out),
            Event::Continue { watch, subject } => {
                self.forget(watch, out);
                self.open(watch, Watchable::Jig(subject), out);
            }
            Event::Close { watch } => self.close(watch, out),
            Event::Streamed { message } => self.streamed(now, message, out),
            Event::StreamGone => self.drop_stream(out),
            Event::Send { attempt, key, ask, value } => self.send(attempt, key, ask, value, out),
            Event::Read { read, what } => self.read(read, what, false, out),
            Event::ReadAgain { read, what } => self.read(read, what, true, out),
            Event::Reply { exchange, message } => {
                self.transport.remove(&exchange);
                if self.batches.contains_key(&exchange) {
                    self.batch_lost(exchange, out);
                    self.batches.remove(&exchange);
                } else {
                    self.answered(exchange, &message, out);
                }
            }
            Event::Part { exchange, message } => self.part(exchange, &message, out),
            Event::Done { exchange } => {
                if self.batches.contains_key(&exchange) {
                    self.transport.remove(&exchange);
                    self.batch_lost(exchange, out);
                    self.batches.remove(&exchange);
                }
            }
            Event::Lost { exchange } => {
                self.transport.remove(&exchange);
                if self.batches.contains_key(&exchange) {
                    self.batch_lost(exchange, out);
                    self.batches.remove(&exchange);
                } else if let Some(attempt) = self.attempts.remove(&exchange) {
                    self.failure(attempt, out);
                }
            }
        }
        // Settle after the whole event: sign-out and stream loss discard held work first.
        // This also covers cancellation and store failure releasing logical attempts.
        self.resume_watches(out);
    }
    pub fn fire(&mut self, now: Time, out: &mut Queue<Request>) {
        if let Some(stream) = &self.stream
            && now >= stream.silence()
        {
            self.drop_stream(out);
        }
    }
    fn send(&mut self, attempt: Token, key: RequestKey, ask: Ask, value: Option<Secret>, out: &mut Queue<Request>) {
        let mut carrying = value.is_some();
        if let Ask::Jig(ask) = &ask
            && let jig_client_domain::contract::Ask::AccountChange {
                change: jig_client_domain::contract::AccountChange::MoveOut { .. },
                ..
            } = ask.as_ref()
        {
            carrying = true;
        }
        // An exchange carrying a value either way never left if it cannot be sent.
        let refuse: fn(Token, &mut Queue<Request>) = if carrying { unsent } else { lost };
        let mut keyed_count = 0_u32;
        for (_, attempt) in &self.attempts {
            match attempt.kind {
                ExchangeKind::Attempt { .. } => keyed_count = keyed_count.saturating_add(1),
                ExchangeKind::LinkAsk
                | ExchangeKind::Authorize
                | ExchangeKind::Approval(_)
                | ExchangeKind::SignInApproval
                | ExchangeKind::SignIn
                | ExchangeKind::SignOut { .. }
                | ExchangeKind::Read { .. }
                | ExchangeKind::WatchOpen { .. }
                | ExchangeKind::WatchClose { .. } => {}
            }
        }
        if keyed_count >= self.limits.client().requests.requests {
            refuse(attempt, out);
            return;
        }
        let Some(next) = self.next.checked_add(1) else {
            refuse(attempt, out);
            return;
        };
        let Ok(record) = keyed(key, ask, value, self.limits.wire()) else {
            refuse(attempt, out);
            return;
        };
        let Ok(bytes) = record.encode(self.limits.wire()) else {
            refuse(attempt, out);
            return;
        };
        let exchange = Token::new(self.next);
        self.next = next;
        let state = Attempt {
            parts: skein_lib::List::with_capacity(0),
            held: None,
            token: attempt,
            owner: record.owner.clone(),
            version: record.version,
            kind: ExchangeKind::Attempt { carrying, key },
        };
        self.attempts.insert(exchange, state).expect("checked exchange capacity");
        self.exchange(exchange, Message::Request(bytes), out);
    }
    fn read(&mut self, read: Token, what: Readable, again: bool, out: &mut Queue<Request>) {
        let turn = if let Readable::Jig(jig_client_domain::contract::Read::Turn { task, attempt, turn }) = &what {
            Some((*task, *attempt, *turn))
        } else {
            None
        };
        let owner = match &what {
            Readable::Jig(_) => w::Owner::Jig,
            Readable::Family { record: value, .. } => w::Owner::Family(w::Family { number: value.family }),
        };
        if !self.has_exchange_room() {
            read_none(read, &owner, jig_client_domain::contract::ReadNone::Unreachable, out);
            return;
        }
        let Some(next) = self.next.checked_add(1) else {
            read_none(read, &owner, jig_client_domain::contract::ReadNone::Unreachable, out);
            return;
        };
        let retry = if again { None } else { Some(what.clone()) };
        let parts = match &what {
            Readable::Jig(jig_client_domain::contract::Read::Tasks { query, .. }) => query.parts.clone(),
            Readable::Jig(_) | Readable::Family { .. } => skein_lib::List::with_capacity(0),
        };
        let Ok(record) = translate::reads::owned(what, self.limits.wire()) else {
            read_none(read, &owner, jig_client_domain::contract::ReadNone::Unreachable, out);
            return;
        };
        let Ok(bytes) = record.encode(self.limits.wire()) else {
            read_none(read, &owner, jig_client_domain::contract::ReadNone::Unreachable, out);
            return;
        };
        let exchange = Token::new(self.next);
        self.next = next;
        let state = Attempt {
            parts,
            held: None,
            token: read,
            owner,
            version: record.version,
            kind: ExchangeKind::Read { turn, retry },
        };
        self.attempts.insert(exchange, state).expect("checked exchange capacity");
        self.exchange(exchange, Message::Read(bytes), out);
    }
    pub(super) fn answered(&mut self, exchange: Token, body: &[u8], out: &mut Queue<Request>) {
        let Some(attempt) = self.attempts.remove(&exchange) else { return };
        match attempt.kind {
            ExchangeKind::SignIn | ExchangeKind::SignInApproval | ExchangeKind::SignOut { .. } => {
                self.signing_answer(attempt.kind, attempt.held, body, out);
                return;
            }
            ExchangeKind::LinkAsk => {
                self.asked_link(body, out);
                return;
            }
            ExchangeKind::Approval(kind) => {
                self.approval_answer(kind, body, out);
                return;
            }
            ExchangeKind::Authorize => {
                self.authorized(body, out);
                return;
            }
            ExchangeKind::Attempt { .. } => {}
            ExchangeKind::WatchOpen { name } | ExchangeKind::WatchClose { name } => {
                self.watch_answer(attempt, name, body, out);
                return;
            }
            ExchangeKind::Read { .. } => {
                self.read_answer(attempt, body, out);
                return;
            }
        }
        let Ok(record) = w::KeyedAnswer::decode(self.limits.wire(), &mut Reader::new(body)) else {
            out.push(Request::Fact { fact: Fact::UndecodableAnswer });
            out.push(Request::Answer {
                attempt: attempt.token,
                envelope: RequestEnvelope::Lost,
                answer: None,
                why: Some(Retry::Undecodable),
            });
            return;
        };
        if let w::KeyedAnswer::NotServed(value) = &record {
            translate::envelope::not_served(value, out);
            out.push(Request::Answer {
                attempt: attempt.token,
                envelope: RequestEnvelope::Again,
                answer: None,
                why: None,
            });
            return;
        }
        let why = match &record {
            w::KeyedAnswer::Busy => Some(Retry::Busy),
            w::KeyedAnswer::AnswersFull(value) => {
                Some(Retry::Full { held: value.held, room_at: Wall::from_nanos(value.room_at) })
            }
            w::KeyedAnswer::NotReady => Some(Retry::NotReady),
            w::KeyedAnswer::SignedOut | w::KeyedAnswer::NotServed(_) => None,
            w::KeyedAnswer::FinalAnswer(_) => Some(Retry::Undecodable),
        };
        let envelope = match record {
            w::KeyedAnswer::NotServed(_) => unreachable!("version refusal returned"),
            w::KeyedAnswer::Busy | w::KeyedAnswer::NotReady | w::KeyedAnswer::AnswersFull(_) => RequestEnvelope::Again,
            w::KeyedAnswer::SignedOut => RequestEnvelope::SignedOut,
            w::KeyedAnswer::FinalAnswer(record) => {
                let credential = translate::answers::credential(&record, &self.limits);
                let answer = self.final_answer(&attempt, record);
                match answer {
                    Ok(answer) => {
                        if let Some((party, credential)) = credential {
                            out.push(Request::ServiceCredential { party, credential });
                        }
                        out.push(Request::Answer {
                            attempt: attempt.token,
                            envelope: RequestEnvelope::Final,
                            answer: Some(answer),
                            why: None,
                        });
                        return;
                    }
                    Err(_) => {
                        out.push(Request::Fact { fact: Fact::UndecodableAnswer });
                        RequestEnvelope::Lost
                    }
                }
            }
        };
        if envelope == RequestEnvelope::SignedOut {
            self.end_sign_in(out);
            out.push(Request::Signing { signing: crate::Signing::SignedOut });
        }
        out.push(Request::Answer { attempt: attempt.token, envelope, answer: None, why });
    }
    fn final_answer(&self, attempt: &Attempt, record: w::Owned) -> Result<Answered, translate::Undecodable> {
        if record.owner == attempt.owner && record.version == attempt.version {
            return match &record.owner {
                w::Owner::Jig => match translate::answers::answer(w::KeyedAnswer::FinalAnswer(record), &self.limits) {
                    Ok(answer) => Ok(Answered::Jig(answer)),
                    Err(problem) => Err(problem),
                },
                w::Owner::Family(owner) => Ok(Answered::Family(Family {
                    family: owner.number,
                    version: record.version,
                    bytes: skein_lib::bytes::copy_of(&record.record),
                })),
            };
        }
        if record.owner == w::Owner::Jig && record.version == w::VERSION && attempt.owner != w::Owner::Jig {
            let answer = translate::answers::answer(w::KeyedAnswer::FinalAnswer(record), &self.limits)?;
            return match answer {
                answer @ jig_client_domain::contract::Answer::Refused { .. } => Ok(Answered::Jig(answer)),
                jig_client_domain::contract::Answer::Done(_) => Err(translate::Undecodable::Version),
            };
        }
        Err(translate::Undecodable::Version)
    }
    fn read_answer(&mut self, attempt: Attempt, body: &[u8], out: &mut Queue<Request>) {
        if let ExchangeKind::Read { turn: Some(identity), .. } = attempt.kind {
            match translate::turn::inspect(body, identity, &self.limits) {
                Ok(None) => {}
                Ok(Some(why)) => {
                    read_none(attempt.token, &attempt.owner, why, out);
                    return;
                }
                Err(_) => {
                    read_none(attempt.token, &attempt.owner, jig_client_domain::contract::ReadNone::Malformed, out);
                    return;
                }
            }
        }
        let result = match w::ReadAnswer::decode(self.limits.wire(), &mut Reader::new(body)) {
            Ok(w::ReadAnswer::NotServed(value)) => {
                translate::envelope::not_served(&value, out);
                out.push(Request::Read {
                    read: attempt.token,
                    result: Readout::NotServed {
                        family: translate::envelope::family(&value.owner),
                        served: translate::envelope::versions(&value),
                    },
                });
                return;
            }
            Ok(w::ReadAnswer::None(record)) => {
                if record.value == w::ReadNone::SignedOut {
                    self.end_sign_in(out);
                    out.push(Request::Signing { signing: crate::Signing::SignedOut });
                }
                read_none(attempt.token, &attempt.owner, translate::reads::none(&record.value), out);
                return;
            }
            Ok(w::ReadAnswer::Value(record)) => read_value(&attempt, record, &self.limits),
            Err(_) => Err(translate::Undecodable::Malformed),
        };
        match result {
            Ok(result) => out.push(Request::Read { read: attempt.token, result }),
            Err(_) => {
                out.push(Request::Fact { fact: Fact::UndecodableDocument });
                match attempt.kind {
                    ExchangeKind::Read { .. } => {
                        read_none(attempt.token, &attempt.owner, jig_client_domain::contract::ReadNone::Malformed, out);
                    }
                    ExchangeKind::LinkAsk
                    | ExchangeKind::Approval(_)
                    | ExchangeKind::SignInApproval
                    | ExchangeKind::Authorize
                    | ExchangeKind::Attempt { .. }
                    | ExchangeKind::SignIn
                    | ExchangeKind::SignOut { .. }
                    | ExchangeKind::WatchOpen { .. }
                    | ExchangeKind::WatchClose { .. } => failed(attempt, out),
                }
            }
        }
    }
}
fn read_value(attempt: &Attempt, record: w::Owned, limits: &Limits) -> Result<Readout, translate::Undecodable> {
    if record.owner != attempt.owner || record.version != attempt.version {
        return Err(translate::Undecodable::Version);
    }
    match &record.owner {
        w::Owner::Jig => {
            let value = translate::reads::value(&record.record, limits)?;
            if let jig_client_domain::contract::ReadValue::Tasks(page) = &value {
                translate::queries::check_rows(&page.live, &attempt.parts)?;
                match &page.ended {
                    jig_client_domain::contract::EndedGroup::Rows { rows, .. } => {
                        translate::queries::check_rows(rows, &attempt.parts)?;
                    }
                    jig_client_domain::contract::EndedGroup::None => (),
                }
            }
            Ok(Readout::Jig(jig_client_domain::contract::ReadResult::Value(value)))
        }
        w::Owner::Family(owner) => {
            let family = owner.number;
            Ok(Readout::Family { family, record: Some(record.record) })
        }
    }
}
fn read_none(read: Token, owner: &w::Owner, why: jig_client_domain::contract::ReadNone, out: &mut Queue<Request>) {
    use jig_client_domain::contract::ReadNone;
    // Busy and loss belong to the exchange, not the family's document: callers can retry them
    // (protocol/client/wire.md, 2.3; protocol/client/README.md, 11).
    let transient = match why {
        ReadNone::Busy | ReadNone::Unreachable => true,
        ReadNone::TooLarge { .. }
        | ReadNone::TooLinked { .. }
        | ReadNone::Malformed
        | ReadNone::Unknown
        | ReadNone::Refused
        | ReadNone::Gone
        | ReadNone::SignedOut => false,
    };
    let result = match owner {
        w::Owner::Family(owner) if !transient => Readout::Family { family: owner.number, record: None },
        w::Owner::Jig | w::Owner::Family(_) => Readout::Jig(jig_client_domain::contract::ReadResult::None(why)),
    };
    out.push(Request::Read { read, result });
}
fn failed(attempt: Attempt, out: &mut Queue<Request>) {
    match attempt.kind {
        ExchangeKind::LinkAsk => {
            out.push(Request::AskedLink(crate::AskedLink::NotAsked(crate::NotAsked::Lost)));
        }
        ExchangeKind::Approval(kind) => out.push(Request::Approval(kind.lost())),
        ExchangeKind::Authorize => out.push(Request::Authorized(crate::Authorized::Lost)),
        ExchangeKind::Attempt { .. } => lost(attempt.token, out),
        ExchangeKind::SignIn | ExchangeKind::SignInApproval | ExchangeKind::SignOut { .. } => {
            out.push(Request::Signing { signing: crate::Signing::Lost });
        }
        ExchangeKind::WatchOpen { .. } | ExchangeKind::WatchClose { .. } => {
            unreachable!("watch failure belongs to its map")
        }
        ExchangeKind::Read { .. } => {
            read_none(attempt.token, &attempt.owner, jig_client_domain::contract::ReadNone::Unreachable, out);
        }
    }
}
fn lost(attempt: Token, out: &mut Queue<Request>) {
    out.push(Request::Answer { attempt, envelope: RequestEnvelope::Lost, answer: None, why: Some(Retry::Lost) });
}
fn unsent(attempt: Token, out: &mut Queue<Request>) {
    out.push(Request::Answer { attempt, envelope: RequestEnvelope::Again, answer: None, why: None });
}
fn keyed(key: RequestKey, ask: Ask, value: Option<Secret>, limits: &w::Limits) -> Result<w::Keyed, w::Problem> {
    let (owner, version, request) = match ask {
        Ask::Jig(ask) => {
            let record = translate::asks::ask(&ask, limits)?;
            let mut writer = Writer::new(skein_lib::length::usize_of(record.measure(limits)?));
            record.write(limits, &mut writer).expect("measured ask");
            (w::Owner::Jig, w::VERSION, writer.finish())
        }
        Ask::Family { scope, record } => {
            let owner = w::Family { number: record.family };
            (w::Owner::Family(owner), record.version, translate::envelope::request(scope, &record, limits)?)
        }
    };
    Ok(w::Keyed { key, owner, version, request, value })
}

impl Link {
    pub(super) fn token(&mut self) -> Option<Token> {
        let next = self.next.checked_add(1)?;
        let token = Token::new(self.next);
        self.next = next;
        Some(token)
    }
    fn open(&mut self, watch: Token, subject: Watchable, out: &mut Queue<Request>) {
        if self.watches.0.contains_key(&watch) {
            return;
        }
        let reserved = u32::from(!self.watches.0.contains_key(&Token::new(0)));
        if self.watches.0.len() >= self.watches.0.capacity().saturating_sub(reserved) {
            out.push(Request::Ended { watch, end: jig_client_domain::contract::WatchEnd::Busy });
            return;
        }
        let parts = match &subject {
            Watchable::Jig(jig_client_domain::contract::Subject::Tasks { query, .. }) => query.parts.clone(),
            Watchable::Jig(_) | Watchable::Family { .. } => skein_lib::List::with_capacity(0),
        };
        let Ok(subject) = translate::watches::owned(subject, self.limits.wire()) else {
            out.push(Request::Ended { watch, end: jig_client_domain::contract::WatchEnd::Dropped });
            return;
        };
        if subject.measure(self.limits.wire()).is_err() {
            out.push(Request::Ended { watch, end: jig_client_domain::contract::WatchEnd::Dropped });
            return;
        }
        let Some(name) = self.token() else {
            out.push(Request::Ended { watch, end: jig_client_domain::contract::WatchEnd::Dropped });
            return;
        };
        let opening = crate::watches::Opening {
            parts,
            name: name.raw(),
            owner: subject.owner.clone(),
            version: subject.version,
            subject: Some(subject),
            live: false,
            admitted: false,
            closing: false,
            held: None,
        };
        self.watches.0.insert(watch, opening).expect("checked watch capacity");
        if let Some(stream) = &self.stream
            && stream.name.is_some()
        {
            self.send_open(watch, out);
        }
    }
    fn open_stream(&mut self, now: Time, _out: &mut Queue<Request>) {
        if self.stream.is_some() {
            return;
        }
        self.begin_stream(now);
    }
    fn begin_stream(&mut self, now: Time) {
        self.watches
            .0
            .insert(
                Token::new(0),
                crate::watches::Opening {
                    parts: skein_lib::List::with_capacity(0),
                    name: 0,
                    owner: w::Owner::Jig,
                    version: wire::VERSION,
                    subject: None,
                    live: false,
                    admitted: true,
                    closing: false,
                    held: None,
                },
            )
            .expect("room reserved for stream me");
        self.stream = Some(crate::silence::Silence::new(self.limits.client().heartbeat, now));
        self.gathered.push(Gathered::OpenStream { credential: self.credential.clone() });
    }
    fn has_watch_room(&self) -> bool {
        let mut sent = self.retired.len();
        for (_, opening) in &self.watches.0 {
            if opening.subject.is_none() {
                sent = sent.saturating_add(1);
            }
        }
        sent < self.limits.client().watches
    }
    fn send_open(&mut self, watch: Token, out: &mut Queue<Request>) {
        if !self.has_watch_room() {
            let opening = self.watches.0.get_mut(&watch).expect("pending opening");
            opening.held = Some(Token::new(opening.name));
            return;
        }
        let Some(stream) = &self.stream else { return };
        let Some(stream) = stream.name else { return };
        let Some(exchange) = self.token() else {
            self.watches.end(watch, jig_client_domain::contract::WatchEnd::Dropped, out);
            return;
        };
        let opening = self.watches.0.get_mut(&watch).expect("pending opening");
        let Some(subject) = opening.subject.take() else { return };
        opening.held = None;
        let record = w::Open { stream, watch: opening.name, subject };
        let attempt = Attempt {
            parts: skein_lib::List::with_capacity(0),
            held: None,
            token: watch,
            owner: opening.owner.clone(),
            version: opening.version,
            kind: ExchangeKind::WatchOpen { name: opening.name },
        };
        self.attempts.insert(exchange, attempt).expect("checked opening exchange capacity");
        let mut writer = Writer::new(skein_lib::length::usize_of(
            record.measure(self.limits.wire()).expect("bounded client record"),
        ));
        record.write(self.limits.wire(), &mut writer).expect("measured opening");
        self.exchange(exchange, Message::Watch(writer.finish()), out);
    }
    fn close(&mut self, watch: Token, out: &mut Queue<Request>) {
        let Some(opening) = self.watches.0.get(&watch) else { return };
        if opening.closing {
            return;
        }
        let name = opening.name;
        let unsent = opening.subject.is_some() || self.unsent_open(watch, name);
        if unsent {
            self.cancel_open(watch, name);
            self.watches.end(watch, jig_client_domain::contract::WatchEnd::Closed, out);
            return;
        }
        let Some(order) = self.token() else { return self.drop_stream(out) };
        let opening = self.watches.0.get_mut(&watch).expect("closing watch");
        opening.closing = true;
        opening.held = Some(order);
    }
    fn unsent_open(&self, watch: Token, name: u64) -> bool {
        for pending in &self.gathered {
            match pending {
                Gathered::Exchange { exchange, .. } => {
                    if let Some(attempt) = self.attempts.get(exchange) {
                        match attempt.kind {
                            ExchangeKind::WatchOpen { name: pending } => {
                                if pending == name && attempt.token == watch {
                                    return true;
                                }
                            }
                            ExchangeKind::LinkAsk
                            | ExchangeKind::Authorize
                            | ExchangeKind::Approval(_)
                            | ExchangeKind::SignInApproval
                            | ExchangeKind::SignIn
                            | ExchangeKind::SignOut { .. }
                            | ExchangeKind::Attempt { .. }
                            | ExchangeKind::Read { .. }
                            | ExchangeKind::WatchClose { .. } => {}
                        }
                    }
                }
                Gathered::OpenStream { .. } | Gathered::CloseStream => {}
            }
        }
        false
    }
    fn send_close(&mut self, watch: Token, out: &mut Queue<Request>) {
        let Some(stream) = &self.stream else { return };
        let stream = stream.name.expect("sent opening has hello");
        let Some(exchange) = self.token() else { return self.drop_stream(out) };
        let opening = self.watches.0.get_mut(&watch).expect("held closing");
        let name = opening.name;
        opening.held = None;
        let attempt = Attempt {
            parts: skein_lib::List::with_capacity(0),
            held: None,
            token: watch,
            owner: opening.owner.clone(),
            version: opening.version,
            kind: ExchangeKind::WatchClose { name },
        };
        self.attempts.insert(exchange, attempt).expect("checked closing exchange capacity");
        self.unwatch(exchange, stream, name, out);
    }
    fn resume_watches(&mut self, out: &mut Queue<Request>) {
        for _ in 0..self.watches.0.len() {
            let mut first: Option<(Token, Token)> = None;
            for (watch, opening) in &self.watches.0 {
                if let Some(order) = opening.held {
                    if !opening.closing && !self.has_watch_room() {
                        continue;
                    }
                    if opening.closing && !opening.admitted {
                        continue;
                    }
                    if let Some((_, previous)) = first
                        && previous < order
                    {
                        continue;
                    }
                    first = Some((*watch, order));
                }
            }
            let Some((watch, _)) = first else { return };
            if self.watches.0.get(&watch).expect("held watch").closing {
                self.send_close(watch, out);
            } else {
                self.send_open(watch, out);
                if let Some(opening) = self.watches.0.get(&watch)
                    && opening.held.is_some()
                {
                    return;
                }
            }
        }
    }
    pub(super) fn cancel_open(&mut self, watch: Token, name: u64) {
        let mut tokens = skein_lib::List::with_capacity(self.attempts.len());
        for (exchange, attempt) in &self.attempts {
            match attempt.kind {
                ExchangeKind::WatchOpen { name: old } | ExchangeKind::WatchClose { name: old } => {
                    if attempt.token == watch && old == name {
                        tokens.push(*exchange).expect("bounded exchanges");
                    }
                }
                ExchangeKind::LinkAsk
                | ExchangeKind::Approval(_)
                | ExchangeKind::SignInApproval
                | ExchangeKind::Authorize
                | ExchangeKind::Attempt { .. }
                | ExchangeKind::Read { .. }
                | ExchangeKind::SignIn
                | ExchangeKind::SignOut { .. } => {}
            }
        }
        for exchange in &tokens {
            assert!(self.attempts.remove(exchange).is_some(), "collected live watch exchange");
        }
    }
    fn forget(&mut self, watch: Token, out: &mut Queue<Request>) {
        if let Some(opening) = self.watches.0.remove(&watch) {
            if opening.subject.is_some() || self.unsent_open(watch, opening.name) {
                self.cancel_open(watch, opening.name);
                return;
            }
            self.retired.insert(opening.name, watch).expect("retired watches keep their engine room");
            if opening.admitted {
                self.cancel_open(watch, opening.name);
                self.retired_close(watch, opening.name, out);
            }
        }
    }
    pub(super) fn unwatch(&mut self, exchange: Token, stream: [u8; 16], watch: u64, out: &mut Queue<Request>) {
        let record = w::WatchClose { stream, watch };
        let mut writer = Writer::new(skein_lib::length::usize_of(
            record.measure(self.limits.wire()).expect("bounded client record"),
        ));
        record.write(self.limits.wire(), &mut writer).expect("measured closing");
        self.exchange(exchange, Message::Unwatch(writer.finish()), out);
    }
    fn watch_answer(&mut self, attempt: Attempt, name: u64, body: &[u8], out: &mut Queue<Request>) {
        if self.watches.named(name) != Some(attempt.token) {
            self.retired_answer(attempt, name, body, out);
            return;
        }
        match attempt.kind {
            ExchangeKind::WatchOpen { .. } => match w::OpenAnswer::decode(self.limits.wire(), &mut Reader::new(body)) {
                Ok(w::OpenAnswer::NotServed(value)) => {
                    translate::envelope::not_served(&value, out);
                    self.watches.end(attempt.token, jig_client_domain::contract::WatchEnd::NotServed, out);
                }
                Ok(w::OpenAnswer::Admitted) => {
                    self.watches.0.get_mut(&attempt.token).expect("named opening").admitted = true;
                    self.watches.opened(attempt.token, out);
                }
                Ok(w::OpenAnswer::Refused(why)) => {
                    let end = translate::watches::refused(&why.why);
                    self.watches.end(attempt.token, end, out);
                    if end == jig_client_domain::contract::WatchEnd::SignedOut {
                        self.end_sign_in(out);
                        out.push(Request::Signing { signing: crate::Signing::SignedOut });
                    }
                }
                Err(_) => {
                    out.push(Request::Fact { fact: Fact::UndecodableWatchAnswer });
                    self.failure(attempt, out);
                }
            },
            ExchangeKind::WatchClose { .. } => {
                match w::CloseAnswer::decode(self.limits.wire(), &mut Reader::new(body)) {
                    Ok(w::CloseAnswer::Closed) => {
                        self.watches.end(attempt.token, jig_client_domain::contract::WatchEnd::Closed, out);
                    }
                    Ok(w::CloseAnswer::SignedOut) => {
                        self.watches.end(attempt.token, jig_client_domain::contract::WatchEnd::SignedOut, out);
                        self.end_sign_in(out);
                        out.push(Request::Signing { signing: crate::Signing::SignedOut });
                    }
                    Err(_) => {
                        out.push(Request::Fact { fact: Fact::UndecodableWatchAnswer });
                        self.failure(attempt, out);
                    }
                }
            }
            ExchangeKind::LinkAsk
            | ExchangeKind::Approval(_)
            | ExchangeKind::SignInApproval
            | ExchangeKind::Authorize
            | ExchangeKind::Attempt { .. }
            | ExchangeKind::Read { .. }
            | ExchangeKind::SignIn
            | ExchangeKind::SignOut { .. } => {
                unreachable!("watch exchange kind")
            }
        }
    }
    pub(super) fn failure(&mut self, attempt: Attempt, out: &mut Queue<Request>) {
        match attempt.kind {
            ExchangeKind::WatchOpen { name } | ExchangeKind::WatchClose { name } => {
                if self.watches.named(name) == Some(attempt.token) || self.retired.get(&name) == Some(&attempt.token) {
                    self.drop_stream(out);
                }
            }
            ExchangeKind::SignOut { generation } if generation != self.generation => {}
            ExchangeKind::SignIn | ExchangeKind::SignInApproval | ExchangeKind::SignOut { .. } => {
                out.push(Request::Signing { signing: crate::Signing::Lost });
            }
            ExchangeKind::LinkAsk => {
                out.push(Request::AskedLink(crate::AskedLink::NotAsked(crate::NotAsked::Lost)));
            }
            ExchangeKind::Approval(kind) => out.push(Request::Approval(kind.lost())),
            ExchangeKind::Read { retry: Some(what), .. } => out.push(Request::ReadAgain { read: attempt.token, what }),
            ExchangeKind::Authorize | ExchangeKind::Attempt { .. } | ExchangeKind::Read { retry: None, .. } => {
                failed(attempt, out);
            }
        }
    }
    fn streamed(&mut self, now: Time, message: Streamed, out: &mut Queue<Request>) {
        let Some(stream) = &mut self.stream else { return };
        stream.progress(now);
        let result = match message {
            Streamed::Full(full) => {
                self.drop_stream(out);
                out.push(Request::StreamsFull { full });
                return;
            }
            Streamed::Heartbeat => {
                self.watches.alive(out);
                return;
            }
            Streamed::SignedOut => {
                self.end_sign_in(out);
                out.push(Request::Signing { signing: crate::Signing::SignedOut });
                return;
            }
            Streamed::Record(record) => {
                let mut reader = Reader::new(&record);
                match w::StreamEvent::decode(self.limits.receiving(), &mut reader) {
                    Ok(record) if reader.remaining() == 0 => self.stream_event(record, out),
                    Ok(_) => Err(StreamRefusal::Record(crate::Undecodable::Malformed)),
                    Err(problem) => Err(StreamRefusal::Record(translate::classify(problem))),
                }
            }
        };
        if let Err(why) = result {
            match why {
                StreamRefusal::Record(crate::Undecodable::Malformed | crate::Undecodable::Version) => {
                    out.push(Request::Fact { fact: Fact::UndecodableEvent });
                }
                StreamRefusal::Record(crate::Undecodable::TooLarge) | StreamRefusal::Order => {}
            }
            self.drop_stream(out);
        }
    }
    fn stream_event(&mut self, record: w::StreamEvent, out: &mut Queue<Request>) -> Result<(), StreamRefusal> {
        let stream = self.stream.as_mut().expect("active stream");
        match record {
            w::StreamEvent::Hello(hello) => {
                if stream.name.is_some() {
                    return Err(StreamRefusal::Order);
                }
                let served = match hello_versions(&hello, None) {
                    Some(range) => crate::serves(range),
                    None => false,
                };
                if served {
                    return Err(StreamRefusal::Order);
                }
                self.hello(hello, out)?;
            }
            w::StreamEvent::Opened(opened) => {
                if stream.name.is_some() {
                    return Err(StreamRefusal::Order);
                }
                self.hello(opened.hello, out)?;
                let value = w::Delivery { watch: 0, document: opened.me };
                if let Err(why) = self.watches.delivery(value, &self.limits, out) {
                    return Err(StreamRefusal::Record(why));
                }
            }
            w::StreamEvent::Delivery(value) => {
                if stream.name.is_none() {
                    return Err(StreamRefusal::Order);
                }
                if let Err(why) = self.watches.delivery(value, &self.limits, out) {
                    return Err(StreamRefusal::Record(why));
                }
            }
            w::StreamEvent::Ended(value) => {
                if stream.name.is_none() {
                    return Err(StreamRefusal::Order);
                }
                if value.watch == 0 {
                    if value.why == w::WatchEnd::SignedOut {
                        self.end_sign_in(out);
                        out.push(Request::Signing { signing: crate::Signing::SignedOut });
                    } else {
                        self.drop_stream(out);
                    }
                    return Ok(());
                }
                if let Some(watch) = self.retired.remove(&value.watch) {
                    self.cancel_open(watch, value.watch);
                }
                if let Some(watch) = self.watches.named(value.watch) {
                    self.cancel_open(watch, value.watch);
                    if let w::WatchEnd::NotServed(value) = &value.why {
                        translate::envelope::not_served(value, out);
                    }
                    let end = translate::watches::ended(&value.why);
                    self.watches.end(watch, end, out);
                    if end == jig_client_domain::contract::WatchEnd::SignedOut {
                        self.end_sign_in(out);
                        out.push(Request::Signing { signing: crate::Signing::SignedOut });
                    }
                }
            }
            w::StreamEvent::Text(value) => {
                if stream.name.is_none() {
                    return Err(StreamRefusal::Order);
                }
                if let Some(watch) = self.watches.named(value.watch) {
                    let opening = self.watches.0.get(&watch).expect("named run watch");
                    if !opening.closing {
                        if opening.owner != w::Owner::Jig || opening.version != w::VERSION {
                            return Err(StreamRefusal::Record(crate::Undecodable::Version));
                        }
                        let value = match translate::live::run_text(value.value, &self.limits) {
                            Ok(value) => value,
                            Err(why) => return Err(StreamRefusal::Record(why)),
                        };
                        self.watches.opened(watch, out);
                        out.push(Request::Watch {
                            watch,
                            event: jig_client_domain::contract::WatchEvent::Delivered(
                                jig_client_domain::contract::Delivery::Run(Box::new(value)),
                            ),
                        });
                    }
                }
            }
        }
        self.watches.alive(out);
        Ok(())
    }
    fn hello(&mut self, hello: w::Hello, out: &mut Queue<Request>) -> Result<(), StreamRefusal> {
        let stream = self.stream.as_mut().expect("active stream");
        if let Err(why) = stream.hello(&hello) {
            return Err(StreamRefusal::Record(why));
        }
        for served in &hello.served {
            if served.first > served.last {
                return Err(StreamRefusal::Record(crate::Undecodable::Malformed));
            }
        }
        self.predecessor = Some(hello.stream);
        if let Err(why) = self.save_stream(hello.stream, out) {
            return Err(StreamRefusal::Record(why));
        }
        out.push(Request::Hello { family: None, versions: hello_versions(&hello, None) });
        for owner in self.limits.client().owners {
            let family = Some(owner.family);
            out.push(Request::Hello { family, versions: hello_versions(&hello, family) });
        }
        let mut pending = skein_lib::List::with_capacity(self.limits.client().watches);
        for (watch, opening) in &self.watches.0 {
            let family = match &opening.owner {
                w::Owner::Jig => None,
                w::Owner::Family(owner) => Some(owner.number),
            };
            let served = match hello_versions(&hello, family) {
                Some(range) => range.contains(opening.version),
                None => false,
            };
            if opening.subject.is_some() && served {
                pending.push(*watch).expect("bounded watches");
            }
        }
        for watch in &pending {
            let opening = self.watches.0.get_mut(watch).expect("pending opening");
            opening.held = Some(Token::new(opening.name));
        }
        Ok(())
    }
    pub(super) fn end_sign_in(&mut self, out: &mut Queue<Request>) {
        self.credential = None;
        self.expires = None;
        while let Some((key, _)) = (&self.attempts).into_iter().next() {
            let key = *key;
            let attempt = self.attempts.remove(&key).expect("listed attempt");
            match attempt.kind {
                ExchangeKind::LinkAsk => {
                    out.push(Request::AskedLink(crate::AskedLink::NotAsked(crate::NotAsked::Lost)));
                }
                ExchangeKind::Approval(kind) => out.push(Request::Approval(kind.lost())),
                ExchangeKind::Authorize => out.push(Request::Authorized(crate::Authorized::Lost)),
                ExchangeKind::SignInApproval
                | ExchangeKind::SignIn
                | ExchangeKind::SignOut { .. }
                | ExchangeKind::Attempt { .. }
                | ExchangeKind::Read { .. }
                | ExchangeKind::WatchOpen { .. }
                | ExchangeKind::WatchClose { .. } => {}
            }
            // The table's attempts end with the sign-in; a carried one has no other answer.
            if attempt.carrying() {
                lost(attempt.token, out);
            }
        }
        // The sign-in's terminal ends the domain's me; it has no opening to answer.
        self.watches.0.remove(&Token::new(0));
        while let Some((watch, _)) = (&self.watches.0).into_iter().next() {
            let watch = *watch;
            self.watches.end(watch, jig_client_domain::contract::WatchEnd::SignedOut, out);
        }
        self.retired = Map::uncapped();
        self.discard_watch_messages();
        if self.stream.take().is_some() {
            self.gathered.push(Gathered::CloseStream);
        }
    }
    fn discard_watch_messages(&mut self) {
        for _ in 0..self.gathered.len() {
            let value = self.gathered.pop().expect("counted gathered transport");
            match value {
                Gathered::Exchange { message: Message::Watch(_) | Message::Unwatch(_), .. } => {}
                value @ (Gathered::Exchange { .. } | Gathered::OpenStream { .. } | Gathered::CloseStream) => {
                    self.gathered.push(value);
                }
            }
        }
    }
    pub(super) fn drop_stream(&mut self, out: &mut Queue<Request>) {
        let Some(_) = self.stream.take() else { return };
        let retired = core::mem::replace(&mut self.retired, Map::uncapped());
        for (name, watch) in &retired {
            self.cancel_open(*watch, *name);
        }
        let mut watches = skein_lib::List::with_capacity(self.limits.client().watches);
        for (watch, _) in &self.watches.0 {
            watches.push(*watch).expect("bounded watches");
        }
        for watch in &watches {
            let name = self.watches.0.get(watch).expect("collected watch").name;
            self.cancel_open(*watch, name);
            self.watches.end(*watch, jig_client_domain::contract::WatchEnd::Dropped, out);
        }
        self.discard_watch_messages();
        self.gathered.push(Gathered::CloseStream);
        out.push(Request::Fact { fact: Fact::StreamDropped });
    }
}

fn hello_versions(hello: &w::Hello, family: Option<u16>) -> Option<jig_client_domain::contract::VersionRange> {
    for served in &hello.served {
        let owner = match &served.owner {
            w::Owner::Jig => None,
            w::Owner::Family(owner) => Some(owner.number),
        };
        if owner == family {
            return Some(jig_client_domain::contract::VersionRange { first: served.first, last: served.last });
        }
    }
    None
}
