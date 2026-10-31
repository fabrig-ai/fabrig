#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

#[path = "../support/fresh_windows.rs"]
mod fresh_windows;
#[path = "../settings/support.rs"]
mod settings_support;

mod activity;
mod admission;
mod application_documents;
mod charters;
mod client_peer;
mod client_peer_routes;
mod completion_retry;
mod credentials;
mod crossed_claims;
mod deadline_spans;
mod decision_reads;
mod deployment_acts;
mod effect_fences;
mod effects;
mod ended_indexes;
mod ended_watches;
mod faults;
mod fields;
mod first_start;
mod forgetting;
mod funding;
mod host_tools;
mod inbox_selection_memory;
mod instructions;
mod joint_clock;
mod memory;
mod operations;
mod orders;
mod parties;
mod peers;
mod placed_holds;
mod procedure_steps;
mod projections;
mod projects;
mod read_filters;
mod reads;
mod referee;
mod refusals;
mod restart;
mod restore_pages;
mod route_orders;
mod run_positions;
mod runs;
mod settings;
mod settings_memory;
mod settings_proposals;
mod settings_reads;
mod simulation;
mod start;
mod start_projections;
mod stopping;
mod store;
mod store_adapter;
mod store_peer;
mod stories;
mod switch_step;
mod task_queries;
mod templates;
mod topics;

mod fuzzy {
    mod faults;
    mod reads;
    mod settings;
    mod stories;
    mod watches;
}
