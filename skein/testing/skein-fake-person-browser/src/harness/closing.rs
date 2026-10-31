//! The browser closes, its child exits, then its server is ended (browser.md, 3).
use super::{BROWSER, PROCESS, SERVER};
use skein_browser::boundary::Request;
use skein_io::{
    self as io,
    kernel::{Signal, Target},
};
use skein_lib::{Queue, Token};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Open,
    Browser,
    Child,
    Forced,
    Server,
}

pub(crate) struct Closing {
    phase: Phase,
    browser: Option<Token>,
    server: Option<Token>,
    browser_exited: bool,
}
impl Closing {
    pub fn new() -> Self {
        Self { phase: Phase::Open, browser: None, server: None, browser_exited: false }
    }
    pub fn active(&self) -> bool {
        self.phase != Phase::Open
    }
    pub fn server_child(&self) -> Option<Token> {
        self.server
    }
    pub fn begin(&mut self, unstarted: bool) -> Option<Request> {
        self.phase = Phase::Browser;
        if unstarted {
            self.browser_exited = true;
            None
        } else {
            Some(Request::Close { entity: BROWSER })
        }
    }
    pub fn browser_closed(&mut self, owner: Token, requests: &mut Queue<io::Request>) {
        if owner == BROWSER && self.phase == Phase::Browser {
            self.phase = Phase::Child;
            if let Some(child) = self.browser {
                end(child, requests);
            }
        }
    }
    /// The run's existing closing deadline forces the browser child first.
    pub fn force(&mut self, requests: &mut Queue<io::Request>) {
        match self.phase {
            Phase::Browser | Phase::Child => {
                if let Some(child) = self.browser {
                    if self.phase == Phase::Browser {
                        end(child, requests);
                    }
                    requests.push(io::Request::Close { entity: child });
                }
                self.phase = Phase::Forced;
            }
            Phase::Open | Phase::Forced | Phase::Server => {}
        }
    }
    pub fn io_event(&mut self, event: &io::Event, requests: &mut Queue<io::Request>) {
        match event {
            io::Event::Spawned { owner, child, .. } => {
                if *owner == PROCESS {
                    self.browser = Some(*child);
                    match self.phase {
                        Phase::Child | Phase::Forced => {
                            end(*child, requests);
                            if self.phase == Phase::Forced {
                                requests.push(io::Request::Close { entity: *child });
                            }
                        }
                        Phase::Open | Phase::Browser | Phase::Server => {}
                    }
                } else if *owner == SERVER {
                    self.server = Some(*child);
                    if self.phase == Phase::Server {
                        end(*child, requests);
                    }
                }
            }
            io::Event::Exited { owner, .. } | io::Event::Closed { owner } => {
                if *owner == PROCESS {
                    self.browser = None;
                    self.browser_exited = true;
                } else if *owner == SERVER {
                    self.server = None;
                }
            }
            io::Event::Failed { owner, .. } if *owner == PROCESS && self.browser.is_none() => {
                self.browser_exited = true;
            }
            io::Event::Failed { .. }
            | io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Stream { .. }
            | io::Event::Output { .. }
            | io::Event::Usage { .. }
            | io::Event::Resized
            | io::Event::Window { .. }
            | io::Event::Shutdown { .. } => {}
        }
        self.advance(requests);
    }
    pub fn advance(&mut self, requests: &mut Queue<io::Request>) {
        if self.active() && self.browser_exited && self.phase != Phase::Server {
            self.phase = Phase::Server;
            if let Some(child) = self.server {
                end(child, requests);
            }
        }
    }
}
fn end(child: Token, requests: &mut Queue<io::Request>) {
    requests.push(io::Request::Signal { child, signal: Signal::Terminate, to: Target::Child });
}
