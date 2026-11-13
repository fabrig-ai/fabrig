//! Shared inbox items (temper/docs/design/client/view-crates.md, 3.3).
use crate::look;
use alloc::boxed::Box;
use jig_client::contract::{ObjectKey, Proposer, ReadValue};

pub(crate) fn task_title(page: &ReadValue) -> Option<&[u8]> {
    if let ReadValue::Question(question) = page {
        return Some(question.task.title.bytes());
    }

    if let ReadValue::Escalation(held) = page {
        return Some(held.task.chip.title.bytes());
    }

    if let ReadValue::PersonTask(task) = page {
        return Some(task.task.title.bytes());
    }
    if let ReadValue::Task(task) = page {
        return Some(task.chip.title.bytes());
    }
    None
}

pub(crate) fn source_task(object: ObjectKey) -> Option<u64> {
    match object {
        ObjectKey::Task(task)
        | ObjectKey::Escalation(task)
        | ObjectKey::PersonTask(task)
        | ObjectKey::Result(task)
        | ObjectKey::Question { task, .. } => Some(task),
        ObjectKey::Proposal(proposal) => match proposal.proposer {
            Proposer::Task(task) => Some(task),
            Proposer::Party(_) => None,
        },
        ObjectKey::Note { .. } => None,
    }
}

/// The card's prose without its controls or decision facts (client/pages/README.md).
pub(crate) fn card_words(page: &ReadValue) -> Box<[u8]> {
    let (title, spec) = match page {
        ReadValue::Question(question) => return question.words.bytes().into(),
        ReadValue::Escalation(held) => (held.task.chip.title.bytes(), &held.task.spec),
        ReadValue::PersonTask(task) => (task.task.title.bytes(), &task.spec),
        ReadValue::Task(task) => (task.chip.title.bytes(), &task.spec),
        ReadValue::Ended(_)
        | ReadValue::Result(_)
        | ReadValue::Proposal(_)
        | ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::TaskText(_)
        | ReadValue::Revisions(_)
        | ReadValue::Tasks(_)
        | ReadValue::Project(_)
        | ReadValue::Me(_)
        | ReadValue::EndedPage(_)
        | ReadValue::History(_)
        | ReadValue::Turn(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Spend(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_) => return Box::default(),
    };
    let words = spec.specification.words.bytes();
    crate::accounts::words::join(&[title, if title.is_empty() || words.is_empty() { b"" } else { b" " }, words])
}

pub(crate) fn card_key(object: ObjectKey) -> skein_ui::Key {
    let (kind, a, b) = match object {
        ObjectKey::Task(task) => (0, task, 0),
        ObjectKey::Proposal(key) => match key.proposer {
            Proposer::Task(task) => (1, task, key.number),
            Proposer::Party(party) => (29, party, key.number),
        },
        ObjectKey::Escalation(task) => (2, task, 0),
        ObjectKey::Question { task, message } => (3, task, message),
        ObjectKey::PersonTask(task) => (4, task, 0),
        ObjectKey::Result(task) => (5, task, 0),
        ObjectKey::Note { project, note } => (6, u64::from(project), note),
    };
    skein_ui::Key { owner: look::owner(), kind, a, b }
}

/// The card's wait; task-page decision facts carry no wait.
pub(crate) fn waiting_since(page: &ReadValue) -> Option<skein_lib::Wall> {
    match page {
        ReadValue::Proposal(proposal) => Some(proposal.waiting.since),

        ReadValue::Question(question) => Some(question.since),

        ReadValue::Escalation(escalation) => match escalation.task.hold.as_ref() {
            Some(hold) => hold.hold_wait_since,
            None => None,
        },

        ReadValue::PersonTask(task) => task.since,
        ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::TaskText(_)
        | ReadValue::Revisions(_)
        | ReadValue::Tasks(_)
        | ReadValue::Project(_)
        | ReadValue::Ended(_)
        | ReadValue::Me(_)
        | ReadValue::Turn(_)
        | ReadValue::Task(_)
        | ReadValue::Result(_)
        | ReadValue::EndedPage(_)
        | ReadValue::History(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Spend(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_) => None,
    }
}
