//! One Tell temper card on Needs you and the Ask tab (client/pages/needs-you.md, 1).
use crate::{
    bindings::Bindings,
    look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_lib::{Wall, bytes::concat};
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{
        Act, team,
        tell::{self, About, Goes},
    },
};

pub(crate) fn main(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::TELL_PAGE);
    b.open(Role::Slot);
    b.style(look::TELL_TOP);
    super::ask::link(b, bindings, &w.say(Word::TellBack, &[]), Address::Ask { project });
    text(b, Role::Text, &w.say(Word::TellTitle, &[]), look::TELL_META);
    b.close();
    if let Some(goes) = tell::goes(root, project, root.pages().tell_about(project)) {
        text(b, Role::Heading { level: 1 }, &w.say(Word::TellHeadline, &[]), look::TELL_HEAD);
        text(b, Role::Paragraph, &w.say(Word::TellLede, &[]), look::TELL_META);
        card(b, root, w, bindings, project, &goes, true);
    }
    b.close();
}
pub(crate) fn region(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    let Some(goes) = tell::goes(root, project, root.pages().tell_about(project)) else { return };
    b.open(Role::Region);
    b.name(&w.say(Word::TellTitle, &[]));
    b.style(look::TELL);
    b.open(Role::Slot);
    b.style(look::TELL_HEAD);
    text(b, Role::Heading { level: 2 }, &w.say(Word::TellTitle, &[]), look::HOME_HEAD);
    text(b, Role::Text, &w.say(Word::TellHint, &[]), look::TELL_META);
    b.close();
    card(b, root, w, bindings, project, &goes, false);
    b.close();
}
fn text(b: &mut Builder, role: Role, words: &[u8], style: skein_ui::Style) {
    b.open(role);
    b.style(style);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
fn project_name(root: &Domain, w: &Words, project: u32) -> Box<[u8]> {
    match super::project_name(root.pages().me(root.client()), project) {
        Some(name) => name,
        None => w.say(Word::Project, &[Hole::Number(u64::from(project))]),
    }
}
fn yours(root: &Domain, holder: &team::Holder) -> bool {
    if let team::Holder::Party(party) = holder
        && let Some(me) = root.pages().me(root.client())
    {
        me.party.number == *party
    } else {
        false
    }
}
fn holder_name(root: &Domain, w: &Words, project: u32, holder: &team::Holder) -> Box<[u8]> {
    if yours(root, holder) {
        return w.say(Word::TellYou, &[]);
    }
    let shown = team::shown_at(root, project, Wall::EPOCH);
    crate::common::team::named_holder(w, shown.facts(), holder)
}
pub(crate) fn destination(root: &Domain, w: &Words, project: u32, goes: &Goes) -> Box<[u8]> {
    let name = project_name(root, w, project);
    match goes {
        Goes::Area { name: area, holder } => {
            if yours(root, holder) {
                w.say(Word::TellYourArea, &[Hole::Name(area.clone())])
            } else {
                w.say(Word::TellArea, &[Hole::Name(holder_name(root, w, project, holder)), Hole::Name(area.clone())])
            }
        }
        Goes::Triager { holder, areas } => {
            let line = if yours(root, holder) {
                w.say(Word::TellYourTriager, &[])
            } else {
                w.say(Word::TellTriager, &[Hole::Name(holder_name(root, w, project, holder)), Hole::Name(name)])
            };
            if *areas { concat(&[&line, &w.say(Word::TellHands, &[])]) } else { line }
        }
        Goes::Owners { areas } => {
            let line = w.say(Word::TellOwners, &[Hole::Name(name)]);
            if *areas { concat(&[&line, &w.say(Word::TellHand, &[])]) } else { line }
        }
    }
}
pub(crate) fn filed(root: &Domain, w: &Words, project: u32, goes: Option<&Goes>) -> Box<[u8]> {
    let name = match goes {
        Some(Goes::Area { holder, .. } | Goes::Triager { holder, .. }) => holder_name(root, w, project, holder),
        Some(Goes::Owners { .. }) => w.say(Word::TellOwnersName, &[Hole::Name(project_name(root, w, project))]),
        None => return w.say(Word::TellFiledUnknown, &[]),
    };
    w.say(Word::TellFiled, &[Hole::Name(name)])
}
fn title(w: &Words, about: &About) -> Box<[u8]> {
    match &about.address {
        Address::Task { task }
        | Address::Talk { task, .. }
        | Address::Chat { task }
        | Address::Team { role: Some(task), .. } => match &about.title {
            Some(title) => title.clone(),
            None => w.say(Word::TeamTaskName, &[Hole::Number(*task)]),
        },
        Address::Team { role: None, .. } => w.say(Word::TeamTitle, &[]),
        Address::Settings { .. } | Address::Role { .. } | Address::Rule { .. } => w.say(Word::SettingsHowTab, &[]),
        Address::Home { .. } | Address::AllHome => w.say(Word::RailHome, &[]),
        Address::Work { .. } => w.say(Word::RailWork, &[]),
        Address::Goals { .. } => w.say(Word::Goals, &[]),
        Address::Changes { .. } => w.say(Word::Changes, &[]),
        Address::Repositories { .. } => w.say(Word::SettingsRepositoriesTab, &[]),
        Address::Models { .. } => w.say(Word::ModelsTab, &[]),
        Address::Builders { .. } => w.say(Word::SettingsBuildersTab, &[]),
        Address::Chats { .. } => w.say(Word::Chats, &[]),
        Address::NewChat { .. } => w.say(Word::NewChat, &[]),
        Address::Inbox { .. }
        | Address::Tell { .. }
        | Address::Ask { .. }
        | Address::Adopt { .. }
        | Address::Members
        | Address::Remembered
        | Address::Told
        | Address::Limits
        | Address::Accounts
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. } => Box::default(),
    }
}
fn card(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32, goes: &Goes, page: bool) {
    let words = root.pages().tell_words();
    let about = root.pages().tell_about(project);
    let sending = if let Some(last) = root.pages().tell() { last.state == tell::State::Sending } else { false };
    let over = words.len().saturating_sub(4096);
    b.open(Role::Slot);
    b.style(look::TELL_CARD);
    b.open(Role::TextArea);
    b.style(look::TELL_FIELD);
    b.name(&w.say(if page { Word::TellField } else { Word::TellHeadline }, &[]));
    b.value(words, temper_client_domain::shown(root));
    if sending {
        b.state(State::Disabled);
    }
    b.bind(bindings.insert(Act::TellWords { words: Box::default() }));
    b.close();
    if over > 0 {
        text(
            b,
            Role::Paragraph,
            &if over == 1 {
                w.say(Word::TellLongOne, &[])
            } else {
                w.say(Word::TellLong, &[Hole::Number(u64::try_from(over).expect("input fits"))])
            },
            look::SHEET_ERROR,
        );
    }
    if let Some(last) = root.pages().tell()
        && last.project == project
    {
        match &last.state {
            tell::State::Refused(reason) => {
                text(b, Role::Paragraph, &crate::shared::finding_refusal(w, reason), look::SHEET_ERROR);
            }
            tell::State::Lost => text(b, Role::Paragraph, &w.say(Word::TellLost, &[]), look::SHEET_ERROR),
            tell::State::Filed { .. } | tell::State::Sending | tell::State::Long { .. } => {}
        }
    }
    let name = project_name(root, w, project);
    let line = match about {
        Some(about) => w.say(Word::TellAboutPage, &[Hole::Name(name), Hole::Name(title(w, about))]),
        None => w.say(Word::TellAbout, &[Hole::Name(name)]),
    };
    b.open(Role::Slot);
    b.style(look::TELL_CONTEXT);
    text(b, Role::Paragraph, &line, look::TELL_META);
    text(b, Role::Paragraph, &destination(root, w, project, goes), look::TELL_META);
    b.close();
    b.open(Role::Slot);
    b.style(look::TELL_ACTS);
    b.open(Role::Button);
    b.name(&w.say(Word::TellFile, &[]));
    b.style(look::RECOMMENDED);
    if sending || over > 0 || blank(words) {
        b.state(State::Disabled);
    }
    b.bind(bindings.insert(Act::Tell { project, words: words.into(), about: about.cloned() }));
    b.close();
    b.open(Role::Button);
    b.name(&w.say(if page { Word::TellQuestion } else { Word::TellAsk }, &[]));
    b.style(look::ASK_QUIET);
    b.bind(bindings.insert(Act::AskWith { words: words.into() }));
    b.close();
    b.close();
    b.close();
}

#[expect(clippy::disallowed_methods, reason = "the input boundary tests Unicode whitespace without cutting the field")]
fn blank(words: &[u8]) -> bool {
    match core::str::from_utf8(words) {
        Ok(words) => words.trim().is_empty(),
        Err(_) => true,
    }
}
