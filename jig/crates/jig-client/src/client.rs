//! Exhaustive routing between the domain and link (client/domain/client/README.md, 3).
use crate::{
    Approval, ApprovalCode, Ask, Document, DomainLimits, Handle, Key, Limits, Link, Message, Pending, Readable, Record,
    Streamed, Taking, Told, Unusable, Watchable, With,
};
use alloc::boxed::Box;
use jig_client_domain::{self as domain, Down, Up};
use jig_client_protocol_link as link;
use skein_lib::{Duration, Env, List, Queue, Rng, Secret, Time, Token, Wall};
#[derive(PartialEq, Eq)]
pub enum Event {
    Start { kept: List<Record> },
    StartUnlinked { kept: List<skein_lib::RequestRecord<crate::Held>> },
    SignIn { with: With },
    SignOut,
    Authorize { query: Box<[u8]> },
    AskApproval,
    ReadApproval { code: ApprovalCode },
    DecideApproval { code: ApprovalCode, approve: bool },
    ApprovalAnswer { answer: Approval },
    SignOutLost,
    Watch { handle: Handle, subject: Watchable },
    Unwatch { handle: Handle },
    Read { handle: Handle, read: Readable },
    Send { key: Key, name: Option<skein_lib::RequestKey>, request: Ask },
    // A value carried either way: sent once under its name, never kept (client/domain/client/requests.md).
    Carry { key: Key, name: skein_lib::RequestKey, request: Ask, value: Option<Secret> },
    Reply { exchange: Token, message: Box<[u8]> },
    Lost { exchange: Token },
    Streamed { message: Streamed },
    StreamGone,
    Stored,
    NotStored,
    Up { up: Up },
}
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Exchange {
        exchange: Token,
        credential: Option<Secret>,
        message: Message,
    },
    OpenStream {
        credential: Option<Secret>,
    },
    CloseStream,
    Save {
        records: List<Record>,
    },
    Erase {
        records: List<Box<[u8]>>,
    },
    Told {
        told: Told,
    },
    Authorized(crate::Authorized),
    Approval(Approval),
    /// Shown once by the host; the client never retains it.
    ServiceCredential {
        party: u64,
        credential: Secret,
    },
}
/// What an unlinked client asks of its host (client/domain/client/README.md, 4.4).
#[derive(Debug, PartialEq, Eq)]
pub enum UnlinkedRequest {
    ApprovalAct(ApprovalAct),
    Told { told: Told },
    Authorized(crate::Authorized),
    Approval(Approval),
    Down { down: Down },
    SignOut,
}
/// Why the last sign-in installed no credential (jig/docs/design/client/domain/client/README.md, 9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotSignedIn {
    Approval(Taking),
    LinkUsed,
    LinkExpired,
    LinkUnknown,
    NotReady,
    Busy,
    Full { sign_ins: u32, earliest: Wall, link_until: Wall },
}
#[derive(Debug)]
pub struct LinkView<'a> {
    pub state: Link,
    pub unreadable: bool,
    pub party: Option<u64>,
    pub not_served: &'a [u16],
    pub sign_out_lost: bool,
    pub not_signed_in: Option<NotSignedIn>,
    pub streams_full: Option<domain::StreamsFull>,
    pub watches_full: bool,
    pub credential: Option<&'a Secret>,
    pub expires: Option<Wall>,
}
#[derive(Debug)]
pub(crate) struct DeferredRead {
    read: Token,
    what: Readable,
}
#[derive(Debug)]
enum SendAgain {
    Approval { secret: Secret },
    Service(Secret),
    Link { secret: Secret, held: Option<Secret> },
}
impl SendAgain {
    fn with(&self) -> With {
        match self {
            Self::Approval { secret } => With::Approval { secret: secret.clone() },
            Self::Service(credential) => With::Service(credential.clone()),
            Self::Link { secret, held } => With::Link { secret: secret.clone(), held: held.clone() },
        }
    }
}
#[derive(Debug)]
pub struct Client {
    domain: domain::Domain,
    link: Option<link::Link>,
    limits: Limits,
    link_limits: link::Limits,
    domain_out: Queue<domain::Request>,
    link_out: Queue<link::Request>,
    store: crate::store::Store,
    send_again: Option<SendAgain>,
    signing_link: bool,
    not_signed_in: Option<NotSignedIn>,
    signing_until: Option<Time>,
    signing_span: Option<Duration>,
    signing_out: bool,
    sign_out_lost: bool,
    rng: Rng,
    saves: Queue<Record>,
    erases: Queue<Box<[u8]>>,
    reads: Queue<DeferredRead>,
}
/// Payload-free admission counters, copied only for an explicit diagnostic.
#[derive(Debug)]
pub struct Admission {
    pub attempts: u32,
    pub deferred_reads: u32,
}
impl Client {
    /// Receiving and retention bounds used by the client (client/domain/client/README.md, 11).
    #[must_use]
    pub const fn domain_limits(&self) -> &domain::Limits {
        &self.limits.domain
    }

