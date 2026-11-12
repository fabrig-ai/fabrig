//! A standing role's framed head (temper/docs/design/client/tui/pages/talk.md, 1).
use super::{foot, launchpad, text};
use crate::{
    Tui, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_lib::{
    List, Wall,
    bytes::{concat, copy_of},
};
use skein_ui::{Builder, Role, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{chat::Talking, team},
};
use temper_client_view::words::{TimeForm, Word as Shared};

#[derive(Debug)]
pub(crate) struct Head {
    pub project: Box<[u8]>,
    pub name: Box<[u8]>,
    pub address: Box<[u8]>,
    pub line: Box<[u8]>,
    pub now: Option<Box<[u8]>>,
    pub columns: u16,
}
pub(crate) fn available(root: &Domain, project: u32) -> bool {
    match team::shown_at(root, project, Wall::EPOCH).facts() {
        Some(facts) => !facts.roles.is_empty(),
        None => false,
    }
}
pub(crate) fn name(talk: &Talking, words: &Words) -> Box<[u8]> {
    match talk.name() {
        Some(name) => copy_of(name),
        None => words.say(Word::RoleNumber, &[Hole::Number(talk.task)]),
    }
}
pub(crate) fn minutes(opened: Wall, wall: Wall) -> u64 {
    wall.as_nanos().saturating_sub(opened.as_nanos()).checked_div(60_000_000_000).expect("minute is nonzero").max(1)
}
pub(crate) fn areas(areas: Option<&team::Areas>, task: u64) -> Box<[u8]> {
    let mut names = List::uncapped();
    if let Some(areas) = areas {
        for area in &areas.rows {
            if area.holder == Some(team::Holder::Task(task)) {
                if !names.is_empty() {
                    names.push(b", ".as_slice()).expect("area names fit memory");
                }
                names.push(area.name.as_ref()).expect("area names fit memory");
            }
        }
    }
    concat(names.as_slice())
}
pub(crate) fn read(root: &Domain, tui: &Tui, words: &Words) -> Option<Head> {
    let talk = root.pages().chat()?.talk()?;
    let project = foot::project_name(root, Some(talk.project)).unwrap_or_default();
    let name = name(talk, words);
    let mut line = words.say(Word::StandingRole, &[]);
    let held = team::areas(root, talk.project);
    let areas = areas(held.as_ref(), talk.task);
    if !areas.is_empty() {
        line = concat(&[&line, &words.say(Word::Owns, &[]), &areas]);
    }
    if let Some(held) = held
        && held.triager == Some(team::Holder::Task(talk.task))
    {
        line = concat(&[&line, &words.say(Word::TriagerOf, &[Hole::Name(project.clone())])]);
    }
    if let Some((spent, budget)) = talk.spend() {
        let period = if team::weekly(root, talk.project) { Word::ThisWeek } else { Word::ThisPeriod };
        line = concat(&[&line, &words.say(period, &[Hole::Number(spent), Hole::Number(budget)])]);
    }
    let now = match talk.now() {
        Some(now) => {
            let prefix = match now.changed {
                Some(at) => {
                    let (at, _) = words.view().time(TimeForm::Clock, at, at);
                    words.say(Word::ItsNowAt, &[Hole::Name(at)])
                }
                None => words.say(Word::ItsNow, &[]),
            };
            let end = skein_lib::bytes::find(&now.body, b"\n").unwrap_or(now.body.len());
            Some(concat(&[&prefix, now.body.get(..end).expect("Now first line")]))
        }
        None => None,
    };
    Some(Head { project, name, address: copy_of(tui.address()), line, now, columns: tui.columns() })
}
pub(crate) fn session(talk: &Talking, words: &Words, wall: Wall) -> Option<Box<[u8]>> {
    let open = talk.open()?;
    let mut line = words.say(Word::SessionMinutes, &[Hole::Number(minutes(open.opened, wall))]);
    if let Some(cost) = talk.cost() {
        line = concat(&[
            &line,
            &words.say(Word::Separator, &[]),
            &words.view().say(Shared::TeamCount, &[Hole::Number(cost)]),
        ]);
    }
    Some(line)
}
pub(crate) fn draw(b: &mut Builder, head: &Head, words: &Words) {
    b.open(Role::Region);
    b.region(look::TOP);
    b.open(Role::Heading { level: 1 });
    text(b, Role::Text, &words.say(Word::Title, &[]), Some(look::TITLE));
    text(b, Role::Text, &words.say(Word::Separator, &[]), Some(look::FAINT));
    text(b, Role::Text, &head.project, Some(look::NAME));
    text(b, Role::Text, &words.say(Word::Separator, &[]), Some(look::FAINT));
    text(b, Role::Text, &words.say(Word::TalkingWith, &[]), Some(look::HINT));
    text(b, Role::Text, &head.name, Some(look::NAME));
    text(b, Role::Text, &head.address, Some(look::ADDRESS));
    b.close();
    let room = if head.columns == 0 { usize::MAX } else { usize::from(head.columns).saturating_sub(4) };
    let (line, end) = launchpad::cut(&concat(&[&head.name, &head.line]), room, words);
    let name_end = head.name.len().min(end);
    b.open(Role::Paragraph);
    text(b, Role::Text, line.get(..name_end).expect("name prefix fits"), Some(look::NAME));
    text(b, Role::Text, line.get(name_end..).expect("line suffix fits"), Some(look::HINT));
    b.close();
    if let Some(now) = &head.now {
        text(b, Role::Paragraph, &launchpad::cut(now, room, words).0, Some(look::HINT));
    }
    b.close();
}
