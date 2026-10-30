#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod account_capacity;
mod account_changes;
mod acts;
mod compound;
mod memory;
mod referee;
mod restore_open;
mod simulation;

mod fuzzy {
    mod simulation;
}

mod start_restore;
