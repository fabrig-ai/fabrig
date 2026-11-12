//! Small live documents and the shared bounded chip construction.
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).

use super::{Chip, ExecutorKind, Missing, PhaseLine, Text};
use crate::Limits;
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{List, bytes::copy_of};

#[must_use]
pub fn text(bytes: &[u8], maximum: u32) -> Text {
    assert!(skein_lib::bytes::is_utf8(bytes), "source text is UTF-8 at the engine entrance");
    let prefix = skein_lib::bytes::utf8_prefix(bytes, skein_lib::length::usize_of(maximum));
    Text { bytes: copy_of(prefix), cut: prefix.len() < bytes.len() }
}

pub fn chip_from_row(limits: &Limits, row: &tasks::TaskRecord) -> Result<Chip, Missing> {
    let number = row.number;
    let first = skein_lib::bytes::find(&row.spec.words, b"\n").unwrap_or(row.spec.words.len());
    let title = text(row.spec.words.get(..first).expect("first line prefix"), limits.documents.text_bytes);
    let executor = match row.executor {
        tasks::Executor::Agent { .. } => ExecutorKind::Agent,
        tasks::Executor::Procedure { .. } => ExecutorKind::Procedure,
        tasks::Executor::Person(_) => ExecutorKind::Person,
        tasks::Executor::Job { .. } => ExecutorKind::Job,
    };
    let phase = match &row.phase {
        tasks::Phase::Waiting => {
            let mut dependencies = List::with_capacity(limits.documents.lines);
            for number in row.waiting_on.iter().take(skein_lib::length::usize_of(limits.documents.lines)) {
                dependencies.push(*number).expect("bounded dependency lines");
            }
            PhaseLine::Waiting {
                more: skein_lib::length::usize_of(dependencies.len()) < row.waiting_on.len(),
                dependencies,
                holds: !row.holds_taken && !row.holdings.is_empty(),
            }
        }
        tasks::Phase::Active(active) => match row.executor {
            tasks::Executor::Person(address) => PhaseLine::Person {
                address,
                taken_by: row.taken_by,
                since: match row.person_wait {
                    Some(wait) => Some(wait.since),
                    None => None,
                },
            },
            tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } | tasks::Executor::Job { .. } => {
                PhaseLine::Active(*active)
            }
        },
        tasks::Phase::Closing(closing) => PhaseLine::Closing(closing.stage),
        tasks::Phase::Held { why, .. } => PhaseLine::Held { reason: *why, revision: row.revisions.hold },
        tasks::Phase::Ended(_) => PhaseLine::Ended,
    };
    Ok(Chip { number, title, executor, phase, details: Box::new(super::chip_phase::from_row(limits, row)?) })
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_text_cut_inside_a_character_keeps_it_out_and_says_it_was_cut() {
        for (maximum, expected, cut) in [(0, "", true), (1, "a", true), (2, "a", true), (3, "aé", false)] {
            let actual = super::text("aé".as_bytes(), maximum);
            assert_eq!(actual.bytes.as_ref(), expected.as_bytes(), "bound {maximum}");
            assert_eq!(actual.cut, cut, "bound {maximum}");
        }
    }
}
