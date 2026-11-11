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
                    return Some((local, now(row.project, row.chip.task)));
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
            Some((8, now(task.project, task.chip.task)))
        }
        _ => None,
    }
}

fn now(project: u32, task: u64) -> c::Read {
    c::Read::Notes {
        scope: c::NoteScope::Task { project, task },
        description: Some(c::Text::new(b"now".as_slice().into(), 3).expect("Now description")),
        after: None,
    }
}
