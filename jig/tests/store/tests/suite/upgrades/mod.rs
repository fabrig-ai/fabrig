//! Store upgrade scenarios on the simulated disk (protocol/store.md, 14).
pub(crate) mod check;
mod frame_refusal;
pub(crate) mod frames;
mod fresh_windows;
mod generations;
pub(crate) mod patching;
mod upgrade_edges;
