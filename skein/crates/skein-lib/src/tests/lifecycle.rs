//! Component lifecycle values (programming-model.md, 5.2).

use crate::{Lifecycle, state_digest};

#[test]
fn lifecycle_phases_have_distinct_identities() {
    let phases = [Lifecycle::Live, Lifecycle::Closing, Lifecycle::Aborting, Lifecycle::Closed];
    for (left_index, left) in phases.iter().enumerate() {
        for (right_index, right) in phases.iter().enumerate() {
            assert_eq!(left == right, left_index == right_index);
            if left_index != right_index {
                assert_ne!(state_digest(left), state_digest(right));
            }
        }
    }
}
