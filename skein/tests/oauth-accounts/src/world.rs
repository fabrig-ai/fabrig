//! Accounts and independent fake issuer processes, joined through shared world
//! scheduling, replay, heap and io contracts (oauth/accounts.md, section 8).

use skein_fake_oauth as fake;
use skein_fake_peers::{Transport, oauth as peer};
use skein_io::{self as io, kernel};
use skein_lib::{Duration, Env, List, Queue, Time, Wall, bytes};
use skein_oauth as oauth;
use skein_oauth_accounts as accounts;
use skein_world::domain::Ledger;
use skein_world::stream::Trace;
use skein_world::{Delegate, Host, Memory, Outcome, Referee, World};
use std::net::{Ipv4Addr, SocketAddr};

const ROOM: u32 = 64;
const AUTHORIZE: &[u8] = b"http://127.0.0.1:31000/authorize";
const TOKEN: &[u8] = b"http://127.0.0.1:31000/token";
const REDIRECT: &[u8] = b"http://localhost:31234/callback";

/// The owner behaviour the referee expects from this run.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Story {
    /// A private record's revoked refresh is refused before its replacement loads.
    SignInAgain,
    /// An access-only source hands in a new generation when its old one is expiring.
    HandedIn,
    /// A held valid grant refreshes at its lead, then the owner closes.
    Lead,
    /// A grant within the lead waits for a rotating refresh and its keep.
    Rotate,
    /// A rate limit retries only after its prior connection settles.
    Retry,
    /// The issuer omits a replacement refresh token; the old one is kept.
    Omit,
    /// The keeper declines the candidate; the previous valid generation serves.
    NotKept,
    /// A provider rejection starts one refresh.
    Rejected,
    /// A second rejection of the still-held generation ends the grant.
    Repeated,
    /// A released account starts no refresh at its lead.
    Released,
    /// The owner closes before the exchange connects, then it drains.
    CloseConnecting,
    /// The owner closes once a request has begun uploading, then it drains.
    CloseSending,
    /// A close with an owner keep still pending waits for its terminal.
    CloseKeeping,
    /// An abort after a close cancels the admitted exchange.
    Abort,
}

/// The component's tiny capacities and shipped deadlines.
#[must_use]
pub fn limits() -> accounts::Limits {
    let client = oauth::ClientLimits {
        document: oauth::Limits { document_bytes: 1024, ..oauth::ClientLimits::TEST.document },
        nonce_bytes: 64,
        metadata_bytes: 1024,
        key_set_bytes: 2048,
        keys: 4,
        backoff_base: Duration::from_secs(1),
        backoff_ceiling: Duration::from_secs(4),
        max_attempts: 2,
        ..oauth::ClientLimits::TEST
    };
    accounts::Limits {
        accounts: 2,
        file_stall: Duration::from_secs(1),
        lock_wait: Duration::from_secs(1),
        exchanges: 2,
        listeners: 1,
        redirect_bytes: client.url_bytes,
        server: crate::streams::server_limits(&issuer_limits(&client)),
        refresh_lead: Duration::from_secs(10),
        client,
        http: skein_http::client::Limits { request: 2048, head: 2048, headers: 16, read: 256, send: 256 },
        tls: skein_tls::client::Limits { read: 2048, send: 2048, records: 131_072 },
        io: io::Limits { sockets: 4, sends: 4, backlog: 2, retry: Duration::from_millis(1), ..io::Limits::TEST },
    }
}

pub(crate) fn issuer_limits(client: &oauth::ClientLimits) -> fake::Limits {
    fake::Limits {
        document: client.document,
        uri_bytes: client.uri_bytes,
        request_bytes: client.request_bytes,
        codes: 1,
        rotations: 2,
        plans: 2,
        carried: 0,
    }
}

pub(crate) fn registration(transport: Transport) -> oauth::Registration {
    oauth::Registration {
        authorization_url: bytes::copy_of(if transport == Transport::Tls {
            b"https://127.0.0.1:31000/authorize"
        } else {
            AUTHORIZE
        }),
        token_endpoint: bytes::copy_of(if transport == Transport::Tls {
            b"https://127.0.0.1:31000/token"
        } else {
            TOKEN
        }),
        client_id: bytes::copy_of(b"client"),
        redirect_uri: bytes::copy_of(REDIRECT),
        scope: bytes::copy_of(b"read"),
        wire: oauth::WireFormat::Form,
        client_secret: None,
        pkce_for_confidential: false,
        metadata_claim: None,
        terms: None,
    }
}

/// Content-free facts retained at the actual owner boundary.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Fact {
    /// The owner was asked to lock the shared record.
    Lock,
    /// The shared write released its owner lock.
    Unlock,
    /// The owner heard a usable generation.
    Granted(u64),
    /// The access-only generation reached its refresh lead once.
    Expiring,
    /// The candidate reached the owner's keeper.
    Keep(u64),
    /// The owner's hold or pending request failed.
    Failed(accounts::Failure),
    /// The record was forgotten, with the issuer's revocation outcome.
    SignedOut(bool),
    /// An account removal settled.
    Removed(u32),
    /// The component settled its owner close.
    Closed,
}

/// Contract referee shared by every tier; uses the kit's request ledger.
pub struct Contracts {
    grant: Ledger<u32, ()>,
    keeps: Ledger<u64, ()>,
    close: Ledger<u32, ()>,
    kept: Vec<u64>,
    last_generation: Option<u64>,
    closed: bool,
    locks: Ledger<u32, ()>,
}

