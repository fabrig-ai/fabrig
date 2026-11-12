//! Reports in and duties out (temper/docs/design/client/tui/README.md, 3).
use crate::limits::{self, Limits};
use jig_client::{Link, Message, With};
use skein_lib::{Duration, Env, List, Queue, Secret, Set, Time, Token, Wall};
use skein_ui::Patch;
use skein_ui_screen::{Cell, Focus, Frame, Rgb, Size, Theme};
use skein_ui_screen_terminal::{Command, Key, command};
use temper_client_domain::{
    self as root, Address, Event, Place,
    address::Filter,
    pages::{
        Act, needs,
        terminal::{self, Command as TuiCommand, Font, Look},
    },
};
use temper_client_shell_terminal::route::{self, Down, Duty};
use temper_client_tui::theme::{THEME_KEY, ThemeChoice};
use temper_client_tui::{self as tui, Local, Note, Refusal, Told};
use temper_client_view::shared;

pub const LOCAL_READS: u8 = 3;
pub const LOCAL_AGAIN: Duration = Duration::from_secs(1);
pub const EXPIRY_SLACK: Duration = Duration::from_secs(60);

struct Need {
    held: bool,
    reads: u8,
    digest: Option<[u8; 32]>,
    phase: Phase,
}
#[derive(Clone, Copy)]
enum Phase {
    Reading,
    Signing(Option<Token>),
    Again(Time),
}

enum Opening {
    Approval,
    Page { address: Box<[u8]> },
}

pub struct Start {
    pub size: Size,
    pub scheme: Option<tui::Scheme>,
    pub background: Option<Rgb>,
    pub address: Box<[u8]>,
    pub local: bool,
    pub boxed: bool,
    pub kept: Vec<jig_client::Record>,
    pub credential: Option<Secret>,
    pub note: Option<tui::Note>,
    /// Whether the TUI is in front of a box.
    pub front: bool,
}

pub enum Report {
    Start(Start),
    Due,
    Key(Key),
    Resize(Size),
    Heard(jig_client::Event),
    Kept {
        written: bool,
    },
    Opened {
        opened: bool,
    },
    Local {
        link: Option<Secret>,
    },
    /// A line of the box's, read.
    Front(temper_client_tui::BoxLine),
    Stage(tui::BoxStage),
}

/// The box's first stop closes its life pipe; the second sends SIGTERM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    First,
    Second,
}

#[derive(Default)]
pub struct Duties {
    pub asked: Vec<jig_client::Request>,
    pub keep: Option<Option<Secret>>,
    pub note: Option<tui::Note>,
    pub address: Option<Box<[u8]>>,
    pub local: bool,
    pub open: Option<Box<[u8]>>,
    pub close: bool,
    pub stop: Option<Stop>,
}

pub struct Settings {
    pub limits: Limits,
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "closing, browsing, address persistence and local sign-in are independent host facts"
)]
pub struct Host {
    #[cfg(test)]
    events: Vec<core::mem::Discriminant<Event>>,
    root: root::Domain,
    state: Option<tui::Tui>,
    view: tui::View,
    env: Env<root::Limits>,
    out: Queue<root::Request>,
    patches: Queue<Patch>,
    bounds: skein_ui::Limits,
    theme: Theme,
    size: Size,
    background: Option<Rgb>,
    terminal: Option<tui::Scheme>,
    choice: ThemeChoice,
    // Each write in asked order: true for the root, false for the host.
    root_writes: Queue<bool>,
    boxed: bool,
    focus: Focus,
    frame: Frame,
    draft_shown: bool,
    mapping: jig_client_protocol_http::Mapping,
    credential: Option<Secret>,
    signing: Set<Token>,
    local: bool,
    note: Option<Note>,
    need: Option<Need>,
    notice_end: Option<Time>,
    local_signing: bool,
    asked_opening: Option<u64>,
    browse: bool,
    opens: Queue<Opening>,
    addressed: bool,
    closed: bool,
    stopping: bool,
    handed: bool,
}

impl Host {
    #[must_use]
    pub fn new(settings: &Settings, seed: u64, now: Time, wall: Wall) -> Host {
        let limits = &settings.limits;
        assert!(limits::valid(limits), "TUI capacities are checked");
        let (_, words) =
            temper_client_shell_terminal::limits::view(&limits.native).expect("native view capacities are checked");
        let words = tui::words::Words::english(words, &limits.native.client).expect("TUI words are checked");
        Host {
            #[cfg(test)]
            events: Vec::new(),
            root: root::new(&limits.native.client, seed),
            state: None,
            view: tui::View::new(&limits.view, words),
            env: Env { now, wall, limits: limits.native.client },
            out: Queue::uncapped(),
            patches: Queue::with_capacity(limits.view.patches),
            bounds: limits.view,
            theme: tui::theme(tui::Scheme::Dark, None),
            size: Size { rows: 0, columns: 0 },
            background: None,
            terminal: None,
            choice: ThemeChoice::Auto,
            root_writes: Queue::uncapped(),
            boxed: false,
            focus: Focus::default(),
            frame: Frame::new(limits.most),
            draft_shown: false,
            mapping: jig_client_protocol_http::Mapping::new(b"/jig").expect("mapping prefix"),
            credential: None,
            signing: Set::with_capacity(temper_client_shell_terminal::limits::EXCHANGES),
            local: false,
            note: None,
            need: None,
            notice_end: None,
            local_signing: false,
            asked_opening: None,
            browse: false,
            opens: Queue::uncapped(),
            addressed: false,
            closed: false,
            stopping: false,
            handed: false,
        }
    }

