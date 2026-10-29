//! Checkout worlds (temper/docs/design/domain/testing.md, 2).

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod git;
mod memory;
mod merges;
mod retries;
mod simulation;

mod fuzzy {
    mod merges;
    mod simulation;
}