impl Contracts {
    /// Opens an owner grant that must have one terminal.
    #[must_use]
    pub fn with_pending_grant() -> Self {
        let mut grant = Ledger::new("owner grants");
        grant.open(0, ());
        Self {
            grant,
            keeps: Ledger::new("candidate keeps"),
            close: Ledger::new("component closes"),
            kept: vec![0],
            last_generation: None,
            closed: false,
            locks: Ledger::new("owner locks"),
        }
    }
    /// Checks a generation before accepting its grant or held notice.
    pub fn granted(&mut self, generation: u64) {
        assert!(!self.closed, "Closed is the last owner event");
        assert!(self.kept.contains(&generation), "nothing lent before its keep terminal");
        if self.grant.contains(0) {
            self.grant.end(0);
        } else {
            assert!(self.last_generation.is_none_or(|prior| generation > prior), "held notices name a new generation");
        }
        self.last_generation = Some(generation);
    }
    /// Opens the candidate's required durability terminal.
    pub fn keep(&mut self, generation: u64) {
        assert!(!self.closed, "Closed is the last owner event");
        self.keeps.open(generation, ());
    }
    /// Delivers exactly the pending keeper terminal.
    pub fn kept(&mut self, generation: u64, kept: bool) {
        self.keeps.end(generation);
        if kept {
            self.kept.push(generation);
        }
    }
    /// Ends a pending grant or the owner's existing hold.
    pub fn failed(&mut self) {
        assert!(!self.closed, "Closed is the last owner event");
        if self.grant.contains(0) {
            self.grant.end(0);
        } else {
            assert!(self.last_generation.take().is_some(), "only a pending or held grant fails");
        }
    }
    /// Opens the component's close obligation once, even when upgraded to abort.
    pub fn closing(&mut self) {
        if !self.close.contains(0) {
            self.close.open(0, ());
        }
    }
    /// Opens the next owner grant after its predecessor ended.
    pub fn requesting(&mut self) {
        self.grant.open(0, ());
    }
    /// Records a shared record read under the owner's lock.
    pub fn read(&mut self, generation: u64) {
        self.kept.push(generation);
    }
    /// Each held owner lock requires one release before Closed.
    pub fn locked(&mut self, account: u32) {
        assert!(!self.closed, "Closed is the last owner event");
        self.locks.open(account, ());
    }
    /// Releases exactly one lock taken by a Held answer.
    pub fn unlocked(&mut self, account: u32) {
        assert!(!self.closed, "Closed is the last owner event");
        self.locks.end(account);
    }
    /// Settles the owner's close only once, after keeper obligations.
    pub fn closed(&mut self) {
        assert!(!self.closed, "one Closed");
        self.close.end(0);
        self.keeps.assert_settled();
        self.locks.assert_settled();
        self.grant.assert_settled();
        self.closed = true;
    }
}

/// The service's owner adapter, running the same component at each tier.
pub struct Client {
    pub(crate) accounts: Option<crate::accounts::Owner>,
    pub(crate) shared: Option<crate::shared::Owner>,
    pub(crate) sign_in: Option<crate::sign_in::Owner>,
    pub(crate) sign_out: Option<crate::sign_out::Owner>,
    pub(crate) terms: Option<oauth::Terms>,
    io: io::Io,
    pub(crate) component: Option<accounts::Component>,
    pub(crate) limits: accounts::Limits,
    events: Queue<io::Event>,
    pub(crate) requests: Queue<io::Request>,
    pub(crate) files: Queue<accounts::FileLower>,
    pub(crate) file_driver: Option<skein_world::files::Driver>,
    file_keep: Option<(skein_lib::Token, oauth::SavedToken)>,
    pub(crate) stop_at_store: Option<bool>,
    pub(crate) stop_at_load: Option<bool>,
    pub(crate) retry_load: Option<()>,
    pub(crate) above: Queue<accounts::Event>,
    completions: Queue<kernel::Complete>,
    submissions: Queue<kernel::Submit>,
    pub(crate) address: Option<SocketAddr>,
    pub(crate) transport: Transport,
    story: Story,
    pub(crate) facts: Vec<Fact>,
    records: Vec<oauth::SavedToken>,
    pub(crate) contracts: Contracts,
    pub(crate) closing: bool,
    pub(crate) closed: bool,
    pub(crate) recovered: Option<u64>,
    rejected: bool,
    pub(crate) started: Time,
}

