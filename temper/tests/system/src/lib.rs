//! Temper's engine and worker roots with Smith's actual agent domain over
//! fake LLM IO, jig's real store and fake parties, and skein's fake forge.
//! Narrow engine-to-Smith fixtures also retain forge and goal checks.

pub mod composed;
#[cfg(test)]
mod forge;
#[cfg(test)]
mod goals;
pub mod world;
