//! One bounded OAuth exchange (oauth/README.md, sections 1 and 3). It keeps the
//! registered endpoints, one exchange and its deadlines; it knows no provider
//! identity or token store. `Client::step` emits visits, HTTP demands and one
//! terminal. HTTP transport and persistence belong to the caller.

mod fetch;
mod revoke;

use crate::record::secret_ref;
use crate::redirect::{form_fields, issued_id, missing_scopes, resource_field, validate_registration};
use crate::{ClaimSelector, DecodeError, Failure, Json, Limits, OAuthError, RefreshState, SavedToken, TokenResponse};
use crate::{classify, decode_error, decode_response, rotate};
use alloc::boxed::Box;
use core::mem;
use skein_json::writer::Encoder;
use skein_lib::{Duration, Queue, Secret, Time, Wall, bytes, usize_of};

/// Startup bounds for one client and exchange.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ClientLimits {
    pub document: Limits,
    pub uri_bytes: u32,
    pub scope_bytes: u32,
    pub state_bytes: u32,
    pub code_bytes: u32,
    pub url_bytes: u32,
    pub request_bytes: u32,
    pub parameter_bytes: u32,
    pub resource_bytes: u32,
    pub nonce_bytes: u32,
    pub metadata_bytes: u32,
    pub key_set_bytes: u32,
    pub keys: u32,
    pub modulus_bits: u32,
    pub skew: Duration,
    pub sign_in_time: Duration,
    pub request_time: Duration,
    pub backoff_base: Duration,
    pub backoff_ceiling: Duration,
    pub max_attempts: u32,
}

impl ClientLimits {
    /// Small client limits for test worlds, overridable at each entrance.
    pub const TEST: Self = Self {
        document: Limits {
            document_bytes: 4096,
            string_bytes: 1024,
            token_bytes: 256,
            client_bytes: 64,
            detail_bytes: 64,
            record_bytes: 2048,
            depth: 8,
            tokens: 64,
            issued_client_bytes: 64,
            granted_bytes: 64,
            subject_bytes: 64,
            id_token_bytes: 1024,
        },
        uri_bytes: 256,
        scope_bytes: 64,
        state_bytes: 64,
        code_bytes: 64,
        url_bytes: 1024,
        request_bytes: 1024,
        parameter_bytes: 0,
        resource_bytes: 0,
        nonce_bytes: 0,
        metadata_bytes: 0,
        key_set_bytes: 0,
        keys: 0,
        modulus_bits: 2048,
        skew: Duration::from_secs(0),
        sign_in_time: Duration::from_secs(120),
        request_time: Duration::from_secs(10),
        backoff_base: Duration::from_secs(2),
        backoff_ceiling: Duration::from_secs(8),
        max_attempts: 3,
    };
}

/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
/// Exchange timing and key sizes are usable, saved records fit their schema,
/// and JSON input and request-field slots have representable lengths.
#[must_use]
pub fn client_valid(limits: &ClientLimits) -> bool {
    limits.max_attempts > 0
        && limits.request_time > Duration::ZERO
        && limits.sign_in_time > Duration::ZERO
        && (2048..=8192).contains(&limits.modulus_bits)
        && crate::record::record_limits(&limits.document).is_some()
        && limits.document.document_bytes < u32::MAX
        && limits.parameter_bytes.checked_add(11).is_some()
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WireFormat {
    Json,
    Form,
}

/// Caller-provided registration, with no provider identity or discovery.
#[derive(Debug)]
pub struct Registration {
    pub authorization_url: Box<[u8]>,
    pub token_endpoint: Box<[u8]>,
    pub client_id: Box<[u8]>,
    pub redirect_uri: Box<[u8]>,
    pub scope: Box<[u8]>,
    pub wire: WireFormat,
    pub client_secret: Option<Secret>,
    pub pkce_for_confidential: bool,
    pub metadata_claim: Option<ClaimSelector>,
    pub terms: Option<crate::Terms>,
}

/// What an HTTP demand asks of the caller's stack (oauth/README.md, section 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Method {
    /// A token exchange or revocation.
    Post,
    /// The issuer's metadata or key set, without a body or secret.
    Get,
}

