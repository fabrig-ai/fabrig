//! temper's frames and project switcher (temper/docs/design/client/view.md, 1 and 3).
pub(crate) use crate::bindings::Bindings;
use crate::bindings::{account_mark, ask_address, forge_mark};
use crate::column::unavailable;
pub use crate::look::owner;
use alloc::boxed::Box;
use skein_ui::{Builder, Region, Role, Run};
use temper_client_domain::{Address, Domain, pages::Act as PageAct};
use temper_client_domain::{
    WorkFilter,
    pages::{rail, switcher},
};
#[must_use]
pub fn link_region() -> Region {
    Region { owner: owner(), number: 0 }
}
fn text(builder: &mut Builder, role: Role, words: &[u8], style: Option<skein_ui::Style>) {
    builder.open(role);
    if let Some(style) = style {
        builder.style(style);
    }
    builder.name(words);
    builder.text(Run::Plain, words);
    builder.close();
}
fn action(builder: &mut Builder, role: Role, words: &[u8], act: PageAct, current: bool, bindings: &mut Bindings) {
    if role == Role::Link {
        crate::link(builder, words, bindings.insert(act));
        builder.text(Run::Plain, words);
    } else {
        builder.open(role);
        builder.name(words);
        builder.bind(bindings.insert(act));
    }
    if current {
        builder.state(skein_ui::State::Current);
    }
    builder.close();
}
pub(crate) fn unavailable_main(
    builder: &mut Builder,
    root: &Domain,
    words: &crate::words::Words,
    bindings: &mut Bindings,
) {
    use crate::words::Word;
    use jig_client::NotSignedIn;
    use skein_ui::phrases::Hole;
    if root.client().link().state == jig_client::Link::SignedOut {
        match root.client().link().not_signed_in {
            Some(NotSignedIn::Full { sign_ins, earliest, link_until }) => crate::outside::words(
                builder,
                words,
                Word::SignInFull,
                &[Hole::Number(u64::from(sign_ins)), Hole::Wall(earliest), Hole::Wall(link_until)],
            ),
            Some(NotSignedIn::Busy) => crate::outside::words(builder, words, Word::SignInBusy, &[]),
            Some(NotSignedIn::LinkUsed) => crate::outside::words(builder, words, Word::SignInLinkUsed, &[]),
            Some(NotSignedIn::LinkExpired) => crate::outside::words(builder, words, Word::SignInLinkExpired, &[]),
            Some(NotSignedIn::LinkUnknown) => crate::outside::words(builder, words, Word::SignInLinkUnknown, &[]),
            Some(NotSignedIn::NotReady) => crate::outside::words(builder, words, Word::SignInNotReady, &[]),
            Some(NotSignedIn::Approval(_)) | None => match words.shell {
                crate::words::Shell::Browser => crate::outside::words(builder, words, Word::SignedOutMain, &[]),
                crate::words::Shell::Terminal => {}
            },
        }
    } else {
        crate::outside::words(builder, words, Word::OutOfDateHow, &[]);
        if words.shell == crate::words::Shell::Browser {
            builder.open(Role::Slot);
            builder.style(crate::look::OUTSIDE_ACTS);
            builder.open(Role::Button);
            builder.style(crate::look::OUTSIDE_PRIMARY);
            builder.name(&words.say(Word::Reload, &[]));
            builder.bind(bindings.reload());
            builder.close();
            builder.close();
        }
    }
}
pub(crate) fn switcher(
    builder: &mut Builder,
    root: &Domain,
    words: &crate::words::Words,
    bindings: &mut Bindings,
    project: u32,
    sign_in: &crate::forge_sign_in::Forges,
    parts: rail::Parts,
) {
    if let Some(me) = root.pages().me(root.client()) {
        builder.open(Role::Disclosure);
        builder.style(crate::look::PROJECTS);
        let name = crate::pages::project_name(Some(me), project).expect("open project has a name");
        let across = rail::across(root);
        builder.name(&if across { words.say(crate::words::Word::AllProjects, &[]) } else { name });
        builder.open(Role::Slot);
        builder.style(crate::look::PROJECTS_MENU);
        if parts.all_projects {
            all_projects(builder, root, words, bindings, me.projects.len(), across);
        }
        for selected in [true, false] {
            for row in &me.projects {
                if (row.project == project) == selected {
                    let name = crate::pages::project_name(Some(me), row.project).expect("project from me has a name");
                    if selected && root.pages().switcher().renaming() == Some(row.project) {
                        rename_form(builder, root, words, &name, bindings);
                        continue;
                    }
                    let split = words.shell == crate::words::Shell::Browser;
                    if split {
                        builder.open(Role::Slot);
                        builder.style(crate::look::PROJECTS_ROW);
                    }
                    builder.open(Role::Button);
                    project_row(builder, root, words, row.project, &name);
                    if selected && !across {
                        builder.state(skein_ui::State::Pressed);
                    }
                    builder.bind(bindings.insert(PageAct::Project { project: row.project }));
                    builder.close();
                    if split {
                        project_repository(builder, root, words, row.project, sign_in);
                        builder.close();
                    }
                }
            }
        }
        builder.open(Role::Slot);
        builder.style(crate::look::PROJECTS_ACTS);
        for row in &me.projects {
            if row.project != project {
                continue;
            }
            let name = crate::pages::project_name(Some(me), row.project).expect("open project has a name");
            if root.pages().switcher().renaming().is_none() && row.role == jig_client::contract::HeldRole::Owner {
                builder.open(Role::Button);
                builder.style(crate::look::PROJECT_RENAME);
                builder.name(&words.say(crate::words::Word::RenameProject, &[skein_ui::phrases::Hole::Name(name)]));
                builder.bind(bindings.insert(PageAct::Switcher(switcher::Act::Rename)));
                builder.close();
            }
        }
        new_project_form(builder, root, words, bindings);
        let name = crate::pages::project_name(Some(me), project).expect("open project has a name");
        if parts.team {
            let label = words.say(crate::words::Word::TeamFor, &[skein_ui::phrases::Hole::Name(name.clone())]);
            switcher_phone_link(
                builder,
                &label,
                PageAct::Go { address: Address::Team { project, role: None } },
                if let Address::Team { project: at, .. } = root.pages().address() { *at == project } else { false },
                bindings,
            );
        }
        let label = words.say(crate::words::Word::RailHowFor, &[skein_ui::phrases::Hole::Name(name)]);
        switcher_phone_link(builder, &label, PageAct::Go { address: Address::Settings { project } }, false, bindings);
        switcher_phone_link(
            builder,
            &words.say(crate::words::Word::RailDeployment, &[]),
            PageAct::Go { address: Address::Accounts },
            false,
            bindings,
        );
        builder.close();
        builder.close();
        builder.close();
    }
}
fn switcher_phone_link(builder: &mut Builder, label: &[u8], act: PageAct, current: bool, bindings: &mut Bindings) {
    crate::link(builder, label, bindings.insert(act));
    builder.style(crate::look::PROJECTS_PHONE);
    builder.text(Run::Plain, label);
    if current {
        builder.state(skein_ui::State::Current);
    }
    builder.close();
}
fn all_projects(
    b: &mut Builder,
    root: &Domain,
    w: &crate::words::Words,
    bindings: &mut Bindings,
    projects: u32,
    across: bool,
) {
    use crate::words::{Shell, Word};
    use skein_ui::phrases::Hole;
    let name = w.say(Word::AllProjects, &[]);
    let count = w.say(Word::RailProjectsCount, &[Hole::Number(u64::from(projects))]);
    let address = match root.pages().address() {
        Address::Inbox { filter } => {
            let mut filter = *filter;
            filter.project = None;
            Address::Inbox { filter }
        }
        Address::AllHome
        | Address::Home { .. }
        | Address::Team { .. }
        | Address::Work { .. }
        | Address::Goals { .. }
        | Address::Changes { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Chat { .. }
        | Address::Talk { .. }
        | Address::Task { .. }
        | Address::Ask { .. }
        | Address::Tell { .. }
        | Address::Settings { .. }
        | Address::Role { .. }
        | Address::Rule { .. }
        | Address::Repositories { .. }
        | Address::Adopt { .. }
        | Address::Accounts
        | Address::Forges
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Told
        | Address::Remembered
        | Address::Members
        | Address::Limits
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. } => Address::AllHome,
    };
    if w.shell == Shell::Browser {
        b.open(Role::Slot);
        b.style(crate::look::PROJECTS_ROW);
    }
    b.open(Role::Button);
    b.name(&name);
    if across {
        b.state(skein_ui::State::Pressed);
    }
    b.bind(bindings.insert(PageAct::Go { address }));
    if w.shell == Shell::Terminal {
        w.text(b, Word::SwitcherRow, &[Hole::Name(name), Hole::Name(count.clone())]);
    }
    b.close();
    if w.shell == Shell::Browser {
        text(b, Role::Text, &count, Some(crate::look::PROJECTS_REPO));
        b.close();
    }
}
fn project_row(b: &mut Builder, root: &Domain, w: &crate::words::Words, project: u32, name: &[u8]) {
    use crate::words::{Shell, Word};
    use skein_ui::phrases::Hole;
    use temper_client_domain::forge::listed::{self, Held};
    b.name(name);
    if w.shell == Shell::Browser {
        return;
    }
    let repository = match listed::held(root.forges(), root.forge().playground(), project) {
        Held::Unknown => None,
        Held::None => Some(w.say(Word::SwitcherNone, &[])),
        Held::One { owner, name, .. } => {
            Some(w.say(Word::SwitcherRepository, &[Hole::Name(owner.into()), Hole::Name(name.into())]))
        }
        Held::Several { owner, name, more, .. } => Some(w.say(
            Word::SwitcherSeveral,
            &[Hole::Name(owner.into()), Hole::Name(name.into()), Hole::Number(u64::from(more))],
        )),
    };
    if let Some(repository) = repository {
        w.text(b, Word::SwitcherRow, &[Hole::Name(name.into()), Hole::Name(repository)]);
    }
}
fn project_repository(
    b: &mut Builder,
    root: &Domain,
    w: &crate::words::Words,
    project: u32,
    sign_in: &crate::forge_sign_in::Forges,
) {
    use crate::words::Word;
    use skein_ui::phrases::Hole;
    use temper_client_domain::forge::listed::{self, Held};
    let held = listed::held(root.forges(), root.forge().playground(), project);
    if held == Held::Unknown {
        return;
    }
    b.open(Role::Text);
    b.style(crate::look::PROJECTS_REPO);
    match held {
        Held::One { forge, owner, name } | Held::Several { forge, owner, name, .. } => {
            crate::forge::repository::name(b, w, forge, sign_in, owner, name);
        }
        Held::None => w.text(b, Word::SwitcherNone, &[]),
        Held::Unknown => {}
    }
    if let Held::Several { more, .. } = held {
        b.open(Role::Text);
        b.style(crate::look::PROJECTS_MORE);
        b.run(Run::Plain, &w.say(Word::SwitcherMore, &[Hole::Number(u64::from(more))]));
        b.close();
    }
    b.close();
}
// The switcher's forms (temper/docs/design/client/view.md, 3).
fn rename_form(b: &mut Builder, root: &Domain, words: &crate::words::Words, project: &[u8], bindings: &mut Bindings) {
    use crate::words::Word;
    let switcher = root.pages().switcher();
    let pending = switcher.pending().is_some();
    b.open(Role::Form);
    b.style(crate::look::PROJECT_RENAME_FORM);
    b.name(&words.say(Word::RenameProject, &[skein_ui::phrases::Hole::Name(project.into())]));
    b.open(Role::TextField);
    b.state(skein_ui::State::SubmitsOnEnter);
    b.name(&words.say(Word::Rename, &[]));
    b.value(switcher.rename_words(), temper_client_domain::pages::shown(root.pages()));
    b.focus(0);
    if pending {
        b.state(skein_ui::State::Disabled);
    }
    b.bind(bindings.insert(PageAct::Switcher(switcher::Act::RenameWords { words: Box::default() })));
    b.close();
    switcher_refusal(b, words, switcher, switcher::Form::Rename);
    switcher_button(b, words, Word::Save, switcher::Act::Save, pending, bindings);
    b.open(Role::Button);
    b.name(&words.say(Word::Cancel, &[]));
    if switcher.pending() == Some(switcher::Form::Rename) {
        b.state(skein_ui::State::Disabled);
    }
    b.bind(bindings.insert(PageAct::Switcher(switcher::Act::Cancel)));
    b.shortcut(skein_ui::Shortcut::ESCAPE);
    b.close();
    b.close();
}
fn new_project_form(b: &mut Builder, root: &Domain, words: &crate::words::Words, bindings: &mut Bindings) {
    use crate::words::Word;
    let switcher = root.pages().switcher();
    let pending = switcher.pending().is_some();
    b.open(Role::Form);
    b.style(crate::look::PROJECT_NEW);
    b.name(&words.say(Word::NewProject, &[]));
    b.open(Role::TextField);
    b.state(skein_ui::State::SubmitsOnEnter);
    b.name(&words.say(Word::NewProject, &[]));
    b.placeholder(&words.say(Word::NewProjectPlaceholder, &[]));
    b.value(switcher.new_words(), temper_client_domain::pages::shown(root.pages()));
    if pending {
        b.state(skein_ui::State::Disabled);
    }
    b.bind(bindings.insert(PageAct::Switcher(switcher::Act::NewWords { words: Box::default() })));
    b.close();
    switcher_refusal(b, words, switcher, switcher::Form::New);
    switcher_button(b, words, Word::CreateProject, switcher::Act::Create, pending, bindings);
    b.close();
}
fn switcher_button(
    b: &mut Builder,
    words: &crate::words::Words,
    word: crate::words::Word,
    act: switcher::Act,
    pending: bool,
    bindings: &mut Bindings,
) {
    b.open(Role::Button);
    b.name(&words.say(word, &[]));
    if pending {
        b.state(skein_ui::State::Disabled);
    }
    b.bind(bindings.insert(PageAct::Switcher(act)));
    b.close();
}
fn switcher_refusal(b: &mut Builder, words: &crate::words::Words, switcher: &switcher::Switcher, form: switcher::Form) {
    if let Some((refused, outcome)) = switcher.refusal()
        && refused == form
    {
        crate::pages::chats::outcome_text(b, words, outcome, words.name_bytes);
    }
}
pub(crate) fn behind(root: &Domain) -> bool {
    let pages = root.pages();
    let me_behind = match root.client().document(temper_client_domain::pages::me_handle()) {
        Some(document) => document.state == jig_client::State::Behind,
        None => false,
    };
    let page_behind = match pages.inbox() {
        Some(inbox) => match root.client().document(inbox.entries) {
            Some(document) => document.state == jig_client::State::Behind,
            None => false,
        },
        None => false,
    };
    me_behind || page_behind
}

