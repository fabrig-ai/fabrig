#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod bounds;
mod budgets;
mod domain;
mod files;
mod goldens;
mod memory;
mod orders;
mod properties;
mod protocol;
mod resolution;
mod simulation;
mod store;

mod fuzzy {
    mod inputs;
    mod memory;
    mod protocol;
    mod simulation;
}
