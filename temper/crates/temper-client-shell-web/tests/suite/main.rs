use skein_world::domain::heap::Counting;

#[global_allocator]
static HEAP: Counting = Counting;

mod host;
mod memory;
