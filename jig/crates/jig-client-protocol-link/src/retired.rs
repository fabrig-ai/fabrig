//! A continued watch's old name keeps its room through admission and closing
//! (protocol/client/README.md, 5 and 10; client/domain/client/watches.md, 1).
use crate::{
    Fact, Link, Request, Signing,
    component::{Attempt, ExchangeKind},
};
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::VERSION;
use skein_lib::{Queue, Reader, Token};
impl Link {
    pub(super) fn retired_close(&mut self, watch: Token, name: u64, out: &mut Queue<Request>) {
        let Some(stream) = &self.stream else { return };
        let Some(stream) = stream.name else { return };
        let Some(exchange) = self.token() else { return self.drop_stream(out) };
        let attempt = Attempt {
            parts: skein_lib::List::with_capacity(0),
            token: watch,
            owner: w::Owner::Jig,
            version: VERSION,
            held: None,
            kind: ExchangeKind::WatchClose { name },
        };
        self.attempts.insert(exchange, attempt).expect("retired closing is bounded by watch room");
        self.unwatch(exchange, stream, name, out);
    }
    pub(super) fn retired_answer(&mut self, attempt: Attempt, name: u64, body: &[u8], out: &mut Queue<Request>) {
        if self.retired.get(&name) != Some(&attempt.token) {
            return;
        }
        match attempt.kind {
            ExchangeKind::WatchOpen { .. } => match w::OpenAnswer::decode(self.limits.wire(), &mut Reader::new(body)) {
                Ok(w::OpenAnswer::Admitted) => self.retired_close(attempt.token, name, out),
                Ok(w::OpenAnswer::NotServed(_)) => {
                    self.retired.remove(&name);
                }
                Ok(w::OpenAnswer::Refused(why)) => {
                    self.retired.remove(&name);
                    match why.why() {
                        w::WatchRefusal::SignedOut => {
                            self.end_sign_in(out);
                            out.push(Request::Signing { signing: Signing::SignedOut });
                        }
                        w::WatchRefusal::Unknown | w::WatchRefusal::NotAllowed | w::WatchRefusal::Busy => {}
                    }
                }
                Err(_) => self.retired_unreadable(out),
            },
            ExchangeKind::WatchClose { .. } => match w::CloseAnswer::decode(self.limits.wire(), &mut Reader::new(body))
            {
                Ok(w::CloseAnswer::Closed) => {
                    self.retired.remove(&name);
                }
                Ok(w::CloseAnswer::SignedOut) => {
                    self.retired.remove(&name);
                    self.end_sign_in(out);
                    out.push(Request::Signing { signing: Signing::SignedOut });
                }
                Err(_) => self.retired_unreadable(out),
            },
            ExchangeKind::LinkAsk
            | ExchangeKind::Authorize
            | ExchangeKind::Approval(_)
            | ExchangeKind::SignInApproval
            | ExchangeKind::SignIn
            | ExchangeKind::SignOut { .. }
            | ExchangeKind::Attempt { .. }
            | ExchangeKind::Read { .. } => unreachable!("retired watch answer"),
        }
    }
    fn retired_unreadable(&mut self, out: &mut Queue<Request>) {
        out.push(Request::Fact { fact: Fact::UndecodableWatchAnswer });
        self.drop_stream(out);
    }
}
