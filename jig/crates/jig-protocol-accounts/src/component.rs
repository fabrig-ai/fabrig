//! The accounts composition and its ordering (jig/docs/design/protocol/llm.md, 5).
use crate::{finishes, installation, keys, moves, party, records, restart, table, translate};
use alloc::boxed::Box;
use jig_domain_accounts as domain;
use jig_protocol_store as store;
use skein_io as io;
use skein_lib::{CryptoRng, Env, List, Map, Queue, Secret, Time, Token, Wall};
use skein_oauth as oauth;
use skein_oauth_accounts as driver;
const SWEEP: Token = Token::new(1_u64 << 34);
const MOVE_OUT: u64 = 1_u64 << 35;
const MOVE_IN: u64 = 1_u64 << 36;
const MOVE_BACK: u64 = 1_u64 << 37;
const MOVE_ADOPT: u64 = 1_u64 << 38;
const FINISH_EXCHANGE: u64 = 1_u64 << 39;
const KEY_EXCHANGE: u64 = 1_u64 << 33;
/// Inputs from the owner, its store and its io routing.
#[derive(Debug)]
pub enum Event {
    Request(domain::Request),
    Io(io::Event),
    Store(store::Event),
    /// The store has acknowledged this start's durable mark.
    Opened {
        installation: [u8; 16],
    },
    Abort,
}
/// Outputs are routed without deciding the accounts' ordering.
#[derive(Debug)]
pub enum Effect {
    Child(domain::Event),
    Io(io::Request),
    Store {
        exchange: Token,
        request: store::Secrets,
    },
    /// Update the owner's credential delivery projection before notifying its child.
    Granted {
        source: u32,
        generation: u64,
        value: table::Value,
        expires: Time,
    },
    /// Withdraw a credential from the owner's delivery projection.
    Removed {
        source: u32,
    },
    /// The driver has released this source; a move may now read its final record.
    SourceRemoved {
        source: u32,
    },
    InvalidSecret,
}
/// Pending store effects and their answers, owned by the composition.
struct StoreState {
    durable: bool,
    installation: [u8; 16],
    answers: Queue<store::Event>,
    requests: Queue<Effect>,
}
impl StoreState {
    fn request(&mut self, exchange: Token, request: store::Secrets) {
        self.requests.push(Effect::Store { exchange, request });
    }
}
/// The driver starts after the start mark is durable (llm.md, 5.4).
enum Phase {
    Loading { crypto: CryptoRng },
    Running,
}
/// One durable erase per source; only sign-out continues to driver removal.
#[derive(Debug)]
enum Erasing {
    Revoked,
    SignOut { confirmed: bool },
    Removing,
}
/// Owns the accounts protocol ordering, the OAuth driver and the credential table
/// (jig/docs/design/protocol/llm.md, 5). Owner request tokens remain opaque.
#[expect(missing_debug_implementations, reason = "owns driver state and secret records")]
pub struct Component {
    driver: Option<driver::Component>,
    phase: Phase,
    env: Env<driver::Limits>,
    providers: Map<u32, driver::Account>,
    required: Map<u32, Box<[u8]>>,
    erasing: Map<u32, Erasing>,
    records: records::Records,
    keys: keys::Keys,
    finishes: finishes::Finishes,
    finish_out: Queue<finishes::Next>,
    moves: moves::Moves,
    source_providers: Map<u32, u32>,
    move_out: Queue<moves::Next>,
    inventory: restart::Inventory,
    key_out: Queue<keys::Next>,
    key_events: Queue<domain::FromComponent>,
    forgotten: Queue<u32>,
    early: Queue<domain::Request>,
    io: Queue<io::Request>,
    above: Queue<driver::Event>,
    /// Empty: each source's keeper is the owner, so the driver files nothing.
    files: Queue<driver::FileLower>,
    /// Aborted: the store is closing, and takes no more records.
    aborting: bool,
    closed: bool,
    dispatching_move: bool,
    store: StoreState,
    table: table::Table,
    output: Queue<Effect>,
    restoring: Map<u32, (u32, domain::Kind)>,
}
fn unkept(account: &driver::Account) -> driver::Account {
    let (registration, endpoint, shared, origins) = match account {
        driver::Account::SignIn { registration, endpoint, shared, origins, .. } => {
            (registration, endpoint, shared, origins)
        }
        driver::Account::HandedIn => unreachable!("provider template is a public sign-in"),
    };
    driver::Account::SignIn {
        registration: oauth::Registration {
            authorization_url: registration.authorization_url.clone(),
            token_endpoint: registration.token_endpoint.clone(),
            client_id: registration.client_id.clone(),
            redirect_uri: registration.redirect_uri.clone(),
            scope: registration.scope.clone(),
            wire: registration.wire,
            client_secret: registration.client_secret.clone(),
            pkce_for_confidential: registration.pkce_for_confidential,
            metadata_claim: registration.metadata_claim.clone(),
            terms: registration.terms.clone(),
        },
        endpoint: driver::Endpoint {
            address: endpoint.address,
            transport: match &endpoint.transport {
                driver::Transport::Tls { server_name, trust } => {
                    driver::Transport::Tls { server_name: server_name.clone(), trust: trust.clone() }
                }
                driver::Transport::Plaintext => driver::Transport::Plaintext,
            },
        },
        keeper: driver::Keeper::Owner { kept: None },
        shared: *shared,
        origins: copy_origins(origins),
    }
}

