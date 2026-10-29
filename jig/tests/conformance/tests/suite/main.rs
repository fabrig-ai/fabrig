//! Worlds for the conformance testing application.

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod application;
mod client_settings;
mod client_settings_proposals;
mod faults;
mod memory;
mod referee;
mod restart;
mod scenarios;
mod simulation;
mod snapshot;
mod trace;

mod fuzzy {
    mod cuts;
    mod scenarios;
    mod snapshot;
}
