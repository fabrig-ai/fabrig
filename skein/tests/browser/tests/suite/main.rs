#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod devtools;
mod memory;
mod page_server;
mod words;

mod fuzzy {
    mod memory;
    mod world;
}