/// A complete bounded request for the caller's HTTP/TLS stack.
pub struct HttpRequest {
    pub id: u64,
    pub method: Method,
    pub endpoint: Box<[u8]>,
    pub content_type: &'static [u8],
    pub body: Box<[u8]>,
    pub deadline: Time,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HttpEvidence {
    Unsent,
    Unknown,
    Response,
}

/// Terminal HTTP answer, including bounded body and transport evidence.
pub struct HttpResponse {
    pub id: u64,
    pub status: u16,
    pub body: Box<[u8]>,
    pub retry_after: Duration,
    pub evidence: HttpEvidence,
    pub now: Time,
    pub wall: Wall,
}

/// A redirect's values as its caller decoded them: the address without its query, and the
/// fields the client reads (oauth/README.md, section 3, and oauth/accounts.md, section 5).
#[derive(Debug)]
pub struct Redirect {
    pub uri: Box<[u8]>,
    pub state: Secret,
    pub code: Option<Secret>,
    pub error: Option<Box<[u8]>>,
    pub client_id: Option<Box<[u8]>>,
}

/// Why `Client::admits` would not take a redirect, in the order it checks (oauth/README.md, section 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Inadmissible {
    /// No sign-in waits for a redirect: any phase but `AwaitRedirect`, or its deadline reached.
    NotWaiting,
    /// Not the bound address; both code and error or neither; or no required issued client id.
    Address,
    /// Another sign-in's state, compared in constant time.
    State,
}

#[derive(Debug)]
pub enum Event {
    SignIn {
        registration: Registration,
        key: u32,
        generation: u64,
        state: Secret,
        verifier: Option<Secret>,
        nonce: Option<Secret>,
        port: Option<u16>,
        issued_client_id: Option<Box<[u8]>>,
        id_token: Option<Secret>,
        subject: Option<Box<[u8]>>,
        now: Time,
    },
    Redirected {
        uri: Box<[u8]>,
        state: Secret,
        code: Option<Secret>,
        error: Option<Box<[u8]>>,
        client_id: Option<Box<[u8]>>,
        now: Time,
    },
    Refresh {
        registration: Registration,
        prior: RefreshState,
        now: Time,
    },
    Revoke {
        registration: Registration,
        prior: RefreshState,
        now: Time,
    },
    Http(HttpResponse),
    Tick {
        now: Time,
    },
    Cancel,
    Reset,
}

#[derive(Debug)]
pub enum Request {
    Visit { url: Secret },
    Http(HttpRequest),
    Tokens { record: SavedToken, missing: Box<[u8]>, profile: crate::ProfileClaims },
    Revoked { confirmed: bool },
    Failed { failure: Failure },
}

/// The exchange belongs to its lifecycle state (programming-model.md, 5.4).
enum Phase {
    Idle,
    AwaitRedirect { exchange: Exchange },
    AwaitKeys { exchange: Exchange, id: u64, deadline: Time },
    AwaitHttp { exchange: Exchange, id: u64, deadline: Time },
    Backoff { exchange: Exchange, due: Time },
    Done,
}

enum Awaits {
    Redirect,
    Keys { id: u64, deadline: Time },
    Http { id: u64, deadline: Time },
}
impl Awaits {
    fn hold(self, exchange: Exchange) -> Phase {
        match self {
            Awaits::Redirect => Phase::AwaitRedirect { exchange },
            Awaits::Keys { id, deadline } => Phase::AwaitKeys { exchange, id, deadline },
            Awaits::Http { id, deadline } => Phase::AwaitHttp { exchange, id, deadline },
        }
    }
}

struct SignInValues {
    key: u32,
    generation: u64,
    state: Secret,
    verifier: Option<Secret>,
    nonce: Option<Secret>,
    port: Option<u16>,
    issued_client_id: Option<Box<[u8]>>,
    id_token: Option<Secret>,
    subject: Option<Box<[u8]>>,
    now: Time,
}

enum Grant {
    SignIn {
        key: u32,
        generation: u64,
        code: Option<Secret>,
        state: Secret,
        verifier: Option<Secret>,
        nonce: Option<Secret>,
        visit: Option<Box<[u8]>>,
        fetch: Option<fetch::Fetch>,
        keys: Box<[crate::keys::Key]>,
    },
    Refresh {
        prior: RefreshState,
    },
    Revoke {
        prior: RefreshState,
    },
}

