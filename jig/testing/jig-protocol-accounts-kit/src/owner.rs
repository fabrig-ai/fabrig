//! Routes the accounts component, its real domain child and the memory secrets
//! store (jig/docs/design/protocol/llm.md, 5 and 8; protocol/store/secrets.md).
use crate::fixtures::{self, ACCOUNT_CHAT, ROOM};
use crate::world::{Installation, Story};
use jig_domain_accounts as child;
use jig_domain_core_vocabulary as core;
use jig_protocol_accounts::{
    Component,
    component::{Effect, Event},
    party,
};
use jig_protocol_store as store;
use skein_io::{self as io, kernel};
use skein_lib::{Env, List, Map, Queue, Secret, Time, Token, Wall, bytes, sha256};
use skein_oauth_accounts as driver;
use skein_world::Host;
use std::net::SocketAddr;

/// A store component over empty memory stores.
fn memory(limits: &store::Limits) -> store::Store {
    store::Store::over(
        limits,
        fixtures::APPLICATION,
        &[],
        store::Stores {
            store: skein_kv::Store::memory(&limits.store),
            secrets: skein_kv::Store::memory(&limits.secrets),
        },
    )
}

/// The store's answers, by what was asked; a replace is answered under its
/// source's number.
const OPEN: Token = Token::new(1);
const GET: Token = Token::new(3);
const SOURCE: u64 = 1 << 32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Heard {
    Granted { account: u32, generation: u64 },
    Expiring { account: u32, generation: u64 },
    Failed { account: u32, failure: child::Failure },
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trace {
    Requested(child::ToComponent),
    /// The store held this generation of the source's record at start.
    Loaded {
        account: u32,
        generation: u64,
    },
    /// The store answered that this generation of the record is durable.
    Kept {
        account: u32,
        generation: u64,
    },
    Heard(Heard),
    /// The driver asked for a sign-in's visit.
    Visit {
        account: u32,
    },
    /// The driver ended a sign-in, cancelled.
    SignInCancelled {
        account: u32,
    },
    Closed,
}
/// A value the table lent a run, by its digest.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Lent {
    pub account: u32,
    pub generation: u64,
    pub digest: [u8; 32],
}
pub struct Owner {
    io: io::Io,
    component: Option<Component>,
    limits: driver::Limits,
    child: child::Domain,
    child_limits: child::Limits,
    child_out: Queue<child::Request>,
    events: Queue<io::Event>,
    requests: Queue<io::Request>,
    effects: Queue<Effect>,
    store: store::Store,
    store_limits: store::Limits,
    store_above: Queue<store::Event>,
    below: store::Below,
    paste: [u8; 1024],
    paste_len: usize,
    paste_wrong: Option<()>,
    sign_in_again_requested: Option<()>,
    finish_carried: Option<u64>,
    move_mode: Option<()>,
    /// The referee's changes wait inline until the owner's next metered call.
    pending_changes: [Option<(Token, child::Change)>; 2],
    move_events: Queue<child::Event>,
    /// Test client's output and preallocated input; the referee only copies bytes.
    moved_file: Option<(u32, Secret)>,
    move_input: Box<[u8]>,
    move_input_len: Option<usize>,
    carried_move: Option<u64>,
    lent: Vec<Lent>,
    last_kept: Option<Secret>,
    completions: Queue<kernel::Complete>,
    submissions: Queue<kernel::Submit>,
    pub address: Option<SocketAddr>,
    heard: Vec<Heard>,
    facts: Vec<child::Fact>,
    trace: Vec<Trace>,
    close_requested: Option<()>,
    reject_requested: Option<()>,
    closing: bool,
    closed: bool,
    usable: bool,
    now: Time,
    story: Story,
    account: u32,
    /// The installation's identifier the store answered at open.
    installation: [u8; 16],
    /// The URL the driver asked to be visited.
    visit: Option<Box<[u8]>>,
    ended: Vec<child::Ended>,
    answers: Vec<child::Answer>,
    removed: Vec<u32>,
    erased: Option<()>,
    account_sign_out_asked: Option<()>,
}
impl std::fmt::Debug for Owner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Owner")
            .field("heard", &self.heard)
            .field("trace", &self.trace)
            .field("closed", &self.closed)
            .finish_non_exhaustive()
    }
}
impl Owner {
    /// The owner over an open memory store holding `held` as its source's
    /// record, as an earlier run left it, and `installation.kept` as the
    /// installation's identifier if an earlier start kept one.
    #[expect(
        clippy::too_many_lines,
        reason = "the explicit domain and protocol fixture restore the same source identities"
    )]
    pub(crate) fn new(story: Story, held: Option<&[u8]>, installation: Installation) -> Self {
        let mut limits = fixtures::limits();
        if story == Story::AccountSignInFull {
            limits.accounts = 1;
        }
        if story == Story::SignInAgain {
            // A cancelled listener counts until it settles (skein skein/docs/design/oauth/accounts.md 5): room beside the live sign-in.
            limits.listeners = 2;
            limits.io.sockets = 2 * limits.listeners + limits.exchanges;
        }
        let child_limits = fixtures::child_limits();
        let account = ACCOUNT_CHAT;
        let store_limits = fixtures::store_limits();
        let mut owner = Self {
            io: io::Io::new(&limits.io),
            component: None,
            child: child::Domain::new(
                &child_limits,
                child::Providers {
                    providers: Box::new([child::Provider {
                        list: None,
                        number: 1,
                        name: b"Fake".as_slice().into(),
                        signed_in_plan: true,
                        manage_usage: Some(b"https://usage.example/".as_slice().into()),
                        offers: child::Offers { sign_in: true, key: false, programs: Box::new([]) },
                    }]),
                },
            ),
            child_out: Queue::with_capacity(ROOM),
            child_limits,
            events: Queue::with_capacity(ROOM),
            requests: Queue::with_capacity(ROOM),
            completions: Queue::with_capacity(ROOM),
            submissions: Queue::with_capacity(ROOM),
            effects: Queue::uncapped(),
            store: memory(&store_limits),
            store_limits,
            store_above: Queue::uncapped(),
            below: store::Below {
                store: Queue::uncapped(),
                secrets: Queue::uncapped(),
                transcripts: Queue::uncapped(),
            },
            paste: [0; 1024],
            paste_len: 0,
            paste_wrong: None,
            sign_in_again_requested: None,
            finish_carried: None,
            move_mode: None,
            pending_changes: [None, None],
            move_events: Queue::with_capacity(ROOM),
            moved_file: None,
            move_input: bytes::zeroed(
                usize::try_from(jig_protocol_accounts::moved::bound(&limits.client.document)).expect("file bound"),
            ),
            move_input_len: None,
            carried_move: None,
            lent: Vec::with_capacity(8),
            last_kept: None,
            limits,
            address: None,
            heard: Vec::with_capacity(8),
            facts: Vec::with_capacity(32),
            trace: Vec::with_capacity(16),
            close_requested: None,
            reject_requested: None,
            closing: false,
            closed: false,
            usable: false,
            now: Time::ZERO,
            story,
            account,
            installation: [0; 16],
            visit: None,
            ended: Vec::new(),
            answers: Vec::new(),
            removed: Vec::new(),
            erased: None,
            account_sign_out_asked: None,
        };
        if !fixtures::account_sign_in(story) {
            let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: child_limits };
            for record in [
                child::Stored::Account {
                    list: None,
                    number: account + 1_000,
                    provider: 1,
                    holder: child::Holder::Deployment,
                    place: 0,
                    uses: None,
                    subject: Box::new([]),
                    display: Box::new([]),
                },
                child::Stored::Source {
                    account: account + 1_000,
                    number: account,
                    place: 0,
                    kind: child::Kind::SignIn,
                    name: Box::new([]),
                    held: None,
                },
            ] {
                child::step(&mut owner.child, &env, child::Event::Restore { record }, &mut owner.child_out);
            }
        }
        if let Some(kept) = installation.kept {
            // An earlier start kept its draw with its first commit; this
            // start opens the same stores again.
            assert_eq!(owner.open(kept), kept, "an empty store answers this start's draw");
            let stores = std::mem::replace(&mut owner.store, memory(&owner.store_limits)).into_stores();
            owner.store = store::Store::over(&owner.store_limits, fixtures::APPLICATION, &[], stores);
        }
        owner.installation = owner.open(installation.drawn);
        if let Some(held) = held {
            let value = Secret::new(bytes::copy_of(held));
            owner.secrets(Time::ZERO, Wall::EPOCH, source_token(account), replace(account, value, None));
            let kept = owner.store_above.pop();
            assert!(matches!(kept, Some(store::Event::Kept { .. })), "the store keeps the record held before the run");
        }
        owner
    }
    #[must_use]
    pub fn new_move(held: bool, moving: bool) -> Self {
        let mut owner = Self::new(Story::SignIn, None, crate::world::FIRST_START);
        owner.move_mode = Some(());
        if held {
            for record in [
                child::Stored::Account {
                    list: None,
                    number: 1000,
                    provider: 1,
                    holder: child::Holder::Party(7),
                    place: 0,
                    uses: None,
                    subject: bytes::copy_of(b"subject"),
                    display: bytes::copy_of(b"Moved"),
                },
                child::Stored::Source {
                    account: 1000,
                    number: 0,
                    place: 0,
                    kind: child::Kind::SignIn,
                    name: Box::new([]),
                    held: moving.then_some(child::Held::Moving),
                },
            ] {
                owner.child_event(Time::ZERO, Wall::EPOCH, child::Event::Restore { record });
            }
            let mut record = fixtures::kept();
            record.subject = Some(bytes::copy_of(b"subject"));
            record.issued_client_id = Some(bytes::copy_of(b"client"));
            record.granted_scopes = Some(bytes::copy_of(b"openid read"));
            record.earliest_refresh_at = Some(Wall::from_nanos(skein_lib::Duration::from_secs(200).as_nanos()));
            let value =
                Secret::new(skein_oauth::encode_record(&record, &owner.limits.client.document).expect("move fixture"));
            owner.secrets(Time::ZERO, Wall::EPOCH, source_token(0), replace(0, value, None));
            assert!(matches!(owner.store_above.pop(), Some(store::Event::Kept { .. })));
        }
        owner
    }
    pub fn change(&mut self, request: u64, change: child::Change) {
        let slot = self.pending_changes.iter_mut().find(|slot| slot.is_none()).expect("at most two pending changes");
        *slot = Some((Token::new(request), change));
    }
    #[must_use]
    pub fn moved_file(&self, source: u32) -> Option<&Secret> {
        self.moved_file.as_ref().filter(|(number, _)| *number == source).map(|(_, value)| value)
    }
    pub fn carry_move(&mut self, value: &Secret) {
        self.move_input[..value.len()].copy_from_slice(value.expose());
        self.move_input_len = Some(value.len());
    }
    #[must_use]
    pub fn carried_move(&self) -> Option<u64> {
        self.carried_move
    }
    #[must_use]
    pub fn source_usable(&self) -> bool {
        self.child.source_grant(self.account, self.now).is_some()
    }
    pub fn carried_value(&mut self, carried: u64) -> Option<Secret> {
        self.inspect_secret(store::SecretOwner::Application, carried).map(|value| {
            let store::SecretValue::Carried { kind: store::CarriedKind::Moved, value } = value else {
                panic!("carried move kind")
            };
            value
        })
    }
    /// Opens the store, as this start drew `drawn`, and answers the
    /// installation's identifier it keeps. At an empty store the owner, as
    /// the core's stand-in, commits the deployment's header first, which
    /// keeps the identifier.
    fn open(&mut self, drawn: [u8; 16]) -> [u8; 16] {
        // Memory stores open no file: the root is never read.
        let root = store::Root { token: OPEN, fd: kernel::Fd::new(3) };
        self.ask(Time::ZERO, Wall::EPOCH, store::Request::Open { owner: OPEN, root, installation: drawn });
        let Some(store::Event::Opened { owner: OPEN, last, installation, pending }) = self.store_above.pop() else {
            panic!("the memory store opens at once");
        };
        // This accounts fixture does not consume the map census (protocol/store/README.md, 6.3).
        let Some(store::Event::Census { .. }) = self.store_above.pop() else { panic!("opened census") };
        if pending {
            // This store-only driver has no engine root to ask for the switch.
            self.ask(Time::ZERO, Wall::EPOCH, store::Request::Switch { owner: OPEN });
            assert!(matches!(self.store_above.pop(), Some(store::Event::Switched { owner: OPEN })));
        }
        {
            let number = last + 1;
            let header = core::Record::Core(core::CoreRecord::Deployment(core::Deployment {
                id: fixtures::DEPLOYMENT,
                tasks: 0,
                people: 0,
                sign_ins: 0,
                messages: 0,
                runs: 0,
                calls: 0,
                connector_rows: 0,
                projects: 0,
                applications: 0,
                commits: number,
            }));
            self.ask(
                Time::ZERO,
                Wall::EPOCH,
                store::Request::Commit { number, writes: Box::new([store::Write::Save(header), store::Write::Wires]) },
            );
            assert!(
                matches!(self.store_above.pop(), Some(store::Event::Committed { .. })),
                "the memory store commits at once"
            );
        }
        installation
    }
    /// The installation's identifier the store answered at this start.
    #[must_use]
    pub fn installation(&self) -> [u8; 16] {
        self.installation
    }
    /// The URL the driver asked to be visited for a sign-in.
    #[must_use]
    pub fn visit(&self) -> Option<&[u8]> {
        self.visit.as_deref()
    }
    #[must_use]
    pub fn child_heard(&self) -> &[Heard] {
        &self.heard
    }
    #[must_use]
    pub fn facts(&self) -> &[child::Fact] {
        &self.facts
    }
    #[must_use]
    pub fn trace(&self) -> &[Trace] {
        &self.trace
    }
    #[must_use]
    pub fn table_value(&self, account: u32, generation: u64) -> Option<&[u8]> {
        Some(self.component.as_ref()?.value(account, generation, self.now)?.token.expose())
    }
    #[must_use]
    pub fn became_usable(&self) -> bool {
        self.usable
    }
    #[must_use]
    pub fn closed(&self) -> bool {
        self.closed
    }
    pub fn close(&mut self) {
        self.close_requested = Some(());
    }
    pub(crate) fn reject(&mut self) {
        self.reject_requested = Some(());
    }
    /// Each value the table lent, in order.
    #[must_use]
    pub fn lent(&self) -> &[Lent] {
        &self.lent
    }
    /// The record the store last answered as kept, read back from it.
    #[must_use]
    pub fn last_kept(&self) -> Option<&[u8]> {
        self.last_kept.as_ref().map(Secret::expose)
    }
    #[must_use]
    pub fn account_sign_in_ended(&self) -> &[child::Ended] {
        &self.ended
    }
    #[must_use]
    pub fn account_answers(&self) -> &[child::Answer] {
        &self.answers
    }
    #[must_use]
    pub fn account_record_erased(&self) -> bool {
        self.erased.is_some()
    }
    #[must_use]
    pub fn account_removed(&self) -> bool {
        !self.removed.is_empty()
    }
    /// The durable source record, read after the story has settled.
    pub fn read_record(&mut self) -> Option<Box<[u8]>> {
        self.inspect_secret(store::SecretOwner::Source, u64::from(self.account)).map(|value| {
            let store::SecretValue::Record(value) = value else { panic!("source record kind") };
            bytes::copy_of(value.expose())
        })
    }
    /// A referee's read owns its temporary handoff buffers outside process calls.
    fn inspect_secret(&mut self, owner: store::SecretOwner, number: u64) -> Option<store::SecretValue> {
        let env = Env { now: self.now, wall: Wall::EPOCH, limits: self.store_limits };
        let mut above = Queue::uncapped();
        let mut below =
            store::Below { store: Queue::uncapped(), secrets: Queue::uncapped(), transcripts: Queue::uncapped() };
        store::down(
            &mut self.store,
            &env,
            store::Request::Secrets { owner: GET, request: store::Secrets::Get { owner, number } },
            &mut above,
            &mut below,
        );
        let Some(store::Event::Secret { owner: GET, value }) = above.pop() else {
            panic!("the memory store answers Get immediately");
        };
        assert!(above.is_empty() && below.store.is_empty() && below.secrets.is_empty() && below.transcripts.is_empty());
        value
    }
    #[must_use]
    pub fn unstarted(&self) -> bool {
        self.component.is_none()
    }
    // The story signs out after the successful account sign-in's HTTP exchanges close.
    // Its grant alone does not mean the driver has released that exchange's place.
    fn account_sign_out_ready(&self) -> bool {
        matches!(self.story, Story::AccountSignOut { .. })
            && self.usable
            && self.account_sign_out_asked.is_none()
            && self.io.is_empty()
            && self.requests.is_empty()
            && self.events.is_empty()
            && self.effects.is_empty()
    }
    pub(crate) fn child_event(&mut self, now: Time, wall: Wall, event: child::Event) {
        child::step(&mut self.child, &Env { now, wall, limits: self.child_limits }, event, &mut self.child_out);
        self.usable |= self.child.source_grant(self.account, now).is_some();
        while let Some(ended) = self.child.pop_ended() {
            self.ended.push(ended);
        }
        while let Some(fact) = self.child.pop_fact() {
            self.facts.push(fact);
        }
    }
    /// The driver starts from the store's records (llm.md 5.1).
    fn start(&mut self, env: &Env<driver::Limits>) {
        let mut providers = Map::with_capacity(1);
        let account = if !fixtures::account_sign_in(self.story) && self.move_mode.is_none() {
            driver::Account::SignIn {
                shared: false,
                origins: List::with_capacity(0),
                registration: fixtures::registration(jig_protocol_accounts::installation::terms(&self.installation)),
                endpoint: driver::Endpoint {
                    address: self.address.expect("issuer"),
                    transport: driver::Transport::Plaintext,
                },
                keeper: driver::Keeper::Owner { kept: None },
            }
        } else {
            self.party_account()
        };
        providers.insert(1, account).expect("provider fits");
        let crypto = skein_lib::CryptoRng::new(&skein_world::crypto_seed(17), 0).expect("seed");
        self.component = Some(Component::new(
            bytes::copy_of(fixtures::MOVE_APPLICATION),
            providers,
            self.limits,
            crypto,
            self.store_limits.secrets.page.rows,
        ));
        self.component.as_mut().expect("component").step(
            env,
            Event::Opened { installation: self.installation },
            &mut self.effects,
        );
        if self.story == Story::AccountSignInFull {
            let mut saved = fixtures::kept();
            saved.key = 999;
            saved.issued_client_id = Some(bytes::copy_of(b"issued-client"));
            saved.granted_scopes = Some(bytes::copy_of(b"openid read"));
            saved.subject = Some(bytes::copy_of(b"subject"));
            let value = skein_oauth::encode_record(&saved, &self.limits.client.document).expect("saved source");
            self.secrets(env.now, env.wall, GET, replace(999, Secret::new(value), None));
            assert!(matches!(self.store_above.pop(), Some(store::Event::Kept { .. })));
            self.component.as_mut().expect("component").step(
                env,
                Event::Request(child::Request::Signing(child::Signing::Restore {
                    source: 999,
                    provider: 1,
                    kind: child::Kind::SignIn,
                })),
                &mut self.effects,
            );
            self.component.as_mut().expect("component").step(
                env,
                Event::Request(child::Request::Component(child::ToComponent::Want { account: 999 })),
                &mut self.effects,
            );
        }
        self.child_event(env.now, env.wall, child::Event::Want);
        if fixtures::account_sign_in(self.story) && self.move_mode.is_none() {
            self.child_event(
                env.now,
                env.wall,
                child::Event::SignIn { request: Token::new(80), holder: child::Holder::Party(7), provider: 1, by: 7 },
            );
        }
    }
    fn ask(&mut self, now: Time, wall: Wall, request: store::Request) {
        let env = Env { now, wall, limits: self.store_limits };
        store::down(&mut self.store, &env, request, &mut self.store_above, &mut self.below);
        assert!(
            self.below.store.is_empty() && self.below.secrets.is_empty() && self.below.transcripts.is_empty(),
            "a memory store files nothing"
        );
    }
    fn secrets(&mut self, now: Time, wall: Wall, owner: Token, request: store::Secrets) {
        self.ask(now, wall, store::Request::Secrets { owner, request });
    }
    fn party_account(&self) -> driver::Account {
        let registration = fixtures::party_registration(self.story);
        let endpoint =
            driver::Endpoint { address: self.address.expect("issuer"), transport: driver::Transport::Plaintext };
        let mut origins = List::with_capacity(1);
        origins
            .push(driver::Origin {
                uri: registration.issuer.clone(),
                endpoint: driver::Endpoint {
                    address: self.address.expect("issuer"),
                    transport: driver::Transport::Plaintext,
                },
            })
            .expect("origin fits");
        party::party_account(&registration, endpoint, origins, &self.installation)
    }

    pub(crate) fn hand_paste(&mut self, uri: &[u8], wrong: bool) {
        self.paste[..uri.len()].copy_from_slice(uri);
        self.paste_len = uri.len();
        self.paste_wrong = wrong.then_some(());
    }
    pub(crate) fn sign_in_again(&mut self) {
        self.sign_in_again_requested = Some(());
    }
    fn restart_sign_in(&mut self, now: Time, wall: Wall) {
        self.visit = None;
        self.child_event(
            now,
            wall,
            child::Event::SignIn { request: Token::new(83), holder: child::Holder::Party(7), by: 7, provider: 1 },
        );
    }
    fn paste(&mut self, env: &Env<driver::Limits>) {
        if self.paste_len == 0 {
            return;
        }
        let mut uri = bytes::copy_of(&self.paste[..self.paste_len]);
        self.paste_len = 0;
        if self.paste_wrong.take().is_some() {
            let at = uri.windows(6).position(|part| part == b"state=").expect("fixture state") + 6;
            uri[at] = if uri[at] == b'a' { b'b' } else { b'a' };
        }
        self.secrets(env.now, env.wall, GET, store::Secrets::Hold { value: Secret::new(uri) });
        let Some(store::Event::Carried { number, .. }) = self.store_above.pop() else { panic!("held address") };
        self.finish_carried = Some(number);
        self.child_event(
            env.now,
            env.wall,
            child::Event::Change {
                request: Token::new(81),
                holder: child::Holder::Party(7),
                change: child::Change::FinishSignIn { source: self.account, carried: number, by: 7 },
            },
        );
    }

    fn child_requests(&mut self, env: &Env<driver::Limits>) {
        while let Some(request) = self.child_out.pop() {
            match request {
                request @ (child::Request::Component(_) | child::Request::Signing(_)) => {
                    if let child::Request::Component(request) = &request {
                        self.trace.push(Trace::Requested(*request));
                    }
                    if let child::Request::Signing(child::Signing::SignIn { source, .. }) = &request {
                        self.account = *source;
                    }
                    self.component.as_mut().expect("started").step(env, Event::Request(request), &mut self.effects);
                }
                child::Request::Answer { request, answer } => {
                    if request == Token::new(81)
                        && let Some(number) = self.finish_carried.take()
                    {
                        self.secrets(
                            env.now,
                            env.wall,
                            GET,
                            store::Secrets::Erase { owner: store::SecretOwner::Application, number },
                        );
                        assert!(matches!(self.store_above.pop(), Some(store::Event::Erased { .. })));
                    }
                    if let child::Answer::MovingOut { source, .. } = &answer {
                        let file = self
                            .component
                            .as_mut()
                            .expect("component")
                            .take_moved(*source)
                            .expect("moving out carries the file");
                        assert!(
                            self.component.as_mut().expect("component").take_moved(*source).is_none(),
                            "file goes out once"
                        );
                        self.moved_file = Some((*source, file));
                    }
                    self.answers.push(answer);
                }
                child::Request::Programs { .. }
                | child::Request::ReadList { .. }
                | child::Request::CheckProgram { .. }
                | child::Request::SignOutProgram { .. }
                | child::Request::Granted { grant: _ }
                | child::Request::Save { .. }
                | child::Request::Erase { .. } => (),
                child::Request::Closed => {
                    self.closed = true;
                    self.trace.push(Trace::Closed);
                }
            }
        }
    }
    fn observe(&mut self, env: &Env<driver::Limits>) {
        while let Some(effect) = self.effects.pop() {
            match effect {
                Effect::Io(request) => self.requests.push(request),
                Effect::Store { exchange, request } => {
                    let record = match &request {
                        store::Secrets::Replace { number, value: store::SecretValue::Record(value), .. } => {
                            Some((u32::try_from(*number).expect("source"), value.clone()))
                        }
                        store::Secrets::Replace { .. }
                        | store::Secrets::Keep { .. }
                        | store::Secrets::Adopt { .. }
                        | store::Secrets::AdoptValue { .. }
                        | store::Secrets::Hold { .. }
                        | store::Secrets::Carry { .. }
                        | store::Secrets::Erase { .. }
                        | store::Secrets::Get { .. }
                        | store::Secrets::Load { .. } => None,
                    };
                    let restoring = matches!(request, store::Secrets::Get { owner: store::SecretOwner::Source, .. });
                    self.secrets(env.now, env.wall, exchange, request);
                    let answer = self.store_above.pop().expect("memory secrets answer");
                    if let Some((account, value)) = record
                        && matches!(answer, store::Event::Kept { .. })
                    {
                        let record = skein_oauth::decode_record(value.expose(), &self.limits.client.document)
                            .expect("kept record");
                        self.trace.push(Trace::Kept { account, generation: record.generation });
                        self.last_kept = Some(value);
                    }
                    if restoring
                        && let store::Event::Secret { value: Some(store::SecretValue::Record(value)), .. } = &answer
                        && let Ok(record) = skein_oauth::decode_record(value.expose(), &self.limits.client.document)
                    {
                        self.trace.push(Trace::Loaded { account: record.key, generation: record.generation });
                    }
                    if matches!(answer, store::Event::Erased { .. }) {
                        self.erased = Some(());
                    }
                    self.component.as_mut().expect("component").step(env, Event::Store(answer), &mut self.effects);
                }
                Effect::Granted { source, generation, value, .. } => {
                    self.lent.push(Lent { account: source, generation, digest: sha256::digest(value.token.expose()) });
                }
                Effect::SourceRemoved { source } => self.removed.push(source),
                Effect::Removed { .. } => (),
                Effect::InvalidSecret => panic!("accounts secret kind"),
                Effect::Child(event) => {
                    // The full-driver fixture occupies a source outside this child's story.
                    if self.story == Story::AccountSignInFull
                        && matches!(event, child::Event::Component(child::FromComponent::Granted { account: 999, .. }))
                    {
                        continue;
                    }
                    if let child::Event::Visited { source, address } = &event {
                        self.trace.push(Trace::Visit { account: *source });
                        self.visit = Some(address.clone());
                    }
                    if let child::Event::Adopted { source, kept: true } = &event {
                        self.account = *source;
                    }
                    if let child::Event::Component(component) = &event {
                        let heard = match component {
                            child::FromComponent::Granted { account, generation, .. } => {
                                Some(Heard::Granted { account: *account, generation: *generation })
                            }
                            child::FromComponent::Expiring { account, generation } => {
                                Some(Heard::Expiring { account: *account, generation: *generation })
                            }
                            child::FromComponent::Failed { account, failure } => {
                                Some(Heard::Failed { account: *account, failure: *failure })
                            }
                            child::FromComponent::KeepFailed { .. }
                            | child::FromComponent::Refreshed { .. }
                            | child::FromComponent::Closed => None,
                        };
                        if let Some(heard) = heard {
                            self.heard.push(heard);
                            self.trace.push(Trace::Heard(heard));
                        }
                    }
                    self.child_event(env.now, env.wall, event);
                }
            }
        }
    }
}
fn source_token(account: u32) -> Token {
    Token::new(SOURCE | u64::from(account))
}
fn replace(account: u32, value: Secret, replaces: Option<[u8; 32]>) -> store::Secrets {
    store::Secrets::Replace {
        owner: store::SecretOwner::Source,
        number: u64::from(account),
        value: jig_protocol_store::SecretValue::Record(value),
        replaces,
    }
}
impl Host for Owner {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.now = now;
        let env = Env { now, wall, limits: self.limits };
        let io_env = Env { now, wall, limits: self.limits.io };
        self.paste(&env);
        if self.component.is_none() && self.address.is_some() {
            self.start(&env);
        }
        if self.component.is_none() {
            return;
        }
        if self.sign_in_again_requested.take().is_some() {
            self.restart_sign_in(now, wall);
        }
        if self.account_sign_out_ready() {
            self.account_sign_out_asked = Some(());
            self.child_event(
                now,
                wall,
                child::Event::SignOut {
                    request: Token::new(81),
                    holder: child::Holder::Party(7),
                    source: self.account,
                },
            );
        }
        if self.close_requested.take().is_some() && !self.closing {
            self.closing = true;
            self.child_event(now, wall, child::Event::Close);
        }
        if self.reject_requested.take().is_some() {
            self.child_event(now, wall, child::Event::Unauthorized { account: self.account, generation: 1 });
        }

