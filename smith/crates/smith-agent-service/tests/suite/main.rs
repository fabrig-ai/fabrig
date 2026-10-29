#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod sink_memory;

mod fuzzy {
    mod profile;
}
