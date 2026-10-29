#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[path = "../support/frame_reads.rs"]
mod frame_reads;

mod qa_check;
mod qa_frame_refusal;
mod qa_frames;
mod qa_fresh_windows;
mod qa_generations;
mod qa_patching;
mod qa_upgrade_edges;
