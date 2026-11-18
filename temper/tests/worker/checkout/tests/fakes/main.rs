//! Checkout stories on the fake forge and fake disk (temper/docs/design/domain/worker/README.md, 9).

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod delivery;
mod git;
mod memory;
mod merges;
mod retries;
mod reuse;
mod simulation;
mod stores;

mod fuzzy {
    mod merges;
    mod simulation;
}
