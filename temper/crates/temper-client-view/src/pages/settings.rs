//! How work runs and Deployment's frames (temper/docs/design/client/pages/README.md, Settings and Deployment).
pub(crate) use crate::common::settings::{
    FlowTab, FlowWay, Tab, button_to, open, open_alone, path, project_name, quiet_band, section_head, text, way_path,
};
use crate::{
    bindings::Bindings,
    forge::bindings::{Bindings as ForgeBindings, Meaning, WaysRow},
    look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Address, ChatsFilter, Domain,
    forge::{
        self, listed,
        ways::{self, Asked, Control, Route, Ways},
    },
    pages::{Act, away::Act as Away},
};

pub(super) fn main(b: &mut Builder) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::SETTINGS);
}
pub(super) fn missing(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) -> bool {
    if root.pages().me(root.client()).is_some()
        && crate::pages::project_name(root.pages().me(root.client()), project).is_none()
    {
        let word = if let Address::Settings { .. } = root.pages().address() {
            match temper_client_domain::pages::team::missing(root, project) {
                temper_client_domain::pages::team::Missing::Reading => return true,
                temper_client_domain::pages::team::Missing::Deleted => Word::ProjectDeleted,
                temper_client_domain::pages::team::Missing::NotOurs => Word::ProjectNotYours,
            }
        } else {
            Word::ProjectNotYours
        };
        crate::cards::text(b, Role::Paragraph, &w.say(word, &[Hole::Number(u64::from(project))]));
        link(b, w, bindings, Word::Chats, Address::Chats { filter: ChatsFilter::All });
        return true;
    }
    false
}
pub(crate) fn how_work_runs(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    forge: &mut ForgeBindings,
    project: u32,
    wall: skein_lib::Wall,
) {
    main(b);
    open(b, root, w, bindings, project, Tab::HowWorkRuns);
    if !missing(b, root, w, bindings, project) {
        b.open(Role::Slot);
        b.style(look::SETTINGS_BODY);
        let stored = root.forge().ways(project);
        let shown = match root.forge().ways_asked(Control::Ways) {
            Some(Asked::Ways(value)) => *value,
            Some(Asked::Questions(_) | Asked::Classes(_)) | None => stored.unwrap_or(Ways::STOCK),
        };
        super::how::path(b, root, w, bindings, project);
        super::how::reaches(b, root, w, bindings, forge, project, wall);
        super::how::rules(b, root, w, bindings, project);
        super::how::follows(b, root, w, project);
        super::how::roles(b, root, w, bindings, project);
        super::how::budgets(b, root, w, project);
        shows(b, root, w, forge, project, stored.is_none(), shown);
        if shown.goals {
            working(b, root, w, forge, project, shown);
        }
        if temper_client_domain::pages::delete_project::offered(root.pages().me(root.client()), project) {
            b.open(Role::Button);
            b.name(&w.say(Word::DeleteProject, &[]));
            b.bind(bindings.insert(Act::DeleteProject { project }));
            b.close();
        }
        b.close();
    }
    b.close();
    crate::confirm::delete_project(b, root, w, bindings);
    b.close();
}

