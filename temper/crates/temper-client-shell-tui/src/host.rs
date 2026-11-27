//! Reports in and duties out (temper/docs/design/client/tui/host.md, 1).
use crate::limits::{self, Limits};
use jig_client::{Link, Message, With, contract::LoginExit};
use skein_lib::{Duration, Env, List, Queue, Secret, Set, Time, Token, Wall};
use skein_ui::Patch;
use skein_ui_screen::{Cell, Focus, Frame, Rgb, Size, Theme};
use skein_ui_screen_terminal::{Command, Key, alias, command};
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
use temper_client_tui::{self as tui, BoxKey, Local, Note, Refusal, Told};
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
    Approval { number: u64 },
    Link,
    Page { address: Box<[u8]> },
}

enum Listener {
    None,
    Opening { asked: Option<root::asked::State> },
    Open { port: u16, asking: bool },
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "terminal capability, local link, box placement, front and prior setup are independent start facts"
)]
pub struct Start {
    pub size: Size,
    pub scheme: Option<tui::Scheme>,
    pub background: Option<Rgb>,
    pub plain: bool,
    pub version: Box<[u8]>,
    pub address: tui::Origin,
    pub local: bool,
    pub boxed: bool,
    pub kept: Vec<jig_client::Record>,
    pub credential: Option<Secret>,
    pub note: Option<tui::Note>,
    /// Whether the TUI is in front of a box.
    pub front: bool,
    pub set_up: bool,
    pub box_log: Box<[u8]>,
}

/// The startup files that belong to one engine, not to the front's preview.
#[derive(Clone)]
pub(crate) struct OriginFiles {
    pub address: tui::Origin,
    pub kept: Vec<jig_client::Record>,
    pub credential: Option<Secret>,
    pub note: Option<tui::Note>,
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
    Handshake {
        failed: Option<tui::Handshake>,
    },
    Listening {
        port: Option<u16>,
    },
    Returned(crate::loopback::Returned),
    /// A line of the box's, read.
    Front(temper_client_tui::BoxLine),
    Stage(tui::BoxStage),
    Lent {
        number: u32,
        exit: LoginExit,
    },
}

/// The box's first stop closes its life pipe; the second sends SIGTERM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    First,
    Second,
}

/// A login taking the terminal (client/tui/box.md, Lending its terminal).
#[derive(Debug, PartialEq, Eq)]
pub struct Lend {
    pub number: u32,
    pub name: Box<[u8]>,
    pub line: Box<[u8]>,
}

#[expect(clippy::struct_excessive_bools, reason = "independent duties of the host seam")]
#[derive(Default)]
pub struct Duties {
    pub asked: Vec<jig_client::Request>,
    pub keep: Option<Option<Secret>>,
    pub note: Option<tui::Note>,
    pub address: Option<tui::Origin>,
    pub local: bool,
    pub open: Option<Box<[u8]>>,
    pub listen: bool,
    pub unlisten: bool,
    pub answer: Option<crate::loopback::Answer>,
    pub close: bool,
    pub stop: Option<Stop>,
    pub lend: Option<Lend>,
    pub unlend: bool,
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
    approval_number: u64,
    listener: Listener,
    opens: Queue<Opening>,
    addressed: bool,
    closed: bool,
    stopping: bool,
    handed: bool,
    front_files: Option<OriginFiles>,
    front_theme: Option<ThemeChoice>,
    terminal_number: u64,
}

impl Host {
    #[must_use]
    pub fn new(settings: &Settings, seed: u64, now: Time, wall: Wall) -> Host {
        let words = limits::checked(&settings.limits).expect("TUI capacities are checked");
        Self::with_checked_words(settings, words, seed, now, wall)
    }

