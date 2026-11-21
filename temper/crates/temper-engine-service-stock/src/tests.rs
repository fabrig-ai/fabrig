#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;
pub(crate) mod fixtures;
mod providers;
mod references;
mod stock;
