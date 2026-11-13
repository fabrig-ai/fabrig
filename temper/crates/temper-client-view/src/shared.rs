//! Words and data shared by the browser and TUI (temper/docs/design/client/tui/pages/README.md, Its words).
use alloc::boxed::Box;
mod cards;
mod findings;
mod people;
pub use findings::finding_refusal;
pub use people::person_line;
mod lists;
mod needs;
mod question;
mod reach;
mod refusal;
pub use reach::{allowed, object, offer, rule, why};
mod words;
pub use cards::{DelegationLine, delegation_line, proposal_line};
pub use lists::{Choice, Confirmation, chats, confirmation, efforts, list_head, models};
pub use needs::{NeedAct, NeedMark, NeedPrompt, need_prompts};
pub use question::{Question, QuestionChoice, question};
pub(crate) use refusal::{Feedback, composer_refusals, live_tasks_words, outcome_feedback};
pub use refusal::{outcome_refusal, refusal, start_goal_refusal};
pub use words::{
    NeedKind, NeedLine, Run, RunLine, TerminalWord, all_home_headline, chat_state, closed_line, description, heading,
    home_headline, joined_brief, need_line, note, offered, running, say,
};

/// Ready's first usable ACCOUNT and whether its source is a program
/// (temper/docs/design/client/tui/box.md, Model's detail).
#[must_use]
pub fn model_account(root: &temper_client_domain::Domain, w: &crate::words::Words) -> Option<(Box<[u8]>, bool)> {
    let (row, source) = crate::pages::accounts::usable_account(root)?;
    Some((
        crate::pages::accounts::account_words(root, w, row, source, false),
        source.kind == jig_client::contract::AccountKind::Program,
    ))
}

/// Whether the open chat's run line is under way (temper/docs/design/client/pages/README.md).
#[must_use]
pub fn working(root: &temper_client_domain::Domain, w: &crate::words::Words) -> bool {
    let Some(chat) = root.pages().chat() else { return false };
    crate::pages::chats::working(root, w, &chat.thread)
}