fn required_scopes(account: &driver::Account) -> Box<[u8]> {
    let registration = match account {
        driver::Account::SignIn { registration, .. } => registration,
        driver::Account::HandedIn => unreachable!("party sources use provider registrations"),
    };
    match &registration.terms {
        Some(terms) => terms.required_scopes.clone().unwrap_or_default(),
        None => Box::default(),
    }
}
/// The protocol child chooses the expected payload; a wrong restored kind is a failed read.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SecretKind {
    Record,
    Key,
    Carried(store::CarriedKind),
    Held,
}

impl Component {
    fn secret(&mut self, value: Option<store::SecretValue>, expected: SecretKind) -> Result<Option<Secret>, ()> {
        let payload = match value {
            None => return Ok(None),
            Some(store::SecretValue::Record(value)) if expected == SecretKind::Record => value,
            Some(store::SecretValue::Key(value)) if expected == SecretKind::Key => value,
            Some(store::SecretValue::Carried { kind, value }) if expected == SecretKind::Carried(kind) => value,
            Some(store::SecretValue::Held(value)) if expected == SecretKind::Held => value,
            Some(
                store::SecretValue::Digest { .. }
                | store::SecretValue::Record(_)
                | store::SecretValue::Key(_)
                | store::SecretValue::Carried { .. }
                | store::SecretValue::Application(_)
                | store::SecretValue::Held(_),
            ) => {
                self.output.push(Effect::InvalidSecret);
                self.abort();
                return Err(());
            }
        };
        Ok(Some(payload))
    }

    #[must_use]
    pub fn startup(&self, source: u32) -> (bool, u32, u32) {
        let mut restores = 0_u32;
        let mut wants = 0_u32;
        for request in &self.early {
            match request {
                domain::Request::Signing(signing) => match signing {
                    domain::Signing::Restore { source: found, .. } => {
                        if *found == source {
                            restores = restores.checked_add(1).expect("bounded early queue");
                        }
                    }
                    domain::Signing::Withdraw { .. }
                    | domain::Signing::Return { .. }
                    | domain::Signing::Refresh { .. }
                    | domain::Signing::Open { .. }
                    | domain::Signing::Adopt { .. }
                    | domain::Signing::Keep { .. }
                    | domain::Signing::Forget { .. }
                    | domain::Signing::Restored
                    | domain::Signing::SignIn { .. }
                    | domain::Signing::Redirected { .. }
                    | domain::Signing::CancelSignIn { .. }
                    | domain::Signing::SignOut { .. }
                    | domain::Signing::Replaced { .. } => (),
                },
                domain::Request::Component(request) => match request {
                    domain::ToComponent::Want { account } => {
                        if *account == source {
                            wants = wants.checked_add(1).expect("bounded early queue");
                        }
                    }
                    domain::ToComponent::Rejected { .. }
                    | domain::ToComponent::Release { .. }
                    | domain::ToComponent::Close => (),
                },
                domain::Request::Granted { .. }
                | domain::Request::Save { .. }
                | domain::Request::Erase { .. }
                | domain::Request::Answer { .. }
                | domain::Request::Programs { .. }
                | domain::Request::CheckProgram { .. }
                | domain::Request::SignOutProgram { .. }
                | domain::Request::Closed => (),
            }
        }
        (self.driver.is_some(), restores, wants)
    }
    pub fn take_moved(&mut self, source: u32) -> Option<Secret> {
        self.moves.take(source)
    }

