//! A project's settings frame and repositories (temper/docs/design/client/README.md, 3.5, Settings).
use crate::{
    frame::Bindings,
    look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Binding, Builder, Role, Run, State, Style, phrases::Hole};
use temper_client_domain::{
    Address, ChatsFilter, Domain,
    forge::{self, listed},
    pages::Act,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Tab {
    HowWorkRuns,
    Repositories,
    Forges,
    Accounts,
}

/// A flow's return tab (temper/docs/design/client/README.md, 3.5, The flows' paths).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum FlowTab {
    Forges,
    Accounts,
    Repositories,
}

/// A flow's path at its head (README.md, 3.5, The flows' paths): nothing with no project, or in the terminal.
pub(crate) fn path(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: Option<u32>,
    tab: FlowTab,
    here: Word,
) {
    if w.shell == crate::words::Shell::Terminal {
        return;
    }
    let Some(project) = project else { return };
    let (word, address) = match tab {
        FlowTab::Forges => (Word::ForgesTitle, Address::Forges),
        FlowTab::Accounts => (Word::SettingsAccountsTab, Address::Accounts),
        FlowTab::Repositories => (Word::SettingsRepositoriesTab, Address::Repositories { project }),
    };
    b.open(Role::Slot);
    b.style(look::SETTINGS_PATH);
    b.open(Role::Navigation);
    b.name(&w.say(Word::ConfigureWhere, &[]));
    b.style(look::CRUMBS);
    crate::link(b, &w.say(word, &[]), bindings.insert(Act::Go { address: address.clone() }));
    b.style(look::BACK);
    b.text(Run::Plain, &w.say(word, &[]));
    b.close();
    b.open(Role::Slot);
    b.style(look::TRAIL);
    for (name, address) in [
        (project_name(root, w, project), Address::Home { project }),
        (w.say(Word::RailSettings, &[]), Address::Settings { project }),
        (w.say(word, &[]), address),
    ] {
        crate::link(b, &name, bindings.insert(Act::Go { address }));
        b.text(Run::Plain, &name);
        b.close();
    }
    text(b, Role::Text, &w.say(here, &[]), look::SETTINGS_DETAIL);
    b.close();
    b.close();
    b.close();
}

