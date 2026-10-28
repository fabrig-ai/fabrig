//! Worlds for ops and its reference root.

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod acts;
mod admission;
mod application;
mod application_documents;
mod client_stories;
mod connector_checks;
mod incident_leads;
mod infrastructure;
mod observability;
mod orders;
mod stories;
mod trace;
mod turn_backoff;

mod fuzzy {
    mod alert_cuts;
    mod client;
    mod connectors;
    mod joint;
    mod stories;
}
