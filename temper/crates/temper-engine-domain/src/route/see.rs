//! Routing `see` reads through the root (temper/docs/design/domain/agent/ask-temper.md, 1).
pub(super) mod forge;
pub(crate) mod person;
pub(super) mod proposal;

use super::host_route::FlightRender;
use super::{Decision, Delivery, Domain, Env, Limits, ReplyTo, Work, emit};
use alloc::boxed::Box;
use jig_domain_core as core;
use temper_engine_domain_see::SeeAnswer;
use temper_engine_domain_see::person::Gathered;
pub(crate) use temper_engine_domain_see::{Address, address};

/// A `see` read in flight (domain/agent/ask-temper.md, 1).
#[derive(Hash, Debug)]
pub(crate) struct SeeFlight {
    pub line: Box<[u8]>,
    pub reading: SeeReading,
}
#[derive(Hash, Debug)]
pub(crate) enum SeeReading {
    Settings,
    Head,
    File,
    Proposal,
    Person(Box<Gathered>),
}

pub(crate) fn call(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    key: crate::CallKey,
    line: Box<[u8]>,
    from: u32,
) {
    let context = domain.core.tasks.delegation(key.task).expect("authorized chat delegation");
    let project = context.project;
    let read = match address(&line, project) {
        Address::Ways => {
            let party = match context.requester {
                jig_domain_tasks::Party::Person(party) => party,
                jig_domain_tasks::Party::Task(_) | jig_domain_tasks::Party::Deployment { .. } => {
                    deliver(env, decision, to, line, from, SeeAnswer::Unknown);
                    return;
                }
            };
            let document = super::ways::document(domain, project, party);
            deliver(env, decision, to, line, from, SeeAnswer::Ways(Box::new(document)));
            return;
        }
        Address::Inventory => core::SettingsRead::Inventory,
        Address::Setting(setting) => core::SettingsRead::Setting { setting, revision: 0 },
        Address::Revisions => core::SettingsRead::Revisions { before: None },
        Address::Revision { number } => core::SettingsRead::Revision { number },
        Address::Landing => {
            forge::landing(domain, env, decision, to, key, line, from);
            return;
        }
        Address::Gates { repository } => {
            forge::gates(domain, env, decision, to, key, line, from, repository);
            return;
        }
        Address::File { repository, path, branch } => {
            forge::file(domain, env, decision, to, key, line, from, repository, path, branch);
            return;
        }
        Address::Proposal { proposer, number } => {
            proposal::read(domain, env, decision, to, key, line, from, proposer, number);
            return;
        }
        Address::Page { name } => {
            deliver(env, decision, to, line, from, SeeAnswer::Page { name });
            return;
        }
        Address::Search { words } => {
            deliver(env, decision, to, line, from, SeeAnswer::Search { words });
            return;
        }
        address @ (Address::Me
        | Address::Project
        | Address::Goals { .. }
        | Address::Tasks
        | Address::Task { .. }
        | Address::Tree { .. }
        | Address::Activity { .. }
        | Address::Turn { .. }
        | Address::Change { .. }
        | Address::Changes
        | Address::Inbox) => {
            person::read(domain, env, decision, to, key, line, from, address);
            return;
        }
        Address::Unknown => {
            deliver(env, decision, to, line, from, SeeAnswer::Unknown);
            return;
        }
    };
    let requested = match &read {
        core::SettingsRead::Revision { number } => Some(u64::from(*number)),
        core::SettingsRead::Inventory | core::SettingsRead::Setting { .. } | core::SettingsRead::Revisions { .. } => {
            None
        }
    };
    let asked = match &read {
        core::SettingsRead::Setting { setting, revision } => Some((*setting, *revision)),
        core::SettingsRead::Inventory | core::SettingsRead::Revision { .. } | core::SettingsRead::Revisions { .. } => {
            None
        }
    };
    let right = to.into_token();
    domain.call_renders.get_mut(&right).expect("decoded see flight").render = FlightRender::Settings {
        see: Some(Box::new(SeeFlight { line, reading: SeeReading::Settings })),
        from,
        requested,
        asked,
    };
    let mut read = read;
    if let core::SettingsRead::Setting { revision, .. } = &mut read {
        *revision = domain.core.current.newest();
    }
    domain.work.push(Work::Core(core::Event::NamedAction {
        to: ReplyTo::new(right),
        key,
        action: core::NamedAction::Settings(read),
    }));
}

pub(super) fn deliver(
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    line: Box<[u8]>,
    from: u32,
    answer: SeeAnswer,
) {
    emit(
        decision,
        &env.limits,
        Delivery::Host(Box::new(super::HostDelivery::See { to, line, from, answer: Box::new(answer) })),
    );
}
