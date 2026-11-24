//! Room kept in an asked task's inbox (jig/docs/design/domain/tasks/messages.md, 2).
use crate::domain::{Domain, record, task_mut};
use crate::{Limits, MessageFamily, MessageKind, Party, ProcedureMessage, Spoken, TaskRecord};
use alloc::boxed::Box;
use skein_lib::{List, Map, Set};

/// One slot and `ANSWER_NOTICE_BYTES`, kept independently of question history.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NoticeCredit {
    pub asker: u64,
    pub question: u64,
}

#[derive(Clone, Copy)]
pub(crate) enum Change {
    Keep(NoticeCredit),
    Spend(NoticeCredit),
}

pub(crate) fn update(row: &mut TaskRecord, limits: &Limits, change: Change) {
    let (added, spent) = match change {
        Change::Keep(credit) => (Some(credit), None),
        Change::Spend(credit) => (None, Some(credit)),
    };
    let mut kept = List::with_capacity(limits.inbox_messages);
    for credit in &row.notice_credits {
        if Some(*credit) != spent {
            kept.push(*credit).expect("bounded notice credits");
        }
    }
    if let Some(credit) = added {
        kept.push(credit).expect("notice room checked");
    }
    row.notice_credits = kept.into_boxed();
}

pub(crate) fn give_back(domain: &mut Domain, limits: &Limits, target: u64, credit: NoticeCredit) {
    let Some(row) = task_mut(domain, target) else { return };
    update(&mut row.record, limits, Change::Spend(credit));
    crate::questions::changed(domain, target);
}

/// Cancel an unspent reservation on the asked task when its question ends.
pub(crate) fn forget(domain: &mut Domain, limits: &Limits, target: u64, question: u64) {
    let Some(row) = record(domain, target) else { return };
    for credit in &row.notice_credits {
        if credit.question == question {
            return give_back(domain, limits, target, *credit);
        }
    }
}

/// The asker may end without taking answers that were already delivered.
pub(crate) fn end_asker(domain: &mut Domain, limits: &Limits, target: u64, asker: u64) {
    let row = record(domain, target).expect("live asked task");
    let mut kept = List::with_capacity(limits.inbox_messages);
    for credit in &row.notice_credits {
        if credit.asker != asker {
            kept.push(*credit).expect("notice credit subset");
        }
    }
    if skein_lib::length::usize_of(kept.len()) != row.notice_credits.len() {
        task_mut(domain, target).expect("live asked task").record.notice_credits = kept.into_boxed();
        crate::questions::changed(domain, target);
    }
}

#[derive(Debug)]
pub(crate) struct Release {
    pub(crate) target: u64,
    pub(crate) credit: NoticeCredit,
}

/// Prove each claim against the exact read before projecting any part of the step.
pub(crate) fn check_step(
    domain: &Domain,
    limits: &Limits,
    task: &TaskRecord,
    read: Option<u64>,
    messages: &[ProcedureMessage],
) -> Result<Box<[Release]>, u64> {
    let mut answers = Map::with_capacity(limits.inbox_messages);
    if let Some(through) = read {
        for word in &task.inbox {
            if word.number > through || task.refused.contains(&word.number) {
                continue;
            }
            match word.kind.family() {
                MessageFamily::Spoken(Spoken::Answer { question }) => {
                    answers.insert(question, word.from).expect("answers bounded by the inbox");
                }
                MessageFamily::Spoken(Spoken::Words | Spoken::Question)
                | MessageFamily::Hint(_)
                | MessageFamily::Slot(_)
                | MessageFamily::Separate(_)
                | MessageFamily::Result
                | MessageFamily::Virtual => {}
            }
        }
    }
    let mut claimed = Set::with_capacity(limits.inbox_messages);
    for part in messages {
        let Some(question) = part.notice else { continue };
        let target = part.message.task;
        if answers.get(&question) != Some(&Party::Task(target))
            || part.message.word.kind != MessageKind::Words
            || part.message.word.words.is_empty()
            || part.message.word.words.len() > crate::ANSWER_NOTICE_BYTES
            || claimed.contains(&question)
        {
            return Err(target);
        }
        claimed.insert(question).expect("one claim per answer");
    }
    let mut releases = List::with_capacity(limits.inbox_messages);
    for (question, sender) in &answers {
        let credit = NoticeCredit { asker: task.number, question: *question };
        match *sender {
            Party::Task(target) => {
                if !claimed.contains(question) && keeps(domain, target, credit) {
                    releases.push(Release { target, credit }).expect("one release per answer");
                }
            }
            Party::Person(_) => {
                // A person can answer on behalf of the asked task, but cannot
                // receive a procedure's notice. Return that task's room.
                for (target, _) in &domain.names {
                    if keeps(domain, *target, credit) {
                        releases.push(Release { target: *target, credit }).expect("one release per answer");
                        break;
                    }
                }
            }
            Party::Deployment { .. } => {}
        }
    }
    Ok(releases.into_boxed())
}

fn keeps(domain: &Domain, target: u64, credit: NoticeCredit) -> bool {
    let Some(row) = record(domain, target) else { return false };
    row.notice_credits.contains(&credit)
}