        if let Some(len) = self.move_input_len.take() {
            let value = Secret::new(bytes::copy_of(&self.move_input[..len]));
            self.secrets(now, wall, GET, store::Secrets::Carry { kind: jig_protocol_store::CarriedKind::Moved, value });
            let Some(store::Event::Carried { number, .. }) = self.store_above.pop() else { panic!("carried move") };
            self.carried_move = Some(number);
            self.change(93, child::Change::MoveIn { carried: number });
        }
        for slot in &mut self.pending_changes {
            if let Some((request, change)) = slot.take() {
                self.move_events.push(child::Event::Change { request, holder: child::Holder::Party(7), change });
            }
        }
        while let Some(event) = self.move_events.pop() {
            self.child_event(now, wall, event);
        }
        self.child_requests(&env);

        for _ in 0..ROOM {
            if !self.io.is_ready() || self.events.room() < 3 || self.submissions.room() < 2 {
                break;
            }
            io::resume(&mut self.io, &io_env, &mut self.events, &mut self.submissions);
        }
        while let Some(complete) = self.completions.pop() {
            io::up(&mut self.io, &io_env, complete, &mut self.events, &mut self.submissions);
        }
        for _ in 0..ROOM {
            if !self.io.is_due(now) || self.events.room() < 3 || self.submissions.room() < 2 {
                break;
            }
            io::fire(&mut self.io, &io_env, &mut self.events, &mut self.submissions);
        }
        while let Some(event) = self.events.pop() {
            self.component.as_mut().expect("started").step(&env, Event::Io(event), &mut self.effects);
        }
        self.component.as_mut().expect("started").fire(&env, &mut self.effects);
        if self.child.is_due(now) && !self.closing {
            child::fire(&mut self.child, &Env { now, wall, limits: self.child_limits }, &mut self.child_out);
        }

