//! Core stories over the in-memory store (jig/docs/design/domain/testing.md, 4 and 9).
#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

use jig_core_world::test_support::run_helpers;
#[path = "../settings/support.rs"]
mod settings_support;

use jig_core_world::test_support::faults as fault_support;
use jig_core_world::test_support::first as first_support;
use jig_core_world::test_support::person as person_support;
use jig_core_world::test_support::switch_step as switch_support;

mod activity;
mod admission;
#[path = "../support/admission.rs"]
mod admission_support;
mod application_documents;
mod application_settings;
use jig_core_world::test_support::bundle;
mod charters;
mod client_peer;
mod client_peer_routes;
mod completion_retry;
mod credentials;
mod deadline_spans;
mod decision_reads;
mod deletion;
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
mod instructions;
mod jobs;
mod joint_clock;
mod lists;
mod memory;
mod model_act;
mod models;
mod named_people;
mod operations;
mod oracle;
mod orders;
mod parties;
mod peers;
mod per_wake;
mod person_reads;
mod person_task;
mod placed_holds;
mod procedure_steps;
mod projections;
mod projects;
mod read_filters;
mod reads;
mod refusals;
mod restart;
mod restore_pages;
mod retention;
mod route_orders;
mod run_positions;
mod runs;
mod sessions;
mod settings;
mod settings_project_policy;
mod settings_proposal_edges;
mod settings_proposals;
mod settings_reads;
mod settings_restore;
mod simulation;
mod standing_now;
mod start_connectors;
mod start_inbox;
mod start_links;
mod start_settings;
mod stopping;
mod store;
mod store_peer;
mod stories;
mod switch_step;
mod task_queries;
mod templates;
mod topics;

mod fuzzy {
    mod decision_reads;
    mod deletion;
    mod faults;
    mod reads;
    mod retention;
    mod sessions;
    mod settings;
    mod stories;
    mod watches;
}

#[path = "../support/reads.rs"]
mod read_support;

mod public;

mod party_runs;

mod party_reads;

mod party_brief;

mod withheld;

use jig_core_world::test_support::store_peer as store_peer_support;
