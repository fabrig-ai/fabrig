//! The accounts' host and its pass over store answers and effects
//! (jig/docs/design/protocol/service.md, 11.1).
use crate::{
    Component,
    component::{Effect, Event, unkept},
    party,
    table::Table,
};
use alloc::boxed::Box;
use jig_domain_accounts as domain;
use jig_protocol_config::{self as config, Path, Problems, What};
use jig_protocol_store as store;
use skein_io as io;
use skein_lib::{CryptoRng, Env, List, Map, Queue, Secret, Time, Token, Wall};
use skein_oauth_accounts as driver;

/// What a process's shell prepares for sign-ins.
#[derive(Debug)]
pub struct Runtime {
    /// LLM account sign-in endpoints and origins, keyed by provider number.
    pub providers: List<(u32, driver::Endpoint, List<driver::Origin>)>,
    /// Bounds of all prepared trust roots, supplied by their shell owner; `None` checks nothing.
    pub trust: Option<TrustBounds>,
}
/// Bounds of the trust roots a shell prepared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrustBounds {
    pub roots: u32,
    pub bytes: u32,
}
/// What the host can't decide. The owner takes each before it asks for the next.
#[expect(missing_debug_implementations, reason = "a store request carries secrets' values")]
pub enum Output {
    /// An event for the root, in the component's order.
    Account(domain::Event),
    /// A request to the secrets' store, on the host's exchange token; the owner tags it.
    Store { exchange: Token, request: store::Secrets },
    /// A restored secret of the wrong kind: the owner says what that means.
    InvalidSecret,
}
fn problem(what: What) -> Problems {
    let mut found = Problems::empty();
    found.push(Path::root().member(b"engine").member(b"llm").member(b"providers"), what);
    found
}
fn registration_input(
    configured: &config::llm::Registration,
    endpoint_transport: &driver::Transport,
) -> party::Registration {
    party::Registration {
        issuer: configured.issuer.clone(),
        client: configured.client.clone(),
        scopes: configured.scopes.clone(),
        name_hint: configured.name_hint.clone(),
        authorization: configured.authorization.clone(),
        token: configured.token.clone(),
        revocation: configured.revocation.clone(),
        keys: configured.keys.clone(),
        endpoint_transport: match endpoint_transport {
            driver::Transport::Tls { server_name, trust } => {
                driver::Transport::Tls { server_name: server_name.clone(), trust: trust.clone() }
            }
            driver::Transport::Plaintext => driver::Transport::Plaintext,
        },
        must_grant: configured.must_grant.clone(),
        resource: configured.resource.clone(),
        redirect_path: configured.redirect_path.clone(),
        redirect_port: configured.redirect_port,
    }
}
fn provider_templates(
    config: &config::Engine,
    runtime: Option<Runtime>,
    limits: &driver::Limits,
    installation_id: &[u8; 16],
) -> Result<Map<u32, driver::Account>, Problems> {
    let mut providers = Map::with_capacity(u32::try_from(config.llm.providers.len()).expect("provider count"));
    if let Some(Runtime { providers: prepared, trust }) = runtime {
        for (number, endpoint, origins) in prepared.into_boxed() {
            let mut registration = None;
            for provider in &config.llm.providers {
                if provider.number == number {
                    registration = provider.sign_in.as_ref();
                    break;
                }
            }
            let registration = registration.ok_or(problem(What::Refers))?;
            if untrusted(trust, has_tls(&endpoint, &origins)) {
                return Err(problem(What::Bound));
            }
            let account = party::party_account(
                &registration_input(registration, &endpoint.transport),
                endpoint,
                origins,
                installation_id,
            );
            let mut check = List::with_capacity(1);
            check.push(unkept(&account)).or(Err(problem(What::Bound)))?;
            driver::Component::check(check, limits).or(Err(problem(What::Form)))?;
            providers.insert(number, account).or(Err(problem(What::Bound)))?;
        }
    }
    for provider in &config.llm.providers {
        if provider.sign_in.is_some() && !providers.contains_key(&provider.number) {
            return Err(problem(What::Refers));
        }
    }
    Ok(providers)
}

