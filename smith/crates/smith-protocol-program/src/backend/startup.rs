//! Startup, window replacement and final resource settlement (smith/docs/design/protocol/programs.md, 5.1, 5.8).
//! A check owns its command; an invocation owns its window and MCP server.
//! Startup and relay EOF alarms exist only in their waiting states. Closing
//! retains the resources it must settle, without accepting another invocation.
//! `io_event` routes check and MCP terminals; `prepare`, `launch`, `initialized`
//! and `switch` advance the invocation. `close` cancels the check, and
//! `close_server` closes MCP once the owner has settled its program child.

use crate::{Command, mcp};
use alloc::boxed::Box;
use core::mem;
use skein_io as io;
use skein_lib::{Queue, Time, Token};

/// Invocation progress observed by Backend's admission and stream routers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Phase {
    /// No invocation has been admitted.
    Idle,
    /// The source status command still owns resources.
    Checking,
    /// Private files, MCP or program initialization are pending.
    Starting,
    /// The invocation accepts completions or is replacing its window.
    Running,
    /// The owner is settling resources and private files.
    Ending,
    /// Every owned resource has settled.
    Ended,
}

/// The session selected for one program invocation.
#[derive(Clone, Copy)]
pub(super) enum Window {
    /// Preparation has not selected a session.
    Unidentified,
    /// A compaction uses its own deterministic command UUID.
    Compaction([u8; 36]),
    /// A new window starts this session.
    New([u8; 16]),
    /// A persisted window resumes this session.
    Resume([u8; 16]),
}

/// One invocation lifecycle and the last window recorded by its input/init.
pub(super) struct Startup {
    state: State,
    recorded: Option<[u8; 16]>,
}

enum State {
    Idle,
    Checking(Command),
    Preparing(Invocation),
    Starting { invocation: Invocation, until: Time },
    Running(Invocation),
    RelayEnded { invocation: Invocation, until: Time },
    Switching(Invocation),
    Closing(Resources),
    Ended,
}

enum Resources {
    Empty,
    Check(Command),
    CancelCheck(Command),
    Invocation(Invocation),
}

struct Invocation {
    window: Window,
    server: Server,
}

enum Server {
    Absent,
    Live(Box<mcp::Server>),
    Close(Box<mcp::Server>),
    Closing(Box<mcp::Server>),
    Ended(Box<mcp::Server>),
}

enum Change {
    Window(Window),
    Server(Box<mcp::Server>),
    Close,
}

impl Invocation {
    fn new() -> Self {
        Self { window: Window::Unidentified, server: Server::Absent }
    }
}

impl Startup {
    pub fn new() -> Self {
        Self { state: State::Idle, recorded: None }
    }

    pub fn owns(&self, owner: Token) -> bool {
        let check = match self.check() {
            Some(check) => check.owns(owner),
            None => false,
        };
        let server = match self.server() {
            Some(server) => server.owns(owner),
            None => false,
        };
        check || server
    }

    pub fn io_event(&mut self, now: Time, event: io::Event, below: &mut Queue<io::Request>) -> Option<io::Event> {
        let owner = event.owner().expect("startup has no signal events");
        if let Some(check) = self.check_mut()
            && check.owns(owner)
        {
            check.from_io(event, below);
            follow(self, below);
            return None;
        }
        if let Some(server) = self.server_mut()
            && server.owns(owner)
        {
            server.from_io(now, event, below);
            follow(self, below);
            return None;
        }
        Some(event)
    }

    pub fn phase(&self) -> Phase {
        match self.state {
            State::Idle => Phase::Idle,
            State::Checking(_) => Phase::Checking,
            State::Preparing(_) | State::Starting { .. } => Phase::Starting,
            State::Running(_) | State::RelayEnded { .. } | State::Switching(_) => Phase::Running,
            State::Closing(_) => Phase::Ending,
            State::Ended => Phase::Ended,
        }
    }

