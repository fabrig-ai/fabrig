//! The browser closes, its child exits, then its server is ended (testing/browser.md, 3).
use super::{BROWSER, CAPTURE_WINDOW, CLOSE_WINDOW, IMAGE, PROCESS, SERVER, SNAPSHOT};
use skein_browser::boundary::Request;
use skein_io::{
    self as io,
    kernel::{Signal, Target},
};
use skein_lib::{Duration, Time, Token};
use std::collections::VecDeque;

// These are cleanup cutoffs, not waits whose expiry can change the verdict.
pub(super) struct Capture {
    until: Time,
    snapshot: bool,
    image: bool,
}
impl Capture {
    pub fn new(now: Time) -> Self {
        Self { until: now.saturating_add(CAPTURE_WINDOW), snapshot: true, image: true }
    }
    pub fn answer(&mut self, op: Token, now: Time) -> bool {
        if now >= self.until {
            return false;
        }
        match op {
            SNAPSHOT => std::mem::take(&mut self.snapshot),
            IMAGE => std::mem::take(&mut self.image),
            _ => false,
        }
    }
    pub fn done(&self, now: Time) -> bool {
        now >= self.until || !(self.snapshot || self.image)
    }
    pub fn until(&self) -> Time {
        self.until
    }
}

pub(super) struct CloseWindow {
    force_at: Time,
    until: Time,
}
impl CloseWindow {
    pub fn new(now: Time, drain: Duration) -> Self {
        // Reserve io's drain time inside the window, rather than signaling at return.
        let grace = Duration::from_nanos(CLOSE_WINDOW.as_nanos().saturating_sub(drain.as_nanos()));
        Self { force_at: now.saturating_add(grace), until: now.saturating_add(CLOSE_WINDOW) }
    }
    pub fn force(&self, now: Time) -> bool {
        now >= self.force_at
    }
    pub fn done(&self, now: Time) -> bool {
        now >= self.until
    }
    pub fn next(&self, now: Time) -> Option<Time> {
        [self.force_at, self.until].into_iter().find(|at| *at > now)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Open,
    Browser,
    Child,
    Forced,
    Server,
    ServerForced,
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
    pub fn browser_closed(&mut self, owner: Token, requests: &mut VecDeque<io::Request>) {
        if owner == BROWSER && self.phase == Phase::Browser {
            self.phase = Phase::Child;
            if let Some(child) = self.browser {
                end(child, requests);
            }
        }
    }
    /// The closing window forces the browser child first.
    pub fn force(&mut self, requests: &mut VecDeque<io::Request>) {
        match self.phase {
            Phase::Browser | Phase::Child => {
                if let Some(child) = self.browser {
                    if self.phase == Phase::Browser {
                        end(child, requests);
                    }
                    requests.push_back(io::Request::Close { entity: child });
                }
                self.phase = Phase::Forced;
            }
            Phase::Server => {
                if let Some(child) = self.server {
                    requests.push_back(io::Request::Close { entity: child });
                }
                self.phase = Phase::ServerForced;
            }
            Phase::Open | Phase::Forced | Phase::ServerForced => {}
        }
    }
    pub fn io_event(&mut self, event: &io::Event, requests: &mut VecDeque<io::Request>) {
        match event {
            io::Event::Spawned { owner, child, .. } => {
                if *owner == PROCESS {
                    self.browser = Some(*child);
                    match self.phase {
                        Phase::Child | Phase::Forced => {
                            end(*child, requests);
                            if self.phase == Phase::Forced {
                                requests.push_back(io::Request::Close { entity: *child });
                            }
                        }
                        Phase::Open | Phase::Browser | Phase::Server | Phase::ServerForced => {}
                    }
                } else if *owner == SERVER {
                    self.server = Some(*child);
                    if matches!(self.phase, Phase::Server | Phase::ServerForced) {
                        end(*child, requests);
                        if self.phase == Phase::ServerForced {
                            requests.push_back(io::Request::Close { entity: *child });
                        }
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
    pub fn advance(&mut self, requests: &mut VecDeque<io::Request>) {
        if self.browser_exited && matches!(self.phase, Phase::Browser | Phase::Child | Phase::Forced) {
            let forced = self.phase == Phase::Forced;
            self.phase = Phase::Server;
            if let Some(child) = self.server {
                end(child, requests);
            }
            if forced {
                self.force(requests);
            }
        }
    }
}
fn end(child: Token, requests: &mut VecDeque<io::Request>) {
    requests.push_back(io::Request::Signal {
        child,
        signal: Signal::Terminate,
        to: Target::Child,
        reason: io::kernel::Reason::Unknown,
    });
}