impl Client {
    pub(crate) fn new(story: Story, transport: Transport) -> Client {
        Self::with_limits(story, transport, &limits(), ROOM)
    }
    pub(crate) fn with_limits(story: Story, transport: Transport, limits: &accounts::Limits, room: u32) -> Client {
        Client {
            accounts: None,
            shared: None,
            sign_in: None,
            sign_out: None,
            terms: None,
            io: io::Io::new(&limits.io),
            component: None,
            limits: *limits,
            events: Queue::uncapped(),
            requests: Queue::uncapped(),
            files: Queue::uncapped(),
            file_driver: None,
            file_keep: None,
            stop_at_store: None,
            stop_at_load: None,
            retry_load: None,
            above: Queue::uncapped(),
            completions: Queue::with_capacity(room),
            submissions: Queue::with_capacity(room),
            address: None,
            transport,
            story,
            facts: Vec::with_capacity(16),
            records: Vec::with_capacity(4),
            contracts: Contracts::with_pending_grant(),
            closing: false,
            closed: false,
            recovered: None,
            rejected: false,
            started: Time::ZERO,
        }
    }
    pub(crate) fn private(root: kernel::Fd, story: Story, terms: Option<oauth::Terms>) -> Client {
        let mut client = Client::new(story, Transport::Plaintext);
        client.terms = terms;
        client.file_driver = Some(skein_world::files::Driver::new(
            root,
            4,
            client.limits.client.document.record_bytes,
            client.limits.file_stall,
            1234,
        ));
        client
    }
    pub(crate) fn recovery(root: kernel::Fd) -> Client {
        let mut client = Client::private(root, Story::NotKept, None);
        client.limits.refresh_lead = Duration::ZERO;
        client
    }
    pub(crate) fn refresh_at_start(&mut self) {
        self.limits.refresh_lead = Duration::from_secs(10);
    }
    pub(crate) fn unstarted(&self) -> bool {
        self.component.is_none()
    }
    #[expect(clippy::too_many_lines, reason = "dispatches sub-owners before configuring the refresh story")]
    pub(crate) fn start(&mut self, env: &Env<accounts::Limits>) {
        if self.sign_in.is_some() {
            crate::sign_in::start(self, env);
            return;
        }
        if self.shared.is_some() {
            crate::shared::start(self, env);
            return;
        }
        if self.sign_out.is_some() {
            crate::sign_out::start(self, env);
            return;
        }
        if self.accounts.is_some() {
            crate::accounts::start(self, env);
            return;
        }
        let mut configured = List::with_capacity(1);
        let seconds = if matches!(
            self.story,
            Story::Lead | Story::Rejected | Story::Repeated | Story::Released | Story::HandedIn
        ) {
            30
        } else {
            5
        };
        let mut record = Some(oauth::SavedToken {
            key: 0,
            generation: 0,
            access_token: skein_lib::Secret::new(bytes::copy_of(b"old")),
            refresh_token: (self.story != Story::HandedIn).then(|| bytes::copy_of(b"seed")).map(skein_lib::Secret::new),
            metadata: None,
            expires_at: Wall::from_nanos(
                env.wall.as_nanos().checked_add(Duration::from_secs(seconds).as_nanos()).expect("wall lifetime"),
            ),
            issued_client_id: None,
            granted_scopes: None,
            subject: None,
            id_token: None,
            earliest_refresh_at: None,
        });
        assert!(
            configured
                .push(if self.story == Story::HandedIn {
                    accounts::Account::HandedIn
                } else {
                    accounts::Account::SignIn {
                        origins: List::with_capacity(0),
                        shared: false,
                        registration: {
                            let mut registration = registration(self.transport);
                            registration.terms.clone_from(&self.terms);
                            registration
                        },
                        endpoint: accounts::Endpoint {
                            address: self.address.expect("referee supplied peer address"),
                            transport: match self.transport {
                                Transport::Plaintext => accounts::Transport::Plaintext,
                                Transport::Tls => accounts::Transport::Tls {
                                    server_name: skein_tls_kit::pki::name(),
                                    trust: skein_tls_kit::pki::client(&[]),
                                },
                            },
                        },
                        keeper: match &self.file_driver {
                            Some(driver) => accounts::Keeper::Private {
                                root: driver.root(),
                                directory: bytes::copy_of(b"secret"),
                                file: bytes::copy_of(b"record"),
                            },
                            None => accounts::Keeper::Owner { kept: record.take() },
                        },
                    }
                })
                .is_ok()
        );
        let mut component = accounts::Component::new(
            configured,
            &self.limits,
            skein_lib::CryptoRng::new(&skein_world::crypto_seed(17), 0).expect("32-byte world seed"),
            b"oauth-accounts-world",
        )
        .expect("configured accounts");
        if self.story == Story::HandedIn {
            let record = record.take().expect("handed-in source keeps its startup record");
            self.records.push(record.clone());
            component.down(
                env,
                accounts::Request::HandIn { account: 0, record },
                &mut self.above,
                &mut self.requests,
                &mut self.files,
            );
        }
        component.down(
            env,
            accounts::Request::Grant { account: 0 },
            &mut self.above,
            &mut self.requests,
            &mut self.files,
        );
        self.component = Some(component);
        self.started = env.now;
    }
    pub(crate) fn ask(&mut self, env: &Env<accounts::Limits>, request: accounts::Request) {
        self.component.as_mut().expect("started component").down(
            env,
            request,
            &mut self.above,
            &mut self.requests,
            &mut self.files,
        );
    }
    pub(crate) fn close(&mut self, env: &Env<accounts::Limits>) {
        if !self.closing {
            self.closing = true;
            self.contracts.closing();
            self.ask(env, accounts::Request::Close);
        }
    }
    fn observe_story(&mut self, env: &Env<accounts::Limits>) -> bool {
        if self.sign_in.is_some() {
            crate::sign_in::observe(self, env);
            return true;
        }
        if self.shared.is_some() {
            crate::shared::observe(self, env);
            return true;
        }
        if self.sign_out.is_some() {
            crate::sign_out::observe(self, env);
            return true;
        }
        if self.accounts.is_some() {
            crate::accounts::observe(self, env);
            return true;
        }
        false
    }

