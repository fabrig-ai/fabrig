#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[path = "../support/frame_reads.rs"]
mod frame_reads;

mod check;
mod frame_refusal;
mod frames;
mod fresh_windows;
mod generations;
mod patching;
mod upgrade_edges;
