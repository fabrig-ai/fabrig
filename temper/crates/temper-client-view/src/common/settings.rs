//! Shared setup items (temper/docs/design/client/view-crates.md, 3.3).
use crate::{
    bindings::Bindings,
    look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Binding, Builder, Role, Run, State, Style, phrases::Hole};
use temper_client_domain::{Address, Domain, pages::Act};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Tab {
    HowWorkRuns,
    Repositories,
    Forges,
    Accounts,
    Models,
    Builders,
    Members,
    Limits,
}

/// A flow's return tab (temper/docs/design/client/pages/README.md, The flows' paths).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum FlowTab {
    Forges,
    Accounts,
}

/// A flow's path at its head (client/pages/README.md, The flows' paths): nothing with no project, or in the terminal.
pub(crate) fn path(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: Option<u32>,
    tab: FlowTab,
    here: Word,
) {
    flow_path(b, root, w, bindings, project, tab, &w.say(here, &[]), None);
}

/// A way under a flow's choices (client/pages/README.md, The flows' paths): the back link
/// and the trail go through the choices, and the way's name is last.
pub(crate) struct FlowWay<'a> {
    pub(crate) choices: Word,
    pub(crate) address: Address,
    pub(crate) name: &'a [u8],
}

pub(crate) fn way_path(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: Option<u32>,
    tab: FlowTab,
    way: &FlowWay<'_>,
) {
    flow_path(b, root, w, bindings, project, tab, way.name, Some(way));
}

#[expect(clippy::too_many_arguments, reason = "choices and ways share one path drawing")]
fn flow_path(
    b: &mut Builder,
    _root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: Option<u32>,
    tab: FlowTab,
    here: &[u8],
    way: Option<&FlowWay<'_>>,
) {
    if w.shell == crate::words::Shell::Terminal {
        return;
    }
    let Some(_project) = project else { return };
    let (word, address) = match tab {
        FlowTab::Forges => (Word::ForgesTitle, Address::Forges),
        FlowTab::Accounts => (Word::SettingsAccountsTab, Address::Accounts),
    };
    b.open(Role::Slot);
    b.style(look::SETTINGS_PATH);
    b.open(Role::Navigation);
    b.name(&w.say(Word::AskWhere, &[]));
    b.style(look::CRUMBS);
    let (back, to) = match way {
        Some(way) => (way.choices, way.address.clone()),
        None => (word, address.clone()),
    };
    crate::bindings::link(b, &w.say(back, &[]), bindings.insert(Act::Go { address: to }));
    b.style(look::BACK);
    b.text(Run::Plain, &w.say(back, &[]));
    b.close();
    b.open(Role::Slot);
    b.style(look::TRAIL);
    for (name, address) in [(w.say(Word::DeploymentTitle, &[]), Address::Accounts), (w.say(word, &[]), address)] {
        crate::bindings::link(b, &name, bindings.insert(Act::Go { address }));
        b.text(Run::Plain, &name);
        b.close();
    }
    if let Some(way) = way {
        let name = w.say(way.choices, &[]);
        crate::bindings::link(b, &name, bindings.insert(Act::Go { address: way.address.clone() }));
        b.text(Run::Plain, &name);
        b.close();
    }
    text(b, Role::Text, here, look::SETTINGS_DETAIL);
    b.close();
    b.close();
    b.close();
}

