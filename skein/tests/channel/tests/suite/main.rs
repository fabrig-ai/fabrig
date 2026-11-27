//! Integration tests collected in one suite.

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

pub mod generic_protocol;
mod memory;

mod fuzzy {
    mod channel;
}
