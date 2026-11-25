//! Task queries and lines preserve the engine's keys and supplied order.
use super::{chips, live, me, owner, w};
use crate::generated::client as mapped;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::documents::{self as d, WaitsOn};
use jig_domain_tasks as t;
use skein_lib::{List, Writer};

pub(super) fn query(value: w::TaskQuery) -> Option<d::TaskQuery> {
    let requester = match value.requester {
        None => None,
        Some(w::Requester::Task(v)) => Some(t::Party::Task(v.value)),
        Some(w::Requester::Person(v)) => Some(t::Party::Person(v.value)),
        Some(w::Requester::Deployment(_)) => return None,
    };
    Some(d::TaskQuery {
        parts: {
            let mut parts = List::with_capacity(value.parts.len());
            for ask in value.parts.into_boxed() {
                parts.push(mapped::part_ask_to_domain(ask)).expect("bounded parts");
            }
            parts
        },
        project: value.project,
        requester,
        root: value.root,
        parent: value.parent,
        label: value.label,
        phase: match value.phase {
            Some(value) => Some(mapped::phase_class_to_domain(value)),
            None => None,
        },
        executor: match value.executor {
            Some(value) => Some(mapped::executor_kind_to_domain(value)),
            None => None,
        },
        tracked: value.tracked,
        order: mapped::query_order_to_domain(value.order),
    })
}
pub(super) fn waits(value: &WaitsOn) -> Result<w::WaitsOn, w::Problem> {
    mapped::waits_on_to_record(value)
}
pub fn line(value: &d::TaskLine, limits: &w::Limits) -> Result<w::TaskLine, w::Problem> {
    line_parts(value, &List::with_capacity(0), limits)
}
pub(super) fn line_parts(
    value: &d::TaskLine,
    slots: &List<super::parts::Slot>,
    limits: &w::Limits,
) -> Result<w::TaskLine, w::Problem> {
    Ok(w::TaskLine {
        parts: super::parts::row(&value.parts, slots)?,
        chip: chips::chip(&value.chip, limits)?,
        project: value.project,
        requester: owner::requester(value.requester),
        tracked: value.tracked,
        label: value.label,
        flags: me::flags(value.flags),
        changed: value.changed.as_nanos(),
        last_activity: value.last_activity.as_nanos(),
        ended_at: match value.ended_at {
            None => None,
            Some(at) => Some(at.as_nanos()),
        },
        budget: owner::budget(&value.budget),
        waits_on: waits(&value.waits_on)?,
        wait: super::waits::first(value.wait.as_ref())?,
        nearest: super::waits::nearest(value.nearest),
        phases: live::phases(value.phases),
        delegates_done: value.delegates_done,
        delegates_total: value.delegates_total,
        held_below: value.held_below,
        conversation: match value.conversation {
            None => None,
            Some(v) => Some(mapped::conversation_read_to_record(&v)),
        },
    })
}
pub(crate) fn lines(
    value: &d::Lines,
    slots: &List<super::parts::Slot>,
    limits: &w::Limits,
) -> Result<w::Tasks, w::Problem> {
    let mut rows = List::with_capacity(value.lines.len());
    for row in &value.lines {
        let row = line_parts(row, slots, limits)?;
        let mut writer = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        row.write(limits, &mut writer).expect("measured task line");
        rows.push(writer.finish()).expect("same line count");
    }
    let rows = super::pieces(&rows, limits.tasks_rows, w::paths::TASKS_ROWS)?;
    Ok(w::Tasks { ended_mark: value.ended_mark, rows, more: value.next.is_some() })
}
