#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod authorize;
mod check_replay;
mod client_construction;
mod faults;
mod incident;
mod memory;
mod orders;
mod owner_authority;
mod owner_budget;
mod person_confirmation;
mod placed_holds;
mod random;
mod referee;
mod settings;
mod shell;
mod stories;
mod text_pages;
mod unlinked;
mod unread_turn;

mod fuzzy {
    mod client;
    mod joint;
    mod memory;
    mod random_regressions;
}