/// The accounts component, its io and one ordered pass over its effects.
#[expect(missing_debug_implementations, reason = "the accounts component holds token records")]
#[expect(clippy::partial_pub_fields, reason = "the owner submits the host's io requests")]
pub struct Accounts {
    component: Component,
    env: Env<driver::Limits>,
    /// Requests for the owner to tag and submit, as its room allows.
    pub io: Queue<io::Request>,
    effects: Queue<Effect>,
    opened: bool,
    held: Option<domain::Event>,
    aborting: bool,
}
impl Accounts {
    /// Checks public provider registrations; restored sources are admitted after the store opens.
    pub fn new(
        application: Box<[u8]>,
        config: &config::Engine,
        crypto: &Secret,
        installation: &[u8; 16],
        runtime: Option<Runtime>,
        secrets: &skein_kv::Limits,
    ) -> Result<Self, Problems> {
        let limits = config.limits.driver;
        if limits.client.document.record_bytes > secrets.value {
            return Err(problem(What::Bound));
        }
        let crypto = CryptoRng::new(crypto, 1).ok_or(problem(What::Bound))?;
        let providers = provider_templates(config, runtime, &limits, installation)?;
        Ok(Self::made(Component::new(application, providers, limits, crypto, secrets.page.rows), limits))
    }
    #[expect(clippy::large_types_passed_by_value, reason = "the host owns its limits")]
    fn made(component: Component, limits: driver::Limits) -> Self {
        Self {
            component,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            io: Queue::uncapped(),
            effects: Queue::uncapped(),
            opened: false,
            held: None,
            aborting: false,
        }
    }
    /// The component's reads for an owner's diagnostics and tests.
    #[must_use]
    pub fn component(&self) -> &Component {
        &self.component
    }
    /// Whether the durable start has reached the component.
    #[must_use]
    pub fn opened(&self) -> bool {
        self.opened
    }
    /// The effects waiting for the owner.
    #[must_use]
    pub fn effects(&self) -> u32 {
        self.effects.len()
    }
    /// Whether an event waits for room in the owner's root.
    #[must_use]
    pub fn held(&self) -> bool {
        self.held.is_some()
    }
    /// The owner's browser location for a sign-in callback.
    pub fn return_to(&mut self, source: u32, location: Box<[u8]>) {
        self.component.step(&self.env, Event::Return { source, location }, &mut self.effects);
    }
    /// Takes the bytes of a withdrawn source.
    #[must_use]
    pub fn take_moved(&mut self, source: u32) -> Option<Secret> {
        self.component.take_moved(source)
    }
    /// Sets the time for the component.
    pub fn clock(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
    }
    /// Hands the component an accounts request.
    pub fn request(&mut self, request: domain::Request) {
        self.component.step(&self.env, Event::Request(request), &mut self.effects);
    }
    /// Hands the component an untagged io event.
    pub fn up(&mut self, event: io::Event) {
        self.component.step(&self.env, Event::Io(event), &mut self.effects);
    }
    /// Opens nothing and drops store requests while the owner closes.
    pub fn abort(&mut self) {
        self.aborting = true;
        self.component.step(&self.env, Event::Abort, &mut self.effects);
    }
    /// No component work, effect, held event or io remains.
    #[must_use]
    pub fn idle(&self) -> bool {
        self.component.is_empty() && self.effects.is_empty() && self.held.is_none() && self.io.is_empty()
    }
    /// The next time the component needs a pass.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.component.next_deadline()
    }
    /// `marked`: the store's start mark is durable. An owner whose own gate waits for it passes `true`.
    #[must_use]
    pub fn work_pending(&self, now: Time, marked: bool) -> bool {
        (marked && !self.opened && !self.aborting)
            || self.component.has_work(now)
            || !self.effects.is_empty()
            || self.held.is_some()
            || !self.io.is_empty()
    }
    /// A pass's first three stages, in order: `Opened` once `marked`, each of `answers`, the component's fire.
    /// Returns the pass's count, for `output`.
    #[must_use]
    pub fn begin(
        &mut self,
        now: Time,
        wall: Wall,
        marked: bool,
        installation: [u8; 16],
        answers: &mut Queue<store::Event>,
    ) -> u32 {
        self.clock(now, wall);
        if marked && !self.opened && !self.aborting {
            self.component.step(&self.env, Event::Opened { installation }, &mut self.effects);
            self.opened = true;
        }
        for _ in 0..answers.len() {
            self.answer(answers.pop().expect("accounts store answers"));
        }
        self.component.fire(&self.env, &mut self.effects);
        self.effects.len().checked_add(u32::from(self.held.is_some())).expect("accounts effects")
    }
    /// One effect, in the component's order; `None` when it stayed inside.
    #[must_use]
    pub fn output(&mut self, table: &mut Table) -> Option<Output> {
        if let Some(event) = self.held.take() {
            return Some(Output::Account(event));
        }
        match self.effects.pop().expect("accounts effects") {
            Effect::Child(event) => Some(Output::Account(event)),
            Effect::Io(request) => {
                self.io.push(request);
                None
            }
            Effect::Store { .. } if self.aborting => None,
            Effect::Store { exchange, request } => Some(Output::Store { exchange, request }),
            Effect::Granted { source, generation, value, expires } => {
                table.granted(source, generation, value, expires).expect("admitted grant");
                None
            }
            Effect::Removed { source } => {
                table.remove(source);
                None
            }
            Effect::SourceRemoved { .. } => None,
            Effect::InvalidSecret => Some(Output::InvalidSecret),
        }
    }
    /// A store's answer, from `store::Secrets` within the pass; its namespace comes off.
    pub fn answer(&mut self, event: store::Event) {
        self.component.step(&self.env, Event::Store(untagged(event)), &mut self.effects);
    }
    /// Gives back an `Output::Account` the owner has no room for: it leaves first in the next pass.
    pub fn hold(&mut self, event: domain::Event) {
        assert!(self.held.is_none(), "one held accounts event");
        self.held = Some(event);
    }
}