struct Exchange {
    registration: Registration,
    grant: Grant,
    deadline: Time,
    attempts: u32,
}

impl Exchange {
    fn fetching(&self) -> bool {
        match &self.grant {
            Grant::SignIn { fetch, .. } => fetch.is_some(),
            Grant::Refresh { .. } | Grant::Revoke { .. } => false,
        }
    }
}

/// One sans-IO sign-in, refresh or revocation. The caller dispatches each entrance's output before the next.
pub struct Client {
    limits: ClientLimits,
    phase: Phase,
    next_id: u64,
}

impl Client {
    pub fn new(limits: ClientLimits) -> Result<Client, Failure> {
        if !client_valid(&limits) {
            return Err(Failure::Limit);
        }
        Ok(Client { limits, phase: Phase::Idle, next_id: 1 })
    }

    #[must_use]
    pub fn is_done(&self) -> bool {
        match &self.phase {
            Phase::Done => true,
            Phase::Idle
            | Phase::AwaitRedirect { .. }
            | Phase::AwaitKeys { .. }
            | Phase::AwaitHttp { .. }
            | Phase::Backoff { .. } => false,
        }
    }

    /// The next time the caller must send `Tick`, or none while idle or done.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        match &self.phase {
            Phase::Idle | Phase::Done => None,
            Phase::AwaitRedirect { exchange } => Some(exchange.deadline),
            Phase::AwaitHttp { deadline, .. } | Phase::AwaitKeys { deadline, .. } => Some(*deadline),
            Phase::Backoff { due, .. } => Some(*due),
        }
    }

    pub fn step(&mut self, event: Event, out: &mut Queue<Request>) {
        assert!(out.is_empty(), "the caller dispatches an entrance's output before the next");
        let phase = mem::replace(&mut self.phase, Phase::Done);
        self.phase = match event {
            Event::SignIn {
                registration,
                key,
                generation,
                state,
                verifier,
                nonce,
                port,
                issued_client_id,
                id_token,
                subject,
                now,
            } => match phase {
                Phase::Idle => {
                    let result = self.sign_in(
                        registration,
                        SignInValues {
                            key,
                            generation,
                            state,
                            verifier,
                            nonce,
                            port,
                            issued_client_id,
                            id_token,
                            subject,
                            now,
                        },
                    );
                    Self::emit_start(result, out)
                }
                phase @ (Phase::AwaitKeys { .. }
                | Phase::AwaitRedirect { .. }
                | Phase::AwaitHttp { .. }
                | Phase::Backoff { .. }
                | Phase::Done) => {
                    out.push(Request::Failed { failure: Failure::Limit });
                    phase
                }
            },
            Event::Refresh { registration, prior, now } => match phase {
                Phase::Idle => {
                    let result = self.refresh(registration, prior, now);
                    Self::emit_start(result, out)
                }
                phase @ (Phase::AwaitKeys { .. }
                | Phase::AwaitRedirect { .. }
                | Phase::AwaitHttp { .. }
                | Phase::Backoff { .. }
                | Phase::Done) => {
                    out.push(Request::Failed { failure: Failure::Limit });
                    phase
                }
            },
            Event::Revoke { registration, prior, now } => match phase {
                Phase::Idle => self.revoke(registration, prior, now, out),
                phase @ (Phase::AwaitKeys { .. }
                | Phase::AwaitRedirect { .. }
                | Phase::AwaitHttp { .. }
                | Phase::Backoff { .. }
                | Phase::Done) => {
                    out.push(Request::Revoked { confirmed: false });
                    phase
                }
            },
            Event::Redirected { uri, state, code, error, client_id, now } => match phase {
                Phase::AwaitRedirect { exchange } => {
                    self.redirected(exchange, Redirect { uri, state, code, error, client_id }, now, out)
                }
                phase @ (Phase::Idle
                | Phase::AwaitKeys { .. }
                | Phase::AwaitHttp { .. }
                | Phase::Backoff { .. }
                | Phase::Done) => phase,
            },
            Event::Http(answer) => self.http(phase, answer, out),
            Event::Tick { now } => self.tick(phase, now, out),
            Event::Cancel => match phase {
                Phase::AwaitKeys { .. }
                | Phase::AwaitRedirect { .. }
                | Phase::AwaitHttp { .. }
                | Phase::Backoff { .. } => Self::finish(Failure::Cancelled, out),
                phase @ (Phase::Idle | Phase::Done) => phase,
            },
            Event::Reset => match phase {
                Phase::Done => Phase::Idle,
                phase @ (Phase::Idle
                | Phase::AwaitKeys { .. }
                | Phase::AwaitRedirect { .. }
                | Phase::AwaitHttp { .. }
                | Phase::Backoff { .. }) => phase,
            },
        };
    }

    fn emit_start(result: Result<(Exchange, Request, Awaits), Failure>, out: &mut Queue<Request>) -> Phase {
        match result {
            Ok((exchange, request, awaits)) => {
                out.push(request);
                awaits.hold(exchange)
            }
            Err(failure) => Self::finish(failure, out),
        }
    }

    fn refresh(
        &mut self,
        mut registration: Registration,
        prior: RefreshState,
        now: Time,
    ) -> Result<(Exchange, Request, Awaits), Failure> {
        validate_registration(&registration, &self.limits, false)?;
        self.refresh_values(&mut registration, &prior)?;
        let deadline = now.checked_add(self.limits.sign_in_time).ok_or(Failure::Limit)?;
        let mut exchange = Exchange { registration, grant: Grant::Refresh { prior }, deadline, attempts: 0 };
        let (request, phase) = self.http_request(&mut exchange, now)?;
        Ok((exchange, Request::Http(request), phase))
    }

    fn refresh_values(&self, registration: &mut Registration, prior: &RefreshState) -> Result<(), Failure> {
        if prior.refresh_token.is_empty() || prior.refresh_token.len() > usize_of(self.limits.document.token_bytes) {
            return Err(Failure::Limit);
        }
        if let Err(error) = crate::record::validate_refresh_additions(prior, &self.limits.document) {
            return Err(admit(error));
        }
        if issued_id(registration) {
            let Some(client_id) = &prior.issued_client_id else { return Err(Failure::Refused) };
            if client_id.is_empty() {
                return Err(Failure::Refused);
            }
            registration.client_id.clone_from(client_id);
        }
        Ok(())
    }

    /// Whether `Redirected` would take this redirect at `now`; changes nothing (oauth/README.md, section 3).
    #[must_use = "the caller must judge admission before taking a redirect"]
    pub fn admits(&self, redirect: &Redirect, now: Time) -> Result<(), Inadmissible> {
        let exchange = match &self.phase {
            Phase::AwaitRedirect { exchange } => exchange,
            Phase::Idle | Phase::AwaitKeys { .. } | Phase::AwaitHttp { .. } | Phase::Backoff { .. } | Phase::Done => {
                return Err(Inadmissible::NotWaiting);
            }
        };
        match Self::check_redirect(exchange, redirect, now) {
            Ok(()) => Ok(()),
            Err((why, _failure)) => Err(why),
        }
    }

    /// Whether a sign-in waits for its redirect before its deadline and `state` is
    /// its own, compared in constant time (skein/docs/design/oauth/README.md, 3, and skein/docs/design/oauth/accounts.md, 5).
    #[must_use]
    pub fn awaits_state(&self, state: &[u8], now: Time) -> bool {
        match &self.phase {
            Phase::AwaitRedirect { exchange } => {
                if now >= exchange.deadline {
                    return false;
                }
                match &exchange.grant {
                    Grant::SignIn { state: expected, .. } => skein_lib::sha256::equal(state, expected.expose()),
                    Grant::Refresh { .. } | Grant::Revoke { .. } => false,
                }
            }
            Phase::Idle | Phase::AwaitKeys { .. } | Phase::AwaitHttp { .. } | Phase::Backoff { .. } | Phase::Done => {
                false
            }
        }
    }

    fn check_redirect(exchange: &Exchange, redirect: &Redirect, now: Time) -> Result<(), (Inadmissible, Failure)> {
        if now >= exchange.deadline {
            return Err((Inadmissible::NotWaiting, Failure::InvalidRedirect));
        }
        if redirect.uri != exchange.registration.redirect_uri {
            return Err((Inadmissible::Address, Failure::InvalidRedirect));
        }
        let code = present(secret_ref(redirect.code.as_ref()));
        let error = present(redirect.error.as_deref());
        if code.is_some() == error.is_some() {
            return Err((Inadmissible::Address, Failure::Malformed));
        }
        if code.is_some() && issued_id(&exchange.registration) && present(redirect.client_id.as_deref()).is_none() {
            return Err((Inadmissible::Address, Failure::Incomplete));
        }
        let state = match &exchange.grant {
            Grant::SignIn { state, .. } => state,
            Grant::Refresh { .. } | Grant::Revoke { .. } => return Err((Inadmissible::Address, Failure::Malformed)),
        };
        if !skein_lib::sha256::equal(redirect.state.expose(), state.expose()) {
            return Err((Inadmissible::State, Failure::InvalidRedirect));
        }
        Ok(())
    }

    fn redirected(&mut self, mut exchange: Exchange, redirect: Redirect, now: Time, out: &mut Queue<Request>) -> Phase {
        if let Err((_why, failure)) = Self::check_redirect(&exchange, &redirect, now) {
            return Self::finish(failure, out);
        }
        if present(redirect.error.as_deref()).is_some() {
            return Self::finish(Failure::Refused, out);
        }
        let code = redirect.code.expect("admitted redirect's code");
        let client_id = redirect.client_id;
        if code.len() > usize_of(self.limits.code_bytes) {
            return Self::finish(Failure::Limit, out);
        }
        if issued_id(&exchange.registration) {
            let client_id = client_id.expect("admitted redirect's issued client id");
            if client_id.len() > usize_of(self.limits.document.issued_client_bytes) {
                return Self::finish(Failure::Limit, out);
            }
            exchange.registration.client_id = client_id;
        }
        match &mut exchange.grant {
            Grant::SignIn { code: slot, .. } => *slot = Some(code),
            Grant::Refresh { .. } | Grant::Revoke { .. } => {
                return Self::finish(Failure::Malformed, out);
            }
        }
        match self.http_request(&mut exchange, now) {
            Ok((request, phase)) => {
                out.push(Request::Http(request));
                phase.hold(exchange)
            }
            Err(failure) => Self::finish(failure, out),
        }
    }

    fn http(&mut self, phase: Phase, answer: HttpResponse, out: &mut Queue<Request>) -> Phase {
        match phase {
            Phase::AwaitKeys { exchange, id, deadline } => {
                if answer.id == id {
                    self.keys_answer(exchange, answer, deadline, out)
                } else {
                    Phase::AwaitKeys { exchange, id, deadline }
                }
            }
            Phase::AwaitHttp { exchange, id, deadline } => {
                if answer.id == id {
                    self.answer(exchange, answer, deadline, out)
                } else {
                    Phase::AwaitHttp { exchange, id, deadline }
                }
            }
            phase @ (Phase::Idle | Phase::AwaitRedirect { .. } | Phase::Backoff { .. } | Phase::Done) => phase,
        }
    }

    fn answer(&mut self, exchange: Exchange, answer: HttpResponse, deadline: Time, out: &mut Queue<Request>) -> Phase {
        if exchange.revoking() {
            return self.revocation_answer(exchange, answer, deadline, out);
        }
        if answer.now >= deadline || answer.now >= exchange.deadline {
            return Self::finish(Failure::TimedOut, out);
        }
        if answer.evidence == HttpEvidence::Unsent {
            return self.retry_or_fail(exchange, Failure::Unavailable, answer.now, answer.retry_after, true, out);
        }
        if answer.body.len() > usize_of(self.limits.document.document_bytes) {
            return Self::finish(Failure::Limit, out);
        }
        if (200..300).contains(&answer.status) {
            let response = match Json::from_bytes(&answer.body, &self.limits.document) {
                Ok(value) => decode_response(&value, &self.limits.document),
                Err(error) => Err(error),
            };
            return match response {
                Ok(tokens) => match self.record(&exchange, &tokens, answer.wall) {
                    Ok((record, profile)) => {
                        let missing = missing_scopes(&exchange.registration, &record);
                        out.push(Request::Tokens { record, missing, profile });
                        Phase::Done
                    }
                    Err(failure) => Self::finish(failure, out),
                },
                Err(error) => Self::finish(admit(error), out),
            };
        }
        let error: Result<Option<OAuthError>, DecodeError> = if answer.body.is_empty() {
            Ok(None)
        } else {
            match Json::from_bytes(&answer.body, &self.limits.document) {
                Ok(value) => match decode_error(&value, &self.limits.document) {
                    Ok(error) => Ok(Some(error)),
                    Err(error) => Err(error),
                },
                Err(error) => Err(error),
            }
        };
        let failure = match error {
            Ok(value) => {
                let mut added = false;
                if let (Some(terms), Some(error)) = (&exchange.registration.terms, &value) {
                    for name in &terms.sign_in_again {
                        if name == &error.code {
                            added = true;
                        }
                    }
                }
                if added { Failure::Refused } else { classify(answer.status, value.as_ref(), answer.retry_after) }
            }
            Err(_) => Failure::Malformed,
        };
        let safe = answer.status == 429 && answer.evidence == HttpEvidence::Response;
        self.retry_or_fail(exchange, failure, answer.now, answer.retry_after, safe, out)
    }

    fn record(
        &self,
        exchange: &Exchange,
        response: &TokenResponse,
        wall: Wall,
    ) -> Result<(SavedToken, crate::ProfileClaims), Failure> {
        let profile;
        let prior = match &exchange.grant {
            Grant::Refresh { prior } => {
                return match rotate(
                    prior,
                    response,
                    exchange.registration.metadata_claim.as_ref(),
                    wall,
                    &self.limits.document,
                ) {
                    Ok(record) => Ok((record, self.refresh_profile(exchange, response, wall))),
                    Err(error) => Err(admit(error)),
                };
            }
            Grant::Revoke { .. } => return Err(Failure::Malformed),
            Grant::SignIn { key, generation, nonce, keys, .. } => {
                let identity = match self.identity(&exchange.registration, nonce.as_ref(), keys, response, wall) {
                    Ok(value) => value,
                    Err(check) => return Err(Failure::Unverified { check }),
                };
                profile = identity.profile;
                SavedToken {
                    key: *key,
                    generation: *generation,
                    access_token: Secret::new(Box::new([])),
                    refresh_token: None,
                    metadata: None,
                    expires_at: wall,
                    earliest_refresh_at: None,
                    issued_client_id: if issued_id(&exchange.registration) {
                        Some(exchange.registration.client_id.clone())
                    } else {
                        None
                    },
                    granted_scopes: None,
                    subject: identity.subject,
                    id_token: identity.token,
                }
            }
        };
        match crate::record::complete(
            prior,
            response,
            exchange.registration.metadata_claim.as_ref(),
            wall,
            &self.limits.document,
        ) {
            Ok(record) => Ok((record, profile)),
            Err(error) => Err(admit(error)),
        }
    }

    fn refresh_profile(&self, exchange: &Exchange, response: &TokenResponse, wall: Wall) -> crate::ProfileClaims {
        let prior = match &exchange.grant {
            Grant::Refresh { prior } => prior,
            Grant::SignIn { .. } | Grant::Revoke { .. } => return crate::ProfileClaims::default(),
        };
        let Some(openid) = (match &exchange.registration.terms {
            Some(terms) => terms.openid.as_ref(),
            None => None,
        }) else {
            return crate::ProfileClaims::default();
        };
        let Some(token) = &response.id_token else { return crate::ProfileClaims::default() };
        let Some(subject) = &prior.subject else { return crate::ProfileClaims::default() };
        let expected = crate::jwt::RefreshExpected {
            issuer: &openid.issuer,
            client_id: &exchange.registration.client_id,
            subject,
        };
        crate::jwt::refreshed_profile(
            token.expose(),
            &expected,
            openid.profile,
            wall,
            self.limits.skew,
            &self.limits.document,
        )
    }

    fn retry_or_fail(
        &mut self,
        exchange: Exchange,
        failure: Failure,
        now: Time,
        retry_after: Duration,
        safe: bool,
        out: &mut Queue<Request>,
    ) -> Phase {
        let transient = match failure {
            Failure::Unavailable | Failure::TimedOut | Failure::RateLimited { .. } => true,
            Failure::Refused
            | Failure::ClientRejected
            | Failure::Malformed
            | Failure::Incomplete
            | Failure::Unverified { .. }
            | Failure::InvalidRedirect
            | Failure::Limit
            | Failure::Cancelled => false,
        };
        if safe && transient && exchange.attempts < self.limits.max_attempts {
            let shift = u64::from(exchange.attempts.saturating_sub(1)).min(63);
            let factor = 1_u64.checked_shl(u32::try_from(shift).expect("shift fits")).unwrap_or(u64::MAX);
            let delay =
                self.limits.backoff_base.saturating_mul(factor).min(self.limits.backoff_ceiling).max(retry_after);
            if let Some(due) = now.checked_add(delay)
                && due < exchange.deadline
            {
                return Phase::Backoff { exchange, due };
            }
        }
        if exchange.revoking() {
            Self::revoked(false, out)
        } else {
            Self::finish(if exchange.fetching() { fetch::failure() } else { failure }, out)
        }
    }

    fn tick(&mut self, phase: Phase, now: Time, out: &mut Queue<Request>) -> Phase {
        match phase {
            Phase::AwaitRedirect { exchange } => {
                if now >= exchange.deadline {
                    Self::finish(Failure::TimedOut, out)
                } else {
                    Phase::AwaitRedirect { exchange }
                }
            }
            Phase::AwaitKeys { exchange, id, deadline } => {
                if now >= deadline {
                    self.retry_or_fail(exchange, Failure::TimedOut, now, Duration::ZERO, true, out)
                } else {
                    Phase::AwaitKeys { exchange, id, deadline }
                }
            }
            Phase::AwaitHttp { exchange, id, deadline } => {
                if now < deadline {
                    Phase::AwaitHttp { exchange, id, deadline }
                } else if exchange.revoking() {
                    Self::revoked(false, out)
                } else {
                    Self::finish(Failure::TimedOut, out)
                }
            }
            Phase::Backoff { mut exchange, due } => {
                if now < due {
                    return Phase::Backoff { exchange, due };
                }
                match self.http_request(&mut exchange, now) {
                    Ok((request, awaits)) => {
                        out.push(Request::Http(request));
                        awaits.hold(exchange)
                    }
                    Err(failure) => {
                        if exchange.revoking() {
                            Self::revoked(false, out)
                        } else {
                            Self::finish(if exchange.fetching() { fetch::failure() } else { failure }, out)
                        }
                    }
                }
            }
            phase @ (Phase::Idle | Phase::Done) => phase,
        }
    }

    fn http_request(&mut self, exchange: &mut Exchange, now: Time) -> Result<(HttpRequest, Awaits), Failure> {
        if now >= exchange.deadline || exchange.attempts >= self.limits.max_attempts {
            return Err(Failure::TimedOut);
        }
        let deadline = now.checked_add(self.limits.request_time).ok_or(Failure::Limit)?.min(exchange.deadline);
        if exchange.fetching() {
            return self.keys_request(exchange, deadline);
        }
        let body = token_body(exchange, &self.limits)?;
        let id = self.next_id;
        self.next_id = self.next_id.checked_add(1).ok_or(Failure::Limit)?;
        exchange.attempts = exchange.attempts.checked_add(1).ok_or(Failure::Limit)?;
        let content_type = match (exchange.revoking(), exchange.registration.wire) {
            (false, WireFormat::Json) => b"application/json".as_slice(),
            (true, WireFormat::Json | WireFormat::Form) | (false, WireFormat::Form) => {
                b"application/x-www-form-urlencoded".as_slice()
            }
        };
        let request = HttpRequest {
            id,
            method: Method::Post,
            endpoint: bytes::copy_of(if exchange.revoking() {
                revoke::endpoint(&exchange.registration).ok_or(Failure::Malformed)?
            } else {
                &exchange.registration.token_endpoint
            }),
            content_type,
            body,
            deadline,
        };
        Ok((request, Awaits::Http { id, deadline }))
    }

    fn finish(failure: Failure, out: &mut Queue<Request>) -> Phase {
        out.push(Request::Failed { failure });
        Phase::Done
    }
}

