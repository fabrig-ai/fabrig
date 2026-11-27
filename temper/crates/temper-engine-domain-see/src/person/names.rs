//! Display identities accompanying a person's answer (domain/agent/ask-temper.md, 1).
use super::{Answer, Gathered, Part, documents};
use crate::{SeeName, SeeNames};
use alloc::boxed::Box;
use jig_domain_core::settings::Setting;
use skein_lib::List;

pub(super) fn make(core: &jig_domain_core::Core, project: u32, gathered: &Gathered) -> SeeNames {
    let project_name = match &gathered.first {
        Some(Part::Jig(Answer::Document(document))) => match document.as_ref() {
            documents::Document::Project(project) => Some(project.name.bytes.clone()),
            documents::Document::Frame(_)
            | documents::Document::Chips(_)
            | documents::Document::Run(_)
            | documents::Document::Task(_)
            | documents::Document::Tree(_)
            | documents::Document::Lines(_)
            | documents::Document::Proposal(_)
            | documents::Document::Escalation(_)
            | documents::Document::Question(_)
            | documents::Document::PersonTask(_)
            | documents::Document::Result(_)
            | documents::Document::Note(_)
            | documents::Document::Inbox(_)
            | documents::Document::Terminal { .. }
            | documents::Document::Accounts(_)
            | documents::Document::Deployment(_)
            | documents::Document::Settings(_) => None,
        },
        Some(
            Part::Jig(
                Answer::TaskText(_)
                | Answer::Setting(_)
                | Answer::Revision(_)
                | Answer::Revisions(_)
                | Answer::Tasks(_)
                | Answer::Waits(_)
                | Answer::Ended(_)
                | Answer::Page(_)
                | Answer::None(_),
            )
            | Part::Forge(_),
        )
        | None => None,
    };
    SeeNames {
        project: SeeName {
            number: u64::from(project),
            name: match project_name {
                Some(name) => name,
                None => project_name_from_record(core, project),
            },
        },
        people: people(core, project),
        charters: charters(core),
    }
}

fn project_name_from_record(core: &jig_domain_core::Core, project: u32) -> Box<[u8]> {
    match core.project_name(project) {
        Some(name) => name.as_bytes().into(),
        None => Box::new([]),
    }
}

fn people(core: &jig_domain_core::Core, project: u32) -> Box<[SeeName]> {
    let Some(members) = core.people.project_members(project) else { return Box::new([]) };
    let mut people = List::with_capacity(u32::try_from(members.len()).expect("bounded project members"));
    for member in members {
        match core.people.kind(member.person) {
            Some(jig_domain_people::Kind::Person) => {}
            Some(jig_domain_people::Kind::Service) | None => continue,
        }
        let Some(identity) = core.people.identity(member.person) else { continue };
        people
            .push(SeeName {
                number: member.person,
                name: if identity.name.is_empty() { identity.login.clone() } else { identity.name.clone() },
            })
            .expect("one name per project person");
    }
    people.into_boxed()
}

fn charters(core: &jig_domain_core::Core) -> Box<[SeeName]> {
    let inventory = documents::settings_inventory(core);
    let mut charters = List::with_capacity(inventory.settings.len());
    for line in &inventory.settings {
        match line.setting {
            Setting::Charter(number) => {
                let name = match &line.shown {
                    Some(name) => name.bytes.clone(),
                    None => Box::new([]),
                };
                charters.push(SeeName { number: u64::from(number), name }).expect("one name per charter");
            }
            Setting::Template(_)
            | Setting::Policy(_)
            | Setting::Provider(_)
            | Setting::Worker(_)
            | Setting::Model(_)
            | Setting::Rules
            | Setting::Limits
            | Setting::ProjectPolicy { .. }
            | Setting::Application { .. } => {}
        }
    }
    charters.into_boxed()
}
