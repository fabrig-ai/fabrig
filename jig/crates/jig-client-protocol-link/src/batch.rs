//! Settled exchanges and each member's own reply (protocol/client/wire.md, 2.3).
use jig_client_protocol_wire::Record as _;
use crate::{BATCH_BYTES, Link, Message, Request, component::Gathered};
use jig_client_protocol_wire as w;
use skein_lib::{List, Map, Queue, Reader, Token, Writer};

impl Link {
    /// Drop a held read unsent; public only to compose the link with jig-client
    /// (protocol/client/README.md, 10). Sent reads keep their exchange to its terminal.
    pub fn cancel_read(&mut self, read: Token) {
        for _ in 0..self.gathered.len() {
            let pending = self.gathered.pop().expect("counted gathered transport");
            let cancel = match &pending {
                Gathered::Exchange { exchange, message: Message::Read(_), .. } => match self.attempts.get(exchange) {
                    Some(attempt) => attempt.token == read,
                    None => false,
                },
                Gathered::Exchange { .. } | Gathered::OpenStream { .. } | Gathered::CloseStream => false,
            };
            if cancel {
                match pending {
                    Gathered::Exchange { exchange, .. } => {
                        self.attempts.remove(&exchange);
                    }
                    Gathered::OpenStream { .. } | Gathered::CloseStream => unreachable!("cancelled read"),
                }
            } else {
                self.gathered.push(pending);
            }
        }
    }
    pub fn settle(&mut self, out: &mut Queue<Request>) {
        let mut waiting = Queue::uncapped();
        for _ in 0..self.gathered.len() {
            let Some(first) = self.gathered.iter().next() else { break };
            match first {
                Gathered::Exchange { exchange, message, .. } => {
                    if tracked(message) && !self.attempts.contains_key(exchange) {
                        self.gathered.pop();
                        continue;
                    }
                    if self.transport.len() >= self.limits.exchanges() {
                        let value = self.gathered.pop().expect("waiting exchange");
                        waiting.push(value);
                        continue;
                    }
                }
                Gathered::OpenStream { .. } | Gathered::CloseStream => {}
            }
            self.settle_one(out);
        }
        assert!(self.gathered.is_empty(), "settling considered every gathered message");
        self.gathered = waiting;
    }

    fn settle_one(&mut self, out: &mut Queue<Request>) {
        let first = self.gathered.pop().expect("counted gathered transport");
        let (exchange, credential, message) = match first {
            Gathered::OpenStream { credential } => {
                out.push(Request::OpenStream {
                    credential,
                    version: jig_client_protocol_wire::VERSION,
                    replaces: self.predecessor,
                });
                return;
            }
            Gathered::CloseStream => {
                out.push(Request::CloseStream);
                return;
            }
            Gathered::Exchange { exchange, credential, message } => (exchange, credential, message),
        };
        let mut members = List::with_capacity(64);
        let mut attempts = Map::with_capacity(64);
        if let Some(first_member) = member(&message, self.limits.wire()) {
            let mut bytes = 4_u32.saturating_add(first_member.measure(self.limits.wire()).expect("bounded client record"));
            members.push(first_member).expect("first member");
            attempts.insert(0, exchange).expect("first member");
            for _ in 0..self.gathered.len() {
                if bytes > BATCH_BYTES || members.len() == 64 {
                    break;
                }
                let Some(next) = self.gathered.iter().next() else { break };
                let (next, held, body) = match next {
                    Gathered::Exchange { exchange, credential, message } => (exchange, credential, message),
                    Gathered::OpenStream { .. } | Gathered::CloseStream => break,
                };
                if held != &credential {
                    break;
                }
                if tracked(body) && !self.attempts.contains_key(next) {
                    self.gathered.pop();
                    continue;
                }
                let Some(part) = member(body, self.limits.wire()) else { break };
                let size = bytes.saturating_add(part.measure(self.limits.wire()).expect("bounded client record"));
                if size > BATCH_BYTES {
                    break;
                }
                let place = u16::try_from(members.len()).expect("at most 64 members");
                attempts.insert(place, *next).expect("at most 64 members");
                members.push(part).expect("at most 64 members");
                self.gathered.pop();
                bytes = size;
            }
        }
        let message = if members.len() > 1 {
            let batch = w::Batch { members };
            let mut writer = Writer::new(skein_lib::length::usize_of(batch.measure(self.limits.wire()).expect("bounded client record")));
            batch.write(self.limits.wire(), &mut writer).expect("measured batch");
            self.batches.insert(exchange, attempts).expect("internal batches are uncapped");
            Message::Batch(writer.finish())
        } else {
            message
        };
        self.transport.insert(exchange, ()).expect("internal exchange accounting is uncapped");
        out.push(Request::Exchange { exchange, credential, message });
    }

    pub(super) fn part(&mut self, exchange: Token, body: &[u8], out: &mut Queue<Request>) {
        let Some(members) = self.batches.get(&exchange) else { return };
        if members.is_empty() {
            return;
        }
        let Ok(part) = w::BatchReply::decode(self.limits.receiving(), &mut Reader::new(body)) else {
            self.batch_lost(exchange, out);
            return;
        };
        let Some(member) = self.batches.get_mut(&exchange).expect("known batch").remove(&part.member) else {
            self.batch_lost(exchange, out);
            return;
        };
        match part.reply {
            Some(reply) => self.answered(member, &reply, out),
            None => {
                if let Some(attempt) = self.attempts.remove(&member) {
                    self.failure(attempt, out);
                }
            }
        }
    }

    pub(super) fn batch_lost(&mut self, exchange: Token, out: &mut Queue<Request>) {
        let pending = core::mem::replace(self.batches.get_mut(&exchange).expect("known batch"), Map::with_capacity(64));
        // Detach every member before a watch's failure cancels the stream's other attempts.
        let mut lost = List::with_capacity(pending.len());
        for (_, token) in &pending {
            if let Some(attempt) = self.attempts.remove(token) {
                lost.push(attempt).expect("batch members");
            }
        }
        for attempt in lost.into_boxed() {
            self.failure(attempt, out);
        }
    }
}

fn tracked(message: &Message) -> bool {
    match message {
        Message::Read(_) | Message::Watch(_) | Message::Request(_) => true,
        Message::Unwatch(_)
        | Message::Batch(_)
        | Message::StreamGone(_)
        | Message::SignInLink(_)
        | Message::SignInService
        | Message::SignOut
        | Message::LinkAsk(_)
        | Message::Authorize(_)
        | Message::ApprovalAsk(_)
        | Message::ApprovalRead(_)
        | Message::ApprovalDecide(_)
        | Message::SignInApproval(_) => false,
    }
}

fn member(message: &Message, limits: &w::Limits) -> Option<w::Member> {
    match message {
        Message::Read(body) => Some(w::Member::Read(w::Owned::decode(limits, &mut Reader::new(body)).ok()?)),
        Message::Watch(body) => Some(w::Member::Open(w::Open::decode(limits, &mut Reader::new(body)).ok()?)),
        Message::Unwatch(body) => Some(w::Member::Close(w::WatchClose::decode(limits, &mut Reader::new(body)).ok()?)),
        Message::Batch(_)
        | Message::Request(_)
        | Message::StreamGone(_)
        | Message::SignInLink(_)
        | Message::SignInService
        | Message::SignOut
        | Message::LinkAsk(_)
        | Message::Authorize(_)
        | Message::ApprovalAsk(_)
        | Message::ApprovalRead(_)
        | Message::ApprovalDecide(_)
        | Message::SignInApproval(_) => None,
    }
}