    pub(crate) fn observe(&mut self, env: &Env<accounts::Limits>) {
        if self.observe_story(env) {
            return;
        }
        if let Some(event) = self.above.pop() {
            assert!(!self.closed, "Closed is once and last");
            match event {
                accounts::Event::Granted { generation, token, valid, .. } => {
                    self.contracts.granted(generation);
                    assert!(valid > Duration::ZERO);
                    assert_eq!(token.expose(), if generation == 0 { b"old" } else { b"new" });
                    self.facts.push(Fact::Granted(generation));
                    if generation == 0 && matches!(self.story, Story::Rejected | Story::Repeated) && !self.rejected {
                        self.rejected = true;
                        self.ask(env, accounts::Request::Rejected { account: 0, generation });
                        if self.story == Story::Repeated {
                            self.ask(env, accounts::Request::Rejected { account: 0, generation });
                            self.close(env);
                        }
                    } else if self.story == Story::Released && generation == 0 {
                        self.ask(env, accounts::Request::Release { account: 0 });
                    } else if generation == 1 || self.story == Story::NotKept {
                        self.close(env);
                    }
                }
                accounts::Event::Keep { account, record } => {
                    self.contracts.keep(record.generation);
                    self.facts.push(Fact::Keep(record.generation));
                    assert_eq!(
                        record.refresh_token.as_ref().map(skein_lib::Secret::expose),
                        Some(if self.story == Story::Omit { b"seed".as_slice() } else { b"next".as_slice() })
                    );
                    let kept = self.story != Story::NotKept;
                    if self.story == Story::CloseKeeping {
                        self.close(env);
                        assert!(!self.closed);
                    }
                    self.contracts.kept(record.generation, kept);
                    self.ask(
                        env,
                        accounts::Request::Kept {
                            account,
                            generation: record.generation,
                            keeping: if kept { accounts::Keeping::Kept } else { accounts::Keeping::NotKept },
                        },
                    );
                    self.records.push(record);
                }
                accounts::Event::Failed { failure, .. } => {
                    self.contracts.failed();
                    self.facts.push(Fact::Failed(failure));
                    if (matches!(failure, accounts::Failure::Unloaded { .. })
                        || failure == accounts::Failure::Exchange(oauth::Failure::Refused))
                        && self.retry_load.take().is_some()
                    {
                        self.contracts.grant.open(0, ());
                        self.ask(env, accounts::Request::Grant { account: 0 });
                    } else {
                        self.close(env);
                    }
                }
                accounts::Event::Closed => {
                    self.contracts.closed();
                    self.facts.push(Fact::Closed);
                    self.closed = true;
                    if let Some(driver) = &mut self.file_driver {
                        driver.close_root(env.now);
                    }
                }
                accounts::Event::Expiring { account, generation } => {
                    assert_eq!(self.story, Story::HandedIn);
                    assert_eq!(generation, 0);
                    self.facts.push(Fact::Expiring);
                    self.contracts.kept.push(1);
                    let mut record = self.records.pop().expect("handed-in access-only record");
                    record.generation = 1;
                    record.access_token = skein_lib::Secret::new(bytes::copy_of(b"new"));
                    record.expires_at = Wall::from_nanos(
                        env.wall.as_nanos().checked_add(Duration::from_secs(30).as_nanos()).expect("wall lifetime"),
                    );
                    self.ask(env, accounts::Request::HandIn { account, record });
                }
                accounts::Event::Removed { .. }
                | accounts::Event::Refused { .. }
                | accounts::Event::Visit { .. }
                | accounts::Event::Lock { .. }
                | accounts::Event::Unlock { .. }
                | accounts::Event::SignedOut { .. }
                | accounts::Event::Taken { .. }
                | accounts::Event::SignedIn { .. }
                | accounts::Event::Refreshed { .. } => {
                    panic!("positive story has no such event")
                }
            }
        }
    }
    fn private_io(&mut self, env: &Env<accounts::Limits>) {
        if self.file_driver.is_none() {
            return;
        }
        while let Some(lower) = self.files.pop() {
            match lower {
                accounts::FileLower::Request { request, deadline } => {
                    crate::accounts::before_file(self, env, &request);
                    if let io::file::Request::Store { owner, bytes, .. } = &request {
                        let record =
                            oauth::decode_record(bytes, &env.limits.client.document).expect("candidate record");
                        if let Some(owner) = &mut self.sign_in {
                            crate::sign_in::keeping(owner, record.generation);
                        } else {
                            self.contracts.keep(record.generation);
                            self.facts.push(Fact::Keep(record.generation));
                        }
                        assert!(self.file_keep.is_none());
                        self.file_keep = Some((*owner, record));
                        if let Some(abort) = self.stop_at_store.take() {
                            self.close(env);
                            if abort {
                                self.ask(env, accounts::Request::Abort);
                            }
                        }
                    }
                    if matches!(request, io::file::Request::Load { .. })
                        && let Some(abort) = self.stop_at_load.take()
                    {
                        self.close(env);
                        if abort {
                            self.ask(env, accounts::Request::Abort);
                        }
                    }
                    self.file_driver.as_mut().expect("file driver").request(request, deadline);
                }
                accounts::FileLower::Remove { owner, root, name, directory, deadline } => {
                    let driver = self.file_driver.as_mut().expect("file driver");
                    let root = driver.descriptor(root).expect("private root");
                    driver.request(io::file::Request::Remove { owner, root, name, directory }, deadline);
                }
                accounts::FileLower::Cancel { owner } => self.file_driver.as_mut().expect("file driver").cancel(owner),
            }
        }
        self.file_driver.as_mut().expect("file driver").progress(env.now);
        while let Some(event) = self.file_driver.as_mut().expect("file driver").take_event() {
            if event.owner() == skein_lib::Token::new(u64::MAX) {
                continue;
            }
            if self.sign_in.is_some() {
                crate::sign_in::file_event(self, env.now, &event);
            }
            if self.sign_in.is_none()
                && let io::file::Event::Loaded { bytes, .. } = &event
                && let Ok(record) = oauth::decode_record(bytes, &env.limits.client.document)
            {
                if !self.contracts.kept.contains(&record.generation) {
                    self.contracts.kept.push(record.generation);
                }
                self.recovered = Some(record.generation);
            }
            if self.file_keep.as_ref().is_some_and(|(owner, _)| *owner == event.owner()) {
                let (_, record) = self.file_keep.take().expect("matching keep");
                let yes = matches!(event, io::file::Event::Stored { .. });
                if let Some(owner) = &mut self.sign_in {
                    crate::sign_in::kept(owner, record, yes);
                } else {
                    self.contracts.kept(record.generation, yes);
                    self.records.push(record);
                }
            }
            self.component.as_mut().expect("started component").filed(
                env,
                event,
                &mut self.above,
                &mut self.requests,
                &mut self.files,
            );
        }
        while let Some(submit) = self.file_driver.as_mut().expect("file driver").take_submit() {
            self.submissions.push(submit);
        }
    }
}

