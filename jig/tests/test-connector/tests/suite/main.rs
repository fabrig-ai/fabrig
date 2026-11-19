#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod application;
mod application_documents;
mod restart_orders;
mod settings;
mod stories;

mod fuzzy {
    mod simulation;
}

#[path = "../../../../../skein/testing/codec-memory.rs"]
mod codec_memory;
mod largest_memory;
