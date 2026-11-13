//! Replay and prove through the shipped host (temper/docs/design/client/tui/journeys.md, 4).
use super::{
    Facts, Line, Streamed, Unreadable, lines,
    record::{asked, duty_order, is_ask},
};
use crate::host::{Host, Report};
use skein_lib::{Secret, Token};
use skein_ui_screen::Frame;
use skein_ui_screen_terminal::Key;

pub struct Mark {
    pub frame: Frame,
    pub live: Option<Box<[u8]>>,
    pub key: Option<Key>,
}

/// The first differing seam line. Debug keeps the private recording's bytes out of logs.
#[derive(Clone, PartialEq, Eq)]
pub struct Stale {
    pub turn: u32,
    pub line: u32,
    pub recorded: Box<[u8]>,
    pub asked: Box<[u8]>,
}
impl core::fmt::Debug for Stale {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Stale")
            .field("turn", &self.turn)
            .field("line", &self.line)
            .field("recorded_len", &self.recorded.len())
            .field("asked_len", &self.asked.len())
            .finish()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    Unreadable(Unreadable),
    Stale(Stale),
    Differs { mark: u32, line: u32 },
}

pub struct Replay {
    lines: Vec<Line>,
    at: usize,
    turn: u32,
    host: Host,
    start: Option<Report>,
    ended: bool,
    failed: Option<Stale>,
}
impl Replay {
    pub fn new(recording: &[u8]) -> Result<Self, Unreadable> {
        let lines = lines(recording)?;
        let (facts, at) = facts(&lines)?;
        envelope(&lines, at)?;
        let (host, start) = facts.start();
        Ok(Self { lines, at, turn: 0, host, start: Some(start), ended: false, failed: None })
    }

    pub fn host(&mut self) -> &mut Host {
        &mut self.host
    }

    /// Advance to a mark, retaining the first stale result if replay has stopped.
    #[expect(clippy::should_implement_trait, reason = "the replay's fallible seam is the journey contract")]
    pub fn next(&mut self) -> Result<Option<Mark>, Stale> {
        if let Some(stale) = &self.failed {
            return Err(stale.clone());
        }
        let result = self.advance();
        if let Err(stale) = &result {
            self.failed = Some(stale.clone());
        }
        result
    }

    fn advance(&mut self) -> Result<Option<Mark>, Stale> {
        if self.ended {
            return Ok(None);
        }
        loop {
            let record = self.lines.get(self.at).expect("validated envelope has a final end");
            match record {
                Line::Turn { now, wall } => {
                    self.host.clocks(*now, *wall);
                    self.turn += 1;
                    self.at += 1;
                }
                Line::Mark(live) | Line::End(live) => {
                    let live = live.clone();
                    let end = matches!(record, Line::End(_));
                    self.at += 1;
                    self.ended = end;
                    let key = if let Some(Line::Key { key, .. }) = self.lines.get(self.at) {
                        Some(key.clone())
                    } else {
                        None
                    };
                    return Ok(Some(Mark { frame: self.host.frame().clone(), live, key }));
                }
                Line::Deadline(_) => {
                    // The run lays its newest frame after the turn's reports, including focus.
                    self.host.frame();
                    self.compare(&Line::Deadline(self.host.deadline()))?;
                }
                Line::Journey { .. }
                | Line::Seed(_)
                | Line::Clocks { .. }
                | Line::Screen { .. }
                | Line::Background(_)
                | Line::Box
                | Line::Front
                | Line::Local
                | Line::Noted(_)
                | Line::Address(_)
                | Line::Stored { .. }
                | Line::Credential(_) => {
                    unreachable!("facts occur only at the start of the validated envelope")
                }
                Line::Start
                | Line::Due
                | Line::Key { .. }
                | Line::Resize(_)
                | Line::Reply { .. }
                | Line::Unanswered(_)
                | Line::Streamed(_)
                | Line::Gone
                | Line::Saved
                | Line::Unsaved
                | Line::Kept { .. }
                | Line::BoxLine(_)
                | Line::BoxStage(_)
                | Line::Lent { .. }
                | Line::Found(_)
                | Line::Opened { .. }
                | Line::Send { .. }
                | Line::Stream(_)
                | Line::Unstream
                | Line::Save { .. }
                | Line::Erase(_)
                | Line::Note(_)
                | Line::Read
                | Line::Keep(_)
                | Line::Signed(_)
                | Line::Browse(_)
                | Line::StopBox
                | Line::StopBoxAgain
                | Line::Lend { .. }
                | Line::Unlend
                | Line::Quit => {
                    let report = self.report()?;
                    self.at += 1;
                    let duties = self.host.report(report);
                    for line in asked(&duties) {
                        self.compare(&line)?;
                    }
                    if self.lines.get(self.at).is_some_and(is_ask) {
                        return Err(self.stale(b"nothing".as_slice().into()));
                    }
                }
            }
        }
    }