    pub fn checking(&mut self, check: Command, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Ended);
        self.state = match state {
            State::Idle => State::Checking(check),
            State::Checking(_)
            | State::Preparing(_)
            | State::Starting { .. }
            | State::Running(_)
            | State::RelayEnded { .. }
            | State::Switching(_)
            | State::Closing(_)
            | State::Ended => unreachable!("check starts once"),
        };
        follow(self, below);
    }

    pub fn prepare(&mut self, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Ended);
        self.state = match state {
            State::Idle => prepared(),
            State::Checking(_) => unreachable!("preparation follows the check terminal"),
            state @ (State::Preparing(_)
            | State::Starting { .. }
            | State::Running(_)
            | State::RelayEnded { .. }
            | State::Switching(_)
            | State::Closing(_)
            | State::Ended) => state,
        };
        follow(self, below);
    }

    pub fn checked(&mut self, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Ended);
        self.state = match state {
            State::Checking(_) => prepared(),
            State::Closing(Resources::Check(_) | Resources::CancelCheck(_)) => State::Closing(Resources::Empty),
            state @ (State::Idle
            | State::Preparing(_)
            | State::Starting { .. }
            | State::Running(_)
            | State::RelayEnded { .. }
            | State::Switching(_)
            | State::Closing(Resources::Empty | Resources::Invocation(_))
            | State::Ended) => state,
        };
        follow(self, below);
    }

    pub fn launch(&mut self, until: Time, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Ended);
        self.state = match state {
            State::Preparing(invocation) | State::Starting { invocation, .. } => State::Starting { invocation, until },
            State::Idle
            | State::Checking(_)
            | State::Running(_)
            | State::RelayEnded { .. }
            | State::Switching(_)
            | State::Closing(_)
            | State::Ended => unreachable!("launch follows invocation preparation"),
        };
        follow(self, below);
    }

    pub fn initialized(&mut self, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Ended);
        self.state = match state {
            State::Preparing(invocation) | State::Starting { invocation, .. } => State::Running(invocation),
            state @ (State::Idle
            | State::Checking(_)
            | State::Running(_)
            | State::RelayEnded { .. }
            | State::Switching(_)
            | State::Closing(_)
            | State::Ended) => state,
        };
        follow(self, below);
    }

    pub fn switch(&mut self, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Ended);
        self.state = match state {
            State::Preparing(invocation)
            | State::Starting { invocation, .. }
            | State::Running(invocation)
            | State::RelayEnded { invocation, .. } => State::Switching(invocation),
            State::Idle => State::Switching(Invocation::new()),
            State::Checking(_) => unreachable!("window replacement follows the check terminal"),
            state @ (State::Switching(_) | State::Closing(_) | State::Ended) => state,
        };
        follow(self, below);
    }

    pub fn switching(&self) -> bool {
        match self.state {
            State::Switching(_) => true,
            State::Idle
            | State::Checking(_)
            | State::Preparing(_)
            | State::Starting { .. }
            | State::Running(_)
            | State::RelayEnded { .. }
            | State::Closing(_)
            | State::Ended => false,
        }
    }

    pub fn reset(&mut self, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Ended);
        self.state = match state {
            State::Switching(invocation) => released(invocation, State::Idle),
            State::Idle
            | State::Checking(_)
            | State::Preparing(_)
            | State::Starting { .. }
            | State::Running(_)
            | State::RelayEnded { .. }
            | State::Closing(_)
            | State::Ended => unreachable!("reset follows a settled window switch"),
        };
        follow(self, below);
    }

    pub fn close(&mut self, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Ended);
        self.state = match state {
            State::Idle => State::Closing(Resources::Empty),
            State::Checking(check) => State::Closing(Resources::CancelCheck(check)),
            State::Preparing(invocation)
            | State::Starting { invocation, .. }
            | State::Running(invocation)
            | State::RelayEnded { invocation, .. }
            | State::Switching(invocation) => State::Closing(Resources::Invocation(invocation)),
            state @ (State::Closing(_) | State::Ended) => state,
        };
        follow(self, below);
    }

    pub fn ended(&mut self, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Ended);
        self.state = match state {
            State::Closing(Resources::Empty) => State::Ended,
            State::Closing(Resources::Invocation(invocation)) => released(invocation, State::Ended),
            State::Closing(Resources::Check(_) | Resources::CancelCheck(_)) => {
                unreachable!("cleanup follows the check terminal")
            }
            State::Idle
            | State::Checking(_)
            | State::Preparing(_)
            | State::Starting { .. }
            | State::Running(_)
            | State::RelayEnded { .. }
            | State::Switching(_)
            | State::Ended => unreachable!("cleanup follows close"),
        };
        follow(self, below);
    }

    pub fn check(&self) -> Option<&Command> {
        match &self.state {
            State::Checking(check) | State::Closing(Resources::Check(check) | Resources::CancelCheck(check)) => {
                Some(check)
            }
            State::Idle
            | State::Preparing(_)
            | State::Starting { .. }
            | State::Running(_)
            | State::RelayEnded { .. }
            | State::Switching(_)
            | State::Closing(Resources::Empty | Resources::Invocation(_))
            | State::Ended => None,
        }
    }

    fn check_mut(&mut self) -> Option<&mut Command> {
        match &mut self.state {
            State::Checking(check) | State::Closing(Resources::Check(check) | Resources::CancelCheck(check)) => {
                Some(check)
            }
            State::Idle
            | State::Preparing(_)
            | State::Starting { .. }
            | State::Running(_)
            | State::RelayEnded { .. }
            | State::Switching(_)
            | State::Closing(Resources::Empty | Resources::Invocation(_))
            | State::Ended => None,
        }
    }

    fn invocation(&self) -> Option<&Invocation> {
        match &self.state {
            State::Preparing(invocation)
            | State::Starting { invocation, .. }
            | State::Running(invocation)
            | State::RelayEnded { invocation, .. }
            | State::Switching(invocation)
            | State::Closing(Resources::Invocation(invocation)) => Some(invocation),
            State::Idle
            | State::Checking(_)
            | State::Closing(Resources::Empty | Resources::Check(_) | Resources::CancelCheck(_))
            | State::Ended => None,
        }
    }

    fn invocation_mut(&mut self) -> Option<&mut Invocation> {
        match &mut self.state {
            State::Preparing(invocation)
            | State::Starting { invocation, .. }
            | State::Running(invocation)
            | State::RelayEnded { invocation, .. }
            | State::Switching(invocation)
            | State::Closing(Resources::Invocation(invocation)) => Some(invocation),
            State::Idle
            | State::Checking(_)
            | State::Closing(Resources::Empty | Resources::Check(_) | Resources::CancelCheck(_))
            | State::Ended => None,
        }
    }

    pub fn window(&mut self, window: Window, below: &mut Queue<io::Request>) {
        self.change(Change::Window(window), below);
    }

    fn change(&mut self, change: Change, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Ended);
        self.state = match state {
            State::Preparing(invocation) => State::Preparing(changed(invocation, change)),
            State::Starting { invocation, until } => State::Starting { invocation: changed(invocation, change), until },
            State::Running(invocation) => State::Running(changed(invocation, change)),
            State::RelayEnded { invocation, until } => {
                State::RelayEnded { invocation: changed(invocation, change), until }
            }
            State::Switching(invocation) => State::Switching(changed(invocation, change)),
            State::Closing(Resources::Invocation(invocation)) => {
                State::Closing(Resources::Invocation(changed(invocation, change)))
            }
            state @ (State::Idle
            | State::Checking(_)
            | State::Closing(Resources::Empty | Resources::Check(_) | Resources::CancelCheck(_))
            | State::Ended) => match change {
                Change::Close => state,
                Change::Window(_) | Change::Server(_) => unreachable!("invocation is prepared before its resources"),
            },
        };
        follow(self, below);
    }

    pub fn running_session(&self) -> Option<[u8; 16]> {
        match self.invocation() {
            Some(invocation) => match invocation.window {
                Window::New(id) | Window::Resume(id) => Some(id),
                Window::Unidentified | Window::Compaction(_) => None,
            },
            None => None,
        }
    }

    pub fn session(&self) -> [u8; 36] {
        match self.invocation() {
            Some(invocation) => match invocation.window {
                Window::New(id) | Window::Resume(id) => super::session::uuid(id),
                Window::Compaction(id) => id,
                Window::Unidentified => [0; 36],
            },
            None => [0; 36],
        }
    }

    pub fn resuming(&self) -> bool {
        match self.invocation() {
            Some(invocation) => match invocation.window {
                Window::Resume(_) => true,
                Window::Unidentified | Window::Compaction(_) | Window::New(_) => false,
            },
            None => false,
        }
    }

    pub fn remember(&mut self, below: &mut Queue<io::Request>) {
        self.recorded = self.running_session();
        follow(self, below);
    }

    pub fn recorded(&self) -> Option<[u8; 16]> {
        self.recorded
    }

    pub fn install_server(&mut self, server: mcp::Server, below: &mut Queue<io::Request>) {
        self.change(Change::Server(Box::new(server)), below);
    }

    pub fn server(&self) -> Option<&mcp::Server> {
        match self.invocation() {
            Some(invocation) => match &invocation.server {
                Server::Live(server) | Server::Close(server) | Server::Closing(server) | Server::Ended(server) => {
                    Some(server)
                }
                Server::Absent => None,
            },
            None => None,
        }
    }

    pub fn server_mut(&mut self) -> Option<&mut mcp::Server> {
        match self.invocation_mut() {
            Some(invocation) => match &mut invocation.server {
                Server::Live(server) | Server::Close(server) | Server::Closing(server) | Server::Ended(server) => {
                    Some(server)
                }
                Server::Absent => None,
            },
            None => None,
        }
    }

    pub fn server_ended(&self) -> bool {
        match self.invocation() {
            Some(invocation) => match invocation.server {
                Server::Absent | Server::Ended(_) => true,
                Server::Live(_) | Server::Close(_) | Server::Closing(_) => false,
            },
            None => true,
        }
    }

    pub fn relay_ended(&mut self, until: Option<Time>, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Ended);
        self.state = match state {
            State::Running(invocation) => match until {
                Some(until) => State::RelayEnded { invocation: relay_ended(invocation), until },
                None => State::Running(relay_ended(invocation)),
            },
            State::RelayEnded { invocation, until: held } => {
                State::RelayEnded { invocation: relay_ended(invocation), until: until.unwrap_or(held).min(held) }
            }
            State::Preparing(invocation) => State::Preparing(relay_ended(invocation)),
            State::Starting { invocation, until } => State::Starting { invocation: relay_ended(invocation), until },
            State::Switching(invocation) => State::Switching(relay_ended(invocation)),
            State::Closing(Resources::Invocation(invocation)) => {
                State::Closing(Resources::Invocation(relay_ended(invocation)))
            }
            state @ (State::Idle
            | State::Checking(_)
            | State::Closing(Resources::Empty | Resources::Check(_) | Resources::CancelCheck(_))
            | State::Ended) => state,
        };
        follow(self, below);
    }

    pub fn close_server(&mut self, below: &mut Queue<io::Request>) {
        self.change(Change::Close, below);
    }

    pub fn startup_deadline(&self) -> Option<Time> {
        match self.state {
            State::Starting { until, .. } => Some(until),
            State::Idle
            | State::Checking(_)
            | State::Preparing(_)
            | State::Running(_)
            | State::RelayEnded { .. }
            | State::Switching(_)
            | State::Closing(_)
            | State::Ended => None,
        }
    }

    pub fn eof_deadline(&self) -> Option<Time> {
        match self.state {
            State::RelayEnded { until, .. } => Some(until),
            State::Idle
            | State::Checking(_)
            | State::Preparing(_)
            | State::Starting { .. }
            | State::Running(_)
            | State::Switching(_)
            | State::Closing(_)
            | State::Ended => None,
        }
    }

    #[cfg(test)]
    pub fn preparing_fixture(&mut self, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Ended);
        self.state = match state {
            State::Idle | State::Checking(_) => prepared(),
            State::Running(invocation)
            | State::Starting { invocation, .. }
            | State::RelayEnded { invocation, .. }
            | State::Switching(invocation)
            | State::Preparing(invocation) => State::Preparing(invocation),
            State::Closing(_) | State::Ended => unreachable!("fixture prepares an open invocation"),
        };
        follow(self, below);
    }

    pub fn fire(&mut self, now: Time, below: &mut Queue<io::Request>) {
        if let Some(check) = self.check_mut() {
            check.fire(now, below);
        }
        if let Some(server) = self.server_mut() {
            server.fire(now, below);
        }
        follow(self, below);
    }
}

