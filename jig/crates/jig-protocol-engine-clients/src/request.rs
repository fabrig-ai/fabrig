//! Keyed request admission and version envelopes (protocol/client/README.md, 3 and 6).
use super::{Box, Component, Env, Limits, Queue, ReplyTo, Request, Token, Waiting, core, people, routes, sign_in, w};
use crate::sign_in::CarriedRequest;
use skein_lib::length::len_u64;
impl Component {
    pub(super) fn request(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        body: &[u8],
        out: &mut Queue<Request>,
    ) {
        if self.stopping() {
            self.respond(exchange, routes::not_ready());
            return;
        }
        let Some(owner) = self.owner(exchange) else {
            return;
        };
        let active = self.active(sign_in);
        if active >= env.limits.exchanges_per_client || self.admitted() >= env.limits.exchanges {
            self.respond(exchange, routes::busy());
            return;
        }
        let keyed = match w::Keyed::decode(&env.limits.wire, &mut skein_lib::Reader::new(body)) {
            Ok(value) => value.into_parts(),
            Err(_) => {
                self.not_a_record(owner);
                return;
            }
        };
        let family = match &keyed.owner {
            w::Owner::Jig => None,
            w::Owner::Family(value) => Some(value.number()),
        };
        if !self.serves(family, keyed.version) {
            let answer =
                w::KeyedAnswer::NotServed(crate::served::not_served(&self.settings, keyed.owner, &env.limits.wire));
            self.respond(exchange, routes::encoded(&answer));
            return;
        }
        let request = match keyed.owner {
            w::Owner::Jig => {
                self.jig_request(env, exchange, sign_in, owner, keyed, out);
                return;
            }
            w::Owner::Family(family) => {
                let Ok(record) =
                    w::FamilyRequest::decode(&env.limits.wire, &mut skein_lib::Reader::new(&keyed.request))
                else {
                    self.not_a_record(owner);
                    return;
                };
                let scope = crate::served::scope(record.scope());
                if let Some(value) = keyed.value {
                    let request = CarriedRequest::Application {
                        key: keyed.key,
                        family: family.number(),
                        version: keyed.version,
                        scope,
                        digest: sign_in::carrying(record.record(), &value),
                        record: record.into_parts().record,
                        carried_digest: skein_lib::sha256::digest(value.expose()),
                    };
                    self.carry(exchange, sign_in, request, value, Some(crate::CarriedKind::Value), out);
                    return;
                }
                let digest = sign_in::digest(record.record()).expose().try_into().expect("32-byte digest");
                let request = crate::ToApplication::Request {
                    reply_to: ReplyTo::new(exchange),
                    sign_in,
                    key: keyed.key,
                    family: family.number(),
                    version: keyed.version,
                    scope,
                    record: record.into_parts().record,
                    digest,
                    carried: None,
                };
                Request::Application(request)
            }
        };
        self.pending
            .exchanges
            .insert(exchange, Waiting::Ask { sign_in, family, version: keyed.version })
            .expect("get released slot");
        out.push(request);
    }
    /// Admit jig's key or moved file before asking the store to keep its value.
    #[expect(
        clippy::too_many_lines,
        reason = "one keyed entrance handles the ordinary ask, a carried value, and a whole settings change"
    )]
    fn jig_request(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        owner: Token,
        keyed: w::KeyedParts,
        out: &mut Queue<Request>,
    ) {
        let mut kind = None;
        let ask = match w::Ask::decode(&env.limits.wire, &mut skein_lib::Reader::new(&keyed.request)) {
            Ok(value) => {
                let value = match value {
                    w::Ask::Propose(value)
                        if match value.action() {
                            w::ProposeAction::Settings(_) => true,
                            w::ProposeAction::Batch(_) | w::ProposeAction::Amendment(_) => false,
                        } =>
                    {
                        if keyed.value.is_some() {
                            self.not_a_record(owner);
                            return;
                        }
                        if self.refuse_llm(env, exchange, &w::Ask::Propose(value.clone())) {
                            return;
                        }
                        let Ok(change) =
                            crate::translate::settings::proposed(value, &self.settings.models, &env.limits.wire)
                        else {
                            self.not_a_record(owner);
                            return;
                        };
                        let digest = sign_in::digest(&keyed.request).expose().try_into().expect("32-byte digest");
                        self.pending
                            .exchanges
                            .insert(exchange, Waiting::Ask { sign_in, family: None, version: keyed.version })
                            .expect("released slot");
                        out.push(Request::Core(Box::new(core::Event::ProposeSettings {
                            reply_to: ReplyTo::new(exchange),
                            sign_in,
                            key: keyed.key,
                            digest,
                            change: Box::new(change),
                        })));
                        return;
                    }
                    w::Ask::ChangeModel(value) => {
                        if keyed.value.is_some() {
                            self.not_a_record(owner);
                            return;
                        }
                        let act = crate::translate::settings::model_act(value);
                        let digest = sign_in::digest(&keyed.request).expose().try_into().expect("32-byte digest");
                        self.pending
                            .exchanges
                            .insert(exchange, Waiting::Ask { sign_in, family: None, version: keyed.version })
                            .expect("released slot");
                        out.push(Request::Core(Box::new(core::Event::ChangeModel {
                            reply_to: ReplyTo::new(exchange),
                            sign_in,
                            key: keyed.key,
                            digest,
                            act,
                        })));
                        return;
                    }
                    w::Ask::ChangeSettings(value) => {
                        if keyed.value.is_some() {
                            self.not_a_record(owner);
                            return;
                        }
                        if self.refuse_llm(env, exchange, &w::Ask::ChangeSettings(value.clone())) {
                            return;
                        }
                        let Ok(change) =
                            crate::translate::settings::change(value, &self.settings.models, &env.limits.wire)
                        else {
                            self.not_a_record(owner);
                            return;
                        };
                        let digest = sign_in::digest(&keyed.request).expose().try_into().expect("32-byte digest");
                        self.pending
                            .exchanges
                            .insert(exchange, Waiting::Ask { sign_in, family: None, version: keyed.version })
                            .expect("get released slot");
                        out.push(Request::Core(Box::new(core::Event::ChangeSettings {
                            reply_to: ReplyTo::new(exchange),
                            sign_in,
                            key: keyed.key,
                            digest,
                            change: Box::new(change),
                        })));
                        return;
                    }
                    other @ (w::Ask::ReadLists(_)
                    | w::Ask::AccountChange(_)
                    | w::Ask::Amend(_)
                    | w::Ask::Answer(_)
                    | w::Ask::Cancel(_)
                    | w::Ask::ChangePolicy(_)
                    | w::Ask::Compound(_)
                    | w::Ask::Create(_)
                    | w::Ask::Decide(_)
                    | w::Ask::DeleteNote(_)
                    | w::Ask::EditNote(_)
                    | w::Ask::HandBack(_)
                    | w::Ask::MakeProject(_)
                    | w::Ask::MakeService(_)
                    | w::Ask::Talk(_)
                    | w::Ask::Leave(_)
                    | w::Ask::Message(_)
                    | w::Ask::MoveTask(_)
                    | w::Ask::MoveToNewest(_)
                    | w::Ask::Restore(_)
                    | w::Ask::Propose(_)
                    | w::Ask::Release(_)
                    | w::Ask::RemoveProject(_)
                    | w::Ask::SetVisibility(_)
                    | w::Ask::Rename(_)
                    | w::Ask::Seen(_)
                    | w::Ask::SetPool(_)
                    | w::Ask::SetRoles(_)
                    | w::Ask::SignIn(_)
                    | w::Ask::SignOut(_)
                    | w::Ask::Lift(_)
                    | w::Ask::Stop(_)
                    | w::Ask::Take(_)
                    | w::Ask::Withdraw(_)) => other,
                };
                let carrying = account_key_digest(&value);
                if let Some(carried) = &keyed.value {
                    let bound = carried_bound(
                        &value,
                        self.settings.key_bytes,
                        self.settings.moved_bytes,
                        self.settings.redirect_bytes,
                    );
                    if len_u64(carried.len()) > bound && carrying.is_some() {
                        let reply = people::Reply::Refused(people::Refusal::Limit { which: people::LimitKind::Words });
                        let answer = crate::translate::answers::answer(&reply, &self.settings.models, &env.limits.wire);
                        self.answer(exchange, answer);
                        return;
                    }
                }
                let valid = match carrying {
                    Some((digest, value_kind)) => {
                        kind = value_kind;
                        match &keyed.value {
                            Some(value) => digest == skein_lib::sha256::digest(value.expose()),
                            None => false,
                        }
                    }
                    None => keyed.value.is_none(),
                };
                if !valid {
                    self.not_a_record(owner);
                    return;
                }
                if let Ok(Some(answer)) = crate::translate::operations::refused(&value, &env.limits.wire) {
                    self.respond(exchange, routes::encoded(&answer));
                    return;
                }
                if self.refuse_llm(env, exchange, &value) {
                    return;
                }
                crate::translate::asks::named(value, &self.settings.models, &env.limits.wire)
            }
            Err(problem) => Err(crate::translate::Untranslatable { path: problem.path }),
        };
        let Ok(ask) = ask else {
            self.not_a_record(owner);
            return;
        };
        if let Some(value) = keyed.value {
            let digest = skein_lib::sha256::digest(value.expose());
            let request = CarriedRequest::AccountKey { key: keyed.key, ask: Box::new(ask.ask), digest };
            self.carry(exchange, sign_in, request, value, kind, out);
            return;
        }
        let request = Request::Core(Box::new(core::Event::People(people::Event::Ask {
            reply_to: ReplyTo::new(exchange),
            sign_in,
            key: keyed.key,
            ask: ask.ask,
            models: ask.models,
        })));
        self.pending
            .exchanges
            .insert(exchange, Waiting::Ask { sign_in, family: None, version: keyed.version })
            .expect("get released slot");
        out.push(request);
    }
    /// Answer unsupported model choices before translating names to core indices
    /// (client/domain/contract/README.md, 3). Malformed records remain the caller's to reject.
    fn refuse_llm(&mut self, env: &Env<Limits>, exchange: Token, ask: &w::Ask) -> bool {
        if let Ok(Some(answer)) = crate::translate::llm::refused(ask, &self.settings.models, &env.limits.wire) {
            self.respond(exchange, routes::encoded(&answer));
            return true;
        }
        false
    }
    fn not_a_record(&mut self, owner: Token) {
        self.connections.get_mut(&owner).expect("exchange owner").respond(crate::Reply::NotARecord);
    }
    /// Keeps the value a request carried before the core sees the
    /// request, which names it by number and digest only
    /// (protocol/client/README.md, 3; protocol/store/secrets.md).
    fn carry(
        &mut self,
        exchange: Token,
        sign_in: u64,
        request: CarriedRequest,
        value: skein_lib::Secret,
        kind: Option<crate::CarriedKind>,
        out: &mut Queue<Request>,
    ) {
        if self.pending.carried.len() == self.pending.carried.capacity() {
            self.respond(exchange, routes::busy());
            return;
        }
        let (family, version) = match &request {
            CarriedRequest::Application { family, version, .. } => (Some(*family), *version),
            CarriedRequest::AccountKey { .. } => (None, jig_client_protocol_wire::VERSION),
        };
        let waiting = Waiting::Carry { sign_in, family, version, request: Box::new(request) };
        self.pending.carried.insert(exchange, None).expect("carried below its capacity");
        self.pending.exchanges.insert(exchange, waiting).expect("get released slot");
        let request = match kind {
            None => crate::SecretsRequest::Hold { exchange, value },
            Some(kind) => crate::SecretsRequest::Carry { exchange, value, kind },
        };
        out.push(Request::Secrets(request));
    }
}

