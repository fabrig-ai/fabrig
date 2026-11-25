#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

pub(crate) mod fixtures;
mod keys;
mod landing_conversion;
mod lowering;
mod records;
mod sizes;

mod tiers;

mod settings;

mod follows;

mod batches;

mod dev_directory;
mod dev_layouts;
mod directory_jobs;
mod history;
mod reds;
mod registered_layouts;

mod value_types;

mod jobs;
mod test_ids;