fn untagged(mut event: store::Event) -> store::Event {
    match &mut event {
        store::Event::Secret { owner, .. }
        | store::Event::Kept { owner }
        | store::Event::Stale { owner }
        | store::Event::Erased { owner }
        | store::Event::SecretsLoaded { owner, .. } => *owner = Token::new(owner.untagged()),
        // The map census belongs to the root, not secrets (protocol/store/README.md, 6.3).
        store::Event::Census { .. }
        | store::Event::Opened { .. }
        | store::Event::Switched { .. }
        | store::Event::OpenRefused { .. }
        | store::Event::Committed { .. }
        | store::Event::Failed { .. }
        | store::Event::FrameTooLarge { .. }
        | store::Event::Page { .. }
        | store::Event::PageRefused { .. }
        | store::Event::Loaded { .. }
        | store::Event::Read { .. }
        | store::Event::Got { .. }
        | store::Event::Turns { .. }
        | store::Event::Unread { .. }
        | store::Event::SecretsNotReady { .. }
        | store::Event::Carried { .. }
        | store::Event::Corrupt { .. }
        | store::Event::Site { .. }
        | store::Event::Decoded { .. }
        | store::Event::Undecodable { .. }
        | store::Event::Rehearsing { .. }
        | store::Event::Back { .. }
        | store::Event::BackRefused { .. }
        | store::Event::Closed { .. }
        | store::Event::Patched { .. } => unreachable!("accounts routing receives only secrets answers"),
    }
    event
}
fn has_tls(endpoint: &driver::Endpoint, origins: &List<driver::Origin>) -> bool {
    match endpoint.transport {
        driver::Transport::Tls { .. } => return true,
        driver::Transport::Plaintext => (),
    }
    for origin in origins {
        match origin.endpoint.transport {
            driver::Transport::Tls { .. } => return true,
            driver::Transport::Plaintext => (),
        }
    }
    false
}

/// Trust roots a shell's bounds refuse; a shell that gives none answers for its own.
fn untrusted(bounds: Option<TrustBounds>, tls: bool) -> bool {
    match bounds {
        Some(bounds) => (bounds.roots == 0 || bounds.bytes == 0) && tls,
        None => false,
    }
}
#[cfg(test)]
#[path = "host_tests.rs"]
mod tests;
