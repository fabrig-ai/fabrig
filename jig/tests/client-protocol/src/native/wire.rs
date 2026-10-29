//! Independent facts at the native byte boundary (jig/docs/design/protocol/service.md, 8).
use super::Received;
use jig_client::{Event, Message, Request, Streamed};
use jig_client_protocol_link::translate;
use jig_client_protocol_wire::v19 as w;
use skein_lib::{Reader, Secret, Token};
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
enum Kind {
    Request(Option<(u64, u32)>, skein_lib::RequestKey),
    Read(Option<jig_client_test_domain::family::Subject>),
    SignIn,
    SignOut,
    Other,
}
/// Every completed printed-link exchange, including answers without a credential.
#[derive(Clone, Copy, Debug)]
pub enum SignInOutcome {
    SignedIn,
    NotSignedIn(jig_client::NotSignedIn),
    Refused,
    Lost,
}
#[derive(Default)]
pub struct Wire {
    diagnostics: bool,
    exchanges: BTreeMap<Token, Kind>,
    watches: BTreeMap<u64, jig_client_test_domain::family::Subject>,
    credential: Option<Secret>,
    pub(super) sign_in_answers: Vec<SignInOutcome>,
    terminals: BTreeMap<skein_lib::RequestKey, jig_client::Outcome>,
}
impl Wire {
    pub(super) fn diagnose(&mut self) {
        self.diagnostics = true;
    }
    #[must_use]
    pub fn credential(&self) -> Option<Secret> {
        self.credential.clone()
    }
    pub fn restore(&mut self, credential: Option<Secret>) {
        self.credential = credential;
    }
    /// A carried request's decoded terminal, retained apart from `Received`.
    #[must_use]
    pub fn carried_terminal(&self, name: skein_lib::RequestKey) -> Option<&jig_client::Outcome> {
        self.terminals.get(&name)
    }
    pub fn sent(&mut self, request: &Request) -> Vec<Received> {
        self.sent_with(request, Some)
    }
    /// Track jig envelopes while leaving an application's bytes to its observer.
    pub fn sent_without_family(&mut self, request: &Request) -> Vec<Received> {
        self.sent_with(request, |_| None)
    }
    fn sent_with(
        &mut self,
        request: &Request,
        family: fn(jig_client::Family) -> Option<jig_client::Family>,
    ) -> Vec<Received> {
        if matches!(request, Request::CloseStream) {
            self.watches.clear();
        }
        let Request::Exchange { exchange, message, .. } = request else { return vec![] };
        let kind = match message {
            Message::Request(bytes) => {
                let keyed = w::Keyed::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("keyed request bytes");
                Kind::Request(compound::message(bytes), *keyed.key())
            }
            Message::Read(bytes) => {
                let record = w::Owned::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("owned read");
                let subject = match record.owner() {
                    w::Owner::Jig => None,
                    w::Owner::Family(owner) => family(jig_client::Family {
                        family: owner.number(),
                        version: record.version(),
                        bytes: w::FamilyRecord::decode(&w::CEILINGS, &mut Reader::new(record.record()))
                            .expect("family read envelope")
                            .into_parts()
                            .record,
                    })
                    .map(|record| jig_client_world::test::family_subject(&record, true)),
                };
                Kind::Read(subject)
            }
            Message::SignInLink(_) | Message::SignInService => Kind::SignIn,
            Message::SignOut => Kind::SignOut,
            Message::Watch(bytes) => {
                let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("open bytes");
                if let w::Owner::Family(owner) = open.subject().owner()
                    && let Some(record) = family(jig_client::Family {
                        family: owner.number(),
                        version: open.subject().version(),
                        bytes: w::FamilyRecord::decode(&w::CEILINGS, &mut Reader::new(open.subject().record()))
                            .expect("family subject envelope")
                            .into_parts()
                            .record,
                    })
                {
                    let subject = jig_client_world::test::family_subject(&record, false);
                    self.watches.insert(open.watch(), subject);
                }
                Kind::Other
            }
            Message::Authorize(_) | Message::Unwatch(_) | Message::StreamGone(_) => Kind::Other,
        };
        self.exchanges.insert(*exchange, kind);
        if let Message::Request(bytes) = message {
            let keyed = w::Keyed::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("keyed request bytes");
            vec![Received::Sent { attempt: *exchange, key: *keyed.key() }]
        } else {
            vec![]
        }
    }
    #[must_use]
    pub fn sign_in_refused(&self, event: &Event) -> bool {
        let Event::Reply { exchange, message } = event else { return false };
        matches!(self.exchanges.get(exchange), Some(Kind::SignIn))
            && matches!(w::SignInAnswer::decode(&w::CEILINGS, &mut Reader::new(message)), Ok(w::SignInAnswer::Refused))
    }
    #[must_use]
    pub fn additional_receipts(&self, event: &Event) -> Vec<jig_client_world::harness::Receipt> {
        use jig_client_world::harness::Receipt;
        if let Event::Streamed { message: Streamed::Record(bytes) } = event {
            if let w::StreamEvent::Delivery(value) =
                w::StreamEvent::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("stream bytes")
                && matches!(value.document().owner(), w::Owner::Family(_))
                && let Some(subject) = self.watches.get(&value.watch())
                && let Some(document) = crate::family::document(*subject, value.document().record())
            {
                return jig_client_world::observe::received(&jig_client_test_domain::Event::First(
                    jig_client_test_domain::first::Event::Watch {
                        watch: Token::new(value.watch()),
                        event: jig_client_test_domain::first::WatchEvent::Delivered(document),
                    },
                ));
            }
            return Vec::new();
        }
        let Event::Reply { exchange, message } = event else { return Vec::new() };
        match self.exchanges.get(exchange) {
            Some(Kind::Request(words, _)) => {
                match w::KeyedAnswer::decode(&w::CEILINGS, &mut Reader::new(message)).expect("answer bytes") {
                    w::KeyedAnswer::FinalAnswer(value) if matches!(value.owner(), w::Owner::Family(_)) => {
                        vec![Receipt::Answer { attempt: *exchange, final_answer: true, words: None }]
                    }
                    w::KeyedAnswer::FinalAnswer(value) => compound::receipt(*exchange, *words, value.record()),
                    w::KeyedAnswer::NotServed(_) | w::KeyedAnswer::SignedOut => vec![Receipt::Parked],
                    w::KeyedAnswer::Busy | w::KeyedAnswer::NotReady | w::KeyedAnswer::AnswersFull(_) => {
                        vec![Receipt::Answer { attempt: *exchange, final_answer: false, words: None }]
                    }
                }
            }
            Some(Kind::Read(Some(subject))) => {
                let w::ReadAnswer::Value(value) =
                    w::ReadAnswer::decode(&w::CEILINGS, &mut Reader::new(message)).expect("read bytes")
                else {
                    return Vec::new();
                };
                let value = crate::family::read_result(*subject, value.record());
                jig_client_world::observe::received(&jig_client_test_domain::Event::First(
                    jig_client_test_domain::first::Event::Read { read: *exchange, result: value },
                ))
            }
            Some(Kind::Read(None) | Kind::SignIn | Kind::SignOut | Kind::Other) | None => Vec::new(),
        }
    }
    pub fn received(&mut self, event: &Event) -> Vec<Received> {
        self.received_with(event, crate::fixtures::link_limits())
    }
    /// Decode the same public bytes under the application's declared receiving profile.
    pub fn received_with(&mut self, event: &Event, limits: jig_client_protocol_link::Limits) -> Vec<Received> {
        match event {
            Event::Reply { exchange, message } => match self.exchanges.remove(exchange) {
                Some(Kind::Request(_, name)) => {
                    let answer = w::KeyedAnswer::decode(&w::CEILINGS, &mut Reader::new(message)).expect("answer bytes");
                    if self.diagnostics {
                        self.terminals.insert(name, request_terminal(&answer, &limits));
                    }
                    translate::answers::answer(answer, &limits)
                        .map_or_else(|_| vec![], |answer| vec![Received::Answer { attempt: *exchange, answer }])
                }
                Some(Kind::Read(_)) => {
                    let answer = w::ReadAnswer::decode(&w::CEILINGS, &mut Reader::new(message)).expect("read bytes");
                    match answer {
                        w::ReadAnswer::Value(value) if matches!(value.owner(), w::Owner::Jig) => {
                            translate::reads::value(value.record(), &limits).map_or_else(
                                |_| vec![],
                                |value| vec![Received::Read(jig_client::contract::ReadResult::Value(value))],
                            )
                        }
                        w::ReadAnswer::None(value) => vec![Received::Read(jig_client::contract::ReadResult::None(
                            translate::reads::none(value.value()),
                        ))],
                        w::ReadAnswer::NotServed(_) | w::ReadAnswer::Value(_) => vec![],
                    }
                }
                Some(Kind::SignIn) => {
                    let answer =
                        w::SignInAnswer::decode(&w::CEILINGS, &mut Reader::new(message)).expect("sign-in bytes");
                    if self.diagnostics {
                        self.sign_in_answers.push(sign_in_outcome(&answer));
                    }
                    match answer {
                        w::SignInAnswer::SignedIn(value) => {
                            self.credential.clone_from(value.token());
                            vec![Received::SignedIn { person: value.party() }]
                        }
                        w::SignInAnswer::Refused
                        | w::SignInAnswer::NotReady
                        | w::SignInAnswer::Busy
                        | w::SignInAnswer::Full(_) => vec![],
                    }
                }
                Some(Kind::SignOut) => {
                    if let w::SignOutAnswer::SignedOut =
                        w::SignOutAnswer::decode(&w::CEILINGS, &mut Reader::new(message)).expect("sign-out bytes")
                    {
                        self.credential = None;
                    }
                    vec![]
                }
                Some(Kind::Other) | None => vec![],
            },
            Event::Streamed { message: Streamed::Record(bytes) } => {
                match w::StreamEvent::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("stream bytes") {
                    w::StreamEvent::Delivery(value) if matches!(value.document().owner(), w::Owner::Jig) => {
                        translate::watches::delivery(value.document().record(), &limits)
                            .map_or_else(|_| vec![], |value| vec![Received::Delivery(value)])
                    }
                    w::StreamEvent::Hello(_)
                    | w::StreamEvent::Delivery(_)
                    | w::StreamEvent::Text(_)
                    | w::StreamEvent::Ended(_) => vec![],
                }
            }
            Event::Lost { exchange } => {
                match self.exchanges.remove(exchange) {
                    Some(Kind::SignIn) if self.diagnostics => self.sign_in_answers.push(SignInOutcome::Lost),
                    Some(Kind::Request(_, name)) if self.diagnostics => {
                        self.terminals.insert(name, jig_client::Outcome::Lost);
                    }
                    Some(Kind::Read(_) | Kind::SignOut | Kind::Other | Kind::SignIn | Kind::Request(_, _)) | None => {}
                }
                vec![]
            }
            Event::Start { .. }
            | Event::StartUnlinked { .. }
            | Event::SignIn { .. }
            | Event::Authorize { .. }
            | Event::SignOut
            | Event::SignOutLost
            | Event::Watch { .. }
            | Event::Unwatch { .. }
            | Event::Read { .. }
            | Event::Send { .. }
            | Event::Carry { .. }
            | Event::Streamed { message: Streamed::Heartbeat | Streamed::SignedOut }
            | Event::StreamGone
            | Event::Stored
            | Event::NotStored
            | Event::Up { .. } => vec![],
        }
    }
}

