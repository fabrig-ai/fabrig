//! Join standing tasks to their Now (domain/agent/standing.md, 6).
use super::reads::Reading;
use crate::jig::rows;
use jig_client::{Readout, contract as c};

/// Sequential note reads share the supplemental slot's pending bit.
pub(super) fn next(reading: &Reading) -> Option<(u32, c::Read)> {
    match reading.called.call.tool {
        rows::ROLE_LIST_AT => {
            let Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page))) = reading.pages.get(&1)? else {
                return None;
            };
            for (index, row) in page.live.as_slice().iter().enumerate() {
                let local = 8u32.checked_add(u32::try_from(index).expect("bounded task rows")).expect("read slot");
                if !reading.pages.contains_key(&local) {
                    return Some((local, note(row.project, row.chip.task, b"now")));
                }
            }
            None
        }
        rows::ROLE_SHOW_AT => {
            if reading.pages.contains_key(&8) {
                return None;
            }
            let Readout::Jig(c::ReadResult::Value(value)) = reading.pages.get(&1)? else { return None };
            let (task, _) = crate::jig::render::read_task(value)?;
            let description =
                if crate::jig::call::value(&reading.called.call, 1).is_some() { b"report".as_slice() } else { b"now" };
            Some((8, note(task.project, task.chip.task, description)))
        }
        _ => None,
    }
}

fn note(project: u32, task: u64, description: &[u8]) -> c::Read {
    c::Read::Notes {
        scope: c::NoteScope::Task { project, task },
        description: Some(c::Text::new(description.into(), 6).expect("role note description")),
        after: None,
    }
}
