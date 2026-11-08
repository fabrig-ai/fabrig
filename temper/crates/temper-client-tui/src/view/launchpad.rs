//! Home's launchpad (temper/docs/design/client/README.md, 6.14, The launchpad).
use super::{Binds, foot, text};
use crate::{
    Tui, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::{Link, contract as c};
use skein_lib::{
    List, Wall,
    bytes::{concat, copy_of},
};
use skein_ui::{Builder, Role, Style, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{chats, home},
};
use temper_client_view::{
    shared::{self, NeedKind, NeedLine},
    words::{TimeForm, Word as Shared},
};

#[derive(Debug)]
pub(crate) struct Launchpad {
    pub project: Option<Box<[u8]>>,
    pub address: Box<[u8]>,
    pub lines: List<Line>,
    pub left_off: Option<LeftOff>,
    pub needs: Box<[NeedLine]>,
    pub notes: foot::Notes,
    pub tick: Option<Wall>,
}
#[derive(Debug)]
pub(crate) struct LeftOff {
    pub title: Box<[u8]>,
    pub state: Box<[u8]>,
}
#[derive(Debug)]
pub(crate) struct Line {
    pub role: Role,
    pub words: Box<[u8]>,
    pub style: Option<Style>,
}
fn push(lines: &mut List<Line>, role: Role, words: &[u8], style: Option<Style>) {
    lines.push(Line { role, words: copy_of(words), style }).expect("header has at most three lines");
}
fn project(root: &Domain, me: &c::Me) -> Option<u32> {
    match root.pages().address() {
        Address::Home { project } => Some(*project),
        Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Ask { .. }
        | Address::Accounts
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. }
        | Address::Task { .. }
        | Address::Chat { .. }
        | Address::Goals { .. }
        | Address::Changes { .. }
        | Address::Settings { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Repositories { .. }
        | Address::Adopt { .. } => {
            let first = me.projects.get(0)?;
            Some(first.project)
        }
    }
}
fn name(me: &c::Me, number: u32) -> Option<Box<[u8]>> {
    for project in &me.projects {
        if project.project == number {
            return Some(copy_of(project.name.bytes()));
        }
    }
    None
}
fn left_off(root: &Domain, project: u32, me: &c::Me, words: &Words) -> Option<LeftOff> {
    let tasks = root.pages().chats()?.live(root.client())?;
    let mut newest: Option<&c::TaskLine> = None;
    for line in &tasks.rows {
        if line.project != project || chats::ended(&line.chip) {
            continue;
        }
        let later = match newest {
            Some(held) => {
                line.last_activity > held.last_activity
                    || (line.last_activity == held.last_activity && line.chip.task < held.chip.task)
            }
            None => true,
        };
        if later {
            newest = Some(line);
        }
    }
    let line = newest?;
    Some(LeftOff { title: copy_of(line.chip.title.bytes()), state: shared::chat_state(line, me, words.view()) })
}
fn home(part: &mut Launchpad, root: &Domain, project: u32, me: &c::Me, words: &Words, wall: Wall) {
    if root.pages().address() != &(Address::Home { project }) {
        return;
    }
    let facts = home::facts(root, project, wall);
    if let Some(brief) = facts.brief() {
        push(&mut part.lines, Role::Paragraph, &concat(&[brief.sentence(), b" ", brief.lede()]), None);
        let (clock, _) = words.view().time(TimeForm::Clock, Wall::from_nanos(brief.at()), wall);
        push(&mut part.lines, Role::Paragraph, &words.say(Word::Byline, &[Hole::Name(clock)]), Some(look::BYLINE));
    } else {
        match facts.needs {
            home::Needs::Unknown => {}
            home::Needs::Count { .. } => {
                let (line, tick) = shared::home_headline(
                    words.view(),
                    part.project.as_deref().expect("project has a name"),
                    &facts,
                    wall,
                );
                push(&mut part.lines, Role::Paragraph, &line, None);
                part.tick = tick;
            }
        }
    }
    part.left_off = left_off(root, project, me, words);
    let count = match facts.needs {
        home::Needs::Unknown => 0,
        home::Needs::Count { count, .. } => count,
    };
    if count > 0
        && let Some(inbox) = root.pages().inbox()
    {
        let mut needs = List::with_capacity(5);
        for need in root.pages().needs().needs(inbox, root.forge().approvals()).iter().take(5) {
            needs.push(shared::need_line(need, words.view())).expect("five launchpad needs");
        }
        part.needs = needs.into_boxed();
    }
}
pub(crate) fn read(root: &Domain, tui: &Tui, words: &Words, wall: Wall) -> Launchpad {
    let me = if root.client().link().state == Link::SignedOut { None } else { root.pages().me(root.client()) };
    let number = match me {
        Some(me) => project(root, me),
        None => None,
    };
    let project = match (me, number) {
        (Some(me), Some(number)) => name(me, number),
        (Some(_), None) | (None, Some(_) | None) => None,
    };
    let mut part = Launchpad {
        project,
        address: copy_of(tui.address()),
        lines: List::with_capacity(3),
        left_off: None,
        needs: Box::default(),
        notes: foot::read_notes(root, tui, words),
        tick: None,
    };
    if root.client().link().state == Link::OutOfDate {
        push(&mut part.lines, Role::Paragraph, &words.view().say(Shared::OutOfDateMain, &[]), None);
    } else if let Some(me) = me {
        if let Some(number) = number {
            if part.project.is_some() {
                home(&mut part, root, number, me, words, wall);
            }
        } else {
            push(&mut part.lines, Role::Paragraph, &words.view().say(Shared::NoProjectYet, &[]), None);
            push(
                &mut part.lines,
                Role::Paragraph,
                &words.say(Word::NoProject, &[Hole::Name(part.address.clone())]),
                None,
            );
        }
    } else {
        push(&mut part.lines, Role::Paragraph, &words.say(Word::Connecting, &[Hole::Name(part.address.clone())]), None);
    }
    if tui.unkept() {
        push(&mut part.lines, Role::Alert, &words.say(Word::NotKept, &[]), Some(look::REFUSED));
    }
    part
}
// The screen adds punctuation to Term and Heading. Paragraph heads keep
// the section labels plain; Description retains the two-column words indent.
fn sections(b: &mut Builder, part: &Launchpad, words: &Words) {
    b.open(Role::Region);
    b.region(look::SECTIONS);
    b.open(Role::Group);
    b.style(look::SPACED);
    if let Some(left) = &part.left_off {
        b.open(Role::Region);
        text(b, Role::Paragraph, &words.say(Word::LeftOff, &[]), Some(look::HEAD));
        b.open(Role::Description);
        b.open(Role::Paragraph);
        text(b, Role::Text, &left.title, None);
        text(b, Role::Text, &words.say(Word::Separator, &[]), Some(look::HINT));
        text(b, Role::Text, &left.state, Some(look::HINT));
        b.close();
        b.close();
        b.close();
    }
    if !part.needs.is_empty() {
        b.open(Role::Region);
        text(b, Role::Paragraph, &words.say(Word::NeedsYou, &[]), Some(look::HEAD));
        for need in part.needs.iter().take(5) {
            b.open(Role::Description);
            b.open(Role::Paragraph);
            let marker = match need.kind {
                NeedKind::Question => Word::Question,
                NeedKind::Other => Word::OtherNeed,
            };
            text(b, Role::Text, &words.say(marker, &[]), Some(look::SIGNAL));
            text(b, Role::Text, &concat(&[b" ", &need.words]), None);
            b.close();
            b.close();
        }
        b.close();
    }
    b.close();
    foot::draw_notes(b, &part.notes, words);
    b.close();
}
pub(crate) fn draw(b: &mut Builder, part: &Launchpad, words: &Words, _binds: &mut Binds) {
    b.open(Role::Main);
    b.open(Role::Region);
    b.region(look::TOP);
    b.open(Role::Heading { level: 1 });
    text(b, Role::Text, &words.say(Word::Title, &[]), Some(look::TITLE));
    if let Some(project) = &part.project {
        text(b, Role::Text, &words.say(Word::Separator, &[]), Some(look::FAINT));
        text(b, Role::Text, project, Some(look::NAME));
        text(b, Role::Text, &part.address, Some(look::ADDRESS));
    }
    b.close();
    for line in &part.lines {
        text(b, line.role, &line.words, line.style);
    }
    b.close();
    sections(b, part, words);
    b.close();
}
