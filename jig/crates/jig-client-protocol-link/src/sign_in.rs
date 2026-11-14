//! Signing belongs to the link (client/domain/client/README.md, 9).
use crate::{
    Link, Message, Request, Signing, With,
    component::{Attempt, ExchangeKind},
};
use jig_client_protocol_wire::v22 as w;
use skein_lib::{Queue, Reader, Secret, Token, Wall, Writer};
impl Link {
    #[must_use]
    pub fn credential(&self) -> Option<&Secret> {
        self.credential.as_ref()
    }
    #[must_use]
    pub fn expires(&self) -> Option<Wall> {
        self.expires
    }
    pub(super) fn sign_in(&mut self, with: With, out: &mut Queue<Request>) {
        self.cancel_approval_take();
        match with {
            With::Held(credential) => {
                self.end_sign_in(out);
                self.generation = self.generation.saturating_add(1);
                self.credential = Some(credential);
            }
            With::Transport => {
                self.end_sign_in(out);
                self.generation = self.generation.saturating_add(1);
            }
            With::Service(credential) => {
                if credential.len() != 40 {
                    out.push(Request::Signing { signing: Signing::Refused });
                    return;
                }
                self.signing_exchange(ExchangeKind::SignIn, Message::SignInService, Some(credential), None, out);
            }
            With::Approval { secret } => {
                let Ok(record) = w::ApprovalSignIn::new(self.limits.wire(), w::ApprovalSignInParts { secret }) else {
                    out.push(Request::Signing { signing: Signing::Approval(crate::Taking::Unknown) });
                    return;
                };
                let mut writer = Writer::new(skein_lib::length::usize_of(record.measure()));
                record.encode(&mut writer).expect("measured approval take");
                self.signing_exchange(
                    ExchangeKind::SignInApproval,
                    Message::SignInApproval(writer.finish()),
                    None,
                    None,
                    out,
                );
            }
            With::Link { secret, held } => {
                let Ok(record) = w::LinkSignIn::new(self.limits.wire(), w::LinkSignInParts { secret }) else {
                    out.push(Request::Signing { signing: Signing::Refused });
                    return;
                };
                let mut writer = Writer::new(skein_lib::length::usize_of(record.measure()));
                record.encode(&mut writer).expect("measured sign-in");
                self.signing_exchange(
                    ExchangeKind::SignIn,
                    Message::SignInLink(writer.finish()),
                    held.clone(),
                    held,
                    out,
                );
            }
        }
    }
    pub(super) fn sign_out(&mut self, out: &mut Queue<Request>) {
        self.cancel_approval_take();
        self.signing_exchange(
            ExchangeKind::SignOut { generation: self.generation },
            Message::SignOut,
            self.credential.clone(),
            None,
            out,
        );
    }
    fn signing_exchange(
        &mut self,
        kind: ExchangeKind,
        message: Message,
        credential: Option<Secret>,
        held: Option<Secret>,
        out: &mut Queue<Request>,
    ) {
        let Some(next) = self.next.checked_add(1) else {
            out.push(Request::Signing { signing: Signing::Lost });
            return;
        };
        if self.attempts.len() == self.attempts.capacity() {
            out.push(Request::Signing { signing: Signing::Lost });
            return;
        }
        let exchange = Token::new(self.next);
        self.next = next;
        self.attempts
            .insert(
                exchange,
                Attempt {
                    held,
                    token: exchange,
                    owner: w::Owner::Jig,
                    version: jig_client_protocol_wire::VERSION,
                    kind,
                },
            )
            .expect("checked exchange capacity");
        self.emit_exchange(exchange, credential, message, out);
    }
    pub(super) fn signing_answer(
        &mut self,
        kind: ExchangeKind,
        held: Option<Secret>,
        body: &[u8],
        out: &mut Queue<Request>,
    ) {
        match kind {
            ExchangeKind::SignOut { generation } if generation != self.generation => return,
            ExchangeKind::SignOut { .. }
            | ExchangeKind::SignIn
            | ExchangeKind::SignInApproval
            | ExchangeKind::Approval(_)
            | ExchangeKind::Authorize
            | ExchangeKind::Attempt { .. }
            | ExchangeKind::Read { .. }
            | ExchangeKind::WatchOpen { .. }
            | ExchangeKind::WatchClose { .. } => {}
        }
        let mut reader = Reader::new(body);
        let signing = {
            match kind {
                ExchangeKind::SignIn => match w::SignInAnswer::decode(self.limits.wire(), &mut reader) {
                    Ok(w::SignInAnswer::SignedIn(value))
                        if reader.remaining() == 0
                            && match value.token() {
                                Some(token) => token.len() == 40,
                                None => true,
                            } =>
                    {
                        let fields = value.into_parts();
                        self.end_sign_in(out);
                        self.credential = fields.token.or(held);
                        self.expires = Some(Wall::from_nanos(fields.expires));
                        self.generation = self.generation.saturating_add(1);
                        Signing::SignedIn { party: fields.party }
                    }
                    Ok(w::SignInAnswer::NotReady) if reader.remaining() == 0 => Signing::Again,
                    Ok(w::SignInAnswer::Busy) if reader.remaining() == 0 => Signing::Busy,
                    Ok(w::SignInAnswer::Full(value)) if reader.remaining() == 0 => Signing::Full {
                        sign_ins: value.sign_ins(),
                        earliest: Wall::from_nanos(value.earliest()),
                        link_until: Wall::from_nanos(value.link_until()),
                    },
                    Ok(w::SignInAnswer::LinkUsed) if reader.remaining() == 0 => Signing::LinkUsed,
                    Ok(w::SignInAnswer::LinkExpired) if reader.remaining() == 0 => Signing::LinkExpired,
                    Ok(w::SignInAnswer::LinkUnknown) if reader.remaining() == 0 => Signing::LinkUnknown,
                    Ok(w::SignInAnswer::Refused) if reader.remaining() == 0 => Signing::Refused,
                    Ok(_) | Err(_) => Signing::Lost,
                },
                ExchangeKind::SignOut { .. } => match w::SignOutAnswer::decode(self.limits.wire(), &mut reader) {
                    Ok(w::SignOutAnswer::SignedOut) if reader.remaining() == 0 => {
                        self.end_sign_in(out);
                        Signing::SignedOut
                    }
                    Ok(w::SignOutAnswer::NotReady) if reader.remaining() == 0 => Signing::Again,
                    Ok(_) | Err(_) => Signing::Lost,
                },
                ExchangeKind::SignInApproval => self.approval_signing_answer(&mut reader, out),
                ExchangeKind::Approval(_)
                | ExchangeKind::Authorize
                | ExchangeKind::Attempt { .. }
                | ExchangeKind::Read { .. }
                | ExchangeKind::WatchOpen { .. }
                | ExchangeKind::WatchClose { .. } => unreachable!("signing exchange"),
            }
        };
        out.push(Request::Signing { signing });
    }
    fn cancel_approval_take(&mut self) {
        let mut tokens = skein_lib::List::with_capacity(self.attempts.len());
        for (token, attempt) in &self.attempts {
            match attempt.kind {
                ExchangeKind::SignInApproval => {
                    tokens.push(*token).expect("bounded attempts");
                }
                ExchangeKind::Approval(_)
                | ExchangeKind::Authorize
                | ExchangeKind::SignIn
                | ExchangeKind::SignOut { .. }
                | ExchangeKind::Attempt { .. }
                | ExchangeKind::Read { .. }
                | ExchangeKind::WatchOpen { .. }
                | ExchangeKind::WatchClose { .. } => {}
            }
        }
        for token in &tokens {
            self.attempts.remove(token);
        }
    }
    fn approval_signing_answer(&mut self, reader: &mut Reader<'_>, out: &mut Queue<Request>) -> Signing {
        let Ok(answer) = w::ApprovalSignInAnswer::decode(self.limits.wire(), reader) else { return Signing::Lost };
        if !reader.is_empty() {
            return Signing::Lost;
        }
        match answer {
            w::ApprovalSignInAnswer::SignedIn(value) => {
                if let Some(token) = value.token()
                    && token.len() != 40
                {
                    return Signing::Lost;
                }
                let fields = value.into_parts();
                self.end_sign_in(out);
                self.credential = fields.token;
                self.expires = Some(Wall::from_nanos(fields.expires));
                self.generation = self.generation.saturating_add(1);
                Signing::SignedIn { party: fields.party }
            }
            w::ApprovalSignInAnswer::Waiting => Signing::Approval(crate::Taking::Waiting),
            w::ApprovalSignInAnswer::Refused => Signing::Approval(crate::Taking::Refused),
            w::ApprovalSignInAnswer::Expired => Signing::Approval(crate::Taking::Expired),
            w::ApprovalSignInAnswer::Used => Signing::Approval(crate::Taking::Used),
            w::ApprovalSignInAnswer::Unknown => Signing::Approval(crate::Taking::Unknown),
            w::ApprovalSignInAnswer::NotReady => Signing::Again,
            w::ApprovalSignInAnswer::Busy => Signing::Busy,
            w::ApprovalSignInAnswer::Full(value) => Signing::Full {
                sign_ins: value.sign_ins(),
                earliest: Wall::from_nanos(value.earliest()),
                link_until: Wall::from_nanos(value.until()),
            },
        }
    }
}
