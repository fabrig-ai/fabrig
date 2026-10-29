#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod journal;
mod memory;
mod request_table;

mod fuzzy {
    mod bytes;
    mod intake;
    mod journal;
    mod request_table;
}
