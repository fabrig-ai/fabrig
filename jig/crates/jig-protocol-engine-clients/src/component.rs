//! Calls, exchanges, and durable sign-in sequencing (protocol/client/sign-in.md).
use crate::{
    Event, Kept, Limits, Request, SecretOwner, SecretsAnswer, SecretsRequest, Settings,
    exchange::Exchange,
    routes::{self, Route},
    sign_in::{self, Link, Past, Pending, Slot, Waiting, Which},
};
use alloc::boxed::Box;
use jig_client_protocol_wire::v20 as w;
use jig_domain_core_vocabulary as core;
use jig_domain_people as people;
use skein_lib::{CryptoRng, Env, Map, Queue, ReplyTo, Time, Token};
#[path = "approval.rs"]
mod approval;
#[path = "closing.rs"]
mod closing;
#[path = "mint.rs"]
mod mint;
#[path = "provider.rs"]
pub(crate) mod provider;
#[path = "public/component.rs"]
mod public;
#[path = "public/me.rs"]
mod public_me;
#[path = "reads.rs"]
mod reads;
#[path = "services.rs"]
pub(crate) mod services;
#[path = "watches.rs"]
mod watches;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unusable {
    Limits,
    Seed,
    LinkSecret,
    LocalSecret,
}
#[derive(Debug)]
pub struct Component {
    settings: Settings,
    provider: provider::State,
    approval: approval::State,
    rng: CryptoRng,
    printed: Slot,
    local: Slot,
    past_links: Queue<(skein_lib::Secret, Past, Which)>,
    connections: Map<Token, Exchange>,
    pending: Pending,
    streams: Map<Token, crate::stream::Stream>,
    next: u64,
    phase: closing::Phase,
    public: public::State,
}
impl Component {
    pub fn new(mut settings: Settings, env: &Env<Limits>, out: &mut Queue<Request>) -> Result<Self, Unusable> {
        let given = settings.link_secret.take();
        let local_given = settings.local_secret.take();
        if !crate::valid(&env.limits) {
            return Err(Unusable::Limits);
        }
        if settings.served.capacity() > env.limits.families
            || settings.providers.capacity() > env.limits.providers
            || settings.addresses.len() > skein_lib::length::usize_of(env.limits.providers)
            || settings.heartbeat == skein_lib::Duration::ZERO
            || settings.rows == 0
            || settings.redirect_bytes > env.limits.wire.keyed_value_some
            || settings.key_bytes > env.limits.wire.keyed_value_some
            || settings.moved_bytes > u64::from(env.limits.wire.keyed_value_some)
            || !crate::translate::llm::fits(&settings.models, &env.limits.wire)
        {
            return Err(Unusable::Limits);
        }
        let Ok(hello) = crate::translate::watches::hello([0; 16], &settings, &env.limits.wire) else {
            return Err(Unusable::Limits);
        };
        if hello.measure() > env.limits.queued_bytes {
            return Err(Unusable::Limits);
        }
        if let Some(secret) = &given
            && secret.expose().len() != crate::LINK_SECRET_BYTES
        {
            return Err(Unusable::LinkSecret);
        }
        if let Some(secret) = &local_given {
            if secret.expose().len() != crate::LINK_SECRET_BYTES {
                return Err(Unusable::LocalSecret);
            }
            if let Some(printed) = &given
                && sign_in::constant_time_eq(secret.expose(), printed.expose())
            {
                return Err(Unusable::LocalSecret);
            }
        }
        if let Some(identity) = &settings.link {
            let bytes = identity.key.subject.len().checked_add(identity.login.len()).ok_or(Unusable::Limits)?;
            let bytes = bytes.checked_add(identity.name.len()).ok_or(Unusable::Limits)?;
            if bytes > skein_lib::length::usize_of(env.limits.identity_bytes) {
                return Err(Unusable::Limits);
            }
        }
        if settings.provider.is_some() {
            let issuer = crate::provider_limits(&env.limits);
            let location = issuer
                .uri_bytes
                .checked_add(issuer.state_bytes.checked_mul(3).ok_or(Unusable::Limits)?)
                .ok_or(Unusable::Limits)?;
            let location = location
                .checked_add(issuer.token_bytes.checked_mul(3).ok_or(Unusable::Limits)?)
                .ok_or(Unusable::Limits)?;
            // Percent-encoded code/state, and room for separators or an issuer error.
            let location = location.checked_add(64).ok_or(Unusable::Limits)?;
            if env.limits.wire.authorize_go_location < location {
                return Err(Unusable::Limits);
            }
        }
        let public = public::State::new(&settings, env)?;
        let rng = CryptoRng::new(&settings.seed, settings.stream).ok_or(Unusable::Seed)?;
        let mut value = Self {
            public,
            provider: provider::State::new(&env.limits),
            approval: approval::State::new(),
            settings,
            rng,
            printed: Slot::new(),
            local: Slot::new(),
            past_links: Queue::with_capacity(sign_in::PAST_LINKS),
            connections: Map::with_capacity(env.limits.exchanges),
            pending: Pending {
                mints: Map::with_capacity(env.limits.mints),
                exchanges: Map::with_capacity(
                    env.limits.exchanges.checked_add(env.limits.exchanges).expect("checked pending capacity"),
                ),
                carried: Map::with_capacity(
                    env.limits.exchanges.checked_add(env.limits.exchanges).expect("checked pending capacity"),
                ),
            },
            streams: Map::with_capacity(env.limits.streams),
            next: 1,
            phase: closing::Phase::Active,
        };
        if let Some(secret) = given
            && value.settings.link.is_some()
        {
            value.printed.live = Some(Link { secret, until: env.now.saturating_add(env.limits.link_span) });
        } else {
            value.fresh_link(Which::Printed, env, out);
        }
        if let Some(secret) = local_given
            && value.settings.link.is_some()
        {
            value.local.live = Some(Link { secret, until: env.now.saturating_add(env.limits.link_span) });
        } else {
            value.fresh_link(Which::Local, env, out);
        }
        out.push(Request::Secrets(SecretsRequest::Get { owner: SecretOwner::PublicNames }));
        Ok(value)
    }
    fn name(&mut self) -> Token {
        let token = Token::new(self.next);
        self.next = self.next.checked_add(1).expect("component names do not exhaust");
        token
    }
    fn slot(&self, which: Which) -> &Slot {
        match which {
            Which::Printed => &self.printed,
            Which::Local => &self.local,
        }
    }
    fn slot_mut(&mut self, which: Which) -> &mut Slot {
        match which {
            Which::Printed => &mut self.printed,
            Which::Local => &mut self.local,
        }
    }
    fn remember_link(&mut self, which: Which, link: Link, past: Past) {
        if self.past_links.room() == 0 {
            self.past_links.pop();
        }
        let digest = sign_in::digest(link.secret.expose());
        self.past_links.push((digest, past, which));
    }
    // Why a secret is not live, and which link it belonged to.
    fn link_answer(&self, secret: &[u8]) -> (Box<[u8]>, Option<Which>) {
        if self.settings.link.is_none() {
            return (routes::refused(), None);
        }
        let digest = sign_in::digest(secret);
        let mut found = None;
        for (past_digest, past, which) in &self.past_links {
            if sign_in::constant_time_eq(digest.expose(), past_digest.expose()) {
                found = Some((*past, *which));
            }
        }
        match found {
            Some((Past::Used, which)) => (routes::link_used(), Some(which)),
            Some((Past::Expired, which)) => (routes::link_expired(), Some(which)),
            None => (routes::link_unknown(), None),
        }
    }
    fn fresh_link(&mut self, which: Which, env: &Env<Limits>, out: &mut Queue<Request>) {
        if self.settings.link.is_none() || self.slot(which).live.is_some() || self.slot(which).spending.is_some() {
            return;
        }
        let secret = sign_in::draw(&mut self.rng);
        self.slot_mut(which).live =
            Some(Link { secret: secret.clone(), until: env.now.saturating_add(env.limits.link_span) });
        out.push(match which {
            Which::Printed => Request::Link { secret },
            Which::Local => Request::LocalLink { secret },
        });
    }
    fn expire_links(&mut self, now: Time) -> bool {
        let mut any = false;
        for which in [Which::Printed, Which::Local] {
            if expired(self.slot(which).live.as_ref(), now) {
                let link = self.slot_mut(which).live.take().expect("expired link");
                self.remember_link(which, link, Past::Expired);
                any = true;
            }
        }
        any
    }
    fn take_reservation(&mut self, exchange: Token) -> Option<(Which, Link)> {
        for which in [Which::Printed, Which::Local] {
            if let Some((reserved, _)) = &self.slot(which).spending
                && *reserved == exchange
            {
                let (_, link) = self.slot_mut(which).spending.take().expect("matching reservation");
                return Some((which, link));
            }
        }
        None
    }
    fn spend_link(&mut self, spent: Option<(Which, Link)>, env: &Env<Limits>, out: &mut Queue<Request>) {
        if let Some((which, link)) = spent {
            self.remember_link(which, link, Past::Used);
            if which == Which::Local && (self.phase == closing::Phase::Active || self.phase == closing::Phase::Stopped)
            {
                self.fresh_link(Which::Local, env, out);
            }
        }
    }
    pub fn down(&mut self, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
        assert!(!self.closed(), "the owner sends nothing after Closed");
        match event {
            Event::Catalogue { models } => {
                assert!(crate::translate::llm::fits(&models, &env.limits.wire), "core catalogue fits the client wire");
                self.settings.models = models;
            }
            Event::Transport(message) => self.transport(env, message, out),
            Event::Reply { to, reply } => self.reply(env, to.into_token(), reply, out),
            Event::SignInRefused { to } => self.sign_in_refused(env, to.into_token(), out),
            Event::Secrets(SecretsAnswer::Got { owner: SecretOwner::PublicNames, value }) => {
                self.public_key(value, out);
            }
            Event::Secrets(SecretsAnswer::Kept { owner: SecretOwner::PublicNames }) => self.public_kept(),
            Event::Secrets(answer) => self.secrets(env, answer, out),
            Event::Public { reply, answer } => self.public_answer(env, reply.into_token(), answer, out),
            Event::Application(crate::FromApplication::Undecodable { to }) => {
                let exchange = to.into_token();
                self.pending.exchanges.remove(&exchange);
                self.unused(exchange, out);
                if let Some(owner) = self.owner(exchange) {
                    self.connections.get_mut(&owner).expect("exchange owner").respond(crate::Reply::NotARecord);
                }
            }
            Event::SignInEnded { sign_in } => self.ended(env, sign_in, out),
            Event::Application(crate::FromApplication::Answered { to, family, record }) => {
                self.application(env, to.into_token(), family, record);
            }
            Event::MovedTaken { exchange, record } => self.moved_taken(env, exchange, record),
            Event::Stop => self.stop(),
            Event::Close => self.close(),
            Event::Abort => self.abort(),
            Event::Read { reply, answer } => {
                if let Some(frame) = public_me::read_frame(&answer) {
                    self.named_read(env, reply.into_token(), frame);
                } else {
                    self.read_answer(env, reply.into_token(), &answer);
                }
            }
            Event::Application(crate::FromApplication::Read { reply, family, record }) => {
                self.application_read(env, reply.into_token(), family, record);
            }
            Event::Admitted { watcher } => self.watch_answer(env, watcher, None),
            Event::WatchRefused { watcher, why } => self.watch_answer(env, watcher, Some(why)),
            Event::Delivery { watcher, document } => {
                if let Some(frame) = public_me::frame(&document) {
                    self.named_delivery(env, watcher, frame, out);
                } else {
                    self.delivery(env, watcher, &document, out);
                }
            }
            Event::WatchEnded { watcher, why } => self.watch_ended(env, watcher, why, out),
            Event::RunText { watcher, turn, from, to, text, cut, call, writing } => {
                self.run_text(
                    env,
                    watcher,
                    jig_domain_views::Writing { turn, from, to, text, cut, call, writing },
                    out,
                );
            }
            Event::Application(crate::FromApplication::Delivery { watcher, family, record }) => {
                self.application_delivery(env, watcher, family, record, out);
            }
            Event::Redeem { exchange, authorization, content_type, body } => {
                let answer = self.redeem(env, authorization.as_ref(), content_type.as_deref(), body.expose());
                out.push(Request::Redeemed { exchange, answer });
            }
            Event::Claims { exchange, authorization } => {
                let answer = self.claims(env, authorization.as_ref());
                out.push(Request::Claimed { exchange, answer });
            }
            Event::Mint { exchange, identity } => self.mint(env, exchange, identity, out),
            Event::Application(crate::FromApplication::Unused { number }) => {
                let mut found = None;
                for (exchange, kept) in &self.pending.carried {
                    if *kept == Some(number) {
                        found = Some(*exchange);
                        break;
                    }
                }
                if let Some(exchange) = found {
                    self.unused(exchange, out);
                }
            }
        }
    }
    /// Advances one queued HTTP transition, bounding each invocation's output.
    #[expect(clippy::too_many_lines, reason = "one bounded transition pump includes the separate public drain")]
    pub fn up(&mut self, env: &Env<Limits>, out: &mut Queue<Request>) -> bool {
        if self.aborting() {
            self.public.abort(env.limits.public.bytes_burst);
            if let Some((name, _)) = self.pending.mints.first() {
                let name = *name;
                self.mint_refused(name, crate::MintRefusal::NotReady, out);
                return true;
            }
            if self.stream_up(env, out) {
                return true;
            }
            if let Some((owner, _)) = self.connections.first() {
                let owner = *owner;
                let mut connection = self.connections.remove(&owner).expect("closing call");
                connection.close();
                return true;
            }
            if self.settled() {
                self.phase = closing::Phase::Closed;
                out.push(Request::Closed);
                return true;
            }
            return false;
        }
        let mut ready = None;
        for (exchange, waiting) in &self.pending.exchanges {
            match waiting {
                Waiting::Checked { .. } => {
                    ready = Some(*exchange);
                    break;
                }
                Waiting::ApprovalWhose { .. }
                | Waiting::ApprovalCore { .. }
                | Waiting::Core { .. }
                | Waiting::LinkGet { .. }
                | Waiting::SignOut { .. }
                | Waiting::Watch { .. }
                | Waiting::Get { .. }
                | Waiting::Keep { .. }
                | Waiting::Moving { .. }
                | Waiting::Made { .. }
                | Waiting::Read { .. }
                | Waiting::Authorize { .. }
                | Waiting::ApplicationRead { .. }
                | Waiting::Ask { .. }
                | Waiting::Carry { .. } => {}
            }
        }
        if let Some(exchange) = ready {
            match self.pending.exchanges.remove(&exchange).expect("ready exchange") {
                Waiting::Checked { sign_in, body, route } => match route {
                    Route::Read => self.read(env, exchange, sign_in, &body, out),
                    Route::Watch | Route::Unwatch | Route::Gone | Route::Stream => {
                        self.watch_route(env, exchange, sign_in, route, &body, out);
                    }
                    Route::Request => self.request(env, exchange, sign_in, &body, out),
                    Route::SignOut => self.sign_out(env, exchange, sign_in, &body, out),
                    Route::Authorize => self.authorize_checked(env, exchange, sign_in, &body, out),
                    Route::ApprovalRead | Route::ApprovalDecide => {
                        self.approval_checked(env, exchange, sign_in, route, &body);
                    }
                    Route::ApprovalAsk | Route::ApprovalTake | Route::Link | Route::Service => {}
                },
                Waiting::ApprovalWhose { .. }
                | Waiting::ApprovalCore { .. }
                | Waiting::Core { .. }
                | Waiting::LinkGet { .. }
                | Waiting::SignOut { .. }
                | Waiting::Watch { .. }
                | Waiting::Get { .. }
                | Waiting::Keep { .. }
                | Waiting::Moving { .. }
                | Waiting::Made { .. }
                | Waiting::Read { .. }
                | Waiting::Authorize { .. }
                | Waiting::ApplicationRead { .. }
                | Waiting::Ask { .. }
                | Waiting::Carry { .. } => {}
            }
            return true;
        }
        let mut selected = None;
        for (owner, connection) in &self.connections {
            if connection.closing || connection.response.is_some() || connection.ended.is_some() {
                selected = Some(*owner);
                break;
            }
        }
        let Some(owner) = selected else {
            if self.stream_up(env, out) {
                return true;
            }
            if !self.public.settled() {
                return false;
            }
            return self.close_up(out);
        };
        let mut connection = self.connections.remove(&owner).expect("selected exchange");
        if let Some(reply) = connection.response.take() {
            out.push(Request::Transport(crate::ToTransport::Reply { exchange: owner, reply }));
        } else if let Some(why) = connection.ended {
            out.push(Request::Transport(crate::ToTransport::StreamEnded { stream: owner, why }));
        }
        self.pending.exchanges.remove(&connection.name);
        if let Some(stream) = self.streams.get_mut(&owner) {
            stream.phase = crate::stream::Phase::Closing;
            stream.closed_connection = true;
        }
        true
    }
    fn transport(&mut self, env: &Env<Limits>, message: crate::FromTransport, out: &mut Queue<Request>) {
        match message {
            crate::FromTransport::Exchange { exchange, credential, browser, message } => {
                let (route, body) = routes::message(message);
                self.receive(env, exchange, (credential, browser), route, body, out);
            }
            crate::FromTransport::StreamOpen { stream, credential } => {
                self.receive(env, stream, (credential, false), Route::Stream, Box::new([]), out);
            }
            crate::FromTransport::StreamTaken { stream } => {
                if let Some(stream) = self.streams.get_mut(&stream)
                    && stream.waiting
                {
                    let taken = stream.sent(env, self.settings.heartbeat);
                    if let Some(event) = crate::stream::taken(taken) {
                        out.push(Request::Core(Box::new(event)));
                    }
                }
            }
            crate::FromTransport::PublicRead { exchange, read } => self.public_read(env, exchange, read, out),
            crate::FromTransport::StreamGone { stream } => self.gone(stream),
            crate::FromTransport::ExchangeGone { exchange } => self.gone(exchange),
        }
    }
    fn gone(&mut self, owner: Token) {
        self.public.gone(owner);
        self.connections.remove(&owner);
        if let Some(value) = self.streams.get_mut(&owner) {
            value.phase = crate::stream::Phase::Closing;
            value.closed_connection = true;
        }
    }
    fn receive(
        &mut self,
        env: &Env<Limits>,
        owner: Token,
        authentication: (Option<skein_lib::Secret>, bool),
        route: Route,
        body: Box<[u8]>,
        out: &mut Queue<Request>,
    ) {
        let exchange = self.name();
        let mut connection = Exchange::new(exchange, route == Route::Stream);
        self.body(env, &mut connection, route, body, authentication, out);
        self.connections.insert(owner, connection).expect("transport bounds active exchanges");
    }
    #[expect(clippy::too_many_lines, reason = "dispatch keeps disabled approval answers before credential lookup")]
    fn body(
        &mut self,
        env: &Env<Limits>,
        connection: &mut Exchange,
        route: Route,
        body: Box<[u8]>,
        authentication: (Option<skein_lib::Secret>, bool),
        out: &mut Queue<Request>,
    ) {
        let exchange = connection.name;
        let (credential, browser) = authentication;
        if self.settings.approval && (route == Route::ApprovalRead || route == Route::ApprovalDecide) && !browser {
            let answer = if route == Route::ApprovalRead {
                routes::approval_state(w::ApprovalState::NotABrowser)
            } else {
                routes::approval_decided(w::ApprovalDecided::NotABrowser)
            };
            connection.respond(crate::Reply::Record(answer));
            return;
        }
        if body.len() > skein_lib::length::usize_of(env.limits.message_bytes) {
            connection.respond(crate::Reply::NotARecord);
            return;
        }
        if self.stopping()
            && let Some(answer) = routes::stopped(route)
        {
            connection.respond(crate::Reply::Record(answer));
            return;
        }
        if route == Route::Authorize && !self.authorize_query(env, connection, &body) {
            return;
        }
        match route {
            Route::ApprovalAsk if self.settings.approval => self.approval_ask(env, connection, &body),
            Route::ApprovalTake if self.settings.approval => self.approval_take(env, connection, &body, out),
            Route::ApprovalAsk => {
                connection.respond(crate::Reply::Record(routes::approval_ask_answer(w::ApprovalAskAnswer::Refused)));
            }
            Route::ApprovalRead if !self.settings.approval => {
                connection.respond(crate::Reply::Record(routes::approval_state(w::ApprovalState::Unknown)));
            }
            Route::ApprovalDecide if !self.settings.approval => {
                connection.respond(crate::Reply::Record(routes::approval_decided(w::ApprovalDecided::Unknown)));
            }
            Route::ApprovalTake => {
                connection.respond(crate::Reply::Record(routes::approval_sign_in(w::ApprovalSignInAnswer::Unknown)));
            }
            Route::Link => {
                let Ok(record) = w::LinkSignIn::decode(&env.limits.wire, &mut skein_lib::Reader::new(&body)) else {
                    connection.respond(crate::Reply::NotARecord);
                    return;
                };
                let digest = sign_in::digest(record.secret().expose());
                self.expire_links(env.now);
                let mut live = None;
                let mut reserved = false;
                for which in [Which::Printed, Which::Local] {
                    if let Some((_, link)) = &self.slot(which).spending {
                        reserved |=
                            sign_in::constant_time_eq(sign_in::digest(link.secret.expose()).expose(), digest.expose());
                    }
                    if let Some(link) = &self.slot(which).live
                        && sign_in::constant_time_eq(sign_in::digest(link.secret.expose()).expose(), digest.expose())
                    {
                        live = Some(which);
                    }
                }
                if reserved {
                    connection.respond(crate::Reply::Record(routes::sign_in_busy()));
                    return;
                }
                let Some(which) = live else {
                    let (answer, past) = self.link_answer(record.secret().expose());
                    if past != Some(Which::Local) {
                        self.fresh_link(Which::Printed, env, out);
                    }
                    self.fresh_link(Which::Local, env, out);
                    connection.respond(crate::Reply::Record(answer));
                    return;
                };
                if self.admitted() >= env.limits.exchanges {
                    connection.respond(crate::Reply::Record(routes::sign_in_busy()));
                    return;
                }
                let slot = self.slot_mut(which);
                if let Some(link) = slot.live.take() {
                    slot.spending = Some((exchange, link));
                }
                if let Some((sign_in, digest)) = sign_in::credential(credential.as_ref()) {
                    self.pending
                        .exchanges
                        .insert(exchange, Waiting::LinkGet { sign_in, digest })
                        .expect("admitted exchange");
                    out.push(Request::Secrets(SecretsRequest::Get { owner: SecretOwner::SignIn(sign_in) }));
                } else {
                    self.link_sign_in(exchange, None, out);
                }
            }
            Route::ApprovalRead
            | Route::ApprovalDecide
            | Route::Authorize
            | Route::Service
            | Route::SignOut
            | Route::Request
            | Route::Read
            | Route::Watch
            | Route::Unwatch
            | Route::Gone
            | Route::Stream => {
                let Some((sign_in, digest)) = sign_in::credential(credential.as_ref()) else {
                    let answer = routes::unauthenticated(route, &env.limits.wire);
                    if route == Route::Stream {
                        connection.ended = Some(crate::Ended::SignedOut);
                    } else {
                        connection.respond(crate::Reply::Record(answer));
                    }
                    return;
                };
                let owner = if route == Route::Service {
                    services::GetOwner::Service(sign_in)
                } else {
                    services::GetOwner::SignIn(sign_in)
                };
                self.pending
                    .exchanges
                    .insert(exchange, Waiting::Get { owner, digest, body, route })
                    .expect("admitted exchange");
                out.push(Request::Secrets(SecretsRequest::Get { owner: owner.secret() }));
            }
        }
    }
    pub(super) fn link_sign_in(&mut self, exchange: Token, held: Option<u64>, out: &mut Queue<Request>) {
        self.pending.exchanges.insert(exchange, Waiting::Core { held }).expect("admitted link slot");
        out.push(Request::Core(Box::new(core::Event::SignIn {
            reply_to: ReplyTo::new(exchange),
            identity: self.settings.link.as_ref().expect("link enabled").clone(),
            held,
        })));
    }
    fn reply(&mut self, env: &Env<Limits>, exchange: Token, reply: people::Reply, out: &mut Queue<Request>) {
        // The core names a value it leaves unused before its answer; one
        // still here is the request's to use (protocol/store/secrets.md).
        self.pending.carried.remove(&exchange);
        let Some(waiting) = self.pending.exchanges.remove(&exchange) else {
            return;
        };
        match waiting {
            Waiting::SignOut { sign_in, committed } => match reply {
                people::Reply::SignedOut => {
                    self.pending
                        .exchanges
                        .insert(exchange, Waiting::SignOut { sign_in, committed })
                        .expect("same sign-out slot");
                    if !committed {
                        self.ended(env, sign_in, out);
                    }
                }
                people::Reply::Again(people::Again::NotReady) => {
                    self.respond(exchange, routes::sign_out_not_ready());
                }
                people::Reply::Signer(_)
                | people::Reply::Full { .. }
                | people::Reply::SignedIn { .. }
                | people::Reply::Outcome(_)
                | people::Reply::Again(_)
                | people::Reply::Refused(_) => {
                    if let Some(owner) = self.owner(exchange) {
                        self.connections.get_mut(&owner).expect("exchange owner").close();
                    }
                }
            },
            Waiting::Authorize { authorization, .. } => self.authorize_reply(env, exchange, reply, authorization),
            Waiting::ApprovalWhose { digest } => self.approval_whose(env, exchange, digest, reply, out),
            Waiting::ApprovalCore { digest } => self.approval_reply(env, exchange, digest, reply, out),
            Waiting::Core { held } => self.sign_in_reply(env, exchange, reply, held, false, out),
            Waiting::Ask { sign_in, .. } => self.ask_reply(env, exchange, sign_in, reply, out, waiting),
            Waiting::LinkGet { .. }
            | Waiting::Watch { .. }
            | Waiting::Read { .. }
            | Waiting::ApplicationRead { .. }
            | Waiting::Keep { .. }
            | Waiting::Get { .. }
            | Waiting::Moving { .. }
            | Waiting::Made { .. }
            | Waiting::Carry { .. }
            | Waiting::Checked { .. } => {
                self.pending.exchanges.insert(exchange, waiting).expect("same slot");
            }
        }
    }
    /// Erases the value `exchange`'s decode kept, which its request will
    /// not use (protocol/store/secrets.md).
    fn unused(&mut self, exchange: Token, out: &mut Queue<Request>) {
        if let Some(Some(number)) = self.pending.carried.remove(&exchange)
            && !self.aborting()
        {
            out.push(Request::Secrets(SecretsRequest::Erase { owner: SecretOwner::Carried(number) }));
        }
    }
    fn ask_reply(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        reply: people::Reply,
        out: &mut Queue<Request>,
        waiting: Waiting,
    ) {
        match reply {
            people::Reply::Outcome(people::Outcome::AccountMovingOut { account, number, source }) => {
                self.pending
                    .exchanges
                    .insert(exchange, Waiting::Moving { sign_in, account, number })
                    .expect("same slot");
                out.push(Request::TakeMoved { exchange, source });
            }
            people::Reply::Outcome(people::Outcome::ServiceMade { person }) => {
                self.pending.exchanges.insert(exchange, Waiting::Made { sign_in, party: person }).expect("same slot");
                out.push(Request::Secrets(SecretsRequest::Get { owner: SecretOwner::Service(person) }));
            }
            people::Reply::Outcome(people::Outcome::Applied { .. }) => {
                self.pending.exchanges.insert(exchange, waiting).expect("same slot");
            }
            people::Reply::Signer(_)
            | people::Reply::Full { .. }
            | people::Reply::SignedIn { .. }
            | people::Reply::SignedOut
            | people::Reply::Outcome(_)
            | people::Reply::Again(_)
            | people::Reply::Refused(_) => {
                let answer = crate::translate::answers::answer(&reply, &self.settings.models, &env.limits.wire);
                self.answer(exchange, answer);
            }
        }
    }
    fn moved_taken(&mut self, env: &Env<Limits>, exchange: Token, record: Option<skein_lib::Secret>) {
        let Some(waiting) = self.pending.exchanges.remove(&exchange) else { return };
        match waiting {
            Waiting::Moving { account, number, .. } => {
                let record = match record {
                    Some(record) => record,
                    None => skein_lib::Secret::new(Box::new([])),
                };
                let answer = crate::translate::answers::moving_out(account, number, record, &env.limits.wire);
                self.answer(exchange, answer);
            }
            Waiting::ApprovalWhose { .. }
            | Waiting::ApprovalCore { .. }
            | Waiting::Core { .. }
            | Waiting::LinkGet { .. }
            | Waiting::SignOut { .. }
            | Waiting::Watch { .. }
            | Waiting::Keep { .. }
            | Waiting::Get { .. }
            | Waiting::Made { .. }
            | Waiting::Read { .. }
            | Waiting::Authorize { .. }
            | Waiting::ApplicationRead { .. }
            | Waiting::Ask { .. }
            | Waiting::Carry { .. }
            | Waiting::Checked { .. } => {
                self.pending.exchanges.insert(exchange, waiting).expect("same slot");
            }
        }
    }
    fn answer(&mut self, exchange: Token, answer: Result<w::KeyedAnswer, w::Problem>) {
        let Some(owner) = self.owner(exchange) else {
            return;
        };
        let connection = self.connections.get_mut(&owner).expect("exchange owner");
        match answer {
            Ok(answer) => {
                let mut body = skein_lib::Writer::new(skein_lib::length::usize_of(answer.measure()));
                answer.encode(&mut body).expect("measured answer");
                connection.respond(crate::Reply::Record(body.finish()));
            }
            Err(_) => connection.close(),
        }
    }
    fn application(&mut self, env: &Env<Limits>, exchange: Token, family: u16, record: Box<[u8]>) {
        // The application's answer: its request used its carried value.
        self.pending.carried.remove(&exchange);
        let Some(waiting) = self.pending.exchanges.remove(&exchange) else {
            return;
        };
        match waiting {
            Waiting::Ask { family: Some(number), version, .. } if number == family => {
                let answer = match w::Family::new(&env.limits.wire, w::FamilyParts { number }) {
                    Ok(value) => {
                        crate::translate::answers::owned(w::Owner::Family(value), version, record, &env.limits.wire)
                    }
                    Err(problem) => Err(problem),
                };
                self.answer(exchange, answer);
            }
            Waiting::ApprovalWhose { .. }
            | Waiting::ApprovalCore { .. }
            | Waiting::Core { .. }
            | Waiting::LinkGet { .. }
            | Waiting::SignOut { .. }
            | Waiting::Watch { .. }
            | Waiting::Keep { .. }
            | Waiting::Get { .. }
            | Waiting::Moving { .. }
            | Waiting::Made { .. }
            | Waiting::Checked { .. }
            | Waiting::Read { .. }
            | Waiting::Authorize { .. }
            | Waiting::ApplicationRead { .. }
            | Waiting::Ask { .. }
            | Waiting::Carry { .. } => {
                self.pending.exchanges.insert(exchange, waiting).expect("same slot");
            }
        }
    }
    fn sign_in_refused(&mut self, env: &Env<Limits>, exchange: Token, out: &mut Queue<Request>) {
        if let Some(waiting) = self.pending.exchanges.get(&exchange)
            && waiting.approval_take()
        {
            self.approval.refused(exchange);
            self.pending.exchanges.remove(&exchange);
            self.respond(exchange, routes::approval_sign_in(w::ApprovalSignInAnswer::Refused));
            return;
        }
        let spent = self.take_reservation(exchange);
        self.spend_link(spent, env, out);
        let why = if self.stopping() { crate::MintRefusal::NotReady } else { crate::MintRefusal::Bound };
        if !self.mint_refused(exchange, why, out) {
            self.refused(exchange);
        }
    }
    fn refused(&mut self, exchange: Token) {
        if self.pending.exchanges.remove(&exchange).is_some() {
            self.respond(exchange, routes::refused());
        }
    }
    fn owner(&self, exchange: Token) -> Option<Token> {
        for (owner, connection) in &self.connections {
            if connection.name == exchange && !connection.closing {
                return Some(*owner);
            }
        }
        None
    }
    fn respond(&mut self, exchange: Token, body: Box<[u8]>) {
        if let Some(owner) = self.owner(exchange) {
            self.connections.get_mut(&owner).expect("exchange owner").respond(crate::Reply::Record(body));
        }
    }
    fn denied(&mut self, env: &Env<Limits>, exchange: Token, route: Route) {
        if let Some(owner) = self.owner(exchange) {
            let connection = self.connections.get_mut(&owner).expect("exchange owner");
            if route == Route::Stream {
                connection.ended = Some(crate::Ended::SignedOut);
            } else {
                connection.respond(crate::Reply::Record(routes::unauthenticated(route, &env.limits.wire)));
            }
        }
    }
    fn admitted(&self) -> u32 {
        let mut count = 0_u32;
        for (_, waiting) in &self.pending.exchanges {
            match waiting {
                Waiting::ApprovalWhose { .. }
                | Waiting::ApprovalCore { .. }
                | Waiting::Core { .. }
                | Waiting::LinkGet { .. }
                | Waiting::SignOut { .. }
                | Waiting::Watch { .. }
                | Waiting::Keep { .. }
                | Waiting::Moving { .. }
                | Waiting::Made { .. }
                | Waiting::Read { .. }
                | Waiting::Authorize { .. }
                | Waiting::ApplicationRead { .. }
                | Waiting::Ask { .. }
                | Waiting::Carry { .. } => {
                    count = count.saturating_add(1);
                }
                Waiting::Get { .. } | Waiting::Checked { .. } => {}
            }
        }
        count
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        let mut next = None;
        for which in [Which::Printed, Which::Local] {
            if let Some(link) = &self.slot(which).live {
                next = Some(match next {
                    Some(next) => Time::min(next, link.until),
                    None => link.until,
                });
            }
        }
        if let Some(deadline) = self.approval.next_deadline() {
            next = Some(match next {
                Some(next) => next.min(deadline),
                None => deadline,
            });
        }
        for (_, stream) in &self.streams {
            if !stream.closing() && !stream.waiting {
                next = Some(match next {
                    Some(next) => next.min(stream.deadline),
                    None => stream.deadline,
                });
            }
        }
        next
    }
    pub fn fire(&mut self, env: &Env<Limits>) {
        self.approval.sweep(env.now);
        if self.expire_links(env.now) {
            return;
        }
        let mut due = None;
        for (owner, stream) in &self.streams {
            if !stream.closing() && !stream.waiting && stream.deadline <= env.now {
                due = Some(*owner);
                break;
            }
        }
        if let Some(owner) = due {
            let stream = self.streams.get_mut(&owner).expect("due stream");
            if stream.waiting {
                stream.phase = crate::stream::Phase::Closing;
            } else {
                stream.heartbeat();
            }
        }
    }
}
fn expired(link: Option<&Link>, now: Time) -> bool {
    match link {
        Some(link) => link.until <= now,
        None => false,
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[path = "request.rs"]
mod request;
