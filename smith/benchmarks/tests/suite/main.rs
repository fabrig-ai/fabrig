#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

#[path = "../common/mod.rs"]
pub mod common;

mod budget_visibility;
mod cache_probes;
mod codex;
mod codex_adapter;
mod events;
mod failure_probes;
mod large_effects;
mod large_write_probe;
mod offline;
mod outcomes;
mod parallel_probes;
mod parity;
mod path_probes;
mod pipeline;
mod probe_command;
mod probes;
mod process_tree_probe;
mod runner;
mod slow_completion_probe;
mod smith_adapter;
mod smith_producer;
mod summary_resources;
mod tool_output_probes;

mod fuzzy {
    mod parsers;
}
