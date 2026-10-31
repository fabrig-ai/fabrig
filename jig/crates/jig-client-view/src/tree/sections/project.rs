//! Project sections compose shared cards and connector parts (pages.md, 6).
use crate::words::Words;
use crate::words::phrases::jig::JigPhrase;
use crate::{JigBindings, tree::cards};

use skein_ui::{Builder, Role};

use crate::{
    Borrowed,
    state::{
        action::Act,
        address::{Address, ProjectSection},
    },
};

pub(crate) mod goals;
pub(crate) mod notes;
pub(crate) mod owner;
pub(crate) mod owner_goals;
/// The project's shared identity while any of its sections is shown.
pub fn header(b: &mut Builder, client: &Borrowed<'_>, words: &Words, _bindings: &mut JigBindings) -> bool {
    let project_page = client.project_page();
    if client.shown_project().is_none() {
        return false;
    }
    b.open(Role::Region);
    b.region(crate::regions::JigRegion::ProjectHeader.region());
    b.name(&words.say(JigPhrase::ProjectPage, &[]));
    if let Some(doc) = &project_page.owner {
        words.text(b, JigPhrase::Project, &[crate::words::name(doc.name.bytes())]);
        crate::tree::fragments::document_text::loss(b, &doc.name, words);
    } else {
        words.status(b, JigPhrase::LoadingObject, &[]);
    }
    b.close();
    true
}
/// Every section linked to its address, with the address's own marked current.
pub fn tabs(b: &mut Builder, client: &Borrowed<'_>, words: &Words, bindings: &mut JigBindings) -> bool {
    let project_page = client.project_page();
    let Some(number) = client.shown_project() else {
        return false;
    };
    if project_page.owner.is_none() {
        return false;
    }
    b.open(Role::Tabs);
    b.region(crate::regions::JigRegion::ProjectTabs.region());
    b.name(project_page.owner.as_ref().expect("loaded owner project for tabs").name.bytes());
    for section in [
        ProjectSection::Overview,
        ProjectSection::Goals,
        ProjectSection::Tasks,
        ProjectSection::Notes,
        ProjectSection::Settings,
        ProjectSection::Resources,
        ProjectSection::Policy,
        ProjectSection::Roles,
        ProjectSection::Spend,
        ProjectSection::Recurring,
    ] {
        let address = Address::Project { project: number, section };
        let current = match client.page() {
            crate::state::page::Page::Work { address: own, .. } => own == &address,
            crate::state::page::Page::Starting
            | crate::state::page::Page::SigningIn
            | crate::state::page::Page::Elsewhere
            | crate::state::page::Page::Missing
            | crate::state::page::Page::OutOfDate
            | crate::state::page::Page::SomeoneElse => false,
        };
        crate::tree::pages::work::tab(b, words, bindings, phrase(section), Act::Go { address }, current);
    }
    b.close();
    true
}
pub(crate) fn all_shown(client: &Borrowed<'_>) -> bool {
    let Some(project) = client.shown_project() else {
        return false;
    };
    for section in [ProjectSection::Overview, ProjectSection::Goals, ProjectSection::Tasks, ProjectSection::Notes] {
        if !client.shows(crate::state::page::Section::Project { project, section }) {
            return false;
        }
    }
    if client.shows(crate::state::page::Section::Project { project, section: ProjectSection::Settings }) {
        return true;
    }
    for section in [
        ProjectSection::Resources,
        ProjectSection::Policy,
        ProjectSection::Roles,
        ProjectSection::Spend,
        ProjectSection::Recurring,
    ] {
        if !client.shows(crate::state::page::Section::Project { project, section }) {
            return false;
        }
    }
    true
}
pub(crate) fn section(
    b: &mut Builder,
    client: &Borrowed<'_>,
    words: &Words,
    bindings: &mut JigBindings,
    project: u32,
    selected: ProjectSection,
) -> bool {
    let project_page = client.project_page();
    if !client.shows(crate::state::page::Section::Project { project, section: selected }) {
        return false;
    }
    crate::tree::pages::work::section(b, words, phrase(selected), region(selected));
    crate::tree::pages::mark_current(b, client, Address::Project { project, section: selected });
    super::status(b, client, words, crate::state::page::Section::Project { project, section: selected });
    if let Some(doc) = &project_page.owner {
        owner::section(b, doc, selected, client, words, bindings);
        b.close();
        return true;
    }
    b.close();
    true
}

const fn phrase(section: ProjectSection) -> JigPhrase {
    match section {
        ProjectSection::Overview => JigPhrase::Overview,
        ProjectSection::Goals => JigPhrase::Goals,
        ProjectSection::Tasks => JigPhrase::Tasks,
        ProjectSection::Notes => JigPhrase::Notes,
        ProjectSection::Settings => JigPhrase::Settings,
        ProjectSection::Resources => JigPhrase::Resources,
        ProjectSection::Policy => JigPhrase::Policy,
        ProjectSection::Roles => JigPhrase::Roles,
        ProjectSection::Spend => JigPhrase::Spend,
        ProjectSection::Recurring => JigPhrase::Recurring,
    }
}

fn inbox_cards(b: &mut Builder, client: &Borrowed<'_>, words: &Words, bindings: &mut JigBindings) {
    for entry in &client.inbox().rows {
        if let Some(id) = client.objects().find(entry.card_key())
            && !client.owes(id)
        {
            cards::card(
                b,
                client,
                id,
                &match entry {
                    jig_client::contract::InboxRow::Owner(_) => cards::Place::Elsewhere,
                },
                words,
                bindings,
            );
        }
    }
}

/// Retained funding and earlier period rows preserve the original spend window.
fn ended_spend(b: &mut Builder, client: &Borrowed<'_>, words: &Words, bindings: &mut JigBindings) {
    let page = client.project_page();
    if let Some(spend) = &page.owner_spend {
        crate::tree::fragments::ended_pages::spend_render(b, spend, words);
    }
    crate::tree::pages::cut(b, words, page.spend_cut);
    crate::tree::pages::work::cursors(
        b,
        words,
        bindings,
        page.spend_before.as_ref(),
        page.spend_after.as_ref(),
        crate::state::page::project::SPEND_WINDOW,
    );
}

pub(crate) const fn region(section: ProjectSection) -> crate::regions::JigRegion {
    use crate::regions::JigRegion;
    match section {
        ProjectSection::Overview => JigRegion::ProjectOverview,
        ProjectSection::Goals => JigRegion::ProjectGoals,
        ProjectSection::Tasks => JigRegion::ProjectTasks,
        ProjectSection::Notes => JigRegion::ProjectNotes,
        ProjectSection::Settings => JigRegion::ProjectSettings,
        ProjectSection::Resources => JigRegion::ProjectResources,
        ProjectSection::Policy => JigRegion::ProjectPolicy,
        ProjectSection::Roles => JigRegion::ProjectRoles,
        ProjectSection::Spend => JigRegion::ProjectSpend,
        ProjectSection::Recurring => JigRegion::ProjectRecurring,
    }
}
