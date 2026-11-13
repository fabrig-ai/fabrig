//! Forge address reads (temper/docs/design/domain/agent/ask-temper.md, 1).
use super::{Decision, Domain, Env, FlightRender, Limits, ReplyTo, SeeAnswer, SeeFlight, SeeReading};
use crate::{CallAnswer, CallKey, SeeLanding, SeeRepository};
use alloc::boxed::Box;
use skein_forge::api::{self, Read};
use skein_lib::{List, Writer};
use temper_engine_domain_forge::{Repository, RepositoryGates};

pub(super) fn landing(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    key: CallKey,
    line: Box<[u8]>,
    from: u32,
) {
    let project = project(domain, key);
    let readable = domain.forge.readable_repositories(project);
    let mut repositories = List::with_capacity(u32::try_from(readable.len()).expect("admitted repositories"));
    for repository in &readable {
        repositories.push(repository_answer(domain, project, repository)).expect("admitted repositories");
    }
    let answer = SeeLanding {
        by: domain.forge.landing_way(project),
        merge_job: match domain.forge.merge_job(project) {
            Some(job) => Some(job.into()),
            None => None,
        },
        after: domain.forge.after_landing(project),
        repositories: repositories.into_boxed(),
    };
    super::deliver(env, decision, to, line, from, SeeAnswer::Landing(Box::new(answer)));
}

#[expect(clippy::too_many_arguments, reason = "the forge read carries its call and repository")]
pub(super) fn gates(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    key: CallKey,
    line: Box<[u8]>,
    from: u32,
    repository: Box<[u8]>,
) {
    let project = project(domain, key);
    let answer = match super::super::forge_route::named_repository(domain, project, Some(&repository)) {
        Ok(adopted) => SeeAnswer::Gates(Box::new(repository_answer(domain, project, &adopted))),
        Err(_) => refused_repository(domain, project, repository),
    };
    super::deliver(env, decision, to, line, from, answer);
}

#[expect(clippy::too_many_arguments, reason = "the file read carries its call and address members")]
#[expect(clippy::boxed_local, reason = "the file address owns its path, recovered from the flight's line")]
pub(super) fn file(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    key: CallKey,
    line: Box<[u8]>,
    from: u32,
    repository: Box<[u8]>,
    _path: Box<[u8]>,
    branch: Option<Box<[u8]>>,
) {
    let project = project(domain, key);
    let Ok(adopted) = super::super::forge_route::named_repository(domain, project, Some(&repository)) else {
        let answer = refused_repository(domain, project, repository);
        super::deliver(env, decision, to, line, from, answer);
        return;
    };
    let branch = match branch {
        Some(branch) => branch,
        None => adopted.settings.default_branch,
    };
    let right = to.into_token();
    domain.call_renders.get_mut(&right).expect("decoded see flight").render = FlightRender::Settings {
        see: Some(Box::new(SeeFlight { line, reading: SeeReading::Head })),
        from,
        requested: None,
        asked: None,
    };
    Domain::read_forge_call(domain, &env.limits, ReplyTo::new(right), key, adopted.provider, Read::Branch { branch });
}

fn project(domain: &Domain, key: CallKey) -> u32 {
    domain.core.tasks.delegation(key.task).expect("authorized chat delegation").project
}

fn repository_answer(domain: &Domain, project: u32, repository: &Repository) -> SeeRepository {
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
    let checks = match domain.forge.repository_gates(project, repository.provider) {
        Some(checks) => checks.clone(),
        None => RepositoryGates { repository: repository.provider, gates: Box::new([]), tiers: Box::new([]) },
    };
    SeeRepository { name: name.finish(), checks }
}

fn refused_repository(domain: &Domain, project: u32, named: Box<[u8]>) -> SeeAnswer {
    SeeAnswer::Call {
        tool: b"read_file",
        answer: Box::new(CallAnswer::ReadRepositoryRefused {
            named: Some(named),
            repositories: super::super::forge_route::read_repositories(domain, project),
        }),
    }
}

pub(crate) fn answered(domain: &mut Domain, to: ReplyTo, answer: CallAnswer) {
    let right = to.into_token();
    let flight = domain.call_renders.get(&right).expect("see forge flight");
    let FlightRender::Settings { see: Some(see), from, .. } = &flight.render else { unreachable!("see forge context") };
    let (head, tool) = match see.reading {
        SeeReading::Head => (read_head(&answer), b"read_branch".as_slice()),
        SeeReading::File => (None, b"read_file".as_slice()),
        SeeReading::Settings | SeeReading::Proposal | SeeReading::Person(_) => {
            unreachable!("forge answer to a forge read")
        }
    };
    if let Some(head) = head {
        let key = flight.key.expect("decoded forge call");
        let project = project(domain, key);
        let (repository, path) = match super::address(&see.line, project) {
            super::Address::File { repository, path, .. } => (repository, path),
            super::Address::Ways
            | super::Address::Inventory
            | super::Address::Setting(_)
            | super::Address::Landing
            | super::Address::Gates { .. }
            | super::Address::Revisions
            | super::Address::Revision { .. }
            | super::Address::Proposal { .. }
            | super::Address::Page { .. }
            | super::Address::Search { .. }
            | super::Address::Me
            | super::Address::Project
            | super::Address::Goals { .. }
            | super::Address::Tasks
            | super::Address::Task { .. }
            | super::Address::Tree { .. }
            | super::Address::Activity { .. }
            | super::Address::Turn { .. }
            | super::Address::Change { .. }
            | super::Address::Changes
            | super::Address::Inbox
            | super::Address::Unknown => unreachable!("file flight keeps its address"),
        };
        let adopted = super::super::forge_route::named_repository(domain, project, Some(&repository));
        match adopted {
            Ok(adopted) => {
                let flight = domain.call_renders.get_mut(&right).expect("see file flight");
                let FlightRender::Settings { see: Some(see), .. } = &mut flight.render else {
                    unreachable!("see file context")
                };
                see.reading = SeeReading::File;
                let limits = domain.limits;
                let read = Read::File { head, path, max_bytes: limits.forge.client.answer_bytes };
                Domain::read_forge_call(domain, &limits, ReplyTo::new(right), key, adopted.provider, read);
            }
            Err(_) => {
                let line = see.line.clone();
                let from = *from;
                let answer = refused_repository(domain, project, repository);
                deliver(domain, ReplyTo::new(right), line, from, answer);
            }
        }
        return;
    }
    let line = see.line.clone();
    let from = *from;
    deliver(domain, ReplyTo::new(right), line, from, SeeAnswer::Call { tool, answer: Box::new(answer) });
}

fn deliver(domain: &mut Domain, to: ReplyTo, line: Box<[u8]>, from: u32, answer: SeeAnswer) {
    let request = crate::boundary::delivery_output(super::Delivery::Host(Box::new(crate::HostDelivery::See {
        to,
        line,
        from,
        answer: Box::new(answer),
    })));
    assert!(crate::domain::now(domain, request), "see forge render door reserved");
}

fn read_head(answer: &CallAnswer) -> Option<api::Commit> {
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
        | CallAnswer::Finding(_) | CallAnswer::Red(_)
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