    #[must_use]
    pub fn admission(&self) -> Admission {
        let attempts = match &self.link {
            Some(link) => link.in_flight(),
            None => 0,
        };
        Admission { attempts, deferred_reads: self.reads.len() }
    }

    /// # Errors
    /// Refuses limits whose queues or receiving bounds cannot be represented.
    pub fn new(limits: &Limits, seed: u64) -> Result<Self, Unusable> {
        if !jig_client_domain::valid(&limits.domain) {
            return Err(Unusable::Limits);
        }
        let Ok(bounds) = link::Limits::new(limits.domain, limits.exchanges) else {
            return Err(Unusable::Limits);
        };
        let Ok(link) = link::Link::new(&bounds) else {
            return Err(Unusable::Limits);
        };
        Ok(Self::build(limits, &bounds, seed, Some(link)))
    }
    /// # Errors
    /// Refuses limits whose queues or receiving bounds cannot be represented.
    pub fn unlinked(limits: &Limits, seed: u64) -> Result<Self, Unusable> {
        if !jig_client_domain::valid(&limits.domain) {
            return Err(Unusable::Limits);
        }
        let bounds = link::Limits::new(limits.domain, limits.exchanges)?;
        Ok(Self::build(limits, &bounds, seed, None))
    }
    fn build(limits: &Limits, link_limits: &link::Limits, seed: u64, link: Option<link::Link>) -> Self {
        Self {
            domain: domain::Domain::new(&limits.domain, seed),
            link,
            limits: *limits,
            link_limits: *link_limits,
            domain_out: Queue::uncapped(),
            link_out: Queue::uncapped(),
            store: crate::store::Store::new(limits.domain.requests.requests),
            send_again: None,
            signing_link: false,
            not_signed_in: None,
            signing_until: None,
            signing_span: None,
            signing_out: false,
            sign_out_lost: false,
            rng: Rng::new(seed),
            saves: Queue::uncapped(),
            erases: Queue::uncapped(),
            reads: Queue::uncapped(),
        }
    }
    pub fn step(&mut self, now: Time, wall: Wall, event: Event, out: &mut Queue<Request>) {
        assert!(self.link.is_some(), "an unlinked client steps with step_unlinked");
        let env = Env { now, wall, limits: self.limits.domain };
        match event {
            Event::Start { kept } => {
                domain::step(
                    &mut self.domain,
                    &env,
                    domain::Event::Start { kept: List::with_capacity(0) },
                    &mut self.domain_out,
                );
                self.link_step(now, link::Event::Start { kept });
            }
            Event::Watch { handle, subject } => self.domain_step(&env, domain::Event::Watch { handle, subject }),
            Event::Unwatch { handle } => self.domain_step(&env, domain::Event::Unwatch { handle }),
            Event::Read { handle, read } => self.domain_step(&env, domain::Event::Read { handle, read }),
            Event::Send { key, name, request } => self.domain_step(&env, domain::Event::Send { key, name, request }),
            Event::Carry { key, name, request, value } => {
                self.domain_step(&env, domain::Event::Carry { key, name, request, value });
            }
            Event::SignIn { with } => {
                self.signing_link = match &with {
                    With::Link { .. } | With::Approval { .. } => true,
                    With::Service(_) | With::Held(_) | With::Transport => false,
                };
                self.not_signed_in(None);
                self.send_again = match &with {
                    With::Service(credential) if credential.len() == 40 => Some(SendAgain::Service(credential.clone())),
                    With::Approval { secret } => Some(SendAgain::Approval { secret: secret.clone() }),
                    With::Link { secret, held } => Some(SendAgain::Link { secret: secret.clone(), held: held.clone() }),
                    With::Service(_) | With::Held(_) | With::Transport => None,
                };
                self.signing_until = None;
                self.signing_span = None;
                self.signing_out = false;
                self.sign_out_lost = false;
                let held = match &with {
                    With::Held(_) | With::Transport => true,
                    With::Link { .. } | With::Approval { .. } | With::Service(_) => false,
                };
                self.link_step(now, link::Event::SignIn { with });
                if held {
                    self.up(&env, Up::SignedIn);
                }
            }
            Event::AskApproval => self.link_step(now, link::Event::AskApproval),
            Event::ReadApproval { code } => self.link_step(now, link::Event::ReadApproval { code }),
            Event::DecideApproval { code, approve } => {
                self.link_step(now, link::Event::DecideApproval { code, approve });
            }
            Event::Authorize { query } => self.link_step(now, link::Event::Authorize { query }),
            Event::SignOut => {
                self.send_again = None;
                self.signing_until = None;
                self.signing_link = false;
                self.signing_out = true;
                self.sign_out_lost = false;
                self.link_step(now, link::Event::SignOut);
            }
            Event::Reply { exchange, message } => {
                self.link_step(now, link::Event::Reply { exchange, message });
            }
            Event::Lost { exchange } => self.link_step(now, link::Event::Lost { exchange }),
            Event::Streamed { message } => self.link_step(now, link::Event::Streamed { message }),
            Event::StreamGone => self.link_step(now, link::Event::StreamGone),
            Event::Stored => self.store_answer(&env, true),
            Event::NotStored => self.store_answer(&env, false),
            Event::ApprovalAnswer { .. } | Event::Up { .. } | Event::StartUnlinked { .. } | Event::SignOutLost => {
                assert!(self.link.is_none(), "Up and StartUnlinked require an unlinked client");
            }
        }
        self.drain(&env, out);
    }
    pub fn fire(&mut self, now: Time, wall: Wall, out: &mut Queue<Request>) {
        assert!(self.link.is_some(), "an unlinked client steps with step_unlinked");
        let env = Env { now, wall, limits: self.limits.domain };
        if let Some(until) = self.signing_until
            && now >= until
        {
            self.signing_until = None;
            if let Some(send_again) = &self.send_again {
                let with = send_again.with();
                self.signing_link = match send_again {
                    SendAgain::Link { .. } | SendAgain::Approval { .. } => true,
                    SendAgain::Service(_) => false,
                };
                self.link_step(now, link::Event::SignIn { with });
            }
        }
        if let Some(link) = &mut self.link {
            link.fire(now, &mut self.link_out);
        }
        self.drain(&env, out);
        if self.domain.link() != Link::OutOfDate {
            domain::fire(&mut self.domain, &env, &mut self.domain_out);
        }
        self.drain(&env, out);
    }
    /// Step an unlinked client (client/domain/client/README.md, 4.4).
    pub fn step_unlinked(&mut self, now: Time, wall: Wall, event: Event, out: &mut Queue<UnlinkedRequest>) {
        assert!(self.link.is_none(), "a linked client steps with step");
        let env = Env { now, wall, limits: self.limits.domain };
        self.unlinked_event(&env, event, out);
        self.drain_unlinked(out);
    }
    /// Fire an unlinked client's deadlines (client/domain/client/README.md, 4.4).
    pub fn fire_unlinked(&mut self, now: Time, wall: Wall, out: &mut Queue<UnlinkedRequest>) {
        assert!(self.link.is_none(), "a linked client steps with step");
        let env = Env { now, wall, limits: self.limits.domain };
        self.drain_unlinked(out);
        if self.domain.link() != Link::OutOfDate {
            domain::fire(&mut self.domain, &env, &mut self.domain_out);
        }
        self.drain_unlinked(out);
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        if self.domain.link() == Link::OutOfDate {
            return None;
        }
        let deadline = match &self.link {
            Some(link) => link.next_deadline(),
            None => None,
        };
        domain::next_deadline(&self.domain).into_iter().chain(deadline).chain(self.signing_until).min()
    }
    pub fn reclaim(&mut self) {
        domain::reclaim(&mut self.domain);
    }
    #[must_use]
    pub fn document(&self, handle: Handle) -> Option<Document<'_>> {
        self.domain.document(handle)
    }
    #[must_use]
    pub fn text(&self, handle: Handle) -> Option<&domain::contract::RunText> {
        self.domain.text(handle)
    }
    #[must_use]
    pub fn pending(&self, key: Key) -> Option<Pending> {
        self.domain.pending(key)
    }
    #[must_use]
    pub fn link(&self) -> LinkView<'_> {
        LinkView {
            state: self.domain.link(),
            unreadable: self.domain.unreadable(),
            streams_full: self.domain.streams_full(),
            watches_full: self.domain.watches_full(),
            party: self.domain.party(),
            not_served: self.domain.not_served(),
            sign_out_lost: self.sign_out_lost,
            not_signed_in: self.not_signed_in,
            credential: match &self.link {
                Some(link) => link.credential(),
                None => None,
            },
            expires: match &self.link {
                Some(link) => link.expires(),
                None => None,
            },
        }
    }
    fn unlinked_event(&mut self, env: &Env<DomainLimits>, event: Event, out: &mut Queue<UnlinkedRequest>) {
        match event {
            Event::StartUnlinked { kept } => self.domain_step(env, domain::Event::Start { kept }),
            Event::Watch { handle, subject } => self.domain_step(env, domain::Event::Watch { handle, subject }),
            Event::Unwatch { handle } => self.domain_step(env, domain::Event::Unwatch { handle }),
            Event::Read { handle, read } => self.domain_step(env, domain::Event::Read { handle, read }),
            Event::Send { key, name, request } => self.domain_step(env, domain::Event::Send { key, name, request }),
            Event::Carry { key, name, request, value } => {
                self.domain_step(env, domain::Event::Carry { key, name, request, value });
            }
            Event::Up { up } => {
                match &up {
                    Up::SignedIn | Up::SignedOut => {
                        self.signing_out = false;
                        self.sign_out_lost = false;
                    }
                    Up::StreamsFull { .. }
                    | Up::Unreadable
                    | Up::Served { .. }
                    | Up::FamilyServed { .. }
                    | Up::Answer { .. }
                    | Up::NotKept { .. }
                    | Up::ReadDone { .. }
                    | Up::Watched { .. }
                    | Up::FamilyDelivered { .. }
                    | Up::WatchEnded { .. } => {}
                }
                self.up(env, up);
            }
            Event::SignIn { with } => {
                assert!(with == With::Transport, "unlinked sign-in belongs to the transport");
                self.up(env, Up::SignedIn);
            }
            Event::AskApproval => out.push(UnlinkedRequest::ApprovalAct(ApprovalAct::Ask)),
            Event::ReadApproval { code } => out.push(UnlinkedRequest::ApprovalAct(ApprovalAct::Read { code })),
            Event::DecideApproval { code, approve } => {
                out.push(UnlinkedRequest::ApprovalAct(ApprovalAct::Decide { code, approve }));
            }
            Event::ApprovalAnswer { answer } => out.push(UnlinkedRequest::Approval(answer)),
            Event::Authorize { .. } => out.push(UnlinkedRequest::Authorized(crate::Authorized::Lost)),
            Event::SignOut => {
                self.signing_out = true;
                self.sign_out_lost = false;
                out.push(UnlinkedRequest::SignOut);
            }
            Event::SignOutLost => self.sign_out_failed(),
            Event::Stored | Event::NotStored => {}
            Event::Start { .. }
            | Event::Reply { .. }
            | Event::Lost { .. }
            | Event::Streamed { .. }
            | Event::StreamGone => {
                assert!(self.link.is_some(), "wire events require a linked client");
            }
        }
    }
    fn link_step(&mut self, now: Time, event: link::Event) {
        self.link.as_mut().expect("linked client").step(now, event, &mut self.link_out);
    }
    fn signing(&mut self, env: &Env<DomainLimits>, signing: link::Signing) {
        if self.signing_link {
            let why = match &signing {
                link::Signing::Approval(taking) => Some(NotSignedIn::Approval(*taking)),
                link::Signing::LinkUsed => Some(NotSignedIn::LinkUsed),
                link::Signing::LinkExpired => Some(NotSignedIn::LinkExpired),
                link::Signing::LinkUnknown => Some(NotSignedIn::LinkUnknown),
                link::Signing::Again => Some(NotSignedIn::NotReady),
                link::Signing::Busy => Some(NotSignedIn::Busy),
                link::Signing::Lost => match self.send_again {
                    Some(SendAgain::Approval { .. }) => Some(NotSignedIn::Busy),
                    Some(SendAgain::Link { .. } | SendAgain::Service(_)) | None => None,
                },
                link::Signing::Full { sign_ins, earliest, link_until } => {
                    Some(NotSignedIn::Full { sign_ins: *sign_ins, earliest: *earliest, link_until: *link_until })
                }
                link::Signing::SignedIn { .. } | link::Signing::SignedOut | link::Signing::Refused => None,
            };
            self.not_signed_in(why);
        }
        self.signing_link = false;
        match signing {
            link::Signing::SignedIn { .. } => {
                self.not_signed_in(None);
                self.send_again = None;
                self.signing_until = None;
                self.signing_out = false;
                self.sign_out_lost = false;
                self.up(env, Up::SignedIn);
            }
            link::Signing::SignedOut => {
                self.send_again = None;
                self.signing_until = None;
                self.signing_out = false;
                self.sign_out_lost = false;
                self.up(env, Up::SignedOut);
            }
            link::Signing::Again | link::Signing::Busy | link::Signing::Lost
                if !self.signing_out && self.send_again.is_some() =>
            {
                let span = match self.signing_span {
                    Some(span) => core::cmp::min(span.saturating_mul(2), env.limits.backoff.most),
                    None => env.limits.backoff.first,
                };
                self.signing_span = Some(span);
                let nanos = span.as_nanos();
                self.signing_until =
                    Some(env.now.saturating_add(Duration::from_nanos(self.rng.between(nanos.div_euclid(2), nanos))));
            }
            link::Signing::Approval(Taking::Waiting) if !self.signing_out && self.send_again.is_some() => {
                self.signing_span = None;
                self.signing_until = Some(env.now.saturating_add(APPROVAL_AGAIN));
            }
            link::Signing::Approval(_)
            | link::Signing::Busy
            | link::Signing::Full { .. }
            | link::Signing::Again
            | link::Signing::Lost
            | link::Signing::LinkUsed
            | link::Signing::LinkExpired
            | link::Signing::LinkUnknown
            | link::Signing::Refused => {
                self.sign_out_failed();
                self.send_again = None;
                self.signing_until = None;
            }
        }
    }
    fn not_signed_in(&mut self, why: Option<NotSignedIn>) {
        if self.not_signed_in != why {
            self.not_signed_in = why;
            self.domain_out.push(domain::Request::Told { told: Told::Link { link: self.domain.link() } });
        }
    }
    fn sign_out_failed(&mut self) {
        self.sign_out_lost = self.signing_out;
        self.signing_out = false;
        if self.sign_out_lost {
            self.domain_out.push(domain::Request::Told { told: Told::Link { link: self.domain.link() } });
        }
    }
    fn store_answer(&mut self, env: &Env<DomainLimits>, stored: bool) {
        if let Some(write) = self.store.answer() {
            while let Some(key) = self.store.take(write) {
                if !stored {
                    self.up(env, Up::NotKept { key });
                }
            }
        }
        if !stored {
            self.link_step(env.now, link::Event::NotStored);
        } else if !self.store.pending() {
            self.link_step(env.now, link::Event::Stored);
        }
        if self.store.pending() {
            self.link.as_mut().expect("linked client").wait_for_store();
        }
    }
    fn domain_step(&mut self, env: &Env<DomainLimits>, event: domain::Event) {
        domain::step(&mut self.domain, env, event, &mut self.domain_out);
    }
    fn up(&mut self, env: &Env<DomainLimits>, up: Up) {
        self.domain_step(env, domain::Event::Up { up });
    }
    fn prune_reads(&mut self) {
        for _ in 0..self.reads.len() {
            let read = self.reads.pop().expect("counted deferred read");
            if self.domain.has_read(read.read) {
                self.reads.push(read);
            }
        }
    }
    fn defer_read(&mut self, now: Time, read: Token, what: Readable) {
        self.prune_reads();
        if self.reads.is_empty() && self.link.as_ref().expect("linked read").has_exchange_room() {
            self.link_step(now, link::Event::Read { read, what });
            return;
        }
        // Validate before retaining cursor/filter/family payloads within their wire envelope.
        // Invalid reads keep their existing terminal; only local exchange pressure waits.
        if link::translate::reads::owned(what.clone(), self.link_limits.wire()).is_err() {
            self.link_step(now, link::Event::Read { read, what });
            return;
        }
        self.reads.push(DeferredRead { read, what });
    }
    fn drain_unlinked(&mut self, out: &mut Queue<UnlinkedRequest>) {
        while let Some(request) = self.domain_out.pop() {
            match request {
                domain::Request::Told { told } => out.push(UnlinkedRequest::Told { told }),
                domain::Request::Down { down } => out.push(UnlinkedRequest::Down { down }),
            }
        }
    }
    fn drain(&mut self, env: &Env<DomainLimits>, out: &mut Queue<Request>) {
        self.prune_reads();
        // Link and domain outputs answer each other within this step; the routed event's bounded
        // fan-out ends this settle (client_settled below).
        loop {
            // link_requests_dispatched: route the link's consequences before the next domain request.
            if let Some(request) = self.link_out.pop() {
                self.route_link(env, request, out);
            } else if let Some(request) = self.domain_out.pop() {
                match request {
                    domain::Request::Told { told } => out.push(Request::Told { told }),
                    domain::Request::Down { down: Down::Read { read, what } } => {
                        if self.domain.link() != Link::OutOfDate && self.domain.has_read(read) {
                            self.defer_read(env.now, read, what);
                        }
                    }
                    domain::Request::Down { down } => {
                        let local = match &down {
                            Down::Save { .. } | Down::Erase { .. } => true,
                            Down::Send { .. }
                            | Down::Read { .. }
                            | Down::Open { .. }
                            | Down::Continue { .. }
                            | Down::Close { .. } => false,
                        };
                        if self.domain.link() != Link::OutOfDate || local {
                            match &down {
                                Down::Save { record } => self.store.save(record.key),
                                Down::Erase { key } => self.store.erase(*key),
                                Down::Send { .. }
                                | Down::Open { .. }
                                | Down::Continue { .. }
                                | Down::Close { .. }
                                | Down::Read { .. } => {}
                            }
                            self.link_step(env.now, down_event(down));
                        }
                    }
                }
            } else if !self.reads.is_empty() {
                let token = self.reads.iter().next().expect("deferred read").read;
                let refusal = self.domain.read_refusal(token);
                if self.domain.has_read(token)
                    && refusal.is_none()
                    && !self.link.as_ref().expect("linked deferred read").has_exchange_room()
                {
                    break;
                }
                let read = self.reads.pop().expect("deferred read");
                if !self.domain.has_read(read.read) {
                    continue;
                }
                match refusal {
                    Some(result) => self.up(env, Up::ReadDone { read: read.read, result }),
                    None => self.link_step(env.now, link::Event::Read { read: read.read, what: read.what }),
                }
            } else {
                break;
            }
        }
        self.prune_reads();
        assert!(
            self.link_out.is_empty() && self.domain_out.is_empty(),
            "client_settled: link and domain outputs empty; deferred reads may wait on exchange credit"
        );
        if !self.saves.is_empty() {
            self.store.write();
            let mut records = List::with_capacity(self.saves.len());
            while let Some(record) = self.saves.pop() {
                records.push(record).expect("gathered saves fit");
            }
            out.push(Request::Save { records });
        }
        if !self.erases.is_empty() {
            self.store.write();
            let mut records = List::with_capacity(self.erases.len());
            while let Some(key) = self.erases.pop() {
                records.push(key).expect("gathered erases fit");
            }
            out.push(Request::Erase { records });
        }
    }
    fn route_link(&mut self, env: &Env<DomainLimits>, request: link::Request, out: &mut Queue<Request>) {
        match request {
            link::Request::Exchange { exchange, credential, message } => {
                if self.domain.link() != Link::OutOfDate {
                    out.push(Request::Exchange { exchange, credential, message });
                }
            }
            link::Request::OpenStream { credential } => {
                if self.domain.link() != Link::OutOfDate {
                    out.push(Request::OpenStream { credential });
                }
            }
            link::Request::CloseStream => out.push(Request::CloseStream),
            link::Request::Save { record } => self.saves.push(record),
            link::Request::Erase { key } => self.erases.push(key),
            link::Request::Answer { attempt, envelope, answer, why } => {
                self.up(env, Up::Answer { attempt, envelope, answer, why });
            }
            link::Request::NotKept { attempt } => {
                self.up(
                    env,
                    Up::Answer { attempt, envelope: skein_lib::RequestEnvelope::Lost, answer: None, why: None },
                );
            }
            link::Request::StreamsFull { full } => self.up(env, Up::StreamsFull { full }),
            link::Request::Watch { watch, event } => self.up(env, Up::Watched { watch, event }),
            link::Request::FamilyDelivery { watch, record } => self.up(env, Up::FamilyDelivered { watch, record }),
            link::Request::Ended { watch, end } => self.up(env, Up::WatchEnded { watch, end }),
            link::Request::Read { read, result } => self.up(env, Up::ReadDone { read, result }),
            link::Request::Hello { family: None, versions } => {
                let served = match versions {
                    Some(range) => link::serves(range),
                    None => false,
                };
                self.up(env, Up::Served { served, versions });
            }
            link::Request::Hello { family: Some(family), versions } => {
                self.up(env, Up::FamilyServed { family, versions });
            }
            link::Request::Approval(answer) => out.push(Request::Approval(answer)),
            link::Request::Authorized(answer) => out.push(Request::Authorized(answer)),
            link::Request::Signing { signing } => self.signing(env, signing),
            link::Request::Kept { record: link::Saved { record, .. } } => {
                let mut kept = List::with_capacity(1);
                kept.push(record).expect("one restored record");
                self.domain.restore_record(env, kept, &mut self.domain_out);
            }
            link::Request::Dropped => out.push(Request::Told { told: Told::Dropped { count: 1 } }),
            link::Request::ServiceCredential { party, credential } => {
                out.push(Request::ServiceCredential { party, credential });
            }
            link::Request::Fact { fact } => match fact {
                link::Fact::UndecodableEvent | link::Fact::UndecodableWatchAnswer => self.up(env, Up::Unreadable),
                link::Fact::UndecodableAnswer
                | link::Fact::UndecodableDocument
                | link::Fact::StreamDropped
                | link::Fact::UndecodableStore => {}
            },
        }
    }
}
fn down_event(down: Down) -> link::Event {
    match down {
        Down::Save { record } => link::Event::Save { record: link::Saved { version: domain::TABLE_VERSION, record } },
        Down::Erase { key } => link::Event::Erase { key },
        Down::Send { attempt, key, request, value } => link::Event::Send { attempt, key, ask: request, value },
        Down::Open { watch, subject } => link::Event::Open { watch, subject },
        Down::Continue { watch, subject } => link::Event::Continue { watch, subject },
        Down::Close { watch } => link::Event::Close { watch },
        Down::Read { read, what } => link::Event::Read { read, what },
    }
}

