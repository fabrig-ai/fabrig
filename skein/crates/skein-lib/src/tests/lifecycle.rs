//! The component phases are distinct plain values (programming-model.md, 5.2).

use crate::{Lifecycle, state_digest};

#[test]
fn lifecycle_names_the_four_component_phases() {
    let phases = [Lifecycle::Live, Lifecycle::Closing, Lifecycle::Aborting, Lifecycle::Closed];
    for (index, phase) in phases.into_iter().enumerate() {
        let copied = phase;
        assert_eq!(phase, copied);
        for other in phases.get(index.checked_add(1).unwrap()..).unwrap() {
            assert_ne!(phase, *other);
            assert_ne!(state_digest(&phase), state_digest(other));
        }
        let name = match phase {
            Lifecycle::Live => "Live",
            Lifecycle::Closing => "Closing",
            Lifecycle::Aborting => "Aborting",
            Lifecycle::Closed => "Closed",
        };
        assert_eq!(alloc::fmt::format(format_args!("{phase:?}")), name);
    }
}
