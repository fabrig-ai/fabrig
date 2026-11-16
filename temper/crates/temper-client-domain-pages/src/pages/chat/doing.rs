//! The pane's live words and one-second call hold (client/pages/chat/document.md, 1).
use super::{Chat, document};
use alloc::boxed::Box;
use jig_client::{contract as c, thread::Line};
use skein_lib::{Duration, Time, Wall};

/// What the pane says, independent of the conversation's run line.
#[derive(Debug, PartialEq, Eq)]
pub enum What {
    Thinking,
    Writing,
    Call(Box<c::CurrentCall>),
}

/// A live pane's words, timed from the person's ask.
#[derive(Debug)]
pub struct Doing<'a> {
    pub what: &'a What,
    pub since: Wall,
}

#[derive(Debug)]
pub(super) struct Held {
    what: What,
    at: Time,
    deadline: Option<Time>,
}
impl Held {
    pub fn new() -> Self {
        Self { what: What::Thinking, at: Time::ZERO, deadline: None }
    }
    pub fn deadline(&self) -> Option<Time> {
        self.deadline
    }
    pub fn due(&self, now: Time) -> bool {
        match self.deadline {
            Some(at) => now >= at,
            None => false,
        }
    }
    fn change(&mut self, now: Time, what: What) {
        self.deadline = None;
        if self.what == what {
            return;
        }
        let call = match self.what {
            What::Thinking => false,
            What::Writing | What::Call(_) => true,
        };
        let until = self.at.saturating_add(Duration::from_secs(1));
        if call && now < until {
            self.deadline = Some(until);
        } else {
            self.what = what;
            self.at = now;
        }
    }
}
impl Chat {
    /// The path's right, with or without a document (client/pages/chat/document.md, 1).
    #[must_use]
    pub fn doing(&self) -> Option<Doing<'_>> {
        let running = match &self.current_run {
            Some(run) => run.attempt.is_some(),
            None => self.run.is_some(),
        };
        if !running && (self.asked_at.is_none() || self.draft.is_some()) {
            return None;
        }
        if self.draft.is_none() && !self.writing && self.asked_at.is_none() && self.write_up_at().is_none() {
            return None;
        }
        let since = if self.draft.is_none() {
            self.asked_at.or(self.write_up_at()).or(self.run_since())
        } else {
            self.run_since()
        }?;
        Some(Doing { what: &self.doing.what, since })
    }

    pub(super) fn doing_changed(&mut self, now: Time) {
        if self.mode == document::Mode::Chat || self.doing().is_none() {
            self.doing = Held::new();
            return;
        }
        let writing = match &self.writing_tool {
            Some(tool) => document::writes(tool.bytes()),
            None => false,
        };
        let what = if writing {
            What::Writing
        } else {
            match self.current_call() {
                Some(call) if document::writes(call.tool.bytes()) => What::Writing,
                Some(call) => What::Call(Box::new(call.clone())),
                None => What::Thinking,
            }
        };
        self.doing.change(now, what);
    }

    fn write_up_at(&self) -> Option<Wall> {
        for line in &self.thread.lines.rows {
            if let Some((_, words)) = line.word_message()
                && words.bytes() == document::WRITE_THIS_UP
            {
                return Some(line.source.at);
            }
        }
        None
    }

    fn run_since(&self) -> Option<Wall> {
        let attempt = match &self.current_run {
            Some(run) => run.attempt,
            None => self.thread.attempt,
        };
        let mut before = None;
        for line in self.thread.lines().into_boxed() {
            match line {
                Line::Turn { name, at, .. } | Line::Undecoded { name, at, .. } => {
                    if Some(name.attempt) == attempt {
                        return Some(before.unwrap_or(at));
                    }
                    before = Some(at);
                }
                Line::Words { at, .. } => before = at.or(before),
                Line::Activity { line } => before = Some(line.source.at),
                Line::Provisional { .. } => {}
            }
        }
        before.or(self.run_began)
    }
}