/// The project a page's frame is the rail of (temper/docs/design/client/view.md, 1).
pub(crate) fn rail_project(root: &Domain, words: &crate::words::Words, sign_in: &crate::sign_in::Ways) -> Option<u32> {
    if let Address::Welcome { .. } = root.pages().address() {
        return None;
    }
    if unavailable(root).is_some()
        || root.authorization().is_some()
        || root.approval().is_some()
        || crate::sign_in::offered(root, words, sign_in)
    {
        return None;
    }
    root.pages().me(root.client())?;
    root.pages().project()
}

/// The rail (temper/docs/design/client/view.md, 3).
#[expect(clippy::too_many_arguments, reason = "the rail carries page-owned forge facts beside its frame bindings")]
pub(crate) fn rail(
    b: &mut Builder,
    root: &Domain,
    w: &crate::words::Words,
    bindings: &mut Bindings,
    forge: &mut crate::forge::bindings::Bindings,
    project: u32,
    times: &mut crate::words::Times,
    sign_in: &crate::forge_sign_in::Forges,
) {
    use crate::words::{Shell, Word};
    let parts = rail::parts(&rail::facts(root, project, times.now));
    b.open(Role::Navigation);
    b.style(crate::look::RAIL);
    let name = crate::pages::project_name(root.pages().me(root.client()), project).expect("open project has a name");
    b.name(&name);
    let (home, _) = targets(root, project);
    crate::link(b, &w.say(Word::BrandHome, &[]), bindings.insert(PageAct::Go { address: home }));
    b.style(crate::look::BRAND);
    w.text(b, Word::Brand, &[]);
    b.close();
    if w.shell == Shell::Browser
        && (root.pages().chat().is_some() || root.pages().address() == &(Address::NewChat { draft: true }))
    {
        b.open(Role::Disclosure);
        b.name(&w.say(Word::RailUnfold, &[]));
        b.state(skein_ui::State::FoldsAway);
        b.style(crate::look::RAIL_UNFOLD);
        b.close();
    }
    switcher(b, root, w, bindings, project, sign_in, parts);
    if w.shell == Shell::Browser && parts.away {
        away(b, root, w, bindings, times, project, crate::look::AWAY_CHIP);
    }
    b.open(Role::Slot);
    b.style(crate::look::RAIL_NAV);
    entries(b, root, w, bindings, project, parts);
    b.close();
    new_box(b, root, w, bindings, project, parts);
    b.open(Role::Slot);
    b.style(crate::look::RAIL_FOOT);
    if parts.away {
        away(b, root, w, bindings, times, project, crate::look::AWAY);
    }
    if root.forge().playground_running() && !root.forge().leaving() {
        let name = w.say(Word::BannerSetUp, &[]);
        crate::link(b, &name, forge.insert(crate::forge::bindings::Meaning::EndPlayground));
        b.style(crate::look::LEAVE_PLAYGROUND);
        b.text(Run::Plain, &name);
        b.close();
    }
    b.open(Role::Region);
    b.region(link_region());
    let (word, style) = match root.client().link().state {
        jig_client::Link::Offline => (Word::Offline, crate::look::OFFLINE),
        jig_client::Link::Live => (Word::Live, crate::look::LIVE),
        jig_client::Link::Starting | jig_client::Link::SignedOut | jig_client::Link::OutOfDate => {
            (Word::Connecting, crate::look::CONNECTING)
        }
    };
    text(b, Role::Status, &w.say(word, &[]), Some(style));
    let behind = behind(root);
    if behind && w.shell == Shell::Terminal {
        text(b, Role::Status, &w.say(Word::Behind, &[]), Some(crate::look::BEHIND));
    }
    b.close();
    if behind && w.shell == Shell::Browser {
        text(b, Role::Status, &w.say(Word::Behind, &[]), Some(crate::look::BEHIND));
    }
    deployment_link(b, root, w, bindings);
    person(b, root, w, bindings);
    if root.client().link().sign_out_lost {
        text(b, Role::Alert, &w.say(Word::SignOutLost, &[]), None);
    }
    b.close();
    if w.shell == Shell::Browser {
        b.open(Role::Button);
        b.style(crate::look::NIGHT_SWITCH);
        b.name(&w.say(Word::NightMode, &[]));
        if root.pages().look().night {
            b.state(skein_ui::State::Pressed);
        }
        b.bind(bindings.insert(PageAct::Terminal(temper_client_domain::pages::terminal::Act::Night)));
        b.close();
    }
    b.close();
}