        self.observe(&env);
        self.child_requests(&env);

        for _ in 0..ROOM {
            if !self.io.takes() || self.submissions.room() < 2 {
                break;
            }
            let Some(request) = self.requests.pop() else { break };
            io::down(&mut self.io, &io_env, request, &mut self.submissions);
        }

        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        self.paste_len > 0
            || self.sign_in_again_requested.is_some()
            || self.io.is_ready()
            || self.account_sign_out_ready()
            || !self.events.is_empty()
            || !self.requests.is_empty()
            || !self.effects.is_empty()
            || !self.store_above.is_empty()
            || self.pending_changes.iter().any(Option::is_some)
            || !self.move_events.is_empty()
            || self.move_input_len.is_some()
            || !self.child_out.is_empty()
            || !self.completions.is_empty()
            || (self.component.is_none() && self.address.is_some())
            || (self.close_requested.is_some() && !self.closing)
            || self.reject_requested.is_some()
            || self.component.as_ref().is_some_and(|component| {
                component.has_work(now) || component.next_deadline().is_some_and(|due| due <= now)
            })
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        [
            self.component.as_ref().and_then(Component::next_deadline),
            (!self.closing).then(|| self.child.next_deadline()).flatten(),
        ]
        .into_iter()
        .flatten()
        .min()
    }
    fn is_empty(&self) -> bool {
        self.closed
            && self.io.is_empty()
            && self.store_above.is_empty()
            && self.events.is_empty()
            && self.requests.is_empty()
            && self.effects.is_empty()
            && self.child_out.is_empty()
            && self.pending_changes.iter().all(Option::is_none)
            && self.completions.is_empty()
            && self.submissions.is_empty()
    }
    fn operations(&self) -> u32 {
        io::operations(&self.limits.io).expect("io operation bound")
    }
}