    fn compare(&mut self, line: &Line) -> Result<(), Stale> {
        if self.lines.get(self.at) != Some(line) {
            return Err(self.stale(text(line)));
        }
        self.at += 1;
        Ok(())
    }

    fn report(&mut self) -> Result<Report, Stale> {
        let line = self.lines.get(self.at).expect("validated report line");
        let report = match line {
            Line::Start => self.start.take().expect("the envelope starts once"),
            Line::Due => Report::Due,
            Line::Lent { number, exit } => Report::Lent { number: *number, exit: *exit },
            Line::Key { key, event } => {
                let made = super::event(&self.host, key);
                if &made != event {
                    return Err(self.stale(text(&Line::Key { key: key.clone(), event: made })));
                }
                Report::Key(key.clone())
            }
            Line::Resize(size) => Report::Resize(*size),
            Line::Reply { exchange, bytes } => {
                Report::Heard(jig_client::Event::Reply { exchange: Token::new(*exchange), message: bytes.clone() })
            }
            Line::Unanswered(exchange) => Report::Heard(jig_client::Event::Lost { exchange: Token::new(*exchange) }),
            Line::Streamed(streamed) => Report::Heard(jig_client::Event::Streamed {
                message: match streamed {
                    Streamed::Record(bytes) => jig_client::Streamed::Record(bytes.clone()),
                    Streamed::Heartbeat => jig_client::Streamed::Heartbeat,
                    Streamed::SignedOut => jig_client::Streamed::SignedOut,
                    Streamed::Full { transport, size } => jig_client::Streamed::Full(jig_client::StreamsFull {
                        bound: if *transport {
                            jig_client::StreamBound::Transport
                        } else {
                            jig_client::StreamBound::Component
                        },
                        size: *size,
                    }),
                },
            }),
            Line::Gone => Report::Heard(jig_client::Event::StreamGone),
            Line::Saved => Report::Heard(jig_client::Event::Stored),
            Line::Unsaved => Report::Heard(jig_client::Event::NotStored),
            Line::Kept { written } => Report::Kept { written: *written },
            Line::Found(bytes) => Report::Local { link: bytes.clone().map(Secret::new) },
            Line::BoxLine(bytes) => Report::Front(temper_client_tui::BoxLine::read(bytes).expect("validated box line")),
            Line::BoxStage(bytes) => {
                Report::Stage(temper_client_tui::BoxStage::read(bytes).expect("validated box stage"))
            }
            Line::Opened { opened } => Report::Opened { opened: *opened },
            Line::Journey { .. }
            | Line::Seed(_)
            | Line::Clocks { .. }
            | Line::Screen { .. }
            | Line::Background(_)
            | Line::Box
            | Line::Front
            | Line::Local
            | Line::Noted(_)
            | Line::Address(_)
            | Line::Stored { .. }
            | Line::Credential(_)
            | Line::Turn { .. }
            | Line::Send { .. }
            | Line::Stream(_)
            | Line::Unstream
            | Line::Save { .. }
            | Line::Erase(_)
            | Line::Note(_)
            | Line::Read
            | Line::Keep(_)
            | Line::Signed(_)
            | Line::Browse(_)
            | Line::Quit
            | Line::StopBox
            | Line::StopBoxAgain
            | Line::Lend { .. }
            | Line::Unlend
            | Line::Deadline(_)
            | Line::Mark(_)
            | Line::End(_) => return Err(self.stale(b"nothing".as_slice().into())),
        };
        Ok(report)
    }