    pub fn clocks(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
    }

    #[expect(clippy::too_many_lines, reason = "one report follows the host seam through its ordered duties")]
    pub fn report(&mut self, report: Report) -> Duties {
        if self.state.is_none() {
            assert!(matches!(report, Report::Start(_)), "a report before the TUI's start");
        }
        if self.closed && !matches!(report, Report::Heard(_) | Report::Kept { .. }) {
            return Duties::default();
        }
        let project = self.root.pages().project();
        let mut duties = Duties::default();
        match report {
            Report::Local { link } => self.found(link),
            Report::Front(line) => self.tell(Told::BoxLine { line }),
            Report::Stage(stage) => self.stage(stage, &mut duties),
            Report::Start(start) => self.start(start, &mut duties),
            Report::Due => {
                if self.notice_end.is_some_and(|end| end <= self.env.now) {
                    self.notice_end = None;
                    self.tell(Told::Notice { notice: None });
                }
                root::fire(&mut self.root, &self.env, &mut self.out);
                if let Some(need) = &mut self.need
                    && let Phase::Again(at) = need.phase
                    && at <= self.env.now
                {
                    need.reads = need.reads.checked_add(1).expect("bounded local reads");
                    need.phase = Phase::Reading;
                    duties.local = true;
                }
            }
            Report::Key(key) => {
                if key == Key::Control(b'p') {
                    if let Some(event) = tui::pane_key(&self.view) {
                        self.event(event, &mut duties);
                    }
                } else if key == Key::Char('n') && tui::walk_key(&self.view).is_some() {
                    self.tell(Told::Walk { by_key: true });
                } else if let Some(event) = self.arrow(&key) {
                    self.event(event, &mut duties);
                } else {
                    match command(&key, self.view.tree(), &mut self.focus, &self.bounds) {
                        Command::Act { node, act } => {
                            if let Some(event) = tui::event(&self.view, node, act) {
                                self.event(event, &mut duties);
                            }
                        }
                        Command::Close => self.quit(&mut duties),
                        Command::Nothing if key == Key::Escape => {
                            if let Some(event) = tui::escape(&self.view) {
                                self.event(event, &mut duties);
                            }
                        }
                        Command::Moved | Command::Nothing => {}
                    }
                }
            }
            Report::Resize(size) => {
                self.size = size;
                self.tell(Told::Size { columns: size.columns });
            }
            Report::Heard(event) => self.heard(event),
            Report::Kept { written } => self.tell(Told::Kept { written }),
            Report::Opened { opened } => match self.opens.pop() {
                Some(Opening::Page { address }) => self.tell(Told::NotOpened { address: (!opened).then_some(address) }),
                Some(Opening::Approval) | None => self.tell(Told::Opened { opened }),
            },
        }
        self.route(&mut duties);
        root::take(&mut self.root, &self.env, &mut self.out);
        self.route(&mut duties);
        self.launchpad(project);
        self.route(&mut duties);
        if self.state().start_chat().is_some() && self.root.pages().start().is_none() {
            if matches!(self.root.pages().needs().toast(), Some(needs::Toasted::Proposed { .. })) {
                self.notice(tui::Notice::Proposed);
            }
            self.tell(Told::Starting { chat: None });
        }
        if let Some(aside) = self.state().aside()
            && !self.aside_stands(aside)
        {
            self.tell(Told::Aside { object: None });
        }
        if !self.words().is_empty()
            && tui::reply_number(&self.view, self.words()).is_none()
            && let Some(event) = self.view.reply_aside()
        {
            self.event(event, &mut duties);
        }
        if self.need.as_ref().is_some_and(|need| matches!(need.phase, Phase::Signing(None))) {
            self.end_local();
        }
        let link = self.root.client().link();
        let state = link.state;
        let live = state == Link::Live && link.party.is_some();
        let credential = link.credential.cloned();
        let expires = link.expires;
        if credential.is_some() && state != Link::SignedOut {
            self.need = None;
        }
        self.tell(Told::Link { link: state });
        if credential != self.credential
            && (credential.is_some() || state == Link::SignedOut)
            && !self.need.as_ref().is_some_and(|need| need.held && state == Link::SignedOut)
        {
            let dropped = self.credential.is_some() && credential.is_none();
            duties.keep = Some(credential.clone());
            self.credential = credential;
            if dropped {
                self.ended(&mut duties);
            }
        }
        if live && !self.addressed {
            duties.address = Some(self.state().address().into());
            self.addressed = true;
        }
        if live
            && let Some(expires) = expires
            && self.note != Some(Note::Until(expires))
        {
            self.note = Some(Note::Until(expires));
            duties.note = self.note;
        }
        self.tell_outcome();
        self.approval(&mut duties);
        let scheme = match self.root.pages().look().theme {
            terminal::Theme::Dark => tui::Scheme::Dark,
            terminal::Theme::Light => tui::Scheme::Light,
        };
        self.theme = tui::theme(scheme, self.background);
        tui::build(
            &mut self.view,
            self.state.as_ref().expect("started TUI"),
            &self.root,
            self.env.wall,
            &mut self.patches,
        );
        self.patches.clear();
        duties
    }

