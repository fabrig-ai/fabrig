//! Scripted documents in the generated client wire vocabulary (client/domain/testing.md, 4).
#![expect(clippy::unnecessary_wraps, reason = "field translations share a fallible signature for bounded constructors")]
#![expect(clippy::trivially_copy_pass_by_ref, reason = "field translations borrow all source facts uniformly")]
#[path = "values/admission_limits.rs"]
mod admission_limits;
pub(super) use admission_limits::*;
#[path = "values/asks.rs"]
mod asks;
pub(super) use asks::*;
#[path = "values/chip_phase.rs"]
mod chip_phase;
pub(super) use chip_phase::*;
#[path = "values/common.rs"]
mod common;
pub(super) use common::*;
#[path = "values/document_text.rs"]
mod document_text;
pub(super) use document_text::*;
#[path = "values/documents.rs"]
mod documents;
pub(super) use documents::*;
#[path = "values/effects.rs"]
mod effects;
pub(super) use effects::*;
#[path = "values/ended_pages.rs"]
mod ended_pages;
pub(super) use ended_pages::*;
#[path = "values/history_rows.rs"]
mod history_rows;
pub(super) use history_rows::*;
#[path = "values/live_call.rs"]
mod live_call;
pub(super) use live_call::*;
#[path = "values/llm.rs"]
mod llm;
pub(super) use llm::*;
#[path = "values/me.rs"]
mod me;
pub(super) use me::*;
#[path = "values/operations.rs"]
mod operations;
pub(super) use operations::*;
#[path = "values/accounts.rs"]
mod accounts;
pub(super) use accounts::*;
#[path = "values/activity.rs"]
mod activity;
pub(super) use activity::*;
#[path = "values/amendment.rs"]
mod amendment;
pub(super) use amendment::*;
#[path = "values/authority.rs"]
mod authority;
pub(super) use authority::*;
#[path = "values/budget.rs"]
mod budget;
pub(super) use budget::*;
#[path = "values/chip.rs"]
mod chip;
pub(super) use chip::*;
#[path = "values/deployment.rs"]
mod deployment;
pub(super) use deployment::*;
#[path = "values/escalation.rs"]
mod escalation;
pub(super) use escalation::*;
#[path = "values/ended.rs"]
mod ended;
pub(super) use ended::*;
#[path = "values/history.rs"]
mod history;
pub(super) use history::*;
#[path = "values/inbox.rs"]
mod inbox;
pub(super) use inbox::*;
#[path = "values/kind_order.rs"]
mod kind_order;
pub(super) use kind_order::*;
#[path = "values/milestone.rs"]
mod milestone;
pub(super) use milestone::*;
#[path = "values/outcome.rs"]
mod outcome;
pub(super) use outcome::*;
#[path = "values/part.rs"]
mod part;
pub(super) use part::*;
#[path = "values/person_task.rs"]
mod person_task;
pub(super) use person_task::*;
#[path = "values/phase_counts.rs"]
mod phase_counts;
pub(super) use phase_counts::*;
#[path = "values/project.rs"]
mod project;
pub(super) use project::*;
#[path = "values/proposal.rs"]
mod proposal;
pub(super) use proposal::*;
#[path = "values/tree.rs"]
mod tree;
pub(super) use tree::*;
#[path = "values/wake.rs"]
mod wake;
pub(super) use wake::*;
#[path = "values/policy_edit.rs"]
mod policy_edit;
pub(super) use policy_edit::*;
#[path = "values/proposed_task.rs"]
mod proposed_task;
pub(super) use proposed_task::*;
#[path = "values/queries.rs"]
mod queries;
pub(super) use queries::*;
#[path = "values/reads.rs"]
mod reads;
pub(super) use reads::*;
#[path = "values/recurring.rs"]
mod recurring;
pub(super) use recurring::*;
#[path = "values/refusals.rs"]
mod refusals;
pub(super) use refusals::*;
#[path = "values/result_details.rs"]
mod result_details;
pub(super) use result_details::*;
#[path = "values/run_facts.rs"]
mod run_facts;
pub(super) use run_facts::*;
#[path = "values/settings.rs"]
mod settings;
pub(super) use settings::*;
#[path = "values/specification.rs"]
mod specification;
pub(super) use specification::*;
#[path = "values/task_hold.rs"]
mod task_hold;
pub(super) use task_hold::*;
#[path = "values/task_result.rs"]
mod task_result;
pub(super) use task_result::*;
#[path = "values/turn.rs"]
mod turn;
pub(super) use turn::*;
#[path = "values/watches.rs"]
mod watches;
pub(super) use watches::*;