fn follow(startup: &mut Startup, below: &mut Queue<io::Request>) {
    let state = mem::replace(&mut startup.state, State::Ended);
    startup.state = match state {
        State::Closing(Resources::CancelCheck(mut check)) => {
            check.cancel(below);
            State::Closing(Resources::Check(check))
        }
        State::Preparing(invocation) => State::Preparing(follow_invocation(invocation, below)),
        State::Starting { invocation, until } => {
            State::Starting { invocation: follow_invocation(invocation, below), until }
        }
        State::Running(invocation) => State::Running(follow_invocation(invocation, below)),
        State::RelayEnded { invocation, until } => {
            State::RelayEnded { invocation: follow_invocation(invocation, below), until }
        }
        State::Switching(invocation) => State::Switching(follow_invocation(invocation, below)),
        State::Closing(Resources::Invocation(invocation)) => {
            State::Closing(Resources::Invocation(follow_invocation(invocation, below)))
        }
        state @ (State::Idle
        | State::Checking(_)
        | State::Closing(Resources::Empty | Resources::Check(_))
        | State::Ended) => state,
    };
}

fn follow_invocation(mut invocation: Invocation, below: &mut Queue<io::Request>) -> Invocation {
    let server = mem::replace(&mut invocation.server, Server::Absent);
    invocation.server = match server {
        Server::Close(mut server) => {
            server.close(below);
            Server::Closing(server)
        }
        server @ (Server::Absent | Server::Live(_) | Server::Closing(_) | Server::Ended(_)) => server,
    };
    invocation
}