    fn start(&mut self, start: Start, duties: &mut Duties) {
        assert!(self.state.is_none(), "the TUI starts once");
        self.state = Some(tui::Tui::new(&start.address));
        if start.front {
            self.tell(Told::Front);
            self.tell(Told::Stage { stage: tui::BoxStage::Finding });
            self.handed = true;
        }
        self.size = start.size;
        self.background = start.background;
        self.terminal = start.scheme;
        self.boxed = start.boxed;
        if self.boxed {
            self.tell(Told::Boxed);
        }
        self.tell(Told::Size { columns: start.size.columns });
        self.credential.clone_from(&start.credential);
        self.local = start.front || start.local;
        self.note = start.note;
        let mut kept = List::with_capacity(u32::try_from(start.kept.len()).expect("bounded store"));
        for record in start.kept {
            if record.key.as_ref() == THEME_KEY {
                self.choice = ThemeChoice::read(&record.bytes);
            } else {
                kept.push(record).expect("counted store");
            }
        }
        self.tell(Told::Theme { choice: self.choice, terminal: self.terminal });
        self.step(Event::Start {
            address: Address::Inbox { filter: Filter::ALL },
            kept,
            with: if start.front { None } else { start.credential.map(With::Held) },
        });
        self.step(Event::LookKept {
            look: Look {
                terminal: true,
                theme: match self.choice.scheme(self.terminal) {
                    tui::Scheme::Dark => terminal::Theme::Dark,
                    tui::Scheme::Light => terminal::Theme::Light,
                },
                font: Font::DejaVu,
                night: false,
            },
        });
        self.step(Event::Launchpad);
        if start.front {
            self.need_local(true, duties);
        } else if self.credential.is_none() {
            self.begin_local(duties);
        }
    }

    fn begin_local(&mut self, duties: &mut Duties) {
        if self.closed
            || !self.local
            || self.note == Some(Note::SignedOut)
            || self.root.client().link().state != Link::SignedOut
        {
            return;
        }
        self.need_local(false, duties);
    }

    fn need_local(&mut self, held: bool, duties: &mut Duties) {
        self.need = Some(Need { held, reads: 1, digest: None, phase: Phase::Reading });
        self.tell(Told::Local { local: Local::Signing });
        duties.local = true;
    }

    fn ended(&mut self, duties: &mut Duties) {
        if let Some(Note::Until(until)) = self.note
            && self.env.wall.as_nanos().saturating_add(EXPIRY_SLACK.as_nanos()) < until.as_nanos()
        {
            self.note = Some(Note::SignedOut);
            duties.note = self.note;
        } else {
            self.begin_local(duties);
        }
    }

    fn found(&mut self, bytes: Option<Secret>) {
        if !self.need.as_ref().is_some_and(|need| matches!(need.phase, Phase::Reading)) {
            return;
        }
        let Some(bytes) = bytes else {
            self.end_local();
            return;
        };
        let Ok(link) = jig_client_protocol_http::printed(&self.mapping, bytes.expose()) else {
            self.end_local();
            return;
        };
        if self.handed {
            if link.host.as_ref() != self.state().address() {
                self.note = None;
            }
            self.tell(Told::Address { address: link.host.clone() });
            self.handed = false;
        } else if link.host.as_ref() != self.state().address() {
            self.end_local();
            return;
        }
        let digest = skein_lib::sha256::digest(link.secret.expose());
        let need = self.need.as_mut().expect("local read's need");
        if need.digest == Some(digest) {
            self.again_local();
            return;
        }
        need.digest = Some(digest);
        need.phase = Phase::Signing(None);
        let held = if need.held { self.credential.clone() } else { None };
        self.local_signing = true;
        self.step(Event::Client {
            event: jig_client::Event::SignIn { with: With::Link { secret: link.secret, held } },
        });
    }

    fn again_local(&mut self) {
        let need = self.need.as_mut().expect("local answer's need");
        if need.reads < LOCAL_READS {
            need.phase = Phase::Again(self.env.now.saturating_add(LOCAL_AGAIN));
        } else {
            self.end_local();
        }
    }

    fn end_local(&mut self) {
        self.need = None;
        self.tell(Told::Local { local: Local::Ended });
    }

    fn state(&self) -> &tui::Tui {
        self.state.as_ref().expect("started TUI")
    }

    fn tell(&mut self, told: Told) {
        self.state.as_mut().expect("started TUI").step(told);
    }

    fn step(&mut self, event: Event) {
        #[cfg(test)]
        self.events.push(core::mem::discriminant(&event));
        root::step(&mut self.root, &self.env, event, &mut self.out);
    }

    fn aside_stands(&self, object: jig_client::contract::ObjectKey) -> bool {
        if self.state().walk().is_some() {
            shared::need_prompts(&self.root, self.view.words().view())
                .iter()
                .any(|prompt| prompt.object == object && matches!(prompt.act, shared::NeedAct::Answer(_)))
        } else {
            shared::question(&self.root, self.view.words().view()).is_some_and(|question| question.object == object)
        }
    }

    fn typed(&mut self, mut value: Box<[u8]>) {
        if self.state().aside().is_none() && self.state().walk().is_some() {
            if self.state().walk().is_some_and(|walk| walk.by_key() && !walk.taken()) {
                value = skein_lib::bytes::concat(&[b"n", &value]);
                self.tell(Told::Cleared);
            }
            if !value.is_empty() {
                self.tell(Told::EndWalk);
            }
        }
        if tui::sign_in_drawn(&self.root, self.state()) {
            self.tell(Told::Typed { value });
        } else if self.root.pages().chat().is_some() {
            self.act(Act::MessageWords { words: value });
        } else if self.root.pages().chats().is_some() {
            if !value.starts_with(b"/") {
                self.tell(Told::LeftOff { task: None });
            }
            self.act(Act::ChatWords { words: value });
        } else {
            self.tell(Told::CommandWords { value });
        }
    }

