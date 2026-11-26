//! The root's ranges, expressed in jig's store vocabulary.
//! Window pages carry whole or cut turns (temper/docs/design/domain/root.md, 7).
use super::route::Flight;
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_protocol_store as store;
use temper_engine_domain::{CoreRange, Key, Range};

/// Startup selects stored families; writer rules are proved by store oracles
/// (temper/docs/design/domain/root-restart.md, 1.1).
pub(super) fn starting(range: &Range) -> bool {
    match range {
        Range::Core(
            CoreRange::Deployment
            | CoreRange::Settings
            | CoreRange::Accounts
            | CoreRange::People
            | CoreRange::Sweep
            | CoreRange::Tasks
            | CoreRange::RunProofs
            | CoreRange::Calls,
        )
        | Range::Census(_)
        | Range::Forge => true,
        Range::Core(
            CoreRange::TaskResult { .. }
            | CoreRange::RequesterWord { .. }
            | CoreRange::Turns { .. }
            | CoreRange::Notes(_),
        )
        | Range::History(_) => false,
    }
}

pub(super) fn keeps(range: &Range, key: &Key) -> bool {
    match range {
        Range::Census(_) => true,
        Range::Core(CoreRange::Calls) => {
            let Key::Core(core::Key::Core(core::CoreKey::Call(_) | core::CoreKey::Part(_))) = key else { return false };
            true
        }
        Range::Core(CoreRange::RunProofs) => {
            let Key::Core(core::Key::Core(core::CoreKey::RunProof(_))) = key else { return false };
            true
        }
        Range::Core(_) | Range::Forge | Range::History(_) => range.contains(key),
    }
}

pub(super) fn cursor(range: &Range, key: &Key) -> Option<store::Cursor> {
    let after = match key {
        Key::Wires | Key::Opening(_) => return None,
        Key::Core(key) => store::keys::encode(key, u32::MAX)?,
        Key::Forge(key) => store::keys::application(&temper_protocol_store::keys::encode(key, u32::MAX)?, u32::MAX)?,
    };
    let interval = match (range, key) {
        (_, Key::Wires | Key::Opening(_)) => return None,
        (Range::Census(census), Key::Core(key)) => census.interval(key)?,
        (Range::Core(range), Key::Core(_)) if !range.restore_ranges().is_empty() => {
            return store::intervals::cursor(range.restore_ranges(), &after);
        }
        (Range::History(range), Key::Forge(key)) => history_cursor(range, key)?,
        (Range::Core(_) | Range::Census(_) | Range::Forge | Range::History(_), Key::Core(_) | Key::Forge(_)) => 0,
    };
    Some(store::Cursor { interval, after })
}

pub(super) fn request(owner: skein_lib::Token, flight: &Flight) -> Option<store::Request> {
    let after = if let Some(cursor) = &flight.cursor {
        Some(cursor.clone())
    } else {
        match &flight.after {
            Some(key) => Some(cursor(&flight.range, key)?),
            None => None,
        }
    };
    let rows = flight.most;
    let bytes = u64::from(flight.bytes);
    let range = match &flight.range {
        Range::History(range) => {
            return Some(store::Request::Load {
                owner,
                range: store::Load::Application(history_intervals(range)?),
                after,
                rows,
                bytes,
            });
        }
        Range::Forge => store::Restore::Application(Box::new([temper_protocol_store::interval()])),
        Range::Census(census) => store::Restore::Jig(Box::new([core::RestoreRange::Census(census.clone())])),
        Range::Core(range) => match range {
            CoreRange::Deployment
            | CoreRange::Settings
            | CoreRange::Accounts
            | CoreRange::People
            | CoreRange::Sweep
            | CoreRange::Tasks
            | CoreRange::Calls
            | CoreRange::RunProofs => store::Restore::Jig(Box::from(range.restore_ranges())),
            CoreRange::RequesterWord { task, position } => {
                let key = core::Key::Tasks(jig_domain_tasks::Key::Activity { task: *task, position: *position });
                return Some(jig_get(owner, key));
            }
            CoreRange::TaskResult { task } => {
                return Some(jig_get(owner, core::Key::Tasks(jig_domain_tasks::Key::Ended(*task))));
            }
            CoreRange::Notes(range) => {
                return Some(store::Request::Load {
                    owner,
                    range: store::Load::Notes(range.clone()),
                    after,
                    rows,
                    bytes,
                });
            }
            CoreRange::Turns { task, .. } => {
                return Some(store::Request::Transcript {
                    owner,
                    read: store::TranscriptRead::Page { task: *task, backwards: false, after: flight.offset },
                    rows,
                    bytes,
                });
            }
        },
    };
    Some(store::Request::Restore { owner, range, after, rows, bytes })
}

fn jig_get(owner: skein_lib::Token, key: core::Key) -> store::Request {
    store::Request::Get { owner, key: store::Get::Jig(key) }
}

/// Intervals stay relative to the application's tag; jig supplies that tag.
fn history_intervals(range: &temper_engine_domain_forge::HistoryRange) -> Option<Box<[skein_kv::Range]>> {
    use temper_engine_domain_forge::{HistoryRange, Key};
    if !range.valid() {
        return None;
    }
    let (key, suffix) = match range {
        HistoryRange::Events { project } => (Key::HistoryEvent { project: *project, number: 0 }, 8),
        HistoryRange::Change { project, change } => {
            (Key::HistoryChange { project: *project, change: *change, number: 0 }, 8)
        }
        HistoryRange::Landing { project, landing } => {
            (Key::HistoryLanding { project: *project, landing: *landing, number: 0 }, 8)
        }
        HistoryRange::Last { project } => (Key::HistoryLast { project: *project, number: 0, change: 0 }, 16),
        HistoryRange::Numbers { project, numbers } => {
            let mut ordered = skein_lib::Map::with_capacity(32);
            for number in numbers {
                ordered.insert(*number, ()).ok()?;
            }
            let mut intervals = skein_lib::List::with_capacity(32);
            for (number, ()) in &ordered {
                let key = temper_protocol_store::keys::encode(
                    &Key::HistoryEvent { project: *project, number: *number },
                    u32::MAX,
                )?;
                intervals.push(skein_kv::Range::prefix(&key)).ok()?;
            }
            return Some(intervals.into_boxed());
        }
    };
    let key = temper_protocol_store::keys::encode(&key, u32::MAX)?;
    Some(Box::new([skein_kv::Range::prefix(key.get(..key.len().checked_sub(suffix)?)?)]))
}

fn history_cursor(
    range: &temper_engine_domain_forge::HistoryRange,
    key: &temper_engine_domain_forge::Key,
) -> Option<u32> {
    let key = temper_protocol_store::keys::encode(key, u32::MAX)?;
    for (index, interval) in history_intervals(range)?.iter().enumerate() {
        let before_end = match &interval.end {
            Some(end) => &key < end,
            None => true,
        };
        if key >= interval.start && before_end {
            return u32::try_from(index).ok();
        }
    }
    None
}
