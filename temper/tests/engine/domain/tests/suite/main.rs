#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

#[path = "../fixtures/stock.rs"]
mod stock;

mod accounts;
mod accounts_restart;
mod admission;
mod applications;
mod budget;
mod demo;
mod document_limits;
mod escalation;
mod forge_routes;
mod host_calls;
mod issue_projection;
mod loads;
mod mark;
mod memory;
mod notes_load;
mod orders;
mod panel_before_change;
mod panels;
mod periods;
mod preparation;
mod reconcile;
mod referee;
mod restore_fit;
mod roles;
mod root_routes;
mod settings;
mod switch;
mod walking;

mod fuzzy {
    mod endings;
    mod escalation;
    mod loads;
    mod roles;
    mod walking;
}