    fn event(&mut self, event: tui::Event, duties: &mut Duties) {
        match event {
            tui::Event::Own { command } => self.own(command, duties),
            tui::Event::Open { address } => self.open(address, duties),
            tui::Event::Walk { object, step } => {
                match step {
                    tui::WalkStep::Choose { words } => self.act(Act::Needs(needs::Act::Choose { object, words })),
                    tui::WalkStep::Open { address } => self.open(address, duties),
                    tui::WalkStep::Next => {}
                }
                self.tell(Told::Walked { object });
            }
            tui::Event::EndWalk => self.tell(Told::EndWalk),
            tui::Event::Typed { value } => {
                self.tell(Told::TalkList { open: false });
                if !value.is_empty() && let Some(tui::Event::Aside { object }) = tui::escape(&self.view) {
                    self.event(tui::Event::Aside { object }, duties);
                }
                self.tell(Told::ThemeList { open: false });
                self.typed(value);
            }
            tui::Event::Theme { choice } => {
                self.tell(Told::ThemeList { open: false });
                if let Some(choice) = choice {
                    self.keep_theme(choice, duties);
                    self.tell(Told::Cleared);
                    self.notice(tui::Notice::Theme { choice, terminal: self.terminal });
                }
            }
            tui::Event::LeftOff { task } => self.tell(Told::LeftOff { task }),
            tui::Event::SignIn if self.state().field().is_empty() && tui::may_ask_again(self.state(), &self.root) => {
                self.ask();
            }
            tui::Event::SignIn => match jig_client_protocol_http::printed(&self.mapping, self.state().field()) {
                Err(_) => self.tell(Told::Refused { refusal: Refusal::NotALink }),
                Ok(link) if link.host.as_ref() != self.state().address() => {
                    self.tell(Told::Refused { refusal: Refusal::OtherAddress { address: link.host } });
                }
                Ok(link) => {
                    self.local_signing = false;
                    self.step(Event::Client {
                        event: jig_client::Event::SignIn {
                            with: With::Link { secret: link.secret, held: self.credential.clone() },
                        },
                    });
                }
            },
            tui::Event::Command { command: TuiCommand::Exit } => {
                self.tell(Told::Cleared);
                self.quit(duties);
            }
            tui::Event::Command { command: TuiCommand::New } => {
                if let Some(project) = self.root.pages().project() {
                    self.went(Address::Home { project });
                }
                self.tell(Told::Cleared);
            }
            tui::Event::Command { command: TuiCommand::Goal } => {
                if tui::goal_offered(&self.root)
                    && let Some(chat) = self.root.pages().chat()
                    && let Some(document) = &chat.draft
                {
                    let task = chat.task;
                    let path = root::pages::chat::sheet::path(&root::pages::chat::document::slug(&document.title));
                    self.act(Act::OpenStart { draft: Some(path), chat: Some(task) });
                    if self.root.pages().start().is_some() {
                        self.tell(Told::Starting { chat: Some(task) });
                    }
                }
                self.act(Act::MessageWords { words: Box::default() });
                self.tell(Told::Cleared);
            }
            tui::Event::Way { template } => {
                self.act(Act::StartWay { template });
                self.act(Act::StartGoal);
            }
            tui::Event::Command { command } => {
                self.act(Act::Terminal(terminal::Act::Run { command }));
                self.tell(Told::Cleared);
            }
            tui::Event::Stop => self.stop(duties),
            tui::Event::Stay => self.tell(Told::Stay),
            tui::Event::Log => {
                if self.state().front() {
                    self.act(Act::Terminal(terminal::Act::Dismiss));
                    self.tell(Told::Cleared);
                    self.tell(Told::Log);
                }
            }
            tui::Event::Send => self.send(duties),
            tui::Event::Act { act } => self.act(act),
            tui::Event::Aside { object } => {
                if matches!(object, jig_client::contract::ObjectKey::Proposal(_)) && self.root.pages().chat().is_some_and(|chat| chat.talk().is_some()) {
                    self.tell(Told::ProposalAside { object });
                } else {
                    self.tell(Told::Aside { object: Some(object) });
                }
            }
            tui::Event::Accept { object, revision } => {
                self.act(Act::Decide { object, revision, act: jig_client::contract::ActKind::Accept });
                self.act(Act::Confirm);
            }
            tui::Event::Leave => self.leave(),
            tui::Event::Answer { words } => {
                self.act(Act::MessageWords { words });
                self.act(Act::Send);
                self.act(Act::MessageWords { words: Box::default() });
                self.tell(Told::Cleared);
            }
            tui::Event::ReplyAside { turn } => self.tell(Told::ReplyAside { turn }),
            tui::Event::Dismiss => self.dismiss(),
        }
    }

    fn dismiss(&mut self) {
        self.act(Act::Terminal(terminal::Act::Dismiss));
        self.tell(Told::LeftOff { task: None });
        self.tell(Told::Cleared);
    }

