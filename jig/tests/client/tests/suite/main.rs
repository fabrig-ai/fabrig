//! Scripted client worlds (jig/docs/design/client/domain/testing.md, 4).
#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod client_construction;
mod faults;
mod incident;
mod memory;
mod orders;
mod person_confirmation;
mod random;
mod referee;
mod settings;
mod shared;
mod shell;
mod stories;
mod text_pages;
mod unread_turn;

mod fuzzy {
    mod client;
    mod memory;
    mod random_regressions;
}
