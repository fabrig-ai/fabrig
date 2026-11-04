#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod compose;
mod debian;
mod decoded;
mod decoders;
mod diff_reuse;
mod entries;
mod errors;
mod evaluate;
mod exports;
mod golden_harness;
mod goldens;
mod laws;
mod memory;
mod oracle;
mod properties;
mod schema;
mod templates;
mod thin;

mod fuzzy {
    mod entries;
    mod evaluate;
    mod oracle;
    mod properties;
    mod schema;
    mod templates;
}
