//! Read exchanges remain pending until the core's held answer is released (protocol/client/README.md, 4).
use super::{Box, Component, Env, Limits, Queue, ReplyTo, Request, Token, Waiting, core, routes, w};
use jig_client_protocol_wire::Record as _;
use skein_lib::{Reader, Wall};
impl Component {
    pub(super) fn serves(&self, family: Option<u16>, version: u32) -> bool {
        crate::served::serves(&self.settings, family, version)
    }
    pub(super) fn read(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        expires: Wall,
        body: &[u8],
        out: &mut Queue<Request>,
    ) {
        if self.owner(exchange).is_none() {
            return;
        }
        if !self.exchange_room(env, exchange, sign_in)
            || self.member_count(sign_in, false) >= env.limits.reads_per_client
        {
            self.respond(exchange, routes::read_none(core::reads::Why::Busy, &env.limits.wire));
            return;
        }
        let mut reader = Reader::new(body);
        let record = match w::Owned::decode(&env.limits.wire, &mut reader) {
            Ok(value) if reader.is_empty() => value,
            Ok(_) | Err(_) => {
                self.response(exchange, crate::Reply::NotARecord);
                return;
            }
        };
        let family = match &record.owner {
            w::Owner::Jig => None,
            w::Owner::Family(value) => Some(value.number),
        };
        if !self.serves(family, record.version) {
            let answer =
                w::ReadAnswer::NotServed(crate::served::not_served(&self.settings, record.owner, &env.limits.wire));
            self.respond(exchange, routes::encoded_read(&answer));
            return;
        }
        let request = match record.owner {
            w::Owner::Jig => {
                let Some(what) = crate::translate::reads::decode(&record.record, &env.limits.wire) else {
                    self.respond(exchange, routes::read_none(core::reads::Why::Unknown, &env.limits.wire));
                    return;
                };
                let Some(core_read) = crate::translate::parts::read(&what, &self.settings, &env.limits.wire) else {
                    self.respond(exchange, routes::read_none(core::reads::Why::Unknown, &env.limits.wire));
                    return;
                };
                let request = Request::Core(Box::new(core::Event::Read {
                    reply: ReplyTo::new(exchange),
                    sign_in,
                    read: core_read,
                }));
                self.pending
                    .exchanges
                    .insert(exchange, Waiting::Read { sign_in, expires, what: Box::new(what) })
                    .expect("same read slot");
                request
            }
            w::Owner::Family(value) => {
                let Ok(body) = w::FamilyRecord::decode(&env.limits.wire, &mut Reader::new(&record.record)) else {
                    self.response(exchange, crate::Reply::NotARecord);
                    return;
                };
                let Some(window) = crate::served::window(&body.window) else {
                    self.response(exchange, crate::Reply::NotARecord);
                    return;
                };
                let scope = crate::served::scope(&body.scope);
                let family = value.number;
                self.pending
                    .exchanges
                    .insert(exchange, Waiting::ApplicationRead { sign_in, family, version: record.version })
                    .expect("same read slot");
                Request::Application(crate::ToApplication::Read {
                    reply: ReplyTo::new(exchange),
                    sign_in,
                    family,
                    version: record.version,
                    scope,
                    window,
                    record: body.record,
                })
            }
        };
        out.push(request);
    }
    pub(super) fn read_answer(&mut self, env: &Env<Limits>, exchange: Token, answer: &core::reads::Answer) {
        let Some(waiting) = self.pending.exchanges.remove(&exchange) else { return };
        match waiting {
            Waiting::Read { what, .. } => {
                self.read_response(
                    exchange,
                    crate::translate::reads::answer(
                        &what,
                        answer,
                        &self.settings.models,
                        &self.settings.private,
                        &self.settings,
                        &env.limits.wire,
                    ),
                    &env.limits.wire,
                );
            }
            waiting @ Waiting::ApplicationRead { .. } => match answer {
                core::reads::Answer::None(why) => {
                    self.respond(exchange, routes::read_none(*why, &env.limits.wire));
                }
                core::reads::Answer::Setting(_)
                | core::reads::Answer::Revision(_)
                | core::reads::Answer::Revisions(_)
                | core::reads::Answer::TaskText(_)
                | core::reads::Answer::Tasks(_)
                | core::reads::Answer::Document(_)
                | core::reads::Answer::Ended(_)
                | core::reads::Answer::Page(_) => {
                    self.pending.exchanges.insert(exchange, waiting).expect("same unrelated slot");
                }
            },
            Waiting::Authorize { .. }
            | Waiting::LinkAskWhose { .. }
            | Waiting::AskedWhose { .. }
            | Waiting::ApprovalWhose { .. }
            | Waiting::ApprovalCore { .. }
            | Waiting::Core { .. }
            | Waiting::LinkGet { .. }
            | Waiting::SignOut { .. }
            | Waiting::Watch { .. }
            | Waiting::Keep { .. }
            | Waiting::Get { .. }
            | Waiting::Moving { .. }
            | Waiting::Made { .. }
            | Waiting::Ask { .. }
            | Waiting::Carry { .. }
            | Waiting::Checked { .. } => {
                self.pending.exchanges.insert(exchange, waiting).expect("same unrelated slot");
            }
        }
    }
    pub(super) fn application_read(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        family: u16,
        record: Option<Box<[u8]>>,
    ) {
        let Some(waiting) = self.pending.exchanges.remove(&exchange) else { return };
        match waiting {
            Waiting::ApplicationRead { family: number, version, .. } if number == family => {
                let answer = match record {
                    Some(record) => match Ok(w::Family { number }) {
                        Ok(family) => {
                            crate::translate::reads::owned(w::Owner::Family(family), version, record, &env.limits.wire)
                        }
                        Err(problem) => Err(problem),
                    },
                    None => Ok(crate::translate::reads::none(core::reads::Why::Unknown, &env.limits.wire)),
                };
                self.read_response(exchange, answer, &env.limits.wire);
            }
            Waiting::Authorize { .. }
            | Waiting::LinkAskWhose { .. }
            | Waiting::AskedWhose { .. }
            | Waiting::ApprovalWhose { .. }
            | Waiting::ApprovalCore { .. }
            | Waiting::Core { .. }
            | Waiting::LinkGet { .. }
            | Waiting::SignOut { .. }
            | Waiting::Watch { .. }
            | Waiting::Keep { .. }
            | Waiting::Get { .. }
            | Waiting::Moving { .. }
            | Waiting::Made { .. }
            | Waiting::Ask { .. }
            | Waiting::Carry { .. }
            | Waiting::Read { .. }
            | Waiting::ApplicationRead { .. }
            | Waiting::Checked { .. } => {
                self.pending.exchanges.insert(exchange, waiting).expect("same unrelated slot");
            }
        }
    }
    fn read_response(&mut self, exchange: Token, answer: Result<w::ReadAnswer, w::Problem>, limits: &w::Limits) {
        let bytes = match answer {
            Ok(value) => value.encode(limits),
            Err(problem) => Err(problem),
        };
        match bytes {
            Ok(bytes) => self.respond(exchange, bytes),
            Err(_) => self.respond(exchange, routes::read_none(core::reads::Why::Malformed, limits)),
        }
    }
}
