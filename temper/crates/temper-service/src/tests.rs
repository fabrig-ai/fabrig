#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;
mod checks;
mod deployment;
pub(crate) mod fixtures;
mod limits;
mod link_room;
mod mirrors;
mod room;
mod root_checks;
mod startup;
mod stock;
mod system;
mod translate;
mod worker;
mod worker_alone;
mod worker_parts;
mod workers;
mod workspace_reuse;
