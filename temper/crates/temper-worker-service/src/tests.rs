#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;
pub(crate) mod fixtures;
mod limits;
mod worker;
mod worker_admission;
mod worker_alone;
mod worker_git;
#[path = "tests/worker_job_answers.rs"]
mod worker_job_answers;
mod worker_orders;
mod worker_parts;
mod worker_slots;

mod checks;
mod root_checks;
mod startup;
mod startup_calls;
mod worker_declaration;
