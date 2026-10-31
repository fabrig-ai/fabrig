//! Immutable semantic history (client/domain/objects.md, 11; domain/tasks.md, 7;
//! view.md, 7 and 10). Stored overrides are not historical effective defaults.
use crate::JigBindings;
use crate::words::Words;
use crate::words::history_words::HistoryPhrase as P;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use crate::Config;
use jig_client::contract::{
    OwnerHistoryChange, OwnerHistoryHolder, OwnerHistoryLine, OwnerMessageProposalKind, OwnerProposalHistory,
    OwnerProposalHistoryState, OwnerRetainedAction, OwnerRetainedAmendment, PersonAddress,
};

/// Conservative retained-proposal bound, including the whole history envelope's spare allowance.
#[must_use]
pub fn proposal_max_nodes(limits: &Config, client: &jig_client::DomainLimits) -> Option<u32> {
    max_nodes(limits, client)
}
/// Finite whole-node envelope reused by bounded history pages.
#[must_use]
pub fn max_nodes(limits: &Config, client: &jig_client::DomainLimits) -> Option<u32> {
    let amendment = crate::tree::fragments::amendment::max_nodes(limits, client)?;
    let authority = crate::tree::fragments::authority::document_nodes(client.rows, client.segments)?;
    let batch = client.rows.checked_mul(crate::sizes::sizes::proposed_task_nodes(limits, client)?)?;
    let delta = crate::tree::fragments::specification::nodes(limits, client)?
        .checked_add(authority)?
        .checked_add(skein_ui::markdown::max_nodes(client.text)?.checked_mul(2)?)?
        .checked_add(client.rows)?
        .checked_add(crate::tree::fragments::wake::policy_nodes())?
        .checked_add(30)?;
    let effect =
        crate::tree::fragments::proposed_task::part_nodes(limits, client)?.checked_add(authority)?.checked_add(8)?;
    100_u32.checked_add(amendment.checked_mul(2)?)?.checked_add(batch.max(delta).max(effect))
}
fn saying(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    let value =
        words.book().say(words.language(), phrase.id(), holes).expect("checked immutable history sayings and holes");
    b.text(Run::Plain, &value.text);
}
fn change(b: &mut Builder, value: OwnerHistoryChange, words: &Words) {
    let phrase = match value {
        OwnerHistoryChange::Cancelled => P::Cancelled,
        OwnerHistoryChange::Released => P::Released,
        OwnerHistoryChange::Amended => P::Amended,
        OwnerHistoryChange::Moved => P::Moved,
        OwnerHistoryChange::Proposed => P::Proposed,
        OwnerHistoryChange::ProposalPassed => P::Passed,
        OwnerHistoryChange::ProposalAccepted => P::Accepted,
        OwnerHistoryChange::ProposalRejected => P::Rejected,
        OwnerHistoryChange::ProposalWithdrawn => P::Withdrawn,
    };
    saying(b, words, phrase, &[]);
}
pub(crate) fn holder(b: &mut Builder, value: OwnerHistoryHolder, words: &Words) {
    match value {
        OwnerHistoryHolder::Task(number) => saying(b, words, P::HolderTask, &[Hole::Number(number)]),
        OwnerHistoryHolder::Person(number) => saying(b, words, P::HolderPerson, &[Hole::Number(number)]),
        OwnerHistoryHolder::Policy { project, kind } => {
            saying(b, words, P::HolderPolicy, &[Hole::Number(u64::from(project))]);
            let phrase = match kind {
                OwnerMessageProposalKind::Settings => return,
                OwnerMessageProposalKind::Batch => P::Batch,
                OwnerMessageProposalKind::Effect => P::Effect,
                OwnerMessageProposalKind::Amend => P::Amend,
                OwnerMessageProposalKind::Widen => P::Widen,
                OwnerMessageProposalKind::Release => P::Release,
            };
            saying(b, words, phrase, &[]);
        }
    }
}
fn retained_amendment(b: &mut Builder, value: &OwnerRetainedAmendment, words: &Words, text_limit: u32, rows: u32) {
    match value.addressed {
        Some(address) => {
            saying(b, words, P::AddressRevision, &[Hole::Number(address.revision.value)]);
            match address.address {
                PersonAddress::Party(number) => saying(b, words, P::Party, &[Hole::Number(number)]),
                PersonAddress::Role(number) => saying(b, words, P::Role, &[Hole::Number(u64::from(number))]),
            }
        }
        None => saying(b, words, P::AddressAbsent, &[]),
    }
    match &value.instructions {
        Some(value) => {
            saying(b, words, P::Instructions, &[]);
            crate::tree::fragments::instructions::document(b, Some(value), words);
        }
        None => saying(b, words, P::InstructionsAbsent, &[]),
    }
    match value.release {
        Some(revision) => saying(b, words, P::HoldRevision, &[Hole::Number(revision)]),
        None => saying(b, words, P::ReleaseAbsent, &[]),
    }
    match &value.spec {
        Some(value) => {
            saying(b, words, P::Spec, &[]);
            crate::tree::fragments::specification::document(b, value, words);
        }
        None => saying(b, words, P::SpecAbsent, &[]),
    }
    match &value.authority {
        Some(value) => {
            saying(b, words, P::Authority, &[]);
            crate::tree::fragments::authority::document_words(b, value, words, text_limit, rows);
        }
        None => saying(b, words, P::AuthorityAbsent, &[]),
    }
    match &value.wake {
        Some(value) => {
            saying(b, words, P::Wake, &[]);
            crate::tree::fragments::wake::policy(b, value, words.book(), words.language());
        }
        None => saying(b, words, P::WakeAbsent, &[]),
    }
    match &value.dependencies {
        Some(values) => {
            saying(b, words, P::Dependencies, &[]);
            for value in values {
                saying(b, words, P::Dependency, &[Hole::Number(*value)]);
            }
        }
        None => saying(b, words, P::DependenciesAbsent, &[]),
    }
    if value.cut {
        saying(b, words, P::DependenciesCut, &[]);
    }
    saying(b, words, P::Reason, &[]);
    crate::tree::fragments::document_text::markdown(b, &value.reason, words);
}
fn action(b: &mut Builder, value: &OwnerRetainedAction, words: &Words, bindings: &mut JigBindings, rows: u32) {
    let text_limit = bindings.text_limit();
    match value {
        OwnerRetainedAction::Settings(_) => {}
        OwnerRetainedAction::Batch(tasks) => {
            saying(b, words, P::Batch, &[]);
            for task in tasks {
                crate::tree::cards::proposal::proposed_task(b, task, words, bindings, rows);
            }
        }
        OwnerRetainedAction::Effect { part, authority, attempt, completion, position, purpose } => {
            saying(b, words, P::Effect, &[]);
            crate::tree::fragments::proposed_task::part(b, part, words, bindings);
            crate::tree::fragments::authority::document_words(b, authority, words, text_limit, rows);
            for (phrase, number) in [
                (P::Attempt, *attempt),
                (P::Completion, u64::from(*completion)),
                (P::Position, u64::from(*position)),
                (P::Purpose, *purpose),
            ] {
                saying(b, words, phrase, &[Hole::Number(number)]);
            }
        }
        OwnerRetainedAction::ProcedureEffect { part, authority, step, purpose } => {
            saying(b, words, P::Effect, &[]);
            crate::tree::fragments::proposed_task::part(b, part, words, bindings);
            crate::tree::fragments::authority::document_words(b, authority, words, text_limit, rows);
            saying(b, words, P::ProcedureStep, &[Hole::Number(*step)]);
            saying(b, words, P::Purpose, &[Hole::Number(*purpose)]);
        }
        OwnerRetainedAction::Amend { task, change } => {
            saying(b, words, P::Amend, &[]);
            saying(b, words, P::Task, &[Hole::Number(*task)]);
            retained_amendment(b, change, words, text_limit, rows);
        }
        OwnerRetainedAction::Widen { task, authority } => {
            saying(b, words, P::Widen, &[]);
            saying(b, words, P::Task, &[Hole::Number(*task)]);
            crate::tree::fragments::authority::document_words(b, authority, words, text_limit, rows);
        }
        OwnerRetainedAction::Release { task } => {
            saying(b, words, P::Release, &[]);
            saying(b, words, P::Task, &[Hole::Number(*task)]);
        }
    }
}
/// Render only supplied immutable facts, borrowing the normal checked language.
pub fn render(b: &mut Builder, value: &OwnerHistoryLine, words: &Words, bindings: &mut JigBindings, rows: u32) {
    let text_limit = bindings.text_limit();
    b.open(Role::Region);
    let name = words.book().say(words.language(), P::History.id(), &[]).expect("history heading checked");
    b.name(&name.text);
    saying(b, words, P::Task, &[Hole::Number(value.task)]);
    saying(b, words, P::Revision, &[Hole::Number(value.revision)]);
    crate::tree::fragments::result::requester(b, value.by, words);
    change(b, value.change, words);
    saying(b, words, P::Reason, &[]);
    crate::tree::fragments::document_text::markdown(b, &value.reason, words);
    match value.at {
        Some(at) => saying(b, words, P::At, &[Hole::Wall(words.adjusted_wall(at))]),
        None => saying(b, words, P::AtAbsent, &[]),
    }
    match &value.amendment {
        Some((before, after)) => {
            saying(b, words, P::Stored, &[]);
            saying(b, words, P::Before, &[]);
            crate::tree::fragments::amendment::render(b, before, words, text_limit, rows);
            saying(b, words, P::After, &[]);
            crate::tree::fragments::amendment::render(b, after, words, text_limit, rows);
        }
        None => saying(b, words, P::SnapshotsAbsent, &[]),
    }
    if let Some(proposal) = &value.proposal {
        self::proposal(b, proposal, words, bindings, rows);
    }
    b.close();
}