    /// Limits and public provider templates have passed the driver's validation.
    /// The durable installation is supplied before work starts; page_rows is the secrets store's page bound.
    #[must_use]
    pub fn new(
        application: Box<[u8]>,
        providers: Map<u32, driver::Account>,
        limits: driver::Limits,
        crypto: CryptoRng,
        page_rows: u32,
    ) -> Self {
        Self {
            driver: None,
            phase: Phase::Loading { crypto },
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            providers,
            required: Map::with_capacity(limits.accounts),
            erasing: Map::with_capacity(limits.accounts),
            records: records::Records::new(limits.accounts),
            keys: keys::Keys::new(limits.accounts, limits.client.document.token_bytes),
            finishes: finishes::Finishes::new(limits.accounts),
            finish_out: Queue::uncapped(),
            moves: moves::Moves::new(application, limits.accounts),
            source_providers: Map::with_capacity(limits.accounts),
            move_out: Queue::uncapped(),
            inventory: restart::Inventory::new(limits.accounts, page_rows),
            key_out: Queue::uncapped(),
            key_events: Queue::uncapped(),
            forgotten: Queue::uncapped(),
            early: Queue::uncapped(),
            io: Queue::uncapped(),
            above: Queue::uncapped(),
            files: Queue::uncapped(),
            aborting: false,
            closed: false,
            dispatching_move: false,
            store: StoreState {
                durable: false,
                installation: [0; 16],
                answers: Queue::uncapped(),
                requests: Queue::uncapped(),
            },
            table: table::Table::new(limits.accounts, limits.client.document.token_bytes),
            output: Queue::uncapped(),
            restoring: Map::with_capacity(limits.accounts),
        }
    }
    fn down(&mut self, request: driver::Request) {
        if let Some(component) = &mut self.driver {
            component.down(&self.env, request, &mut self.above, &mut self.io, &mut self.files);
        }
    }
    /// The domain's request: to the driver once it runs, held until then.
    fn request(&mut self, request: domain::Request) {
        match request {
            domain::Request::Component(_) | domain::Request::Signing(_) => {
                self.early.push(request);
            }
            domain::Request::Granted { .. }
            | domain::Request::Save { .. }
            | domain::Request::Erase { .. }
            | domain::Request::Answer { .. }
            | domain::Request::Programs { .. }
            | domain::Request::CheckProgram { .. }
            | domain::Request::SignOutProgram { .. }
            | domain::Request::Closed => (),
        }
    }
    fn dispatch(&mut self, request: domain::Request, store: &mut StoreState) {
        match request {
            domain::Request::Component(request) => match request {
                domain::ToComponent::Want { account } if self.keys.contains(account) => {
                    if let Some(event) = self.keys.want(account, self.env.now) {
                        self.key_events.push(event);
                    }
                }
                domain::ToComponent::Rejected { account, .. } | domain::ToComponent::Release { account }
                    if self.keys.contains(account) => {}
                domain::ToComponent::Want { account } => {
                    self.ready(account, false);
                    self.down(driver::Request::Grant { account });
                }
                domain::ToComponent::Close => {
                    self.moves.close();
                    self.down(driver::Request::Close);
                }
                other @ (domain::ToComponent::Rejected { .. } | domain::ToComponent::Release { .. }) => {
                    self.down(translate::down(other));
                }
            },
            domain::Request::Signing(signing) => self.signing(translate::down_signing(signing), store),
            domain::Request::Granted { .. }
            | domain::Request::Save { .. }
            | domain::Request::Erase { .. }
            | domain::Request::Answer { .. }
            | domain::Request::Programs { .. }
            | domain::Request::CheckProgram { .. }
            | domain::Request::SignOutProgram { .. }
            | domain::Request::Closed => (),
        }
    }
    fn restore_source(&mut self, source: u32, provider: u32, kind: domain::Kind, store: &mut StoreState) {
        self.inventory.name(source);
        self.restoring.insert(source, (provider, kind)).expect("admitted restored source");
        store.request(
            Token::new(u64::from(source)),
            store::Secrets::Get { owner: store::SecretOwner::Source, number: u64::from(source) },
        );
    }
    fn restored_value(&mut self, source: u32, value: Option<store::SecretValue>, values: &mut table::Table) {
        let (provider, kind) = self.restoring.remove(&source).expect("pending source restore");
        let expected = match kind {
            domain::Kind::Key => SecretKind::Key,
            domain::Kind::SignIn => SecretKind::Record,
            domain::Kind::Program | domain::Kind::None => return,
        };
        let Ok(value) = self.secret(value, expected) else { return };
        match kind {
            domain::Kind::Key => self.keys.restore(source, value, values).expect("restored key fits"),
            domain::Kind::SignIn => self.restore_signed_in(source, provider, value),
            domain::Kind::Program | domain::Kind::None => (),
        }
    }
    fn signing(&mut self, signing: translate::Down, store: &mut StoreState) {
        match signing {
            translate::Down::Restore { source, provider, kind } => {
                self.restore_source(source, provider, kind, store);
            }
            translate::Down::Withdraw { source, display } => {
                self.dispatching_move = true;
                self.ready(source, false);
                let next = self.moves.withdraw(source, display);
                self.move_out.push(next);
            }
            translate::Down::Return { source } => {
                self.dispatching_move = true;
                let next = self.moves.returning(source);
                self.move_out.push(next);
            }
            translate::Down::Refresh { source } => {
                self.ready(source, true);
            }
            translate::Down::Open { source, carried } => {
                self.dispatching_move = true;
                let next = self.moves.open(source, carried);
                self.move_out.push(next);
            }
            translate::Down::Adopt { source, carried } => {
                self.dispatching_move = true;
                let next = self.moves.adopt(source, carried, &self.env.limits.client.document);
                self.move_out.push(next);
            }
            translate::Down::Restored => {
                self.inventory.restored();
            }
            translate::Down::Keep { source, carried, replaces } => {
                self.inventory.name(source);
                let next = self.keys.keep(source, carried, replaces);
                self.key_out.push(next);
            }
            translate::Down::Forget { source } => match self.moves.forget(source) {
                moves::Forget::Pending => (),
                moves::Forget::Withdrawn => {
                    self.records.remove(source);
                    let _required = self.required.remove(&source);
                    let _provider = self.source_providers.remove(&source);
                    self.inventory.forget(source);
                    self.forgotten.push(source);
                }
                moves::Forget::Source => {
                    if self.required.contains_key(&source) {
                        self.down(driver::Request::Remove { account: source });
                    }
                    self.keys.forget(source);
                    self.inventory.forget(source);
                    self.forgotten.push(source);
                }
            },
            translate::Down::Redirected { source, carried } => {
                let next = self.finishes.redirected(source, carried);
                self.finish_out.push(next);
            }
            translate::Down::CancelSignIn { source } => self.down(driver::Request::Cancel { account: source }),
            translate::Down::SignOut { source } => {
                self.ready(source, false);
                self.down(driver::Request::SignOut { account: source });
            }
            translate::Down::SignIn { source, provider } => {
                let Some(template) = self.providers.get(&provider) else {
                    self.above.push(driver::Event::Refused {
                        account: source,
                        asked: driver::Asked::Add,
                        why: driver::Refusal::Source,
                    });
                    return;
                };
                let account = unkept(template);
                let required = required_scopes(&account);
                let before = self.above.len();
                self.driver.as_mut().expect("running driver").add(&self.env, source, account, &mut self.above);
                if self.above.len() != before {
                    return;
                }
                self.inventory.name(source);
                self.source_providers.insert(source, provider).expect("admitted source");
                self.required.insert(source, required).expect("driver admitted account");
                self.records.admit(source).expect("driver admitted account");
                self.down(driver::Request::SignIn { account: source, subject: None });
            }
        }
    }
    fn clock(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
    }
    fn up(&mut self, event: io::Event) {
        if let Some(component) = &mut self.driver {
            component.up(&self.env, event, &mut self.above, &mut self.io, &mut self.files);
        }
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        (self.aborting || (!self.inventory.active() && !self.keys.pending() && !self.finishes.pending()))
            && self.move_out.is_empty()
            && self.finish_out.is_empty()
            && self.key_out.is_empty()
            && self.key_events.is_empty()
            && self.forgotten.is_empty()
            && self.closed
            && self.io.is_empty()
            && self.above.is_empty()
            && self.files.is_empty()
            && self.early.is_empty()
            && self.output.is_empty()
            && self.store.answers.is_empty()
            && self.store.requests.is_empty()
            && self.restoring.is_empty()
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        if self.closed {
            return None;
        }
        let driver = match &self.driver {
            Some(component) => component.next_deadline(),
            None => None,
        };
        match self.table.next_expiry() {
            Some(expiry) => Some(match driver {
                Some(time) => time.min(expiry),
                None => expiry,
            }),
            None => driver,
        }
    }
    fn work_pending(&self, now: Time, marked: bool) -> bool {
        let starting = match &self.phase {
            // The driver cannot start until the store acknowledges the mark.
            // Claiming work before then keeps delayed io from completing.
            Phase::Loading { .. } => marked,
            Phase::Running => !self.early.is_empty() && !self.dispatching_move && self.restoring.is_empty(),
        };
        starting
            || self.inventory.work_pending()
            || !self.move_out.is_empty()
            || !self.finish_out.is_empty()
            || !self.key_out.is_empty()
            || !self.key_events.is_empty()
            || !self.forgotten.is_empty()
            || !self.above.is_empty()
            || !self.io.is_empty()
            || !self.files.is_empty()
            || match &self.driver {
                Some(component) => component.has_work(),
                None => false,
            }
            || match self.next_deadline() {
                Some(deadline) => deadline <= now,
                None => false,
            }
    }
    fn abort(&mut self) {
        self.aborting = true;
        self.dispatching_move = false;
        self.restoring = Map::with_capacity(self.env.limits.accounts);
        self.finishes.abort();
        for _ in 0..self.finish_out.len() {
            let _unasked = self.finish_out.pop();
        }
        self.moves.close();
        for _ in 0..self.move_out.len() {
            let _unasked = self.move_out.pop();
        }
        if self.driver.is_some() {
            self.down(driver::Request::Abort);
        } else {
            self.closed = true;
        }
        self.phase = Phase::Running;
        for _ in 0..self.early.len() {
            let _unasked = self.early.pop();
        }
        for _ in 0..self.key_out.len() {
            let _unkept = self.key_out.pop();
        }
        for _ in 0..self.key_events.len() {
            let _unheard = self.key_events.pop();
        }
        for _ in 0..self.forgotten.len() {
            let _unforgotten = self.forgotten.pop();
        }
    }
    fn iterate(
        &mut self,
        now: Time,
        wall: Wall,
        values: &mut table::Table,
        events: &mut Queue<domain::Event>,
        store: &mut StoreState,
    ) {
        self.clock(now, wall);
        self.answers(store, values, events);
        self.start(store);
        self.move_steps(events, store);
        self.key_steps(values, events, store);
        self.finish_steps(events, store);
        self.sweep(store);
        self.fire_driver();
        self.driver_events(values, events, store);
        assert!(self.files.is_empty(), "owner keepers file nothing");
        values.expire(now);
        if let Some(component) = &mut self.driver {
            component.reclaim();
        }
    }
    fn driver_events(&mut self, values: &mut table::Table, events: &mut Queue<domain::Event>, store: &mut StoreState) {
        let now = self.env.now;
        for _ in 0..self.above.len() {
            let event = self.above.pop().expect("account events");
            let number = event_account(&event);
            let required = match number {
                Some(number) => self.required.get(&number),
                None => None,
            };
            let required = match required {
                Some(value) => &**value,
                None => b"",
            };
            let event = match self.records.up(event, now, required) {
                translate::Up::Granted { mut event, account, generation, value, expires } => {
                    let granted_form = self.granted(account, generation, value, expires, values);
                    match &mut event {
                        domain::FromComponent::Granted { form, .. } => *form = granted_form,
                        domain::FromComponent::Expiring { .. }
                        | domain::FromComponent::KeepFailed { .. }
                        | domain::FromComponent::Failed { .. }
                        | domain::FromComponent::Closed => unreachable!("a lent value accompanies its grant"),
                    }
                    event
                }
                translate::Up::Child(event) => event,
                translate::Up::Revoked { source } => {
                    values.remove(source);
                    self.output.push(Effect::Removed { source });
                    self.erase(source, Erasing::Revoked, store);
                    domain::FromComponent::Failed { account: source, failure: domain::Failure::SignInAgain }
                }
                translate::Up::Finishing { source } => {
                    events.push(domain::Event::Finishing { source });
                    continue;
                }
                translate::Up::FinishRefused { source, why } => {
                    events.push(domain::Event::FinishRefused { source, why });
                    continue;
                }
                translate::Up::Visited { source, address } => {
                    events.push(domain::Event::Visited { source, address });
                    continue;
                }
                translate::Up::SignedIn { source, subject, display } => {
                    events.push(domain::Event::SignedIn { source, subject, display, declined: false });
                    continue;
                }
                translate::Up::SignInFailed { source, failure } => {
                    events.push(domain::Event::SignInFailed { source, failure });
                    // Every failed account sign-in gives its driver place back.
                    if self.required.contains_key(&source) {
                        self.down(driver::Request::SignOut { account: source });
                    }
                    continue;
                }
                translate::Up::SignedOut { source, confirmed } => {
                    self.erase(source, Erasing::SignOut { confirmed }, store);
                    continue;
                }
                translate::Up::Removed { source } => {
                    self.removed(source, values);
                    continue;
                }
                translate::Up::Keep { account, record } => {
                    if self.aborting {
                        continue;
                    }
                    match self.records.keep(account, record.0, &self.env.limits.client.document) {
                        records::Next::Driver(request) => self.down(request),
                        records::Next::Store(keep) => replace(store, keep),
                    }
                    continue;
                }
            };
            match &event {
                domain::FromComponent::Granted { .. }
                | domain::FromComponent::Expiring { .. }
                | domain::FromComponent::Failed { .. }
                | domain::FromComponent::KeepFailed { .. } => {}
                domain::FromComponent::Closed => self.closed = true,
            }
            events.push(domain::Event::Component(event));
        }
    }
    fn removed(&mut self, source: u32, values: &mut table::Table) {
        self.output.push(Effect::SourceRemoved { source });
        if let Some(next) = self.moves.removed(source) {
            self.move_out.push(next);
            return;
        }
        let _erase = self.erasing.remove(&source);
        let _required = self.required.remove(&source);
        let _provider = self.source_providers.remove(&source);
        self.records.remove(source);
        self.inventory.forget(source);
        values.remove(source);
        self.output.push(Effect::Removed { source });
    }
    fn fire_driver(&mut self) {
        if let Some(component) = &mut self.driver {
            component.fire(&self.env, &mut self.above, &mut self.io, &mut self.files);
        }
    }
    fn restore_signed_in(&mut self, source: u32, provider: u32, value: Option<Secret>) {
        let Some(template) = self.providers.get(&provider) else {
            self.key_events.push(domain::FromComponent::Failed {
                account: source,
                failure: domain::Failure::Unloaded { at: domain::Place::File, why: domain::Unread::Unreadable },
            });
            return;
        };
        let account =
            party::restore(unkept(template), source, value, &mut self.records, &self.env.limits.client.document);
        self.required.insert(source, required_scopes(&account)).expect("restored source fits driver");
        self.source_providers.insert(source, provider).expect("restored source fits driver");
        self.moves.stage(source, account);
    }
    fn ready(&mut self, source: u32, refresh: bool) {
        if let Some(account) = self.moves.ready(source, refresh) {
            self.driver.as_mut().expect("running driver").add(&self.env, source, account, &mut self.above);
        }
    }
    fn move_steps(&mut self, events: &mut Queue<domain::Event>, store: &mut StoreState) {
        if self.aborting {
            return;
        }
        for _ in 0..self.move_out.len() {
            match self.move_out.pop().expect("move steps") {
                moves::Next::Driver(request) => self.down(request),
                moves::Next::Child(event) => {
                    self.dispatching_move = false;
                    events.push(event);
                }
                moves::Next::Store { source, request } => {
                    let (prefix, request) = match request {
                        moves::Store::ReadOut => (
                            MOVE_OUT,
                            store::Secrets::Get { owner: store::SecretOwner::Source, number: u64::from(source) },
                        ),
                        moves::Store::ReadBack => (
                            MOVE_BACK,
                            store::Secrets::Get { owner: store::SecretOwner::Source, number: u64::from(source) },
                        ),
                        moves::Store::ReadIn { carried } => {
                            (MOVE_IN, store::Secrets::Get { owner: store::SecretOwner::Application, number: carried })
                        }
                        moves::Store::AdoptValue { carried, holds, value } => (
                            MOVE_ADOPT,
                            store::Secrets::AdoptValue {
                                carried,
                                holds,
                                owner: store::SecretOwner::Source,
                                number: u64::from(source),
                                value: store::SecretValue::Record(value),
                                replaces: None,
                            },
                        ),
                    };
                    store.request(Token::new(prefix | u64::from(source)), request);
                }
            }
        }
    }
    fn move_answer(&mut self, event: store::Event, events: &mut Queue<domain::Event>) {
        if self.aborting {
            return;
        }
        match event {
            store::Event::Page { .. } | store::Event::PageRefused { .. } => {
                unreachable!("a start's pages need a start-mode store")
            }
            store::Event::Secret { owner, value } if owner.raw() & MOVE_OUT == MOVE_OUT => {
                let source = move_source(owner);
                let Ok(value) = self.secret(value, SecretKind::Record) else {
                    return;
                };
                let provider = self.source_providers.get(&source).expect("withdrawn source provider");
                let registration = self.providers.get(provider).expect("source provider");
                let next = self.moves.read(source, value, registration, &self.env.limits.client.document);
                self.move_out.push(next);
            }
            store::Event::Secret { owner, value } if owner.raw() & MOVE_BACK == MOVE_BACK => {
                let source = move_source(owner);
                let provider = *self.source_providers.get(&source).expect("returning source provider");
                let Ok(value) = self.secret(value, SecretKind::Record) else {
                    return;
                };
                self.restore_signed_in(source, provider, value);
                self.dispatching_move = false;
            }
            store::Event::Secret { owner, value } => {
                let Ok(value) = self.secret(value, SecretKind::Carried(store::CarriedKind::Moved)) else {
                    return;
                };
                let next =
                    self.moves.opened(move_source(owner), value, &self.providers, &self.env.limits.client.document);
                self.move_out.push(next);
            }
            store::Event::Kept { owner } => {
                let source = move_source(owner);
                self.dispatching_move = false;
                if let Some(adoption) = self.moves.adopted(source, true) {
                    self.inventory.name(source);
                    self.restore_signed_in(source, adoption.provider, Some(adoption.value));
                    events.push(domain::Event::Adopted { source, kept: true });
                }
            }
            store::Event::Stale { owner } => {
                let source = move_source(owner);
                self.dispatching_move = false;
                let _unkept = self.moves.adopted(source, false);
                events.push(domain::Event::Adopted { source, kept: false });
            }
            store::Event::Switched { .. }
            | store::Event::Opened { .. }
            | store::Event::OpenRefused { .. }
            | store::Event::Site { .. }
            | store::Event::Patched { .. }
            | store::Event::Committed { .. }
            | store::Event::Failed { .. }
            | store::Event::FrameTooLarge { .. }
            | store::Event::Loaded { .. }
            | store::Event::Read { .. }
            | store::Event::Got { .. }
            | store::Event::Turns { .. }
            | store::Event::Unread { .. }
            | store::Event::Carried { .. }
            | store::Event::Erased { .. }
            | store::Event::SecretsLoaded { .. }
            | store::Event::SecretsNotReady { .. }
            | store::Event::Corrupt { .. } => unreachable!("move exchange answers read or adoption"),
        }
    }
    fn sweep_loaded(&mut self, rows: Box<[store::SecretRow]>, next: Option<u64>) {
        for row in rows {
            if row.owner == store::SecretOwner::Source {
                self.inventory.row(row.number);
            }
        }
        self.inventory.loaded(next);
    }
    fn sweep(&mut self, store: &mut StoreState) {
        if self.aborting || !store.durable {
            return;
        }
        let Some(step) = self.inventory.step() else { return };
        let request = match step {
            restart::Sweep::Load { after } => store::Secrets::Load { owner: store::SecretOwner::Source, after },
            restart::Sweep::Erase { number } => store::Secrets::Erase { owner: store::SecretOwner::Source, number },
        };
        store.request(SWEEP, request);
    }

