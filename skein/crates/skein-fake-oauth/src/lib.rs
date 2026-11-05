//! Independent bounded OAuth issuer. No IO, clock reads, or provider policy.
//! Credentials and carrier debug views follow skein/docs/design/oauth.md, 4
//! and skein/docs/design/lib.md, 6.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use skein_json::Token;
use skein_lib::{Duration, List, Queue, Secret, Time, Wall, Writer, bytes, usize_of};
use skein_oauth::{
    HttpEvidence, HttpRequest, HttpResponse, Json, Limits as DocumentLimits, OAuthError, TokenResponse, challenge,
    encode_error, encode_response,
};

mod key;
mod openid;
pub use openid::{Fault, OpenId};
mod terms;
pub use terms::{ISSUED, Terms};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub document: DocumentLimits,
    pub uri_bytes: u32,
    pub request_bytes: u32,
    pub codes: u32,
    pub rotations: u32,
    pub plans: u32,
    /// Names retained from the last request beyond OAuth’s own fields.
    pub carried: u32,
}

/// Whether these limits are representable: every edge bound they size is finite
/// and every capacity they derive fits (programming-model.md, 6.3).
/// Codes and rotations are nonempty, request-field slots fit u32, JSON input
/// has a representable end probe, and saved-token fields fit their schema.
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    let document = &limits.document;
    let record = skein_oauth::v3::Limits {
        saved_token_access_token: document.token_bytes,
        saved_token_refresh_token_some: document.token_bytes,
        saved_token_metadata_some: document.token_bytes,
        saved_token_issued_client_id_some: document.issued_client_bytes,
        saved_token_granted_scopes_some: document.granted_bytes,
        saved_token_subject_some: document.subject_bytes,
        saved_token_id_token_some: document.id_token_bytes,
    };
    limits.codes > 0
        && limits.rotations > 0
        && limits.carried.checked_add(10).is_some()
        && document.document_bytes < u32::MAX
        && skein_oauth::v3::check_limits(&record).is_ok()
}

#[derive(Debug)]
pub struct Config {
    pub authorization_url: Box<[u8]>,
    pub token_endpoint: Box<[u8]>,
    pub client_id: Box<[u8]>,
    pub client_secret: Option<Secret>,
    pub redirect_uri: Box<[u8]>,
    pub refresh_token: Secret,
    /// Optional issued-registration and revocation terms.
    pub terms: Option<Terms>,
}

pub enum Body {
    Token(TokenResponse),
    Error(OAuthError),
    Raw(Box<[u8]>),
}

#[derive(Debug)]
pub struct Plan {
    pub status: u16,
    pub body: Body,
    pub delay: Duration,
    pub retry_after: Duration,
}

pub enum Event {
    Authorize { url: Box<[u8]>, now: Time },
    Post { request: HttpRequest, now: Time, wall: Wall },
    Get { id: u64, endpoint: Box<[u8]>, now: Time, wall: Wall },
    Tick { now: Time, wall: Wall },
    LoseResponse,
}

#[derive(Debug)]
pub enum Request {
    Redirect { uri: Box<[u8]>, state: Secret, code: Secret, client_id: Option<Box<[u8]>> },
    Http(HttpResponse),
    Refused,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    Limits,
    Full,
    Plan,
}

/// Who signs in at the fake's authorization (oauth.md, 5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Account {
    pub subject: Box<[u8]>,
    pub email: Option<Email>,
    pub name: Option<Box<[u8]>>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Email {
    pub address: Box<[u8]>,
    pub verified: bool,
}

struct Code {
    value: Secret,
    challenge: Option<Secret>,
    redirect_uri: Box<[u8]>,
    nonce: Option<Secret>,
    account: Option<Account>,
    used: bool,
    expires: Time,
}
struct Pending {
    id: u64,
    status: u16,
    body: Box<[u8]>,
    retry_after: Duration,
    due: Time,
    revoke: bool,
}
struct Field {
    key: Box<[u8]>,
    value: Secret,
}

pub struct Issuer {
    config: Config,
    account: Option<Account>,
    limits: Limits,
    plans: Queue<Plan>,
    codes: List<Code>,
    spent: List<Secret>,
    generation: u64,
    posts: u64,
    next_code: u64,
    pending: Option<Pending>,
    carried: List<Box<[u8]>>,
    authorizations: usize,
    revoked: bool,
}