/// Opens the tab's column; the caller draws its body and closes it (README.md, 3.5, Settings).
pub(crate) fn open(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32, tab: Tab) {
    b.open(Role::Slot);
    b.style(look::SETTINGS_COL);
    b.open(Role::Slot);
    b.style(look::SETTINGS_INTRO);
    let name = project_name(root, w, project);
    text(b, Role::Paragraph, &w.say(Word::SettingsKicker, &[Hole::Name(name.clone())]), look::SETTINGS_KICKER);
    let (heading, lede) = match tab {
        Tab::HowWorkRuns => (Word::SettingsHowHeading, Word::SettingsHowLede),
        Tab::Repositories => (Word::SettingsRepositoriesHeading, Word::SettingsRepositoriesLede),
        Tab::Forges => (Word::SettingsForgesHeading, Word::SettingsForgesLede),
        Tab::Accounts => (Word::SettingsAccountsHeading, Word::SettingsAccountsLede),
    };
    let title = match tab {
        Tab::HowWorkRuns | Tab::Repositories => w.say(heading, &[Hole::Name(name.clone())]),
        Tab::Forges | Tab::Accounts => w.say(heading, &[]),
    };
    text(b, Role::Heading { level: 1 }, &title, look::SETTINGS_HEADLINE);
    let lede = match tab {
        Tab::Repositories => w.say(lede, &[Hole::Name(name)]),
        Tab::HowWorkRuns | Tab::Forges | Tab::Accounts => w.say(lede, &[]),
    };
    text(b, Role::Paragraph, &lede, look::SETTINGS_LEDE);
    b.close();
    b.open(Role::Navigation);
    b.name(&w.say(Word::RailSettings, &[]));
    b.style(look::SETTINGS_TABS);
    for (item, word, address) in [
        (Tab::HowWorkRuns, Word::SettingsHowTab, Address::Settings { project }),
        (Tab::Repositories, Word::SettingsRepositoriesTab, Address::Repositories { project }),
        (Tab::Forges, Word::ForgesTitle, Address::Forges),
        (Tab::Accounts, Word::SettingsAccountsTab, Address::Accounts),
    ] {
        let name = w.say(word, &[]);
        crate::link(b, &name, bindings.insert(Act::Go { address }));
        b.text(Run::Plain, &name);
        if item == tab {
            b.state(State::Current);
        }
        match item {
            Tab::Forges => crate::frame::forge_mark(b, root, w, false),
            Tab::Accounts => crate::frame::account_mark(b, root, w, false),
            Tab::HowWorkRuns | Tab::Repositories => {}
        }
        b.close();
    }
    b.close();
}
/// Opens the column without a project's kicker or tabs (README.md, 3.5, Settings).
pub(crate) fn open_alone(b: &mut Builder, heading: &[u8], lede: &[u8]) {
    b.open(Role::Slot);
    b.style(look::SETTINGS_COL);
    b.open(Role::Slot);
    b.style(look::SETTINGS_INTRO);
    text(b, Role::Heading { level: 1 }, heading, look::SETTINGS_HEADLINE);
    text(b, Role::Paragraph, lede, look::SETTINGS_LEDE);
    b.close();
}
/// Opens the section and its head; the caller adds the act and closes both.
pub(crate) fn section_head(b: &mut Builder, title: &[u8]) {
    b.open(Role::Slot);
    b.style(look::SETTINGS_SECTION);
    b.open(Role::Slot);
    b.style(look::SETTINGS_SECTION_HEAD);
    text(b, Role::Heading { level: 2 }, title, look::SETTINGS_SECTION_TITLE);
}
/// Opens a quiet band; the caller adds its act and closes it.
pub(crate) fn quiet_band(b: &mut Builder, tag: &[u8], words: &[u8]) {
    b.open(Role::Slot);
    b.style(look::SETTINGS_BAND);
    b.style(look::SETTINGS_QUIET_BAND);
    b.open(Role::Paragraph);
    b.name(words);
    text(b, Role::Text, tag, look::SETTINGS_TAG);
    b.text(Run::Plain, words);
    b.close();
}
/// A link in the settings button's look, primary or small.
pub(crate) fn button_to(b: &mut Builder, name: &[u8], to: Binding, primary: bool) {
    crate::link(b, name, to);
    if primary {
        b.style(look::SETTINGS_PRIMARY);
    } else {
        b.style(look::SETTINGS_BTN);
        b.style(look::SETTINGS_SMALL);
    }
    b.text(Run::Plain, name);
    b.close();
}
fn project_name(root: &Domain, w: &Words, project: u32) -> Box<[u8]> {
    match crate::pages::project_name(root.pages().me(root.client()), project) {
        Some(name) => name,
        None => w.say(Word::Project, &[Hole::Number(u64::from(project))]),
    }
}
fn main(b: &mut Builder) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::SETTINGS);
}
fn missing(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) -> bool {
    if root.pages().me(root.client()).is_some()
        && crate::pages::project_name(root.pages().me(root.client()), project).is_none()
    {
        crate::cards::text(b, Role::Paragraph, &w.say(Word::ProjectNotYours, &[Hole::Number(u64::from(project))]));
        link(b, w, bindings, Word::Chats, Address::Chats { filter: ChatsFilter::All, start: false });
        return true;
    }
    false
}
pub(crate) fn how_work_runs(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    main(b);
    open(b, root, w, bindings, project, Tab::HowWorkRuns);
    if !missing(b, root, w, bindings, project) {
        b.open(Role::Slot);
        b.style(look::SETTINGS_BODY);
        b.open(Role::Slot);
        b.style(look::SETTINGS_SECTION);
        b.open(Role::Slot);
        b.style(look::SETTINGS_SETTING);
        b.open(Role::Slot);
        crate::cards::text(b, Role::Text, &w.say(Word::ConfigureTitle, &[]));
        text(b, Role::Text, &w.say(Word::SettingsConfigureHelp, &[]), look::SETTINGS_DETAIL);
        b.close();
        b.open(Role::Slot);
        b.style(look::SETTINGS_END);
        let task = match root.pages().chats() {
            Some(chats) => chats.open_configuring(root.client(), root.pages().me(root.client()), project),
            None => None,
        };
        let (word, address) = match task {
            Some(task) => (Word::ForgeGoOn, Address::Chat { task }),
            None => (Word::HomeStart, Address::Configure { project }),
        };
        button_link(b, w, bindings, word, address, true);
        b.close();
        b.close();
        b.close();
        b.close();
    }
    b.close();
    b.close();
}

