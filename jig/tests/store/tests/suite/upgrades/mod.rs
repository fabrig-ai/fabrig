//! Store upgrade scenarios on the simulated disk (protocol/store/README.md, 9).
pub(crate) mod check;
mod frame_refusal;
pub(crate) mod frames;
mod fresh_windows;
mod generations;
pub(crate) mod patching;
mod rehearsal;
mod upgrade_edges;

mod baseline;
