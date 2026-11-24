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
mod layouts;
mod link;
mod liveness;
mod memory;
mod parts;
mod pause;
mod prepared;
mod records;
mod relays;
mod restarting;
mod retime;
mod sources;

mod fuzzy {
    mod layouts;
    mod restarting;
    mod worker_side;
}

mod figures;
