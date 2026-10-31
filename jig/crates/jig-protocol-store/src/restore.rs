//! Restart pages, including the inbox census joins (protocol/store.md, 7.2).
mod diagnostic;
pub use diagnostic::{
    Arithmetic, Cause, DecodeFailure, Failure, JoinFailure, OpenFailure, RangeFailure, RowSite, WalkFailure,
};
pub(crate) use diagnostic::{ready, site};
pub(crate) mod reader;
use crate::{Below, Limits, secrets};
use crate::{Cursor, Event, Phase, Restore, Store, intervals};
pub use reader::{Item, Loaded, page};
use skein_lib::{Env, Queue, Token};

pub(crate) struct Page {
    pub range: Restore,
    pub after: Option<Cursor>,
    pub rows: u32,
    pub bytes: u64,
}
pub(crate) fn restore(
    store: &mut Store,
    env: &Env<Limits>,
    owner: Token,
    page: Page,
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    let Page { range, after, rows, bytes } = page;
    let people = people_interval(&range);
    match self::page(store, range, after.as_ref(), rows, bytes) {
        Ok(walked) => {
            if finish(store, env, people, walked.next.as_ref(), above, below) {
                above.push(Event::Page { owner, items: walked.items, next: walked.next });
            }
        }
        Err(failure) => {
            store.phase = Phase::Stopped;
            above.push(Event::PageRefused { owner, failure });
        }
    }
}

/// Restore completes people's inventory before arming its sweep (store.md, 9).
fn finish(
    store: &mut Store,
    env: &Env<Limits>,
    people: Option<u32>,
    next: Option<&Cursor>,
    above: &mut Queue<Event>,
    below: &mut Below,
) -> bool {
    if let Some(interval) = people {
        let done = match next {
            Some(next) => next.interval > interval,
            None => true,
        };
        if done && secrets::start_sweep(store) {
            secrets::sweep(store, env, above, below);
        }
    }
    store.phase != Phase::Stopped
}

/// An empty page either ends the walk or advances its exclusive place.
pub(crate) fn assert_progress(after: Option<&Cursor>, empty: bool, next: Option<&Cursor>) {
    if empty && let Some(next) = next {
        let advanced = match after {
            Some(after) => {
                next.interval > after.interval || (next.interval == after.interval && next.after > after.after)
            }
            None => next.interval > 0 || !next.after.is_empty(),
        };
        assert!(advanced, "empty nonterminal restore page must advance its cursor");
    }
}

fn people_interval(range: &Restore) -> Option<u32> {
    match range {
        Restore::Jig(ranges) => {
            let mut interval = 0_u32;
            let mut people = None;
            for range in ranges {
                match range {
                    jig_domain_core::RestoreRange::People => {
                        let count = u32::try_from(intervals::restore(range).len()).ok()?;
                        people = Some(interval.checked_add(count.checked_sub(1)?)?);
                    }
                    jig_domain_core::RestoreRange::Deployment
                    | jig_domain_core::RestoreRange::Settings
                    | jig_domain_core::RestoreRange::Accounts
                    | jig_domain_core::RestoreRange::Tasks
                    | jig_domain_core::RestoreRange::Core
                    | jig_domain_core::RestoreRange::Census(_) => {}
                }
                interval = interval.checked_add(u32::try_from(intervals::restore(range).len()).ok()?)?;
            }
            people
        }
        Restore::Application(_) => None,
    }
}
