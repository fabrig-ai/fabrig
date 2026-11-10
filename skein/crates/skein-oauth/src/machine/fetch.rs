//! A sign-in's bounded GETs, retries and move to its visit (oauth/README.md, section 3).
use super::{
    Awaits, Client, ClientLimits, Exchange, Failure, HttpEvidence, HttpRequest, HttpResponse, Method, Phase,
    Registration, Request,
};
use super::{Grant, SignInValues, admit};
use crate::record::secret_ref;
use crate::redirect::{authorization_url, has_openid, sign_in_values, validate_registration};
use crate::{Check, keys, pkce};
use alloc::boxed::Box;
use skein_lib::{Queue, Secret, Time, Writer, usize_of};

pub(super) enum Fetch {
    Metadata { endpoint: Box<[u8]> },
    Keys { endpoint: Box<[u8]> },
}
pub(super) fn failure() -> Failure {
    Failure::Unverified { check: Check::Keys }
}

pub(super) fn initial(registration: &Registration, limits: &ClientLimits) -> Result<Fetch, Failure> {
    let openid = registration.terms.as_ref().ok_or(failure())?.openid.as_ref().ok_or(failure())?;
    if let Some(endpoint) = &openid.key_set {
        return Ok(Fetch::Keys { endpoint: endpoint.clone() });
    }
    let suffix = b"/.well-known/openid-configuration";
    let size = openid.issuer.len().checked_add(suffix.len()).ok_or(failure())?;
    if size > usize_of(limits.uri_bytes) {
        return Err(failure());
    }
    let mut out = Writer::new(size);
    out.put(&openid.issuer).expect("measured metadata URL");
    out.put(suffix).expect("measured metadata URL");
    Ok(Fetch::Metadata { endpoint: out.finish() })
}

impl Client {
    pub(super) fn sign_in(
        &mut self,
        mut registration: Registration,
        values: SignInValues,
    ) -> Result<(Exchange, Request, Awaits), Failure> {
        let SignInValues { key, generation, state, verifier, nonce, port, issued_client_id, id_token, subject, now } =
            values;
        validate_registration(&registration, &self.limits, true)?;
        sign_in_values(&mut registration, secret_ref(nonce.as_ref()), port, &self.limits)?;
        if state.len() < 16 || state.len() > usize_of(self.limits.state_bytes) {
            return Err(Failure::Limit);
        }
        let pkce_required = registration.client_secret.is_none() || registration.pkce_for_confidential;
        let challenge = match (&verifier, pkce_required) {
            (Some(value), true) => Some(match pkce::challenge(value.expose()) {
                Ok(challenge) => challenge,
                Err(error) => return Err(admit(error)),
            }),
            (None, false) => None,
            (Some(_), false) | (None, true) => return Err(Failure::Malformed),
        };
        let url = authorization_url(
            &registration,
            state.expose(),
            challenge.as_deref(),
            secret_ref(nonce.as_ref()),
            [issued_client_id.as_deref(), secret_ref(id_token.as_ref()), subject.as_deref()],
            &self.limits,
        )?;
        let deadline = now.checked_add(self.limits.sign_in_time).ok_or(Failure::Limit)?;
        let fetch = if has_openid(&registration) { Some(initial(&registration, &self.limits)?) } else { None };
        let (visit, url) = if fetch.is_some() { (Some(url), None) } else { (None, Some(url)) };
        let mut exchange = Exchange {
            registration,
            grant: Grant::SignIn {
                key,
                generation,
                code: None,
                state,
                verifier,
                nonce,
                visit,
                fetch,
                keys: Box::default(),
            },
            deadline,
            attempts: 0,
        };
        if exchange.fetching() {
            let (request, phase) = self.http_request(&mut exchange, now)?;
            Ok((exchange, Request::Http(request), phase))
        } else {
            let url = Secret::new(url.expect("sign-in visits without a fetch"));
            Ok((exchange, Request::Visit { url }, Awaits::Redirect))
        }
    }

