//! Tests that replay (tools/tiers/layers.md, 6.1).
use temper_qa_tool::testing::{case, landing, pages, session};
#[path = "../suite/configuration.rs"]
mod configuration;
#[path = "../suite/fuzzy.rs"]
mod fuzzy;
mod gaps;
#[path = "../suite/ledger.rs"]
mod ledger;
#[path = "../suite/passes.rs"]
mod passes;
#[path = "../suite/reference.rs"]
mod reference;
mod run;
#[path = "../suite/stat.rs"]
mod stat;
mod table;
mod tune;
