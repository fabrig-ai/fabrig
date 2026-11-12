//! The playground, quitting, the box's state and /log (temper/docs/design/client/tui/box.md).
use super::foot::{OptionItem, Options};
use crate::{
    BoxEnd, BoxStage, BoxState, Event, Tui, limits, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_lib::{
    Decimal, List,
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

#[derive(Debug)]
pub(crate) struct StoppingNote {
    pub log: LogNote,
    pub hint: Box<[u8]>,
    pub end: Option<Box<[u8]>>,
}

pub(crate) fn stage(tui: &Tui, words: &Words) -> Option<(Box<[u8]>, Style)> {
    let line = match tui.box_stage() {
        BoxStage::Finding => words.say(Word::FindingBox, &[]),
        BoxStage::Fetching { done, total } => match total {
            Some(total) => {
                words.say(Word::DownloadingBoxTotal, &[Hole::Name(megabytes(*done)), Hole::Name(megabytes(*total))])
            }
            None => words.say(Word::DownloadingBox, &[Hole::Name(megabytes(*done))]),
        },
        BoxStage::Starting => words.say(Word::StartingBox, &[]),
        BoxStage::Failed { why } => {
            return Some((words.say(Word::FailedBox, &[Hole::Name(lossy(why))]), look::REFUSED));
        }
        BoxStage::Running | BoxStage::Stopping | BoxStage::Ended { .. } => return None,
    };
    Some((line, look::HINT))
}
fn megabytes(bytes: u64) -> Box<[u8]> {
    concat(&[
        Decimal::of(bytes.checked_div(1_000_000).expect("nonzero divisor")).as_bytes(),
        b".",
        Decimal::of(bytes.checked_div(100_000).expect("nonzero divisor").checked_rem(10).expect("nonzero divisor"))
            .as_bytes(),
    ])
}
fn end_words(end: BoxEnd, words: &Words) -> Box<[u8]> {
    match end {
        BoxEnd::Exited { code } => words.say(Word::BoxExited, &[Hole::Number(u64::from(code))]),
        BoxEnd::Killed { signal } => words.say(Word::BoxKilled, &[Hole::Number(u64::from(signal))]),
    }
}
pub(crate) fn stopping_note(tui: &Tui, words: &Words) -> Option<StoppingNote> {
    if !tui.front() || !tui.stop_asked() {
        return None;
    }
    let end = match tui.box_stage() {
        BoxStage::Stopping => None,
        BoxStage::Ended { end } => Some(end_words(*end, words)),
        BoxStage::Finding
        | BoxStage::Fetching { .. }
        | BoxStage::Starting
        | BoxStage::Running
        | BoxStage::Failed { .. } => return None,
    };
    Some(StoppingNote {
        log: LogNote { head: words.say(Word::StoppingBox, &[]), lines: log_lines(tui, words), end: tui.revision() },
        hint: words.say(Word::StoppingBoxHint, &[]),
        end,
    })
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
        BoxState::Stopping => Some((words.say(Word::Stopping, &[]), look::SIGNAL)),
        BoxState::Stopped { .. } => Some((words.say(Word::BoxStopped, &[]), look::DOWN)),
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
    Some(LogNote { head: words.say(Word::LogHead, &[]), lines: log_lines(tui, words), end: tui.revision() })
}
fn log_lines(tui: &Tui, words: &Words) -> Box<[Box<[u8]>]> {
    let mut lines = List::with_capacity(limits::BOX_LOG);
    for line in tui.box_log() {
        lines
            .push(copy_of(utf8_prefix(&lossy(line), usize::try_from(limits::LOG_LINE).expect("log line fits memory"))))
            .expect("the box keeps at most BOX_LOG lines");
    }
    if lines.is_empty() {
        lines.push(words.say(Word::LogEmpty, &[])).expect("room for the empty log's line");
    }
    lines.into_boxed()
}
pub(crate) fn gave_up_note(tui: &Tui, words: &Words) -> Option<Box<[u8]>> {
    match tui.box_state() {
        BoxState::Running | BoxState::Restarting { .. } | BoxState::Stopping => None,
        BoxState::Stopped { end } => {
            if tui.stop_asked() {
                None
            } else {
                Some(words.say(Word::BoxEndNote, &[Hole::Name(end_words(end, words))]))
            }
        }
        BoxState::GaveUp { step: bytes } => Some(words.say(Word::GaveUpNote, &[Hole::Name(step(bytes))])),
    }
}
