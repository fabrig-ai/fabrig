//! Tests that replay (tools/tiers/layers.md, 6.1).
use temper_box_kit as support;
mod admission_edges;
mod configuration;
mod fake_opening;
mod forge_land;
#[path = "../suite/inbox_reply.rs"]
mod inbox_reply;
#[path = "../suite/producer_policy.rs"]
mod producer_policy;
mod relay;
mod startup;
mod support_tests;
