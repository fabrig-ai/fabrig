#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

use jig_store_world::frame_reads;

mod accounts;
mod activity;
mod carried;
mod crashes;
mod directories;
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
mod sweep;
mod transcript_golden;
mod upgrades;

mod fuzzy {
    mod crashes;
    mod generations;
    mod pending_switch;
    #[path = "../readers_fuzzy.rs"]
    mod readers;
    mod upgrades;
}
