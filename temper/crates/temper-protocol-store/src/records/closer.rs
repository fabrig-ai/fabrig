//! Goal closer rows (temper/docs/design/domain/forge.md, 11.2).
use super::{FITS, w};
use skein_lib::List;
use temper_engine_domain_forge::CloserRow;
use temper_engine_domain_forge_closer as c;

fn encode_step(step: c::Step) -> w::ClosingStep {
    match step {
        c::Step::Close => w::ClosingStep::Close,
        c::Step::Land => w::ClosingStep::Land,
    }
}
fn decode_step(step: w::ClosingStep) -> c::Step {
    match step {
        w::ClosingStep::Close => c::Step::Close,
        w::ClosingStep::Land => c::Step::Land,
    }
}
fn encode_run(run: Option<c::Running>) -> Option<w::ClosingRun> {
    let run = run?;
    Some(
        w::ClosingRun::new(&w::CEILINGS, w::ClosingRunParts { step: encode_step(run.step), change: run.change })
            .expect(FITS),
    )
}
fn decode_run(run: Option<w::ClosingRun>) -> Option<c::Running> {
    let w::ClosingRunParts { step, change } = run?.into_parts();
    Some(c::Running { step: decode_step(step), change })
}
pub(super) fn encode(row: &CloserRow) -> w::Closer {
    let closed = match &row.closer.closed {
        Some(tasks) => Some(
            w::ClosingTasks::new(&w::CEILINGS, w::ClosingTasksParts { tasks: List::from_boxed(tasks.clone()) })
                .expect(FITS),
        ),
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
    w::Closer::new(
        &w::CEILINGS,
        w::CloserParts {
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
            done: row.done,
        },
    )
    .expect(FITS)
}
pub(super) fn decode(row: w::Closer) -> CloserRow {
    let w::CloserParts {
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
        done,
    } = row.into_parts();
    let closed = match closed {
        Some(tasks) => Some(tasks.into_parts().tasks.into_boxed()),
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
        closer: c::Closer { closed, running: decode_run(running), landing, hold, retry },
        making,
        close_changes: close_changes.into_boxed(),
        landing_changes: landing_changes.into_boxed(),
        done,
    }
}
