#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[path = "../support/frame_reads.rs"]
mod frame_reads;

mod accounts;
mod activity;
mod carried;
mod crashes;
mod directories;
mod frame_memory;
mod fresh_windows;
mod generations;
mod in_flight;
mod installation;
mod joins;
mod layout_golden;
mod mark_gate;
mod orphan_memory;
mod pages;
mod patching;
mod patching_memory;
mod pending_switch;
mod readers;
mod records;
mod refused_after_switch;
mod transcript_golden;

mod fuzzy {
    mod crashes;
    mod frames;
    mod patching;
    #[path = "../readers_fuzzy.rs"]
    mod readers;
}