fn request_terminal(answer: &w::KeyedAnswer, limits: &jig_client_protocol_link::Limits) -> jig_client::Outcome {
    match answer {
        w::KeyedAnswer::AnswersFull(full) => {
            jig_client::Outcome::Full { held: full.held(), room_at: skein_lib::Wall::from_nanos(full.room_at()) }
        }
        w::KeyedAnswer::Busy | w::KeyedAnswer::NotReady | w::KeyedAnswer::NotServed(_) | w::KeyedAnswer::SignedOut => {
            jig_client::Outcome::Unsent
        }
        w::KeyedAnswer::FinalAnswer(_) => match translate::answers::answer(answer.clone(), limits) {
            Ok(answer) => jig_client::Outcome::Answered(jig_client::Answered::Jig(answer)),
            Err(_) => jig_client::Outcome::Lost,
        },
    }
}
fn sign_in_outcome(answer: &w::SignInAnswer) -> SignInOutcome {
    match answer {
        w::SignInAnswer::SignedIn(_) => SignInOutcome::SignedIn,
        w::SignInAnswer::Refused => SignInOutcome::Refused,
        w::SignInAnswer::NotReady => SignInOutcome::NotSignedIn(jig_client::NotSignedIn::NotReady),
        w::SignInAnswer::Busy => SignInOutcome::NotSignedIn(jig_client::NotSignedIn::Busy),
        w::SignInAnswer::Full(full) => SignInOutcome::NotSignedIn(jig_client::NotSignedIn::Full {
            sign_ins: full.sign_ins(),
            earliest: skein_lib::Wall::from_nanos(full.earliest()),
            link_until: skein_lib::Wall::from_nanos(full.link_until()),
        }),
    }
}

