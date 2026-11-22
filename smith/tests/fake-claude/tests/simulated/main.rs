#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod support;

mod acts;
mod cues;
mod encodings;
mod mcp;
mod refusals;
mod relay;
mod simulated;
mod stream;
mod turns;
mod usage;
