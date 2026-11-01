//! Bounded source text reads (jig/docs/design/client/domain/contract.md, 6).
use super::{Answer, Why};
use crate::{Core, Limits};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;

/// Which source words a task text read retrieves.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskTextField {
    /// The task's stored specification words.
    Specification,
    /// Its override, or the charter instructions at its recorded settings revision.
    Instructions,
}

/// One UTF-8 chunk with byte offsets into the complete source.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TaskText {
    pub task: u64,
    pub field: TaskTextField,
    pub from: u64,
    pub total: u64,
    pub next: Option<u64>,
    pub bytes: Box<[u8]>,
}

/// Read a visible live or retained owner without projecting its summary first.
/// Reject offsets outside the text or inside a UTF-8 character. An unavailable
/// effective instruction source is unknown, rather than an empty invention.
pub fn task_text_from_row(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &tasks::TaskRecord,
    field: TaskTextField,
    from: u64,
) -> Result<TaskText, Why> {
    if !core.open_project(row.project) || core.people.role(party, row.project).is_none() {
        return Err(Why::Unknown);
    }
    let source = match field {
        TaskTextField::Specification => row.spec.words.as_ref(),
        TaskTextField::Instructions => {
            let tasks::Executor::Agent { charter } = row.executor else { return Err(Why::Unknown) };
            match &row.instructions {
                Some(words) => words.as_ref(),
                None => &core.charter_at(charter, row.settings.task).ok_or(Why::Unknown)?.policy.instructions,
            }
        }
    };
    chunk(row.number, field, source, from, limits.documents.text_bytes)
}

fn chunk(task: u64, field: TaskTextField, source: &[u8], from: u64, maximum: u32) -> Result<TaskText, Why> {
    let Ok(start) = usize::try_from(from) else { return Err(Why::Unknown) };
    let rest = source.get(start..).ok_or(Why::Unknown)?;
    if let Some(first) = rest.first()
        && first & 0xc0 == 0x80
    {
        return Err(Why::Unknown);
    }
    let prefix = skein_lib::bytes::utf8_prefix(rest, skein_lib::length::usize_of(maximum));
    let bytes = skein_lib::bytes::copy_of(prefix);
    if bytes.is_empty() && !rest.is_empty() {
        return Err(Why::Busy);
    }
    let total = u64::try_from(source.len()).expect("source length fits");
    let end = from.checked_add(u64::try_from(bytes.len()).expect("chunk length fits")).expect("source bound");
    let next = if end < total { Some(end) } else { None };
    Ok(TaskText { task, field, from, total, next, bytes })
}

pub(super) fn answer(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &tasks::TaskRecord,
    field: TaskTextField,
    from: u64,
) -> Answer {
    match task_text_from_row(core, limits, party, row, field, from) {
        Ok(value) => Answer::TaskText(Box::new(value)),
        Err(why) => Answer::None(why),
    }
}
