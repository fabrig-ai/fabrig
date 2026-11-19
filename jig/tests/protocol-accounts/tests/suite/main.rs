#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod drift;
mod failures;
mod installation;
mod keys;
mod memory;
mod moves;
mod restart;
mod stories;

mod fuzzy {
    mod moved;
}

#[path = "../../../../../skein/testing/codec-memory.rs"]
mod codec_memory;
mod largest_memory;
