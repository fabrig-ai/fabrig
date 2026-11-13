//! The chat's waiting question (temper/docs/design/client/tui/pages/chat.md, 2).
use crate::words::Words;
use alloc::boxed::Box;
use jig_client::contract as c;
use temper_client_domain::{Domain, pages::inbox::Card};
/// A parsed choice (temper/docs/design/client/pages/README.md, A goal's question).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QuestionChoice {
    pub words: Box<[u8]>,
    pub recommended: bool,
}
/// The first answerable question (temper/docs/design/client/tui/pages/chat.md, 2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Question {
    pub object: c::ObjectKey,
    pub head: Box<[u8]>,
    pub choices: Box<[QuestionChoice]>,
    pub primary: Option<usize>,
}
/// The browser terminal's question, whatever the look (temper/docs/design/client/tui/pages/chat.md, 2).
#[must_use]
pub fn question(root: &Domain, w: &Words) -> Option<Question> {
    if root.pages().confirmation().is_some() {
        return None;
    }
    let inbox = root.pages().inbox()?;
    for card in &root.pages().needs().open(inbox) {
        if let Some(question) = for_card(card, w) {
            return Some(question);
        }
    }
    None
}

pub(super) fn for_card(card: &Card, w: &Words) -> Option<Question> {
    if card.group != c::InboxGroup::Questions || card.key.is_some() || card.project.is_none() {
        return None;
    }
    let page = card.page.as_ref()?;
    let Some(c::Admission::Allowed | c::Admission::Bounded { .. }) =
        jig_client::facts::offer(temper_client_domain::acts::offers(page), c::ActKind::Answer)
    else {
        return None;
    };
    let asked = crate::common::ask::read(card, w);
    let parsed = asked.parsed?;
    let primary = parsed.primary();
    let mut choices = skein_lib::List::with_capacity(u32::try_from(parsed.choices.len()).expect("parsed choices fit"));
    for choice in &parsed.choices {
        choices
            .push(QuestionChoice { words: choice.words.clone(), recommended: choice.recommended })
            .expect("parsed choices fit");
    }
    Some(Question { object: card.object, head: parsed.question, choices: choices.into_boxed(), primary })
}
