#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod drift;
mod issuer_tokens;
mod memory;
mod openid;
mod record_memory;
mod terms;
mod world;

mod fuzzy {
    mod record;
    mod world;
}
