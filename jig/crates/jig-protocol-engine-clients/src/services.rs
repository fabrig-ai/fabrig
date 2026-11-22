//! Service credentials and sign-in digests use separate owners (protocol/client/sign-in.md).
use super::{
    Box, Component, Env, Kept, Limits, Queue, ReplyTo, Request, SecretOwner, SecretsAnswer, SecretsRequest, Token,
    Waiting, core, people, routes, sign_in,
};
use crate::sign_in::{Answer, CarriedRequest, Reissue};
use skein_lib::{Secret, Wall};
#[derive(Clone, Copy, Debug)]
pub(crate) enum GetOwner {
    SignIn(u64),
    Service(u64),
    LinkAskService(u64),
}
impl GetOwner {
    pub(crate) fn secret(self) -> SecretOwner {
        match self {
            Self::SignIn(number) => SecretOwner::SignIn(number),
            Self::Service(number) | Self::LinkAskService(number) => SecretOwner::Service(number),
        }
    }
}
impl Component {
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive route for secret-store answers and their waiting exchanges"
    )]
    pub(super) fn secrets(&mut self, env: &Env<Limits>, answer: SecretsAnswer, out: &mut Queue<Request>) {
        match answer {
            SecretsAnswer::Kept { owner } => self.kept(env, owner, out),
            SecretsAnswer::Got { owner, value } => {
                // Gets answer in the same iterate; waiting gets of one owner can use it.
                for _ in 0..env.limits.exchanges {
                    let mut found = None;
                    for (exchange, waiting) in &self.pending.exchanges {
                        let wanted = match waiting {
                            Waiting::Get { owner, .. } => Some(owner.secret()),
                            Waiting::LinkGet { sign_in, .. } => Some(SecretOwner::SignIn(*sign_in)),
                            Waiting::Made { party, .. } => Some(SecretOwner::Service(*party)),
                            Waiting::Moving { .. }
                            | Waiting::LinkAskWhose { .. }
                            | Waiting::AskedWhose { .. }
                            | Waiting::ApprovalWhose { .. }
                            | Waiting::ApprovalCore { .. }
                            | Waiting::Core { .. }
                            | Waiting::SignOut { .. }
                            | Waiting::Watch { .. }
                            | Waiting::Keep { .. }
                            | Waiting::Reissue { .. }
                            | Waiting::Ask { .. }
                            | Waiting::Carry { .. }
                            | Waiting::Read { .. }
                            | Waiting::Authorize { .. }
                            | Waiting::ApplicationRead { .. }
                            | Waiting::Checked { .. } => None,
                        };
                        if wanted == Some(owner) {
                            found = Some(*exchange);
                            break;
                        }
                    }
                    let Some(exchange) = found else { break };
                    match self.pending.exchanges.remove(&exchange).expect("found get") {
                        Waiting::LinkGet { sign_in, digest } => {
                            let held =
                                if sign_in::accepted(value.as_ref(), &digest, env.wall) { Some(sign_in) } else { None };
                            self.link_sign_in(exchange, held, out);
                        }
                        Waiting::Get { owner, digest, body, route } => {
                            if sign_in::accepted(value.as_ref(), &digest, env.wall) {
                                match owner {
                                    GetOwner::SignIn(sign_in) => {
                                        self.pending
                                            .exchanges
                                            .insert(
                                                exchange,
                                                Waiting::Checked {
                                                    sign_in,
                                                    expires: value.as_ref().expect("accepted value").expires,
                                                    body,
                                                    route,
                                                },
                                            )
                                            .expect("same get slot");
                                    }
                                    GetOwner::Service(party) => {
                                        if self.reissuing(party) {
                                            self.respond(exchange, routes::sign_in_busy());
                                        } else {
                                            self.service_sign_in(env, exchange, party, out);
                                        }
                                    }
                                    GetOwner::LinkAskService(_) => {
                                        self.respond(
                                            exchange,
                                            routes::link_ask_answer(super::w::LinkAskAnswer::Refused),
                                        );
                                    }
                                }
                            } else {
                                match owner {
                                    GetOwner::SignIn(number) if route == super::Route::LinkAsk => {
                                        let owner = GetOwner::LinkAskService(number);
                                        self.pending
                                            .exchanges
                                            .insert(exchange, Waiting::Get { owner, digest, body, route })
                                            .expect("same get slot");
                                        out.push(Request::Secrets(SecretsRequest::Get { owner: owner.secret() }));
                                    }
                                    GetOwner::SignIn(_) | GetOwner::LinkAskService(_) => {
                                        self.denied(env, exchange, route);
                                    }
                                    GetOwner::Service(_) => {
                                        self.respond(exchange, routes::refused());
                                    }
                                }
                            }
                        }
                        Waiting::Made { sign_in, party } => {
                            self.made(env, exchange, sign_in, party, value.as_ref(), out);
                        }
                        Waiting::LinkAskWhose { .. }
                        | Waiting::AskedWhose { .. }
                        | Waiting::ApprovalWhose { .. }
                        | Waiting::ApprovalCore { .. }
                        | Waiting::Core { .. }
                        | Waiting::SignOut { .. }
                        | Waiting::Watch { .. }
                        | Waiting::Keep { .. }
                        | Waiting::Moving { .. }
                        | Waiting::Reissue { .. }
                        | Waiting::Ask { .. }
                        | Waiting::Carry { .. }
                        | Waiting::Read { .. }
                        | Waiting::Authorize { .. }
                        | Waiting::ApplicationRead { .. }
                        | Waiting::Checked { .. } => {}
                    }
                }
            }
            SecretsAnswer::Carried { exchange, number } => self.carried(exchange, number, out),
            SecretsAnswer::Failed => self.abort(),
            SecretsAnswer::Erased { owner } => self.erased(owner),
        }
    }
    fn keeper(&self, owner: SecretOwner) -> Option<Token> {
        for (exchange, waiting) in &self.pending.exchanges {
            if waiting.keep_owner() == Some(owner) {
                return Some(*exchange);
            }
        }
        None
    }
    fn kept(&mut self, env: &Env<Limits>, owner: SecretOwner, out: &mut Queue<Request>) {
        let Some(exchange) = self.keeper(owner) else { return };
        let Some(waiting) = self.pending.exchanges.remove(&exchange) else { unreachable!("found keep") };
        match waiting {
            Waiting::Keep { answer, .. } => match answer {
                Answer::Approval(value) => {
                    self.respond(exchange, routes::approval_sign_in(super::w::ApprovalSignInAnswer::SignedIn(value)));
                }
                Answer::SignedIn(value) => self.respond(exchange, routes::signed_in(value)),
                Answer::Minted { credential, expires } => {
                    let exchange = self.pending.mints.remove(&exchange).expect("mint owner");
                    out.push(Request::Minted { exchange, credential, expires });
                }
                Answer::Service(answer) => {
                    if let Some(answer) = answer {
                        self.answer(exchange, Ok(answer), &env.limits.wire);
                    }
                    self.replayed(env, owner);
                    self.resume_reissue(env, owner, out);
                }
            },
            Waiting::Reissue { sign_in, phase: Reissue::Keep(answer), .. } => {
                if sign_in.is_some() {
                    self.answer(exchange, answer, &env.limits.wire);
                }
                self.replayed(env, owner);
                self.resume_reissue(env, owner, out);
            }
            Waiting::Reissue { phase: Reissue::Asked | Reissue::Draw, .. }
            | Waiting::LinkAskWhose { .. }
            | Waiting::AskedWhose { .. }
            | Waiting::ApprovalWhose { .. }
            | Waiting::ApprovalCore { .. }
            | Waiting::Authorize { .. }
            | Waiting::Core { .. }
            | Waiting::LinkGet { .. }
            | Waiting::SignOut { .. }
            | Waiting::Watch { .. }
            | Waiting::Get { .. }
            | Waiting::Made { .. }
            | Waiting::Moving { .. }
            | Waiting::Read { .. }
            | Waiting::ApplicationRead { .. }
            | Waiting::Ask { .. }
            | Waiting::Carry { .. }
            | Waiting::Checked { .. } => unreachable!("found keep"),
        }
    }
    /// The store kept a request's carried value: the core sees the request
    /// now; one whose exchange is gone, or not kept, is answered or erased.
    fn carried(&mut self, exchange: Token, number: Option<u64>, out: &mut Queue<Request>) {
        let Some(slot) = self.pending.carried.get_mut(&exchange) else {
            unreachable!("the store answers each carry once")
        };
        let waiting = self.pending.exchanges.remove(&exchange);
        let Some(number) = number else {
            self.pending.carried.remove(&exchange);
            if let Some(waiting) = waiting {
                if let Waiting::Carry { .. } = waiting {
                    self.respond(exchange, routes::busy());
                } else {
                    self.pending.exchanges.insert(exchange, waiting).expect("same slot");
                }
            }
            return;
        };
        *slot = Some(number);
        match waiting {
            Some(Waiting::Carry { sign_in, family, version, request }) => {
                let request = match *request {
                    CarriedRequest::Application { key, family, version, scope, record, digest, carried_digest } => {
                        Request::Application(crate::ToApplication::Request {
                            reply_to: ReplyTo::new(exchange),
                            sign_in,
                            key,
                            family,
                            version,
                            scope,
                            record,
                            digest,
                            carried: Some(core::Carried { number, digest: carried_digest }),
                        })
                    }
                    CarriedRequest::AccountKey { key, ask, digest } => Request::Core(Box::new(core::Event::Key {
                        reply_to: ReplyTo::new(exchange),
                        sign_in,
                        key,
                        ask,
                        carried: core::Carried { number, digest },
                    })),
                };
                self.pending
                    .exchanges
                    .insert(exchange, Waiting::Ask { sign_in, family, version })
                    .expect("same carry slot");
                out.push(request);
            }
            Some(waiting) => {
                self.pending.exchanges.insert(exchange, waiting).expect("same slot");
                self.unused(exchange, out);
            }
            None => self.unused(exchange, out),
        }
    }
    fn service_sign_in(&mut self, env: &Env<Limits>, exchange: Token, party: u64, out: &mut Queue<Request>) {
        if self.stopping() {
            self.respond(exchange, routes::sign_in_not_ready());
            return;
        }
        if self.admitted() >= env.limits.exchanges {
            self.respond(exchange, routes::refused());
            return;
        }
        let identity = people::Identity {
            key: people::IdentityKey {
                provider: self.settings.deployment_provider,
                subject: Box::new(party.to_be_bytes()),
            },
            login: Box::new([]),
            name: Box::new([]),
        };
        self.pending.exchanges.insert(exchange, Waiting::Core { held: None }).expect("same get slot");
        out.push(Request::Core(Box::new(core::Event::SignIn {
            until: None,
            held: None,
            reply_to: ReplyTo::new(exchange),
            identity,
        })));
    }
    fn reissuing(&self, party: u64) -> bool {
        for (_, waiting) in &self.pending.exchanges {
            if let Some((_, target, _)) = waiting.holding_reissue()
                && target == party
            {
                return true;
            }
        }
        false
    }
    pub(super) fn reissue_reply(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: Option<u64>,
        party: u64,
        reply: people::Reply,
        out: &mut Queue<Request>,
    ) {
        match reply {
            people::Reply::Outcome(people::Outcome::ServiceReissued { party: answered }) if answered == party => {
                if self.keeper(SecretOwner::Service(party)).is_some() {
                    self.pending
                        .exchanges
                        .insert(exchange, Waiting::Reissue { sign_in, party, phase: Reissue::Draw })
                        .expect("same reissue slot");
                } else {
                    self.draw_reissue(env, exchange, sign_in, party, out);
                }
            }
            people::Reply::Signer(_)
            | people::Reply::Full { .. }
            | people::Reply::SignedIn { .. }
            | people::Reply::SignedOut
            | people::Reply::Outcome(_)
            | people::Reply::Again(_)
            | people::Reply::Refused(_) => {
                if sign_in.is_some() {
                    let answer = crate::translate::answers::answer(&reply, &self.settings.models, &env.limits.wire);
                    self.answer(exchange, answer, &env.limits.wire);
                }
            }
        }
    }
    fn draw_reissue(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: Option<u64>,
        party: u64,
        out: &mut Queue<Request>,
    ) {
        let credential = sign_in::token(&mut self.rng, party);
        let digest = sign_in::digest(credential.expose());
        let answer = crate::translate::answers::reissued(party, credential, &env.limits.wire);
        // The core has already committed the reissue, even if its answer cannot be encoded.
        self.pending
            .exchanges
            .insert(exchange, Waiting::Reissue { sign_in, party, phase: Reissue::Keep(answer) })
            .expect("same reissue slot");
        out.push(Request::Secrets(SecretsRequest::Keep {
            owner: SecretOwner::Service(party),
            value: Kept { digest, expires: Wall::from_nanos(u64::MAX) },
        }));
    }
    fn resume_reissue(&mut self, env: &Env<Limits>, owner: SecretOwner, out: &mut Queue<Request>) {
        let mut ready = None;
        for (exchange, waiting) in &self.pending.exchanges {
            if let Some((sign_in, party, Reissue::Draw)) = waiting.holding_reissue()
                && owner == SecretOwner::Service(party)
            {
                ready = Some((*exchange, sign_in, party));
                break;
            }
        }
        if let Some((exchange, sign_in, party)) = ready {
            self.draw_reissue(env, exchange, sign_in, party, out);
        }
    }
    fn made(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        party: u64,
        kept: Option<&Kept>,
        out: &mut Queue<Request>,
    ) {
        let owner = SecretOwner::Service(party);
        if self.keeper(owner).is_some() {
            self.pending.exchanges.insert(exchange, Waiting::Made { sign_in, party }).expect("same made slot");
            return;
        }
        if kept.is_some() {
            self.answer(
                exchange,
                crate::translate::answers::service(party, Secret::new(Box::new([])), &env.limits.wire),
                &env.limits.wire,
            );
            return;
        }
        let credential = sign_in::token(&mut self.rng, party);
        let digest = sign_in::digest(credential.expose());
        let answer = crate::translate::answers::service(party, credential, &env.limits.wire);
        match answer {
            Ok(answer) => {
                self.pending
                    .exchanges
                    .insert(exchange, Waiting::Keep { sign_in, owner, answer: Answer::Service(Some(answer)) })
                    .expect("same made slot");
                out.push(Request::Secrets(SecretsRequest::Keep {
                    owner,
                    value: Kept { digest, expires: Wall::from_nanos(u64::MAX) },
                }));
            }
            Err(problem) => self.answer(exchange, Err(problem), &env.limits.wire),
        }
    }
    fn replayed(&mut self, env: &Env<Limits>, owner: SecretOwner) {
        for _ in 0..env.limits.exchanges {
            let mut found = None;
            for (exchange, waiting) in &self.pending.exchanges {
                if let Waiting::Made { party, .. } = waiting
                    && owner == SecretOwner::Service(*party)
                {
                    found = Some((*exchange, *party));
                    break;
                }
            }
            let Some((exchange, party)) = found else { break };
            self.pending.exchanges.remove(&exchange);
            self.answer(
                exchange,
                crate::translate::answers::service(party, Secret::new(Box::new([])), &env.limits.wire),
                &env.limits.wire,
            );
        }
    }
}
