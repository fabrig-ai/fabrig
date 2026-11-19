#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod drift;

mod fuzzy {
    mod records;
}

mod codec_memory {
    include!("../../../../../skein/testing/codec-memory.rs");
}
mod largest_memory;
