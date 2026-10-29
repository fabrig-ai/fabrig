//! Integration tests collected in one suite.

#![expect(clippy::duplicate_mod, reason = "protocol cases keep their focused and fuzzy test namespaces")]

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

pub mod generic_protocol;
mod memory;

mod fuzzy {
    mod channel;
}
