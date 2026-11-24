#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

pub mod common;

mod codex_adapter;
mod pipeline;
mod probe_command;
mod runner;
mod smith_adapter;