/// Render supplied retained proposal facts inline, without current controls or outer history metadata.
pub fn proposal(
    b: &mut Builder,
    proposal: &OwnerProposalHistory,
    words: &Words,
    bindings: &mut JigBindings,
    rows: u32,
) {
    saying(b, words, P::Proposal, &[Hole::Number(proposal.number)]);
    saying(b, words, P::Proposer, &[Hole::Number(proposal.proposer)]);
    saying(b, words, if proposal.as_holder { P::AsHolder } else { P::NotAsHolder }, &[]);
    action(b, &proposal.action, words, bindings, rows);
    match &proposal.state {
        OwnerProposalHistoryState::Pending { holder: value, since } => {
            saying(b, words, P::Pending, &[]);
            holder(b, *value, words);
            saying(b, words, P::Since, &[Hole::Wall(words.adjusted_wall(*since))]);
        }
        OwnerProposalHistoryState::Accepted { by } => {
            saying(b, words, P::Accepted, &[]);
            crate::tree::fragments::result::requester(b, *by, words);
        }
        OwnerProposalHistoryState::Rejected { by, reason } => {
            saying(b, words, P::Rejected, &[]);
            crate::tree::fragments::result::requester(b, *by, words);
            crate::tree::fragments::document_text::markdown(b, reason, words);
        }
        OwnerProposalHistoryState::Withdrawn => saying(b, words, P::Withdrawn, &[]),
    }
}
