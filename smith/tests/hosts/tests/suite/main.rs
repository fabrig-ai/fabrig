#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod list;
mod memory;
mod program_source;
mod program_usage;
mod referee;
mod simulation;

mod fuzzy {
    mod crashes;
}