    fn own(&mut self, command: tui::Own, duties: &mut Duties) {
        match command {
            tui::Own::Draft | tui::Own::Change | tui::Own::Ask | tui::Own::Tell => {
                let words = skein_lib::bytes::concat(&[command.name(), b" "]);
                self.composer_words(words);
                self.tell(Told::Cleared);
            }
            tui::Own::Brief => {
                self.act(Act::Terminal(terminal::Act::Dismiss));
                self.tell(Told::Cleared);
                self.tell(Told::Brief);
            }
            tui::Own::Needs => {
                self.act(Act::Terminal(terminal::Act::Dismiss));
                self.tell(Told::Cleared);
                self.tell(Told::Walk { by_key: false });
            }
            tui::Own::Talk => {
                self.act(Act::Terminal(terminal::Act::Dismiss));
                self.tell(Told::Cleared);
                self.tell(Told::TalkList { open: true });
                self.focus.active = None;
            }
            tui::Own::Theme => {
                self.act(Act::Terminal(terminal::Act::Dismiss));
                self.tell(Told::Cleared);
                self.tell(Told::ThemeList { open: true });
                self.focus.active = None;
            }
            tui::Own::Open => {
                if let Some(address) = tui::focus(&self.view) {
                    let address = address.into();
                    self.act(Act::Terminal(terminal::Act::Dismiss));
                    self.tell(Told::Cleared);
                    self.open(address, duties);
                }
            }
        }
    }

    fn open(&mut self, address: Box<[u8]>, duties: &mut Duties) {
        self.opens.push(Opening::Page { address: address.clone() });
        duties.open = Some(address);
    }

    fn act(&mut self, act: Act) {
        if act == Act::StartCancel {
            self.tell(Told::Starting { chat: None });
        }
        self.step(Event::Act { action: root::Action::Pages(act) });
    }

    fn words(&self) -> &[u8] {
        if let Some(chat) = self.root.pages().chat() {
            &chat.words
        } else if let Some(chats) = self.root.pages().chats() {
            &chats.words
        } else {
            self.state().command_words()
        }
    }

    fn composer_words(&mut self, words: Box<[u8]>) {
        if self.root.pages().chat().is_some() {
            self.act(Act::MessageWords { words });
        } else {
            self.act(Act::ChatWords { words });
        }
    }

    fn tell_outcome(&mut self) {
        use root::pages::tell::State;
        let Some(tell) = self.root.pages().tell() else { return };
        if tell.serial <= self.state().tell_serial() {
            return;
        }
        let serial = tell.serial;
        let refusal = match &tell.state {
            State::Sending => return,
            State::Filed { number } => {
                let number = *number;
                let project = self
                    .root
                    .pages()
                    .me(self.root.client())
                    .and_then(|me| me.projects.iter().find(|row| row.project == tell.project))
                    .map(|row| row.name.bytes().into())
                    .unwrap_or_default();
                self.notice(tui::Notice::Filed { number, project });
                None
            }
            State::Refused(reason) => Some(shared::finding_refusal(self.view.words().view(), reason)),
            State::Lost => Some(self.view.words().tell_lost()),
            State::Long { over } => Some(self.view.words().tell_long(*over)),
        };
        if refusal.is_some() {
            let tell = self.root.pages().tell().expect("outcome kept");
            let words = skein_lib::bytes::concat(&[tui::Own::Tell.name(), b" ", &tell.words]);
            self.composer_words(words);
        }
        self.tell(Told::Tell { serial, refusal });
    }

    fn send(&mut self, duties: &mut Duties) {
        if self.words() == tui::EXIT && self.root.pages().chat().is_some_and(|chat| chat.talk().is_some()) {
            self.leave();
            return;
        }
        if self.state().front() && (self.words() == b"/quit" || self.words() == tui::EXIT) {
            self.tell(Told::Cleared);
            self.quit(duties);
            return;
        }
        if tui::landing(&self.root) {
            self.notice(tui::Notice::SigningIn);
            return;
        }
        self.tell(Told::NotOpened { address: None });
        self.tell(Told::Send);
        let project = self
            .root
            .pages()
            .chat()
            .and_then(|chat| chat.task(self.root.client()))
            .map(|task| task.project)
            .or(self.root.pages().project());
        if let Some(project) = project
            && let Some(words) = self.words().strip_prefix(b"/tell ").filter(|words| !words.is_empty())
        {
            let words = words.into();
            self.act(Act::Tell { project, words });
            self.composer_words(Box::default());
            self.tell(Told::Cleared);
            return;
        }
        let words = self.words();
        let command_words = words.starts_with(b"/")
            && core::str::from_utf8(words).is_ok_and(|words| !words.chars().any(char::is_whitespace));
        if self.root.pages().chat().is_none() && self.root.pages().project().is_some() && self.launchpad_words() {
            return;
        }
        let words = self.words();
        if command_words {
            let own = tui::offered_own(&self.root, self.state())
                .iter()
                .find(|command| command.name().starts_with(words))
                .copied();
            let root = if words == tui::EXIT {
                Some(TuiCommand::Exit)
            } else {
                tui::offered(&self.root, self.view.words())
                    .iter()
                    .find(|command| tui::command_name(**command).starts_with(words))
                    .copied()
            };
            if let Some(command @ (tui::Own::Draft | tui::Own::Change | tui::Own::Talk | tui::Own::Ask)) = own {
                self.event(tui::Event::Own { command }, duties);
            } else if let Some(command) = root {
                self.event(tui::Event::Command { command }, duties);
            } else if let Some(command) = own {
                self.event(tui::Event::Own { command }, duties);
            } else if self.state().front() && tui::LOG_COMMAND.starts_with(words) {
                self.event(tui::Event::Log, duties);
            } else {
                self.unknown();
            }
            return;
        }
        if let Some(object) = self.state().aside().filter(|_| self.state().walk().is_some()) {
            let words = words.into();
            self.act(Act::Needs(needs::Act::OpenReply { object }));
            self.act(Act::CardWords { object, words });
            self.act(Act::Needs(needs::Act::SendReply { object }));
            self.act(Act::ChatWords { words: Box::default() });
            self.tell(Told::Walked { object });
            return;
        }
        if let Some(event) = tui::reply_number(&self.view, self.words()) {
            if matches!(&event, tui::Event::ReplyAside { .. }) {
                self.act(Act::MessageWords { words: Box::default() });
                self.tell(Told::Cleared);
            }
            self.event(event, duties);
            return;
        }
        if self.root.pages().chat().is_some()
            && let Some(question) = shared::question(&self.root, self.view.words().view())
        {
            let object = question.object;
            let words = self.words().into();
            self.act(Act::Needs(needs::Act::OpenReply { object }));
            self.act(Act::CardWords { object, words });
            self.act(Act::Needs(needs::Act::SendReply { object }));
            self.act(Act::MessageWords { words: Box::default() });
        } else if self.root.pages().project().is_none() {
            return;
        } else if self.root.pages().chat().is_some() {
            self.act(Act::Send);
        } else {
            self.act(Act::StartChat);
        }
        self.tell(Told::Cleared);
    }