/// A setting's label and small words; the caller draws its right-hand control,
/// then closes the row with its refusal (client/pages/settings.md, 2).
fn row(b: &mut Builder, w: &Words, label: &[u8], small: Option<Word>) {
    let small = match small {
        Some(word) => w.say(word, &[]),
        None => Box::default(),
    };
    row_line(b, label, &small);
}
fn row_line(b: &mut Builder, label: &[u8], small: &[u8]) {
    b.open(Role::Slot);
    b.style(look::SETTINGS_SETTING);
    b.open(Role::Slot);
    crate::cards::text(b, Role::Text, label);
    if !small.is_empty() {
        text(b, Role::Text, small, look::SETTINGS_DETAIL);
    }
    b.close();
}
fn section(b: &mut Builder, w: &Words, title: Word) {
    b.open(Role::Slot);
    b.style(look::SETTINGS_SECTION);
    text(b, Role::Heading { level: 2 }, &w.say(title, &[]), look::SETTINGS_SECTION_TITLE);
}
fn refused(b: &mut Builder, root: &Domain, w: &Words, control: Control) {
    if let Some(refusal) = root.forge().ways_refused(control) {
        b.open(Role::Slot);
        b.style(look::SETTINGS_REFUSAL);
        crate::cards::refusal(b, w, refusal);
        b.close();
    }
}
fn ways_refused(b: &mut Builder, root: &Domain, w: &Words, bindings: &ForgeBindings, project: u32, row: WaysRow) {
    let last = match bindings.ways_row {
        Some((in_project, last)) if in_project == project => last,
        Some((_, _)) | None => WaysRow::Drafts,
    };
    if last == row {
        refused(b, root, w, Control::Ways);
    }
}
fn shows(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut ForgeBindings,
    project: u32,
    unknown: bool,
    shown: Ways,
) {
    section(b, w, Word::SettingsShows);
    let disabled = unknown || root.forge().ways_asked(Control::Ways).is_some();
    for (key, label, line, value, asked) in [
        (
            WaysRow::Drafts,
            Word::SettingsDrafts,
            Word::SettingsDraftsLine,
            shown.drafts,
            Ways { drafts: !shown.drafts, ..shown },
        ),
        (
            WaysRow::Goals,
            Word::SettingsGoals,
            Word::SettingsGoalsLine,
            shown.goals,
            Ways { goals: !shown.goals, ..shown },
        ),
        (
            WaysRow::Changes,
            Word::SettingsChanges,
            Word::SettingsChangesLine,
            shown.changes,
            Ways { changes: !shown.changes, ..shown },
        ),
    ] {
        let label = w.say(label, &[]);
        row(b, w, &label, Some(line));
        b.open(Role::Slot);
        b.style(look::SETTINGS_TOGGLE_ROW);
        let word = match (key, value) {
            (WaysRow::Drafts, true) => Word::SettingsOn,
            (WaysRow::Drafts, false) => Word::SettingsOff,
            (WaysRow::Goals | WaysRow::Changes | WaysRow::Way(_), true) => Word::SettingsShown,
            (WaysRow::Goals | WaysRow::Changes | WaysRow::Way(_), false) => Word::SettingsHidden,
        };
        crate::cards::text(b, Role::Text, &w.say(word, &[]));
        b.open(Role::Check);
        b.name(&label);
        b.style(look::SETTINGS_SWITCH);
        b.bind(bindings.insert_way(ways::Act::Set { project, asked: Asked::Ways(asked) }, key));
        if value {
            b.state(State::Pressed);
        }
        if disabled {
            b.state(State::Disabled);
        }
        b.close();
        b.close();
        ways_refused(b, root, w, bindings, project, key);
        b.close();
    }
    b.close();
}
fn working(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, project: u32, shown: Ways) {
    section(b, w, Word::SettingsWays);
    b.open(Role::ChoiceGroup);
    b.name(&w.say(Word::SettingsWays, &[]));
    let selected = match root.pages().me(root.client()) {
        Some(me) => ways::default_way(me, project, &shown),
        None => None,
    };
    let offered = match root.pages().me(root.client()) {
        Some(me) => crate::common::goals::ways(me, project, w),
        None => skein_lib::List::uncapped(),
    };
    for way in &offered {
        // TaskLine records no template: there is no delivered usage count.
        row_line(b, &way.name, way.line.as_deref().unwrap_or(&[]));
        b.open(Role::Slot);
        b.style(look::SETTINGS_TOGGLE_ROW);
        b.open(Role::Choice);
        b.name(&w.say(Word::SettingsDefault, &[]));
        if selected == Some(way.number) {
            b.state(State::Selected);
        }
        if root.forge().ways_asked(Control::Ways).is_some() {
            b.state(State::Disabled);
        }
        b.bind(bindings.insert_way(
            ways::Act::Set { project, asked: Asked::Ways(Ways { way: way.number, ..shown }) },
            WaysRow::Way(way.number),
        ));
        b.close();
        b.close();
        ways_refused(b, root, w, bindings, project, WaysRow::Way(way.number));
        b.close();
    }
    b.close();
    b.close();
}
pub(super) fn questions(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut ForgeBindings, project: u32) {
    let stored = root.forge().questions(project);
    let shown = match root.forge().ways_asked(Control::Questions) {
        Some(Asked::Questions(route)) => *route,
        Some(Asked::Ways(_) | Asked::Classes(_)) | None => stored.unwrap_or(Route::None),
    };
    let label = w.say(Word::SettingsQuestions, &[]);
    row(b, w, &label, None);
    b.open(Role::Select);
    b.name(&label);
    if stored.is_none() || root.forge().ways_asked(Control::Questions).is_some() {
        b.state(State::Disabled);
    }
    for (route, word) in [(Route::Up, Word::SettingsUp), (Route::None, Word::SettingsAlone)] {
        b.open(Role::Option);
        b.name(&w.say(word, &[]));
        b.bind(bindings.insert(Meaning::Ways(ways::Act::Set { project, asked: Asked::Questions(route) })));
        if shown == route {
            b.state(State::Selected);
        }
        b.close();
    }
    b.close();
    refused(b, root, w, Control::Questions);
    b.close();
}
/// The duration stays editable while Away is off (client/pages/settings.md, 2, *Its rows*).
pub(super) fn away_until(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
    wall: skein_lib::Wall,
) {
    if root.away_shown(project, wall) {
        let row_value = root.forge().away(project);
        let eight = match row_value {
            Some(row) => row.until.is_some(),
            None => true,
        };
        let disabled = root.pages().away_busy();
        row(b, w, &w.say(Word::AwaySettingsUntil, &[]), Some(Word::AwayUntilLine));
        b.open(Role::Select);
        b.name(&w.say(Word::AwaySettingsUntil, &[]));
        if disabled {
            b.state(State::Disabled);
        }
        for (choice, word) in [(true, Word::AwayEight), (false, Word::AwayByHand)] {
            b.open(Role::Option);
            b.name(&w.say(word, &[]));
            b.bind(bindings.insert(Act::Away(Away::Until { project, eight: choice })));
            if choice == eight {
                b.state(State::Selected);
            }
            b.close();
        }
        b.close();
        b.close();
    }
}
pub(crate) fn repositories(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
    sign_in: &crate::forge_sign_in::Forges,
    forge: &mut crate::forge::bindings::Bindings,
) {
    main(b);
    open(b, root, w, bindings, project, Tab::Repositories);
    if !missing(b, root, w, bindings, project) {
        repository_section(b, root, w, bindings, project, sign_in, forge);
    }
    if let Address::Adopt { .. } = root.pages().address() {
        crate::forge::sheet::draw(b, root, w, bindings, forge, project);
    }
    b.close();
    b.close();
}
#[expect(clippy::too_many_lines, reason = "the section keeps adopted counts and making rows in one list")]
fn repository_section(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
    sign_in: &crate::forge_sign_in::Forges,
    forge: &mut ForgeBindings,
) {
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
    crate::bindings::link(
        b,
        &crate::forge::words::say(w, forge::words::Word::AddRepositories, &[]),
        bindings.insert(Act::Go { address: Address::Adopt { project } }),
    );
    b.style(look::SETTINGS_BTN);
    b.style(look::SETTINGS_SMALL);
    b.text(Run::Plain, &crate::forge::words::say(w, forge::words::Word::AddRepositories, &[]));
    b.close();
    b.close();
    let mut has_makings = false;
    for row in &rows {
        for making in &row.makings {
            has_makings |= match &making.into {
                Some(into) => into.project() == project,
                None => false,
            };
        }
    }
    if known && count == 0 && !has_makings {
        quiet_band(
            b,
            &w.say(Word::SettingsNoneTag, &[]),
            &w.say(Word::SettingsNoneBody, &[Hole::Name(project_name(root, w, project))]),
        );
        crate::bindings::link(
            b,
            &crate::forge::words::say(w, forge::words::Word::AddRepositories, &[]),
            bindings.insert(Act::Go { address: Address::Adopt { project } }),
        );
        b.style(look::SETTINGS_BTN);
        b.style(look::SETTINGS_SMALL);
        b.text(Run::Plain, &crate::forge::words::say(w, forge::words::Word::AddRepositories, &[]));
        b.close();
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
                repository_row(b, w, None, sign_in, &playground.owner, &playground.name, &name);
            }
        } else {
            for row in &rows {
                for repository in &row.repositories {
                    if repository.project == project {
                        repository_row(
                            b,
                            w,
                            Some(row),
                            sign_in,
                            &repository.owner,
                            &repository.name,
                            &crate::forge::connect::name(w, row, root),
                        );
                    }
                }
            }
        }
        for row in &rows {
            for making in &row.makings {
                if match &making.into {
                    Some(into) => into.project() == project,
                    None => false,
                } {
                    making_row(b, root, w, forge, row, making);
                }
            }
        }
        b.close();
    }
    b.close();
}
fn making_row(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut ForgeBindings,
    row: &forge::forges::Row,
    making: &forge::making::Making,
) {
    use forge::{sheet::Act, words::Word};
    use temper_client_protocol_wire::v1 as wire;
    let failed = match &making.state {
        wire::MakingState::Failed(failed) => Some(failed),
        wire::MakingState::Waiting
        | wire::MakingState::Making
        | wire::MakingState::Made(_)
        | wire::MakingState::Adopting(_) => None,
    };
    b.open(Role::Slot);
    if failed.is_some() {
        b.style(look::CONNECT_FAILED);
    }
    crate::cards::text(b, Role::Text, &forge::words::full_name(&making.org, &making.name));
    let word = match (&making.from, failed.is_some()) {
        (wire::MakeFrom::Copy(_), false) => Word::CopyingOn,
        (wire::MakeFrom::Empty, false) => Word::MakingOn,
        (wire::MakeFrom::Copy(_), true) => Word::CopyFailedOn,
        (wire::MakeFrom::Empty, true) => Word::MakeFailedOn,
    };
    text(
        b,
        Role::Text,
        &crate::forge::words::say(w, word, &[Hole::Name(crate::forge::connect::name(w, row, root))]),
        look::SETTINGS_WHEN,
    );
    let dismissal = root.forge().making_dismissal(row.number, &making.org, &making.name);
    if let Some(failed) = failed {
        b.open(Role::Paragraph);
        crate::forge::words::text(w, b, Word::ForgeSaid, &[Hole::Name(forge::words::lossy(failed.words()))]);
        let forge = row.number;
        let org = making.org.clone();
        let name = making.name.clone();
        let (word, act) = if *failed.reason() == wire::MakeFailure::Taken {
            (Word::AdoptIt, Act::AdoptAgain { forge, org, name })
        } else {
            match &making.from {
                wire::MakeFrom::Copy(_) => (Word::CopyAgain, Act::CopyAgain { forge, org, name }),
                wire::MakeFrom::Empty => (Word::MakeAgain, Act::MakeAgain { forge, org, name }),
            }
        };
        let busy = match dismissal {
            Some(dismissal) => dismissal.pending.is_some(),
            None => false,
        };
        for (word, act) in [
            (word, act),
            (Word::DismissMaking, Act::Dismiss { forge, org: making.org.clone(), name: making.name.clone() }),
        ] {
            b.open(Role::Button);
            b.style(look::CONNECT_QUIET);
            b.name(&crate::forge::words::say(w, word, &[]));
            b.bind(bindings.insert(Meaning::Sheet(act)));
            if busy {
                b.state(State::Disabled);
            }
            b.close();
        }
        b.close();
    }
    if let Some(dismissal) = dismissal
        && let Some(words) = &dismissal.words
    {
        crate::cards::text(b, Role::Alert, words);
    }
    b.close();
}
fn repository_row(
    b: &mut Builder,
    w: &Words,
    row: Option<&forge::forges::Row>,
    sign_in: &crate::forge_sign_in::Forges,
    owner: &[u8],
    name: &[u8],
    forge: &[u8],
) {
    b.open(Role::Slot);
    b.open(Role::Text);
    b.name(&forge::words::full_name(owner, name));
    crate::forge::repository::name(b, w, row, sign_in, owner, name);
    b.close();
    text(b, Role::Text, forge, look::SETTINGS_WHEN);
    b.close();
}
pub(crate) fn sign_out(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    let Some(me) = root.pages().me(root.client()) else { return };
    // A distinct form keeps its element when account sections arrive beside it.
    b.open(Role::Form);
    b.name(&w.say(Word::SignOut, &[]));
    b.style(look::SETTINGS_SETTING);
    b.style(look::SETTINGS_SIGN_OUT);
    b.open(Role::Slot);
    crate::cards::text(b, Role::Text, &w.say(Word::SettingsSignedIn, &[Hole::Name(me.party.name.bytes().into())]));
    text(b, Role::Text, me.party.login.bytes(), look::SETTINGS_DETAIL);
    if root.client().link().sign_out_lost {
        crate::cards::text(b, Role::Alert, &w.say(Word::SignOutLost, &[]));
    }
    b.close();
    b.open(Role::Slot);
    b.style(look::SETTINGS_END);
    link(b, w, bindings, Word::RememberedTitle, Address::Remembered);
    link(b, w, bindings, Word::ToldTitle, Address::Told);
    b.open(Role::Button);
    b.style(look::SETTINGS_BTN);
    b.style(look::SETTINGS_SMALL);
    b.name(&w.say(Word::SignOut, &[]));
    b.bind(bindings.insert(Act::SignOut));
    b.close();
    b.close();
    b.close();
}
fn link(b: &mut Builder, w: &Words, bindings: &mut Bindings, word: Word, address: Address) {
    let name = w.say(word, &[]);
    crate::bindings::link(b, &name, bindings.insert(Act::Go { address }));
    b.text(Run::Plain, &name);
    b.close();
}