/// The rail's person: a line in the terminal, a menu in a browser
/// (temper/docs/design/client/view.md, 3).
fn person(b: &mut Builder, root: &Domain, w: &crate::words::Words, bindings: &mut Bindings) {
    use crate::words::{Shell, Word};
    let Some(me) = root.pages().me(root.client()) else { return };
    match w.shell {
        Shell::Terminal => text(b, Role::Status, me.party.name.bytes(), None),
        Shell::Browser => {
            b.open(Role::Slot);
            b.style(crate::look::RAIL_PERSON);
            b.open(Role::Menu);
            b.name(me.party.name.bytes());
            for (word, act) in [
                (Word::SettingsAccountsTab, PageAct::Go { address: Address::Accounts }),
                (Word::RememberedTitle, PageAct::Go { address: Address::Remembered }),
                (Word::ToldTitle, PageAct::Go { address: Address::Told }),
                (Word::SignOut, PageAct::SignOut),
            ] {
                b.open(Role::MenuItem);
                b.name(&w.say(word, &[]));
                b.bind(bindings.insert(act));
                b.close();
            }
            b.close();
            b.close();
        }
    }
}

/// The shared switch and browser chip (temper/docs/design/client/pages/README.md, Away).
fn away(
    b: &mut Builder,
    root: &Domain,
    w: &crate::words::Words,
    bindings: &mut Bindings,
    times: &mut crate::words::Times,
    project: u32,
    style: skein_ui::Style,
) {
    use crate::words::{TimeForm, Word};
    use temper_client_domain::pages::{away::Act, away_all, rail};
    let across = if rail::across(root) { Some(away_all::across(root, times.now)) } else { None };
    let (on, until, act) = match across {
        Some(counts) => (counts.projects > 0 && counts.on == counts.projects, counts.until, Act::ToggleAll),
        None => (
            root.away_on(project, times.now),
            match root.forge().away(project) {
                Some(row) => row.until,
                None => None,
            },
            Act::Toggle { project },
        ),
    };
    if let Some(until) = until
        && (on || across.is_some())
    {
        times.next = [times.next, Some(until)].into_iter().flatten().min();
    }
    let said = if let Some(counts) = across
        && counts.on > 0
        && counts.on < counts.projects
    {
        w.say(
            Word::AwayAcross,
            &[
                skein_ui::phrases::Hole::Number(u64::from(counts.on)),
                skein_ui::phrases::Hole::Number(u64::from(counts.projects)),
            ],
        )
    } else if on {
        match until {
            Some(until) => {
                let clock = if across.is_some() { w.turn_clock(until) } else { times.say(w, TimeForm::Clock, until) };
                w.say(Word::AwayUntil, &[skein_ui::phrases::Hole::Name(clock)])
            }
            None => w.say(Word::Away, &[]),
        }
    } else {
        w.say(if style == crate::look::AWAY_CHIP { Word::AwayChipOff } else { Word::Away }, &[])
    };
    b.open(Role::Button);
    b.style(style);
    b.name(&said);
    b.bind(bindings.insert(PageAct::Away(act)));
    if on {
        b.state(skein_ui::State::Pressed);
    }
    if root.pages().away_busy() {
        b.state(skein_ui::State::Disabled);
    }
    if style == crate::look::AWAY {
        b.open(Role::Slot);
        b.style(crate::look::AWAY_TRACK);
        b.close();
    }
    b.close();
}

