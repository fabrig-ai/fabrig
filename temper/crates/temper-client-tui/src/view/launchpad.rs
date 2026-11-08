//! The first screen's lines, filled by tui-launchpad (client/README.md, 6.14).
use super::{Binds, text};
use crate::{
    Tui, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::Link;
use skein_lib::{List, Wall, Writer, bytes::copy_of};
use skein_ui::{Builder, Role, Style, phrases::Hole};
use temper_client_domain::Domain;
use temper_client_view::words::Word as Shared;

#[derive(Debug)]
pub(crate) struct Launchpad {
    pub lines: List<Line>,
}
#[derive(Debug)]
pub(crate) struct Line {
    pub role: Role,
    pub words: Box<[u8]>,
    pub style: Option<Style>,
}
fn push(lines: &mut List<Line>, role: Role, words: &[u8], style: Option<Style>) {
    lines.push(Line { role, words: copy_of(words), style }).expect("at most three first-screen lines");
}
fn projects(me: &jig_client::contract::Me, bound: u32) -> Box<[u8]> {
    let mut bytes = 0_u32;
    let mut count = 0_usize;
    for project in &me.projects {
        let separator = if count == 0 { 0 } else { 4 };
        let name = u32::try_from(project.name.bytes().len()).expect("domain text fits u32");
        let Some(next) = bytes.checked_add(separator) else { break };
        let Some(next) = next.checked_add(name) else { break };
        if next > bound {
            break;
        }
        bytes = next;
        count = count.saturating_add(1);
    }
    let mut out = Writer::new(skein_lib::usize_of(bytes));
    for (index, project) in me.projects.as_slice().iter().take(count).enumerate() {
        if index != 0 {
            out.put(" · ".as_bytes()).expect("separator counted");
        }
        out.put(project.name.bytes()).expect("name counted");
    }
    out.finish()
}
pub(crate) fn read(root: &Domain, tui: &Tui, words: &Words, _wall: Wall) -> Launchpad {
    let mut lines = List::with_capacity(3);
    let me = if root.client().link().state == Link::SignedOut { None } else { root.pages().me(root.client()) };
    if root.client().link().state == Link::OutOfDate {
        push(&mut lines, Role::Paragraph, &words.view().say(Shared::OutOfDateMain, &[]), None);
    } else if let Some(me) = me {
        push(
            &mut lines,
            Role::Paragraph,
            &words.say(
                Word::Connected,
                &[Hole::Name(copy_of(tui.address())), Hole::Name(copy_of(me.party.name.bytes()))],
            ),
            None,
        );
        let line = if me.projects.is_empty() {
            words.view().say(Shared::NoProjectYet, &[])
        } else {
            words.say(Word::Projects, &[Hole::Name(projects(me, words.projects_bytes))])
        };
        push(&mut lines, Role::Paragraph, &line, None);
    } else {
        push(&mut lines, Role::Paragraph, &words.say(Word::Connecting, &[Hole::Name(copy_of(tui.address()))]), None);
    }
    if tui.unkept() {
        push(&mut lines, Role::Alert, &words.say(Word::NotKept, &[]), Some(look::REFUSED));
    }
    Launchpad { lines }
}

pub(crate) fn draw(b: &mut Builder, part: &Launchpad, words: &Words, _binds: &mut Binds) {
    b.open(Role::Main);
    b.region(look::TOP);
    text(b, Role::Heading { level: 1 }, &words.say(Word::Title, &[]), Some(look::TITLE));
    for line in &part.lines {
        text(b, line.role, &line.words, line.style);
    }
    b.close();
}
