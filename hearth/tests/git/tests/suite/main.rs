#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod batch;
mod hunks;
mod listing;
mod memory;
mod recordings;
mod tail;

mod fuzzy {
    mod git;
    mod revisions;
}