    pub(super) fn keys_request(
        &mut self,
        exchange: &mut Exchange,
        deadline: Time,
    ) -> Result<(HttpRequest, Awaits), Failure> {
        let fetch = match &exchange.grant {
            Grant::SignIn { fetch, .. } => fetch,
            Grant::Refresh { .. } | Grant::Revoke { .. } => return Err(failure()),
        };
        let endpoint = match fetch.as_ref().ok_or(failure())? {
            Fetch::Metadata { endpoint } | Fetch::Keys { endpoint } => endpoint.clone(),
        };
        let id = self.next_id;
        self.next_id = self.next_id.checked_add(1).ok_or(failure())?;
        exchange.attempts = exchange.attempts.checked_add(1).ok_or(failure())?;
        Ok((
            HttpRequest { id, method: Method::Get, endpoint, content_type: b"", body: Box::default(), deadline },
            Awaits::Keys { id, deadline },
        ))
    }

    pub(super) fn keys_answer(
        &mut self,
        mut exchange: Exchange,
        answer: HttpResponse,
        deadline: Time,
        out: &mut Queue<Request>,
    ) -> Phase {
        if answer.now >= exchange.deadline {
            return Self::finish(failure(), out);
        }
        if answer.now >= deadline
            || answer.evidence != HttpEvidence::Response
            || answer.status == 429
            || answer.status >= 500
        {
            return self.retry_or_fail(exchange, Failure::Unavailable, answer.now, answer.retry_after, true, out);
        }
        if !(200..300).contains(&answer.status) {
            return Self::finish(failure(), out);
        }
        let result = self.read_keys(&mut exchange, &answer.body, answer.now);
        match result {
            Ok((request, phase)) => {
                out.push(request);
                phase.hold(exchange)
            }
            Err(_) => Self::finish(failure(), out),
        }
    }

    fn read_keys(&mut self, exchange: &mut Exchange, body: &[u8], now: Time) -> Result<(Request, Awaits), Check> {
        let (fetch, visit, keys) = match &mut exchange.grant {
            Grant::SignIn { fetch, visit, keys, .. } => (fetch, visit, keys),
            Grant::Refresh { .. } | Grant::Revoke { .. } => return Err(Check::Keys),
        };
        match fetch.take().ok_or(Check::Keys)? {
            Fetch::Metadata { .. } => {
                let terms = exchange.registration.terms.as_ref().ok_or(Check::Keys)?;
                let issuer = &terms.openid.as_ref().ok_or(Check::Keys)?.issuer;
                let endpoint = keys::issuer_key_set(body, issuer, &self.limits)?;
                *fetch = Some(Fetch::Keys { endpoint });
                exchange.attempts = 0;
                let (request, phase) = self.http_request(exchange, now).or(Err(Check::Keys))?;
                Ok((Request::Http(request), phase))
            }
            Fetch::Keys { .. } => {
                *keys = keys::fitting_keys(body, &self.limits)?;
                exchange.attempts = 0;
                let url = visit.take().ok_or(Check::Keys)?;
                Ok((Request::Visit { url: Secret::new(url) }, Awaits::Redirect))
            }
        }
    }
}

pub(super) struct Identity {
    pub(super) subject: Option<Box<[u8]>>,
    pub(super) token: Option<Secret>,
    pub(super) profile: crate::ProfileClaims,
}
impl Client {
    pub(super) fn identity(
        &self,
        registration: &Registration,
        nonce: Option<&Secret>,
        keys: &[keys::Key],
        response: &crate::TokenResponse,
        wall: skein_lib::Wall,
    ) -> Result<Identity, Check> {
        if !has_openid(registration) {
            return Ok(Identity { subject: None, token: None, profile: crate::ProfileClaims::default() });
        }
        let token = response.id_token.as_ref().ok_or(Check::Missing)?.expose();
        let terms = registration.terms.as_ref().ok_or(Check::Issuer)?;
        let openid = terms.openid.as_ref().ok_or(Check::Issuer)?;
        let expected = crate::jwt::Expected {
            issuer: &openid.issuer,
            client_id: &registration.client_id,
            nonce: secret_ref(nonce).ok_or(Check::Nonce)?,
        };
        let verified = crate::jwt::verify_id_token(
            token,
            keys,
            &expected,
            openid.profile,
            wall,
            self.limits.skew,
            &self.limits.document,
        )?;
        Ok(Identity {
            profile: verified.profile,
            subject: Some(verified.subject),
            token: Some(Secret::new(skein_lib::bytes::copy_of(token))),
        })
    }
}
