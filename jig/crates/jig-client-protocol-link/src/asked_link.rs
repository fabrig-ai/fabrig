//! Asked links are transient exchanges (jig/docs/design/client/domain/client/README.md, 9).
use crate::component::{Attempt, ExchangeKind};
use crate::{AskedLink, Link, Message, NotAsked, Request, Signing};
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_client_protocol_wire::VERSION;
use skein_lib::{Queue, Reader, Writer};

impl Link {
    pub(super) fn ask_link(&mut self, out: &mut Queue<Request>) {
        for (_, attempt) in &self.attempts {
            match attempt.kind {
                ExchangeKind::LinkAsk => {
                    out.push(Request::AskedLink(AskedLink::NotAsked(NotAsked::Busy)));
                    return;
                }
                ExchangeKind::Approval(_)
                | ExchangeKind::Authorize
                | ExchangeKind::SignIn
                | ExchangeKind::SignInApproval
                | ExchangeKind::SignOut { .. }
                | ExchangeKind::Attempt { .. }
                | ExchangeKind::Read { .. }
                | ExchangeKind::WatchOpen { .. }
                | ExchangeKind::WatchClose { .. } => {}
            }
        }
        if self.attempts.len() == self.attempts.capacity() {
            out.push(Request::AskedLink(AskedLink::NotAsked(NotAsked::Busy)));
            return;
        }
        let Some(exchange) = self.token() else {
            out.push(Request::AskedLink(AskedLink::NotAsked(NotAsked::Lost)));
            return;
        };
        self.attempts
            .insert(
                exchange,
                Attempt {
                    token: exchange,
                    owner: w::Owner::Jig,
                    version: VERSION,
                    kind: ExchangeKind::LinkAsk,
                    held: None,
                    parts: skein_lib::List::with_capacity(0),
                },
            )
            .expect("checked exchange capacity");
        let record = w::LinkAsk {};
        let mut bytes = Writer::new(skein_lib::length::usize_of(
            record.measure(self.limits.wire()).expect("bounded client record"),
        ));
        record.write(self.limits.wire(), &mut bytes).expect("measured link ask");
        self.emit_exchange(exchange, self.credential.clone(), Message::LinkAsk(bytes.finish()));
    }

    pub(super) fn asked_link(&mut self, body: &[u8], out: &mut Queue<Request>) {
        let mut reader = Reader::new(body);
        let answer = match w::LinkAskAnswer::decode(self.limits.wire(), &mut reader) {
            Ok(answer) if reader.is_empty() => match answer {
                w::LinkAskAnswer::Asked(value) => {
                    let fields = value;
                    if fields.secret.len() == 32 {
                        AskedLink::Asked { secret: fields.secret, expires: fields.expires }
                    } else {
                        AskedLink::NotAsked(NotAsked::Lost)
                    }
                }
                w::LinkAskAnswer::Refused => AskedLink::NotAsked(NotAsked::Refused),
                w::LinkAskAnswer::SignedOut => AskedLink::SignedOut,
                w::LinkAskAnswer::NotReady => AskedLink::NotAsked(NotAsked::NotReady),
                w::LinkAskAnswer::Busy => AskedLink::NotAsked(NotAsked::Busy),
            },
            Ok(_) | Err(_) => AskedLink::NotAsked(NotAsked::Lost),
        };
        match &answer {
            AskedLink::SignedOut => {
                self.end_sign_in(out);
                out.push(Request::Signing { signing: Signing::SignedOut });
            }
            AskedLink::Asked { .. } | AskedLink::NotAsked(_) => {}
        }
        out.push(Request::AskedLink(answer));
    }
}