/// The interval between approval takes answered waiting (jig/docs/design/client/domain/client/README.md, 9).
pub const APPROVAL_AGAIN: Duration = Duration::from_secs(1);

/// What an unlinked client's host carries for approval (jig/docs/design/client/domain/client/README.md, 4.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApprovalAct {
    Ask,
    Read { code: ApprovalCode },
    Decide { code: ApprovalCode, approve: bool },
}

// Replies can carry credentials (jig/docs/design/protocol/client/sign-in.md).
impl core::fmt::Debug for Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Start { kept } => f.debug_struct("Start").field("kept", kept).finish(),
            Self::StartUnlinked { kept } => f.debug_struct("StartUnlinked").field("kept", kept).finish(),
            Self::SignIn { with } => f.debug_struct("SignIn").field("with", with).finish(),
            Self::SignOut => f.write_str("SignOut"),
            Self::Authorize { query } => f.debug_struct("Authorize").field("query", query).finish(),
            Self::AskApproval => f.write_str("AskApproval"),
            Self::ReadApproval { code } => f.debug_struct("ReadApproval").field("code", code).finish(),
            Self::DecideApproval { code, approve } => {
                f.debug_struct("DecideApproval").field("code", code).field("approve", approve).finish()
            }
            Self::ApprovalAnswer { answer } => f.debug_struct("ApprovalAnswer").field("answer", answer).finish(),
            Self::SignOutLost => f.write_str("SignOutLost"),
            Self::Watch { handle, subject } => {
                f.debug_struct("Watch").field("handle", handle).field("subject", subject).finish()
            }
            Self::Unwatch { handle } => f.debug_struct("Unwatch").field("handle", handle).finish(),
            Self::Read { handle, read } => f.debug_struct("Read").field("handle", handle).field("read", read).finish(),
            Self::Send { key, name, request } => {
                f.debug_struct("Send").field("key", key).field("name", name).field("request", request).finish()
            }
            Self::Carry { key, name, request, value } => f
                .debug_struct("Carry")
                .field("key", key)
                .field("name", name)
                .field("request", request)
                .field("value", value)
                .finish(),
            Self::Reply { exchange, message } => {
                f.debug_struct("Reply").field("exchange", exchange).field("body_len", &message.len()).finish()
            }
            Self::Lost { exchange } => f.debug_struct("Lost").field("exchange", exchange).finish(),
            Self::Streamed { message } => f.debug_struct("Streamed").field("message", message).finish(),
            Self::StreamGone => f.write_str("StreamGone"),
            Self::Stored => f.write_str("Stored"),
            Self::NotStored => f.write_str("NotStored"),
            Self::Up { up } => f.debug_struct("Up").field("up", up).finish(),
        }
    }
}
