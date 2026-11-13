//! One authorization in flight; no retained request (protocol/client/openid.md).
use crate::{
    AuthorizeWhy, Authorized, Link, Message, Request,
    component::{Attempt, ExchangeKind},
};
use alloc::boxed::Box;
use jig_client_protocol_wire::{VERSION, v21 as w};
use skein_lib::{Queue, Reader, Writer};
impl Link {
    pub(super) fn authorize(&mut self, query: Box<[u8]>, out: &mut Queue<Request>) {
        for (_, attempt) in &self.attempts {
            match attempt.kind {
                ExchangeKind::Authorize => {
                    out.push(Request::Authorized(Authorized::Busy));
                    return;
                }
                ExchangeKind::Approval(_)
                | ExchangeKind::SignInApproval
                | ExchangeKind::SignIn
                | ExchangeKind::SignOut { .. }
                | ExchangeKind::Attempt { .. }
                | ExchangeKind::Read { .. }
                | ExchangeKind::WatchOpen { .. }
                | ExchangeKind::WatchClose { .. } => {}
            }
        }
        if self.in_flight() >= self.limits.exchanges() {
            out.push(Request::Authorized(Authorized::Busy));
            return;
        }
        let Ok(record) = w::Authorize::new(self.limits.wire(), w::AuthorizeParts { query }) else {
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
                    token: exchange,
                    owner: w::Owner::Jig,
                    version: VERSION,
                    kind: ExchangeKind::Authorize,
                    held: None,
                },
            )
            .expect("checked exchange capacity");
        let mut writer = Writer::new(skein_lib::length::usize_of(record.measure()));
        record.encode(&mut writer).expect("measured authorization");
        self.exchange(exchange, Message::Authorize(writer.finish()), out);
    }
    pub(super) fn authorized(&self, body: &[u8], out: &mut Queue<Request>) {
        let mut reader = Reader::new(body);
        let answer = match w::AuthorizeAnswer::decode(self.limits.wire(), &mut reader) {
            Ok(answer) if reader.is_empty() => match answer {
                w::AuthorizeAnswer::Go(go) => Authorized::Go { location: go.into_parts().location },
                w::AuthorizeAnswer::Refused(refused) => Authorized::Refused(match refused.why() {
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