    /// The words have passed the native and TUI capacity checks with these settings.
    pub(crate) fn with_checked_words(
        settings: &Settings,
        words: tui::words::Words,
        seed: u64,
        now: Time,
        wall: Wall,
    ) -> Host {
        let limits = &settings.limits;
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
            mapping: jig_client_protocol_http::Mapping::with_page(b"/jig", temper_client_domain::sign_in::LINK_PAGE)
                .expect("temper mapping"),
            credential: None,
            signing: Set::with_capacity(temper_client_shell_terminal::limits::EXCHANGES),
            local: false,
            note: None,
            need: None,
            notice_end: None,
            local_signing: false,
            asked_opening: None,
            browse: false,
            approval_number: 0,
            listener: Listener::None,
            opens: Queue::uncapped(),
            addressed: false,
            closed: false,
            stopping: false,
            handed: false,
            front_files: None,
            front_theme: None,
            terminal_number: skein_lib::Rng::new(seed).next_u64(),
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
        if self.closed && !matches!(report, Report::Heard(_) | Report::Kept { .. } | Report::Lent { .. }) {
            return Duties {
                answer: matches!(report, Report::Returned(_)).then_some(crate::loopback::Answer::NotFound),
                unlisten: matches!(report, Report::Listening { port: Some(_) }),
                ..Duties::default()
            };
        }
        let mut duties = Duties::default();
        if self.refused() {
            match report {
                Report::Returned(_) => duties.answer = Some(crate::loopback::Answer::NotFound),
                Report::Listening { port } => duties.unlisten = port.is_some(),
                Report::Key(Key::Control(b'c')) => self.close(&mut duties),
                Report::Resize(size) => self.size = size,
                Report::Start(_)
                | Report::Due
                | Report::Key(_)
                | Report::Heard(_)
                | Report::Kept { .. }
                | Report::Opened { .. }
                | Report::Handshake { .. }
                | Report::Local { .. }
                | Report::Front(_)
                | Report::Stage(_)
                | Report::Lent { .. } => {}
            }
            self.draw();
            return duties;
        }
        let project = self.root.pages().project();
        match report {
            Report::Listening { port } => self.listening(port, &mut duties),
            Report::Returned(returned) => duties.answer = Some(self.returned(&returned)),
            Report::Lent { number, exit } => self.lent(number, exit),
            Report::Local { link } => self.found(link),
            Report::Handshake { failed } => self.tell(Told::Handshake { failed }),
            Report::Front(line) => {
                if let tui::BoxLine::Started { name } = &line {
                    self.notice(tui::Notice::Started { name: name.clone() });
                }
                self.tell(Told::BoxLine { line });
            }
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
                if key == Key::Control(b'o') {
                    if let Some(event) = tui::pane_key(&self.view) {
                        self.event(event, &mut duties);
                    }
                } else if let Some(event) = self.arrow(&key) {
                    self.event(event, &mut duties);
                } else if let Some(event) = self.back(&key) {
                    self.event(event, &mut duties);
                } else {
                    match list_command(&self.view, &key, &mut self.focus, &self.bounds) {
                        Command::Act { node, act } => {
                            if let Some(event) = tui::event(&self.view, node, act) {
                                self.event(event, &mut duties);
                            }
                        }
                        Command::Close => self.quit(&mut duties),
                        Command::Nothing => match self.key_without_act(&key) {
                            Some(tui::Event::Command { command: TuiCommand::Pause }) => {
                                // Esc stops the turn without submitting or clearing the box.
                                self.act(Act::Stop);
                            }
                            Some(event) => self.event(event, &mut duties),
                            None => {}
                        },
                        Command::Moved => {}
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
                Some(Opening::Approval { number }) => {
                    if number == self.approval_number {
                        self.tell(Told::Opened { opened });
                    }
                }
                None => self.tell(Told::Opened { opened }),
                Some(Opening::Link) => {
                    if !opened {
                        self.tell(Told::LinkNotOpened);
                    }
                }
            },
        }
        if self.refused() {
            self.out.clear();
            let mut duties = Duties { answer: duties.answer, unlisten: duties.unlisten, ..Duties::default() };
            self.unlisten(&mut duties);
            self.draw();
            return duties;
        }
        self.route(&mut duties);
        root::take(&mut self.root, &self.env, &mut self.out);
        self.route(&mut duties);
        if self.state().front() && self.root.pages().me(self.root.client()).is_some() {
            self.tell(Told::MeRead);
        }
        self.closed_chat();
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
            duties.address = Some(self.state().address().clone());
            self.addressed = true;
        }
        if live
            && let Some(expires) = expires
            && self.note != Some(Note::Until(expires))
        {
            self.note = Some(Note::Until(expires));
            duties.note = self.note;
        }
        self.lending(&mut duties);
        self.tell_outcome();
        self.approval(&mut duties);
        root::settle(&mut self.root, &self.env, &mut self.out);
        self.route(&mut duties);
        if self.need.as_ref().is_some_and(|need| matches!(need.phase, Phase::Signing(None))) {
            self.end_local();
        }

        let linked = self.root.forge().linked().unwrap_or(false);
        if self.state().last_linked() != Some(linked) {
            self.tell(Told::Linked { linked });
        }
        self.draw();
        duties
    }

    fn refused(&self) -> bool {
        self.state.as_ref().is_some_and(|state| tui::refused(&self.root, state))
    }

    fn draw(&mut self) {
        let scheme = match self.root.pages().look().theme {
            terminal::Theme::Dark => tui::Scheme::Dark,
            terminal::Theme::Light => tui::Scheme::Light,
        };
        self.theme = tui::theme(scheme, self.background);
        self.build_view();
        if self.state().picked() != self.view.picked() {
            match self.view.picked() {
                Some((row, in_list)) => self.tell(Told::Picked { row: Some(row), in_list }),
                None => self.clear_pick(),
            }
            self.build_view();
        }
        if self.view.list_focused() {
            self.focus.node = None;
        }
    }

