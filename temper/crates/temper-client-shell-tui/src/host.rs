//! Reports in and duties out (temper/docs/design/client/README.md, 6.13).
use crate::limits::{self, Limits};
use jig_client::{Link, Message, With};
use skein_lib::{Env, List, Queue, Secret, Set, Time, Token, Wall};
use skein_ui::Patch;
use skein_ui_screen::{Cell, Focus, Frame, Size, Theme};
use skein_ui_screen_terminal::{Command, Key, command};
use temper_client_domain::{self as root, Address, Event, address::Filter};
use temper_client_shell_terminal::route::{self, Down, Duty};
use temper_client_tui::{self as tui, Refusal, Told};

pub struct Start {
    pub size: Size,
    pub scheme: tui::Scheme,
    pub address: Box<[u8]>,
    pub kept: Vec<jig_client::Record>,
    pub credential: Option<Secret>,
}

pub enum Report {
    Start(Start),
    Due,
    Key(Key),
    Resize(Size),
    Heard(jig_client::Event),
    Kept { written: bool },
}

#[derive(Default)]
pub struct Duties {
    pub asked: Vec<jig_client::Request>,
    pub keep: Option<Option<Secret>>,
    pub address: Option<Box<[u8]>>,
    pub open: Option<Box<[u8]>>,
    pub close: bool,
}

pub struct Settings {
    pub limits: Limits,
}

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
            Report::Start(start) => self.start(start),
            Report::Due => root::fire(&mut self.root, &self.env, &mut self.out),
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
        }
        self.route(&mut duties);
        root::take(&mut self.root, &self.env, &mut self.out);
        self.route(&mut duties);
        let link = self.root.client().link();
        let state = link.state;
        let live = state == Link::Live && link.party.is_some();
        let credential = link.credential.cloned();
        self.tell(Told::Link { link: state });
        if credential != self.credential && (credential.is_some() || state == Link::SignedOut) {
            duties.keep = Some(credential.clone());
            self.credential = credential;
        }
        if live && !self.addressed {
            duties.address = Some(self.state().address().into());
            self.addressed = true;
        }
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

    fn start(&mut self, start: Start) {
        assert!(self.state.is_none(), "the TUI starts once");
        self.state = Some(tui::Tui::new(&start.address));
        self.size = start.size;
        self.theme = tui::theme(start.scheme);
        self.credential.clone_from(&start.credential);
        let mut kept = List::with_capacity(u32::try_from(start.kept.len()).expect("bounded store"));
        for record in start.kept {
            kept.push(record).expect("counted store");
        }
        self.step(Event::Start {
            address: Address::Inbox { filter: Filter::ALL },
            kept,
            with: start.credential.map(With::Held),
        });
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
            tui::Event::SignIn => match jig_client_protocol_http::printed(&self.mapping, self.state().field()) {
                Err(_) => self.tell(Told::Refused { refusal: Refusal::NotALink }),
                Ok(link) if link.host.as_ref() != self.state().address() => {
                    self.tell(Told::Refused { refusal: Refusal::OtherAddress { address: link.host } });
                }
                Ok(link) => self.step(Event::Client {
                    event: jig_client::Event::SignIn {
                        with: With::Link { secret: link.secret, held: self.credential.clone() },
                    },
                }),
            },
            tui::Event::Quit => self.close(duties),
        }
    }

    fn heard(&mut self, event: jig_client::Event) {
        let answered = if let jig_client::Event::Reply { exchange, .. } = &event {
            self.signing.remove(exchange).then_some(false)
        } else if let jig_client::Event::Lost { exchange } = &event {
            self.signing.remove(exchange).then_some(true)
        } else {
            None
        };
        self.step(Event::Client { event });
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
                        self.signing.insert(*exchange).expect("bounded sign-in exchanges");
                        self.tell(Told::Asked);
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
        if self.closed { None } else { root::next_deadline(&self.root) }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
