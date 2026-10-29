#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod exchange;
mod fake_peer;
mod handshake;
mod memory;
mod native_memory;
mod native_output;
mod stack;
mod startup;

mod fuzzy {
    mod native;
    mod world;
}
