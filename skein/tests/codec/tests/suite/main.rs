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
mod every;
mod fingerprint;
mod generic;
mod largest;
mod lean_bounds;
mod memory;
mod named_limits;
mod patch;
mod wire;
mod worst;

mod fuzzy {
    mod decode;
    mod generic;
    mod layout;
    mod patch;
}

mod codec_memory {
    include!("../../../../testing/codec-memory.rs");
}
mod largest_memory;