/// The value's digest is the sole secret-derived fact admitted to the core.
fn account_key_digest(ask: &w::Ask) -> Option<([u8; 32], Option<crate::CarriedKind>)> {
    let change = match ask {
        w::Ask::AccountChange(row) => row.change(),
        w::Ask::ReadLists(_)
        | w::Ask::ChangeModel(_)
        | w::Ask::ChangeSettings(_)
        | w::Ask::MoveToNewest(_)
        | w::Ask::Restore(_)
        | w::Ask::Talk(_)
        | w::Ask::Leave(_)
        | w::Ask::Message(_)
        | w::Ask::Answer(_)
        | w::Ask::Take(_)
        | w::Ask::HandBack(_)
        | w::Ask::Decide(_)
        | w::Ask::Release(_)
        | w::Ask::Amend(_)
        | w::Ask::Lift(_)
        | w::Ask::Stop(_)
        | w::Ask::Cancel(_)
        | w::Ask::MoveTask(_)
        | w::Ask::Seen(_)
        | w::Ask::EditNote(_)
        | w::Ask::DeleteNote(_)
        | w::Ask::ChangePolicy(_)
        | w::Ask::SetRoles(_)
        | w::Ask::MakeService(_)
        | w::Ask::RemoveProject(_)
        | w::Ask::SetVisibility(_)
        | w::Ask::Rename(_)
        | w::Ask::MakeProject(_)
        | w::Ask::SetPool(_)
        | w::Ask::Compound(_)
        | w::Ask::Create(_)
        | w::Ask::Propose(_)
        | w::Ask::Withdraw(_)
        | w::Ask::SignIn(_)
        | w::Ask::SignOut(_) => return None,
    };
    match change {
        w::AccountChange::AddKey(row) => Some((*row.digest(), Some(crate::CarriedKind::Key))),
        w::AccountChange::ReplaceKey(row) => Some((*row.digest(), Some(crate::CarriedKind::Key))),
        w::AccountChange::FinishSignIn(row) => Some((*row.digest(), None)),
        w::AccountChange::MoveIn(row) => Some((*row.digest(), Some(crate::CarriedKind::Moved))),
        w::AccountChange::LoginProgram(_)
        | w::AccountChange::CancelLogin(_)
        | w::AccountChange::LoginEnded(_)
        | w::AccountChange::DeclareProgram(_)
        | w::AccountChange::CheckProgram(_)
        | w::AccountChange::RemoveKey(_)
        | w::AccountChange::Resume(_)
        | w::AccountChange::Remove(_)
        | w::AccountChange::MoveAccount(_)
        | w::AccountChange::MoveSource(_)
        | w::AccountChange::SetUses(_)
        | w::AccountChange::MoveOut(_)
        | w::AccountChange::Written(_) => None,
    }
}

