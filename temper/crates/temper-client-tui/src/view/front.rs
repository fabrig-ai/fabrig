//! The playground, quitting, the box's state and /log (temper/docs/design/client/tui/box.md).
use super::foot::{OptionItem, Options};
use crate::{
    BoxState, Event, Tui, limits, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_lib::{
    List,
    bytes::{concat, copy_of, utf8_prefix},
};
use skein_ui::{Style, phrases::Hole};
use temper_client_domain::{Domain, forge::words::lossy};

#[derive(Debug)]
pub(crate) struct LogNote {
    pub head: Box<[u8]>,
    pub lines: Box<[Box<[u8]>]>,
    pub end: u64,
}

pub(crate) fn playground(root: &Domain, project: Option<u32>) -> bool {
    if !root.forge().playground_running() {
        return false;
    }
    match root.forge().playground() {
        Some(read) => project.is_some() && read.project == project,
        None => false,
    }
}
fn step(bytes: &[u8]) -> Box<[u8]> {
    copy_of(utf8_prefix(&lossy(bytes), usize::try_from(limits::STEP).expect("step limit fits memory")))
}
pub(crate) fn box_state(tui: &Tui, words: &Words) -> Option<(Box<[u8]>, Style)> {
    match tui.box_state() {
        BoxState::Running => None,
        BoxState::Restarting { step: bytes } => {
            Some((words.say(Word::Restarting, &[Hole::Name(step(bytes))]), look::SIGNAL))
        }
        BoxState::GaveUp { step: bytes } => Some((words.say(Word::GaveUp, &[Hole::Name(step(bytes))]), look::DOWN)),
    }
}
pub(crate) fn quit(root: &Domain, tui: &Tui, words: &Words) -> Option<Options> {
    if !tui.quitting() {
        return None;
    }
    let running = super::work::running(root);
    let head = match running.count {
        0 => words.say(Word::QuitNone, &[]),
        1 => words.say(Word::QuitOne, &[Hole::Name(running.title.expect("one working chat has a title"))]),
        2.. => words.say(Word::QuitMany, &[Hole::Number(u64::from(running.count))]),
    };
    Some(Options {
        style: look::OPTION,
        why: None,
        name: head.clone(),
        head: Some(head),
        items: Box::new([
            OptionItem {
                name: concat(&[&words.say(Word::Numbered, &[Hole::Number(1)]), &words.say(Word::StopBox, &[])]),
                words: Box::default(),
                selected: false,
                event: Event::Stop,
            },
            OptionItem {
                name: concat(&[&words.say(Word::Numbered, &[Hole::Number(2)]), &words.say(Word::DontQuit, &[])]),
                words: Box::default(),
                selected: false,
                event: Event::Stay,
            },
        ]),
        escape: Event::Stay,
    })
}
pub(crate) fn log_note(tui: &Tui, words: &Words) -> Option<LogNote> {
    if !tui.log_shown() || !tui.front() {
        return None;
    }
    let mut lines = List::with_capacity(limits::BOX_LOG);
    for line in tui.box_log() {
        lines
            .push(copy_of(utf8_prefix(&lossy(line), usize::try_from(limits::LOG_LINE).expect("log line fits memory"))))
            .expect("the box keeps at most BOX_LOG lines");
    }
    if lines.is_empty() {
        lines.push(words.say(Word::LogEmpty, &[])).expect("room for the empty log's line");
    }
    Some(LogNote { head: words.say(Word::LogHead, &[]), lines: lines.into_boxed(), end: tui.revision() })
}
pub(crate) fn gave_up_note(tui: &Tui, words: &Words) -> Option<Box<[u8]>> {
    match tui.box_state() {
        BoxState::Running | BoxState::Restarting { .. } => None,
        BoxState::GaveUp { step: bytes } => Some(words.say(Word::GaveUpNote, &[Hole::Name(step(bytes))])),
    }
}
