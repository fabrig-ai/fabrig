#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

#[path = "../../../../../testing/reference_settings.rs"]
mod reference_settings;
#[path = "../fixtures/stock.rs"]
mod stock;
mod store_check;

mod accounts;
mod accounts_restart;
mod admission;
mod agent_reads;
mod applications;
mod budget;
mod catching_up;
mod catching_up_progress;
mod checkpoints;
mod checks;
mod configuring;
mod deleted_project;
mod direct_driver;
mod document_limits;
mod drafts;
mod ends;
mod escalation;
mod fabrig_scale;
mod findings;
mod forge_routes;
mod hello_refused;
mod host_calls;
mod issue_projection;
mod loads;
mod mark;
mod memory;
mod notes_load;
mod open;
mod opened;
mod opening;
mod oracle;
mod orders;
mod panel_before_change;
mod panels;
mod periods;
mod playground;
mod preparation;
mod push;
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
    mod closer_attempts;
    mod endings;
    mod escalation;
    mod findings;
    mod follows;
    mod landing_settings;
    mod loads;
    mod opening;
    mod reds_owed;
    mod restarts;
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
