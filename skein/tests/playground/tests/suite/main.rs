//! The playground kit's world tests (playground.md, 8).

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod memory;
mod replay;
mod slices;
mod visitor;
