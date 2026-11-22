//! Temper's engine and worker roots with Smith's actual agent domain over
//! fake LLM IO, jig's real store and fake parties, and skein's fake forge.
//! Narrow engine-to-Smith fixtures also retain forge and goal checks.

// The focused composed and forge stories report a median of 17 commits.
const STORE_CHECK_ONE_IN: u64 = 32;

pub mod composed;
#[cfg(test)]
mod forge;
#[cfg(test)]
mod goals;
pub mod world;
