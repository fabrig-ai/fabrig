//! The command’s Copy (temper/docs/design/client/README.md, 3.5).
use alloc::boxed::Box;
use skein_lib::{Duration, Queue, Time};

pub const SHOWN: Duration = Duration::from_secs(2);
/// What the shell’s clipboard said.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Written {
    Done,
    Refused,
    Failed,
}
/// What the button reads.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Label {
    Copy,
    Copied,
    NotCopied,
}

#[derive(Debug)]
pub(crate) struct Copy {
    token: u32,
    label: Label,
    deadline: Option<Time>,
    pub(crate) shown: u64,
}
impl Copy {
    pub(crate) const fn new() -> Self {
        Self { token: 0, label: Label::Copy, deadline: None, shown: 0 }
    }
    pub(crate) const fn label(&self) -> Label {
        self.label
    }
    pub(crate) const fn next_deadline(&self) -> Option<Time> {
        self.deadline
    }
    pub(crate) fn press(&mut self, words: Box<[u8]>, out: &mut Queue<crate::Request>) {
        self.token = self.token.checked_add(1).expect("copy tokens fit");
        out.push(crate::Request::Copy { token: self.token, words });
    }
    pub(crate) fn copied(&mut self, now: Time, token: u32, written: Written) {
        if token != self.token || token == 0 {
            return;
        }
        self.label = match written {
            Written::Done => Label::Copied,
            Written::Refused | Written::Failed => Label::NotCopied,
        };
        self.deadline = Some(now.saturating_add(SHOWN));
        self.changed();
    }
    pub(crate) fn fire(&mut self, now: Time) {
        if let Some(deadline) = self.deadline
            && now >= deadline
        {
            self.label = Label::Copy;
            self.deadline = None;
            self.changed();
        }
    }
    fn changed(&mut self) {
        self.shown = self.shown.checked_add(1).expect("copy revision fits");
    }
}
