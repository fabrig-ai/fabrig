//! Integration tests collected in one suite.

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod golden;
mod memory;
mod real;
mod store;

mod fuzzy {
    mod map;
}
