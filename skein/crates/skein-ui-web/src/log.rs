//! The session log's text and its keeping (playground.md, 6).
use crate::{bridge, fetch};
use skein_lib::{Time, Wall, sha256::Sha256};

mod reader;
mod text;

/// A digest of what the world said between two inputs (playground.md, 6.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Said(pub [u8; 32]);

/// The fold of what the world said: its reports and its lines apart, and its stop.
#[derive(Clone, Debug)]
pub struct Saying {
    reports: Sha256,
    lines: Sha256,
    stopped: bool,
}
impl Saying {
    #[must_use]
    #[expect(clippy::new_without_default, reason = "the session log contract names new")]
    pub fn new() -> Saying {
        Saying { reports: Sha256::new(), lines: Sha256::new(), stopped: false }
    }
    pub fn report(&mut self, report: &fetch::Report) {
        self.reports.update(text::report(report).as_bytes());
    }
    pub fn printed(&mut self, printed: &bridge::Printed) {
        self.lines.update(text::printed(printed).as_bytes());
    }
    pub fn stopped(&mut self) {
        self.stopped = true;
    }
    #[must_use]
    pub fn said(&self) -> Said {
        let mut digest = Sha256::new();
        digest.update(&self.reports.clone().finish());
        digest.update(&self.lines.clone().finish());
        digest.update(&[u8::from(self.stopped)]);
        Said(digest.finish())
    }
}

/// One kept input: its stamp, the input, and what the world said since the one before.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub stamp: bridge::Stamp,
    pub input: bridge::Input,
    pub said: Said,
}

/// How a log ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    /// The last clock the page heard, and what the world said from the last input to it.
    At { step: u64, instant: Time, said: Said },
    /// The log reached its bound; a replay runs to the last input kept.
    Cut,
}

/// A log read back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Read {
    /// The bundle's digest directory's name; `None` for `-`, a log kept natively.
    pub build: Option<Box<[u8]>>,
    pub seed: u64,
    pub start: Wall,
    pub entries: Vec<Entry>,
    pub end: End,
}

/// Why a text is no log this build reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The first line names another format.
    Format,
    /// Another version of the format, named as the log names it.
    Version { found: Box<[u8]> },
    /// A line that is not the format's, from 1.
    Line { line: u32 },
}

/// The page's log, kept while the visit runs (playground.md, 6).
#[derive(Debug)]
pub struct Log {
    text: Vec<u8>,
    waiting: std::collections::VecDeque<bridge::Input>,
    unstamped: usize,
    saying: Saying,
    end: End,
    bound: usize,
    last_room: usize,
}
impl Log {
    /// `build` is the bundle's digest directory's name, `None` natively; `bound` is the
    /// text's most bytes, the application's.
    #[must_use]
    pub fn new(build: Option<&[u8]>, start: &bridge::Start, bound: u32) -> Log {
        let saying = Saying::new();
        let end = End::At { step: 0, instant: Time::ZERO, said: saying.said() };
        // Reserve the longest end line: a later clock can grow both decimal fields.
        let last_room =
            text::end(End::At { step: u64::MAX, instant: Time::from_nanos(u64::MAX), said: saying.said() }).len();
        let mut log = Log {
            text: text::first(build, start.seed, start.start),
            waiting: std::collections::VecDeque::new(),
            unstamped: 0,
            saying,
            end,
            bound: usize::try_from(bound).expect("a log bound fits usize"),
            last_room,
        };
        if log.text.len().saturating_add(text::end(end).len()) > log.bound {
            log.end = End::Cut;
        }
        log
    }
    /// An input the page sent, kept until its stamp comes.
    pub fn sent(&mut self, input: &bridge::Input) {
        self.unstamped = self.unstamped.checked_add(1).expect("the number of unstamped inputs fits usize");
        match self.end {
            End::At { .. } => self.waiting.push_back(input.clone()),
            End::Cut => {}
        }
    }
    /// A world's message: a stamp keeps the oldest input sent and unstamped; a report, a line
    /// and the stop are folded; a clock marks the end.
    pub fn heard(&mut self, message: &bridge::ToPage) {
        match message {
            bridge::ToPage::Stamp(stamp) => self.stamped(*stamp),
            bridge::ToPage::Report(report) => match self.end {
                End::At { .. } => self.saying.report(report),
                End::Cut => {}
            },
            bridge::ToPage::Printed(printed) => match self.end {
                End::At { .. } => self.saying.printed(printed),
                End::Cut => {}
            },
            bridge::ToPage::Stopped => match self.end {
                End::At { .. } => self.saying.stopped(),
                End::Cut => {}
            },
            bridge::ToPage::Clock(clock) => match self.end {
                End::At { .. } => {
                    self.end = End::At { step: clock.step, instant: clock.now, said: self.saying.said() };
                    if self.text.len().saturating_add(text::end(self.end).len()) > self.bound {
                        self.end = End::Cut;
                        self.waiting.clear();
                    }
                }
                End::Cut => {}
            },
            bridge::ToPage::Failed { words: _ } => {}
        }
    }
    fn stamped(&mut self, stamp: bridge::Stamp) {
        self.unstamped = self.unstamped.checked_sub(1).expect("a stamp has an input waiting");
        match self.end {
            End::Cut => return,
            End::At { .. } => {}
        }
        let input = self.waiting.pop_front().expect("a stamp has an input waiting");
        let entry = Entry { stamp, input, said: self.saying.said() };
        let line = text::entry(&entry);
        if self.text.len().saturating_add(line.len()).saturating_add(self.last_room) > self.bound {
            self.end = End::Cut;
            self.waiting.clear();
            return;
        }
        self.text.extend_from_slice(line.as_bytes());
        self.saying = Saying::new();
    }
    /// The file offered: its first line, each kept input's, and its last line.
    #[must_use]
    pub fn text(&self) -> Box<[u8]> {
        let mut out = self.text.clone();
        out.extend_from_slice(text::end(self.end).as_bytes());
        out.into_boxed_slice()
    }
}

/// Reads exactly the session log's versioned text (playground.md, 6.1).
pub fn read(text: &[u8]) -> Result<Read, Refused> {
    reader::read(text)
}
