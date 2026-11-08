//! Words and data shared by the browser and TUI (temper/docs/design/client/README.md, 6.14, Its words).
mod cards;
mod lists;
mod question;
mod refusal;
mod words;
pub use cards::{DelegationLine, delegation_line, proposal_line};
pub use lists::{Choice, Confirmation, chats, confirmation, efforts, list_head, models};
pub use question::{Question, QuestionChoice, question};
pub use refusal::refusal;
pub(crate) use refusal::{Feedback, composer_refusals, outcome_feedback};
pub use words::{
    NeedKind, NeedLine, Run, RunLine, TerminalWord, chat_state, closed_line, description, heading, home_headline,
    need_line, note, offered, running, say,
};
