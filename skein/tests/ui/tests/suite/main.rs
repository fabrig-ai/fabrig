#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod memory;