/// A phone's tab bar, the browser's alone (temper/docs/design/client/view.md, 3).
pub(crate) fn tabbar(
    b: &mut Builder,
    root: &Domain,
    w: &crate::words::Words,
    bindings: &mut Bindings,
    project: u32,
    wall: skein_lib::Wall,
) {
    use crate::words::Word;
    let tabs = rail::tabs(&rail::parts(&rail::facts(root, project, wall)));
    let current = shown(root);
    let (home, needs) = targets(root, project);
    b.open(Role::Navigation);
    b.style(crate::look::TABBAR);
    b.name(&w.say(Word::Sections, &[]));
    action(b, Role::Link, &w.say(Word::RailHome, &[]), PageAct::Go { address: home }, current.home, bindings);
    if tabs.needs {
        let name = needs_name(w, tabs.count, tabs.cut, [Word::NeedsTab, Word::NeedsTabCount, Word::NeedsTabCountCut]);
        needs_link(b, bindings, needs, &name, tabs.count, tabs.cut, current.inbox);
    }
    if tabs.work {
        action(
            b,
            Role::Link,
            &w.say(Word::RailWork, &[]),
            PageAct::Go { address: Address::Work { project, filter: WorkFilter::All } },
            current.work,
            bindings,
        );
    }
    if tabs.ask {
        if ask_current(root, project) {
            b.style(crate::look::TABBAR_ASK);
        }
        ask_link(b, root, w, bindings, project, Word::AskTab);
    }
    if tabs.deployment {
        b.style(crate::look::TABBAR_BEGINNING);
        deployment_link(b, root, w, bindings);
    }
    b.close();
}

