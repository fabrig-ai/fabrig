//! Integration tests for the stacked LLM client.

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod anthropic;
mod anthropic_admission;
mod archives;
mod bounded_peer_memory;
mod classes;
mod client;
mod credentials;
mod derive;
mod document;
mod failures;
mod fake;
mod fake_credentials;
mod fake_failures;
mod fake_memory;
mod meaning;
mod measure;
mod memory;
mod prepared_raw;
mod public_failures;
mod public_route;
mod raw_call_history;
mod replay;
mod replay_limits;
mod replay_total;
mod selective;
mod stream_head;
mod tool_identities;

mod fuzzy {
    mod client;
    mod derive;
}
