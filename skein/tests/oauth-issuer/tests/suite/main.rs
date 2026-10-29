#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[path = "../support/mod.rs"]
mod support;
mod world;

mod fuzzy {
    mod world;
}