/// Home and Needs you stay across projects together (temper/docs/design/client/view.md, 3).
fn targets(root: &Domain, project: u32) -> (Address, Option<u32>) {
    if rail::across(root) { (Address::AllHome, None) } else { (Address::Home { project }, Some(project)) }
}

#[expect(clippy::struct_excessive_bools, reason = "one current state per navigation entry")]
struct Shown {
    home: bool,
    inbox: bool,
    work: bool,
    how: bool,
    team: bool,
}
#[expect(clippy::match_like_matches_macro, reason = "the step crate excludes matches!; its address check uses if let")]
fn shown(root: &Domain) -> Shown {
    let (home, inbox, work, how) = match root.pages().address() {
        Address::AllHome | Address::Home { .. } => (true, false, false, false),
        Address::Inbox { .. } => (false, true, false, false),
        Address::Work { .. }
        | Address::Goals { .. }
        | Address::Changes { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Task { .. } => (false, false, true, false),
        Address::Chat { .. } | Address::Talk { .. } => (false, false, !crate::pages::chat::asking(root), false),
        Address::Settings { .. }
        | Address::Role { .. }
        | Address::Rule { .. }
        | Address::Repositories { .. }
        | Address::Adopt { .. } => (false, false, false, true),
        Address::Ask { .. }
        | Address::Tell { .. }
        | Address::Team { .. }
        | Address::Accounts
        | Address::Forges
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Told
        | Address::Remembered
        | Address::Members
        | Address::Limits
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. } => (false, false, false, false),
    };
    Shown { home, inbox, work, how, team: if let Address::Team { .. } = root.pages().address() { true } else { false } }
}
fn entries(
    b: &mut Builder,
    root: &Domain,
    w: &crate::words::Words,
    bindings: &mut Bindings,
    project: u32,
    parts: rail::Parts,
) {
    use crate::words::Word;
    let Shown { home, inbox, work, how, team } = shown(root);
    let (home_address, needs) = targets(root, project);
    action(b, Role::Link, &w.say(Word::RailHome, &[]), PageAct::Go { address: home_address }, home, bindings);
    if parts.needs_you {
        let name = needs_name(w, parts.count, parts.cut, [Word::NeedsYou, Word::NeedsYouCount, Word::NeedsYouCountCut]);
        needs_link(b, bindings, needs, &name, parts.count, parts.cut, inbox);
    }
    if parts.work {
        action(
            b,
            Role::Link,
            &w.say(Word::RailWork, &[]),
            PageAct::Go { address: Address::Work { project, filter: WorkFilter::All } },
            work,
            bindings,
        );
    }
    if parts.team {
        action(
            b,
            Role::Link,
            &w.say(Word::TeamTitle, &[]),
            PageAct::Go { address: Address::Team { project, role: None } },
            team,
            bindings,
        );
    }
    if parts.ask {
        ask_link(b, root, w, bindings, project, Word::AskTitle);
    }
    if parts.how {
        b.open(Role::Slot);
        b.style(crate::look::RAIL_SEP);
        b.close();
        action(
            b,
            Role::Link,
            &w.say(Word::SettingsHowTab, &[]),
            PageAct::Go { address: Address::Settings { project } },
            how,
            bindings,
        );
    }
}
fn needs_name(w: &crate::words::Words, count: u32, cut: bool, words: [crate::words::Word; 3]) -> Box<[u8]> {
    let [plain, counted, limited] = words;
    if count == 0 {
        w.say(plain, &[])
    } else {
        w.say(if cut { limited } else { counted }, &[skein_ui::phrases::Hole::Number(u64::from(count))])
    }
}
fn needs_link(
    b: &mut Builder,
    bindings: &mut Bindings,
    project: Option<u32>,
    name: &[u8],
    count: u32,
    cut: bool,
    current: bool,
) {
    crate::link(
        b,
        name,
        bindings.insert(PageAct::Go {
            address: Address::Inbox {
                filter: temper_client_domain::address::Filter { project, kind: None, goal: None },
            },
        }),
    );
    if current {
        b.state(skein_ui::State::Current);
    }
    if count == 0 {
        b.text(Run::Plain, name);
    } else {
        let number = skein_lib::Decimal::of(u64::from(count));
        let suffix = number.as_bytes().len().checked_add(usize::from(cut)).expect("count suffix fits");
        let prefix = name.len().checked_sub(suffix).expect("count is the phrase suffix");
        b.text(Run::Plain, name.get(..prefix).expect("prefix fits"));
        b.open(Role::Text);
        b.style(crate::look::NAV_COUNT);
        b.run(Run::Plain, name.get(prefix..).expect("count suffix fits"));
        b.close();
    }
    b.close();
}
fn ask_current(root: &Domain, project: u32) -> bool {
    root.pages().address() == &(Address::Ask { project })
        || root.pages().address() == &(Address::Tell { project })
        || crate::pages::chat::asking(root)
}
fn ask_link(
    b: &mut Builder,
    root: &Domain,
    w: &crate::words::Words,
    bindings: &mut Bindings,
    project: u32,
    word: crate::words::Word,
) {
    crate::link(
        b,
        &w.say(crate::words::Word::AskTitle, &[]),
        bindings.insert(PageAct::Go { address: ask_address(root, project) }),
    );
    w.text(b, word, &[]);
    if ask_current(root, project) {
        b.state(skein_ui::State::Current);
    }
    b.close();
}
fn new_box(
    b: &mut Builder,
    root: &Domain,
    w: &crate::words::Words,
    bindings: &mut Bindings,
    project: u32,
    parts: rail::Parts,
) {
    use crate::words::Word;
    match parts.new {
        rail::New::Hidden => return,
        rail::New::Change | rail::New::DraftOrChange => {}
    }
    b.open(Role::Slot);
    b.style(crate::look::RAIL_NEW);
    match parts.new {
        rail::New::Hidden => {}
        rail::New::Change => action(
            b,
            Role::Link,
            &w.say(Word::RailNewChange, &[]),
            PageAct::Go { address: Address::NewChat { draft: false } },
            false,
            bindings,
        ),
        rail::New::DraftOrChange => {
            b.open(Role::Menu);
            b.name(&w.say(Word::RailNewDraftOrChange, &[]));
            for (name, description, address) in [
                (Word::DraftDraft, Word::RailDraftDescription, Address::NewChat { draft: true }),
                (Word::DelegationLineChange, Word::RailChangeDescription, Address::NewChat { draft: false }),
                (Word::AskTitle, Word::RailAskDescription, ask_address(root, project)),
            ] {
                if name == Word::AskTitle && !parts.ask {
                    continue;
                }
                b.open(Role::MenuItem);
                b.name(&w.say(name, &[]));
                b.description(&w.say(description, &[]));
                b.bind(bindings.insert(PageAct::Go { address }));
                b.close();
            }
            b.close();
        }
    }
    b.close();
}
fn deployment_link(b: &mut Builder, root: &Domain, w: &crate::words::Words, bindings: &mut Bindings) {
    let name = w.say(crate::words::Word::RailDeployment, &[]);
    crate::link(b, &name, bindings.insert(PageAct::Go { address: Address::Accounts }));
    b.text(Run::Plain, &name);
    let current = match root.pages().address() {
        Address::Accounts
        | Address::Forges
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Members
        | Address::Limits
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. } => true,
        Address::Told
        | Address::Remembered
        | Address::Home { .. }
        | Address::Inbox { .. }
        | Address::Work { .. }
        | Address::Goals { .. }
        | Address::Changes { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Chat { .. }
        | Address::Talk { .. }
        | Address::Task { .. }
        | Address::Ask { .. }
        | Address::Tell { .. }
        | Address::Settings { .. }
        | Address::Role { .. }
        | Address::Rule { .. }
        | Address::Repositories { .. }
        | Address::Team { .. }
        | Address::AllHome
        | Address::Adopt { .. }
        | Address::Welcome { .. } => false,
    };
    if current {
        b.state(skein_ui::State::Current);
    }
    forge_mark(b, root, w, true);
    account_mark(b, root, w, true);
    b.close();
}

