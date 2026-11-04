//! A project's first page, and its settings (temper/docs/design/client/README.md, 3.5).
use crate::{
    frame::Bindings,
    look,
    words::{Shell, TimeForm, Times},
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Address, ChatsFilter, Domain,
    forge::{
        self,
        forges::{Connection, Kind, Mode},
    },
    pages::{
        Act,
        home::{Facts, Repository},
    },
};

/// A project's first page (temper/docs/design/client/README.md, 3.5).
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
    sign_in: &crate::forge_sign_in::Forges,
    times: &mut Times,
) {
    let me = root.pages().me(root.client());
    let name = crate::pages::project_name(me, project);
    if me.is_some() && name.is_none() {
        page(b, root, w, bindings, project, false);
        return;
    }
    let fallback = w.say(Word::Project, &[Hole::Number(u64::from(project))]);
    let name = name.as_deref().unwrap_or(&fallback);
    let facts = temper_client_domain::pages::home::facts(root, project);
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::HOME);
    intro(b, root, w, project, name, &facts, times);
    to_begin(b, root, w, bindings, project, name, &facts);
    connected(b, root, w, bindings, project, name, sign_in);
    b.close();
}

fn intro(b: &mut Builder, root: &Domain, w: &Words, project: u32, name: &[u8], facts: &Facts, times: &mut Times) {
    b.open(Role::Slot);
    b.style(look::HOME_INTRO);
    // The kicker is this Home's day and time, not a project's creation time.
    let day = times.say(w, TimeForm::Day, times.now);
    let clock = times.say(w, TimeForm::Clock, times.now);
    let kicker = w.say(Word::HomeDayTime, &[Hole::Name(day), Hole::Name(clock)]);
    let minute = skein_lib::Duration::from_secs(60);
    let tick = times.now.as_nanos().checked_div(minute.as_nanos()).expect("minute is nonzero");
    let next = match tick.checked_add(1) {
        Some(tick) => match tick.checked_mul(minute.as_nanos()) {
            Some(nanos) => Some(skein_lib::Wall::from_nanos(nanos)),
            None => None,
        },
        None => None,
    };
    times.next = [times.next, next].into_iter().flatten().min();
    text(b, Role::Paragraph, &kicker, Some(look::HOME_KICKER));
    let headline = match facts.left() {
        0 => w.say(Word::HomeReady, &[Hole::Name(name.into())]),
        1 => w.say(Word::HomeOne, &[]),
        2 => w.say(Word::HomeTwo, &[]),
        3..=u8::MAX => w.say(Word::HomeThree, &[]),
    };
    text(b, Role::Heading { level: 1 }, &headline, None);
    let mut lede = Box::default();
    if let Some(account) = super::accounts::first_usable_kind(root, w) {
        lede = w.say(Word::HomeAgents, &[Hole::Name(account)]);
    }
    for row in &forge::listed::rows(root.forges(), forge::connect::playground(root.forge())) {
        if row.mode == Mode::Fake {
            continue;
        }
        match row.connection {
            Connection::Connected { .. } => {
                let sentence = if row.kind == Kind::Forgejo && row.mode == Mode::Owned {
                    w.say(Word::HomeForgeHere, &[])
                } else {
                    w.say(Word::HomeForge, &[Hole::Name(crate::forge::connect::name(w, row, root))])
                };
                lede = sentence_after(&lede, &sentence);
                break;
            }
            Connection::NotConnected
            | Connection::NotSetUp
            | Connection::SettingUp { .. }
            | Connection::SetUpFailed { .. }
            | Connection::Connecting
            | Connection::Stopped { .. } => {}
        }
    }
    if forge::listed::held(root.forges(), root.forge().playground(), project) == forge::listed::Held::None {
        lede = sentence_after(&lede, &w.say(Word::HomeNoRepository, &[Hole::Name(name.into())]));
    }
    if !lede.is_empty() {
        text(b, Role::Paragraph, &lede, Some(look::HOME_LEDE));
    }
    b.close();
}
fn sentence_after(before: &[u8], sentence: &[u8]) -> Box<[u8]> {
    crate::accounts::words::join(&[before, if before.is_empty() { b"" } else { b" " }, sentence])
}
fn text(b: &mut Builder, role: Role, words: &[u8], style: Option<skein_ui::Style>) {
    b.open(role);
    if let Some(style) = style {
        b.style(style);
    }
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
fn section(b: &mut Builder, w: &Words, word: Word, style: skein_ui::Style) {
    let name = w.say(word, &[]);
    b.open(Role::Region);
    b.style(style);
    b.name(&name);
    text(b, Role::Heading { level: 2 }, &name, None);
}
fn step(b: &mut Builder, w: &Words, number: u64, done: bool, heading: &[u8], detail: &[u8]) {
    b.open(Role::Item);
    b.style(look::HOME_STEP);
    if done {
        b.style(look::HOME_DONE);
    }
    let heading = match w.shell {
        Shell::Browser => heading.into(),
        Shell::Terminal => {
            let done = if done { w.say(Word::HomeDone, &[]) } else { Box::default() };
            crate::accounts::words::join(&[&w.say(Word::HomeNumber, &[Hole::Number(number)]), &done, heading])
        }
    };
    text(b, Role::Heading { level: 3 }, &heading, None);
    text(b, Role::Paragraph, detail, None);
}
fn acts(b: &mut Builder) {
    b.open(Role::Slot);
    b.style(look::HOME_ACTS);
}
fn go(b: &mut Builder, bindings: &mut Bindings, label: &[u8], address: Address, style: Option<skein_ui::Style>) {
    crate::link(b, label, bindings.insert(Act::Go { address }));
    if let Some(style) = style {
        b.style(style);
    }
    b.text(Run::Plain, label);
    b.close();
}
fn button(b: &mut Builder, bindings: &mut Bindings, label: &[u8], act: Act, style: skein_ui::Style, disabled: bool) {
    b.open(Role::Button);
    b.style(style);
    b.name(label);
    if disabled {
        b.state(State::Disabled);
    }
    b.bind(bindings.insert(act));
    b.close();
}
fn to_begin(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
    name: &[u8],
    facts: &Facts,
) {
    section(b, w, Word::HomeBegin, look::HOME_BEGIN);
    b.open(Role::List);
    b.style(look::HOME_STEPS);
    step(b, w, 1, facts.configured, &w.say(Word::HomeHow, &[]), &w.say(Word::HomeHowDetail, &[]));
    if !facts.configured {
        acts(b);
        let pending = match root.pages().chats() {
            Some(chats) => chats.start.is_some(),
            None => false,
        };
        button(b, bindings, &w.say(Word::HomeStart, &[]), Act::FirstChat, look::HOME_PRIMARY, pending);
        b.close();
    }
    b.close();
    repository_step(b, w, bindings, project, name, &facts.repository);
    step(b, w, 3, facts.chat_asked, &w.say(Word::HomeChat, &[]), &w.say(Word::HomeChatDetail, &[]));
    if !facts.chat_asked {
        acts(b);
        go(b, bindings, &w.say(Word::HomeNewChat, &[]), Address::Chats { filter: ChatsFilter::All, start: true }, None);
        b.close();
    }
    b.close();
    b.close();
    b.close();
}
fn repository_step(
    b: &mut Builder,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
    name: &[u8],
    repository: &Repository,
) {
    let adoption = w.say(Word::Adoption, &[]);
    match repository {
        Repository::Unknown => return,
        Repository::In { owner, name: repo } => {
            step(
                b,
                w,
                2,
                true,
                &w.say(Word::HomeIn, &[Hole::Name(owner.clone()), Hole::Name(repo.clone()), Hole::Name(name.into())]),
                &w.say(Word::HomeInDetail, &[]),
            );
        }
        Repository::Adopting(first) => {
            step(
                b,
                w,
                2,
                false,
                &w.say(
                    Word::HomeAdopting,
                    &[Hole::Name(first.org.clone()), Hole::Name(first.name.clone()), Hole::Name(name.into())],
                ),
                &w.say(Word::HomeAdoptingDetail, &[]),
            );
        }
        Repository::Offer { first, refused } => {
            step(
                b,
                w,
                2,
                false,
                &adoption,
                &w.say(Word::HomeOffer, &[Hole::Name(first.org.clone()), Hole::Name(first.name.clone())]),
            );
            if let Some(answer) = refused {
                let forge = skein_lib::Decimal::of(u64::from(first.forge));
                crate::forge::page::outcome(
                    b,
                    (&first.org, &first.name, b"temper/", forge.as_bytes()),
                    answer,
                    w,
                    w.name_bytes,
                );
            }
            acts(b);
            button(
                b,
                bindings,
                &w.say(Word::HomeAdopt, &[Hole::Name(first.org.clone()), Hole::Name(first.name.clone())]),
                Act::AdoptFirst,
                look::HOME_PRIMARY,
                false,
            );
            go(b, bindings, &w.say(Word::HomeAnother, &[]), Address::Adopt { project }, Some(look::HOME_QUIET));
            b.close();
        }
        Repository::GitHub { account } => {
            let detail = match account {
                Some(account) => w.say(Word::HomeGitHub, &[Hole::Name(account.clone())]),
                None => w.say(Word::HomeNone, &[]),
            };
            step(b, w, 2, false, &adoption, &detail);
            acts(b);
            go(b, bindings, &w.say(Word::HomeAdoptLink, &[]), Address::Adopt { project }, Some(look::HOME_PRIMARY));
            b.close();
        }
        Repository::None => {
            step(b, w, 2, false, &adoption, &w.say(Word::HomeNone, &[]));
            acts(b);
            go(b, bindings, &w.say(Word::HomeAdoptLink, &[]), Address::Adopt { project }, Some(look::HOME_PRIMARY));
            b.close();
        }
    }
    b.close();
}
fn line(b: &mut Builder, label: &[u8], detail: &[u8]) {
    b.open(Role::Item);
    b.open(Role::Paragraph);
    b.name(&crate::accounts::words::join(&[label, detail]));
    b.text(Run::Plain, label);
    if !detail.is_empty() {
        b.open(Role::Text);
        b.style(look::HOME_MUTED);
        b.run(Run::Plain, detail);
        b.close();
    }
    b.close();
}
fn connected(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
    name: &[u8],
    sign_in: &crate::forge_sign_in::Forges,
) {
    section(b, w, Word::HomeConnected, look::HOME_CONNECTED);
    b.open(Role::List);
    b.style(look::HOME_LINES);
    for row in &forge::listed::rows(root.forges(), forge::connect::playground(root.forge())) {
        if row.mode == Mode::Fake {
            continue;
        }
        let mut detail = Box::default();
        match &row.connection {
            Connection::Connected { login, .. } => {
                detail = w.say(Word::HomeLogin, &[Hole::Name(login.clone())]);
            }
            Connection::NotConnected
            | Connection::NotSetUp
            | Connection::SettingUp { .. }
            | Connection::SetUpFailed { .. }
            | Connection::Connecting
            | Connection::Stopped { .. } => {}
        }
        for repo in &row.repositories {
            if repo.project == project {
                detail = crate::accounts::words::join(&[
                    &detail,
                    &w.say(
                        Word::HomeRepository,
                        &[Hole::Name(repo.owner.clone()), Hole::Name(repo.name.clone()), Hole::Name(name.into())],
                    ),
                ]);
                break;
            }
        }
        line(b, &crate::forge::connect::name(w, row, root), &detail);
        if !crate::forge::forges::open_link(b, w, row, sign_in, None) {
            go(b, bindings, &w.say(Word::ForgesTitle, &[]), Address::Forges, None);
        }
        b.close();
    }
    if let Some(account) = super::accounts::first_usable_kind(root, w) {
        let mut model = None;
        if let Some(me) = root.pages().me(root.client()) {
            for row in &me.projects {
                if row.project == project {
                    model = row.models.get(0);
                    break;
                }
            }
        }
        let detail = match model {
            Some(model) => w.say(Word::HomeModel, &[Hole::Name(model.name.bytes().into())]),
            None => Box::default(),
        };
        line(b, &account, &detail);
        go(
            b,
            bindings,
            &crate::accounts::words::say(w, crate::accounts::words::Word::Accounts, &[]),
            Address::Accounts,
            None,
        );
        b.close();
    }
    let detail = match forge::listed::held(root.forges(), root.forge().playground(), project) {
        forge::listed::Held::One { owner, name } | forge::listed::Held::Several { owner, name, .. } => {
            w.say(Word::HomeProjectRepository, &[Hole::Name(owner.into()), Hole::Name(name.into())])
        }
        forge::listed::Held::None => w.say(Word::HomeProjectNone, &[]),
        forge::listed::Held::Unknown => Box::default(),
    };
    line(b, &w.say(Word::HomeProject, &[Hole::Name(name.into())]), &detail);
    if let Some(me) = root.pages().me(root.client()) {
        for row in &me.projects {
            if row.project == project && row.role == jig_client::contract::HeldRole::Owner {
                if root.pages().switcher().renaming_here() && root.pages().switcher().renaming() == Some(project) {
                    crate::frame::rename_form(b, root, w, name, bindings);
                } else {
                    button(
                        b,
                        bindings,
                        &w.say(Word::Rename, &[]),
                        Act::Switcher(temper_client_domain::pages::switcher::Act::RenameHere),
                        look::PROJECT_RENAME,
                        false,
                    );
                }
            }
        }
    }
    b.close();
    b.close();
    b.close();
}

/// A project's settings (temper/docs/design/client/README.md, 3.5).
pub(crate) fn settings(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    page(b, root, w, bindings, project, true);
}

fn page(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32, settings: bool) {
    b.open(Role::Main);
    b.style(look::MAIN);
    if settings {
        b.style(look::SETTINGS);
    } else {
        b.style(look::HOME);
    }
    let me = root.pages().me(root.client());
    let name = crate::pages::project_name(me, project);
    let fallback = w.say(Word::Project, &[Hole::Number(u64::from(project))]);
    let missing = me.is_some() && name.is_none();
    let name = name.as_deref().unwrap_or(&fallback);
    let heading = if settings { w.say(Word::SettingsFor, &[Hole::Name(name.into())]) } else { name.into() };
    b.open(Role::Heading { level: 1 });
    b.name(&heading);
    b.text(Run::Plain, &heading);
    b.close();
    if missing {
        b.open(Role::Paragraph);
        w.text(b, Word::ProjectNotYours, &[Hole::Number(u64::from(project))]);
        b.close();
        let chats = w.say(Word::Chats, &[]);
        crate::link(
            b,
            &chats,
            bindings.insert(Act::Go { address: Address::Chats { filter: ChatsFilter::All, start: false } }),
        );
        b.text(Run::Plain, &chats);
        b.close();
    } else if settings {
        settings_controls(b, root, w, bindings, project);
    }
    b.close();
}

fn settings_controls(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    b.open(Role::List);
    b.open(Role::Item);
    crate::frame::forges_link(b, root, w, bindings);
    b.close();
    b.open(Role::Item);
    crate::frame::accounts_link(b, root, w, bindings);
    b.close();
    b.open(Role::Item);
    let name = w.say(Word::Adoption, &[]);
    crate::link(b, &name, bindings.insert(Act::Go { address: Address::Adopt { project } }));
    b.text(Run::Plain, &name);
    b.close();
    b.close();
    b.close();
    if root.client().link().sign_out_lost {
        b.open(Role::Alert);
        b.name(&w.say(Word::SignOutLost, &[]));
        w.text(b, Word::SignOutLost, &[]);
        b.close();
    }
    if root.pages().me(root.client()).is_some() || root.client().link().sign_out_lost {
        b.open(Role::Button);
        b.name(&w.say(Word::SignOut, &[]));
        b.bind(bindings.insert(Act::SignOut));
        b.close();
    }
}
