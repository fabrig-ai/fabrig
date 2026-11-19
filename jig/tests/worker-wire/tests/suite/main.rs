#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod drift;

mod fuzzy {
    mod records;
}

#[path = "../../../../../skein/testing/codec-memory.rs"]
mod codec_memory;
mod largest_memory;
