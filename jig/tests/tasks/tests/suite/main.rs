#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod activity;
mod added;
mod circles;
mod exceeded;
mod funding;
mod holds;
mod inbox;
mod kept;
mod lifecycle;
mod llm;
mod memory;
mod met;
mod open;
mod person;
mod placed;
mod procedures;
mod proposals;
mod questions;
mod recurring;
mod referee;
mod repin_order;
mod replay;
mod restart;
mod restore_open;
mod verdict_fields;

mod fuzzy {
    mod amendments;
    mod lifecycle;
}