/// The playground's reset line (temper/docs/design/client/view.md, 1).
pub(crate) fn playground_banner(
    b: &mut Builder,
    root: &Domain,
    words: &crate::words::Words,
    bindings: &mut crate::forge::bindings::Bindings,
) {
    use crate::words::Word;
    if unavailable(root).is_some() || (!root.forge().playground_running() && !root.forge().leaving()) {
        return;
    }
    b.open(Role::Region);
    b.style(crate::look::ACCOUNTS);
    b.name(&words.say(Word::Playground, &[]));
    b.open(Role::Paragraph);
    if root.forge().leaving() {
        words.text(b, Word::PlaygroundLeaving, &[]);
    } else {
        words.text(b, Word::PlaygroundBanner, &[]);
        b.text(Run::Plain, b" ");
        let name = words.say(Word::PlaygroundSetUp, &[]);
        crate::link(b, &name, bindings.insert(crate::forge::bindings::Meaning::EndPlayground));
        b.text(Run::Plain, &name);
        b.close();
    }
    b.close();
    b.close();
}

/// The session's transient status and held answer's Undo (client/pages/README.md).
#[expect(clippy::too_many_lines, reason = "the session toast draws every answered act in one exhaustive dispatch")]
pub(crate) fn toast(
    b: &mut Builder,
    root: &Domain,
    w: &crate::words::Words,
    bindings: &mut Bindings,
    times: &mut crate::words::Times,
) {
    use crate::words::Word;
    use temper_client_domain::pages::needs::{self, Toasted};
    let session = root.pages().needs();
    let Some(toast) = session.toast() else { return };
    b.open(Role::Region);
    b.name(&w.say(Word::ToastRegion, &[]));
    let (said, undo) = match toast {
        Toasted::Answered { object, label } => {
            let label = crate::pages::ask::answered_label(w, label);
            (w.say(Word::ToastAnswered, &[skein_ui::phrases::Hole::Name(label)]), Some(*object))
        }
        Toasted::Way { name, revision, kept } => {
            let word = if *kept { Word::ToastWayKept } else { Word::ToastWayUsed };
            (
                w.say(
                    word,
                    &[
                        skein_ui::phrases::Hole::Name(name.clone()),
                        skein_ui::phrases::Hole::Number(u64::from(*revision)),
                    ],
                ),
                None,
            )
        }
        Toasted::AreaHanded { title, revision } => (
            w.say(
                Word::HandToast,
                &[skein_ui::phrases::Hole::Name(title.clone()), skein_ui::phrases::Hole::Number(u64::from(*revision))],
            ),
            None,
        ),
        Toasted::AreasGivenBack { revision } => {
            (w.say(Word::GiveToast, &[skein_ui::phrases::Hole::Number(u64::from(*revision))]), None)
        }
        Toasted::TellFiled { project, goes } => (crate::pages::tell::filed(root, w, *project, goes.as_ref()), None),
        Toasted::AreasProposed => (w.say(Word::HandProposed, &[]), None),
        Toasted::StopApplied => (w.say(Word::StopAskingApplied, &[]), None),
        Toasted::StopProposed => (w.say(Word::StopAskingProposed, &[]), None),
        Toasted::KeptAsking => (w.say(Word::KeepAskingKept, &[]), None),
        Toasted::KeepAskingRefused => (w.say(Word::KeepAskingFailed, &[]), None),
        Toasted::Later => (w.say(Word::ToastLater, &[]), None),
        Toasted::NotSent => (w.say(Word::ToastNotSent, &[]), None),
        Toasted::Paused => (w.say(Word::ToastPaused, &[]), None),
        Toasted::LeftTalk => (w.say(Word::LeftTalk, &[]), None),
        Toasted::Resumed => (w.say(Word::ToastResumed, &[]), None),
        Toasted::AwayOn { project, until } => {
            let name =
                crate::pages::project_name(root.pages().me(root.client()), *project).expect("Away project has a name");
            let said = match until {
                Some(until) => {
                    let clock = times.say(w, crate::words::TimeForm::Clock, *until);
                    w.say(
                        Word::ToastAwayOnUntil,
                        &[skein_ui::phrases::Hole::Name(clock), skein_ui::phrases::Hole::Name(name)],
                    )
                }
                None => w.say(Word::ToastAwayOn, &[skein_ui::phrases::Hole::Name(name)]),
            };
            (said, None)
        }
        Toasted::AwayOff => (w.say(Word::ToastAwayOff, &[]), None),
        Toasted::AwayRefused => (w.say(Word::ToastAwayRefused, &[]), None),
        Toasted::AwayAll { on, changed, sent } => {
            let said = if changed < sent {
                w.say(
                    Word::ToastAwayAllPartial,
                    &[
                        skein_ui::phrases::Hole::Number(u64::from(*changed)),
                        skein_ui::phrases::Hole::Number(u64::from(*sent)),
                    ],
                )
            } else {
                let word = if *on {
                    if *changed == 1 { Word::ToastAwayAllOnOne } else { Word::ToastAwayAllOn }
                } else if *changed == 1 {
                    Word::ToastAwayAllOffOne
                } else {
                    Word::ToastAwayAllOff
                };
                w.say(word, &[skein_ui::phrases::Hole::Number(u64::from(*changed))])
            };
            (said, None)
        }
        Toasted::Overturned => (w.say(Word::ToastOverturned, &[]), None),
        Toasted::OverturnEnded => (w.say(Word::ToastOverturnEnded, &[]), None),
        Toasted::OverturnAlready => (w.say(Word::ToastOverturnAlready, &[]), None),
        Toasted::Proposed { .. } | Toasted::Refused { .. } => (Box::default(), None),
    };
    b.open(Role::Status);
    b.style(crate::look::TOAST);
    match toast {
        Toasted::Proposed { to } => {
            proposed(b, w, to);
        }
        Toasted::Refused { outcome } => {
            crate::confirm::outcome_words(b, w, outcome);
            if let jig_client::Outcome::Answered(jig_client::Answered::Jig(jig_client::contract::Answer::Refused {
                refusal: jig_client::contract::Refusal::Moved { .. },
                ..
            })) = outcome
            {
                action(
                    b,
                    Role::Button,
                    &w.say(Word::LookAgain, &[]),
                    PageAct::Needs(needs::Act::Dismiss),
                    false,
                    bindings,
                );
            }
        }
        Toasted::TellFiled { .. }
        | Toasted::AreaHanded { .. }
        | Toasted::AreasGivenBack { .. }
        | Toasted::AreasProposed
        | Toasted::StopApplied
        | Toasted::StopProposed
        | Toasted::KeptAsking
        | Toasted::KeepAskingRefused
        | Toasted::Way { .. }
        | Toasted::AwayOn { .. }
        | Toasted::AwayOff
        | Toasted::AwayAll { .. }
        | Toasted::AwayRefused
        | Toasted::Overturned
        | Toasted::OverturnEnded
        | Toasted::OverturnAlready
        | Toasted::Answered { .. }
        | Toasted::Later
        | Toasted::NotSent
        | Toasted::Paused
        | Toasted::LeftTalk
        | Toasted::Resumed => b.text(Run::Plain, &said),
    }
    if let Toasted::AwayOn { project, .. } = toast {
        action(
            b,
            Role::Button,
            &w.say(Word::AwayLimits, &[]),
            PageAct::Away(temper_client_domain::pages::away::Act::Limits { project: *project }),
            false,
            bindings,
        );
    }
    if let Some(object) = undo
        && session.held(object)
    {
        action(
            b,
            Role::Button,
            &w.say(Word::ToastUndo, &[]),
            PageAct::Needs(needs::Act::Undo { object }),
            false,
            bindings,
        );
    }
    b.close();
    b.close();
}

