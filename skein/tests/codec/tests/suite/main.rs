//! Integration tests collected in one suite.

extern crate alloc;

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[path = "../support/generic.rs"]
mod generic_support;
mod support;

mod bounds;
mod diff;
mod drawn;
mod drift;
mod fingerprint;
mod generic;
mod memory;
mod names;
mod patch;
mod wire;
mod worst;

mod fuzzy {
    mod decode;
    mod generic;
    mod patch;
}
