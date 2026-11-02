#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod memory;
mod page_server;

mod fuzzy {
    mod memory;
    mod world;
}
