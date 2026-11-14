#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

pub(crate) mod fixtures;
mod keys;
mod records;
mod sizes;

mod tiers;

mod settings;

mod follows;

mod catch_up;

mod batches;

mod history;
mod reds;
mod registered_layouts;
mod unseeded_upgrade;
mod value_types;
