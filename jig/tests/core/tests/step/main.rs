//! Referee controls and direct step inputs (jig/docs/design/domain/testing.md, 9).
#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

use jig_core_world::test_support::bundle;
use jig_core_world::test_support::first as first_support;

mod inbox_selection_memory;
mod memory;
mod open;
mod procedure_seeds;
mod referee;
mod settings_memory;
mod settings_proposal_edges;
mod start;
mod start_projections;
mod start_runs;

mod store_adapter;
mod store_check;
mod store_peer;
use jig_core_world::test_support::store_peer as store_peer_support;
