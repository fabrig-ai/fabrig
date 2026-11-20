//! One authorization in flight; no retained request (protocol/client/openid.md).
use crate::{
    AuthorizeWhy, Authorized, Link, Message, Request,
    component::{Attempt, ExchangeKind},
};
use alloc::boxed::Box;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_client_protocol_wire::VERSION;
use skein_lib::{Queue, Reader};
impl Link {
    pub(super) fn authorize(&mut self, query: Box<[u8]>, out: &mut Queue<Request>) {
        for (_, attempt) in &self.attempts {
            match attempt.kind {
                ExchangeKind::Authorize => {
                    out.push(Request::Authorized(Authorized::Busy));
                    return;
                }
                ExchangeKind::LinkAsk
                | ExchangeKind::Approval(_)
                | ExchangeKind::SignInApproval
                | ExchangeKind::SignIn
                | ExchangeKind::SignOut { .. }
                | ExchangeKind::Attempt { .. }
                | ExchangeKind::Read { .. }
                | ExchangeKind::WatchOpen { .. }
                | ExchangeKind::WatchClose { .. } => {}
            }
        }
        let Ok(record) = (w::Authorize { query }).encode(self.limits.wire()) else {
            out.push(Request::Authorized(Authorized::Lost));
            return;
        };
        let Some(exchange) = self.token() else {
            out.push(Request::Authorized(Authorized::Lost));
            return;
        };
        self.attempts
            .insert(
                exchange,
                Attempt {
                    parts: skein_lib::List::with_capacity(0),
                    token: exchange,
                    owner: w::Owner::Jig,
                    version: VERSION,
                    kind: ExchangeKind::Authorize,
                    held: None,
                },
            )
            .expect("checked exchange capacity");
        self.exchange(exchange, Message::Authorize(record), out);
    }
    pub(super) fn authorized(&self, body: &[u8], out: &mut Queue<Request>) {
        let mut reader = Reader::new(body);
        let answer = match w::AuthorizeAnswer::decode(self.limits.wire(), &mut reader) {
            Ok(answer) if reader.is_empty() => match answer {
                w::AuthorizeAnswer::Go(go) => Authorized::Go { location: go.location },
                w::AuthorizeAnswer::Refused(refused) => Authorized::Refused(match &refused.why {
                    w::AuthorizeWhy::Malformed => AuthorizeWhy::Malformed,
                    w::AuthorizeWhy::UnknownClient => AuthorizeWhy::UnknownClient,
                    w::AuthorizeWhy::UnknownRedirect => AuthorizeWhy::UnknownRedirect,
                    w::AuthorizeWhy::NoProvider => AuthorizeWhy::NoProvider,
                    w::AuthorizeWhy::NotAPerson => AuthorizeWhy::NotAPerson,
                }),
                w::AuthorizeAnswer::SignedOut => Authorized::SignedOut,
                w::AuthorizeAnswer::NotReady => Authorized::NotReady,
                w::AuthorizeAnswer::Busy => Authorized::Busy,
            },
            Ok(_) | Err(_) => Authorized::Lost,
        };
        out.push(Request::Authorized(answer));
    }
}
