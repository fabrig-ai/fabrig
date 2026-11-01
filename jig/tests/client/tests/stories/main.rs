//! Project stories through the public client and semantic controls.
#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod projects;
