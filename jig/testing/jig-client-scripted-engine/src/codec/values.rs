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
#[path = "values/historical_pages.rs"]
mod historical_pages;
pub(super) use historical_pages::*;
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
#[path = "values/owner_accounts.rs"]
mod owner_accounts;
pub(super) use owner_accounts::*;
#[path = "values/owner_activity.rs"]
mod owner_activity;
pub(super) use owner_activity::*;
#[path = "values/owner_amendment.rs"]
mod owner_amendment;
pub(super) use owner_amendment::*;
#[path = "values/owner_authority.rs"]
mod owner_authority;
pub(super) use owner_authority::*;
#[path = "values/owner_budget.rs"]
mod owner_budget;
pub(super) use owner_budget::*;
#[path = "values/owner_chip.rs"]
mod owner_chip;
pub(super) use owner_chip::*;
#[path = "values/owner_deployment.rs"]
mod owner_deployment;
pub(super) use owner_deployment::*;
#[path = "values/owner_escalation.rs"]
mod owner_escalation;
pub(super) use owner_escalation::*;
#[path = "values/owner_historical.rs"]
mod owner_historical;
pub(super) use owner_historical::*;
#[path = "values/owner_history.rs"]
mod owner_history;
pub(super) use owner_history::*;
#[path = "values/owner_inbox.rs"]
mod owner_inbox;
pub(super) use owner_inbox::*;
#[path = "values/owner_kind_order.rs"]
mod owner_kind_order;
pub(super) use owner_kind_order::*;
#[path = "values/owner_milestone.rs"]
mod owner_milestone;
pub(super) use owner_milestone::*;
#[path = "values/owner_outcome.rs"]
mod owner_outcome;
pub(super) use owner_outcome::*;
#[path = "values/owner_part.rs"]
mod owner_part;
pub(super) use owner_part::*;
#[path = "values/owner_person_task.rs"]
mod owner_person_task;
pub(super) use owner_person_task::*;
#[path = "values/owner_phase_counts.rs"]
mod owner_phase_counts;
pub(super) use owner_phase_counts::*;
#[path = "values/owner_project.rs"]
mod owner_project;
pub(super) use owner_project::*;
#[path = "values/owner_proposal.rs"]
mod owner_proposal;
pub(super) use owner_proposal::*;
#[path = "values/owner_tree.rs"]
mod owner_tree;
pub(super) use owner_tree::*;
#[path = "values/owner_wake.rs"]
mod owner_wake;
pub(super) use owner_wake::*;
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