impl Host for Client {
    fn iterate(&mut self, now: Time, wall: Wall) {
        let env = Env { now, wall, limits: self.limits };
        if self.sign_in.is_some() {
            crate::sign_in::restart(self, &env);
        }
        if self.component.is_none() && self.address.is_some() {
            self.start(&env);
        }
        if self.sign_in.is_some() {
            crate::sign_in::redirected(self, &env);
        }
        let io_env = Env { now, wall, limits: self.limits.io };
        self.private_io(&env);
        // The file driver shares the kernel's completions: it takes its own (they submit only into its
        // queue, which `private_io` drains), and io's stage takes the rest in their order.
        for _ in 0..self.completions.len() {
            let Some(complete) = self.completions.pop() else { break };
            if skein_world::files::Driver::owns(complete.op) {
                self.file_driver.as_mut().expect("tagged file completion").up(complete);
            } else {
                self.completions.push(complete);
            }
        }
        io::stage::up(&mut self.io, &io_env, &mut self.completions, &mut self.events, &mut self.submissions, now);
        for _ in 0..ROOM {
            let Some(event) = self.events.pop() else { break };
            if self.sign_in.is_some() {
                crate::sign_in::accepted(self, &event);
            }
            self.component.as_mut().expect("io follows started component").up(
                &env,
                event,
                &mut self.above,
                &mut self.requests,
                &mut self.files,
            );
        }
        if let Some(component) = &mut self.component
            && (component.has_work() || component.next_deadline().is_some_and(|due| due <= now))
        {
            component.fire(&env, &mut self.above, &mut self.requests, &mut self.files);
        }
        {
            self.observe(&env);
            if self.story == Story::Released
                && !self.closing
                && now >= self.started.saturating_add(Duration::from_secs(40))
            {
                self.close(&env);
            }
        }
        for _ in 0..ROOM {
            if !self.io.takes() || self.submissions.room() < 2 {
                break;
            }
            let Some(request) = self.requests.pop() else { break };
            if !self.closing
                && ((matches!(request, io::Request::Connect { .. })
                    && matches!(self.story, Story::CloseConnecting | Story::Abort))
                    || (matches!(request, io::Request::Stream { down: skein_lib::stream::Down::Send(_), .. })
                        && self.story == Story::CloseSending))
            {
                self.close(&env);
                if self.story == Story::Abort {
                    self.ask(&env, accounts::Request::Abort);
                }
            }
            crate::accounts::before_socket(self, &env, &request);
            crate::shared::before_socket(self, &request);
            io::down(&mut self.io, &io_env, request, &mut self.submissions);
        }
        self.private_io(&env);
        self.io.reclaim();
        if let Some(component) = &mut self.component {
            component.reclaim();
        }
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        self.file_driver.as_ref().is_some_and(skein_world::files::Driver::has_work)
            || !self.files.is_empty()
            || self.io.is_ready()
            || !self.events.is_empty()
            || !self.requests.is_empty()
            || !self.above.is_empty()
            || !self.completions.is_empty()
            || self.sign_in.as_ref().is_some_and(crate::sign_in::work_pending)
            || (self.component.is_none() && self.address.is_some())
            || self.component.as_ref().is_some_and(|component| {
                component.has_work() || component.next_deadline().is_some_and(|due| due <= now)
            })
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        [
            self.io.next_deadline(),
            self.file_driver.as_ref().and_then(skein_world::files::Driver::next_deadline),
            self.sign_in.as_ref().and_then(crate::sign_in::next_deadline),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        [
            self.component.as_ref().and_then(accounts::Component::next_deadline),
            (self.story == Story::Released && self.component.is_some() && !self.closing)
                .then(|| self.started.saturating_add(Duration::from_secs(40))),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    fn is_empty(&self) -> bool {
        self.closed
            && self.io.is_empty()
            && self.file_driver.as_ref().is_none_or(skein_world::files::Driver::is_empty)
            && self.events.is_empty()
            && self.requests.is_empty()
            && self.files.is_empty()
            && self.above.is_empty()
            && self.completions.is_empty()
            && self.submissions.is_empty()
    }
    fn operations(&self) -> u32 {
        io::operations(&self.limits.io).expect("io operations") + if self.file_driver.is_some() { 4 } else { 0 }
    }
}

/// The owner and independent peers of a component world.
pub enum Process {
    /// The owner-driven component with its own io loop.
    Client(Box<Client>),
    /// The independent scripted issuer, which stays live until the owner settles.
    Issuer(Box<peer::Peer>),
    /// The browser receives the authorization redirect and callback page.
    Browser(Box<crate::sign_in::Browser>),
}

impl Process {
    /// Returns the client process, when present.
    #[must_use]
    pub fn client(&self) -> Option<&Client> {
        match self {
            Self::Client(host) => Some(host),
            Self::Issuer(_) | Self::Browser(_) => None,
        }
    }
    /// Returns the client process, when present.
    #[must_use]
    pub fn client_mut(&mut self) -> Option<&mut Client> {
        match self {
            Self::Client(host) => Some(host),
            Self::Issuer(_) | Self::Browser(_) => None,
        }
    }
    /// Returns the issuer process, when present.
    #[must_use]
    pub fn issuer(&self) -> Option<&peer::Peer> {
        match self {
            Self::Issuer(host) => Some(host),
            Self::Client(_) | Self::Browser(_) => None,
        }
    }
    /// Returns the issuer process, when present.
    #[must_use]
    pub fn issuer_mut(&mut self) -> Option<&mut peer::Peer> {
        match self {
            Self::Issuer(host) => Some(host),
            Self::Client(_) | Self::Browser(_) => None,
        }
    }
    /// Returns the browser process, when present.
    #[must_use]
    pub fn browser(&self) -> Option<&crate::sign_in::Browser> {
        match self {
            Self::Browser(host) => Some(host),
            Self::Client(_) | Self::Issuer(_) => None,
        }
    }
}

impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Process::Client(host) => host.as_ref(),
            Process::Issuer(host) => host.as_ref(),
            Process::Browser(host) => host.as_ref(),
        }
    }

    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Process::Client(host) => host.as_mut(),
            Process::Issuer(host) => host.as_mut(),
            Process::Browser(host) => host.as_mut(),
        }
    }
}