    fn launchpad_words(&mut self) -> bool {
        let words = self.words();
        if let Some(words) = words.strip_prefix(b"/talk ").filter(|words| !words.is_empty()) {
            let words: Box<[u8]> = words.into();
            let project = self.root.pages().project().expect("launchpad project");
            let team = root::pages::team::shown_at(&self.root, project, self.env.wall);
            if let Some(role) = team.facts().roles.iter().find(|role| role.name.get(..words.len()).is_some_and(|prefix| prefix.eq_ignore_ascii_case(&words))) {
                self.act(Act::Go { address: Address::Talk { project, task: role.task } });
                self.tell(Told::Cleared);
            } else {
                let project = self.root.pages().me(self.root.client()).and_then(|me| me.projects.iter().find(|row| row.project == project)).map(|row| row.name.bytes().into()).unwrap_or_default();
                self.notice(tui::Notice::NoRole { project, words });
            }
        } else if let Some(words) = words.strip_prefix(b"/draft ").filter(|words| !words.is_empty())
            && self.root.pages().project().is_some_and(|project| root::pages::drafts_offered(&self.root, project))
        {
            let words = words.into();
            self.went(Address::NewChat { draft: true });
            self.act(Act::ChatWords { words });
            self.act(Act::StartChat);
            self.tell(Told::Cleared);
        } else if let Some(words) = words.strip_prefix(b"/change ").filter(|words| !words.is_empty()) {
            let words = words.into();
            self.went(Address::NewChat { draft: false });
            self.act(Act::ChatWords { words });
            self.act(Act::StartChat);
            self.tell(Told::Cleared);
        } else if let Some(words) = words.strip_prefix(b"/ask ").filter(|words| !words.is_empty()) {
            let words = words.into();
            if let Some(project) = self.root.pages().project()
                && self.root.pages().address() != &(Address::Home { project })
            {
                self.went(Address::Home { project });
            }
            self.act(Act::ChatWords { words });
            self.act(Act::StartChat);
            self.tell(Told::Cleared);
        } else if let Some(number) = words.strip_prefix(b"/chats ") {
            let task = core::str::from_utf8(number)
                .ok()
                .and_then(|number| number.parse::<u32>().ok())
                .and_then(|number| tui::chat_number(&self.view, number));
            if let Some(task) = task {
                self.act(Act::Go { address: Address::Chat { task } });
                self.tell(Told::Cleared);
            } else {
                self.unknown();
            }
        } else {
            return false;
        }
        true
    }

    fn notice(&mut self, notice: tui::Notice) {
        self.tell(Told::Notice { notice: Some(notice) });
        self.notice_end = Some(self.env.now.saturating_add(tui::limits::NOTICE));
    }

    fn unknown(&mut self) {
        let words = self.words().into();
        self.act(Act::Terminal(terminal::Act::Dismiss));
        self.notice(tui::Notice::Unknown { words });
        self.tell(Told::Cleared);
    }

    fn leave(&mut self) {
        let Some(talk) = self.root.pages().chat().and_then(|chat| chat.talk()) else { return };
        let project = talk.project;
        if let Some(open) = talk.open() {
            let name = talk.name().unwrap_or(b"").into();
            let minutes = self.env.wall.as_nanos().saturating_sub(open.opened.as_nanos()).checked_div(60_000_000_000).expect("minute is nonzero").max(1);
            let cost = talk.cost();
            self.notice(tui::Notice::Left { name, minutes, cost });
        }
        self.went(Address::Home { project });
    }
    fn went(&mut self, address: Address) {
        self.step(Event::Went { place: Place::Address(address) });
        self.tell(Told::Went);
    }

    fn launchpad(&mut self, project: Option<u32>) {
        if self.root.client().link().state == Link::Live
            && let Some(project) = project.or(self.root.pages().project())
            && let Some(me) = self.root.pages().me(self.root.client())
            && !me.projects.iter().any(|row| row.project == project)
        {
            self.went(Address::Inbox { filter: Filter::ALL });
            return;
        }
        if self.root.pages().address() == &(Address::Inbox { filter: Filter::ALL })
            && self.root.client().link().state == Link::Live
            && let Some(me) = self.root.pages().me(self.root.client())
            && let Some(project) = me.projects.get(0)
        {
            let words: Box<[u8]> = self.state().command_words().into();
            self.went(Address::Home { project: project.project });
            if !words.is_empty() {
                self.act(Act::ChatWords { words });
            }
        }
    }

