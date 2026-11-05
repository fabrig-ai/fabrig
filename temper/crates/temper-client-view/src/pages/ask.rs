//! Shared card words and parts (temper/docs/design/client/README.md, 3.5).
use crate::{
    frame::Bindings,
    look,
    words::{TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract::{InboxGroup, ReadValue};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain, PlanFilter,
    pages::{Act, inbox::Card, needs},
};

pub(crate) enum Kind {
    Question,
    Proposal,
    Held,
    Funds { spent: u64, limit: u64 },
    PersonTask,
}
pub(crate) struct Asked<'a> {
    pub card: &'a Card,
    pub kind: Kind,
    pub title: Box<[u8]>,
    pub parsed: Option<temper_client_domain::question::Parsed>,
    pub short: Box<[u8]>,
}
pub(crate) fn read<'a>(card: &'a Card, w: &Words) -> Asked<'a> {
    let mut kind = match card.group {
        InboxGroup::Questions => Kind::Question,
        InboxGroup::Proposals => Kind::Proposal,
        InboxGroup::Escalations => Kind::Held,
        InboxGroup::PersonTasks | InboxGroup::Results | InboxGroup::Replies => Kind::PersonTask,
    };
    if let Some(ReadValue::Escalation(held)) = &card.page
        && let Some(hold) = &held.task.hold
        && hold.reason == jig_client::contract::HoldCategory::Budget
    {
        let numbers = held.task.budget.numbers;
        kind = Kind::Funds { spent: numbers.spent, limit: numbers.budget };
    }
    let parsed = if let Some(ReadValue::Question(q)) = &card.page {
        Some(temper_client_domain::question::parse(q.words.bytes(), q.words.cut))
    } else {
        None
    };
    let title = title_for(card, w, Word::Task);
    let short = match &parsed {
        Some(parsed) => parsed.question.clone(),
        None => match &card.page {
            Some(ReadValue::Proposal(proposal)) => {
                if proposal.reason.bytes().is_empty() {
                    match &proposal.action {
                        jig_client::contract::ProposalAction::Batch(tasks) => match tasks.get(0) {
                            Some(task) => {
                                let words = task.spec.specification.words.bytes();
                                let end = skein_lib::bytes::find(words, b"\n").unwrap_or(words.len());
                                words.get(..end).expect("batch heading").into()
                            },
                            None => title.clone(),
                        },
                        jig_client::contract::ProposalAction::Settings(_)
                        | jig_client::contract::ProposalAction::Effect { .. }
                        | jig_client::contract::ProposalAction::ProcedureEffect { .. }
                        | jig_client::contract::ProposalAction::Amend { .. }
                        | jig_client::contract::ProposalAction::Widen { .. }
                        | jig_client::contract::ProposalAction::Release { .. } => title.clone(),
                    }
                } else {
                    first_sentence(proposal.reason.bytes())
                }
            },
            Some(page) => {
                let words = super::inbox::card_words(page);
                if words.is_empty() { title.clone() } else { words }
            }
            None => title.clone(),
        },
    };
    let short = match kind {
        Kind::Funds { spent, limit } => w.say(Word::NeedsFunds, &[Hole::Number(spent), Hole::Number(limit)]),
        Kind::Held | Kind::PersonTask => title.clone(),
        Kind::Question | Kind::Proposal => short,
    };
    Asked { card, kind, title, parsed, short }
}
pub(crate) fn need_title(card: &Card, w: &Words) -> Box<[u8]> {
    title_for(card, w, Word::TaskWho)
}
fn title_for(card: &Card, w: &Words, numbered: Word) -> Box<[u8]> {
    if let Some(page) = &card.page
        && let Some(title) = super::inbox::task_title(page)
        && !title.is_empty()
    {
        return title.into();
    }
    match super::inbox::source_task(card.object) {
        Some(task) => w.say(numbered, &[Hole::Number(task)]),
        None => w.say(need_kind(card.group), &[]),
    }
}
pub(crate) fn need_kind(group: InboxGroup) -> Word {
    match group {
        InboxGroup::Proposals => Word::HomeProposal,
        InboxGroup::Questions => Word::HomeQuestion,
        InboxGroup::Escalations => Word::HomeHeld,
        InboxGroup::PersonTasks => Word::HomePersonTask,
        InboxGroup::Results => Word::InboxUpdates,
        InboxGroup::Replies => Word::Reply,
    }
}
pub(crate) fn need_when(card: &Card, w: &Words, times: &mut Times) -> Box<[u8]> {
    when(card, need_kind(card.group), w, times)
}
fn when(card: &Card, kind: Word, w: &Words, times: &mut Times) -> Box<[u8]> {
    let kind = w.say(kind, &[]);
    let since = match &card.page {
        Some(page) => super::inbox::waiting_since(page),
        None => None,
    };
    match since {
        Some(since) => crate::accounts::words::join(&[
            &kind,
            &w.say(Word::ToolSeparator, &[]),
            &times.say(w, TimeForm::Span, since),
        ]),
        None => kind,
    }
}
pub(crate) fn meta(
    b: &mut Builder,
    asked: &Asked<'_>,
    project: Option<&[u8]>,
    later: bool,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    let sep = w.say(Word::ToolSeparator, &[]);
    let when = match asked.kind {
        Kind::Funds { .. } => when(asked.card, Word::NeedsKindFunds, w, times),
        Kind::Question | Kind::Proposal | Kind::Held | Kind::PersonTask => need_when(asked.card, w, times),
    };
    let later = if later { w.say(Word::NeedsLaterMeta, &[]) } else { Box::default() };
    let prefix = match project {
        Some(project) => crate::accounts::words::join(&[project, &sep]),
        None => Box::default(),
    };
    let suffix = crate::accounts::words::join(&[&sep, &when, &later]);
    b.open(Role::Paragraph);
    b.style(look::ASK_META);
    b.name(&crate::accounts::words::join(&[&prefix, &asked.title, &suffix]));
    b.text(Run::Plain, &prefix);
    if let Some(task) = super::inbox::source_task(asked.card.object) {
        link(b, bindings, &asked.title, Address::Task { task, filter: PlanFilter::All });
    } else {
        b.text(Run::Plain, &asked.title);
    }
    b.text(Run::Plain, &suffix);
    b.close();
}
pub(crate) fn answered_label(w: &Words, label: &needs::Label) -> Box<[u8]> {
    match label {
        needs::Label::Choice(words) => words.clone(),
        needs::Label::Reply(words) => w.say(Word::AnsweredReply, &[Hole::Name(words.clone())]),
    }
}
pub(crate) fn answered_lines(
    b: &mut Builder,
    root: &Domain,
    lines: &[&needs::Answered],
    w: &Words,
    bindings: &mut Bindings,
) {
    for line in lines {
        b.open(Role::Slot);
        b.style(look::ANSWERED);
        let title = if line.title.is_empty() {
            match super::inbox::source_task(line.object) {
                Some(task) => w.say(Word::Task, &[Hole::Number(task)]),
                None => w.say(Word::HomeQuestion, &[]),
            }
        } else {
            line.title.clone()
        };
        b.open(Role::Text);
        if let Some(task) = super::inbox::source_task(line.object) {
            link(b, bindings, &title, Address::Task { task, filter: PlanFilter::All });
        } else {
            b.text(Run::Plain, &title);
        }
        b.text(Run::Plain, b":");
        b.close();
        text(b, Role::Text, &answered_label(w, &line.label), look::ANSWERED_LABEL);
        if root.pages().needs().held(line.object) {
            button(
                b,
                bindings,
                &w.say(Word::ToastUndo, &[]),
                Act::Needs(needs::Act::Undo { object: line.object }),
                look::ANSWERED_UNDO,
            );
        }
        b.close();
    }
}
pub(crate) fn reply_form(b: &mut Builder, root: &Domain, asked: &Asked<'_>, w: &Words, bindings: &mut Bindings) {
    let object = asked.card.object;
    if !root.pages().needs().replying(object) {
        return;
    }
    b.open(Role::Slot);
    b.style(look::ASK_REPLY);
    b.open(Role::TextArea);
    b.name(&w.say(Word::NeedsYourReply, &[]));
    b.placeholder(&w.say(Word::NeedsPlaceholder, &[Hole::Name(asked.title.clone())]));
    b.bind(bindings.insert(Act::CardWords { object, words: Box::default() }));
    b.value(
        match &asked.card.words {
            Some(words) => words.bytes(),
            None => b"",
        },
        temper_client_domain::pages::shown(root.pages()),
    );
    b.focus(0);
    b.close();
    b.open(Role::Slot);
    b.style(look::ASK_REPLY_ACTS);
    button(
        b,
        bindings,
        &w.say(Word::NeedsSendReply, &[]),
        Act::Needs(needs::Act::SendReply { object }),
        look::RECOMMENDED,
    );
    button(b, bindings, &w.say(Word::Cancel, &[]), Act::Needs(needs::Act::CancelReply), look::ASK_QUIET);
    b.close();
    b.close();
}
pub(crate) fn text(b: &mut Builder, role: Role, words: &[u8], style: skein_ui::Style) {
    b.open(role);
    b.style(style);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
pub(crate) fn button(b: &mut Builder, bindings: &mut Bindings, words: &[u8], act: Act, style: skein_ui::Style) {
    b.open(Role::Button);
    b.style(style);
    b.name(words);
    b.bind(bindings.insert(act));
    b.close();
}
pub(crate) fn link(b: &mut Builder, bindings: &mut Bindings, words: &[u8], address: Address) {
    crate::link(b, words, bindings.insert(Act::Go { address }));
    b.text(Run::Plain, words);
    b.close();
}

fn first_sentence(words: &[u8]) -> Box<[u8]> {
    let sentence = match skein_lib::bytes::find(words, b". ") {
        Some(at) => at.checked_add(1).expect("sentence end fits"),
        None => words.len(),
    };
    let line = skein_lib::bytes::find(words, b"\n").unwrap_or(words.len());
    words.get(..sentence.min(line)).expect("sentence in words").into()
}
