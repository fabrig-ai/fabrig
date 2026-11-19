//! Referee controls over supplied observations, without driving a world.

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod referee;
