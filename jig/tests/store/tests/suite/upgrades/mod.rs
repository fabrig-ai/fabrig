//! Store upgrade scenarios on the simulated disk (protocol/store/testing.md, 1).
pub(crate) mod check;
pub(crate) mod commits;
mod frame_jobs;
mod frame_refusal;
pub(crate) mod frames;
pub(crate) mod generation_cuts;
mod generations;
mod jobs;
mod links;
pub(crate) mod logs;
pub(crate) mod rehearsal;
mod restores;
mod upgrade_edges;