struct Judge {
    wake: Option<Time>,
    deadline: Option<Time>,
    story: Story,
    shutdown: bool,
    passed: bool,
}
impl Referee<Process> for Judge {
    fn act(&mut self, now: Time, processes: &mut [Process]) {
        if self.deadline.is_none() {
            self.deadline = Some(now.saturating_add(Duration::from_secs(120)));
        }
        let address = processes.iter().find_map(|process| process.issuer().and_then(peer::Peer::address));
        let settled = processes.iter().any(|process| matches!(process, Process::Client(client) if client.is_empty()));
        for process in processes {
            match process {
                Process::Client(client) => client.address = address,
                Process::Issuer(peer) if settled && !self.shutdown => {
                    peer.shutdown();
                    self.shutdown = true;
                }
                Process::Issuer(_) | Process::Browser(_) => {}
            }
        }
    }
    fn watch(&mut self, now: Time, processes: &[Process]) -> Result<(), String> {
        let client = processes.iter().find_map(Process::client).expect("client");
        self.wake = ((client.component.is_none()
            && processes.iter().any(|process| process.issuer().is_some_and(|peer| peer.address().is_some())))
            || (client.is_empty() && !self.shutdown))
            .then_some(now);
        let posts = processes
            .iter()
            .filter_map(Process::issuer)
            .flat_map(peer::Peer::observations)
            .filter(|event| matches!(event, peer::Observation::Post { .. }))
            .count();
        if posts > if self.story == Story::Retry { 2 } else { 1 } {
            return Err(String::from("bounded refresh attempts per generation"));
        }
        if client.closed {
            if client.facts.last() != Some(&Fact::Closed) {
                return Err(format!(
                    "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                    client.facts.last(),
                    Some(&Fact::Closed)
                ));
            }
            match self.story {
                Story::HandedIn => {
                    if posts != 0 {
                        return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", posts, 0));
                    }
                    if client.facts != vec![Fact::Granted(0), Fact::Expiring, Fact::Granted(1), Fact::Closed] {
                        return Err(format!(
                            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                            client.facts,
                            vec![Fact::Granted(0), Fact::Expiring, Fact::Granted(1), Fact::Closed]
                        ));
                    }
                }
                Story::SignInAgain => {
                    if posts != 1 {
                        return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", posts, 1));
                    }
                }
                Story::Repeated => {
                    if !client.facts.contains(&Fact::Failed(accounts::Failure::Expired)) {
                        return Err(String::from(
                            "assertion failed: client.facts.contains(&Fact::Failed(accounts::Failure::Expired))",
                        ));
                    }
                    if posts != 0 {
                        return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", posts, 0));
                    }
                }
                Story::Released => {
                    if posts != 0 {
                        return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", posts, 0));
                    }
                    if client.facts != vec![Fact::Granted(0), Fact::Closed] {
                        return Err(format!(
                            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                            client.facts,
                            vec![Fact::Granted(0), Fact::Closed]
                        ));
                    }
                }
                Story::Abort => {
                    if !client.facts.contains(&Fact::Failed(accounts::Failure::Exchange(oauth::Failure::Cancelled))) {
                        return Err(String::from(
                            "assertion failed: client.facts.contains(&Fact::Failed(accounts::Failure::Exchange(oauth::Failure::Cancelled)))",
                        ));
                    }
                }
                Story::NotKept => {
                    if posts != 1 {
                        return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", posts, 1));
                    }
                    if client.facts != vec![Fact::Keep(1), Fact::Granted(0), Fact::Closed] {
                        return Err(format!(
                            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                            client.facts,
                            vec![Fact::Keep(1), Fact::Granted(0), Fact::Closed]
                        ));
                    }
                }
                Story::Retry => {
                    if posts != 2 {
                        return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", posts, 2));
                    }
                    if !client.facts.contains(&Fact::Granted(1)) {
                        return Err(format!("owner facts: {:?}", client.facts));
                    }
                    if !client.facts.contains(&Fact::Keep(1)) {
                        return Err(String::from("assertion failed: client.facts.contains(&Fact::Keep(1))"));
                    }
                }
                Story::Lead
                | Story::Rejected
                | Story::Rotate
                | Story::Omit
                | Story::CloseConnecting
                | Story::CloseSending
                | Story::CloseKeeping => {
                    if posts != 1 {
                        return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", posts, 1));
                    }
                    if !client.facts.contains(&Fact::Granted(1)) {
                        return Err(format!("owner facts: {:?}", client.facts));
                    }
                    if !client.facts.contains(&Fact::Keep(1)) {
                        return Err(String::from("assertion failed: client.facts.contains(&Fact::Keep(1))"));
                    }
                }
            }
        }
        self.passed = self.shutdown && processes.iter().all(Host::is_empty);
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        self.wake.or(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (self.deadline.is_some_and(|deadline| now >= deadline) && !self.passed)
            .then(|| "OAuth accounts world did not settle".to_owned())
    }
    fn passed(&self) -> bool {
        self.passed
    }
}

pub(crate) fn issuer(transport: Transport, story: Story, address: SocketAddr) -> Process {
    issuer_with_terms(transport, story, address, None, 200)
}
pub(crate) fn issuer_with_terms(
    transport: Transport,
    story: Story,
    address: SocketAddr,
    terms: Option<fake::Terms>,
    status: u16,
) -> Process {
    let profile = limits();
    let issuer_limits = issuer_limits(&profile.client);
    let token_endpoint = terms
        .as_ref()
        .and_then(|terms| terms.revocation_endpoint.clone())
        .unwrap_or_else(|| registration(transport).token_endpoint);
    let mut config = crate::streams::issuer_config(&registration(transport), terms);
    config.token_endpoint = token_endpoint;
    config.refresh_token =
        skein_lib::Secret::new(bytes::copy_of(if story == Story::SignInAgain { b"revoked" } else { b"seed" }));
    let mut peer = peer::Peer::new(
        address,
        transport,
        skein_fake_peers::Limits {
            io: profile.io,
            connections: 2,
            queue: ROOM,
            plaintext: 32_768,
            ciphertext: 32_768,
            observations: 16,
            observation_bytes: 4096,
        },
        config,
        issuer_limits,
        crate::streams::server_limits(&issuer_limits),
    )
    .expect("independent issuer");
    if story == Story::Retry {
        peer.queue(fake::Plan {
            status: 429,
            body: fake::Body::Raw(skein_lib::Secret::new(Box::new([]))),
            delay: Duration::from_millis(5),
            retry_after: Duration::from_secs(1),
        })
        .expect("rate limit before rotating refresh");
    }
    peer.queue(rotation(story, status)).expect("one rotation plan");
    Process::Issuer(Box::new(peer))
}

/// Runs the actual owner component over io and the simulator, with seeded stream cuts.
#[must_use]
pub fn run(seed: u64, story: Story, faulted: bool, memory: Memory) -> Outcome<Process> {
    let mut config = skein_sim::Config::calm();
    config.wall = skein_tls_kit::pki::VALID;
    if faulted {
        config.buffer = 127;
        config.faults.short_recv = 300;
        config.faults.short_send = 300;
    }
    let mut world =
        World::new(seed, config, Judge { wake: None, deadline: None, story, shutdown: false, passed: false }, memory);
    world.spawn(|| issuer(Transport::Plaintext, story, (Ipv4Addr::LOCALHOST, 31000).into()));
    world.spawn(|| Process::Client(Box::new(Client::new(story, Transport::Plaintext))));
    world.run()
}

/// Runs a rotating refresh through TLS on the actual shared ring and loopback fake issuer.
pub fn real_refresh() {
    // Rotation is owned by simulated::a_rotated_refresh_token_is_kept_before_it_is_lent.
    // Lead waits 20 seconds for its refresh, beyond this real world's 10-second patience;
    // Rotate exercises one TLS refresh with the shared judge and issuer.
    let mut world = skein_world::real::World::new(Judge {
        wake: None,
        deadline: None,
        story: Story::Rotate,
        shutdown: false,
        passed: false,
    });
    world.spawn(|| issuer(Transport::Tls, Story::Rotate, (Ipv4Addr::LOCALHOST, 0).into()));
    world.spawn(|| Process::Client(Box::new(Client::new(Story::Rotate, Transport::Tls))));
    let clock = skein_shell::Clock::new();
    drop(world.run(&clock, Duration::from_secs(10)));
}

/// Replay compares shared kernel trace and content-free owner facts.
pub fn assert_replay(first: &Outcome<Process>, second: &Outcome<Process>) {
    assert_eq!(first.trace, second.trace);
    for (first, second) in first.procs.iter().zip(&second.procs) {
        match (first, second) {
            (Process::Client(first), Process::Client(second)) => assert_eq!(first.facts, second.facts),
            (Process::Issuer(first), Process::Issuer(second)) => {
                assert!(first.observations() == second.observations(), "outside issuer facts replay");
            }
            _ => {
                panic!("process order replays")
            }
        }
    }
}

/// Runs the same owner and independent issuer directly over shared seeded streams.
/// Returns the content-free owner facts and stream trace for protocol replay.
#[must_use]
pub fn protocol(seed: u64, story: Story) -> (Vec<Fact>, Trace) {
    let (client, trace) = protocol_client(seed, Client::new(story, Transport::Plaintext));
    (client.facts, trace)
}

pub(crate) fn protocol_client(seed: u64, client: Client) -> (Client, Trace) {
    let rig = client.protocol_rig(true);
    let (client, trace, _) = crate::streams::drive(seed, client, rig);
    (client, trace)
}

/// The close observed at the socket and TLS peer (skein/docs/design/net/link.md, 4).
#[derive(Debug, Default)]
pub struct TlsClose {
    pub peer_closed: bool,
    pub started: Option<Time>,
    pub deadline: Option<Time>,
    pub aborted: Option<Time>,
}

/// Runs a rotating refresh over TLS, optionally withholding the close's room.
#[must_use]
pub fn tls_close(seed: u64, room: bool) -> TlsClose {
    let client = Client::new(Story::Rotate, Transport::Tls);
    let rig = client.protocol_rig(room);
    crate::streams::drive(seed, client, rig).2
}

impl Client {
    fn protocol_rig(&self, room: bool) -> crate::streams::Rig {
        let story = self.story;
        let sign_out = self.sign_out.as_ref().map(|owner| owner.story);
        let profile = limits();
        let registration = registration(self.transport);
        let (endpoint, target): (&[u8], &[u8]) = if sign_out.is_some() {
            (b"http://127.0.0.1:31000/revoke", b"/revoke")
        } else {
            (&registration.token_endpoint, b"/token")
        };
        let issuer_limits = issuer_limits(&profile.client);
        let mut config = crate::streams::issuer_config(&registration, sign_out.map(crate::sign_out::issuer_terms));
        config.token_endpoint = bytes::copy_of(endpoint);
        let mut issuer = fake::Issuer::new(config, issuer_limits).expect("issuer");
        issuer.queue(rotation(story, sign_out.map_or(200, crate::sign_out::status))).expect("one issuer response");
        crate::streams::Rig {
            issuer,
            endpoint: bytes::copy_of(endpoint),
            target,
            profile,
            server_limits: crate::streams::server_limits(&issuer_limits),
            rewires: false,
            callbacks: None,
            room,
            expected_posts: None,
            turns: 100_000,
        }
    }