impl core::fmt::Debug for Body {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Body::Token(value) => f.debug_tuple("Token").field(value).finish(),
            Body::Error(value) => f.debug_tuple("Error").field(value).finish(),
            Body::Raw(value) => f.debug_struct("Raw").field("len", &value.len()).finish(),
        }
    }
}

impl core::fmt::Debug for Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Event::Authorize { url, now } => {
                f.debug_struct("Authorize").field("url_len", &url.len()).field("now", now).finish()
            }
            Event::Post { request, now, wall } => {
                f.debug_struct("Post").field("request", request).field("now", now).field("wall", wall).finish()
            }
            Event::Get { id, endpoint, now, wall } => f
                .debug_struct("Get")
                .field("id", id)
                .field("endpoint", endpoint)
                .field("now", now)
                .field("wall", wall)
                .finish(),
            Event::Tick { now, wall } => f.debug_struct("Tick").field("now", now).field("wall", wall).finish(),
            Event::LoseResponse => f.write_str("LoseResponse"),
        }
    }
}

impl core::fmt::Debug for Issuer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Issuer").field("posts", &self.posts).finish_non_exhaustive()
    }
}

impl Issuer {
    pub fn new(config: Config, limits: Limits) -> Result<Issuer, Error> {
        let secret_too_large = match &config.client_secret {
            Some(value) => value.len() > usize_of(limits.document.client_bytes),
            None => false,
        };
        if !valid(&limits)
            || config.client_id.is_empty()
            || config.refresh_token.is_empty()
            || config.client_id.len() > usize_of(limits.document.client_bytes)
            || config.refresh_token.len() > usize_of(limits.document.token_bytes)
            || config.authorization_url.len() > usize_of(limits.uri_bytes)
            || config.token_endpoint.len() > usize_of(limits.uri_bytes)
            || config.redirect_uri.len() > usize_of(limits.uri_bytes)
            || !terms::valid(config.terms.as_ref(), &limits)
            || secret_too_large
        {
            return Err(Error::Limits);
        }
        #[expect(clippy::manual_map, reason = "programming-model.md, 10.3 forbids closures in step code")]
        let account = match openid::configured(&config) {
            Some(openid) => Some(Account { subject: openid.subject.clone(), email: None, name: None }),
            None => None,
        };
        Ok(Issuer {
            config,
            account,
            limits,
            plans: Queue::with_capacity(limits.plans),
            codes: List::with_capacity(limits.codes),
            spent: List::with_capacity(limits.rotations),
            generation: 0,
            posts: 0,
            next_code: 1,
            pending: None,
            carried: List::with_capacity(limits.carried),
            authorizations: 0,
            revoked: false,
        })
    }

    /// The account the following authorizations sign in, until another is set.
    /// `Err(Error::Limits)` for an empty subject or any part past `limits.document.string_bytes`;
    /// `Err(Error::Plan)` without `OpenID` terms.
    pub fn sign_in_as(&mut self, account: Account) -> Result<(), Error> {
        if openid::configured(&self.config).is_none() {
            return Err(Error::Plan);
        }
        let cap = usize_of(self.limits.document.string_bytes);
        let email_too_large = match &account.email {
            Some(email) => email.address.len() > cap,
            None => false,
        };
        let name_too_large = match &account.name {
            Some(name) => name.len() > cap,
            None => false,
        };
        if account.subject.is_empty() || account.subject.len() > cap || email_too_large || name_too_large {
            return Err(Error::Limits);
        }
        self.account = Some(account);
        Ok(())
    }

    pub fn queue(&mut self, plan: Plan) -> Result<(), Error> {
        if self.plans.room() == 0 {
            return Err(Error::Full);
        }
        if !(200..600).contains(&plan.status) {
            return Err(Error::Plan);
        }
        match &plan.body {
            Body::Token(value) => {
                if encode_response(value, &self.limits.document).is_err() {
                    return Err(Error::Plan);
                }
            }
            Body::Error(value) => {
                if encode_error(value, &self.limits.document).is_err() {
                    return Err(Error::Plan);
                }
            }
            Body::Raw(value) => {
                if value.len() > usize_of(self.limits.document.document_bytes) {
                    return Err(Error::Plan);
                }
            }
        }
        self.plans.push(plan);
        Ok(())
    }

