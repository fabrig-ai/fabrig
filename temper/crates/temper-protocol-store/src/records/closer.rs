//! Goal closer rows (temper/docs/design/domain/forge/branches-closer.md, 1).
use super::{FITS, w};
use alloc::boxed::Box;
use skein_lib::List;
use temper_engine_domain_forge_closer as c;
use temper_engine_domain_forge_vocabulary::CloserRow;

fn encode_step(step: c::Step) -> w::ClosingStep {
    crate::generated::store::closing_step_to_record(&step)
}
fn decode_step(step: w::ClosingStep) -> c::Step {
    crate::generated::store::closing_step_to_domain(step)
}
fn encode_run(run: Option<c::Running>) -> Option<w::ClosingRun> {
    let run = run?;
    Some(crate::generated::store::closing_run_to_record(&run))
}
fn decode_run(run: Option<w::ClosingRun>) -> Option<c::Running> {
    let run = run?;
    Some(crate::generated::store::closing_run_to_domain(run))
}
pub(super) fn encode(row: &CloserRow) -> w::Closer {
    let closed = match &row.closer.closed {
        Some(tasks) => Some(w::ClosingTasks { tasks: List::from_boxed(tasks.clone()) }),
        None => None,
    };
    let caught_up = match &row.closer.caught_up {
        Some(tasks) => Some(w::ClosingTasks { tasks: List::from_boxed(tasks.clone()) }),
        None => None,
    };
    let hold = match row.closer.hold {
        Some(c::Hold::ClosingStep { step, change }) => encode_run(Some(c::Running { step, change })),
        None => None,
    };
    let retry = match row.closer.retry {
        Some(step) => Some(encode_step(step)),
        None => None,
    };
    let making = match row.making {
        Some(step) => Some(encode_step(step)),
        None => None,
    };
    w::Closer {
        report_check: crate::generated::store::closing_report_check_to_record(&row.closer.report_check),
        goal_left: row.closer.goal_left.clone(),
        goal_asked: row.closer.goal_asked.clone(),
        task: row.task,
        goal: row.goal,
        closed,
        running: encode_run(row.closer.running),
        landing: row.closer.landing,
        hold,
        retry,
        making,
        close_changes: List::from_boxed(row.close_changes.clone()),
        landing_changes: List::from_boxed(row.landing_changes.clone()),
        selections: encode_selections(&row.closer.selections),
        done: row.done,
        relay: encode_relay(&row.relay),
        superseded: List::from_boxed(row.superseded.clone()),
        caught_up,
        catch_up_changes: List::from_boxed(row.catch_up_changes.clone()),
    }
}
pub(super) fn decode(row: w::Closer) -> CloserRow {
    let w::Closer {
        report_check,
        goal_left,
        goal_asked,
        task,
        goal,
        closed,
        running,
        landing,
        hold,
        retry,
        making,
        close_changes,
        landing_changes,
        selections,
        done,
        relay,
        superseded,
        caught_up,
        catch_up_changes,
    } = row;
    let closed = match closed {
        Some(tasks) => Some(tasks.tasks.into_boxed()),
        None => None,
    };
    let caught_up = match caught_up {
        Some(tasks) => Some(tasks.tasks.into_boxed()),
        None => None,
    };
    let hold = match decode_run(hold) {
        Some(c::Running { step, change }) => Some(c::Hold::ClosingStep { step, change }),
        None => None,
    };
    let retry = match retry {
        Some(step) => Some(decode_step(step)),
        None => None,
    };
    let making = match making {
        Some(step) => Some(decode_step(step)),
        None => None,
    };
    CloserRow {
        task,
        goal,
        closer: c::Closer {
            closed,
            caught_up,
            running: decode_run(running),
            landing,
            hold,
            retry,
            selections: decode_selections(selections),
            report_check: crate::generated::store::closing_report_check_to_domain(report_check),
            goal_left,
            goal_asked,
        },
        making,
        close_changes: close_changes.into_boxed(),
        catch_up_changes: catch_up_changes.into_boxed(),
        superseded: superseded.into_boxed(),
        landing_changes: landing_changes.into_boxed(),
        relay: decode_relay(relay),
        done,
    }
}

fn encode_relay(relay: &c::relay::Relay) -> Option<w::ClosingRelay> {
    if relay.open.is_none() && relay.waiting.is_empty() && relay.replies.is_empty() {
        return None;
    }
    Some(crate::generated::store::closing_relay_to_record(relay).expect(FITS))
}
fn decode_relay(relay: Option<w::ClosingRelay>) -> c::relay::Relay {
    match relay {
        Some(relay) => crate::generated::store::closing_relay_to_domain(relay),
        None => c::relay::Relay::new(),
    }
}

fn encode_selections(selections: &[c::PullSelection]) -> List<w::ClosingSelection> {
    let mut rows = List::with_capacity(u32::try_from(selections.len()).expect(FITS));
    for selection in selections {
        rows.push(crate::generated::store::closing_selection_to_record(selection).expect(FITS)).expect(FITS);
    }
    rows
}

fn decode_selections(selections: List<w::ClosingSelection>) -> Box<[c::PullSelection]> {
    let mut rows = List::with_capacity(selections.len());
    for selection in selections.into_boxed() {
        rows.push(crate::generated::store::closing_selection_to_domain(selection)).expect(FITS);
    }
    rows.into_boxed()
}
