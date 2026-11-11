//! Checkout worlds (temper/docs/design/testing/README.md, 2).

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod delivery;
mod git;
mod memory;
mod merges;
mod real_delivery;
mod retries;
mod reuse;
mod simulation;
mod stores;

mod fuzzy {
    mod merges;
    mod simulation;
}