    #[must_use]
    pub const fn posts(&self) -> u64 {
        self.posts
    }
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        Some(self.pending.as_ref()?.due)
    }

    /// Names beyond OAuth’s own fields in the last authorization or POST, in order.
    #[must_use]
    pub fn carried(&self) -> &[Box<[u8]>] {
        self.carried.as_slice()
    }

    /// Access tokens in queued successful token plans; the script mints none.
    #[must_use]
    pub fn granted(&self) -> Box<[Secret]> {
        let mut tokens = List::with_capacity(self.plans.len());
        for plan in &self.plans {
            if (200..300).contains(&plan.status) {
                match &plan.body {
                    Body::Token(token) => {
                        tokens.push(token.access_token.clone()).expect("one per plan");
                    }
                    Body::Error(_) | Body::Raw(_) => {}
                }
            }
        }
        tokens.into_boxed()
    }

    pub fn step(&mut self, event: Event, out: &mut Queue<Request>) {
        match event {
            Event::Authorize { url, now } => self.authorize(&url, now, out),
            Event::Post { request, now, wall } => self.post(request, now, wall, out),
            Event::Get { id, endpoint, now, wall } => {
                openid::get(&self.config, &self.limits, id, &endpoint, now, wall, out);
            }
            Event::Tick { now, wall } => self.deliver(now, wall, out),
            Event::LoseResponse => self.pending = None,
        }
    }

    fn authorize(&mut self, url: &[u8], now: Time, out: &mut Queue<Request>) {
        self.carried.clear();
        let Some(query_at) = bytes::find(url, b"?") else {
            out.push(Request::Refused);
            return;
        };
        let Some(base) = url.get(..query_at) else {
            out.push(Request::Refused);
            return;
        };
        if base != self.config.authorization_url.as_ref() {
            out.push(Request::Refused);
            return;
        }
        let Some(query) = url.get(query_at.saturating_add(1)..) else {
            out.push(Request::Refused);
            return;
        };
        let Ok(fields) = form_fields(query, &self.limits) else {
            out.push(Request::Refused);
            return;
        };
        if !terms::observe(&fields, &mut self.carried, &self.limits) {
            out.push(Request::Refused);
            return;
        }
        let redirect = field(&fields, b"redirect_uri");
        if let Some(nonce) = field(&fields, b"nonce")
            && nonce.len() > usize_of(self.limits.document.string_bytes)
        {
            out.push(Request::Refused);
            return;
        }
        if field(&fields, b"response_type") != Some(b"code".as_slice())
            || !terms::authorization_client(&self.config, self.authorizations, field(&fields, b"client_id"))
            || !terms::redirect(&self.config, redirect)
            || terms::too_long(redirect, self.limits.uri_bytes)
            || self.codes.room() == 0
        {
            out.push(Request::Refused);
            return;
        }
        let Some(state) = field(&fields, b"state") else {
            out.push(Request::Refused);
            return;
        };
        if state.len() < 16 || state.len() > usize_of(self.limits.document.string_bytes) {
            out.push(Request::Refused);
            return;
        }
        let challenge = field(&fields, b"code_challenge");
        let method = field(&fields, b"code_challenge_method");
        if self.config.client_secret.is_none() && (challenge.is_none() || method != Some(b"S256".as_slice())) {
            out.push(Request::Refused);
            return;
        }
        if challenge.is_some() && method != Some(b"S256".as_slice()) {
            out.push(Request::Refused);
            return;
        }
        let code = Secret::new(code_bytes(self.next_code));
        let Some(next) = self.next_code.checked_add(1) else {
            out.push(Request::Refused);
            return;
        };
        self.next_code = next;
        let entry = Code {
            value: code.clone(),
            challenge: copy_optional(challenge),
            redirect_uri: bytes::copy_of(redirect.expect("redirect checked")),
            nonce: copy_optional(field(&fields, b"nonce")),
            account: self.account.clone(),
            used: false,
            expires: now.saturating_add(Duration::from_secs(120)),
        };
        if self.codes.push(entry).is_err() {
            out.push(Request::Refused);
            return;
        }
        self.authorizations = self.authorizations.saturating_add(1);
        out.push(Request::Redirect {
            uri: bytes::copy_of(redirect.expect("redirect checked")),
            client_id: terms::issued(&self.config, self.authorizations),
            state: Secret::new(bytes::copy_of(state)),
            code,
        });
    }

    #[expect(clippy::manual_map, reason = "programming-model.md, 10.3 forbids closures in step code")]
    #[expect(clippy::too_many_lines, reason = "the bounded token POST keeps its credential conversions in place")]
    fn post(&mut self, request: HttpRequest, now: Time, wall: Wall, out: &mut Queue<Request>) {
        self.carried.clear();
        if self.pending.is_some() {
            out.push(Request::Refused);
            return;
        }
        let Some(posts) = self.posts.checked_add(1) else {
            out.push(Request::Refused);
            return;
        };
        self.posts = posts;
        let revocation = terms::revocation(&self.config, &request.endpoint);
        if (!revocation && request.endpoint != self.config.token_endpoint)
            || request.body.len() > usize_of(self.limits.request_bytes)
        {
            self.refuse(request.id, now, wall, out);
            return;
        }
        let fields = if request.content_type == b"application/json" {
            json_fields(&request.body, &self.limits)
        } else if request.content_type == b"application/x-www-form-urlencoded" {
            form_fields(&request.body, &self.limits)
        } else {
            Err(Error::Plan)
        };
        let Ok(fields) = fields else {
            self.refuse(request.id, now, wall, out);
            return;
        };
        if !terms::observe(&fields, &mut self.carried, &self.limits) {
            out.push(Request::Refused);
            return;
        }
        if field(&fields, b"client_id") != Some(terms::latest(&self.config, self.authorizations))
            || field(&fields, b"client_secret") != expose_optional(self.config.client_secret.as_ref())
        {
            self.refuse(request.id, now, wall, out);
            return;
        }
        let accepted = self.accepted(&fields, revocation, now);
        if !accepted {
            self.refuse(request.id, now, wall, out);
            return;
        }
        let Some(plan) = self.plans.pop() else {
            out.push(Request::Refused);
            return;
        };
        let (body, next_refresh) = match plan.body {
            Body::Token(mut value) => {
                if self.add_id_token(&mut value, &fields, revocation, plan.status, wall).is_err() {
                    out.push(Request::Refused);
                    return;
                }
                let next = if !revocation && (200..300).contains(&plan.status) {
                    match &value.refresh_token {
                        Some(token) => Some(token.clone()),
                        None => None,
                    }
                } else {
                    None
                };
                let Ok(body) = encode_response(&value, &self.limits.document) else {
                    out.push(Request::Refused);
                    return;
                };
                (body, next)
            }
            Body::Error(value) => {
                let Ok(body) = encode_error(&value, &self.limits.document) else {
                    out.push(Request::Refused);
                    return;
                };
                (body, None)
            }
            Body::Raw(value) => (value, None),
        };
        if let Some(next) = next_refresh
            && next != self.config.refresh_token
        {
            if self.spent.room() == 0 {
                out.push(Request::Refused);
                return;
            }
            for old in self.spent.as_slice() {
                if old == &next {
                    out.push(Request::Refused);
                    return;
                }
            }
            let Some(generation) = self.generation.checked_add(1) else {
                out.push(Request::Refused);
                return;
            };
            let old = core::mem::replace(&mut self.config.refresh_token, next);
            self.spent.push(old).expect("room checked");
            self.generation = generation;
        }
        self.pending = Some(Pending {
            id: request.id,
            status: plan.status,
            body,
            retry_after: plan.retry_after,
            due: now.saturating_add(plan.delay),
            revoke: revocation && (200..300).contains(&plan.status),
        });
        self.deliver(now, wall, out);
    }

    #[expect(clippy::manual_map, reason = "programming-model.md, 10.3 forbids closures in step code")]
    fn add_id_token(
        &self,
        value: &mut TokenResponse,
        fields: &List<Field>,
        revocation: bool,
        status: u16,
        wall: Wall,
    ) -> Result<(), Error> {
        if !revocation
            && (200..300).contains(&status)
            && value.id_token.is_none()
            && field(fields, b"grant_type") == Some(b"authorization_code".as_slice())
            && let Some(openid) = openid::configured(&self.config)
        {
            let code = self.code(fields).ok_or(Error::Plan)?;
            value.id_token = match openid::token(
                openid,
                &self.limits,
                field(fields, b"client_id").expect("accepted client"),
                code.account.as_ref().ok_or(Error::Plan)?,
                expose_optional(code.nonce.as_ref()),
                wall,
            )? {
                Some(token) => Some(Secret::new(token)),
                None => None,
            };
        }
        Ok(())
    }

    #[expect(clippy::manual_find, reason = "programming-model.md, 10.3 forbids closures in step code")]
    fn code(&self, fields: &List<Field>) -> Option<&Code> {
        for entry in self.codes.as_slice() {
            if Some(entry.value.expose()) == field(fields, b"code") {
                return Some(entry);
            }
        }
        None
    }

    fn accepted(&mut self, fields: &List<Field>, revocation: bool, now: Time) -> bool {
        if revocation {
            !self.revoked
                && field(fields, b"token") == Some(self.config.refresh_token.expose())
                && field(fields, b"token_type_hint") == Some(b"refresh_token".as_slice())
        } else {
            match field(fields, b"grant_type") {
                Some(b"refresh_token") => self.refresh(fields),
                Some(b"authorization_code") => self.redeem(fields, now),
                Some(_) | None => false,
            }
        }
    }

    fn refresh(&self, fields: &List<Field>) -> bool {
        !self.revoked && field(fields, b"refresh_token") == Some(self.config.refresh_token.expose())
    }

    fn redeem(&mut self, fields: &List<Field>, now: Time) -> bool {
        let Some(code) = field(fields, b"code") else { return false };
        for index in 0..self.codes.len() {
            let Some(entry) = self.codes.get_mut(index) else { return false };
            if entry.value.expose() == code && !entry.used && entry.expires > now {
                if field(fields, b"redirect_uri") != Some(entry.redirect_uri.as_ref()) {
                    return false;
                }
                if let Some(expected) = &entry.challenge {
                    let Some(verifier) = field(fields, b"code_verifier") else { return false };
                    let Ok(got) = challenge(verifier) else { return false };
                    if got.as_ref() != expected.expose() {
                        return false;
                    }
                }
                entry.used = true;
                return true;
            }
        }
        false
    }

    fn refuse(&mut self, id: u64, now: Time, wall: Wall, out: &mut Queue<Request>) {
        let error =
            OAuthError { code: bytes::copy_of(b"invalid_grant"), detail: bytes::copy_of(b"credential refused") };
        let body = match encode_error(&error, &self.limits.document) {
            Ok(body) => body,
            Err(_) => bytes::copy_of(b"{}"),
        };
        self.pending = Some(Pending { id, status: 400, body, retry_after: Duration::ZERO, due: now, revoke: false });
        self.deliver(now, wall, out);
    }

    fn deliver(&mut self, now: Time, wall: Wall, out: &mut Queue<Request>) {
        let Some(pending) = self.pending.take() else { return };
        if now < pending.due {
            self.pending = Some(pending);
            return;
        }
        if pending.revoke {
            self.revoked = true;
        }
        out.push(Request::Http(HttpResponse {
            id: pending.id,
            status: pending.status,
            body: pending.body,
            retry_after: pending.retry_after,
            evidence: HttpEvidence::Response,
            now,
            wall,
        }));
    }
}

