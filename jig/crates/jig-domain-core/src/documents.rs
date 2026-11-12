//! Typed live documents, made on demand from the core's committed vocabulary
//! (`client/domain/contract/documents.md` and `client/domain/contract/README.md`, section 5; `domain/engine/views.md`).
//! No document is retained here, and this module neither delivers nor decides.
//! `make` authenticates visibility from a supplied party number and copies only
//! bounded text and lists. Its caller holds the answer until durable.
//!
//! Frame and inbox counts include every live need and bounded unread updates.
//! Read positions move only on a keyed mark. Objects carry actor-specific offers
//! from the shared admission judges. Run writing status,
//! provisional text and calls belong to the watch stream; committed turns never
//! enter its text. Application subjects belong to their connectors.
//!
//! Store-only rows use public bounded row constructors without restoring them.
//! Chips retain owner phase clocks, readiness, retries, resource references and
//! endings. Core-aware constructors also join fenced best-effort observations;
//! their engine receipt clocks are distinct from placement/adoption clocks.
//! Retained failure and dependency explanations use exact task/activity reads.
//! Closing chips name the connectors they wait on (domain/engine/views.md, 3). Effect
//! descriptions, prices and counts remain in each connector's panel. Recurring wall
//! schedules still require their schedule owner. `make_with_clock` projects
//! host grace from monotonic deadlines; account deadlines carry their source wall.

#![expect(
    hidden_glob_reexports,
    reason = "the core keeps its constructor modules private while re-exporting their vocabulary"
)]

pub use crate::below::documents::*;

mod accounts;
mod catalogue;
mod chip_phase;
mod deployment;
mod inbox_deployment;
mod inbox_selection;
mod live_delegates;
mod objects;
pub(crate) use objects::run_proposal;
mod offers;
mod programs;
mod project_note;
mod settings;
mod settings_review;
mod small;
mod task_lines;
pub mod task_query;
pub(crate) mod task_tree;
mod task_wait;

pub use deployment::{DocumentClock, make_with_clock};
pub(crate) use inbox_selection::retained as inbox_selection_from_ancestry;
pub use objects::{
    EndedDelegate, ended_delegate_from_row, history_from_row, milestone_from_row, person_from_row, result_from_row,
    retained_action,
};
pub use project_note::note_from_entry;
pub use settings::inventory as settings_inventory;
pub use task_lines::task_line;
pub use task_query::lines;
pub use task_tree::task_from_row;

use crate::{Core, Limits};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{List, Wall};

/// Make the current visible document without changing state or emitting effects.
/// Clamp its window to the engine's rows (`client/domain/contract/README.md`, 5.2).
pub fn make(
    core: &Core,
    limits: &Limits,
    party: u64,
    subject: &Subject,
    window: &Window,
    decision: Wall,
) -> Result<Document, Missing> {
    if document_bound(&limits.documents).is_none() {
        return Err(Missing::TooLarge);
    }
    let window = &Window { rows: window.rows.min(limits.documents.rows), ..*window };
    core.people.identity(party).ok_or(Missing::Unknown)?;
    match subject {
        Subject::Application { .. } => Err(Missing::Unknown),
        Subject::Frame { party: owner } => {
            if *owner != party {
                return Err(Missing::NotAllowed);
            }
            let frame = small::frame(core, limits, party)?;
            Ok(Document::Frame(frame))
        }
        Subject::Chips { tasks: named } => {
            if named.len() > limits.documents.chips {
                return Err(Missing::TooLarge);
            }
            let mut chips = List::with_capacity(named.len());
            for number in named {
                chips.push(small::chip(core, limits, party, *number)?).expect("named chips fit");
            }
            Ok(Document::Chips(chips))
        }
        Subject::Run { task } => {
            let run = small::run(core, limits, party, *task)?;
            Ok(Document::Run(run))
        }
        Subject::Task { task } => {
            let task = task_tree::task(core, limits, party, *task, decision)?;
            Ok(Document::Task(Box::new(task)))
        }
        Subject::Tree { task } => {
            let tree = task_tree::tree(core, limits, party, *task, window)?;
            Ok(Document::Tree(tree))
        }
        Subject::Query { query } => Ok(Document::Lines(lines(core, limits, party, query, window)?)),
        Subject::Inbox { party: owner } => {
            if *owner != party {
                return Err(Missing::NotAllowed);
            }
            Ok(Document::Inbox(inbox_deployment::inbox(core, limits, party, window)?))
        }
        Subject::Accounts { party: owner } => {
            Ok(Document::Accounts(Box::new(accounts::make(core, limits, party, *owner, window)?)))
        }
        Subject::Settings => Ok(Document::Settings(Box::new(settings::make(core, party)?))),
        Subject::Deployment => Ok(Document::Deployment(Box::new(deployment::deployment(core, limits, party, None)?))),
        Subject::Proposal { proposer, number } => {
            Ok(Document::Proposal(Box::new(objects::proposal(core, limits, party, *proposer, *number, decision)?)))
        }
        Subject::Escalation { task } => {
            Ok(Document::Escalation(objects::escalation(core, limits, party, *task, decision)?))
        }
        Subject::Question { task, message } => {
            Ok(Document::Question(objects::question(core, limits, party, *task, *message, decision)?))
        }
        Subject::PersonTask { task } => {
            Ok(Document::PersonTask(Box::new(objects::person(core, limits, party, *task, decision)?)))
        }
        Subject::Result { task } => {
            Ok(Document::Result(Box::new(objects::result(core, limits, party, *task, decision)?)))
        }
        Subject::Project { project } => {
            Ok(Document::Project(Box::new(project_note::project(core, limits, party, *project, decision)?)))
        }
        Subject::Note { .. } => Err(Missing::Ended),
    }
}

/// Join volatile live observations to a loaded owner row. The reader checks
/// project visibility first; this does not restore or change the row.
pub fn chip_from_row_with_core(core: &Core, limits: &Limits, row: &tasks::TaskRecord) -> Result<Chip, Missing> {
    small::chip_with_core(core, limits, 0, row)
}

/// Funding numbers copied from their retained owner row.
#[must_use]
pub fn budget_from_row(row: &tasks::TaskRecord) -> BudgetLine {
    task_tree::budget(row)
}

/// Complete bounded authority facts used when judging draft controls.
pub(crate) fn act_authority(limits: &Limits, value: &tasks::Authority) -> Result<AuthorityLine, Missing> {
    task_tree::authority(limits, &crate::translate::tasks_authority(value))
}