    fn stale(&self, asked: Box<[u8]>) -> Stale {
        Stale {
            turn: self.turn,
            line: number(self.at),
            recorded: self.lines.get(self.at).map_or_else(|| b"nothing".as_slice().into(), text),
            asked,
        }
    }
}

fn number(index: usize) -> u32 {
    u32::try_from(index + 1).expect("recording line count fits u32")
}
fn text(line: &Line) -> Box<[u8]> {
    let mut bytes = Vec::new();
    line.write(&mut bytes);
    bytes.pop();
    bytes.into()
}

pub(super) fn facts(lines: &[Line]) -> Result<(Facts, usize), Unreadable> {
    let error = |at| Unreadable { line: number(at), why: "expected ordered start facts for journey 2" };
    let mut at = 0;
    if lines.get(at) != Some(&Line::Journey { version: 2 }) {
        return Err(error(at));
    }
    at += 1;
    let Some(Line::Seed(seed)) = lines.get(at) else {
        return Err(error(at));
    };
    at += 1;
    let Some(Line::Clocks { now, wall }) = lines.get(at) else {
        return Err(error(at));
    };
    at += 1;
    let Some(Line::Screen { size, scheme }) = lines.get(at) else {
        return Err(error(at));
    };
    at += 1;
    let background = if let Some(Line::Background(rgb)) = lines.get(at) {
        at += 1;
        Some(*rgb)
    } else {
        None
    };
    let Some(Line::Address(address)) = lines.get(at) else {
        return Err(error(at));
    };
    at += 1;
    let local = lines.get(at) == Some(&Line::Local);
    if local {
        at += 1;
    }
    let front = lines.get(at) == Some(&Line::Front);
    if front {
        at += 1;
    }
    let boxed = lines.get(at) == Some(&Line::Box);
    if boxed {
        at += 1;
    }
    let mut kept = Vec::new();
    while let Some(Line::Stored { key, bytes }) = lines.get(at) {
        kept.push(jig_client::Record { key: key.clone(), bytes: bytes.clone() });
        at += 1;
    }
    let credential = if let Some(Line::Credential(bytes)) = lines.get(at) {
        at += 1;
        Some(Secret::new(bytes.clone()))
    } else {
        None
    };
    let note = if let Some(Line::Noted(note)) = lines.get(at) {
        at += 1;
        Some(*note)
    } else {
        None
    };
    Ok((
        Facts {
            seed: *seed,
            now: *now,
            wall: *wall,
            size: *size,
            scheme: *scheme,
            background,
            boxed,
            address: address.clone(),
            local,
            front,
            note,
            kept,
            credential,
        },
        at,
    ))
}

