#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;
pub(crate) mod fixtures;
mod checks;
mod startup;
mod root_checks;
mod room;
mod link_room;
mod workspace_reuse;
mod mirrors;
mod limits;
mod translate;
mod worker;
mod worker_alone;
mod worker_parts;
mod system;
mod stock;
mod workers;
mod deployment;
