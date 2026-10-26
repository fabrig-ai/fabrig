//! The journeys: the client's pages recorded, replayed and printed (client/journeys.md).

pub mod print;
/// Where the journeys live, by the manifest's path (client/journeys.md, 5.1).
pub const JOURNEYS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/design/client/journeys");

#[cfg(test)]
mod tests;
