#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod memory;
mod positions;
mod tagged;
mod transcripts;
mod world;
mod writer;

mod fuzzy {
    mod collector;
    mod documents;
    mod writer;
}
