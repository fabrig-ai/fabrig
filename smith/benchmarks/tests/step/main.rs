#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

pub use smith_bench_fixtures::common;

mod budget_visibility;
mod cache_probes;
mod codex;
mod codex_adapter;
mod events;
mod fault {
    mod tests;
}
mod failure_probes;
mod large_effects;
mod large_write_probe;
mod offline;
mod outcomes;
mod parallel_probes;
mod parity;
mod path_probes;
mod probe_command;
mod probes;
mod process_tree_probe;
mod slow_completion_probe;
mod smith_adapter;
mod smith_producer;
mod summary_resources;
mod tool_output_probes;

mod fuzzy {
    mod parsers;
}
