#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

#[path = "../fixtures/stock.rs"]
mod stock;

mod accounts;
mod accounts_restart;
mod admission;
mod agent_reads;
mod applications;
mod budget;
mod configuring;
mod document_limits;
mod escalation;
mod fabrig_scale;
mod forge_routes;
mod hello_refused;
mod host_calls;
mod issue_projection;
mod loads;
mod mark;
mod memory;
mod notes_load;
mod oracle;
mod orders;
mod panel_before_change;
mod panels;
mod drafts;
mod periods;
mod playground;
mod preparation;
mod reconcile;
mod referee;
mod removal;
mod reset;
mod restore_fit;
mod roles;
mod root_routes;
mod settings;
mod switch;
mod trials;
mod walking;
mod watch_room;
mod ways;

mod fuzzy {
    mod endings;
    mod escalation;
    mod loads;
    #[expect(clippy::duplicate_mod, reason = "the same crash cases run in both profiles")]
    #[path = "../removal.rs"]
    mod removal;
    #[expect(clippy::duplicate_mod, reason = "the same crash cases run in both profiles")]
    #[path = "../reset.rs"]
    mod reset;
    mod roles;
    mod see;
    mod walking;
}

mod added_deps;
mod change_panel;
mod first_repository;
mod github_land;
mod kept_change;
mod making;
mod upgrade_transcripts;
