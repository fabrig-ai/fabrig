#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod look_world;
mod memory;
mod page_server;
mod support;

mod fuzzy {
    mod memory;
    mod world;
}
