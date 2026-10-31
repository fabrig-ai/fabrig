//! The accounts driver, its records in the secrets' store and secret-free
//! translation (llm.md, 5; store.md, 9).
use crate::{Files, Seeds, values::Values};
use alloc::boxed::Box;
use jig_domain_accounts as domain;
use jig_domain_core as core;
use jig_protocol_accounts::{
    Component,
    component::{Effect, Event},
    party,
};
use jig_protocol_config::{self as config, Path, Problems, What};
use jig_protocol_store as store;
use jig_test_domain as root;
use skein_io::{
    self as io,
    kernel::{Complete, Op, Submit},
};
use skein_lib::{CryptoRng, Env, List, Map, Queue, Secret, Time, Token, Wall};
use skein_oauth as oauth;
use skein_oauth_accounts as driver;

#[derive(Debug)]
pub struct AccountsRuntime {
    /// LLM account sign-in endpoints and origins, keyed by provider number.
    pub providers: List<(u32, driver::Endpoint, List<driver::Origin>)>,
    /// Bounds of all prepared trust roots, supplied by their shell owner.
    pub trust_roots: u32,
    pub trust_root_bytes: u32,
}
fn problem(path: Path, what: What) -> Problems {
    let mut found = Problems::empty();
    found.push(path, what);
    found
}
impl Files {
    pub(crate) fn contents(&self, path: &[u8]) -> Option<&[u8]> {
        for (name, value) in &self.entries {
            if name.as_ref() == path {
                return Some(value.expose());
            }
        }
        None
    }
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
/// A provider registration copied without a source record.
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

fn provider_templates(
    config: &config::Engine,
    runtime: Option<&mut AccountsRuntime>,
    limits: &driver::Limits,
    crypto: &CryptoRng,
    installation_id: &[u8; 16],
) -> Result<Map<u32, driver::Account>, Problems> {
    let mut providers = Map::with_capacity(u32::try_from(config.llm.providers.len()).expect("provider count"));
    if let Some(runtime) = runtime {
        for (number, endpoint, origins) in
            ::core::mem::replace(&mut runtime.providers, List::with_capacity(0)).into_boxed()
        {
            let mut registration = None;
            for provider in &config.llm.providers {
                if provider.number == number {
                    registration = provider.sign_in.as_ref();
                    break;
                }
            }
            let registration = registration
                .ok_or(problem(Path::root().member(b"engine").member(b"llm").member(b"providers"), What::Refers))?;
            if (runtime.trust_roots == 0 || runtime.trust_root_bytes == 0) && has_tls(&endpoint, &origins) {
                return Err(problem(Path::root().member(b"engine").member(b"llm").member(b"providers"), What::Bound));
            }
            let account = party::party_account(
                &registration_input(registration, &endpoint.transport),
                endpoint,
                origins,
                installation_id,
            );
            let mut check = List::with_capacity(1);
            check
                .push(unkept(&account))
                .or(Err(problem(Path::root().member(b"engine").member(b"llm").member(b"providers"), What::Bound)))?;
            let _checked = driver::Component::new(check, limits, crypto.clone())
                .or(Err(problem(Path::root().member(b"engine").member(b"llm").member(b"providers"), What::Form)))?;
            providers
                .insert(number, account)
                .or(Err(problem(Path::root().member(b"engine").member(b"llm").member(b"providers"), What::Bound)))?;
        }
    }
    for provider in &config.llm.providers {
        if provider.sign_in.is_some() && !providers.contains_key(&provider.number) {
            return Err(problem(Path::root().member(b"engine").member(b"llm").member(b"providers"), What::Refers));
        }
    }
    Ok(providers)
}

pub(super) struct Accounts {
    component: Component,
    env: Env<driver::Limits>,
    pub(super) io: Queue<io::Request>,
    effects: Queue<Effect>,
    opened: bool,
}
impl Accounts {
    /// Checks public provider registrations; restored sources are admitted after the store opens.
    pub fn new(
        application: Box<[u8]>,
        config: &config::Engine,
        seeds: &Seeds,
        mut runtime: Option<AccountsRuntime>,
        secrets: &skein_kv::Limits,
    ) -> Result<Self, Problems> {
        let limits = config.limits.driver;
        if limits.client.document.record_bytes > secrets.value {
            return Err(problem(Path::root().member(b"engine").member(b"llm").member(b"providers"), What::Bound));
        }
        let crypto = CryptoRng::new(&seeds.crypto, 1)
            .ok_or(problem(Path::root().member(b"engine").member(b"llm").member(b"providers"), What::Bound))?;
        let providers = provider_templates(config, runtime.as_mut(), &limits, &crypto, &seeds.installation)?;
        Ok(Self {
            component: Component::new(application, providers, limits, crypto, secrets.page.rows),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            io: Queue::uncapped(),
            effects: Queue::uncapped(),
            opened: false,
        })
    }

