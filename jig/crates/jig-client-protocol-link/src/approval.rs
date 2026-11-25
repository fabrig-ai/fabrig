//! Approval exchanges are transient and answered once (jig/docs/design/protocol/client/sign-in.md).
use crate::component::{Attempt, ExchangeKind};
use crate::{Approval, ApprovalCode, ApprovalState, Decided, Link, Message, NotAsked, Request, Signing};
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_client_protocol_wire::VERSION;
use skein_lib::{Queue, Reader, Wall, Writer};

#[derive(Clone, Copy, Debug)]
pub(super) enum ApprovalKind {
    Ask,
    Read { code: ApprovalCode },
    Decide { code: ApprovalCode },
}
impl ApprovalKind {
    fn asking(self) -> bool {
        match self {
            Self::Ask => true,
            Self::Read { .. } | Self::Decide { .. } => false,
        }
    }
    pub(super) fn lost(self) -> Approval {
        match self {
            Self::Ask => Approval::NotAsked(NotAsked::Lost),
            Self::Read { code } => Approval::Read { code, state: ApprovalState::Lost },
            Self::Decide { code } => Approval::Decided { code, decided: Decided::Lost },
        }
    }
    fn busy(self) -> Approval {
        match self {
            Self::Ask => Approval::NotAsked(NotAsked::Busy),
            Self::Read { code } => Approval::Read { code, state: ApprovalState::Busy },
            Self::Decide { code } => Approval::Decided { code, decided: Decided::Busy },
        }
    }
}
impl Link {
    pub(super) fn ask_approval(&mut self, port: Option<u16>, out: &mut Queue<Request>) {
        let record = w::ApprovalAsk { port };
        let mut bytes = Writer::new(skein_lib::length::usize_of(
            record.measure(self.limits.wire()).expect("bounded client record"),
        ));
        record.write(self.limits.wire(), &mut bytes).expect("measured approval ask");
        self.approval_exchange(ApprovalKind::Ask, Message::ApprovalAsk(bytes.finish()), out);
    }
    pub(super) fn read_approval(&mut self, code: ApprovalCode, out: &mut Queue<Request>) {
        let record = w::ApprovalRead { code: *code.letters() };
        let mut bytes = Writer::new(skein_lib::length::usize_of(
            record.measure(self.limits.wire()).expect("bounded client record"),
        ));
        record.write(self.limits.wire(), &mut bytes).expect("measured approval read");
        self.approval_exchange(ApprovalKind::Read { code }, Message::ApprovalRead(bytes.finish()), out);
    }
    pub(super) fn decide_approval(&mut self, code: ApprovalCode, approve: bool, out: &mut Queue<Request>) {
        let record = w::ApprovalDecide { code: *code.letters(), approve };
        let mut bytes = Writer::new(skein_lib::length::usize_of(
            record.measure(self.limits.wire()).expect("bounded client record"),
        ));
        record.write(self.limits.wire(), &mut bytes).expect("measured approval decision");
        self.approval_exchange(ApprovalKind::Decide { code }, Message::ApprovalDecide(bytes.finish()), out);
    }
    fn approval_exchange(&mut self, kind: ApprovalKind, message: Message, out: &mut Queue<Request>) {
        for (_, attempt) in &self.attempts {
            match attempt.kind {
                ExchangeKind::Approval(other) if other.asking() == kind.asking() => {
                    out.push(Request::Approval(kind.busy()));
                    return;
                }
                ExchangeKind::LinkAsk
                | ExchangeKind::Approval(_)
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
            out.push(Request::Approval(kind.busy()));
            return;
        }
        let Some(exchange) = self.token() else {
            out.push(Request::Approval(kind.lost()));
            return;
        };
        self.attempts
            .insert(
                exchange,
                Attempt {
                    parts: skein_lib::List::with_capacity(0),
                    token: exchange,
                    owner: w::Owner::Jig,
                    version: VERSION,
                    kind: ExchangeKind::Approval(kind),
                    held: None,
                },
            )
            .expect("checked exchange capacity");
        let credential = if kind.asking() { None } else { self.credential.clone() };
        self.emit_exchange(exchange, credential, message);
    }
    pub(super) fn approval_answer(&mut self, kind: ApprovalKind, body: &[u8], out: &mut Queue<Request>) {
        let mut reader = Reader::new(body);
        let answer = match kind {
            ApprovalKind::Ask => match w::ApprovalAskAnswer::decode(self.limits.wire(), &mut reader) {
                Ok(answer) if reader.is_empty() => match answer {
                    w::ApprovalAskAnswer::Asked(value) => {
                        let fields = value;
                        match ApprovalCode::parse(&fields.code) {
                            Some(code) if code.letters() == &fields.code && fields.secret.len() == 32 => {
                                Approval::Asked {
                                    secret: fields.secret,
                                    code,
                                    expires: Wall::from_nanos(fields.expires),
                                }
                            }
                            Some(_) | None => kind.lost(),
                        }
                    }
                    w::ApprovalAskAnswer::Refused => Approval::NotAsked(NotAsked::Refused),
                    w::ApprovalAskAnswer::NotReady => Approval::NotAsked(NotAsked::NotReady),
                    w::ApprovalAskAnswer::Busy => Approval::NotAsked(NotAsked::Busy),
                },
                Ok(_) | Err(_) => kind.lost(),
            },
            ApprovalKind::Read { code } => {
                let state = match w::ApprovalState::decode(self.limits.wire(), &mut reader) {
                    Ok(answer) if reader.is_empty() => match answer {
                        w::ApprovalState::Waiting(value) => ApprovalState::Waiting {
                            asked: Wall::from_nanos(value.asked),
                            kind: value.kind.clone(),
                            source: address(value.source.as_ref()),
                            reader: address(value.reader.as_ref()),
                        },
                        w::ApprovalState::Approved => ApprovalState::Approved,
                        w::ApprovalState::Taken => ApprovalState::Taken,
                        w::ApprovalState::Refused => ApprovalState::Refused,
                        w::ApprovalState::Expired => ApprovalState::Expired,
                        w::ApprovalState::Unknown => ApprovalState::Unknown,
                        w::ApprovalState::NotABrowser => ApprovalState::NotABrowser,
                        w::ApprovalState::SignedOut => ApprovalState::SignedOut,
                        w::ApprovalState::NotReady => ApprovalState::NotReady,
                        w::ApprovalState::Busy => ApprovalState::Busy,
                    },
                    Ok(_) | Err(_) => ApprovalState::Lost,
                };
                Approval::Read { code, state }
            }
            ApprovalKind::Decide { code } => {
                let decided = match w::ApprovalDecided::decode(self.limits.wire(), &mut reader) {
                    Ok(answer) if reader.is_empty() => match answer {
                        w::ApprovalDecided::Approved(value) => {
                            let loopback = match value.loopback {
                                Some(value) => Some(crate::generated::client::approval_loopback_to_domain(value)),
                                None => None,
                            };
                            Decided::Approved { loopback }
                        }
                        w::ApprovalDecided::Refused => Decided::Refused,
                        w::ApprovalDecided::Used => Decided::Used,
                        w::ApprovalDecided::Expired => Decided::Expired,
                        w::ApprovalDecided::Unknown => Decided::Unknown,
                        w::ApprovalDecided::NotABrowser => Decided::NotABrowser,
                        w::ApprovalDecided::SignedOut => Decided::SignedOut,
                        w::ApprovalDecided::NotReady => Decided::NotReady,
                        w::ApprovalDecided::Busy => Decided::Busy,
                    },
                    Ok(_) | Err(_) => Decided::Lost,
                };
                Approval::Decided { code, decided }
            }
        };
        match &answer {
            Approval::Read { state: ApprovalState::SignedOut, .. }
            | Approval::Decided { decided: Decided::SignedOut, .. } => {
                self.end_sign_in(out);
                out.push(Request::Signing { signing: Signing::SignedOut });
            }
            Approval::Asked { .. } | Approval::NotAsked(_) | Approval::Read { .. } | Approval::Decided { .. } => {}
        }
        out.push(Request::Approval(answer));
    }
}

fn address(octets: Option<&[u8; 16]>) -> Option<core::net::IpAddr> {
    match octets {
        Some(octets) => Some(core::net::Ipv6Addr::from(*octets).to_canonical()),
        None => None,
    }
}
