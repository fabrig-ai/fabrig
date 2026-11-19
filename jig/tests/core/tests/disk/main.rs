//! Simulated disk cuts, generations and owner-set fixtures (jig/docs/design/domain/testing.md, 9).
#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

#[path = "../support/closed_proofs.rs"]
mod closed_proofs;
use jig_core_world::test_support::faults as fault_support;
use jig_core_world::test_support::person as person_support;
use jig_core_world::test_support::switch_step as switch_support;

mod closed_windows;
mod crossed_claims;
mod faults;
mod runs;
mod store_adapter;
mod switch_step;

mod fuzzy {
    mod faults;
}

use jig_core_world::test_support::run_helpers;