    pub fn startup(&self, source: u32) -> (bool, u32, u32) {
        self.component.startup(source)
    }
    pub fn take_moved(&mut self, source: u32) -> Option<Secret> {
        self.component.take_moved(source)
    }
    pub fn clock(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
    }
    pub fn request(&mut self, request: domain::Request) {
        self.component.step(&self.env, Event::Request(request), &mut self.effects);
    }
    pub fn up(&mut self, event: io::Event) {
        self.component.step(&self.env, Event::Io(event), &mut self.effects);
    }
    pub fn abort(&mut self) {
        self.component.step(&self.env, Event::Abort, &mut self.effects);
    }
    pub fn is_empty(&self) -> bool {
        self.component.is_empty() && self.effects.is_empty() && self.io.is_empty()
    }
    pub fn next_deadline(&self) -> Option<Time> {
        self.component.next_deadline()
    }
    pub fn work_pending(&self, now: Time, marked: bool) -> bool {
        (marked && !self.opened) || self.component.has_work(now) || !self.effects.is_empty() || !self.io.is_empty()
    }
    pub fn iterate(
        &mut self,
        now: Time,
        wall: Wall,
        values: &mut Values,
        events: &mut Queue<root::Event>,
        store: &mut crate::store::Store,
    ) {
        self.clock(now, wall);
        if !self.opened && store.component.mark_durable() {
            self.component.step(&self.env, Event::Opened { installation: store.installation }, &mut self.effects);
            self.opened = true;
        }
        for _ in 0..store.sources.len() {
            let event = untag_store(store.sources.pop().expect("accounts store answers"));
            self.component.step(&self.env, Event::Store(event), &mut self.effects);
        }
        self.component.fire(&self.env, &mut self.effects);
        for _ in 0..self.effects.len() {
            match self.effects.pop().expect("accounts effects") {
                Effect::Child(event) => events.push(root::Event::Core(core::Event::Account(event))),
                Effect::Io(request) => self.io.push(request),
                Effect::Store { exchange, request } => {
                    if let Some(event) =
                        store.secrets(crate::tokens::token(crate::tokens::ACCOUNTS, exchange.raw()), request)
                    {
                        self.component.step(&self.env, Event::Store(untag_store(event)), &mut self.effects);
                    }
                }
                Effect::Granted { source, generation, value, expires } => {
                    values.accounts.granted(source, generation, value, expires).expect("admitted grant");
                }
                Effect::Removed { source } => values.accounts.remove(source),
                Effect::SourceRemoved { .. } => (),
                Effect::InvalidSecret => store.above.push(store::Event::PageRefused {
                    owner: Token::new(0),
                    failure: store::restore::Failure {
                        cause: store::restore::Cause::Decode(store::restore::DecodeFailure::Value),
                        row: None,
                    },
                }),
            }
        }
        values.accounts.expire(now);
    }
}
fn untag_store(mut event: store::Event) -> store::Event {
    match &mut event {
        store::Event::Secret { owner, .. }
        | store::Event::Kept { owner }
        | store::Event::Stale { owner }
        | store::Event::Erased { owner }
        | store::Event::SecretsLoaded { owner, .. } => *owner = Token::new(owner.untagged()),
        store::Event::Opened { .. }
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
        | store::Event::Patched { .. } => unreachable!("accounts routing receives only secrets answers"),
    }
    event
}
pub(super) fn earliest(left: Option<Time>, right: Option<Time>) -> Option<Time> {
    match right {
        Some(right) => Some(match left {
            Some(left) => left.min(right),
            None => right,
        }),
        None => left,
    }
}

#[derive(Debug)]
pub(super) struct Kernel {
    routes: Map<Token, (Token, Option<Token>)>,
    retired: Map<Token, Token>,
    next: u64,
}
impl Kernel {
    pub fn new(room: u32) -> Self {
        Self { routes: Map::with_capacity(room), retired: Map::with_capacity(room), next: 1 }
    }
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty() && self.retired.is_empty()
    }
    pub fn publish(&mut self, source: &mut Queue<Submit>, target: &mut Queue<Submit>) {
        for _ in 0..source.len() {
            if target.room() == 0 || self.routes.len() == self.routes.capacity() {
                break;
            }
            let mut submit = source.pop().expect("kernel submissions");
            let mut cancel = None;
            if let Op::Cancel { target } = &mut submit.kind {
                cancel = Some(*target);
                let mut found = None;
                for (external, (internal, _)) in &self.routes {
                    if *internal == *target {
                        found = Some(*external);
                        break;
                    }
                }
                *target = match found {
                    Some(external) => external,
                    None => self.retired.remove(target).expect("cancel names submitted operation"),
                };
            }
            let external = Token::new(self.next);
            self.next = self.next.checked_add(1).expect("kernel operation ids");
            self.routes.insert(external, (submit.op, cancel)).expect("kernel route room");
            submit.op = external;
            target.push(submit);
        }
    }
    pub fn complete(&mut self, mut complete: Complete, sockets: &Queue<Submit>) -> Complete {
        let (internal, cancel) = self.routes.remove(&complete.op).expect("submitted operation route");
        for queued in sockets {
            if let Op::Cancel { target } = queued.kind
                && target == internal
            {
                self.retired.insert(internal, complete.op).expect("pending cancellation route");
                break;
            }
        }
        complete.op = internal;
        if let Some(internal) = cancel
            && let Op::Cancel { target } = &mut complete.kind
        {
            *target = internal;
        }
        complete
    }
}

pub(super) fn owns(token: Token) -> bool {
    token.tag() == crate::tokens::ACCOUNTS
}
pub(super) fn tag(request: io::Request) -> io::Request {
    crate::tokens::tag(request, crate::tokens::ACCOUNTS)
}
pub(super) fn untag(event: io::Event) -> io::Event {
    crate::tokens::untag(event)
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

#[cfg(test)]
#[path = "tests/accounts.rs"]
mod tests;
