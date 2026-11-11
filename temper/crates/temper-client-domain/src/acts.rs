//! Inbox acts retain the document revision that the person acted on (client/README.md, 3.6).
use crate::{
    Limits,
    owner::Owner,
    pages::{Act, Pages, Request, inbox::Card},
};
use alloc::boxed::Box;
use jig_client::{
    Client, Key, Told,
    contract::{self, ActKind, Admission, Ask, DecideOn, Decision, ObjectKey, Offered, ReadValue, Text},
};
use skein_lib::{Env, Queue};
/// The page's one open confirmation. Its object and revision come from the binding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Confirm {
    pub object: ObjectKey,
    pub revision: Option<contract::Revision>,
    pub act: ActKind,
    pub again: Option<bool>,
    /// The ask this sheet sent, until it is answered (client/README.md, 3.6).
    pub sent: Option<Key>,
    /// Its last answer other than done, said above its acts.
    pub failed: Option<jig_client::Outcome>,
}
#[must_use]
pub fn offers(page: &ReadValue) -> &[Offered] {
    match page {
        ReadValue::Task(v) => v.acts.as_slice(),

        ReadValue::Proposal(v) => v.acts.as_slice(),
        ReadValue::Question(v) => v.acts.as_slice(),

        ReadValue::Escalation(v) => v.acts.as_slice(),

        ReadValue::PersonTask(v) => v.acts.as_slice(),
        ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::TaskText(_)
        | ReadValue::Revisions(_)
        | ReadValue::Tasks(_)
        | ReadValue::Project(_)
        | ReadValue::Ended(_)
        | ReadValue::Me(_)
        | ReadValue::Result(_)
        | ReadValue::EndedPage(_)
        | ReadValue::History(_)
        | ReadValue::Turn(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Spend(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_) => &[],
    }
}
#[must_use]
pub fn revision(page: &ReadValue) -> Option<contract::Revision> {
    match page {
        ReadValue::Task(v) => match &v.hold {
            Some(hold) => Some(hold.revision()),
            None => None,
        },

        ReadValue::Escalation(v) => match &v.task.hold {
            Some(hold) => Some(hold.revision()),
            None => None,
        },

        ReadValue::PersonTask(v) => Some(v.revision),
        ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::TaskText(_)
        | ReadValue::Revisions(_)
        | ReadValue::Tasks(_)
        | ReadValue::Project(_)
        | ReadValue::Ended(_)
        | ReadValue::Me(_)
        | ReadValue::Result(_)
        | ReadValue::EndedPage(_)
        | ReadValue::History(_)
        | ReadValue::Turn(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Spend(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_)
        | ReadValue::Proposal(_)
        | ReadValue::Question(_) => None,
    }
}
fn find_card(pages: &mut Pages, object: ObjectKey) -> Option<&mut Card> {
    let cards = pages.cards_mut()?;
    for index in 0..cards.len() {
        if cards.get(index)?.object == object {
            return cards.get_mut(index);
        }
    }
    None
}
#[expect(clippy::too_many_lines, reason = "deciding dispatch lists each act and retains the sheet through its ask")]
pub(super) fn step(pages: &mut Pages, env: &Env<Limits>, act: Act, out: &mut Queue<Request>) {
    match act {
        Act::CardWords { object, words } => {
            crate::pages::needs::open_reply(pages, object);
            if pages.needs().replying(object) {
                crate::pages::needs::write_reply(pages, object, words, env.limits.client.domain.long_text);
                return;
            }
            if let Ok(words) = Text::new(words, env.limits.client.domain.text)
                && let Some(card) = find_card(pages, object)
            {
                card.words = Some(words);
            }
        }
        Act::Decide { object, revision, act } => {
            let Some(card) = find_card(pages, object) else { return };
            if card.key.is_some() {
                return;
            }
            let Some(page) = &card.page else { return };
            if !allowed(page, act) {
                return;
            }
            let confirm = Confirm { object, revision, act, again: None, sent: None, failed: None };
            if jig_client::facts::dangerous(act) || (act == ActKind::Accept || act == ActKind::Release) {
                pages.confirm = Some(confirm);
            } else {
                let _sent = send(pages, confirm, out);
            }
        }
        Act::Again { again } => {
            if let Some(open) = &mut pages.confirm {
                open.again = Some(again);
            }
        }
        Act::Confirm => {
            if let Some(mut open) = pages.confirm.take() {
                if open.sent.is_none() {
                    open.failed = None;
                    if let Some(key) = send(pages, open.clone(), out) {
                        open.sent = Some(key);
                        pages.confirm = Some(open);
                    }
                } else {
                    pages.confirm = Some(open);
                }
            }
        }
        Act::Cancel => pages.confirm = None,
        Act::ChangeProposal { .. }
        | Act::MessageWords { .. }
        | Act::Send
        | Act::Pause
        | Act::Resume
        | Act::Stop
        | Act::Close
        | Act::StartOver
        | Act::DelegateFate { .. }
        | Act::Older
        | Act::Newer
        | Act::Latest
        | Act::AskLine { .. }
        | Act::AskWith { .. }
        | Act::ChatWords { .. }
        | Act::StartChat
        | Act::WayTab { .. }
        | Act::WayStop { .. }
        | Act::UseWay { .. }
        | Act::KeepWay { .. }
        | Act::SkipToBegin
        | Act::RetryPlayground
        | Act::Playground
        | Act::Suggest { .. }
        | Act::MoreGoals
        | Act::GoalsFilter { .. }
        | Act::ChatFilter { .. }
        | Act::ClosedChats => unreachable!("chat acts routed to chats"),
        Act::OpenStart { .. }
        | Act::OpenLanded { .. }
        | Act::StartDraft { .. }
        | Act::StartName { .. }
        | Act::StartWay { .. }
        | Act::StartAsks { .. }
        | Act::StartCancel
        | Act::StartGoal
        | Act::Switcher(_)
        | Act::Away(_)
        | Act::Terminal(_)
        | Act::Remembered(_)
        | Act::Models(_)
        | Act::StopAsking(_)
        | Act::Needs(_)
        | Act::Choice(_)
        | Act::Accounts(_)
        | Act::ConnectModel(_)
        | Act::MakeDefault
        | Act::AdoptFirst
        | Act::CopyAgain { .. }
        | Act::AddOpen { .. }
        | Act::AddChoose { .. }
        | Act::AddWrite { .. }
        | Act::AddClose
        | Act::AddSubmit
        | Act::CopyOffered
        | Act::DefaultName { .. }
        | Act::OpenProject { .. }
        | Act::Project { .. }
        | Act::SignOut
        | Act::Filter { .. }
        | Act::OpenResult { .. }
        | Act::Amend
        | Act::AmendWords { .. }
        | Act::AmendTracked
        | Act::AmendPriority { .. }
        | Act::DiscardAmendment
        | Act::SendAmendment
        | Act::AmendAgain
        | Act::AddDraft
        | Act::Mode { .. }
        | Act::ShowSection { .. }
        | Act::Go { .. } => {
            unreachable!("only deciding acts routed here")
        }
    }
}
#[expect(clippy::too_many_lines, reason = "the act dispatch retains each page variant and its shown revision")]
fn send(pages: &mut Pages, open: Confirm, out: &mut Queue<Request>) -> Option<Key> {
    let card = find_card(pages, open.object)?;
    if card.key.is_some() {
        return None;
    }
    let Some(page) = &card.page else { return None };
    // A proposal is decided under its own project, which a chat's card doesn't
    // hold (temper/docs/design/client/pages/settings.md, 2).
    let project = if let ReadValue::Proposal(proposal) = page { proposal.project } else { card.project? };
    if !allowed(page, open.act) {
        return None;
    }
    let words = card.words.clone();
    let ask = match (open.object, open.act) {
        (ObjectKey::Proposal(proposal), ActKind::Accept | ActKind::Reject) => Ask::Decide {
            project,
            on: DecideOn::Proposal {
                proposal,
                running: if open.act == ActKind::Accept
                    && let ReadValue::Proposal(proposal) = page
                    && let contract::ProposalAction::Settings(_) = proposal.action
                {
                    Some(contract::Running::Stays)
                } else {
                    None
                },
                decision: if open.act == ActKind::Accept {
                    Decision::Accepted
                } else {
                    let Some(reason) = words else {
                        pages.confirm = Some(open);
                        return None;
                    };
                    Decision::Rejected { reason }
                },
            },
        },
        (ObjectKey::Escalation(task) | ObjectKey::Task(task), ActKind::Release) => {
            if jig_client::facts::effect(offers(page), open.act)
                == Some(&contract::Effect::Release(contract::ReleaseEffect::Uncertain))
                && open.again.is_none()
            {
                return None;
            }
            let revision = open.revision?;
            Ask::Release { project, task, revision, again: open.again }
        }
        (ObjectKey::Escalation(task), ActKind::LeaveHeld) => {
            let (Some(revision), Some(reason)) = (open.revision, words) else { return None };
            Ask::Decide {
                project,
                on: DecideOn::Escalation { task, revision, decision: Decision::LeftHeld { reason } },
            }
        }
        (ObjectKey::Question { task, message }, ActKind::Answer) => {
            let words = words?;
            Ask::Answer { project, on: contract::AnswerOn::Question { task, message, words } }
        }
        (ObjectKey::PersonTask(task), ActKind::Answer) => {
            match page {
                ReadValue::PersonTask(person) => {
                    if !crate::question::in_words(person) {
                        return None;
                    }
                }
                ReadValue::Task(_)
                | ReadValue::Proposal(_)
                | ReadValue::Question(_)
                | ReadValue::Escalation(_)
                | ReadValue::Settings(_)
                | ReadValue::Setting(_)
                | ReadValue::Revision(_)
                | ReadValue::TaskText(_)
                | ReadValue::Revisions(_)
                | ReadValue::Tasks(_)
                | ReadValue::Project(_)
                | ReadValue::Ended(_)
                | ReadValue::Me(_)
                | ReadValue::Result(_)
                | ReadValue::EndedPage(_)
                | ReadValue::History(_)
                | ReadValue::Turn(_)
                | ReadValue::Activity(_)
                | ReadValue::Inbox(_)
                | ReadValue::Spend(_)
                | ReadValue::Notes(_)
                | ReadValue::Deployment(_) => return None,
            }
            Ask::Answer {
                project,
                on: contract::AnswerOn::Person {
                    task,
                    revision: open.revision?,
                    answer: contract::PersonAnswer::Words(words?),
                },
            }
        }
        (ObjectKey::PersonTask(task), ActKind::Take) => Ask::Take { project, task },
        (ObjectKey::PersonTask(task), ActKind::HandBack) => Ask::HandBack { project, task },
        _ => return None,
    };
    pages.serial = pages.serial.checked_add(1).expect("pages keys fit");
    let key = Key { asker: Owner::Pages.asker(), number: pages.serial };
    let card = find_card(pages, open.object).expect("card retained");
    card.key = Some(key);
    card.outcome = None;
    out.push(Request::Client(jig_client::Event::Send {
        key,
        name: None,
        request: jig_client::Ask::Jig(Box::new(ask)),
    }));
    Some(key)
}
pub(super) fn told(pages: &mut Pages, told: &Told) -> bool {
    if let Told::Answered { key, outcome } = told
        && let Some(cards) = pages.cards_mut()
    {
        for index in 0..cards.len() {
            let card = cards.get_mut(index).expect("bounded index");
            if card.key == Some(*key) {
                if !super::pages::seen_answered(outcome) {
                    card.key = None;
                }
                card.outcome = Some(outcome.clone());
                return true;
            }
        }
    }
    match told {
        Told::Pending { .. } => true,
        Told::Link { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Changed { .. }
        | Told::Text { .. }
        | Told::Ended { .. }
        | Told::Page { .. } => false,
    }
}
#[must_use]
pub fn pending(card: &Card, client: &Client) -> Option<jig_client::Progress> {
    Some(client.pending(card.key?)?.progress)
}

pub(crate) fn allowed(page: &ReadValue, act: ActKind) -> bool {
    match jig_client::facts::offer(offers(page), act) {
        Some(Admission::Allowed | Admission::Bounded { .. }) => true,
        Some(Admission::Refused(_) | Admission::Proposed { .. }) | None => false,
    }
}
