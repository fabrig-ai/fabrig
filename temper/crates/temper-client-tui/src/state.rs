//! The TUI's own state beside the client root (temper/docs/design/client/tui/pages/README.md and temper/docs/design/client/tui/box.md and temper/docs/design/client/README.md, 4.1).
use crate::{BoxEnd, BoxLine, BoxStage, Scheme, limits, theme::ThemeChoice};
use alloc::boxed::Box;
use jig_client::contract::{ObjectKey, TurnName};
use skein_lib::{List, Queue, Secret, bytes::copy_of};

#[derive(Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "the TUI holds independent sign-in, credential, front and note facts"
)]
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
    aside: Option<ObjectKey>,
    reply_aside: Option<TurnName>,
    start_chat: Option<u64>,
    notice: Option<Notice>,
    focus: u64,
    box_written: u64,
    left_off: Option<u64>,
    boxed: bool,
    front: bool,
    box_stage: BoxStage,
    stop_asked: bool,
    printed: Option<Secret>,
    box_log: Queue<Box<[u8]>>,
    restarting: Option<Box<[u8]>>,
    gave_up: Option<Box<[u8]>>,
    quitting: bool,
    log_shown: bool,
    walk: Option<Walk>,
    brief_shown: bool,
    not_opened: Option<Box<[u8]>>,
    theme_choice: ThemeChoice,
    terminal_scheme: Option<Scheme>,
    theme_list: bool,
}
/// The box's state, given up winning (temper/docs/design/client/tui/box.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoxState<'a> {
    Running,
    Restarting { step: &'a [u8] },
    Stopping,
    Stopped { end: BoxEnd },
    GaveUp { step: &'a [u8] },
}
/// The sign-in from the engine's link (temper/docs/design/client/tui/pages/README.md).
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
/// A one-line notice on the quiet line (temper/docs/design/client/tui/pages/README.md).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Notice {
    Unknown { words: Box<[u8]> },
    SigningIn,
    Proposed,
    Theme { choice: ThemeChoice, terminal: Option<Scheme> },
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Told {
    Walk {
        by_key: bool,
    },
    Walked {
        object: ObjectKey,
    },
    EndWalk,
    Brief,
    NotOpened {
        address: Option<Box<[u8]>>,
    },
    Size {
        columns: u16,
    },
    Went,
    Starting {
        chat: Option<u64>,
    },
    LeftOff {
        task: Option<u64>,
    },
    Boxed,
    Cleared,
    Aside {
        object: Option<ObjectKey>,
    },
    ReplyAside {
        turn: TurnName,
    },
    Notice {
        notice: Option<Notice>,
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
    /// The authority of the first local link handed by the box.
    Address {
        address: Box<[u8]>,
    },
    Stage {
        stage: BoxStage,
    },
    BoxLine {
        line: BoxLine,
    },
    Quitting,
    Stay,
    Log,
    Theme {
        choice: ThemeChoice,
        terminal: Option<Scheme>,
    },
    ThemeList {
        open: bool,
    },
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
            reply_aside: None,
            start_chat: None,
            notice: None,
            focus: 1,
            box_written: 0,
            front: false,
            box_stage: BoxStage::Finding,
            stop_asked: false,
            printed: None,
            box_log: Queue::with_capacity(limits::BOX_LOG),
            restarting: None,
            gave_up: None,
            quitting: false,
            log_shown: false,
            walk: None,
            brief_shown: false,
            not_opened: None,
            left_off: None,
            boxed: false,
            theme_choice: ThemeChoice::Auto,
            terminal_scheme: None,
            theme_list: false,
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
    pub const fn aside(&self) -> Option<ObjectKey> {
        self.aside
    }
    #[must_use]
    pub const fn reply_aside(&self) -> Option<TurnName> {
        self.reply_aside
    }
    /// The chat kept while its goal's start sheet stands (client/tui/pages/design-session.md).
    #[must_use]
    pub const fn start_chat(&self) -> Option<u64> {
        self.start_chat
    }
    #[must_use]
    pub fn notice(&self) -> Option<&Notice> {
        self.notice.as_ref()
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
    pub const fn box_stage(&self) -> &BoxStage {
        &self.box_stage
    }
    pub(crate) const fn stop_asked(&self) -> bool {
        self.stop_asked
    }
    #[must_use]
    pub fn box_state(&self) -> BoxState<'_> {
        if let Some(step) = &self.gave_up {
            return BoxState::GaveUp { step };
        }
        match self.box_stage {
            BoxStage::Ended { end } => BoxState::Stopped { end },
            BoxStage::Stopping => BoxState::Stopping,
            BoxStage::Finding
            | BoxStage::Fetching { .. }
            | BoxStage::Starting
            | BoxStage::Running
            | BoxStage::Failed { .. } => match &self.restarting {
                Some(step) => BoxState::Restarting { step },
                None => BoxState::Running,
            },
        }
    }
    fn stage(&mut self, stage: BoxStage) -> bool {
        match &self.box_stage {
            BoxStage::Ended { .. } => return false,
            BoxStage::Failed { .. } => {
                let BoxStage::Ended { .. } = &stage else { return false };
            }
            BoxStage::Finding
            | BoxStage::Fetching { .. }
            | BoxStage::Starting
            | BoxStage::Running
            | BoxStage::Stopping => {}
        }
        let stage = match stage {
            BoxStage::Failed { why } => {
                let first = skein_lib::bytes::find(&why, b"\n").unwrap_or(why.len());
                let end = first.min(usize::try_from(limits::LOG_LINE).expect("log bound fits memory"));
                BoxStage::Failed { why: copy_of(why.get(..end).expect("bounded first line")) }
            }
            BoxStage::Finding
            | BoxStage::Fetching { .. }
            | BoxStage::Starting
            | BoxStage::Running
            | BoxStage::Stopping
            | BoxStage::Ended { .. } => stage,
        };
        let changed = self.box_stage != stage;
        if stage == BoxStage::Stopping {
            self.stop_asked = true;
        }
        self.box_stage = stage;
        changed
    }
    #[must_use]
    pub const fn quitting(&self) -> bool {
        self.quitting
    }
    #[must_use]
    pub const fn log_shown(&self) -> bool {
        self.log_shown
    }
    #[must_use]
    pub fn walk(&self) -> Option<&Walk> {
        self.walk.as_ref()
    }
    #[must_use]
    pub const fn brief_shown(&self) -> bool {
        self.brief_shown
    }
    #[must_use]
    pub fn not_opened(&self) -> Option<&[u8]> {
        self.not_opened.as_deref()
    }
    #[must_use]
    pub const fn theme_choice(&self) -> ThemeChoice {
        self.theme_choice
    }
    #[must_use]
    pub const fn terminal_scheme(&self) -> Option<Scheme> {
        self.terminal_scheme
    }
    #[must_use]
    pub const fn theme_list(&self) -> bool {
        self.theme_list
    }
    fn clear_launchpad(&mut self) -> bool {
        let changed = self.walk.is_some() || self.brief_shown || self.not_opened.is_some() || self.theme_list;
        self.walk = None;
        self.theme_list = false;
        self.brief_shown = false;
        self.not_opened = None;
        changed
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
    fn set_link(&mut self, link: jig_client::Link) -> bool {
        match link {
            jig_client::Link::SignedOut => {
                if self.linked {
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
            Told::Walk { by_key } => {
                let changed = match &self.walk {
                    Some(walk) => walk.by_key != by_key || walk.taken,
                    None => true,
                };
                self.walk = Some(Walk { by_key, shown: List::with_capacity(limits::WALK), taken: false });
                changed
            }
            Told::Walked { object } => match &mut self.walk {
                Some(walk) => {
                    let mut changed = !walk.taken;
                    walk.taken = true;
                    if !walk.shown(&object) && walk.shown.room() > 0 {
                        walk.shown.push(object).expect("room for a shown object");
                        changed = true;
                    }
                    changed
                }
                None => false,
            },
            Told::EndWalk => self.walk.take().is_some(),
            Told::Brief => {
                let changed = !self.brief_shown;
                self.brief_shown = true;
                changed
            }
            Told::NotOpened { address } => {
                let changed = self.not_opened != address;
                self.not_opened = address;
                changed
            }
            Told::Front => {
                let changed = !self.front;
                self.front = true;
                changed
            }
            Told::Address { address } => {
                let changed = self.address != address;
                self.address = address;
                changed
            }
            Told::Stage { stage } => self.stage(stage),
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
            Told::Starting { chat } => {
                let changed = self.start_chat != chat;
                self.start_chat = chat;
                changed
            }
            Told::Went => {
                self.start_chat = None;
                let focus = self.focus.saturating_add(1);
                let written = self.box_written.saturating_add(1);
                let changed =
                    self.focus != focus || self.box_written != written || self.log_shown || self.left_off.is_some();
                let cleared = self.clear_launchpad();
                self.log_shown = false;
                self.left_off = None;
                self.focus = focus;
                self.command_words = Box::default();
                self.box_written = written;
                changed || cleared
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
                let cleared = self.clear_launchpad();
                self.command_words = Box::default();
                let written = self.box_written.saturating_add(1);
                let changed = self.box_written != written || self.log_shown;
                self.log_shown = false;
                self.box_written = written;
                changed || cleared
            }
            Told::Aside { object } => {
                let changed = self.aside != object;
                self.aside = object;
                changed
            }
            Told::ReplyAside { turn } => {
                let changed = self.reply_aside != Some(turn);
                self.reply_aside = Some(turn);
                changed
            }
            Told::Notice { notice } => {
                let changed = self.notice != notice;
                self.notice = notice;
                changed
            }
            Told::Local { local } => self.set_local(local),
            Told::Typed { value } => {
                let signing = match self.signing {
                    Signing::Refused(..) | Signing::Answered { .. } | Signing::Idle => Signing::Idle,
                    Signing::Asked => Signing::Asked,
                };
                let ended = !value.is_empty() && self.walk.take().is_some();
                let changed = self.field != value || self.signing != signing || ended;
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
            Told::Theme { choice, terminal } => {
                let changed = self.theme_choice != choice || self.terminal_scheme != terminal;
                self.theme_choice = choice;
                self.terminal_scheme = terminal;
                changed
            }
            Told::ThemeList { open } => {
                let changed = self.theme_list != open;
                self.theme_list = open;
                changed
            }
        };
        if changed {
            self.revision = self.revision.saturating_add(1);
        }
    }
}

/// The needs already shown during this walk (temper/docs/design/client/tui/pages/README.md).
#[derive(Debug)]
pub struct Walk {
    shown: List<ObjectKey>,
    by_key: bool,
    taken: bool,
}
impl Walk {
    #[must_use]
    pub fn shown(&self, object: &ObjectKey) -> bool {
        self.shown.as_slice().contains(object)
    }
    #[must_use]
    pub const fn by_key(&self) -> bool {
        self.by_key
    }
    #[must_use]
    pub const fn taken(&self) -> bool {
        self.taken
    }
}
