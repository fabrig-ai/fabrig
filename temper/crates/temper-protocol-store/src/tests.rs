#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

pub(crate) mod fixtures;
mod keys;
mod records;
mod sizes;

mod tiers;

mod settings;

mod follows;

mod batches;

mod directory_jobs;
mod history;
mod reds;
mod registered_layouts;

// Import paths of the frozen F0 and F1 fixtures.
use directory_jobs as unseeded_upgrade;
use fixtures as catch_up;
mod value_types;

mod jobs;

mod baseline;

mod store_floor;
