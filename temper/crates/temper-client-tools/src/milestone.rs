//! Milestone joins through the shared client (client/ctl/milestones.md, 4).
mod print;
use crate::jig::{call::number, render, rows, runner::reads::Reading};
use jig_client::{Readable, Readout, contract as c};
pub(crate) use print::print;
use temper_client_domain_forge::forge;
// Totals contain integers alone; no forge field or check array is received.
const TOTALS: forge::Limits = forge::Limits { field_bytes: 0, checks: 0 };
pub(crate) fn next(reading: &Reading) -> Option<(u32, Readable)> {
    if reading.called.call.tool != rows::MILESTONE_SHOW_AT {
        return None;
    }
    let Readout::Jig(c::ReadResult::Value(value)) = reading.pages.get(&1)? else { return None };
    let (task, _) = render::read_task(value)?;
    if !reading.pages.contains_key(&2) {
        let query = c::TaskQuery {
            parts: skein_lib::List::with_capacity(0),
            project: Some(task.project),
            requester: None,
            root: None,
            parent: Some(task.chip.task),
            label: None,
            phase: None,
            executor: None,
            tracked: None,
            order: c::QueryOrder::Number,
        };
        return Some((2, Readable::Jig(c::Read::Tasks { query, live: None, ended: None })));
    }
    if !reading.pages.contains_key(&6) {
        let query = c::TaskQuery {
            parts: skein_lib::List::with_capacity(0),
            project: Some(task.project),
            requester: None,
            root: None,
            parent: Some(task.chip.task),
            label: None,
            phase: Some(c::PhaseClass::Ended),
            executor: None,
            tracked: None,
            order: c::QueryOrder::Number,
        };
        return Some((6, Readable::Jig(c::Read::Tasks { query, live: None, ended: None })));
    }
    if !reading.pages.contains_key(&3) {
        let read = forge::milestones::read(task.project, number(&reading.called.call, 0)?, &TOTALS)
            .expect("milestone read within tools limits");
        return Some((3, read));
    }
    if !reading.pages.contains_key(&4) {
        return Some((4, Readable::Jig(c::Read::Project { project: task.project })));
    }
    if !reading.pages.contains_key(&5) {
        let totals = forge::milestones::answer(reading.pages.get(&3)?, &TOTALS)?;
        if let temper_client_protocol_wire::v1::MilestoneResult::Totals(totals) = totals
            && totals.ended.is_none()
            && let Some(landing) = &totals.last_landing
        {
            let Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page))) = reading.pages.get(&2)? else {
                return None;
            };
            for row in &page.live {
                if row.chip.task == landing.goal {
                    return None;
                }
            }
            let Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(ended))) = reading.pages.get(&6)? else {
                return None;
            };
            if let c::EndedGroup::Rows { rows, .. } = &ended.ended {
                for row in rows {
                    if row.chip.task == landing.goal {
                        return None;
                    }
                }
            }
            return Some((5, Readable::Jig(c::Read::Task { task: landing.goal })));
        }
    }
    None
}
