#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod account_keys;
mod accounts;
mod drift;
#[path = "../fixtures/mod.rs"]
mod fixtures;
mod placed;
mod placed_holds;
mod remove_project;
mod set_visibility;
mod settings;
mod settings_proposals;
mod task_settings;
mod terminal_login;
mod test_client_wire;

mod fuzzy {
    mod client_wire;
    mod public_wire;
}