fn proposed(b: &mut Builder, w: &crate::words::Words, to: &jig_client::contract::Recipient) {
    use crate::{
        cards::{P, words::say},
        words::Word,
    };
    use jig_client::contract::Recipient;
    use skein_ui::phrases::Hole;
    w.text(b, Word::ToastProposed, &[]);
    match to {
        Recipient::Party(number) => b.text(Run::Plain, &say(w, P::ToPerson, &[Hole::Number(*number)])),
        Recipient::Task(number) => b.text(Run::Plain, &say(w, P::ToTask, &[Hole::Number(*number)])),
        Recipient::Role { project, role } => {
            b.text(
                Run::Plain,
                &say(w, P::ToRole, &[Hole::Number(u64::from(*role)), Hole::Number(u64::from(*project))]),
            );
        }
        Recipient::Roles { project, roles } => {
            for role in roles {
                b.text(
                    Run::Plain,
                    &say(w, P::ToRole, &[Hole::Number(u64::from(*role)), Hole::Number(u64::from(*project))]),
                );
            }
        }
    }
    w.text(b, Word::ToastProposedEnd, &[]);
}

/// Ask opens the person's newest open conversation, or its first-message page.
#[cfg(test)]
mod start_goal_tests {
    use super::*;
    use crate::tests::fixtures as f;
    use jig_client::contract::Recipient;
    use skein_lib::List;
    #[test]
    fn proposed_toast_names_the_recipient_and_waits_for_acceptance() {
        for w in [f::words(), f::browser_words()] {
            for (to, expected) in [
                (Recipient::Party(4), "Proposed to Person 4: it starts once accepted."),
                (Recipient::Task(4), "Proposed to Task 4: it starts once accepted."),
                (Recipient::Role { project: 1, role: 2 }, "Proposed to Role 2 in project 1: it starts once accepted."),
                (
                    Recipient::Roles { project: 1, roles: List::from_boxed(Box::new([2])) },
                    "Proposed to Role 2 in project 1: it starts once accepted.",
                ),
            ] {
                let mut b = Builder::new(&f::tree_limits());
                b.open(Role::Status);
                proposed(&mut b, &w, &to);
                b.close();
                let tree = b.finish();
                assert_eq!(f::text(&tree).as_ref(), expected.as_bytes());
            }
        }
    }
}