    fn key_steps(&mut self, values: &mut table::Table, events: &mut Queue<domain::Event>, store: &mut StoreState) {
        if self.aborting {
            return;
        }
        for _ in 0..self.forgotten.len() {
            let source = self.forgotten.pop().expect("forgotten LLM key");
            values.remove(source);
            self.output.push(Effect::Removed { source });
            store.request(
                Token::new(u64::from(source)),
                store::Secrets::Erase { owner: store::SecretOwner::Source, number: u64::from(source) },
            );
        }
        for _ in 0..self.key_out.len() {
            match self.key_out.pop().expect("LLM key steps") {
                keys::Next::Store { exchange, request } => {
                    let owner = Token::new(KEY_EXCHANGE | exchange.raw());
                    let request = match request {
                        keys::Store::Get { carried } => {
                            store::Secrets::Get { owner: store::SecretOwner::Application, number: carried }
                        }
                        keys::Store::Adopt { carried, source, replaces } => store::Secrets::Adopt {
                            carried,
                            owner: store::SecretOwner::Source,
                            number: u64::from(source),
                            replaces,
                        },
                    };
                    store.request(owner, request);
                }

                keys::Next::Child(event) => {
                    match &event {
                        domain::FromComponent::KeepFailed { account, .. } => {
                            if !self.keys.contains(*account) {
                                self.inventory.forget(*account);
                            }
                        }
                        domain::FromComponent::Granted { .. }
                        | domain::FromComponent::Expiring { .. }
                        | domain::FromComponent::Failed { .. }
                        | domain::FromComponent::Closed => (),
                    }
                    events.push(domain::Event::Component(event));
                }
                keys::Next::Granted { event, source, generation, value } => {
                    self.output.push(Effect::Granted {
                        source,
                        generation,
                        value: value.clone(),
                        expires: Time::from_nanos(u64::MAX),
                    });
                    values
                        .granted(source, generation, value, Time::from_nanos(u64::MAX))
                        .expect("admitted LLM key fits table");
                    events.push(domain::Event::Component(event));
                }
            }
        }
        for _ in 0..self.key_events.len() {
            let event = self.key_events.pop().expect("LLM key grant events");
            match &event {
                domain::FromComponent::Granted { account, generation, .. } => {
                    if let Some(value) = values.value(*account, *generation, self.env.now) {
                        self.output.push(Effect::Granted {
                            source: *account,
                            generation: *generation,
                            value: value.clone(),
                            expires: Time::from_nanos(u64::MAX),
                        });
                    }
                }
                domain::FromComponent::Expiring { .. }
                | domain::FromComponent::KeepFailed { .. }
                | domain::FromComponent::Failed { .. }
                | domain::FromComponent::Closed => (),
            }
            events.push(domain::Event::Component(event));
        }
    }
    fn finish_steps(&mut self, events: &mut Queue<domain::Event>, store: &mut StoreState) {
        for _ in 0..self.finish_out.len() {
            match self.finish_out.pop().expect("finish steps") {
                finishes::Next::Get { exchange, carried } => {
                    store.request(
                        Token::new(FINISH_EXCHANGE | exchange.raw()),
                        store::Secrets::Get { owner: store::SecretOwner::Application, number: carried },
                    );
                }

                finishes::Next::Driver(request) => self.down(request),
                finishes::Next::Child(event) => events.push(event),
            }
        }
    }
    fn granted(
        &mut self,
        account: u32,
        generation: u64,
        value: table::Value,
        expires: Time,
        values: &mut table::Table,
    ) -> domain::Form {
        let form = translate::form(&value.scheme);
        self.output.push(Effect::Granted { source: account, generation, value: value.clone(), expires });
        values.granted(account, generation, value, expires).expect("startup checks token bounds");
        form
    }
    fn erase(&mut self, source: u32, reason: Erasing, store: &mut StoreState) {
        if let Some(pending) = self.erasing.get_mut(&source) {
            match pending {
                Erasing::Revoked => match reason {
                    Erasing::SignOut { confirmed } => *pending = Erasing::SignOut { confirmed },
                    Erasing::Revoked | Erasing::Removing => (),
                },
                Erasing::SignOut { .. } | Erasing::Removing => (),
            }
            return;
        }
        self.erasing.insert(source, reason).expect("one erase per driver account");
        store.request(
            Token::new(u64::from(source)),
            store::Secrets::Erase { owner: store::SecretOwner::Source, number: u64::from(source) },
        );
    }

