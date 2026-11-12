//! Live runs, their preparation, calls and text (`jig/docs/design/domain/engine/executors.md` and `jig/docs/design/domain/engine/briefs.md`, section 1).

mod account_llm;
pub(crate) mod account_preference;
pub(crate) mod brief_text;
pub(crate) mod conversation;
pub(crate) mod coverage;
pub(crate) mod delegation;
pub(crate) mod host_rule;
pub(crate) mod live;
pub(crate) mod llm;
pub(crate) mod model_names;
mod model_ways;
pub(crate) mod retained;
pub(crate) mod run_choice;
pub(crate) mod state;
pub(crate) mod transcript_text;
pub mod waits;

pub(crate) mod calls;
pub(crate) mod proofs;
