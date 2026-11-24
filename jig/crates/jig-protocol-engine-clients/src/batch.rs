//! One exchange, independently released members (protocol/client/wire.md, 2.2).
use super::{Box, Component, Env, Limits, Queue, Request, Token, Waiting, routes, w};
use crate::{Reply, ToTransport};
use jig_client_protocol_wire::Record as _;
use skein_lib::{List, Reader, Wall, Writer};
#[derive(Debug)]
pub(super) struct Batch {
    members: List<w::Member>,
    parts: Queue<Box<[u8]>>,
    remaining: u32,
    started: bool,
    gone: bool,
    sign_in: u64,
    pub(super) admitted: bool,
    reply_bytes: u32,
}
#[derive(Debug)]
pub(super) struct Member {
    pub(super) batch: Token,
    index: u16,
}
impl Component {
    pub(super) fn batch_decode(&mut self, env: &Env<Limits>, exchange: Token, body: &[u8]) -> bool {
        if body.len() > skein_lib::length::usize_of(env.limits.message_bytes) {
            return false;
        }
        let mut reader = Reader::new(body);
        let Ok(record) = w::Batch::decode(&env.limits.wire, &mut reader) else { return false };
        let members = record.members;
        let remaining = members.len();
        self.batches
            .insert(
                exchange,
                Batch {
                    members,
                    parts: Queue::with_capacity(64),
                    remaining,
                    started: false,
                    gone: false,
                    sign_in: 0,
                    admitted: false,
                    reply_bytes: env.limits.wire.batch_reply_reply_some,
                },
            )
            .expect("one batch per transport exchange");
        true
    }
    pub(super) fn batch_denied(&mut self, exchange: Token) {
        self.pending
            .exchanges
            .insert(
                exchange,
                Waiting::Checked { sign_in: 0, expires: Wall::EPOCH, body: Box::new([]), route: routes::Route::Batch },
            )
            .expect("same credential slot");
    }
    pub(super) fn batch_start(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        expires: Wall,
        out: &mut Queue<Request>,
    ) {
        if self.owner(exchange).is_none() {
            return;
        }
        let admitted = sign_in != 0
            && self.active(sign_in) < env.limits.exchanges_per_client
            && self.admitted() < env.limits.exchanges;
        let batch = self.batches.get_mut(&exchange).expect("decoded batch");
        batch.started = true;
        batch.sign_in = sign_in;
        batch.admitted = admitted;
        let members = core::mem::replace(&mut batch.members, List::with_capacity(0));
        for (index, member) in members.into_boxed().into_iter().enumerate() {
            let name = self.name();
            self.members
                .insert(name, Member { batch: exchange, index: u16::try_from(index).expect("64 members") })
                .expect("bounded member work");
            let (route, body) = match member {
                w::Member::Read(value) => {
                    let mut bytes = Writer::new(skein_lib::length::usize_of(
                        value.measure(&env.limits.wire).expect("decoded client record"),
                    ));
                    value.write(&env.limits.wire, &mut bytes).expect("measured read");
                    (routes::Route::Read, bytes.finish())
                }
                w::Member::Open(value) => {
                    let mut bytes = Writer::new(skein_lib::length::usize_of(
                        value.measure(&env.limits.wire).expect("decoded client record"),
                    ));
                    value.write(&env.limits.wire, &mut bytes).expect("measured opening");
                    (routes::Route::Watch, bytes.finish())
                }
                w::Member::Close(value) => {
                    let mut bytes = Writer::new(skein_lib::length::usize_of(
                        value.measure(&env.limits.wire).expect("decoded client record"),
                    ));
                    value.write(&env.limits.wire, &mut bytes).expect("measured closing");
                    (routes::Route::Unwatch, bytes.finish())
                }
            };
            if sign_in == 0 {
                self.respond(name, routes::unauthenticated(route));
            } else {
                match route {
                    routes::Route::Read => self.read(env, name, sign_in, expires, &body, out),
                    routes::Route::Watch | routes::Route::Unwatch => {
                        self.watch_route(env, name, sign_in, route, &body, out);
                    }
                    routes::Route::LinkAsk
                    | routes::Route::Batch
                    | routes::Route::ApprovalAsk
                    | routes::Route::ApprovalRead
                    | routes::Route::ApprovalDecide
                    | routes::Route::ApprovalTake
                    | routes::Route::Link
                    | routes::Route::Service
                    | routes::Route::SignOut
                    | routes::Route::Authorize
                    | routes::Route::Request
                    | routes::Route::Gone
                    | routes::Route::Stream => unreachable!("batch member route"),
                }
            }
        }
    }
    pub(super) fn response(&mut self, exchange: Token, reply: Reply) {
        if let Some(member) = self.members.remove(&exchange) {
            let Some(batch) = self.batches.get_mut(&member.batch) else { return };
            batch.remaining = batch.remaining.checked_sub(1).expect("member answered once");
            if batch.gone {
                return;
            }
            let reply = match reply {
                Reply::Record(bytes) => Some(bytes),
                Reply::NotARecord | Reply::Conflict => None,
                Reply::StreamsFull { .. } => unreachable!("a batch has no stream opening"),
            };
            let limits = w::Limits { batch_reply_reply_some: batch.reply_bytes, ..w::CEILINGS };
            let part = w::BatchReply { member: member.index, reply };
            let mut bytes =
                Writer::new(skein_lib::length::usize_of(part.measure(&limits).expect("decoded client record")));
            part.write(&limits, &mut bytes).expect("measured part");
            batch.parts.push(bytes.finish());
        } else if let Some(owner) = self.owner(exchange) {
            self.connections.get_mut(&owner).expect("exchange owner").respond(reply);
        }
    }
    pub(super) fn batch_gone(&mut self, exchange: Token) {
        if let Some(batch) = self.batches.get_mut(&exchange) {
            if batch.started {
                batch.gone = true;
                batch.parts.clear();
            } else {
                self.batches.remove(&exchange);
            }
        }
    }
    pub(super) fn batch_up(&mut self, out: &mut Queue<Request>) -> bool {
        let mut ready = None;
        for (exchange, batch) in &self.batches {
            if batch.started && (!batch.parts.is_empty() || batch.remaining == 0) {
                ready = Some(*exchange);
                break;
            }
        }
        let Some(exchange) = ready else { return false };
        let owner = self.owner(exchange);
        let batch = self.batches.get_mut(&exchange).expect("ready batch");
        if let Some(message) = batch.parts.pop() {
            if let Some(owner) = owner {
                out.push(Request::Transport(ToTransport::Part { exchange: owner, message }));
            }
        } else {
            self.batches.remove(&exchange);
            if let Some(owner) = owner {
                self.connections.remove(&owner);
                out.push(Request::Transport(ToTransport::Done { exchange: owner }));
            }
        }
        true
    }
    pub(super) fn exchange_room(&self, env: &Env<Limits>, exchange: Token, sign_in: u64) -> bool {
        if let Some(member) = self.members.get(&exchange) {
            return match self.batches.get(&member.batch) {
                Some(batch) => batch.admitted,
                None => false,
            };
        }
        self.active(sign_in) < env.limits.exchanges_per_client && self.admitted() < env.limits.exchanges
    }
    pub(super) fn member_count(&self, sign_in: u64, openings: bool) -> u32 {
        let mut count = 0_u32;
        for (_, waiting) in &self.pending.exchanges {
            let number = match waiting {
                Waiting::Read { sign_in, .. } | Waiting::ApplicationRead { sign_in, .. } if !openings => Some(*sign_in),
                Waiting::Watch { sign_in, opening: true, .. } if openings => Some(*sign_in),
                Waiting::Read { .. }
                | Waiting::ApplicationRead { .. }
                | Waiting::Watch { .. }
                | Waiting::LinkAskWhose { .. }
                | Waiting::AskedWhose { .. }
                | Waiting::ApprovalWhose { .. }
                | Waiting::ApprovalCore { .. }
                | Waiting::Core { .. }
                | Waiting::LinkGet { .. }
                | Waiting::SignOut { .. }
                | Waiting::Keep { .. }
                | Waiting::Get { .. }
                | Waiting::Moving { .. }
                | Waiting::Made { .. }
                | Waiting::Reissue { .. }
                | Waiting::Checked { .. }
                | Waiting::Ask { .. }
                | Waiting::Carry { .. }
                | Waiting::Authorize { .. } => None,
            };
            if number == Some(sign_in) {
                count = count.saturating_add(1);
            }
        }
        count
    }
    pub(super) fn active_batches(&self, sign_in: u64) -> u32 {
        let mut count = 0_u32;
        for (_, batch) in &self.batches {
            if batch.admitted && batch.sign_in == sign_in {
                count = count.saturating_add(1);
            }
        }
        count
    }
}
