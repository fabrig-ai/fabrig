//! Need marks and prompts (temper/docs/design/client/tui/pages/launchpad.md, 2, 5 and 6).
use super::{NeedLine, Question, need_line, question::for_card};
use crate::{
    common::{approval, ask, inbox},
    forge_sign_in::Forges,
    words::Words,
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::{List, Wall};
use temper_client_domain::{Domain, forge::approvals::Yours, pages::needs::Need};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NeedMark {
    Question,
    Money,
    SignOff,
    Other,
}

#[derive(Debug)]
pub struct NeedPrompt {
    /// A card's `object`; a sign-off's `c::ObjectKey::Task(row.task)`.
    pub object: c::ObjectKey,
    pub project: Option<u32>,
    pub mark: NeedMark,
    /// As `need_line` says it.
    pub line: NeedLine,
    pub act: NeedAct,
    pub since: Option<Wall>,
    pub why: Option<Box<[u8]>>,
}

#[derive(Debug)]
pub enum NeedAct {
    /// A question the person may answer, as `shared::question` builds one.
    Answer(Question),
    /// The browser's review or sign-off words and the forge's review page.
    Review { words: Box<[u8]>, address: Box<[u8]> },
    /// Decided on the web.
    Web,
}

#[must_use]
pub fn need_prompts(root: &Domain, w: &Words) -> Box<[NeedPrompt]> {
    let Some(inbox) = root.pages().inbox() else { return Box::default() };
    let needs = root.pages().needs().needs(inbox, root.forge().approvals());
    let mut prompts = List::with_capacity(u32::try_from(needs.len()).expect("needs fit"));
    let forges = Forges::new();
    for need in &needs {
        let (object, project, mark, act) = match need {
            Need::Card(card) => {
                let mark = match ask::read(card, w).kind {
                    ask::Kind::Question => NeedMark::Question,
                    ask::Kind::Funds { .. } => NeedMark::Money,
                    ask::Kind::Proposal | ask::Kind::Held | ask::Kind::PersonTask => NeedMark::Other,
                };
                let act = match for_card(card, w) {
                    Some(question) => NeedAct::Answer(question),
                    None => NeedAct::Web,
                };
                (card.object, card.project, mark, act)
            }
            Need::Approval(row) => {
                let act = if row.yours == Yours::Counts {
                    match approval::address(row, &forges) {
                        Some(address) => NeedAct::Review { words: approval::review_words(row, w), address },
                        None => NeedAct::Web,
                    }
                } else {
                    NeedAct::Web
                };
                (c::ObjectKey::Task(row.task), Some(row.project), NeedMark::SignOff, act)
            }
        };
        let since = match need {
            Need::Card(card) => match &card.page {
                Some(page) => inbox::waiting_since(page),
                None => None,
            },
            Need::Approval(row) => Some(row.since),
        };
        let why = super::why(root, w, *need);
        prompts
            .push(NeedPrompt { object, project, mark, line: need_line(need, w), act, since, why })
            .expect("needs fit");
    }
    prompts.into_boxed()
}