    fn erased(&mut self, number: u32, _owner: Token, events: &mut Queue<domain::Event>) {
        let Some(pending) = self.erasing.remove(&number) else { return };
        match pending {
            Erasing::Revoked => self.records.erased(number),
            Erasing::SignOut { confirmed } => {
                self.records.erased(number);
                events.push(domain::Event::SignedOut { source: number, confirmed });
                if self.required.contains_key(&number) {
                    self.erasing.insert(number, Erasing::Removing).expect("the removed erase's place");
                    self.down(driver::Request::Remove { account: number });
                }
            }
            Erasing::Removing => {
                self.erasing.insert(number, Erasing::Removing).expect("the removed erase's place");
            }
        }
    }
    /// The store's answers: the records it holds, and whether each write was
    /// kept (store.md 9).
    fn answers(&mut self, store: &mut StoreState, values: &mut table::Table, events: &mut Queue<domain::Event>) {
        for _ in 0..store.answers.len() {
            let answer = store.answers.pop().expect("source answers");
            if self.aborting {
                continue;
            }
            match answer {
                store::Event::Page { .. } | store::Event::PageRefused { .. } => {
                    unreachable!("a start's pages need a start-mode store")
                }
                store::Event::Secret { owner, value }
                    if u32::try_from(owner.raw()).is_ok() && self.restoring.contains_key(&source(owner)) =>
                {
                    self.restored_value(source(owner), value, values);
                }
                store::Event::Secret { owner, value } if owner.raw() & FINISH_EXCHANGE == FINISH_EXCHANGE => {
                    let Ok(value) = self.secret(value, SecretKind::Held) else {
                        return;
                    };
                    if !self.aborting
                        && let Some(next) = self.finishes.read(Token::new(owner.raw() & !FINISH_EXCHANGE), value)
                    {
                        self.finish_out.push(next);
                    }
                }
                store::Event::Secret { owner, value } if owner.raw() & KEY_EXCHANGE == KEY_EXCHANGE => {
                    let Ok(value) = self.secret(value, SecretKind::Carried(store::CarriedKind::Key)) else {
                        return;
                    };
                    if !self.aborting
                        && let Some(next) = self.keys.read(Token::new(owner.raw() & !KEY_EXCHANGE), value)
                    {
                        self.key_out.push(next);
                    }
                }
                event @ store::Event::Secret { .. } => self.move_answer(event, events),
                event @ (store::Event::Kept { owner } | store::Event::Stale { owner })
                    if owner.raw() & MOVE_ADOPT == MOVE_ADOPT =>
                {
                    self.move_answer(event, events);
                }
                store::Event::SecretsLoaded { owner: SWEEP, rows, next } => self.sweep_loaded(rows, next),
                store::Event::SecretsLoaded { .. } => unreachable!("only the source inventory loads pages"),
                store::Event::Kept { owner } if owner.raw() & KEY_EXCHANGE == KEY_EXCHANGE => {
                    if !self.aborting
                        && let Some(next) = self.keys.kept(Token::new(owner.raw() & !KEY_EXCHANGE), true, self.env.now)
                    {
                        self.key_out.push(next);
                    }
                }
                store::Event::Stale { owner } if owner.raw() & KEY_EXCHANGE == KEY_EXCHANGE => {
                    if !self.aborting
                        && let Some(next) = self.keys.kept(Token::new(owner.raw() & !KEY_EXCHANGE), false, self.env.now)
                    {
                        self.key_out.push(next);
                    }
                }
                store::Event::Kept { owner } => {
                    if let Some(request) = self.records.kept(source(owner)) {
                        self.down(request);
                    }
                }
                store::Event::Stale { owner } => {
                    if let Some(request) = self.records.stale(source(owner)) {
                        self.down(request);
                    }
                }
                store::Event::Erased { owner: SWEEP } => self.inventory.erased(),
                store::Event::Erased { owner } => self.erased(source(owner), owner, events),
                store::Event::Carried { .. } => {
                    unreachable!("the accounts only load and replace their records")
                }
                store::Event::Switched { .. }
                | store::Event::Opened { .. }
                | store::Event::OpenRefused { .. }
                | store::Event::Site { .. }
                | store::Event::Patched { .. }
                | store::Event::Committed { .. }
                | store::Event::Failed { .. }
                | store::Event::FrameTooLarge { .. }
                | store::Event::Loaded { .. }
                | store::Event::Read { .. }
                | store::Event::Got { .. }
                | store::Event::Turns { .. }
                | store::Event::Unread { .. }
                | store::Event::SecretsNotReady { .. }
                | store::Event::Corrupt { .. } => {
                    unreachable!("the store hands the accounts only their secrets' answers")
                }
            }
        }
    }
    /// The durable installation identity replaces this start's draw before any restore or sign-in.
    fn opened(&mut self, kept: &[u8; 16]) {
        let crypto = match core::mem::replace(&mut self.phase, Phase::Running) {
            Phase::Loading { crypto } => crypto,
            Phase::Running => unreachable!("driver starts once"),
        };
        let component = driver::Component::new(List::with_capacity(0), &self.env.limits, crypto)
            .expect("startup checked driver bounds");
        // The stored installation id, rather than this start's draw, goes on account sign-ins.
        let mut after = 0;
        for _ in 0..self.providers.len() {
            let Some((&number, _)) = self.providers.range(after..).next() else { break };
            let account = self.providers.get_mut(&number).expect("named provider");
            match account {
                driver::Account::SignIn { registration, .. } => {
                    if let Some(terms) = &mut registration.terms {
                        terms.parameters.first_mut().expect("installation parameter").value = installation::urn(kept);
                    }
                }
                driver::Account::HandedIn => unreachable!("provider template is a sign-in"),
            }
            let Some(next) = number.checked_add(1) else { break };
            after = next;
        }
        self.driver = Some(component);
    }
    /// Wait for the durable start mark, then route the child's restored sources and runtime requests.
    fn start(&mut self, store: &mut StoreState) {
        match &mut self.phase {
            Phase::Loading { .. } => {
                if store.durable {
                    self.opened(&store.installation);
                }
            }
            Phase::Running => {
                for _ in 0..self.early.len() {
                    // Dispatch the previous move before another entrance.
                    if self.dispatching_move || !self.move_out.is_empty() || !self.restoring.is_empty() {
                        break;
                    }
                    let request = self.early.pop().expect("early requests");
                    self.dispatch(request, store);
                }
            }
        }
    }
}
fn replace(store: &mut StoreState, keep: records::Keep) {
    let records::Keep { account, value, replaces } = keep;
    store.request(
        Token::new(u64::from(account)),
        store::Secrets::Replace {
            owner: store::SecretOwner::Source,
            number: u64::from(account),
            value: store::SecretValue::Record(value),
            replaces,
        },
    );
}
fn move_source(owner: Token) -> u32 {
    u32::try_from(owner.raw() & u64::from(u32::MAX)).expect("source number")
}
fn source(owner: Token) -> u32 {
    u32::try_from(owner.raw()).expect("source number")
}
fn copy_endpoint(endpoint: &driver::Endpoint) -> driver::Endpoint {
    driver::Endpoint {
        address: endpoint.address,
        transport: match &endpoint.transport {
            driver::Transport::Plaintext => driver::Transport::Plaintext,
            driver::Transport::Tls { server_name, trust } => {
                driver::Transport::Tls { server_name: server_name.clone(), trust: trust.clone() }
            }
        },
    }
}
fn copy_origins(origins: &List<driver::Origin>) -> List<driver::Origin> {
    let mut copy = List::with_capacity(origins.len());
    for origin in origins {
        copy.push(driver::Origin { uri: origin.uri.clone(), endpoint: copy_endpoint(&origin.endpoint) })
            .expect("configured origins");
    }
    copy
}
fn event_account(event: &driver::Event) -> Option<u32> {
    match event {
        driver::Event::Granted { account, .. }
        | driver::Event::Expiring { account, .. }
        | driver::Event::Failed { account, .. }
        | driver::Event::Refused { account, .. }
        | driver::Event::Visit { account, .. }
        | driver::Event::Taken { account }
        | driver::Event::SignedIn { account, .. }
        | driver::Event::Keep { account, .. }
        | driver::Event::Removed { account }
        | driver::Event::SignedOut { account, .. }
        | driver::Event::Lock { account }
        | driver::Event::Unlock { account } => Some(*account),
        driver::Event::Closed => None,
    }
}

