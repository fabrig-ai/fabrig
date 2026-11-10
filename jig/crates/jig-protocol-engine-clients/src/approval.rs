//! Volatile browser approvals and reserved takes (jig/docs/design/protocol/client/sign-in.md).
use super::{Component, Env, Limits, Queue, ReplyTo, Request, Token, Waiting, core, people, routes, sign_in, w};
use crate::{Reply, exchange::Exchange, routes::Route};
use alloc::boxed::Box;
use jig_client_protocol_wire::ApprovalCode;
use skein_lib::{CryptoRng, Duration, Map, Reader, Secret, Time, Wall, sha256};

pub(crate) const WAITING: u32 = 4;
pub(crate) const SPAN: Duration = Duration::from_secs(600);
pub(crate) const PAST: u32 = 8;
// Rejection draws are bounded; an RNG unable to supply a candidate fails stop.
const DRAW_ATTEMPTS: u32 = 128;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum End {
    Taken,
    Refused,
    Expired,
}
#[derive(Clone, Copy, Debug)]
enum Decision {
    Waiting,
    Approved { sign_in: u64 },
    Taking { sign_in: u64, exchange: Token },
}
#[derive(Debug)]
struct Approval {
    code: ApprovalCode,
    asked: Wall,
    until: Time,
    expires: Wall,
    decision: Decision,
}
#[derive(Debug)]
struct Past {
    digest: Secret,
    code: ApprovalCode,
    end: End,
}
#[derive(Debug)]
pub(super) struct State {
    live: Map<[u8; 32], Approval>,
    past: Queue<Past>,
}
impl State {
    pub(super) fn new() -> Self {
        Self { live: Map::with_capacity(WAITING), past: Queue::with_capacity(PAST) }
    }
    fn end(&mut self, digest: &[u8; 32], end: End) {
        let Some(approval) = self.live.remove(digest) else { return };
        if self.past.room() == 0 {
            self.past.pop();
        }
        self.past.push(Past { digest: Secret::new(Box::new(*digest)), code: approval.code, end });
    }
    pub(super) fn sweep(&mut self, now: Time) {
        for _ in 0..WAITING {
            let mut found = None;
            for (digest, approval) in &self.live {
                match approval.decision {
                    Decision::Waiting | Decision::Approved { .. } => {
                        if approval.until <= now {
                            found = Some(*digest);
                            break;
                        }
                    }
                    // A take admitted before expiry owns its reservation until the core answers, as a link does.
                    Decision::Taking { .. } => {}
                }
            }
            let Some(digest) = found else { break };
            self.end(&digest, End::Expired);
        }
    }
    pub(super) fn next_deadline(&self) -> Option<Time> {
        let mut next = None;
        for (_, approval) in &self.live {
            match approval.decision {
                Decision::Waiting | Decision::Approved { .. } => {
                    next = Some(match next {
                        Some(next) => approval.until.min(next),
                        None => approval.until,
                    });
                }
                Decision::Taking { .. } => {}
            }
        }
        next
    }
    fn code(&self, code: [u8; 8]) -> Option<[u8; 32]> {
        for (digest, approval) in &self.live {
            if approval.code.letters() == &code {
                return Some(*digest);
            }
        }
        None
    }
    fn past_code(&self, code: [u8; 8]) -> Option<End> {
        let mut found = None;
        for past in &self.past {
            if past.code.letters() == &code {
                found = Some(past.end);
            }
        }
        found
    }
    fn digest(&self, digest: &[u8; 32]) -> Option<[u8; 32]> {
        let mut found = None;
        for (key, _) in &self.live {
            if sign_in::constant_time_eq(key, digest) {
                found = Some(*key);
            }
        }
        found
    }
    fn past_digest(&self, digest: &[u8; 32]) -> Option<End> {
        let mut found = None;
        for past in &self.past {
            if sign_in::constant_time_eq(past.digest.expose(), digest) {
                found = Some(past.end);
            }
        }
        found
    }
    pub(super) fn refused(&mut self, exchange: Token) -> bool {
        let mut found = None;
        for (digest, approval) in &self.live {
            match approval.decision {
                Decision::Taking { exchange: taking, .. } if taking == exchange => {
                    found = Some(*digest);
                    break;
                }
                Decision::Taking { .. } | Decision::Approved { .. } | Decision::Waiting => {}
            }
        }
        match found {
            Some(digest) => {
                self.end(&digest, End::Refused);
                true
            }
            None => false,
        }
    }
    fn release(&mut self, digest: &[u8; 32]) {
        if let Some(approval) = self.live.get_mut(digest) {
            match approval.decision {
                Decision::Taking { sign_in, .. } => approval.decision = Decision::Approved { sign_in },
                Decision::Waiting | Decision::Approved { .. } => unreachable!("take owns its reservation"),
            }
        }
    }
    fn draw_code(&self, rng: &mut CryptoRng) -> ApprovalCode {
        for _ in 0..DRAW_ATTEMPTS {
            let mut indexes = [0; 8];
            for index in &mut indexes {
                let mut accepted = None;
                for _ in 0..DRAW_ATTEMPTS {
                    let mut byte = [0];
                    rng.fill(&mut byte);
                    let byte = *byte.first().expect("one random byte");
                    if byte < 240 {
                        accepted = Some(byte.checked_rem(20).expect("nonzero alphabet"));
                        break;
                    }
                }
                *index = accepted.expect("random stream supplies an alphabet index");
            }
            let code = ApprovalCode::from_indexes(indexes).expect("indexes below twenty");
            if self.code(*code.letters()).is_none() {
                return code;
            }
        }
        unreachable!("random stream supplies a code not held by four live approvals")
    }
}
impl Component {
    pub(super) fn approval_ask(&mut self, env: &Env<Limits>, connection: &mut Exchange, body: &[u8]) {
        let mut reader = Reader::new(body);
        if w::ApprovalAsk::decode(&env.limits.wire, &mut reader).is_err() || !reader.is_empty() {
            connection.respond(Reply::NotARecord);
            return;
        }
        self.approval.sweep(env.now);
        let answer = if self.approval.live.len() == WAITING {
            w::ApprovalAskAnswer::Busy
        } else {
            let secret = sign_in::draw(&mut self.rng);
            let digest = sha256::digest(secret.expose());
            let code = self.approval.draw_code(&mut self.rng);
            let expires = Wall::from_nanos(env.wall.as_nanos().saturating_add(SPAN.as_nanos()));
            self.approval
                .live
                .insert(
                    digest,
                    Approval {
                        code,
                        asked: env.wall,
                        expires,
                        until: env.now.saturating_add(SPAN),
                        decision: Decision::Waiting,
                    },
                )
                .expect("room for approval");
            w::ApprovalAskAnswer::Asked(
                w::ApprovalAsked::new(
                    &env.limits.wire,
                    w::ApprovalAskedParts { secret, code: *code.letters(), expires: expires.as_nanos() },
                )
                .expect("fixed approval fits"),
            )
        };
        connection.respond(Reply::Record(routes::approval_ask_answer(answer)));
    }
    pub(super) fn approval_checked(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        route: Route,
        body: &[u8],
    ) {
        if self.stopping() {
            self.respond(exchange, routes::stopped(route).expect("approval stopping answer"));
            return;
        }
        if self.admitted() >= env.limits.exchanges {
            let answer = if route == Route::ApprovalRead {
                routes::approval_state(w::ApprovalState::Busy)
            } else {
                routes::approval_decided(w::ApprovalDecided::Busy)
            };
            self.respond(exchange, answer);
            return;
        }
        self.approval.sweep(env.now);
        let mut reader = Reader::new(body);
        let answer = if route == Route::ApprovalRead {
            match w::ApprovalRead::decode(&env.limits.wire, &mut reader) {
                Ok(record) if reader.is_empty() => {
                    Some(routes::approval_state(self.approval_read(env, *record.code())))
                }
                Ok(_) | Err(_) => None,
            }
        } else {
            match w::ApprovalDecide::decode(&env.limits.wire, &mut reader) {
                Ok(record) if reader.is_empty() => {
                    Some(routes::approval_decided(self.approval_decide(*record.code(), record.approve(), sign_in)))
                }
                Ok(_) | Err(_) => None,
            }
        };
        match answer {
            Some(answer) => self.respond(exchange, answer),
            None => {
                if let Some(owner) = self.owner(exchange) {
                    self.connections.get_mut(&owner).expect("approval exchange").respond(Reply::NotARecord);
                }
            }
        }
    }
    fn approval_read(&self, env: &Env<Limits>, code: [u8; 8]) -> w::ApprovalState {
        if let Some(digest) = self.approval.code(code) {
            let approval = self.approval.live.get(&digest).expect("live approval");
            return match approval.decision {
                Decision::Waiting => w::ApprovalState::Waiting(
                    w::ApprovalWaiting::new(
                        &env.limits.wire,
                        w::ApprovalWaitingParts { asked: approval.asked.as_nanos() },
                    )
                    .expect("fixed waiting fits"),
                ),
                Decision::Approved { .. } | Decision::Taking { .. } => w::ApprovalState::Approved,
            };
        }
        match self.approval.past_code(code) {
            Some(End::Taken) => w::ApprovalState::Taken,
            Some(End::Refused) => w::ApprovalState::Refused,
            Some(End::Expired) => w::ApprovalState::Expired,
            None => w::ApprovalState::Unknown,
        }
    }
    fn approval_decide(&mut self, code: [u8; 8], approve: bool, sign_in: u64) -> w::ApprovalDecided {
        if let Some(digest) = self.approval.code(code) {
            let approval = self.approval.live.get_mut(&digest).expect("live approval");
            return match approval.decision {
                Decision::Waiting => {
                    if approve {
                        approval.decision = Decision::Approved { sign_in };
                        w::ApprovalDecided::Approved
                    } else {
                        self.approval.end(&digest, End::Refused);
                        w::ApprovalDecided::Refused
                    }
                }
                Decision::Approved { .. } | Decision::Taking { .. } => w::ApprovalDecided::Used,
            };
        }
        match self.approval.past_code(code) {
            Some(End::Taken | End::Refused) => w::ApprovalDecided::Used,
            Some(End::Expired) => w::ApprovalDecided::Expired,
            None => w::ApprovalDecided::Unknown,
        }
    }
    pub(super) fn approval_take(
        &mut self,
        env: &Env<Limits>,
        connection: &mut Exchange,
        body: &[u8],
        out: &mut Queue<Request>,
    ) {
        let mut reader = Reader::new(body);
        let Ok(record) = w::ApprovalSignIn::decode(&env.limits.wire, &mut reader) else {
            connection.respond(Reply::NotARecord);
            return;
        };
        if !reader.is_empty() {
            connection.respond(Reply::NotARecord);
            return;
        }
        self.approval.sweep(env.now);
        let digest = sha256::digest(record.secret().expose());
        let answer = match self.approval.digest(&digest) {
            Some(digest) => {
                let approval = self.approval.live.get_mut(&digest).expect("live approval");
                match approval.decision {
                    Decision::Waiting => Some(w::ApprovalSignInAnswer::Waiting),
                    Decision::Taking { .. } => Some(w::ApprovalSignInAnswer::Busy),
                    Decision::Approved { sign_in } => {
                        if self.admitted() >= env.limits.exchanges {
                            Some(w::ApprovalSignInAnswer::Busy)
                        } else {
                            self.approval.live.get_mut(&digest).expect("live approval").decision =
                                Decision::Taking { sign_in, exchange: connection.name };
                            self.pending
                                .exchanges
                                .insert(connection.name, Waiting::ApprovalWhose { digest })
                                .expect("admitted take");
                            out.push(Request::Core(Box::new(core::Event::People(people::Event::Whose {
                                reply_to: ReplyTo::new(connection.name),
                                sign_in,
                            }))));
                            None
                        }
                    }
                }
            }
            None => Some(match self.approval.past_digest(&digest) {
                Some(End::Taken) => w::ApprovalSignInAnswer::Used,
                Some(End::Refused) => w::ApprovalSignInAnswer::Refused,
                Some(End::Expired) => w::ApprovalSignInAnswer::Expired,
                None => w::ApprovalSignInAnswer::Unknown,
            }),
        };
        if let Some(answer) = answer {
            connection.respond(Reply::Record(routes::approval_sign_in(answer)));
        }
    }
    pub(super) fn approval_whose(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        digest: [u8; 32],
        reply: people::Reply,
        out: &mut Queue<Request>,
    ) {
        if self.stopping() {
            if self.phase == super::closing::Phase::Stopped {
                self.approval.release(&digest);
            }
            self.respond(exchange, routes::approval_sign_in(w::ApprovalSignInAnswer::NotReady));
            return;
        }
        match reply {
            people::Reply::Signer(Some(signer)) => {
                self.pending.exchanges.insert(exchange, Waiting::ApprovalCore { digest }).expect("reserved take slot");
                out.push(Request::Core(Box::new(core::Event::SignIn {
                    reply_to: ReplyTo::new(exchange),
                    identity: signer.identity,
                    held: None,
                })));
            }
            people::Reply::Again(_) | people::Reply::Full { .. } => {
                self.approval_reply(env, exchange, digest, reply, out);
            }
            people::Reply::Signer(None)
            | people::Reply::SignedIn { .. }
            | people::Reply::SignedOut
            | people::Reply::Outcome(_)
            | people::Reply::Refused(_) => {
                self.approval.end(&digest, End::Refused);
                self.respond(exchange, routes::approval_sign_in(w::ApprovalSignInAnswer::Refused));
            }
        }
    }
    pub(super) fn approval_reply(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        digest: [u8; 32],
        reply: people::Reply,
        out: &mut Queue<Request>,
    ) {
        if !self.approval.live.contains_key(&digest) && self.closing() {
            match reply {
                people::Reply::SignedIn { .. } => self.sign_in_reply(env, exchange, reply, None, true, out),
                people::Reply::Signer(_)
                | people::Reply::SignedOut
                | people::Reply::Outcome(_)
                | people::Reply::Again(_)
                | people::Reply::Full { .. }
                | people::Reply::Refused(_) => {
                    self.respond(exchange, routes::approval_sign_in(w::ApprovalSignInAnswer::NotReady));
                }
            }
            return;
        }
        let answer = match reply {
            people::Reply::SignedIn { .. } => {
                self.approval.end(&digest, End::Taken);
                self.sign_in_reply(env, exchange, reply, None, true, out);
                return;
            }
            people::Reply::Again(people::Again::NotReady) => w::ApprovalSignInAnswer::NotReady,
            people::Reply::Again(people::Again::Busy | people::Again::Full { .. }) => w::ApprovalSignInAnswer::Busy,
            people::Reply::Full { sign_ins, earliest } => {
                let expires = self.approval.live.get(&digest).expect("reserved approval").expires;
                w::ApprovalSignInAnswer::Full(
                    w::ApprovalFull::new(
                        &env.limits.wire,
                        w::ApprovalFullParts { sign_ins, earliest: earliest.as_nanos(), until: expires.as_nanos() },
                    )
                    .expect("fixed full fits"),
                )
            }
            people::Reply::Signer(_)
            | people::Reply::SignedOut
            | people::Reply::Outcome(_)
            | people::Reply::Refused(_) => {
                self.approval.end(&digest, End::Refused);
                w::ApprovalSignInAnswer::Refused
            }
        };
        self.approval.release(&digest);
        self.respond(exchange, routes::approval_sign_in(answer));
    }
}
