//! Read admission and retained answer bounds (jig/docs/design/domain/engine/README.md, 3).
pub mod ended;
pub mod indexed;
pub mod rows;
pub mod run_task;
pub mod settings;
pub mod task_text;
pub mod vocabulary;

use crate::{Limits, documents};
use alloc::boxed::Box;
use core::mem::size_of;
pub use rows::row_key;
pub use settings::{ProposalRead, SettingsAnswer, SettingsRead};
use skein_lib::List;
pub use task_text::{TaskText, TaskTextField};
pub use vocabulary::*;

/// Which physical rows a read scope loads (protocol/store/README.md, 4.3, and protocol/store/loads.md, 3).
/// Cursor admission is separate: notes lines are joined to entry rows before paging.
#[must_use]
pub fn admits(party: u64, scope: &Scope, key: &crate::Key) -> bool {
    use crate::Key;
    use jig_domain_notes as notes;
    use jig_domain_people::Proposer;
    use jig_domain_tasks::{Funder, Key as TaskKey};
    match scope {
        Scope::SettingsHistory { first, last, newest, .. } => match key {
            Key::Core(crate::CoreKey::Settings(crate::settings::Key::Held(n))) => n >= first && n <= last,
            Key::Core(crate::CoreKey::Settings(crate::settings::Key::History(n))) => {
                n != newest && n >= first && n <= last
            }
            Key::Core(_) | Key::Tasks(_) | Key::People(_) | Key::Notes(_) | Key::Accounts(_) => false,
        },
        Scope::TaskText { task, .. } | Scope::Task { task } | Scope::PersonTask { task } | Scope::Result { task } => {
            key == &Key::Tasks(TaskKey::Live(*task)) || key == &Key::Tasks(TaskKey::Ended(*task))
        }
        Scope::Reply { task, activity } => key == &Key::Tasks(TaskKey::Activity { task: *task, position: *activity }),
        Scope::History { task, subtree: false } => match key {
            Key::Tasks(TaskKey::History { task: owner, .. } | TaskKey::Milestone { task: owner, .. }) => owner == task,
            Key::Tasks(_) | Key::Core(_) | Key::People(_) | Key::Notes(_) | Key::Accounts(_) => false,
        },
        Scope::Proposal { proposer: Proposer::Party(owner), proposal } => {
            *owner == party
                && (key == &Key::Tasks(TaskKey::PersonProposal(*proposal))
                    || key == &Key::Tasks(TaskKey::DecidedProposal(*proposal)))
        }
        Scope::EndedDelegates { task } => match key {
            Key::Tasks(TaskKey::EndedDelegate { requester, .. }) => requester == task,
            Key::Tasks(_) | Key::Core(_) | Key::People(_) | Key::Notes(_) | Key::Accounts(_) => false,
        },
        Scope::Spend { project } => match key {
            Key::Tasks(
                TaskKey::Ledger(Funder::Period { project: owner, .. })
                | TaskKey::ClosedLedger(Funder::Period { project: owner, .. }),
            ) => *project == jig_domain_people::DEPLOYMENT || owner == project,
            Key::Tasks(_) | Key::Core(_) | Key::People(_) | Key::Notes(_) | Key::Accounts(_) => false,
        },
        Scope::Notes { project } => match key {
            Key::Notes(notes::Key::Line { scope, .. }) => match scope {
                notes::Scope::Goal { project: owner, .. } => owner == project,
                notes::Scope::Party { .. }
                | notes::Scope::Deployment
                | notes::Scope::Project { .. }
                | notes::Scope::Resources { .. } => false,
            },
            Key::Notes(_) | Key::Tasks(_) | Key::Core(_) | Key::People(_) | Key::Accounts(_) => false,
        },
        Scope::ScopedNotes { scope: wanted, .. } => match key {
            Key::Notes(notes::Key::Line { scope, .. }) => scope == wanted,
            Key::Notes(_) | Key::Tasks(_) | Key::Core(_) | Key::People(_) | Key::Accounts(_) => false,
        },
        Scope::Inbox { group: Group::Updates | Group::Results | Group::Replies, .. } => match key {
            Key::Tasks(TaskKey::Update { party: owner, .. }) => *owner == party,
            Key::Tasks(_) | Key::Core(_) | Key::People(_) | Key::Notes(_) | Key::Accounts(_) => false,
        },
        Scope::EndedTasks
        | Scope::TaskRows { .. }
        | Scope::Tasks { .. }
        | Scope::Proposal { proposer: Proposer::Task(_), .. }
        | Scope::Escalation { .. }
        | Scope::Question { .. }
        | Scope::Turn { .. }
        | Scope::History { subtree: true, .. }
        | Scope::Activity { .. }
        | Scope::Inbox {
            group: Group::Needs | Group::Proposals | Group::Escalations | Group::Questions | Group::PersonTasks,
            ..
        } => false,
    }
}
/// Maximum answer payload retained in a root's held-output slot.
#[must_use]
pub fn answer_bound(limits: &Limits, kinds: &[crate::settings::Kind]) -> Option<u64> {
    let document = documents::document_with_context_bound(limits, kinds)?;
    // Run settings answers own the same documents, in an inline sum carrier.
    u64::try_from(size_of::<Answer>().max(size_of::<SettingsAnswer>()).max(size_of::<ProposalRead>()))
        .ok()?
        .checked_add(
            document
                .checked_mul(u64::from(limits.documents.rows).checked_add(3)?)?
                .checked_add(limits.load_bytes.checked_mul(2)?)?
                .checked_add(List::<PageRow>::price(limits.documents.rows)?)?
                .checked_add(u64::try_from(size_of::<Page>()).ok()?)?
                .checked_add(query_bound(limits)?)?
                .checked_add(run_task::answer_bound(limits)?)?
                .checked_add(settings::load_bytes(limits, limits.documents.rows)?.checked_mul(2)?)?,
        )
}
// Resource pattern segments and literal bytes, plus one description query.
#[must_use]
pub fn query_bound(limits: &Limits) -> Option<u64> {
    u64::from(limits.notes.pattern_bytes)
        .checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?.checked_add(1)?)?
        .checked_add(u64::from(limits.notes.description_bytes))
}
