#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod application;
mod application_documents;
mod release;
mod restart_orders;
mod stories;

mod fuzzy {
    mod simulation;
}
