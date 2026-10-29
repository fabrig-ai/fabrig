#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod client;
mod events;
mod memory;
mod memory_server;
mod requests;
mod server;
mod stack;
mod transcripts;
mod writer;

mod fuzzy {
    mod client;
    mod events;
    mod gzip;
    mod server;
    mod writer;
}