    fn show(&mut self, address: Address) {
        if matches!(&address, Address::Task { .. })
            && let Some(task) = self.state().start_chat()
        {
            self.tell(Told::Starting { chat: None });
            self.act(Act::Go { address: Address::Chat { task } });
            return;
        }
        let project = match &address {
            Address::AllHome | Address::Home { .. } | Address::Chat { .. } | Address::Talk { .. } => {
                self.went(address);
                return;
            }
            Address::Inbox { filter } if *filter == Filter::ALL => {
                self.went(address);
                return;
            }
            Address::Work { project, .. }
            | Address::Goals { project, .. }
            | Address::Changes { project }
            | Address::Settings { project }
            | Address::Repositories { project }
            | Address::Models { project }
            | Address::Builders { project }
            | Address::Team { project, .. }
            | Address::Role { project, .. }
            | Address::Rule { project, .. }
            | Address::Ask { project }
            | Address::Adopt { project } => Some(*project),
            Address::Inbox { filter } => filter.project,
            Address::Chats { .. }
            | Address::NewChat { .. }
            | Address::Told
            | Address::Remembered
            | Address::Members
            | Address::Limits
            | Address::Accounts
            | Address::Forges
            | Address::ConnectForge { .. }
            | Address::ConnectModel { .. }
            | Address::Welcome { .. }
            | Address::Task { .. } => None,
        };
        if let Some(project) = project.or(self.root.pages().project()) {
            self.went(Address::Home { project });
        }
    }

    fn ask(&mut self) {
        self.step(Event::AskApproval);
        self.asked_opening = Some(self.state().openings());
        self.browse = true;
    }

    fn approval(&mut self, duties: &mut Duties) {
        if self.closed || self.need.is_some() {
            return;
        }
        if self.root.client().link().state == Link::SignedOut && self.asked_opening != Some(self.state().openings()) {
            self.ask();
            self.route(duties);
        }
        if self.browse {
            match self.root.asked() {
                Some(root::asked::State::Asked { code, .. }) => {
                    duties.open = Some(tui::approval_address(self.state().address(), code));
                    self.opens.push(Opening::Approval);
                    self.tell(Told::Browse);
                    self.browse = false;
                }
                Some(root::asked::State::NotAsked(_)) => self.browse = false,
                None | Some(root::asked::State::Asking) => {}
            }
        }
    }

    fn heard(&mut self, event: jig_client::Event) {
        if matches!(&event, jig_client::Event::Stored | jig_client::Event::NotStored)
            && self.root_writes.pop() != Some(true)
        {
            return;
        }
        let local = match &event {
            jig_client::Event::Reply { exchange, .. } | jig_client::Event::Lost { exchange } => self
                .need
                .as_ref()
                .is_some_and(|need| matches!(need.phase, Phase::Signing(Some(token)) if token == *exchange)),
            jig_client::Event::Start { .. }
            | jig_client::Event::StartUnlinked { .. }
            | jig_client::Event::SignIn { .. }
            | jig_client::Event::SignOut
            | jig_client::Event::Authorize { .. }
            | jig_client::Event::AskApproval
            | jig_client::Event::ReadApproval { .. }
            | jig_client::Event::DecideApproval { .. }
            | jig_client::Event::ApprovalAnswer { .. }
            | jig_client::Event::SignOutLost
            | jig_client::Event::Watch { .. }
            | jig_client::Event::Unwatch { .. }
            | jig_client::Event::Read { .. }
            | jig_client::Event::Send { .. }
            | jig_client::Event::Carry { .. }
            | jig_client::Event::Up { .. }
            | jig_client::Event::Stored
            | jig_client::Event::NotStored
            | jig_client::Event::Streamed { .. }
            | jig_client::Event::StreamGone => false,
        };
        let answered = if let jig_client::Event::Reply { exchange, .. } = &event {
            self.signing.remove(exchange).then_some(false)
        } else if let jig_client::Event::Lost { exchange } = &event {
            self.signing.remove(exchange).then_some(true)
        } else {
            None
        };
        self.step(Event::Client { event });
        if local && self.root.client().link().state == Link::SignedOut {
            match self.root.client().link().not_signed_in {
                Some(jig_client::NotSignedIn::LinkUsed | jig_client::NotSignedIn::LinkExpired) => self.again_local(),
                Some(jig_client::NotSignedIn::NotReady | jig_client::NotSignedIn::Busy) if self.state().front() => {}
                None
                | Some(
                    jig_client::NotSignedIn::Approval(_)
                    | jig_client::NotSignedIn::LinkUnknown
                    | jig_client::NotSignedIn::NotReady
                    | jig_client::NotSignedIn::Busy
                    | jig_client::NotSignedIn::Full { .. },
                ) => self.end_local(),
            }
        }
        if let Some(lost) = answered {
            self.tell(Told::Answered { lost });
        }
    }

    fn route(&mut self, duties: &mut Duties) {
        while let Some(request) = self.out.pop() {
            match route::down(request) {
                Down::Shell(jig_client::Request::ServiceCredential { .. })
                | Down::Duty(Duty::SignOut | Duty::Fact(_))
                | Down::Notice(_)
                | Down::Unlinked(_) => {}
                Down::Shell(request) => {
                    if let jig_client::Request::Exchange { exchange, message: Message::SignInLink(_), .. } = &request {
                        if self.local_signing {
                            if let Some(need) = &mut self.need
                                && let Phase::Signing(token) = &mut need.phase
                            {
                                *token = Some(*exchange);
                            }
                        } else {
                            self.signing.insert(*exchange).expect("bounded sign-in exchanges");
                            self.tell(Told::Asked);
                        }
                    }
                    if matches!(&request, jig_client::Request::Exchange { message: Message::SignInApproval(_), .. }) {
                        self.local_signing = false;
                    }
                    if matches!(&request, jig_client::Request::Save { .. } | jig_client::Request::Erase { .. }) {
                        self.root_writes.push(true);
                    }
                    duties.asked.push(request);
                }
                Down::Duty(Duty::Copy { token }) => {
                    self.step(Event::Copied { token, written: root::copy::Written::Refused });
                }
                Down::Duty(Duty::Show { address, .. }) => self.show(address),
            }
        }
    }