fn code_bytes(number: u64) -> Box<[u8]> {
    let mut out = [0_u8; 16];
    for (at, slot) in out.iter_mut().enumerate() {
        let shift = u32::try_from(15_usize.saturating_sub(at).saturating_mul(4)).expect("shift fits");
        *slot = bytes::hex_digit(u8::try_from((number >> shift) & 15_u64).expect("nibble fits"));
    }
    bytes::copy_of(&out)
}

fn field<'a>(fields: &'a List<Field>, key: &[u8]) -> Option<&'a [u8]> {
    let mut found = None;
    for entry in fields.as_slice() {
        if entry.key.as_ref() == key {
            if found.is_some() {
                return None;
            }
            found = Some(entry.value.expose());
        }
    }
    found
}

fn json_fields(body: &[u8], limits: &Limits) -> Result<List<Field>, Error> {
    let value = Json::from_bytes(body, &limits.document).or(Err(Error::Plan))?;
    let tokens = value.as_tokens();
    if tokens.first() != Some(&Token::ObjectStart) || tokens.last() != Some(&Token::ObjectEnd) {
        return Err(Error::Plan);
    }
    let mut fields = List::with_capacity(10_u32.checked_add(limits.carried).ok_or(Error::Limits)?);
    let mut at = 1_usize;
    while at < tokens.len().saturating_sub(1) {
        let Some(Token::Key(key)) = tokens.get(at) else { return Err(Error::Plan) };
        let Some(Token::String(value)) = tokens.get(at.saturating_add(1)) else { return Err(Error::Plan) };
        fields.push(Field { key: key.clone(), value: Secret::new(value.clone()) }).or(Err(Error::Plan))?;
        at = at.saturating_add(2);
    }
    Ok(fields)
}

