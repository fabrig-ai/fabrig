#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod core_wait;
pub(crate) mod fixtures;
mod limits;
mod store_route;

mod store;

mod accounts;
mod approval;
mod engine;
mod first_model;
mod first_period;
mod hello_refused;
mod stop;

mod tools;
mod translate;
mod worker_parts;
mod workers;

mod clients;
mod clients_unwatch;
mod worker;
mod worker_alone;
mod worker_admission;
mod worker_git;
#[path = "tests/worker_job_answers.rs"]
mod worker_jobs;
mod worker_orders;
mod worker_slots;

mod box_link;
mod checks;
mod family;
mod reference_landing;
mod root_checks;
mod startup;
mod system;

mod forge;
mod forge_runtime;
mod forge_sign_in;

mod bundle;
mod server;

pub(crate) mod sign_in;

mod sign_in_configuration;

mod provider;

mod ask_bound;
mod stock;

mod room;

mod workspace_reuse;

mod link_room;

mod mirrors;