fn admit(error: DecodeError) -> Failure {
    match error {
        DecodeError::TooLarge => Failure::Limit,
        DecodeError::Malformed | DecodeError::Missing | DecodeError::WrongType | DecodeError::Version => {
            Failure::Malformed
        }
    }
}

fn token_body(exchange: &Exchange, limits: &ClientLimits) -> Result<Box<[u8]>, Failure> {
    let registration = &exchange.registration;
    let (grant_type, name, value, redirect, verifier) = match &exchange.grant {
        Grant::SignIn { code: Some(code), verifier, .. } => (
            b"authorization_code".as_slice(),
            b"code".as_slice(),
            code.expose(),
            Some(registration.redirect_uri.as_ref()),
            secret_ref(verifier.as_ref()),
        ),
        Grant::SignIn { code: None, .. } => return Err(Failure::Malformed),
        Grant::Revoke { prior } => return revoke::body(registration, prior, limits),
        Grant::Refresh { prior } => {
            (b"refresh_token".as_slice(), b"refresh_token".as_slice(), prior.refresh_token.expose(), None, None)
        }
    };
    let fields = [
        resource_field(registration),
        Some((b"grant_type".as_slice(), grant_type)),
        Some((b"client_id".as_slice(), registration.client_id.as_ref())),
        Some((name, value)),
        match redirect {
            Some(item) => Some((b"redirect_uri".as_slice(), item)),
            None => None,
        },
        match verifier {
            Some(item) => Some((b"code_verifier".as_slice(), item)),
            None => None,
        },
        match registration.client_secret.as_ref() {
            Some(item) => Some((b"client_secret".as_slice(), item.expose())),
            None => None,
        },
    ];
    match registration.wire {
        WireFormat::Form => form_fields(&fields, limits.request_bytes),
        WireFormat::Json => json_fields(&fields, limits),
    }
}

