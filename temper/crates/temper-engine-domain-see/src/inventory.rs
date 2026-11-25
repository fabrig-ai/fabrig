//! Settings inventory context (temper/docs/design/domain/agent/agents.md, 1).
use crate::{ForgePerson, OfferedModel, RepositoryIdentity};
use alloc::boxed::Box;
use jig_domain_core::{Core, SettingsAnswer};
use skein_lib::List;
use temper_engine_domain_forge::Domain as Forge;

/// Enrich only authorized inventories, scoped to the caller's retained project.
#[must_use]
pub fn repositories(core: &Core, forge: &Forge, task: u64, answer: &SettingsAnswer) -> Box<[RepositoryIdentity]> {
    if let SettingsAnswer::Inventory(_) = answer {
        let Some(context) = core.tasks.delegation(task) else { return Box::new([]) };
        crate::forge::repositories(forge, context.project)
    } else {
        Box::new([])
    }
}

/// The task's payer and model authority bound an authorized inventory's offer.
#[must_use]
pub fn offer(core: &Core, task: u64, answer: &SettingsAnswer) -> Box<[OfferedModel]> {
    match answer {
        SettingsAnswer::Inventory(_) => {}
        SettingsAnswer::Setting(_)
        | SettingsAnswer::Revision(_)
        | SettingsAnswer::Revisions(_)
        | SettingsAnswer::Unknown => return Box::new([]),
    }
    let Some(payer) = core.tasks.payer(task) else { return Box::new([]) };
    let Some(context) = core.tasks.delegation(task) else { return Box::new([]) };
    let offered = core.offer(payer, context.project, context.authority.models);
    let mut offer = List::with_capacity(offered.len());
    for row in &offered {
        let model = core.settings.models.get(usize::from(row.model)).expect("offered configured model");
        let provider = core.settings.llm_providers.get(model.provider).expect("offered model provider");
        let effort = match model.default_effort() {
            Some(effort) => Some(skein_lib::bytes::copy_of(effort)),
            None => None,
        };
        offer
            .push(OfferedModel {
                name: model.name.clone(),
                provider: provider.name.clone(),
                efforts: model.efforts.clone(),
                effort,
            })
            .expect("one row per offered model");
    }
    offer.into_boxed()
}

/// Owned forge logins are named once per readable host in the caller's project.
#[must_use]
pub fn forge_people(core: &Core, forge: &Forge, task: u64, answer: &SettingsAnswer) -> Box<[ForgePerson]> {
    match answer {
        SettingsAnswer::Inventory(_) => {}
        SettingsAnswer::Setting(_)
        | SettingsAnswer::Revision(_)
        | SettingsAnswer::Revisions(_)
        | SettingsAnswer::Unknown => return Box::new([]),
    }
    let Some(context) = core.tasks.delegation(task) else { return Box::new([]) };
    let repositories = forge.readable_repositories(context.project);
    let mut people =
        List::<ForgePerson>::with_capacity(u32::try_from(repositories.len()).expect("bounded repositories"));
    for repository in &repositories {
        let Some(login) = forge.owned_person(repository.provider.forge) else { continue };
        let mut named = false;
        for person in &people {
            named |= person.host == repository.host;
        }
        if !named {
            people
                .push(ForgePerson { host: repository.host.clone(), login: login.into() })
                .expect("at most one person per host");
        }
    }
    people.into_boxed()
}
