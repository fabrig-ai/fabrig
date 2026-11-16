//! Store upgrade scenarios on the simulated disk (protocol/store/README.md, 9).
pub(crate) mod check;
pub(crate) mod commits;
mod frame_jobs;
mod frame_refusal;
pub(crate) mod frames;
mod generations;
mod jobs;
mod links;
pub(crate) mod logs;
pub(crate) mod patching;
mod rehearsal;
mod restores;
mod upgrade_edges;
