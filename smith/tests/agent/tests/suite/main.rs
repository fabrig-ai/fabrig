#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod accounts;
mod brief;
mod budget;
mod compaction;
mod feedback;
mod feedback_table;
mod host_delivery;
mod host_messages;
mod host_tools;
mod initial_merge;
mod memory_returns;
mod messages;
mod messages_referee;
mod models;
mod parallel_children;
mod referee;
mod simulation;
mod workspace;

mod fuzzy {
    mod memory;
    mod messages;
    mod simulation;
}
