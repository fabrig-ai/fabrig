//! The short words of task admission, without the problem's associated rows.
use crate::{AccountWhy, LlmFault, QuestionAnswerer, Refusal};
use alloc::boxed::Box;
use skein_lib::{Decimal, bytes::concat};

impl Refusal {
    /// Render the refusal itself, when the caller does not carry a whole `Problem`.
    #[must_use]
    pub fn words(self) -> Box<[u8]> {
        let reason: &[u8] = match self {
            Self::Concluded { .. } => b"concluded",
            Self::NotStanding => b"the task does not stand",
            Self::TalkOpen { .. } => b"another party has an open talk",
            Self::NoTalk => b"no talk of yours to leave",
            Self::Placed { .. } => b"held by a placed hold",
            Self::HoldOwner { .. } => b"only the placed hold's owner may release it",
            Self::HoldRevision { .. } => b"placed hold revision changed",
            Self::Unproposable => b"no proposal could be accepted",
            Self::PersonRevision { .. } => b"person revision",
            Self::ResourceUnavailable => b"resource unavailable",
            Self::Unreported => b"unreported",
            Self::NotReady => b"not ready",
            Self::Unknown => b"unknown",
            Self::Busy => b"busy",
            Self::Duplicate => b"duplicate",
            Self::Empty => b"empty",
            Self::Batch => b"batch",
            Self::Live => b"live",
            Self::Project => b"project",
            Self::Tree => b"tree",
            Self::Depth => b"depth",
            Self::Delegates => b"delegates",
            Self::Reference => b"reference",
            Self::Subscription => b"subscription",
            Self::Dependencies => b"dependencies",
            Self::Circle { .. } => b"these tasks would wait for one another",
            Self::DependencyEnded { .. } => b"a dependency has ended",
            Self::QuestionAnswered { question, by } => return concat(&[b"Refused: ", &by.answered_words(question)]),
            Self::PersonAnswers { .. } => b"the person answers this question",
            Self::Kept | Self::KeptQuestion { .. } => b"kept",
            Self::Executor => b"executor",
            Self::Spec => b"spec",
            Self::Contract => b"contract",
            Self::AuthorityShape => b"authority shape",
            Self::Inputs => b"inputs",
            Self::State => b"state",
            Self::Goals => b"goals",
            Self::Grace => b"grace",
            Self::Attempt => b"attempt",
            Self::LiveDelegates => b"live delegates",
            Self::Restore => b"restore",
            Self::Read => b"read",
            Self::Turn => b"turn",
            Self::Funding => b"funding",
            Self::HoldKind => b"hold kind",
            Self::HoldTaken => b"hold taken",
            Self::Holds => b"holds",
            Self::Account { account, why } => {
                let why: &[u8] = match why {
                    AccountWhy::Unknown => b"account not found",
                    AccountWhy::Provider => b"account belongs to another provider",
                    AccountWhy::Uses => b"account does not serve the payer",
                    AccountWhy::Limited => b"kept account is limited",
                };
                return concat(&[b"Refused account ", Decimal::of(u64::from(account)).as_bytes(), b": ", why]);
            }
            Self::Llm { why, .. } => match why {
                LlmFault::Unconfigured => b"model not configured",
                LlmFault::Effort => b"effort not declared",
                LlmFault::Charter => b"model outside the charter",
                LlmFault::Authority => b"model outside authority",
                LlmFault::Procedure => b"no model runs this task",
                LlmFault::Repeated => b"model repeated",
                LlmFault::Speed => b"speed is not one the model declares",
            },
        };
        concat(&[b"Refused: ", reason])
    }
}

impl QuestionAnswerer {
    /// The first answer's author, in tool answer words.
    #[must_use]
    pub fn answered_words(self, question: u64) -> Box<[u8]> {
        let (kind, number) = match self {
            Self::Task(task) => (b"task ".as_slice(), task),
            Self::Person(person) => (b"person ".as_slice(), person),
        };
        concat(&[
            b"question ",
            Decimal::of(question).as_bytes(),
            b" was answered by ",
            kind,
            Decimal::of(number).as_bytes(),
        ])
    }
}