    pub(crate) fn protocol_turn(&mut self, env: &Env<accounts::Limits>, encrypted: bool, close: &mut TlsClose) {
        let now = env.now;
        let story = self.story;
        if encrypted && close.started.is_none() && self.facts.contains(&Fact::Keep(1)) {
            close.started = Some(now);
            close.deadline = self.component.as_ref().expect("component").next_deadline();
        }
        crate::accounts::protocol_files(self, env);
        crate::sign_out::protocol_files(self, env);
        if story == Story::Released && now >= Time::ZERO.saturating_add(Duration::from_secs(40)) {
            self.close(env);
        }
    }

    pub(crate) fn protocol_socket(&mut self, env: &Env<accounts::Limits>, request: &io::Request) {
        use skein_lib::stream::Down;
        let story = self.story;
        crate::accounts::before_socket(self, env, request);
        crate::shared::before_socket(self, request);
        if !self.closing
            && ((matches!(request, io::Request::Connect { .. })
                && matches!(story, Story::CloseConnecting | Story::Abort))
                || (matches!(request, io::Request::Stream { down: Down::Send(_), .. }) && story == Story::CloseSending))
        {
            self.close(env);
            if story == Story::Abort {
                self.ask(env, accounts::Request::Abort);
            }
        }
    }

    pub(crate) fn deliver(&mut self, env: &Env<accounts::Limits>, event: io::Event) {
        self.component.as_mut().expect("component").up(
            env,
            event,
            &mut self.above,
            &mut self.requests,
            &mut self.files,
        );
    }
}

pub(crate) fn terms() -> oauth::Terms {
    oauth::Terms {
        parameters: Box::new([]),
        openid: None,
        issued_client_id: false,
        varying_port: false,
        resource: None,
        required_scopes: None,
        revocation_endpoint: None,
        sign_in_again: Box::new([]),
    }
}

/// Pause between creating OAuth output and the next component progress turn.
#[must_use]
pub fn delayed_oauth_output() -> Vec<&'static str> {
    use skein_lib::Token;

