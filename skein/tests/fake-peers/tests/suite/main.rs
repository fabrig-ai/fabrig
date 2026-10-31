//! Integration tests collected in one suite.

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod forge;
mod github_memory;
mod llm;
mod oauth;
