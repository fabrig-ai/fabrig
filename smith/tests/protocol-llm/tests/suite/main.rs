//! Protocol LLM integration tests.

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod adapter_boundary;
mod adapter_continuation;
mod brief;
mod budget;
mod client;
mod completion_deadlines;
mod component_boundary;
mod component_peer;
mod content_observations;
mod endpoints_grants;
mod guard;
mod history_window;
mod jobs;
mod native_root_restore;
mod nesting;
mod pool_waiting;
mod production_contract;
mod production_render;
mod production_tools;
mod referee;
mod root_memory;
mod root_wire;
mod workspace;

mod fuzzy {
    mod pool;
}
