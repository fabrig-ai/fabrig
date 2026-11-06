//! Refresh-token revocation and its confirmed or unconfirmed terminal.

use super::{
    Awaits, Client, ClientLimits, Exchange, Failure, Grant, HttpEvidence, HttpResponse, Phase, Registration, Request,
};
use crate::RefreshState;
use crate::redirect::{form_fields, validate_registration};
use alloc::boxed::Box;
use skein_lib::{Queue, Time};

pub(super) fn endpoint(registration: &Registration) -> Option<&[u8]> {
    registration.terms.as_ref()?.revocation_endpoint.as_deref()
}

impl Exchange {
    pub(super) fn revoking(&self) -> bool {
        match self.grant {
            Grant::Revoke { .. } => true,
            Grant::SignIn { .. } | Grant::Refresh { .. } => false,
        }
    }
}

impl Client {
    pub(super) fn revoke(
        &mut self,
        registration: Registration,
        prior: RefreshState,
        now: Time,
        out: &mut Queue<Request>,
    ) -> Phase {
        let result = self.start_revocation(registration, prior, now);
        match result {
            Ok(start) => Self::emit_start(Ok(start), out),
            Err(_) => Self::revoked(false, out),
        }
    }

    fn start_revocation(
        &mut self,
        mut registration: Registration,
        prior: RefreshState,
        now: Time,
    ) -> Result<(Exchange, Request, Awaits), Failure> {
        validate_registration(&registration, &self.limits, false)?;
        endpoint(&registration).ok_or(Failure::Malformed)?;
        self.refresh_values(&mut registration, &prior)?;
        let deadline = now.checked_add(self.limits.sign_in_time).ok_or(Failure::Limit)?;
        let mut exchange = Exchange { registration, grant: Grant::Revoke { prior }, deadline, attempts: 0 };
        let (request, phase) = self.http_request(&mut exchange, now)?;
        Ok((exchange, Request::Http(request), phase))
    }

    pub(super) fn revocation_answer(
        &mut self,
        exchange: Exchange,
        answer: HttpResponse,
        deadline: Time,
        out: &mut Queue<Request>,
    ) -> Phase {
        if answer.now >= deadline || answer.now >= exchange.deadline {
            Self::revoked(false, out)
        } else if answer.evidence == HttpEvidence::Unsent {
            self.retry_or_fail(exchange, Failure::Unavailable, answer.now, answer.retry_after, true, out)
        } else if answer.status == 429 && answer.evidence == HttpEvidence::Response {
            self.retry_or_fail(
                exchange,
                Failure::RateLimited { retry_after: answer.retry_after },
                answer.now,
                answer.retry_after,
                true,
                out,
            )
        } else {
            Self::revoked(answer.status == 200 && answer.evidence == HttpEvidence::Response, out)
        }
    }

    pub(super) fn revoked(confirmed: bool, out: &mut Queue<Request>) -> Phase {
        out.push(Request::Revoked { confirmed });
        Phase::Done
    }
}

pub(super) fn body(
    registration: &Registration,
    prior: &RefreshState,
    limits: &ClientLimits,
) -> Result<Box<[u8]>, Failure> {
    let fields = [
        Some((b"token".as_slice(), prior.refresh_token.expose())),
        Some((b"token_type_hint".as_slice(), b"refresh_token".as_slice())),
        Some((b"client_id".as_slice(), registration.client_id.as_ref())),
        match registration.client_secret.as_ref() {
            Some(secret) => Some((b"client_secret".as_slice(), secret.expose())),
            None => None,
        },
    ];
    form_fields(&fields, limits.request_bytes)
}
