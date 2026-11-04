//! Integration tests collected in one suite.

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod disk;
mod golden;
mod memory;
mod open;
mod real;
mod store;

mod fuzzy {
    mod map;
}