    fn build_view(&mut self) {
        tui::build(
            &mut self.view,
            self.state.as_ref().expect("started TUI"),
            &self.root,
            self.env.wall,
            &mut self.patches,
        );
        self.patches.clear();
    }

    fn start(&mut self, start: Start, duties: &mut Duties) {
        assert!(self.state.is_none(), "the TUI starts once");
        // The front can draw and take terminal keys before it knows its engine.
        let preview =
            if start.front { crate::start::selected_address(None, None, None) } else { start.address.clone() };
        let mut state = tui::Tui::new(&preview);
        state.header(start.plain, start.version);
        self.state = Some(state);
        if start.front {
            self.tell(Told::Front { set_up: start.set_up, box_log: start.box_log });
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
        self.local = start.front || start.local;
        let files =
            OriginFiles { address: start.address, kept: start.kept, credential: start.credential, note: start.note };
        if start.front {
            self.front_files = Some(files);
            self.look();
            self.step(Event::Launchpad);
            self.step(Event::Lends { terminal: self.terminal_number });
            self.need_local(true, duties);
        } else {
            self.start_client(files, false);
            if self.credential.is_none() {
                self.begin_local(duties);
            }
        }
    }

    pub(crate) fn front_files(&mut self, files: OriginFiles) {
        assert!(self.handed, "files are supplied before the handed link");
        self.front_files = Some(files);
    }

    fn start_client(&mut self, files: OriginFiles, front: bool) {
        self.credential.clone_from(&files.credential);
        self.note = files.note;
        let mut kept = List::with_capacity(u32::try_from(files.kept.len()).expect("bounded store"));
        for record in files.kept {
            if record.key.as_ref() == THEME_KEY {
                self.choice = ThemeChoice::read(&record.bytes);
            } else {
                kept.push(record).expect("counted store");
            }
        }
        if let Some(choice) = self.front_theme.take() {
            self.choice = choice;
        }
        self.step(Event::Start {
            address: Address::Inbox { filter: Filter::ALL },
            kept,
            with: if front { None } else { files.credential.map(With::Held) },
        });
        self.look();
        self.step(Event::Launchpad);
    }

    fn look(&mut self) {
        self.tell(Told::Theme { choice: self.choice, terminal: self.terminal });
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
    }

    fn lending(&mut self, duties: &mut Duties) {
        if !self.state().front() || self.closed {
            return;
        }
        let Some(login) = self.root.terminal_login() else { return };
        if let Some(lending) = self.state().lending() {
            if login.number == lending.number && login.cancelled && !lending.unlend {
                duties.unlend = true;
                self.tell(Told::Unlend);
            }
        } else if !login.cancelled && !self.state().login_taken(login.number) {
            let number = login.number;
            let name = login.name.clone();
            duties.lend =
                Some(Lend { number, name: name.clone(), line: self.view.words().say(tui::words::Word::LendLine, &[]) });
            self.tell(Told::Lending { number, name });
        }
    }

    fn lent(&mut self, number: u32, exit: LoginExit) {
        let Some(lending) = self.state().lending() else { return };
        if lending.number != number {
            return;
        }
        let notice = if lending.unlend {
            tui::Notice::LoginCancelled
        } else if exit == (LoginExit::Exited { code: 0 }) {
            tui::Notice::LoginSignedIn
        } else {
            tui::Notice::LoginNotFinished
        };
        self.step(Event::Act { action: root::Action::LoginEnded { number, exit } });
        self.tell(Told::Lent);
        self.notice(notice);
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
            let Ok(address) = tui::Origin::parse(&skein_lib::bytes::concat(&[b"http://", &link.host])) else {
                self.end_local();
                return;
            };
            let files = self.front_files.take().filter(|files| files.address == address).unwrap_or_else(|| {
                OriginFiles { address: address.clone(), kept: Vec::new(), credential: None, note: None }
            });
            self.tell(Told::Address { address });
            self.handed = false;
            self.start_client(files, true);
        } else if !same_origin(self.state().address(), &link.host) {
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

    fn clear_pick(&mut self) {
        self.tell(Told::Picked { row: None, in_list: false });
    }

    fn typed(&mut self, value: Box<[u8]>) {
        if self.state().aside().is_none() && self.state().walk().is_some() && !value.is_empty() {
            self.tell(Told::EndWalk);
        }
        if let Some((row, _)) = self.state().picked() {
            self.tell(Told::Picked { row: value.starts_with(b"/").then_some(row), in_list: false });
        }
        if tui::sign_in_drawn(&self.root, self.state()) {
            if !self.state().address().at_name() {
                self.tell(Told::Typed { value });
            }
        } else if self.root.pages().chat().is_some() {
            self.act(Act::MessageWords { words: value });
        } else if self.root.pages().chats().is_some() {
            self.act(Act::ChatWords { words: value });
        } else {
            self.tell(Told::CommandWords { value });
        }
    }

    #[expect(clippy::too_many_lines, reason = "one exhaustive dispatch binds each TUI event to its host acts")]
    fn event(&mut self, event: tui::Event, duties: &mut Duties) {
        if self.refused() {
            return;
        }
        match event {
            tui::Event::Own { command } => self.own(command, duties),
            tui::Event::Open { address } => {
                self.clear_pick();
                self.open(address, duties);
            }
            tui::Event::WalkFrom { object } => {
                self.clear_pick();
                self.walk(Some(object));
            }
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
                if !value.is_empty()
                    && let Some(tui::Event::Aside { object }) = tui::escape(&self.view)
                {
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
            tui::Event::Picked { row } => self.tell(Told::Picked { row, in_list: row.is_some() }),
            tui::Event::TypeCode => {
                if tui::may_type_instead(self.state(), &self.root) {
                    self.unlisten(duties);
                    self.next_approval();
                    self.step(Event::AskApproval { port: None });
                    self.tell(Told::Opened { opened: false });
                }
            }
            tui::Event::SignIn if self.state().address().at_name() => {
                if tui::may_ask_again(self.state(), &self.root) {
                    self.ask(duties);
                }
            }
            tui::Event::SignIn if self.state().field().is_empty() && tui::may_ask_again(self.state(), &self.root) => {
                self.ask(duties);
            }
            tui::Event::SignIn => match jig_client_protocol_http::printed(&self.mapping, self.state().field()) {
                Err(_) => self.tell(Told::Refused { refusal: Refusal::NotALink }),
                Ok(link) if !same_origin(self.state().address(), &link.host) => {
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
                self.pressed(TuiCommand::New.name());
                if let Some(project) = self.root.pages().project() {
                    self.went(Address::Home { project });
                } else {
                    self.tell(Told::Cleared);
                }
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
            tui::Event::Act { act: Act::Confirm } => self.confirm(),
            tui::Event::Act { act } => {
                if let Act::Go { .. } = act {
                    self.clear_pick();
                }
                self.act(act);
            }
            tui::Event::Aside { object } => {
                if matches!(object, jig_client::contract::ObjectKey::Proposal(_))
                    && self.root.pages().chat().is_some_and(|chat| chat.talk().is_some())
                {
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
            tui::Event::Back => {
                if let Address::NewChat { .. } = self.root.pages().address()
                    && let Some(project) = self.root.pages().project()
                {
                    self.went(Address::Home { project });
                } else if self.state().place().is_some() {
                    self.change_place(None);
                } else {
                    self.tell(Told::Cleared);
                }
            }
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
        self.clear_pick();
        self.tell(Told::Cleared);
    }

    fn own(&mut self, command: tui::Own, duties: &mut Duties) {
        self.clear_pick();
        match command {
            tui::Own::Draft | tui::Own::Change | tui::Own::Ask | tui::Own::Tell => {
                self.pressed(command.name());
                self.change_place(Some(command));
            }
            tui::Own::Brief => {
                self.act(Act::Terminal(terminal::Act::Dismiss));
                self.tell(Told::Cleared);
                self.tell(Told::Brief);
            }
            tui::Own::Needs => self.walk(None),
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
            tui::Own::Web => {
                self.act(Act::Terminal(terminal::Act::Dismiss));
                self.tell(Told::Cleared);
                if let Some(link) = self.state().live_link() {
                    duties.open = Some(link.expose().into());
                    self.opens.push(Opening::Link);
                } else {
                    let path = match self.root.pages().project() {
                        Some(project) => root::address::print(&Address::Home { project }),
                        None => Box::from(b"/".as_slice()),
                    };
                    let address = skein_lib::bytes::concat(&[&self.state().address().print(), &path]);
                    self.open(address, duties);
                }
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

    fn walk(&mut self, first: Option<jig_client::contract::ObjectKey>) {
        self.act(Act::Terminal(terminal::Act::Dismiss));
        self.tell(Told::Cleared);
        self.tell(Told::Walk { first });
        self.focus.active = None;
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
        } else if self.root.pages().chats().is_some() {
            self.act(Act::ChatWords { words });
        } else {
            self.tell(Told::CommandWords { value: words });
        }
    }

    fn pressed(&mut self, name: &[u8]) {
        if let Ok(words) = core::str::from_utf8(self.words()) {
            let words = words.trim_end().as_bytes();
            if words.starts_with(b"/") && name.starts_with(words) {
                self.composer_words(Box::default());
            }
        }
    }

    fn box_key(&self) -> Option<BoxKey> {
        if let Some(command) = self.state().place() {
            return Some(BoxKey::Place { command });
        }
        let pages = self.root.pages();
        match *pages.address() {
            Address::Chat { .. } => Some(BoxKey::Chat { task: pages.chat()?.task }),
            Address::Talk { .. } => Some(BoxKey::Talk { task: pages.chat()?.task }),
            Address::Home { .. } | Address::AllHome | Address::Inbox { .. } => Some(BoxKey::Launchpad),
            Address::Chats { .. }
            | Address::NewChat { .. }
            | Address::Accounts
            | Address::Forges
            | Address::ConnectForge { .. }
            | Address::ConnectModel { .. }
            | Address::Welcome { .. }
            | Address::Task { .. }
            | Address::Goals { .. }
            | Address::Changes { .. }
            | Address::Waits { .. }
            | Address::Settings { .. }
            | Address::Repositories { .. }
            | Address::Models { .. }
            | Address::Builders { .. }
            | Address::Role { .. }
            | Address::Rule { .. }
            | Address::Team { .. }
            | Address::Ask { .. }
            | Address::Tell { .. }
            | Address::Adopt { .. }
            | Address::Work { .. }
            | Address::Members
            | Address::Remembered
            | Address::Told
            | Address::Limits => None,
        }
    }

    fn leaving_box(&self) -> Option<(BoxKey, Box<[u8]>)> {
        self.box_key().map(|key| (key, self.words().into()))
    }

    fn enter_box(&mut self, left: Option<(BoxKey, Box<[u8]>)>) {
        let entering = self.box_key();
        let state = self.state.as_mut().expect("started TUI");
        // Take the returning box first, so it cannot be evicted by the one left.
        let mut words = entering.and_then(|key| state.take_words(key)).unwrap_or_default();
        if let Some((key, prior)) = left {
            if entering == Some(key) {
                // The launchpad can change its root address without changing boxes.
                words = prior;
            } else {
                state.keep_words(key, prior);
            }
        }
        if self.words() != words.as_ref() {
            self.composer_words(words);
        }
    }

    fn change_place(&mut self, command: Option<tui::Own>) {
        let left = self.leaving_box();
        self.tell(Told::Cleared);
        self.tell(Told::Place { command });
        self.enter_box(left);
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
                if self.state().place() == Some(tui::Own::Tell) {
                    self.composer_words(Box::default());
                    self.change_place(None);
                }
                self.notice(tui::Notice::Filed { number, project });
                None
            }
            State::Refused(reason) => Some(shared::finding_refusal(self.view.words().view(), reason)),
            State::Lost => Some(self.view.words().tell_lost()),
            State::Long { over } => Some(self.view.words().tell_long(*over)),
        };
        if refusal.is_some() {
            let tell = self.root.pages().tell().expect("outcome kept");
            let words = if self.state().place() == Some(tui::Own::Tell) {
                tell.words.clone()
            } else {
                skein_lib::bytes::concat(&[tui::Own::Tell.name(), b" ", &tell.words])
            };
            self.composer_words(words);
        }
        self.tell(Told::Tell { serial, refusal });
    }

    #[expect(clippy::too_many_lines, reason = "composer sends route terminal commands and their screen context")]
    fn send(&mut self, duties: &mut Duties) {
        if self.words() == tui::EXIT && self.root.pages().chat().is_some_and(|chat| chat.talk().is_some()) {
            self.composer_words(Box::default());
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
        if let Some(command) = self.bare_own() {
            self.own(command, duties);
            return;
        }
        if let Some(command) = self.state().place()
            && !self.words().starts_with(b"/")
        {
            if core::str::from_utf8(self.words()).is_ok_and(|words| words.chars().all(char::is_whitespace)) {
                return;
            }
            self.tell(Told::Send);
            let words = skein_lib::bytes::concat(&[command.name(), b" ", self.words()]);
            if command == tui::Own::Tell {
                self.tell_words(&words);
            } else {
                self.launchpad_words(&words);
            }
            return;
        }
        self.tell(Told::NotOpened { address: None });
        self.tell(Told::Send);
        let value: Box<[u8]> = self.words().into();
        if self.tell_words(&value) {
            return;
        }
        let words = self.words();
        let command_words = words.starts_with(b"/")
            && core::str::from_utf8(words).is_ok_and(|words| !words.chars().any(char::is_whitespace));
        if self.root.pages().chat().is_none() && self.root.pages().project().is_some() && self.launchpad_words(&value) {
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

    fn tell_words(&mut self, value: &[u8]) -> bool {
        let project = self
            .root
            .pages()
            .chat()
            .and_then(|chat| chat.task(self.root.client()))
            .map(|task| task.project)
            .or(self.root.pages().project());
        if let Some(project) = project
            && let Some(words) = value.strip_prefix(b"/tell ").filter(|words| !words.is_empty())
        {
            let words = words.into();
            let title = self
                .root
                .pages()
                .chat()
                .and_then(|chat| chat.task(self.root.client()))
                .map(|task| task.chip.title.bytes().into());
            let about = Some(root::pages::tell::About { address: self.root.pages().address().clone(), title });
            self.act(Act::Tell { project, words, about });
            self.composer_words(Box::default());
            self.tell(Told::Cleared);
            return true;
        }
        false
    }

    fn bare_own(&self) -> Option<tui::Own> {
        tui::offered_own(&self.root, self.state()).iter().copied().find(|command| match command {
            tui::Own::Draft | tui::Own::Change | tui::Own::Ask | tui::Own::Tell => self
                .words()
                .strip_prefix(command.name())
                .is_some_and(|rest| core::str::from_utf8(rest).is_ok_and(|rest| rest.chars().all(char::is_whitespace))),
            tui::Own::Brief | tui::Own::Needs | tui::Own::Talk | tui::Own::Theme | tui::Own::Open | tui::Own::Web => {
                false
            }
        })
    }

    fn launchpad_words(&mut self, words: &[u8]) -> bool {
        if let Some(words) = words.strip_prefix(b"/talk ").filter(|words| !words.is_empty()) {
            let words: Box<[u8]> = words.into();
            let project = self.root.pages().project().expect("launchpad project");
            let team = root::pages::team::shown_at(&self.root, project, self.env.wall);
            if let Some(role) = team
                .facts()
                .into_iter()
                .flat_map(|facts| facts.roles.iter())
                .find(|role| role.name.get(..words.len()).is_some_and(|prefix| prefix.eq_ignore_ascii_case(&words)))
            {
                let address = Address::Talk { project, task: role.task };
                self.composer_words(Box::default());
                self.act(Act::Go { address });
                self.tell(Told::Cleared);
            } else {
                let project = self
                    .root
                    .pages()
                    .me(self.root.client())
                    .and_then(|me| me.projects.iter().find(|row| row.project == project))
                    .map(|row| row.name.bytes().into())
                    .unwrap_or_default();
                self.notice(tui::Notice::NoRole { project, words });
            }
        } else if let Some(words) = words.strip_prefix(b"/draft ").filter(|words| !words.is_empty())
            && self.root.pages().project().is_some_and(|project| root::pages::drafts_offered(&self.root, project))
        {
            let words = words.into();
            self.start_at(Address::NewChat { draft: true });
            self.act(Act::ChatWords { words });
            self.act(Act::StartChat);
            self.tell(Told::Cleared);
        } else if let Some(words) = words.strip_prefix(b"/change ").filter(|words| !words.is_empty()) {
            let words = words.into();
            self.start_at(Address::NewChat { draft: false });
            self.act(Act::ChatWords { words });
            self.act(Act::StartChat);
            self.tell(Told::Cleared);
        } else if let Some(words) = words.strip_prefix(b"/ask ").filter(|words| !words.is_empty()) {
            let words = words.into();
            if let Some(project) = self.root.pages().project()
                && self.root.pages().address() != &(Address::Home { project })
            {
                self.start_at(Address::Home { project });
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
                self.composer_words(Box::default());
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

    fn confirm(&mut self) {
        let closing = self.root.pages().chat().and_then(|chat| {
            let open = self.root.pages().confirmation()?;
            if chat.restarting
                || chat.closing.is_none()
                || open.sent.is_some()
                || open.object != jig_client::contract::ObjectKey::Task(chat.task)
                || open.act != jig_client::contract::ActKind::Cancel
            {
                return None;
            }
            let task = chat.task(self.root.client())?;
            let title = task.chip.title.bytes();
            Some(tui::Closing {
                task: chat.task,
                project: task.project,
                title: (!title.is_empty()).then(|| title.into()),
            })
        });
        self.act(Act::Confirm);
        if closing.is_some() && self.root.pages().confirmation().is_some_and(|open| open.sent.is_some()) {
            self.tell(Told::Closing { chat: closing });
        }
    }

    fn closed_chat(&mut self) {
        let Some(closing) = self.state().closing() else { return };
        let Some(chat) = self.root.pages().chat().filter(|chat| chat.task == closing.task) else {
            self.tell(Told::Closing { chat: None });
            return;
        };
        if chat.outcome_kind != Some(root::pages::chat::Sending::Close) {
            return;
        }
        let Some(outcome) = &chat.outcome else { return };
        let done = matches!(
            outcome,
            jig_client::Outcome::Answered(jig_client::Answered::Jig(jig_client::contract::Answer::Done(_)))
        );
        let closing = closing.clone();
        self.tell(Told::Closing { chat: None });
        if done {
            self.tell(Told::Closed { task: closing.task });
            self.notice(tui::Notice::Closed { task: closing.task, title: closing.title });
            self.went(Address::Home { project: closing.project });
        }
    }

    fn leave(&mut self) {
        let Some(talk) = self.root.pages().chat().and_then(|chat| chat.talk()) else { return };
        let project = talk.project;
        if let Some(open) = talk.open() {
            let name = talk.name().unwrap_or(b"").into();
            let minutes = self
                .env
                .wall
                .as_nanos()
                .saturating_sub(open.opened.as_nanos())
                .checked_div(60_000_000_000)
                .expect("minute is nonzero")
                .max(1);
            let cost = talk.cost();
            self.notice(tui::Notice::Left { name, minutes, cost });
        }
        self.went(Address::Home { project });
    }
    fn went(&mut self, address: Address) {
        let left = self.leaving_box();
        self.tell(Told::Place { command: None });
        self.start_at(address);
        self.enter_box(left);
    }

    /// A start may pass through `NewChat` or `Home` without ending the TUI place.
    fn start_at(&mut self, address: Address) {
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
            self.went(Address::Home { project: project.project });
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
            | Address::Waits { project, .. }
            | Address::Changes { project }
            | Address::Settings { project }
            | Address::Repositories { project }
            | Address::Models { project }
            | Address::Builders { project }
            | Address::Team { project, .. }
            | Address::Role { project, .. }
            | Address::Rule { project, .. }
            | Address::Ask { project }
            | Address::Tell { project }
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

    fn next_approval(&mut self) {
        self.approval_number = self.approval_number.checked_add(1).expect("approval number fits u64");
        self.asked_opening = Some(self.state().openings());
        self.browse = false;
    }

    fn ask(&mut self, duties: &mut Duties) {
        self.next_approval();
        if self.state().address().at_name() {
            self.listener = Listener::Opening { asked: self.root.asked() };
            self.tell(Told::Listener { opening: true });
            duties.listen = true;
        } else {
            self.browse = true;
            self.step(Event::AskApproval { port: None });
        }
    }

    fn listener_wanted(&self) -> bool {
        if self.closed || self.refused() || self.need.is_some() || self.root.client().link().state != Link::SignedOut {
            return false;
        }
        match self.listener {
            Listener::None => false,
            Listener::Opening { asked } => {
                self.asked_opening == Some(self.state().openings()) && self.root.asked() == asked
            }
            Listener::Open { port, asking } => {
                !tui::may_ask_again(self.state(), &self.root)
                    && match self.root.asked() {
                        // Once our ask was answered, Asking belongs to a replacement.
                        Some(root::asked::State::Asking) => asking,
                        Some(root::asked::State::Asked { port: asked, .. }) => asked == Some(port),
                        None | Some(root::asked::State::NotAsked(_)) => false,
                    }
            }
        }
    }

    fn unlisten(&mut self, duties: &mut Duties) {
        if !matches!(self.listener, Listener::None) {
            self.listener = Listener::None;
            self.browse = false;
            self.tell(Told::Listener { opening: false });
            duties.unlisten = true;
        }
    }

    fn listening(&mut self, port: Option<u16>, duties: &mut Duties) {
        if matches!(self.listener, Listener::Opening { .. }) && self.listener_wanted() {
            self.listener = port.map_or(Listener::None, |port| Listener::Open { port, asking: true });
            self.tell(Told::Listener { opening: false });
            self.browse = true;
            self.step(Event::AskApproval { port });
        } else {
            self.unlisten(duties);
            duties.unlisten |= port.is_some();
        }
    }

    fn returned(&mut self, returned: &crate::loopback::Returned) -> crate::loopback::Answer {
        if let Listener::Open { port, .. } = self.listener
            && self.listener_wanted()
            && let Some(root::asked::State::Asked { code, port: Some(asked), .. }) = self.root.asked()
            && asked == port
            && let Some(grant) = crate::loopback::grant(returned, port)
        {
            let to = tui::approval_address(self.state().address(), Some(code));
            self.step(Event::Granted { grant });
            return crate::loopback::Answer::Redirect { to };
        }
        crate::loopback::Answer::NotFound
    }

    fn approval(&mut self, duties: &mut Duties) {
        if !self.listener_wanted() {
            self.unlisten(duties);
        } else if let Listener::Open { asking, .. } = &mut self.listener
            && matches!(self.root.asked(), Some(root::asked::State::Asked { .. }))
        {
            *asking = false;
        }
        if self.closed || self.refused() || self.need.is_some() {
            return;
        }
        if self.root.client().link().state == Link::SignedOut && self.asked_opening != Some(self.state().openings()) {
            self.ask(duties);
            self.route(duties);
        }
        if self.browse {
            match self.root.asked() {
                Some(root::asked::State::Asked { code, port, .. }) => {
                    let code = if self.state().address().at_name() && port.is_none() { None } else { Some(code) };
                    duties.open = Some(tui::approval_address(self.state().address(), code));
                    self.opens.push(Opening::Approval { number: self.approval_number });
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
            | jig_client::Event::AskLink
            | jig_client::Event::AskApproval { .. }
            | jig_client::Event::ReadApproval { .. }
            | jig_client::Event::DecideApproval { .. }
            | jig_client::Event::ApprovalAnswer { .. }
            | jig_client::Event::SignOutLost
            | jig_client::Event::Watch { .. }
            | jig_client::Event::Unwatch { .. }
            | jig_client::Event::Read { .. }
            | jig_client::Event::Send { .. }
            | jig_client::Event::Carry { .. }
            | jig_client::Event::Part { .. }
            | jig_client::Event::Done { .. }
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
        if self.refused() {
            return;
        }
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
        if self.handed {
            self.front_theme = Some(choice);
        }
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
        self.unlisten(duties);
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
        if self.refused() {
            return None;
        }
        if *key == Key::Control(b'o') {
            return tui::pane_key(&self.view);
        }
        if let Some(event) = self.arrow(key) {
            return Some(event);
        }
        if let Some(event) = self.back(key) {
            return Some(event);
        }
        let mut focus = self.focus;
        match list_command(&self.view, key, &mut focus, &self.bounds) {
            Command::Act { node, act } => tui::event(&self.view, node, act),
            Command::Nothing => self.key_without_act(key),
            Command::Moved | Command::Close => None,
        }
    }

    fn back(&self, key: &Key) -> Option<tui::Event> {
        if *key == Key::Left { tui::back(&self.view, self.focus.node?, self.focus.caret) } else { None }
    }

    fn arrow(&self, key: &Key) -> Option<tui::Event> {
        let aliased = alias(key, self.view.tree(), &self.focus);
        let arrow = match aliased.as_ref().unwrap_or(key) {
            Key::Up => tui::Arrow::Up,
            Key::Down => tui::Arrow::Down,
            Key::Tab => tui::Arrow::Tab,
            Key::BackTab => tui::Arrow::BackTab,
            Key::Enter | Key::Right => tui::Arrow::Open,
            Key::Char(_)
            | Key::Paste(_)
            | Key::Backspace
            | Key::Delete
            | Key::Left
            | Key::Home
            | Key::End
            | Key::PageUp
            | Key::PageDown
            | Key::Escape
            | Key::Control(_)
            | Key::Alt(_) => return None,
        };
        tui::arrow(&self.view, arrow)
    }

    fn key_without_act(&self, key: &Key) -> Option<tui::Event> {
        if *key == Key::Escape {
            tui::escape(&self.view)
        } else if self.view.list_focused() && types(key) {
            // Delete at the end and Backspace in an empty box edit no bytes.
            Some(tui::Event::Picked { row: None })
        } else {
            None
        }
    }

    #[must_use]
    pub fn deadline(&self) -> Option<Time> {
        if self.closed || self.refused() {
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

fn types(key: &Key) -> bool {
    matches!(key, Key::Char(_) | Key::Paste(_) | Key::Backspace | Key::Delete)
}

fn list_command(view: &tui::View, key: &Key, focus: &mut Focus, bounds: &skein_ui::Limits) -> Command {
    if view.list_focused() {
        if types(key) {
            let Some(node) = view.box_node() else { return Command::Nothing };
            focus.node = Some(node);
            focus.caret = view
                .tree()
                .find(node)
                .and_then(|node| node.value.as_ref())
                .map_or(0, |value| u32::try_from(value.text.len()).expect("box text fits"));
        } else if !matches!(key, Key::PageUp | Key::PageDown | Key::Control(b'c')) {
            return Command::Nothing;
        }
    }
    command(key, view.tree(), focus, bounds)
}

fn same_origin(origin: &tui::Origin, authority: &[u8]) -> bool {
    match origin {
        tui::Origin::Http(_) => origin.authority().as_ref() == authority,
        tui::Origin::Https { .. } => false,
    }
}
