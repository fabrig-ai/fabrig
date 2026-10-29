#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[path = "../support/mod.rs"]
mod support;

mod acts;
mod binary;
mod cues;
mod encodings;
mod mcp;
mod relay;
mod sign_in;
mod simulated;
mod stream;
mod turns;
