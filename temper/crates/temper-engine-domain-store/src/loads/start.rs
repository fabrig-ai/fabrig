//! Adapt the engine's startup walks to temper's paged ranges (jig/docs/design/domain/engine/vocabulary.md, 2.6).
use crate::{CoreRange, Key, Range};
use alloc::boxed::Box;
use jig_domain_core::{RestoreRange, engine::store::Cursor};

/// The next place the adapter can serve. The opaque engine cursor names this
/// place; the store's key remains typed and does not cross the engine port.
#[derive(Hash, Debug)]
pub enum Place {
    Page {
        range: Range,
        after: Option<Key>,
    },
    /// Let the root check its supplied charters after the last row is restored,
    /// before the engine finishes restoring settings and swaps their defaults.
    CoreEnd,
}

#[derive(Hash, Debug)]
pub struct Walk {
    sequence: u32,
    next: Option<Place>,
}

impl Walk {
    #[must_use]
    pub const fn empty() -> Self {
        Self { sequence: 0, next: None }
    }

    pub fn request(&mut self, ranges: Box<[RestoreRange]>, after: Option<Cursor>) -> Place {
        if let Some(after) = after {
            assert_eq!(after.interval, self.sequence, "start repeats the store's cursor");
            assert!(after.after.is_empty(), "opaque cursor carries only its place number");
            return self.next.take().expect("the start cursor names a page");
        }
        assert!(self.next.is_none(), "one start walk at a time");
        let range = if *ranges == RestoreRange::LIVE {
            Range::Core(CoreRange::Deployment)
        } else {
            let [only] = *<Box<[RestoreRange; 1]>>::try_from(ranges).expect("one non-LIVE start range");
            match only {
                RestoreRange::Census(census) => Range::Census(census),
                RestoreRange::People => Range::Core(CoreRange::Sweep),
                RestoreRange::Deployment
                | RestoreRange::Settings
                | RestoreRange::Accounts
                | RestoreRange::Tasks
                | RestoreRange::Core => unreachable!("the start walks LIVE together"),
            }
        };
        Place::Page { range, after: None }
    }

    /// The page owed by this ask, even before its delivery leaves the journal.
    #[must_use]
    pub fn waiting(&self, ranges: &[RestoreRange], after: Option<&Cursor>) -> Option<super::Kind> {
        if after.is_some() {
            return match &self.next {
                Some(Place::Page { range, .. }) => Some(super::kind(range)),
                Some(Place::CoreEnd) | None => None,
            };
        }
        if ranges == RestoreRange::LIVE {
            Some(super::Kind::Deployment)
        } else {
            match ranges {
                [RestoreRange::Census(_)] => Some(super::Kind::Census),
                [RestoreRange::People] => Some(super::Kind::People),
                [] | [_] | [_, _, ..] => unreachable!("the engine starts one of its three walks"),
            }
        }
    }

    #[must_use]
    pub fn at_core_end(&self, after: &Cursor) -> bool {
        match &self.next {
            Some(Place::CoreEnd) => after.interval == self.sequence,
            Some(Place::Page { .. }) | None => false,
        }
    }

    pub fn answered(&mut self, range: Range, after: Option<Key>) -> Option<Cursor> {
        let next = match after {
            Some(after) => Place::Page { range, after: Some(after) },
            None => {
                let range = match range {
                    Range::Core(CoreRange::Deployment) => CoreRange::Settings,
                    Range::Core(CoreRange::Settings) => CoreRange::Accounts,
                    Range::Core(CoreRange::Accounts) => CoreRange::People,
                    Range::Core(CoreRange::People) => CoreRange::Tasks,
                    Range::Core(CoreRange::Tasks) => CoreRange::RunProofs,
                    Range::Core(CoreRange::RunProofs) => CoreRange::Calls,
                    Range::Core(CoreRange::Calls) => return Some(self.cursor(Place::CoreEnd)),
                    Range::Core(CoreRange::Sweep) | Range::Census(_) => return None,
                    Range::Core(
                        CoreRange::RequesterWord { .. }
                        | CoreRange::TaskResult { .. }
                        | CoreRange::Turns { .. }
                        | CoreRange::ApplicationNotes(_),
                    )
                    | Range::Forge
                    | Range::History(_) => unreachable!("only engine start walks use this adapter"),
                };
                Place::Page { range: Range::Core(range), after: None }
            }
        };
        Some(self.cursor(next))
    }

    fn cursor(&mut self, place: Place) -> Cursor {
        assert!(self.next.replace(place).is_none(), "only one continuation per page");
        self.sequence = self.sequence.checked_add(1).expect("start walk page count");
        Cursor { interval: self.sequence, after: Box::new([]) }
    }
}