    fn keep_theme(&mut self, choice: ThemeChoice, duties: &mut Duties) {
        self.choice = choice;
        self.tell(Told::Theme { choice, terminal: self.terminal });
        let theme = match choice.scheme(self.terminal) {
            tui::Scheme::Dark => terminal::Theme::Dark,
            tui::Scheme::Light => terminal::Theme::Light,
        };
        self.step(Event::LookKept { look: Look { theme, ..self.root.pages().look() } });
        self.theme = tui::theme(choice.scheme(self.terminal), self.background);
        self.route(duties);
        let mut records = List::with_capacity(1);
        records
            .push(jig_client::Record { key: THEME_KEY.into(), bytes: choice.print().into() })
            .expect("one theme record");
        duties.asked.push(jig_client::Request::Save { records });
        self.root_writes.push(false);
    }

    fn stage(&mut self, stage: tui::BoxStage, duties: &mut Duties) {
        let ended = matches!(stage, tui::BoxStage::Ended { .. });
        if !self.stopping || matches!(stage, tui::BoxStage::Ended { .. } | tui::BoxStage::Failed { .. }) {
            self.tell(Told::Stage { stage });
        }
        if self.state().box_stage() == &tui::BoxStage::Stopping {
            self.stopping = true;
            self.need = None;
            self.tell(Told::Stay);
        }
        if ended && self.stopping {
            self.close(duties);
        }
    }

    fn stop(&mut self, duties: &mut Duties) {
        if !self.state().front() || matches!(self.state().box_stage(), tui::BoxStage::Ended { .. }) {
            self.close(duties);
        } else if self.stopping {
            duties.stop = Some(Stop::Second);
        } else {
            self.stopping = true;
            self.need = None;
            self.tell(Told::Stay);
            self.tell(Told::Stage { stage: tui::BoxStage::Stopping });
            duties.stop = Some(Stop::First);
        }
    }

    fn quit(&mut self, duties: &mut Duties) {
        if !self.state().front() || matches!(self.state().box_stage(), tui::BoxStage::Ended { .. }) {
            self.close(duties);
        } else if self.stopping
            || self.state().quitting()
            || !matches!(self.state().box_stage(), tui::BoxStage::Running)
            || matches!(self.state().box_state(), tui::BoxState::GaveUp { .. })
            || !tui::work_runs(&self.root, self.state())
        {
            self.stop(duties);
        } else {
            self.tell(Told::Quitting);
        }
    }

    fn close(&mut self, duties: &mut Duties) {
        self.closed = true;
        duties.close = true;
    }

    pub fn frame(&mut self) -> &Frame {
        if self.state.is_some() {
            let draft = self.view.pane_mode() == Some(root::pages::chat::document::Mode::Draft);
            if draft != self.draft_shown {
                self.focus.follow = !draft;
                if draft {
                    self.focus.scroll = 0;
                }
                self.draft_shown = draft;
            }
            skein_ui_screen::render(self.view.tree(), &self.theme, self.size, &mut self.focus, &mut self.frame);
        } else {
            self.frame.clear(self.size, Cell::BLANK);
        }
        &self.frame
    }

    #[must_use]
    pub fn tree(&self) -> &skein_ui::Tree {
        self.view.tree()
    }

    #[must_use]
    pub fn event_of(&self, key: &Key) -> Option<tui::Event> {
        if *key == Key::Control(b'p') {
            return tui::pane_key(&self.view);
        }
        if *key == Key::Char('n')
            && let Some(event) = tui::walk_key(&self.view)
        {
            return Some(event);
        }
        if let Some(event) = self.arrow(key) {
            return Some(event);
        }
        let mut focus = self.focus;
        match command(key, self.view.tree(), &mut focus, &self.bounds) {
            Command::Act { node, act } => tui::event(&self.view, node, act),
            Command::Nothing if *key == Key::Escape => tui::escape(&self.view),
            Command::Moved | Command::Close | Command::Nothing => None,
        }
    }

    fn arrow(&self, key: &Key) -> Option<tui::Event> {
        if *key == Key::Up {
            tui::arrow(&self.view, tui::Arrow::Up)
        } else if *key == Key::Down {
            tui::arrow(&self.view, tui::Arrow::Down)
        } else {
            None
        }
    }

    #[must_use]
    pub fn deadline(&self) -> Option<Time> {
        if self.closed {
            return None;
        }
        let tick = self.view.tick().map(|tick| {
            let span = Duration::from_nanos(tick.as_nanos().saturating_sub(self.env.wall.as_nanos()));
            self.env.now.saturating_add(span)
        });
        let local = self.need.as_ref().and_then(|need| match need.phase {
            Phase::Again(at) => Some(at),
            Phase::Reading | Phase::Signing(_) => None,
        });
        skein_lib::earliest(
            skein_lib::earliest(skein_lib::earliest(root::next_deadline(&self.root), tick), local),
            self.notice_end,
        )
    }
}

#[cfg(test)]
#[path = "tests.rs"]
pub(crate) mod tests;