/// Opens the tab's column; the caller draws its body and closes it (client/pages/README.md, Settings).
#[expect(clippy::too_many_lines, reason = "one frame draws the closed set of project and deployment tabs")]
pub(crate) fn open(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32, tab: Tab) {
    b.open(Role::Slot);
    b.style(look::SETTINGS_COL);
    let deployment = deployment(tab);
    if tab == Tab::HowWorkRuns
        && root.pages().me(root.client()).is_some()
        && crate::common::project_name(root.pages().me(root.client()), project).is_none()
    {
        text(b, Role::Heading { level: 1 }, &super::how::head(root, w, project).0, look::SETTINGS_HEADLINE);
        return;
    }
    if tab == Tab::HowWorkRuns {
        b.style(look::HOW_WIDE);
    }
    let detail = match root.pages().address() {
        Address::Role { .. } | Address::Rule { .. } => true,
        Address::AllHome
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Work { .. }
        | Address::Told
        | Address::Remembered
        | Address::Members
        | Address::Limits
        | Address::Accounts
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. }
        | Address::Chat { .. }
        | Address::Talk { .. }
        | Address::Task { .. }
        | Address::Home { .. }
        | Address::Goals { .. }
        | Address::Changes { .. }
        | Address::Team { .. }
        | Address::Settings { .. }
        | Address::Repositories { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Ask { .. }
        | Address::Tell { .. }
        | Address::Adopt { .. } => false,
    };
    if !detail {
        b.open(Role::Slot);
        b.style(look::SETTINGS_INTRO);
        let name = project_name(root, w, project);
        let kicker = if deployment {
            w.say(Word::DeploymentTitle, &[])
        } else {
            w.say(Word::SettingsKicker, &[Hole::Name(name.clone())])
        };
        text(b, Role::Paragraph, &kicker, look::SETTINGS_KICKER);
        let (heading, lede) = match tab {
            Tab::HowWorkRuns => (Word::SettingsHowHeading, Word::HowChange),
            Tab::Repositories => (Word::SettingsRepositoriesHeading, Word::SettingsRepositoriesLede),
            Tab::Forges => (Word::SettingsForgesHeading, Word::SettingsForgesLede),
            Tab::Accounts => (Word::SettingsAccountsHeading, Word::SettingsAccountsLede),
            Tab::Models => (Word::ModelsHeading, Word::ModelsLede),
            Tab::Builders => (Word::SettingsBuildersHeading, Word::SettingsBuildersLede),
            Tab::Members => (Word::MembersHeading, Word::MembersLede),
            Tab::Limits => (Word::LimitsHeading, Word::LimitsLede),
        };
        let title = match tab {
            Tab::HowWorkRuns => super::how::head(root, w, project).0,
            Tab::Repositories => w.say(heading, &[Hole::Name(name.clone())]),
            Tab::Forges | Tab::Accounts | Tab::Models | Tab::Builders | Tab::Members | Tab::Limits => {
                w.say(heading, &[])
            }
        };
        text(b, Role::Heading { level: 1 }, &title, look::SETTINGS_HEADLINE);
        let lede = match tab {
            Tab::Repositories => w.say(lede, &[Hole::Name(name)]),
            Tab::HowWorkRuns => super::how::head(root, w, project).1,
            Tab::Forges | Tab::Accounts | Tab::Models | Tab::Builders | Tab::Members | Tab::Limits => w.say(lede, &[]),
        };
        text(b, Role::Paragraph, &lede, look::SETTINGS_LEDE);
        if tab == Tab::HowWorkRuns {
            super::how::ask(b, root, w, bindings, project);
        }
        b.close();
    }
    b.open(Role::Navigation);
    b.name(&w.say(if deployment { Word::DeploymentTitle } else { Word::SettingsHowTab }, &[]));
    b.style(look::SETTINGS_TABS);
    for (item, word, address) in [
        (Tab::HowWorkRuns, Word::SettingsHowTab, Address::Settings { project }),
        (Tab::Repositories, Word::SettingsRepositoriesTab, Address::Repositories { project }),
        (Tab::Accounts, Word::SettingsAccountsTab, Address::Accounts),
        (Tab::Forges, Word::ForgesTitle, Address::Forges),
        (Tab::Models, Word::ModelsTab, Address::Models { project }),
        (Tab::Builders, Word::SettingsBuildersTab, Address::Builders { project }),
        (Tab::Members, Word::MembersTab, Address::Members),
        (Tab::Limits, Word::LimitsTab, Address::Limits),
    ] {
        if self::deployment(item) != deployment {
            continue;
        }
        let name = w.say(word, &[]);
        crate::bindings::link(b, &name, bindings.insert(Act::Go { address }));
        b.text(Run::Plain, &name);
        if item == tab {
            b.state(State::Current);
        }
        match item {
            Tab::Forges => crate::bindings::forge_mark(b, root, w, false),
            Tab::Accounts => crate::bindings::account_mark(b, root, w, false),
            Tab::HowWorkRuns | Tab::Repositories | Tab::Models | Tab::Builders | Tab::Members | Tab::Limits => {}
        }
        b.close();
    }
    b.close();
}

fn deployment(tab: Tab) -> bool {
    match tab {
        Tab::HowWorkRuns | Tab::Repositories => false,
        Tab::Accounts | Tab::Forges | Tab::Models | Tab::Builders | Tab::Members | Tab::Limits => true,
    }
}

/// Opens the column without a project's kicker or tabs (client/pages/README.md, Settings).
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
    crate::bindings::link(b, name, to);
    if primary {
        b.style(look::SETTINGS_PRIMARY);
    } else {
        b.style(look::SETTINGS_BTN);
        b.style(look::SETTINGS_SMALL);
    }
    b.text(Run::Plain, name);
    b.close();
}

pub(crate) fn project_name(root: &Domain, w: &Words, project: u32) -> Box<[u8]> {
    match crate::common::project_name(root.pages().me(root.client()), project) {
        Some(name) => name,
        None => w.say(Word::Project, &[Hole::Number(u64::from(project))]),
    }
}

pub(crate) fn text(b: &mut Builder, role: Role, said: &[u8], style: Style) {
    b.open(role);
    b.style(style);
    b.name(said);
    b.text(Run::Plain, said);
    b.close();
}
