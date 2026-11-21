#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod real;

#[path = "tls_memory.rs"]
mod memory;
#[path = "tls_world.rs"]
mod world;
