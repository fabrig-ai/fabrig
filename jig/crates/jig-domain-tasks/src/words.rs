//! Words retained beyond their run's fence (jig/docs/design/domain/tasks/messages.md, 2).
use crate::domain::{Domain, entrance, publish, refused, task_mut};
use crate::{EndedBy, Executor, Limits, MessageKind, Party, Refusal, Request, TaskRecord, Word};
use skein_lib::{Env, List, Queue, ReplyTo};

/// Whether words direct this task: a person's, or its requester task's
/// (jig/docs/design/domain/tasks/messages.md, 2). This does not test whether the task keeps words.
#[must_use]
pub fn directing_words(record: &TaskRecord, word: &Word) -> bool {
    word.kind == MessageKind::Words
        && match word.from {
            Party::Person(_) => true,
            Party::Task(_) => word.from == record.requester,
            Party::Deployment { .. } => false,
        }
}

/// Fresh-brief agent tasks keep words independently of questions and dependencies.
pub(crate) fn keeping(domain: &Domain, row: &TaskRecord) -> bool {
    if row.flags.conversation {
        return false;
    }
    match row.flags.ended_by {
        EndedBy::Requester | EndedBy::Standing => return false,
        EndedBy::Result | EndedBy::Closer(_) => {}
    }
    let charter = match row.executor {
        Executor::Agent { charter } => charter,
        Executor::Procedure { .. } | Executor::Person(_) | Executor::Job { .. } => return false,
    };
    let mut latest = 0;
    let mut resumes = false;
    for version in domain.charters() {
        if version.number == charter && version.revision <= row.settings.task && version.revision >= latest {
            latest = version.revision;
            resumes = version.resumes;
        }
    }
    !resumes
}

pub(crate) fn record(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    number: u64,
    messages: &[u64],
    out: &mut Queue<Request>,
) {
    if !domain.ready() {
        return refused(to, Some(number), Refusal::NotReady, out);
    }
    let to = match entrance(domain, to, number) {
        Ok(to) => to,
        Err((to, why)) => return refused(to, Some(number), why, out),
    };
    let task = task_mut(domain, number).expect("recording entrance checked");
    let capacity = u32::try_from(messages.len()).expect("root bounds recording names");
    let mut recorded = List::with_capacity(capacity);
    let mut unknown = List::with_capacity(capacity);
    let mut kept = List::with_capacity(u32::try_from(task.record.unrecorded.len()).expect("bounded words"));
    for word in &task.record.unrecorded {
        if !messages.contains(&word.number) {
            kept.push(word.clone()).expect("retained subset fits");
        }
    }
    for &message in messages {
        let mut found = false;
        for word in &task.record.unrecorded {
            found |= word.number == message;
        }
        if found {
            recorded.push(message).expect("recorded names counted");
        } else {
            unknown.push(message).expect("unknown names counted");
        }
    }
    task.record.unrecorded = kept.into_boxed();
    publish(domain, env, number, out);
    out.push(Request::WordsRecorded {
        reply_to: to,
        task: number,
        recorded: recorded.into_boxed(),
        unknown: unknown.into_boxed(),
    });
}