#[cfg(test)]
mod tests;

mod compound;

#[cfg(test)]
mod diagnostic_tests {
    use super::*;
    use skein_lib::Writer;

    /// Carried requests stop at retryable wire answers instead of being sent again.
    #[test]
    fn retryable_carried_answers_have_an_unsent_terminal() {
        const NAME: skein_lib::RequestKey = [61; 16];
        const EXCHANGE: Token = Token::new(7);
        for answer in [w::KeyedAnswer::Busy, w::KeyedAnswer::NotReady, w::KeyedAnswer::SignedOut] {
            let mut wire = Wire::default();
            wire.diagnose();
            wire.exchanges.insert(EXCHANGE, Kind::Request(None, NAME));
            let mut writer = Writer::new(usize::try_from(answer.measure()).expect("fixture length"));
            answer.encode(&mut writer).expect("fixture answer");
            wire.received(&Event::Reply { exchange: EXCHANGE, message: writer.finish() });
            assert_eq!(wire.carried_terminal(NAME), Some(&jig_client::Outcome::Unsent));
        }
    }
    /// Loss remains visible even though no decoded `Received` fact is made.
    #[test]
    fn lost_exchanges_retain_sign_in_and_carried_terminals() {
        const NAME: skein_lib::RequestKey = [61; 16];
        const SIGN_IN: Token = Token::new(7);
        const CARRY: Token = Token::new(8);
        let mut wire = Wire::default();
        wire.diagnose();
        wire.exchanges.insert(SIGN_IN, Kind::SignIn);
        wire.exchanges.insert(CARRY, Kind::Request(None, NAME));
        wire.received(&Event::Lost { exchange: SIGN_IN });
        wire.received(&Event::Lost { exchange: CARRY });
        assert!(matches!(wire.sign_in_answers.as_slice(), [SignInOutcome::Lost]));
        assert_eq!(wire.carried_terminal(NAME), Some(&jig_client::Outcome::Lost));
    }
}