/// Only account key changes and a moved file carry jig values.
fn carried_bound(ask: &w::Ask, key_bytes: u32, moved_bytes: u64, redirect_bytes: u32) -> u64 {
    let change = match ask {
        w::Ask::AccountChange(row) => row.change(),
        w::Ask::ReadLists(_)
        | w::Ask::ChangeModel(_)
        | w::Ask::ChangeSettings(_)
        | w::Ask::MoveToNewest(_)
        | w::Ask::Restore(_)
        | w::Ask::Talk(_)
        | w::Ask::Leave(_)
        | w::Ask::Message(_)
        | w::Ask::Answer(_)
        | w::Ask::Take(_)
        | w::Ask::HandBack(_)
        | w::Ask::Decide(_)
        | w::Ask::Release(_)
        | w::Ask::Amend(_)
        | w::Ask::Lift(_)
        | w::Ask::Stop(_)
        | w::Ask::Cancel(_)
        | w::Ask::MoveTask(_)
        | w::Ask::Seen(_)
        | w::Ask::EditNote(_)
        | w::Ask::DeleteNote(_)
        | w::Ask::ChangePolicy(_)
        | w::Ask::SetRoles(_)
        | w::Ask::MakeService(_)
        | w::Ask::RemoveProject(_)
        | w::Ask::SetVisibility(_)
        | w::Ask::Rename(_)
        | w::Ask::MakeProject(_)
        | w::Ask::SetPool(_)
        | w::Ask::Compound(_)
        | w::Ask::Create(_)
        | w::Ask::Propose(_)
        | w::Ask::Withdraw(_)
        | w::Ask::SignIn(_)
        | w::Ask::SignOut(_) => return 0,
    };
    match change {
        w::AccountChange::FinishSignIn(_) => u64::from(redirect_bytes),
        w::AccountChange::MoveIn(_) => moved_bytes,
        w::AccountChange::AddKey(_) | w::AccountChange::ReplaceKey(_) => u64::from(key_bytes),
        w::AccountChange::LoginProgram(_)
        | w::AccountChange::CancelLogin(_)
        | w::AccountChange::LoginEnded(_)
        | w::AccountChange::DeclareProgram(_)
        | w::AccountChange::CheckProgram(_)
        | w::AccountChange::RemoveKey(_)
        | w::AccountChange::Resume(_)
        | w::AccountChange::Remove(_)
        | w::AccountChange::MoveAccount(_)
        | w::AccountChange::MoveSource(_)
        | w::AccountChange::SetUses(_)
        | w::AccountChange::MoveOut(_)
        | w::AccountChange::Written(_) => 0,
    }
}
