//! The TUI's own state beside the client root (temper/docs/design/client/README.md, 6.14, 6.17 and 7.1).
use crate::{BoxLine, limits};
use alloc::boxed::Box;
use skein_lib::{Queue, Secret, bytes::copy_of};

#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "the TUI holds independent sign-in, credential and front facts")]
pub struct Tui {
    address: Box<[u8]>,
    field: Box<[u8]>,
    command_words: Box<[u8]>,
    signing: Signing,
    local: Local,
    browser: Option<Browser>,
    pasted: bool,
    signed_out: bool,
    unkept: bool,
    linked: bool,
    openings: u64,
    revision: u64,
    written: u64,
    columns: u16,
    aside: Option<jig_client::contract::ObjectKey>,
    unknown: Option<Box<[u8]>>,
    focus: u64,
    box_written: u64,
    left_off: Option<u64>,
    boxed: bool,
    front: bool,
    printed: Option<Secret>,
    box_log: Queue<Box<[u8]>>,
    restarting: Option<Box<[u8]>>,
    gave_up: Option<Box<[u8]>>,
    quitting: bool,
    log_shown: bool,
}
/// The box's state, given up winning (temper/docs/design/client/README.md, 6.17).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoxState<'a> {
    Running,
    Restarting { step: &'a [u8] },
    GaveUp { step: &'a [u8] },
}
/// The sign-in from the engine's link (temper/docs/design/client/README.md, 6.14).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Local {
    Idle,
    Signing,
    Ended,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Browser {
    Opening,
    Opened,
    None,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Signing {
    Idle,
    Refused(Refusal),
    Asked,
    Answered { lost: bool },
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Refusal {
    NotALink,
    OtherAddress { address: Box<[u8]> },
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Told {
    Size {
        columns: u16,
    },
    Went,
    LeftOff {
        task: Option<u64>,
    },
    Boxed,
    Cleared,
    Aside {
        object: Option<jig_client::contract::ObjectKey>,
    },
    Unknown {
        words: Option<Box<[u8]>>,
    },
    Local {
        local: Local,
    },
    Typed {
        value: Box<[u8]>,
    },
    /// Replace the no-project composer without changing the sign-in field.
    CommandWords {
        value: Box<[u8]>,
    },
    Refused {
        refusal: Refusal,
    },
    Asked,
    Browse,
    Opened {
        opened: bool,
    },
    Answered {
        lost: bool,
    },
    Link {
        link: jig_client::Link,
    },
    Kept {
        written: bool,
    },
    Front,
    BoxLine {
        line: BoxLine,
    },
    Quitting,
    Stay,
    Log,
}
impl Tui {
    #[must_use]
    pub fn new(address: &[u8]) -> Tui {
        Tui {
            address: copy_of(address),
            field: copy_of(b""),
            command_words: Box::default(),
            signing: Signing::Idle,
            local: Local::Idle,
            browser: None,
            pasted: false,
            signed_out: false,
            unkept: false,
            linked: false,
            openings: 1,
            revision: 0,
            written: 0,
            columns: 0,
            aside: None,
            unknown: None,
            focus: 1,
            box_written: 0,
            front: false,
            printed: None,
            box_log: Queue::with_capacity(limits::BOX_LOG),
            restarting: None,
            gave_up: None,
            quitting: false,
            log_shown: false,
            left_off: None,
            boxed: false,
        }
    }
    #[must_use]
    pub fn address(&self) -> &[u8] {
        &self.address
    }
    #[must_use]
    pub fn field(&self) -> &[u8] {
        &self.field
    }
    /// Words in the command-only composer while no project is selected.
    #[must_use]
    pub fn command_words(&self) -> &[u8] {
        &self.command_words
    }
    #[must_use]
    pub const fn signing(&self) -> &Signing {
        &self.signing
    }
    #[must_use]
    pub const fn local(&self) -> Local {
        self.local
    }
    #[must_use]
    pub const fn browser(&self) -> Option<Browser> {
        self.browser
    }
    #[must_use]
    pub const fn pasted(&self) -> bool {
        self.pasted
    }
    #[must_use]
    pub const fn signed_out(&self) -> bool {
        self.signed_out
    }
    #[must_use]
    pub const fn unkept(&self) -> bool {
        self.unkept
    }
    #[must_use]
    pub const fn openings(&self) -> u64 {
        self.openings
    }
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }
    pub(crate) const fn written(&self) -> u64 {
        self.written
    }
    #[must_use]
    pub const fn columns(&self) -> u16 {
        self.columns
    }
    #[must_use]
    pub const fn aside(&self) -> Option<jig_client::contract::ObjectKey> {
        self.aside
    }
    #[must_use]
    pub fn unknown(&self) -> Option<&[u8]> {
        self.unknown.as_deref()
    }
    #[must_use]
    pub const fn focus(&self) -> u64 {
        self.focus
    }
    #[must_use]
    pub const fn box_written(&self) -> u64 {
        self.box_written
    }
    #[must_use]
    pub const fn front(&self) -> bool {
        self.front
    }
    #[must_use]
    pub const fn printed(&self) -> Option<&Secret> {
        self.printed.as_ref()
    }
    #[expect(clippy::disallowed_methods, reason = "the borrowed log iterator maps each owned line through Box::as_ref")]
    pub fn box_log(&self) -> impl Iterator<Item = &[u8]> {
        self.box_log.iter().map(Box::as_ref)
    }
    #[must_use]
    pub fn box_state(&self) -> BoxState<'_> {
        if let Some(step) = &self.gave_up {
            BoxState::GaveUp { step }
        } else if let Some(step) = &self.restarting {
            BoxState::Restarting { step }
        } else {
            BoxState::Running
        }
    }
    #[must_use]
    pub const fn quitting(&self) -> bool {
        self.quitting
    }
    #[must_use]
    pub const fn log_shown(&self) -> bool {
        self.log_shown
    }
    fn box_line(&mut self, line: BoxLine) -> bool {
        match line {
            BoxLine::Link { url } => {
                let changed = self.printed.as_ref() != Some(&url);
                self.printed = Some(url);
                changed
            }
            BoxLine::Log { line } => {
                if self.box_log.room() == 0 {
                    self.box_log.pop().expect("a full log has an oldest line");
                }
                self.box_log.push(line);
                true
            }
            BoxLine::Restarting { step } => {
                let changed = self.restarting.as_ref() != Some(&step);
                self.restarting = Some(step);
                changed
            }
            BoxLine::Running { step } => {
                if self.restarting.as_ref() == Some(&step) {
                    self.restarting = None;
                    true
                } else {
                    false
                }
            }
            BoxLine::GaveUp { step } => {
                if self.gave_up.is_none() {
                    self.gave_up = Some(step);
                    true
                } else {
                    false
                }
            }
        }
    }
    #[must_use]
    pub const fn left_off(&self) -> Option<u64> {
        self.left_off
    }
    #[must_use]
    pub const fn boxed(&self) -> bool {
        self.boxed
    }
    fn empty(&mut self) {
        self.field = copy_of(b"");
        self.written = self.written.saturating_add(1);
    }
    fn set_local(&mut self, local: Local) -> bool {
        let mut changed = self.local != local;
        if local == Local::Ended {
            if self.local != Local::Ended {
                let focus = self.focus.saturating_add(1);
                changed |= self.focus != focus;
                self.focus = focus;
            }
            changed |= self.signing != Signing::Idle || self.pasted || !self.field.is_empty();
            if !self.field.is_empty() {
                self.empty();
            }
            self.signing = Signing::Idle;
            self.pasted = false;
        }
        self.local = local;
        changed
    }
    fn clear_command_words(&mut self) {
        if !self.command_words.is_empty() {
            self.command_words = Box::default();
            self.box_written = self.box_written.saturating_add(1);
        }
    }
    fn set_link(&mut self, link: jig_client::Link) -> bool {
        match link {
            jig_client::Link::SignedOut => {
                if self.linked {
                    self.clear_command_words();
                    self.signing = Signing::Idle;
                    self.browser = None;
                    self.pasted = false;
                    self.empty();
                    self.linked = false;
                    self.signed_out = true;
                    self.openings = self.openings.saturating_add(1);
                    self.focus = self.focus.saturating_add(1);
                    true
                } else {
                    false
                }
            }
            jig_client::Link::Starting
            | jig_client::Link::Live
            | jig_client::Link::Offline
            | jig_client::Link::OutOfDate => {
                if !self.linked {
                    self.clear_command_words();
                    self.focus = self.focus.saturating_add(1);
                }
                let changed = self.signing != Signing::Idle
                    || !self.field.is_empty()
                    || !self.linked
                    || self.signed_out
                    || self.local != Local::Idle;
                if changed {
                    self.empty();
                }
                self.signing = Signing::Idle;
                self.local = Local::Idle;
                self.linked = true;
                self.signed_out = false;
                changed
            }
        }
    }
    #[expect(clippy::too_many_lines, reason = "one dispatch updates the TUI's independent surface facts")]
    pub fn step(&mut self, told: Told) {
        let changed = match told {
            Told::Front => {
                let changed = !self.front;
                self.front = true;
                changed
            }
            Told::BoxLine { line } => self.box_line(line),
            Told::Quitting => {
                let changed = !self.quitting;
                self.quitting = true;
                changed
            }
            Told::Stay => {
                let changed = self.quitting;
                self.quitting = false;
                changed
            }
            Told::Log => {
                let changed = !self.log_shown;
                self.log_shown = true;
                changed
            }
            Told::Size { columns } => {
                let changed = self.columns != columns;
                self.columns = columns;
                changed
            }
            Told::Went => {
                let focus = self.focus.saturating_add(1);
                let written = self.box_written.saturating_add(1);
                let changed =
                    self.focus != focus || self.box_written != written || self.log_shown || self.left_off.is_some();
                self.log_shown = false;
                self.left_off = None;
                self.focus = focus;
                self.command_words = Box::default();
                self.box_written = written;
                changed
            }
            Told::LeftOff { task } => {
                let changed = self.left_off != task;
                self.left_off = task;
                changed
            }
            Told::Boxed => {
                let changed = !self.boxed;
                self.boxed = true;
                changed
            }
            Told::CommandWords { value } => {
                let changed = self.command_words != value;
                self.command_words = value;
                changed
            }
            Told::Cleared => {
                self.command_words = Box::default();
                let written = self.box_written.saturating_add(1);
                let changed = self.box_written != written;
                self.box_written = written;
                changed
            }
            Told::Aside { object } => {
                let changed = self.aside != object;
                self.aside = object;
                changed
            }
            Told::Unknown { words } => {
                let changed = self.unknown != words || self.log_shown;
                self.log_shown = false;
                self.unknown = words;
                changed
            }
            Told::Local { local } => self.set_local(local),
            Told::Typed { value } => {
                let signing = match self.signing {
                    Signing::Refused(..) | Signing::Answered { .. } | Signing::Idle => Signing::Idle,
                    Signing::Asked => Signing::Asked,
                };
                let changed = self.field != value || self.signing != signing;
                self.field = value;
                self.signing = signing;
                changed
            }
            Told::Refused { refusal } => {
                let signing = Signing::Refused(refusal);
                let changed = self.signing != signing;
                self.signing = signing;
                changed
            }
            Told::Asked => {
                let changed = self.signing != Signing::Asked
                    || !self.field.is_empty()
                    || !self.pasted
                    || self.local == Local::Ended;
                if self.local == Local::Ended {
                    self.local = Local::Idle;
                }
                if changed {
                    self.empty();
                }
                self.signing = Signing::Asked;
                self.pasted = true;
                changed
            }
            Told::Browse => {
                let changed = self.browser != Some(Browser::Opening) || self.pasted;
                self.browser = Some(Browser::Opening);
                self.pasted = false;
                changed
            }
            Told::Opened { opened } => {
                let browser = Some(if opened { Browser::Opened } else { Browser::None });
                let changed = self.browser != browser;
                self.browser = browser;
                changed
            }
            Told::Answered { lost } => match self.signing {
                Signing::Asked => {
                    self.signing = Signing::Answered { lost };
                    true
                }
                Signing::Idle | Signing::Refused(..) | Signing::Answered { .. } => false,
            },
            Told::Link { link } => self.set_link(link),
            Told::Kept { written } => {
                let changed = self.unkept == written;
                self.unkept = !written;
                changed
            }
        };
        if changed {
            self.revision = self.revision.saturating_add(1);
        }
    }
}
