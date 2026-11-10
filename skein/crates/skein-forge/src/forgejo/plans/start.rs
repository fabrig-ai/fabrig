//! Start reads, without scheduling or identity decisions (forge/README.md, section 4.1).
use super::{Body, Heard as ExchangeHeard, plan::Exchange};
use crate::forgejo::{document::Document, request::Request};
use skein_lib::Duration;

/// One owner-selected start exchange.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Read {
    User,
    ApiSettings,
}

/// Send is answered with exchange events; Done is terminal for this read.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Step {
    Send(Exchange),
    Done(Heard),
}

/// What one start read heard.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Heard {
    /// `/user`'s `id`; the owner checks identity.
    User { id: u64 },
    /// A positive `/settings/api` page cap.
    ApiSettings { max_response_items: u32 },
    /// A non-200 status with its retry wait.
    Status { status: u16, retry: Option<Duration> },
    /// No usable answer.
    Unanswered,
}

/// One read, ending at most once; its owner runs `User` before `ApiSettings`.
#[derive(Debug)]
pub struct Plan {
    state: State,
}

#[derive(Debug)]
enum State {
    Asked(Read),
    Ended,
}

impl Plan {
    #[must_use]
    pub fn new(read: Read) -> Self {
        Self { state: State::Asked(read) }
    }

    pub fn begin(&mut self) -> Step {
        let request = match self.state {
            State::Asked(read) => match read {
                Read::User => Request::User,
                Read::ApiSettings => Request::ApiSettings,
            },
            State::Ended => unreachable!("the owner begins a start plan once"),
        };
        Step::Send(Exchange { request, range: None })
    }

    pub fn heard(&mut self, heard: ExchangeHeard) -> Option<Step> {
        let answer;
        self.state = match core::mem::replace(&mut self.state, State::Ended) {
            State::Asked(read) => {
                answer = match heard {
                    ExchangeHeard::Head { status, fields } if status != 200 => {
                        Heard::Status { status, retry: fields.retry_after }
                    }
                    ExchangeHeard::Head { .. } | ExchangeHeard::Row(_) => {
                        self.state = State::Asked(read);
                        return None;
                    }
                    ExchangeHeard::Done(Body::Document(document)) => document_answer(read, document),
                    ExchangeHeard::Done(Body::Empty | Body::Raw { .. } | Body::Malformed(_))
                    | ExchangeHeard::Failed(_) => Heard::Unanswered,
                };
                State::Ended
            }
            State::Ended => return None,
        };
        Some(Step::Done(answer))
    }
}

#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
)]
fn document_answer(read: Read, document: Document) -> Heard {
    match read {
        Read::User => match document {
            Document::User(user) => Heard::User { id: user.id },
            _ => Heard::Unanswered,
        },
        Read::ApiSettings => match document {
            Document::ApiSettings(settings) if settings.max_response_items > 0 => {
                Heard::ApiSettings { max_response_items: settings.max_response_items }
            }
            _ => Heard::Unanswered,
        },
    }
}
