//! Per-run grouping for the run world's independent oracles.
//! Contract: smith/docs/design/domain/run.md, section 13.

use std::collections::BTreeMap;

use skein_lib::{Time, Token};
use skein_world::domain::{Expectations, Referee, Verdict};

pub(crate) fn judge<Meeting>(seen: &[(Time, Token, Meeting::Seen)]) -> Vec<Verdict>
where
    Meeting: Expectations + Default,
    Meeting::Seen: Copy,
{
    let mut meetings = BTreeMap::new();
    for (at, run, seen) in seen {
        let meeting = meetings.entry(*run).or_insert_with(|| Referee::new(Meeting::default()));
        meeting.observe(*at, *seen, &mut Vec::new());
    }
    meetings.values().map(Referee::verdict).collect()
}