fn prepared() -> State {
    State::Preparing(Invocation::new())
}

fn changed(mut invocation: Invocation, change: Change) -> Invocation {
    match change {
        Change::Window(window) => invocation.window = window,
        Change::Server(server) => {
            invocation.server = match invocation.server {
                Server::Absent => Server::Live(server),
                Server::Live(_) | Server::Close(_) | Server::Closing(_) | Server::Ended(_) => {
                    unreachable!("one MCP server per invocation")
                }
            };
        }
        Change::Close => {
            let server = mem::replace(&mut invocation.server, Server::Absent);
            invocation.server = match server {
                Server::Live(server) => Server::Close(server),
                server @ (Server::Absent | Server::Close(_) | Server::Closing(_) | Server::Ended(_)) => server,
            };
        }
    }
    invocation
}

fn relay_ended(mut invocation: Invocation) -> Invocation {
    let server = mem::replace(&mut invocation.server, Server::Absent);
    invocation.server = match server {
        Server::Live(server) | Server::Close(server) | Server::Closing(server) | Server::Ended(server) => {
            Server::Ended(server)
        }
        Server::Absent => Server::Absent,
    };
    invocation
}

fn released(invocation: Invocation, next: State) -> State {
    match invocation.server {
        Server::Absent | Server::Ended(_) => next,
        Server::Live(_) | Server::Close(_) | Server::Closing(_) => {
            unreachable!("MCP settlement precedes invocation release")
        }
    }
}

#[cfg(test)]
mod tests;
