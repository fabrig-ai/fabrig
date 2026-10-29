#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

pub(crate) mod fixtures;
mod keys;
mod records;
mod sizes;
