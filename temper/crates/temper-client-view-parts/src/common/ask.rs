//! Shared ask items (temper/docs/design/client/view-crates.md, 3.3).
use crate::words::{Word, Words};
use alloc::boxed::Box;
use jig_client::contract::{InboxGroup, ReadValue, Requester};
use skein_ui::phrases::Hole;
use temper_client_domain::pages::inbox::{Card, QuestionWords};

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
    pub source: Option<u64>,
    pub parsed: Option<temper_client_domain::question::Parsed>,
    pub short: Box<[u8]>,
    pub cut: bool,
}

pub(crate) fn read<'a>(card: &'a Card, w: &Words) -> Asked<'a> {
    read_parts(card, w, true)
}

pub(crate) fn read_parts<'a>(card: &'a Card, w: &Words, whole: bool) -> Asked<'a> {
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
    let mut cut = false;
    let parsed = match card.question_words() {
        Some(read) => {
            kind = Kind::Question;
            let (bytes, partial, reading) = if whole {
                match read {
                    QuestionWords::Whole(bytes) => (bytes, false, false),
                    QuestionWords::Reading(bytes) => (bytes, true, true),
                    QuestionWords::Cut(bytes) => (bytes, true, false),
                }
            } else {
                let words = bounded_words(card);
                (words.bytes(), words.cut, false)
            };
            cut = partial && !reading;
            let mut parsed = temper_client_domain::question::parse(bytes, partial);
            if reading {
                let ending = if parsed.context.is_empty() { &mut parsed.question } else { &mut parsed.context };
                *ending = skein_lib::bytes::concat(&[ending, "…".as_bytes()]);
            }
            Some(parsed)
        }
        None => None,
    };
    let source = match person_in_words(card) {
        Some(task) => match task.requester {
            Requester::Task(task) => Some(task),
            Requester::Person(_) | Requester::Deployment { .. } => super::inbox::source_task(card.object),
        },
        None => super::inbox::source_task(card.object),
    };
    let title = title_for(card, w, Word::Task);
    let short = match &parsed {
        Some(_) => {
            let words = bounded_words(card).bytes();
            let end = skein_lib::bytes::find(words, b"\n").unwrap_or(words.len());
            words.get(..end).expect("question first line").trim_ascii_end().into()
        }
        None => match &card.page {
            Some(ReadValue::Proposal(proposal)) => {
                if proposal.reason.bytes().is_empty() {
                    match &proposal.action {
                        jig_client::contract::ProposalAction::Batch(tasks) => match tasks.get(0) {
                            Some(task) => {
                                let words = task.spec.specification.words.bytes();
                                let end = skein_lib::bytes::find(words, b"\n").unwrap_or(words.len());
                                words.get(..end).expect("batch heading").into()
                            }
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
                    first_sentence(proposal.reason.bytes(), &title)
                }
            }
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
    Asked { card, kind, title, source, parsed, short, cut }
}

fn bounded_words(card: &Card) -> &jig_client::contract::DocumentText {
    if let Some(ReadValue::Question(question)) = &card.page {
        return &question.words;
    }
    &person_in_words(card).expect("question in words").spec.specification.words
}

pub(crate) fn need_title(card: &Card, w: &Words) -> Box<[u8]> {
    title_for(card, w, Word::TaskWho)
}

fn title_for(card: &Card, w: &Words, numbered: Word) -> Box<[u8]> {
    if let Some(person) = person_in_words(card) {
        match person.requester {
            Requester::Task(task) => return w.say(numbered, &[Hole::Number(task)]),
            Requester::Person(_) | Requester::Deployment { .. } => {}
        }
    }
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

pub(crate) fn person_in_words(card: &Card) -> Option<&jig_client::contract::PersonTask> {
    match card.page.as_ref()? {
        ReadValue::PersonTask(task) => {
            if temper_client_domain::question::in_words(task) {
                Some(task)
            } else {
                None
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
        | ReadValue::Deployment(_) => None,
    }
}

fn first_sentence(words: &[u8], fallback: &[u8]) -> Box<[u8]> {
    let mut start = 0;
    for end in 0..=words.len() {
        if end == words.len() || words.get(end) == Some(&b'\n') {
            let mut line = words.get(start..end).expect("reason line bounds").trim_ascii();
            let mut marks = 0_usize;
            for byte in line {
                if *byte != b'#' {
                    break;
                }
                marks = marks.checked_add(1).expect("bounded heading marks");
            }
            if (1..=6).contains(&marks) && (line.get(marks) == Some(&b' ') || marks == line.len()) {
                line = line.get(marks..).expect("heading marks in line").trim_ascii_start();
            }
            if !line.is_empty() {
                let sentence = match skein_lib::bytes::find(line, b". ") {
                    Some(at) => at.checked_add(1).expect("sentence end fits"),
                    None => line.len(),
                };
                return line.get(..sentence).expect("sentence in line").into();
            }
            start = end.checked_add(1).expect("bounded reason line");
        }
    }
    fallback.into()
}
