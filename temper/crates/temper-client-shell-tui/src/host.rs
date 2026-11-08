//! Reports in and duties out (temper/docs/design/client/README.md, 6.13).
use crate::limits::{self, Limits};
use jig_client::{Link, Message, With};
use skein_lib::{Duration, Env, List, Queue, Secret, Set, Time, Token, Wall};
use skein_ui::Patch;
use skein_ui_screen::{Cell, Focus, Frame, Size, Theme};
use skein_ui_screen_terminal::{Command, Key, command};
use temper_client_domain::{self as root, Address, Event, address::Filter};
use temper_client_shell_terminal::route::{self, Down, Duty};
use temper_client_tui::{self as tui, Local, Note, Refusal, Told};

pub const LOCAL_READS: u8 = 3;
pub const LOCAL_AGAIN: Duration = Duration::from_secs(1);
pub const EXPIRY_SLACK: Duration = Duration::from_secs(60);

struct Need {
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

pub struct Start {
    pub size: Size,
    pub scheme: tui::Scheme,
    pub address: Box<[u8]>,
    pub local: bool,
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
    Kept { written: bool },
    Opened { opened: bool },
    Local { link: Option<Secret> },
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
}

pub struct Settings {
    pub limits: Limits,
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "closing, browsing, address persistence and local sign-in are independent host facts"
)]
pub struct Host {
    root: root::Domain,
    state: Option<tui::Tui>,
    view: tui::View,
    env: Env<root::Limits>,
    out: Queue<root::Request>,
    patches: Queue<Patch>,
    bounds: skein_ui::Limits,
    theme: Theme,
    size: Size,
    focus: Focus,
    frame: Frame,
    mapping: jig_client_protocol_http::Mapping,
    credential: Option<Secret>,
    signing: Set<Token>,
    local: bool,
    note: Option<Note>,
    need: Option<Need>,
    local_signing: bool,
    asked_opening: Option<u64>,
    browse: bool,
    addressed: bool,
    closed: bool,
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
            root: root::new(&limits.native.client, seed),
            state: None,
            view: tui::View::new(&limits.view, words),
            env: Env { now, wall, limits: limits.native.client },
            out: Queue::uncapped(),
            patches: Queue::with_capacity(limits.view.patches),
            bounds: limits.view,
            theme: tui::theme(tui::Scheme::Dark),
            size: Size { rows: 0, columns: 0 },
            focus: Focus::default(),
            frame: Frame::new(limits.most),
            mapping: jig_client_protocol_http::Mapping::new(b"/jig").expect("mapping prefix"),
            credential: None,
            signing: Set::with_capacity(temper_client_shell_terminal::limits::EXCHANGES),
            local: false,
            note: None,
            need: None,
            local_signing: false,
            asked_opening: None,
            browse: false,
            addressed: false,
            closed: false,
        }
    }

    pub fn clocks(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
    }

    pub fn report(&mut self, report: Report) -> Duties {
        if self.state.is_none() {
            assert!(matches!(report, Report::Start(_)), "a report before the TUI's start");
        }
        if self.closed && !matches!(report, Report::Heard(_) | Report::Kept { .. }) {
            return Duties::default();
        }
        let mut duties = Duties::default();
        match report {
            Report::Local { link } => self.found(link),
            Report::Start(start) => self.start(start, &mut duties),
            Report::Due => {
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
            Report::Key(key) => match command(&key, self.view.tree(), &mut self.focus, &self.bounds) {
                Command::Act { node, act } => {
                    if let Some(event) = tui::event(&self.view, node, act) {
                        self.event(event, &mut duties);
                    }
                }
                Command::Close => self.close(&mut duties),
                Command::Moved | Command::Nothing => {}
            },
            Report::Resize(size) => self.size = size,
            Report::Heard(event) => self.heard(event),
            Report::Kept { written } => self.tell(Told::Kept { written }),
            Report::Opened { opened } => self.tell(Told::Opened { opened }),
        }
        self.route(&mut duties);
        root::take(&mut self.root, &self.env, &mut self.out);
        self.route(&mut duties);
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
        if credential != self.credential && (credential.is_some() || state == Link::SignedOut) {
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
        self.approval(&mut duties);
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
        self.size = start.size;
        self.theme = tui::theme(start.scheme);
        self.credential.clone_from(&start.credential);
        self.local = start.local;
        self.note = start.note;
        let mut kept = List::with_capacity(u32::try_from(start.kept.len()).expect("bounded store"));
        for record in start.kept {
            kept.push(record).expect("counted store");
        }
        self.step(Event::Start {
            address: Address::Inbox { filter: Filter::ALL },
            kept,
            with: start.credential.map(With::Held),
        });
        if self.credential.is_none() {
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
        self.need = Some(Need { reads: 1, digest: None, phase: Phase::Reading });
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
        if link.host.as_ref() != self.state().address() {
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
        self.local_signing = true;
        self.step(Event::Client {
            event: jig_client::Event::SignIn { with: With::Link { secret: link.secret, held: None } },
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
        root::step(&mut self.root, &self.env, event, &mut self.out);
    }

    fn event(&mut self, event: tui::Event, duties: &mut Duties) {
        match event {
            tui::Event::Typed { value } => self.tell(Told::Typed { value }),
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
            tui::Event::Quit => self.close(duties),
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
                    self.tell(Told::Browse);
                    self.browse = false;
                }
                Some(root::asked::State::NotAsked(_)) => self.browse = false,
                None | Some(root::asked::State::Asking) => {}
            }
        }
    }

    fn heard(&mut self, event: jig_client::Event) {
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
                | Down::Duty(Duty::Show { .. } | Duty::SignOut | Duty::Fact(_))
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
                    duties.asked.push(request);
                }
                Down::Duty(Duty::Copy { token }) => {
                    self.step(Event::Copied { token, written: root::copy::Written::Refused });
                }
            }
        }
    }

    fn close(&mut self, duties: &mut Duties) {
        self.closed = true;
        duties.close = true;
    }

    pub fn frame(&mut self) -> &Frame {
        if self.state.is_some() {
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
        let mut focus = self.focus;
        match command(key, self.view.tree(), &mut focus, &self.bounds) {
            Command::Act { node, act } => tui::event(&self.view, node, act),
            Command::Moved | Command::Close | Command::Nothing => None,
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
        skein_lib::earliest(skein_lib::earliest(root::next_deadline(&self.root), tick), local)
    }
}

#[cfg(test)]
#[path = "tests.rs"]
pub(crate) mod tests;
