//! The forge's answers to `see` addresses (temper/docs/design/domain/agent/ask-temper.md, 1).
use crate::{RepositoryIdentity, SeeLanding, SeeRepository};
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::{List, Writer};
use temper_engine_domain_forge::{Domain, Repository, RepositoryGates};
use temper_engine_domain_store::CallAnswer;

#[must_use]
pub fn landing(forge: &Domain, project: u32) -> SeeLanding {
    let readable = forge.readable_repositories(project);
    let mut repositories = List::with_capacity(u32::try_from(readable.len()).expect("admitted repositories"));
    for repository in &readable {
        repositories.push(repository_answer(forge, project, repository)).expect("admitted repositories");
    }
    SeeLanding {
        by: forge.landing_way(project),
        merge_job: match forge.merge_job(project) {
            Some(job) => Some(job.into()),
            None => None,
        },
        after: forge.after_landing(project),
        repositories: repositories.into_boxed(),
    }
}

#[must_use]
pub fn repository_answer(forge: &Domain, project: u32, repository: &Repository) -> SeeRepository {
    let length = repository
        .owner
        .len()
        .checked_add(repository.name.len())
        .expect("repository name bound")
        .checked_add(1)
        .expect("repository slash");
    let mut name = Writer::new(length);
    name.put(&repository.owner).expect("repository owner");
    name.put(b"/").expect("repository slash");
    name.put(&repository.name).expect("repository name");
    let checks = match forge.repository_gates(project, repository.provider) {
        Some(checks) => checks.clone(),
        None => RepositoryGates { repository: repository.provider, gates: Box::new([]), tiers: Box::new([]) },
    };
    SeeRepository { name: name.finish(), checks }
}

#[must_use]
pub fn read_head(answer: &CallAnswer) -> Option<api::Commit> {
    match answer {
        CallAnswer::ForgeRead(answer) => match answer.as_ref() {
            Ok(answer) => match answer {
                api::Answer::Commit(head) => Some(*head),
                api::Answer::Located { .. }
                | api::Answer::Items { .. }
                | api::Answer::Item { .. }
                | api::Answer::Pull(_)
                | api::Answer::Reviews { .. }
                | api::Answer::Statuses { .. }
                | api::Answer::Remarks { .. }
                | api::Answer::Branches(_)
                | api::Answer::PullFiles { .. }
                | api::Answer::Compare { .. }
                | api::Answer::Checks(_)
                | api::Answer::JobState { .. }
                | api::Answer::Job { .. }
                | api::Answer::File { .. }
                | api::Answer::Directory { .. }
                | api::Answer::Protection(_)
                | api::Answer::Settings(_)
                | api::Answer::Collaborators { .. }
                | api::Answer::Permission(_)
                | api::Answer::Created(_)
                | api::Answer::Commented(_)
                | api::Answer::Reviewed(_)
                | api::Answer::Merged(_)
                | api::Answer::Branch(_)
                | api::Answer::Done
                | api::Answer::Run(_)
                | api::Answer::Runs { .. }
                | api::Answer::Commits { .. }
                | api::Answer::Tree { .. }
                | api::Answer::Queued => None,
            },
            Err(_) => None,
        },
        CallAnswer::Follow(_)
        | CallAnswer::Finding(_)
        | CallAnswer::Red(_)
        | CallAnswer::Decided { .. }
        | CallAnswer::TaskRead(_)
        | CallAnswer::SettingsRefused(_)
        | CallAnswer::HoldOwner { .. }
        | CallAnswer::Again { .. }
        | CallAnswer::PassedUp { .. }
        | CallAnswer::Revised { .. }
        | CallAnswer::Amended { .. }
        | CallAnswer::Moved { .. }
        | CallAnswer::ForgeEffect { .. }
        | CallAnswer::ForgeEffectRefused(_)
        | CallAnswer::ForgeEffectDenied { .. }
        | CallAnswer::ToolDenied { .. }
        | CallAnswer::EscalationDecided { .. }
        | CallAnswer::EscalationRefused(_)
        | CallAnswer::Proposed { .. }
        | CallAnswer::ProposalDecided { .. }
        | CallAnswer::ProposalRefused(_)
        | CallAnswer::Controlled
        | CallAnswer::ControlRefused(_)
        | CallAnswer::EffectRefused(_)
        | CallAnswer::ControlDenied { .. }
        | CallAnswer::Sent { .. }
        | CallAnswer::Introduced
        | CallAnswer::MessageRefused(_)
        | CallAnswer::Subscribed { .. }
        | CallAnswer::Unsubscribed
        | CallAnswer::SubscriptionRefused(_)
        | CallAnswer::ForgeSubscriptionRefused { .. }
        | CallAnswer::Delegated(_)
        | CallAnswer::DelegationDenied { .. }
        | CallAnswer::DelegationRefused(_)
        | CallAnswer::ReadRepositoryRefused { .. }
        | CallAnswer::SubscriptionRepositoryRefused { .. }
        | CallAnswer::ChangeRefused { .. }
        | CallAnswer::WordsRecorded { .. }
        | CallAnswer::NoteWritten { .. }
        | CallAnswer::NoteDeleted { .. }
        | CallAnswer::NoteRecalled { .. }
        | CallAnswer::NoteRefused(_)
        | CallAnswer::Document(_)
        | CallAnswer::Remember(_)
        | CallAnswer::Parts(_)
        | CallAnswer::Unavailable => None,
    }
}

pub(crate) fn repositories(forge: &Domain, project: u32) -> Box<[RepositoryIdentity]> {
    let repositories = forge.readable_repositories(project);
    let mut identities = List::with_capacity(u32::try_from(repositories.len()).expect("bounded repositories"));
    for repository in &repositories {
        identities
            .push(RepositoryIdentity {
                host: repository.host.clone(),
                owner: repository.owner.clone(),
                name: repository.name.clone(),
                landing: repository.landing.clone(),
            })
            .expect("one identity per readable repository");
    }
    identities.into_boxed()
}
