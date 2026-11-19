#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod asked_link;
mod authorize;
mod closing;
mod gone;
mod placed_hold_client;
mod placed_holds;
mod provider;
mod reads;
mod requests;
mod settings;
mod sign_in;
mod stream;

mod public;
mod public_http;

mod fuzzy {
    mod cursor;
}

#[path = "../../../../../skein/testing/codec-memory.rs"]
mod codec_memory;
mod largest_memory;
