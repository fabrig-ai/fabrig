//! Every deletion commit ends as the uncut run does (`jig/docs/design/domain/testing.md`, 5).

use crate::deletion::{a_cut_deletion_ends_the_same, uncut_deletion};
use jig_core_world::world::Cut;

#[test]
fn a_deletion_cut_after_any_commit_ends_the_same() {
    let (first, last, expected) = uncut_deletion();
    let cuts: Vec<_> = (first..=last).flat_map(|number| [Cut::Submitted(number), Cut::Durable(number)]).collect();
    a_cut_deletion_ends_the_same(first, &cuts, &expected);
}
