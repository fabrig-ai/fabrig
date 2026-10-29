//! Integration tests collected in one suite.

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod access;
mod github_calls;
mod github_delivery_log;
mod github_installation_loss;
mod github_installations;
mod github_job_log;
mod github_joint;
mod github_peer;
mod github_renewal;
mod github_start;
mod github_unknown_repository;
mod hooks;
mod machine;
mod memory;

mod fuzzy {
    mod hooks;
    mod machine;
    mod requests;
    mod tls;
}