impl Component {
    pub fn step(&mut self, env: &Env<driver::Limits>, event: Event, out: &mut Queue<Effect>) {
        self.clock(env.now, env.wall);
        match event {
            Event::Request(request) => self.request(request),
            Event::Io(event) => self.up(event),
            Event::Store(event) if self.aborting => {
                drop(event);
            }
            Event::Store(event) => self.store.answers.push(event),
            Event::Opened { installation } => {
                self.store.installation = installation;
                self.store.durable = true;
            }
            Event::Abort => self.abort(),
        }
        self.publish(out);
    }
    pub fn fire(&mut self, env: &Env<driver::Limits>, out: &mut Queue<Effect>) {
        let mut store = core::mem::replace(
            &mut self.store,
            StoreState {
                durable: false,
                installation: [0; 16],
                answers: Queue::uncapped(),
                requests: Queue::uncapped(),
            },
        );
        let mut table = core::mem::replace(&mut self.table, table::Table::new(0, 0));
        let mut events = Queue::uncapped();
        self.iterate(env.now, env.wall, &mut table, &mut events, &mut store);
        for _ in 0..events.len() {
            self.output.push(Effect::Child(events.pop().expect("child events")));
        }
        for _ in 0..store.requests.len() {
            let effect = store.requests.pop().expect("store requests");
            if !self.aborting {
                self.output.push(effect);
            }
        }
        self.table = table;
        self.store = store;
        self.publish(out);
    }
    fn publish(&mut self, out: &mut Queue<Effect>) {
        for _ in 0..self.output.len() {
            out.push(self.output.pop().expect("component effects"));
        }
        for _ in 0..self.io.len() {
            out.push(Effect::Io(self.io.pop().expect("driver io")));
        }
    }
    #[must_use]
    pub fn value(&self, source: u32, generation: u64, now: Time) -> Option<&table::Value> {
        self.table.value(source, generation, now)
    }
    #[must_use]
    pub fn has_work(&self, now: Time) -> bool {
        self.work_pending(now, self.store.durable) || !self.store.answers.is_empty()
    }
}

#[cfg(test)]
#[path = "component_tests.rs"]
mod tests;
