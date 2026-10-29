#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod memory;
mod program_source;
mod referee;
mod simulation;

mod fuzzy {
    mod crashes;
}