    // Each fire advances one child handoff; this covers the refresh and its
    // timeout settlement without introducing another entrance.
    const TURNS: u32 = 16;
    const SOCKET: Token = Token::new(9);
    let mut client = Client::new(Story::Rotate, Transport::Plaintext);
    client.address = Some((Ipv4Addr::LOCALHOST, 31000).into());
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    client.start(&env);
    let component = client.component.as_mut().expect("started component");
    let mut trace = Vec::new();
    if component.has_work() && client.requests.is_empty() {
        trace.push("held OAuth output");
    }
    env.now = component.next_deadline().expect("refresh HTTP deadline").saturating_add(Duration::from_nanos(1));
    env.wall = Wall::from_nanos(env.now.as_nanos());
    for _ in 0..TURNS {
        component.fire(&env, &mut client.above, &mut client.requests, &mut client.files);
        for _ in 0..client.requests.len() {
            let request = client.requests.pop().expect("turn's requests");
            match request {
                io::Request::Connect { owner, .. } => {
                    trace.push("dispatched HTTP request");
                    component.up(
                        &env,
                        io::Event::Connecting { owner, socket: SOCKET },
                        &mut client.above,
                        &mut client.requests,
                        &mut client.files,
                    );
                }
                io::Request::Abort { .. } => trace.push("Tick aborted socket"),
                other @ (io::Request::Listen { .. }
                | io::Request::Bind { .. }
                | io::Request::Reject { .. }
                | io::Request::Stream { .. }
                | io::Request::Output { .. }
                | io::Request::Spawn { .. }
                | io::Request::SpawnAlone { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. }
                | io::Request::Close { .. }) => panic!("the expired refresh only connects then aborts: {other:?}"),
            }
        }
    }
    trace
}

fn rotation(story: Story, status: u16) -> fake::Plan {
    crate::streams::token_plan(
        status,
        bytes::copy_of(b"new"),
        (story != Story::Omit).then(|| bytes::copy_of(b"next")),
        None,
    )
}