pub(crate) fn repositories(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    main(b);
    open(b, root, w, bindings, project, Tab::Repositories);
    if !missing(b, root, w, bindings, project) {
        repository_section(b, root, w, bindings, project);
    }
    b.close();
    b.close();
}
fn repository_section(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    let playground = root.forge().playground();
    let own_playground = match playground {
        Some(p) => p.project == Some(project),
        None => false,
    };
    let rows = listed::rows(root.forges(), forge::connect::playground(root.forge()));
    let known = own_playground
        || match root.forges() {
            forge::forges::State::Rows(_) => true,
            forge::forges::State::Unknown | forge::forges::State::Ended => false,
        };
    let mut count = u64::from(own_playground);
    if !own_playground {
        for row in &rows {
            for repository in &row.repositories {
                if repository.project == project {
                    count = count.checked_add(1).expect("bounded repositories");
                }
            }
        }
    }
    let title = if known {
        w.say(Word::SettingsRepositoriesCount, &[Hole::Number(count)])
    } else {
        w.say(Word::SettingsRepositoriesTab, &[])
    };
    section_head(b, &title);
    link(b, w, bindings, Word::Adoption, Address::Adopt { project });
    b.close();
    if known && count == 0 {
        quiet_band(
            b,
            &w.say(Word::SettingsNoneTag, &[]),
            &w.say(Word::SettingsNoneBody, &[Hole::Name(project_name(root, w, project))]),
        );
        button_link(b, w, bindings, Word::Adoption, Address::Adopt { project }, false);
        b.close();
    } else if known {
        b.open(Role::Slot);
        b.style(look::SETTINGS_LINES);
        if own_playground {
            if let Some(playground) = playground {
                let name = match listed::playground(root) {
                    Some((_, row)) => crate::forge::connect::name(w, row, root),
                    None => w.say(Word::ForgePlayground, &[]),
                };
                repository_row(b, &playground.owner, &playground.name, &name);
            }
        } else {
            for row in &rows {
                for repository in &row.repositories {
                    if repository.project == project {
                        repository_row(
                            b,
                            &repository.owner,
                            &repository.name,
                            &crate::forge::connect::name(w, row, root),
                        );
                    }
                }
            }
        }
        b.close();
    }
    b.close();
}
fn repository_row(b: &mut Builder, owner: &[u8], name: &[u8], forge: &[u8]) {
    b.open(Role::Slot);
    crate::cards::text(b, Role::Text, &forge::words::full_name(owner, name));
    text(b, Role::Text, forge, look::SETTINGS_WHEN);
    b.close();
}
pub(crate) fn sign_out(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    let Some(me) = root.pages().me(root.client()) else { return };
    b.open(Role::Slot);
    b.style(look::SETTINGS_SETTING);
    b.open(Role::Slot);
    crate::cards::text(b, Role::Text, &w.say(Word::SettingsSignedIn, &[Hole::Name(me.party.name.bytes().into())]));
    text(b, Role::Text, me.party.login.bytes(), look::SETTINGS_DETAIL);
    if root.client().link().sign_out_lost {
        crate::cards::text(b, Role::Alert, &w.say(Word::SignOutLost, &[]));
    }
    b.close();
    b.open(Role::Slot);
    b.style(look::SETTINGS_END);
    b.open(Role::Button);
    b.style(look::SETTINGS_BTN);
    b.style(look::SETTINGS_SMALL);
    b.name(&w.say(Word::SignOut, &[]));
    b.bind(bindings.insert(Act::SignOut));
    b.close();
    b.close();
    b.close();
}
fn text(b: &mut Builder, role: Role, said: &[u8], style: Style) {
    b.open(role);
    b.style(style);
    b.name(said);
    b.text(Run::Plain, said);
    b.close();
}
fn link(b: &mut Builder, w: &Words, bindings: &mut Bindings, word: Word, address: Address) {
    let name = w.say(word, &[]);
    crate::link(b, &name, bindings.insert(Act::Go { address }));
    b.text(Run::Plain, &name);
    b.close();
}
fn button_link(b: &mut Builder, w: &Words, bindings: &mut Bindings, word: Word, address: Address, primary: bool) {
    let name = w.say(word, &[]);
    crate::link(b, &name, bindings.insert(Act::Go { address }));
    b.style(if primary { look::SETTINGS_PRIMARY } else { look::SETTINGS_BTN });
    b.style(look::SETTINGS_SMALL);
    b.text(Run::Plain, &name);
    b.close();
}