fn form_fields(body: &[u8], limits: &Limits) -> Result<List<Field>, Error> {
    if body.len() > usize_of(limits.request_bytes) {
        return Err(Error::Plan);
    }
    let mut fields = List::with_capacity(10_u32.checked_add(limits.carried).ok_or(Error::Limits)?);
    let mut at = 0_usize;
    loop {
        let remaining = body.get(at..).ok_or(Error::Plan)?;
        let end = match bytes::find(remaining, b"&") {
            Some(offset) => offset,
            None => remaining.len(),
        };
        let pair = remaining.get(..end).ok_or(Error::Plan)?;
        let Some(equal) = bytes::find(pair, b"=") else { return Err(Error::Plan) };
        let key = percent_decode(pair.get(..equal).ok_or(Error::Plan)?, limits.request_bytes)?;
        let value = percent_decode(pair.get(equal.saturating_add(1)..).ok_or(Error::Plan)?, limits.request_bytes)?;
        fields.push(Field { key, value: Secret::new(value) }).or(Err(Error::Plan))?;
        if end == remaining.len() {
            break;
        }
        let position = at.checked_add(end).ok_or(Error::Plan)?;
        at = position.checked_add(1).ok_or(Error::Plan)?;
    }
    Ok(fields)
}

fn percent_decode(value: &[u8], cap: u32) -> Result<Box<[u8]>, Error> {
    if value.len() > usize_of(cap) {
        return Err(Error::Plan);
    }
    let mut size = 0_usize;
    let mut at = 0_usize;
    while at < value.len() {
        size = size.checked_add(1).ok_or(Error::Plan)?;
        let byte = *value.get(at).ok_or(Error::Plan)?;
        at = at.checked_add(if byte == b'%' { 3 } else { 1 }).ok_or(Error::Plan)?;
        if at > value.len() {
            return Err(Error::Plan);
        }
    }
    let mut writer = Writer::new(size);
    at = 0;
    while at < value.len() {
        let byte = *value.get(at).ok_or(Error::Plan)?;
        if byte == b'%' {
            let high = bytes::hex_value(*value.get(at.saturating_add(1)).ok_or(Error::Plan)?).ok_or(Error::Plan)?;
            let low = bytes::hex_value(*value.get(at.saturating_add(2)).ok_or(Error::Plan)?).ok_or(Error::Plan)?;
            writer.put(&[(high << 4_u32) | low]).expect("measured decode");
            at = at.saturating_add(3);
        } else {
            writer.put(&[if byte == b'+' { b' ' } else { byte }]).expect("measured decode");
            at = at.saturating_add(1);
        }
    }
    Ok(writer.finish())
}

fn copy_optional(value: Option<&[u8]>) -> Option<Secret> {
    let value = value?;
    Some(Secret::new(bytes::copy_of(value)))
}

fn expose_optional(value: Option<&Secret>) -> Option<&[u8]> {
    match value {
        Some(value) => Some(value.expose()),
        None => None,
    }
}
