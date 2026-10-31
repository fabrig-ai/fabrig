//! Live runs, their preparation, calls and text (`jig/docs/design/domain/engine.md`, sections 7 and 9).

pub(crate) mod account_preference;
pub(crate) mod brief_text;
pub(crate) mod conversation;
pub(crate) mod coverage;
pub(crate) mod delegation;
pub(crate) mod host_rule;
pub(crate) mod live;
pub(crate) mod llm;
pub(crate) mod model_names;
pub(crate) mod retained;
pub(crate) mod run_choice;
pub(crate) mod state;
pub(crate) mod transcript_text;
pub mod waits;

pub(crate) mod calls;
pub(crate) mod proofs;
