//! The bridge in memory, for a native test (playground.md, 5).

use alloc::rc::Rc;

use skein_lib::{Time, Wall};
use skein_shell::Host;
use skein_sim_world::Machine;
use skein_ui_web::{
    bridge, fetch,
    inbox::Inbox,
    log::Log,
    platform::{
        Platform,
        worker::{self, Heard, Page},
    },
    store,
    transport::StreamDuty,
};

use crate::slices::{Slices, Wake};

/// A platform carrying the transport's duties to the slices in the same thread,
/// raising their reports as the worker's platform does (playground.md, 5).
pub struct Memory<P, M, R> {
    page: Page,
    slices: Slices<P, M>,
    log: Log,
    inbox: Rc<Inbox<R>>,
    fetch: fn(fetch::Report) -> R,
    heard: fn(Heard) -> R,
    shown: Option<Box<[u8]>>,
}

impl<P: core::fmt::Debug, M: core::fmt::Debug, R> core::fmt::Debug for Memory<P, M, R> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Memory")
            .field("page", &self.page)
            .field("slices", &self.slices)
            .field("log", &self.log)
            .field("shown", &self.shown)
            .finish_non_exhaustive()
    }
}

impl<P: Host, M: Machine, R> Memory<P, M, R> {
    #[must_use]
    pub fn new(
        slices: Slices<P, M>,
        start: bridge::Start,
        log_bound: u32,
        inbox: Rc<Inbox<R>>,
        fetch: fn(fetch::Report) -> R,
        heard: fn(Heard) -> R,
    ) -> Self {
        let log = Log::new(None, &start, log_bound);
        let mut page = Page::new(start, Time::ZERO);
        page.ready();
        Self { page, slices, log, inbox, fetch, heard, shown: None }
    }

    /// One slice at the test's browser instant; its posts raised in the inbox.
    pub fn slice(&mut self, browser: Time) -> Wake {
        let mut outgoing = Vec::new();
        self.page.outgoing(&mut outgoing);
        for message in outgoing {
            match message {
                bridge::ToWorld::Input(input) => self.slices.input(input),
                bridge::ToWorld::Pace(speed) => self.slices.pace(speed, browser),
                bridge::ToWorld::Start(_) => {}
            }
        }
        let mut posts = Vec::new();
        let wake = self.slices.slice(browser, &mut posts);
        for message in posts {
            self.page.heard(&message, browser);
            self.log.heard(&message);
            match message {
                bridge::ToPage::Report(report) => self.inbox.raise((self.fetch)(report)),
                bridge::ToPage::Printed(printed) => self.inbox.raise((self.heard)(Heard::Printed(printed))),
                bridge::ToPage::Stopped => self.inbox.raise((self.heard)(Heard::Stopped)),
                bridge::ToPage::Failed { words } => self.inbox.raise((self.heard)(Heard::Failed { words })),
                bridge::ToPage::Clock(_) => self.inbox.raise((self.heard)(Heard::Clock)),
                bridge::ToPage::Stamp(_) => {}
            }
        }
        wake
    }

    pub fn pace(&mut self, speed: bridge::Speed, browser: Time) {
        self.page.pace(speed, browser);
    }

    pub fn clocks(&mut self, browser: Time) -> (Time, Wall) {
        self.page.clocks(browser)
    }

    #[must_use]
    pub fn timer(&self, deadline: Time, browser: Time) -> Option<Time> {
        self.page.timer(deadline, browser)
    }

    #[must_use]
    pub fn log(&self) -> Box<[u8]> {
        self.log.text()
    }

    /// The last navigation `worker::shown` accepted; natively none is followed.
    #[must_use]
    pub fn shown(&self) -> Option<&[u8]> {
        self.shown.as_deref()
    }

    #[must_use]
    pub fn slices(&self) -> &Slices<P, M> {
        &self.slices
    }

    pub fn slices_mut(&mut self) -> &mut Slices<P, M> {
        &mut self.slices
    }

    fn send_input(&mut self, input: bridge::Input) {
        self.log.sent(&input);
        self.page.send(input);
    }
}

impl<P: Host, M: Machine, R> Platform for Memory<P, M, R> {
    fn read_all(&self) -> Vec<store::Record> {
        self.page.read_all()
    }

    fn write(&mut self, key: &[u8], value: &[u8]) -> bool {
        self.page.write(key, value)
    }

    fn remove(&mut self, key: &[u8]) {
        self.page.remove(key);
    }

    fn send(&mut self, exchange: &fetch::Exchange) {
        self.send_input(bridge::Input::Exchange(exchange.clone()));
    }

    fn stream(&mut self, duty: StreamDuty) {
        self.send_input(bridge::Input::Stream(duty));
    }

    fn abandon(&mut self, exchange: u32) {
        self.send_input(bridge::Input::Abandon { exchange });
    }

    fn read_on(&mut self) {
        self.send_input(bridge::Input::ReadOn);
    }

    fn navigate(&mut self, to: &[u8]) {
        if worker::shown(to) {
            self.shown = Some(to.into());
        }
    }
}
