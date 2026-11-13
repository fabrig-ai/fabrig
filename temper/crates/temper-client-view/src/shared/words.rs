//! Words shared by the browser and TUI (temper/docs/design/client/tui/pages/launchpad.md, 2).
/// A run of calls (temper/docs/design/client/pages/README.md, its calls).
pub use crate::pages::chat_line::Run;
/// The terminal's vocabulary (temper/docs/design/client/pages/README.md, A chat's terminal).
pub use crate::words::terminal::TerminalWord;
use crate::words::{Times, Words};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::Wall;
use temper_client_domain::{
    Domain,
    pages::{
        all_home,
        home::Facts,
        needs::Need,
        terminal::{Command, Note},
    },
};

/// A terminal phrase (temper/docs/design/client/pages/README.md, A chat's terminal).
#[must_use]
pub fn say(w: &Words, word: TerminalWord) -> Box<[u8]> {
    crate::words::terminal::say(w, word)
}
/// Home's headline and its next tick (temper/docs/design/client/pages/README.md, the headline).
#[must_use]
pub fn home_headline(w: &Words, name: &[u8], facts: &Facts, now: Wall) -> (Box<[u8]>, Option<Wall>) {
    let mut times = Times::new(now);
    let intro = crate::pages::home::intro_words(w, name, facts, &mut times);
    (intro.headline, times.next)
}
/// All projects' headline and its next tick (temper/docs/design/client/tui/pages/launchpad.md, 2).
#[must_use]
pub fn all_home_headline(w: &Words, facts: &all_home::Facts, now: Wall) -> (Box<[u8]>, Option<Wall>) {
    let mut times = Times::new(now);
    let headline = crate::pages::all_home::headline(w, facts, &mut times);
    (headline, times.next)
}
/// All projects' lede, without its strong runs (temper/docs/design/client/tui/pages/launchpad.md, 2).
#[must_use]
pub fn joined_brief(_w: &Words, brief: &all_home::Brief) -> Box<[u8]> {
    let mut words: Box<[u8]> = Box::default();
    for (index, line) in brief.lines.iter().take(4).enumerate() {
        let gap: &[u8] = if index == 0 { b"" } else { b" " };
        words = skein_lib::bytes::concat(&[&words, gap, &line.name, b": ", &line.sentence]);
    }
    words
}
/// A Home chat row's state (temper/docs/design/client/pages/README.md, Home, Chats).
#[must_use]
pub fn chat_state(line: &c::TaskLine, me: &c::Me, w: &Words) -> Box<[u8]> {
    crate::pages::chats::status_name(line, me, w)
}
/// A need's kind (temper/docs/design/client/tui/pages/launchpad.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NeedKind {
    Question,
    Other,
}
/// A need's line (temper/docs/design/client/tui/pages/launchpad.md, 5).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NeedLine {
    pub kind: NeedKind,
    pub words: Box<[u8]>,
}
/// Home's need sentence (temper/docs/design/client/pages/README.md, Home, Needs you).
#[must_use]
pub fn need_line(need: &Need<'_>, w: &Words) -> NeedLine {
    let (title, short, kind) = crate::pages::home::need_words(need, w);
    NeedLine { kind, words: crate::pages::home::need_sentence(&title, &short, w) }
}
/// A run's line (temper/docs/design/client/pages/README.md, its calls).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RunLine {
    pub name: Box<[u8]>,
    pub description: Option<Box<[u8]>>,
    pub failed: u64,
    pub running: bool,
}
/// The running line's words (temper/docs/design/client/pages/README.md, The running line).
#[must_use]
pub fn running(root: &Domain, w: &Words) -> Option<Box<[u8]>> {
    crate::pages::terminal::running_words(root, w)
}
/// The picker's closed line (temper/docs/design/client/pages/README.md, the quiet line).
#[must_use]
pub fn closed_line(root: &Domain, task: Option<&c::Task>, w: &Words) -> Box<[u8]> {
    crate::choice::closed_line(root, task, w)
}
/// A chat's short heading (temper/docs/design/client/pages/README.md, A chat's heading is short).
#[must_use]
pub fn heading(title: &[u8]) -> Box<[u8]> {
    crate::pages::chat::short_title(title)
}
/// A command's words (temper/docs/design/client/pages/README.md, Commands).
#[must_use]
pub fn description(command: Command) -> &'static [u8] {
    crate::pages::terminal::commands::description(command)
}
/// When a command is offered (temper/docs/design/client/pages/README.md, Commands).
#[must_use]
pub fn offered(root: &Domain, task: Option<&c::Task>, command: Command) -> bool {
    crate::pages::terminal::commands::offered(root, task, command)
}
/// A terminal note's words (temper/docs/design/client/pages/README.md, The terminal's notes).
#[must_use]
pub fn note(note: &Note) -> Box<[u8]> {
    crate::pages::terminal::commands::note(note)
}
