#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

#[path = "../../src/joined.rs"]
mod joined_world;

mod agreement;
mod busy;
mod cascades;
mod dialling;
mod grace;
mod jobs;
mod joined;
mod link;
mod liveness;
mod memory;
mod records;
mod relays;
mod sources;
mod statement;
mod translation;

mod fuzzy {
    mod translation;
    mod worker_side;
}
