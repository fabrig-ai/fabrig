#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod memory;
mod placed;
mod profiles;
mod questions;
mod runs;
mod secrets;
mod task_bounds;
mod values;

mod fuzzy {
    mod decode;
}

#[path = "../../../../../skein/testing/codec-memory.rs"]
mod codec_memory;
mod largest_memory;
