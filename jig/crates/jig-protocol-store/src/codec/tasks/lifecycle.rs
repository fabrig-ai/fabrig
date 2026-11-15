//! Attempt and hold rows (domain/tasks/lifecycle.md, 12.1 and 12.2).
use super::{activity, phase, value};
use crate::tasks_v13 as wire;
use jig_domain_tasks as domain;
use skein_lib::Wall;

fn end(source: Option<domain::EndKind>, limits: &wire::Limits) -> Result<Option<wire::EndKind>, wire::Problem> {
    match source {
        Some(end) => Ok(Some(activity::end_kind(&end, limits)?)),
        None => Ok(None),
    }
}
fn read_end(source: Option<wire::EndKind>) -> Result<Option<domain::EndKind>, wire::Problem> {
    match source {
        Some(end) => Ok(Some(activity::read_end_kind(end)?)),
        None => Ok(None),
    }
}
fn wall(source: Option<Wall>) -> Option<u64> {
    match source {
        Some(at) => Some(at.as_nanos()),
        None => None,
    }
}

pub(super) fn attempt(row: &domain::AttemptRow, limits: &wire::Limits) -> Result<wire::AttemptRow, wire::Problem> {
    wire::AttemptRow::new(
        limits,
        wire::AttemptRowParts {
            task: row.task,
            attempt: row.attempt,
            claimed: wall(row.claimed),
            started: wall(row.started),
            ended: wall(row.ended),
            end: end(row.end, limits)?,
            phases: row.phases,
        },
    )
}
pub(super) fn read_attempt(row: wire::AttemptRow) -> Result<domain::AttemptRow, wire::Problem> {
    let wire::AttemptRowParts { task, attempt, claimed, started, ended, end, phases } = row.into_parts();
    Ok(domain::AttemptRow {
        task,
        attempt,
        claimed: value::read_wall(claimed),
        started: value::read_wall(started),
        ended: value::read_wall(ended),
        end: read_end(end)?,
        phases,
    })
}
pub(super) fn open(
    row: Option<&domain::OpenAttempt>,
    limits: &wire::Limits,
) -> Result<Option<wire::OpenAttempt>, wire::Problem> {
    match row {
        Some(row) => Ok(Some(wire::OpenAttempt::new(
            limits,
            wire::OpenAttemptParts {
                attempt: row.attempt,
                claimed: wall(row.claimed),
                started: wall(row.started),
                from: row.from,
                end: end(row.end, limits)?,
            },
        )?)),
        None => Ok(None),
    }
}
pub(super) fn read_open(row: Option<wire::OpenAttempt>) -> Result<Option<domain::OpenAttempt>, wire::Problem> {
    match row {
        Some(row) => {
            let wire::OpenAttemptParts { attempt, claimed, started, from, end } = row.into_parts();
            Ok(Some(domain::OpenAttempt {
                attempt,
                claimed: value::read_wall(claimed),
                started: value::read_wall(started),
                from,
                end: read_end(end)?,
            }))
        }
        None => Ok(None),
    }
}
pub(super) fn hold(row: &domain::HoldRow, limits: &wire::Limits) -> Result<wire::HoldRow, wire::Problem> {
    wire::HoldRow::new(
        limits,
        wire::HoldRowParts {
            task: row.task,
            position: row.position,
            at: row.at.as_nanos(),
            held: wire::Held::new(
                limits,
                wire::HeldParts {
                    was: phase::was(&row.held.was, limits)?,
                    why: phase::hold(&row.held.why, limits)?,
                    source: phase::optional_source(row.held.source.as_deref(), limits)?,
                },
            )?,
        },
    )
}
pub(super) fn read_hold(row: wire::HoldRow, limits: &wire::Limits) -> Result<domain::HoldRow, wire::Problem> {
    let wire::HoldRowParts { task, position, at, held } = row.into_parts();
    let wire::HeldParts { was, why, source } = held.into_parts();
    Ok(domain::HoldRow {
        task,
        position,
        at: Wall::from_nanos(at),
        held: domain::Held {
            was: phase::read_was(was, limits)?,
            why: phase::read_hold(why)?,
            source: phase::read_optional_source(source, limits)?,
        },
    })
}
