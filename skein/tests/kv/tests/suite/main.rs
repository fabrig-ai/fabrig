//! Integration tests collected in one suite.

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod census;
mod disk;
mod golden;
mod holds;
mod logs;
mod memory;
mod open;
mod real;
mod recovery;
mod store;
mod target;

mod fuzzy {
    mod census;
    mod holds;
    mod logs;
    mod map;
}