/// Validate placement before a malformed file can reach the host's assertions.
#[expect(clippy::too_many_lines, reason = "one envelope validates the ordered facts, reports and duties")]
fn envelope(lines: &[Line], at: usize) -> Result<(), Unreadable> {
    let mut in_turn = false;
    let mut kept = false;
    let mut started = false;
    let mut may_ask = false;
    let mut duty = None;
    for (index, line) in lines.iter().enumerate().skip(at) {
        let bad = || Unreadable { line: number(index), why: "record out of place" };
        match line {
            Line::Turn { .. } => {
                if in_turn {
                    return Err(bad());
                }
                in_turn = true;
                kept = false;
                may_ask = false;
            }
            Line::End(_) => {
                if in_turn || index + 1 != lines.len() {
                    return Err(bad());
                }
                return Ok(());
            }
            Line::Deadline(_) => {
                if !in_turn || !kept {
                    return Err(bad());
                }
                in_turn = false;
            }
            Line::Start => {
                if !in_turn || started || kept {
                    return Err(bad());
                }
                started = true;
                kept = true;
                may_ask = true;
                duty = None;
            }
            Line::Mark(_) => {
                if !in_turn || !started {
                    return Err(bad());
                }
                kept = true;
                may_ask = false;
            }
            Line::Due
            | Line::Key { .. }
            | Line::Resize(_)
            | Line::Reply { .. }
            | Line::Unanswered(_)
            | Line::Streamed(_)
            | Line::Gone
            | Line::Saved
            | Line::Unsaved
            | Line::Kept { .. }
            | Line::BoxLine(_)
            | Line::BoxStage(_)
            | Line::Lent { .. }
            | Line::Found(_)
            | Line::Opened { .. } => {
                if !in_turn || !started {
                    return Err(bad());
                }
                if let Line::BoxLine(bytes) = line
                    && temper_client_tui::BoxLine::read(bytes).is_none()
                {
                    return Err(Unreadable { line: number(index), why: "unreadable box line" });
                }
                if let Line::BoxStage(bytes) = line
                    && temper_client_tui::BoxStage::read(bytes).is_none()
                {
                    return Err(Unreadable { line: number(index), why: "unreadable box stage" });
                }
                kept = true;
                may_ask = true;
                duty = None;
            }
            Line::Send { .. }
            | Line::Stream(_)
            | Line::Unstream
            | Line::Save { .. }
            | Line::Erase(_)
            | Line::Note(_)
            | Line::Read
            | Line::Keep(_)
            | Line::Signed(_)
            | Line::Browse(_)
            | Line::StopBox
            | Line::StopBoxAgain
            | Line::Lend { .. }
            | Line::Unlend
            | Line::Quit => {
                if !in_turn || !may_ask {
                    return Err(bad());
                }
                let order = duty_order(line).expect("a duty record");
                if duty.is_some_and(|prior| order < prior) {
                    return Err(bad());
                }
                duty = Some(order);
            }
            Line::Journey { .. }
            | Line::Seed(_)
            | Line::Clocks { .. }
            | Line::Screen { .. }
            | Line::Background(_)
            | Line::Box
            | Line::Front
            | Line::Local
            | Line::Noted(_)
            | Line::Address(_)
            | Line::Stored { .. }
            | Line::Credential(_) => return Err(bad()),
        }
    }
    Err(Unreadable { line: number(lines.len()), why: "missing end" })
}

/// Prove each live frame, then remove the prints from marks and end.
pub fn prove(recording: &[u8]) -> Result<Box<[u8]>, Refusal> {
    let mut replay = Replay::new(recording).map_err(Refusal::Unreadable)?;
    let mut number = 0;
    while let Some(mark) = replay.next().map_err(Refusal::Stale)? {
        number += 1;
        let print = mark.frame.print();
        let Some(live) = mark.live else {
            return Err(Refusal::Differs { mark: number, line: 1 });
        };
        if print != live {
            let differing = difference(&print, &live);
            return Err(Refusal::Differs { mark: number, line: differing });
        }
    }
    let mut out = Vec::new();
    for mut line in replay.lines {
        if let Line::Mark(print) | Line::End(print) = &mut line {
            *print = None;
        }
        line.write(&mut out);
    }
    Ok(out.into())
}

fn difference(left: &[u8], right: &[u8]) -> u32 {
    let mut left = left.split(|byte| *byte == b'\n');
    let mut right = right.split(|byte| *byte == b'\n');
    let mut line = 1;
    loop {
        match (left.next(), right.next()) {
            (Some(a), Some(b)) if a == b => line += 1,
            (Some(_) | None, Some(_) | None) => return line,
        }
    }
}