fn json_fields(fields: &[Option<(&[u8], &[u8])>], limits: &ClientLimits) -> Result<Box<[u8]>, Failure> {
    let writer_limits = skein_json::writer::Limits { depth: 2, length: limits.request_bytes };
    let mut measure = Encoder::measure(&writer_limits);
    measure.object_start();
    for field in fields {
        let Some((key, value)) = field else { continue };
        measure.key(key);
        measure.string(value);
    }
    measure.object_end();
    let length = match measure.measured() {
        Ok(length) => length,
        Err(skein_json::writer::Refusal::TooLong | skein_json::writer::Refusal::TooDeep) => return Err(Failure::Limit),
        Err(skein_json::writer::Refusal::Text | skein_json::writer::Refusal::Number) => return Err(Failure::Malformed),
    };
    let mut writer = Encoder::write(length, &writer_limits);
    writer.object_start();
    for field in fields {
        let Some((key, value)) = field else { continue };
        writer.key(key);
        writer.string(value);
    }
    writer.object_end();
    Ok(writer.finish())
}

fn present(value: Option<&[u8]>) -> Option<&[u8]> {
    match value {
        Some(value) if !value.is_empty() => Some(value),
        Some(_) | None => None,
    }
}

impl core::fmt::Debug for HttpRequest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HttpRequest")
            .field("id", &self.id)
            .field("method", &self.method)
            .field("endpoint", &self.endpoint)
            .field("content_type", &self.content_type)
            .field("body_len", &self.body.len())
            .field("deadline", &self.deadline)
            .finish()
    }
}

impl core::fmt::Debug for HttpResponse {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HttpResponse")
            .field("id", &self.id)
            .field("status", &self.status)
            .field("body_len", &self.body.len())
            .field("retry_after", &self.retry_after)
            .field("evidence", &self.evidence)
            .field("now", &self.now)
            .field("wall", &self.wall)
            .finish()
    }
}

impl core::fmt::Debug for Phase {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Phase::Idle => f.write_str("Idle"),
            Phase::AwaitRedirect { .. } => f.write_str("AwaitRedirect"),
            Phase::AwaitKeys { id, deadline, .. } => {
                f.debug_struct("AwaitKeys").field("id", id).field("deadline", deadline).finish()
            }
            Phase::AwaitHttp { id, deadline, .. } => {
                f.debug_struct("AwaitHttp").field("id", id).field("deadline", deadline).finish()
            }
            Phase::Backoff { due, .. } => f.debug_struct("Backoff").field("due", due).finish(),
            Phase::Done => f.write_str("Done"),
        }
    }
}

impl core::fmt::Debug for Client {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Client").field("phase", &self.phase).finish_non_exhaustive()
    }
}
