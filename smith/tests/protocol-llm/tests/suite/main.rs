//! Protocol LLM integration tests.

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod budget;
mod codec;
mod codec_boundary;
mod completion_deadlines;
mod component_boundary;
mod component_peer;
mod content_observations;
mod continuation;
mod contract;
mod endpoints_grants;
mod guard;
mod history_window;
mod host_feedback;
mod jobs;
mod nesting;
mod pool_waiting;
mod prefix;
mod referee;
mod render;
mod replay;
mod tools;
mod workspace;

mod fuzzy {
    mod pool;
}
