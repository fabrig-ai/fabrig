#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[path = "../support/frame_reads.rs"]
mod frame_reads;

mod accounts;
mod activity;
mod carried;
mod checkpoint_close;
mod checkpoints;
mod crashes;
mod directories;
mod frame_memory;
mod generations;
mod installation;
mod joins;
mod layout_golden;
mod map_held;
mod mark_gate;
mod openings;
mod orphan_memory;
mod pages;
mod pending_switch;
mod readers;
mod records;
mod refused_after_switch;
mod registry_memory;
mod transcript_golden;
mod upgrades;

mod fuzzy {
    mod checkpoints;
    mod crashes;
    mod frames;
    mod generations;
    mod pending_switch;
    #[path = "../readers_fuzzy.rs"]
    mod readers;
    mod upgrades;
}
