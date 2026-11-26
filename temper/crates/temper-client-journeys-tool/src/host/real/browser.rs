//! One raw Pump for the command's effects. The check, not the transport, judges the reads.

use std::{
    ffi::OsStr,
    os::unix::ffi::OsStrExt as _,
    path::{Path, PathBuf},
};

use skein_browser::{Event, Request};
use skein_fake_person_browser::harness::session::{RawPump, Report};
use skein_lib::{Duration, Token};

use crate::{
    EnvironmentEntry,
    host::browser::{Answer, Command, Observation, Reply},
    table::CHROMIUM,
};

// Allow a page's setup to make more than one round trip. The kit also bounds each wire command.
const WAIT: Duration = Duration::from_secs(30);
// If a lost browser does not answer its close command, let io end and reap its child.
const CLOSE: Duration = Duration::from_secs(3);

pub(in crate::host) struct Browser {
    pump: RawPump,
    serial: u64,
}

pub(super) fn ask(stored: &mut Option<Box<Browser>>, environment: &[EnvironmentEntry], command: Command) -> Answer {
    let mut trouble = Vec::new();
    let reply = match command {
        Command::Start { profile } => match Browser::start(environment, profile) {
            Ok(browser) => {
                let next = stored.insert(Box::new(browser));
                next.wait(Waiting::Ready, &mut trouble)
            }
            Err(why) => Reply::Failed(why),
        },
        Command::Stop => stored.take().map_or(Reply::Done, |mut browser| browser.stop(&mut trouble)),
        command => stored.as_mut().map_or_else(
            || Reply::Failed("Chromium not started".into()),
            |browser| browser.command(command, &mut trouble),
        ),
    };
    Answer { reply, trouble }
}

#[derive(Clone, Copy)]
enum Waiting {
    Ready,
    Person,
    Open(Token),
    Operation(Token),
    Close(Token),
}

impl Waiting {
    fn take(&self, event: Event, trouble: &mut Vec<Observation>) -> Option<Reply> {
        match event {
            Event::Ready { .. } if matches!(self, Self::Ready) => Some(Reply::Done),
            Event::Opened { page } if matches!(self, Self::Open(expected) if *expected == page) => Some(Reply::Done),
            Event::Done { op } if self.operation(op) => Some(Reply::Done),
            Event::Elements { op, elements, more } if self.operation(op) => {
                Some(Reply::Elements { elements: elements.into_boxed().into_vec(), more })
            }
            Event::Style { op, style } if self.operation(op) => Some(Reply::Style(style.into_boxed().into_vec())),
            Event::Rect { op, rect } if self.operation(op) => Some(Reply::Rect(rect)),
            Event::Refused { op, why, detail } if self.operation(op) => Some(Reply::Refused { why, detail }),
            Event::Closed { owner } if matches!(self, Self::Close(page) if *page == owner) => Some(Reply::Done),
            Event::Closed { .. } => Some(Reply::Failed("Chromium or its page closed".into())),
            event => {
                observe(event, trouble);
                None
            }
        }
    }

    fn operation(&self, op: Token) -> bool {
        matches!(self, Self::Operation(expected) if *expected == op)
    }
}

impl Browser {
    fn start(environment: &[EnvironmentEntry], profile: PathBuf) -> Result<Self, String> {
        let chromium = CHROMIUM
            .value(environment)
            .filter(|bytes| !bytes.is_empty() && !bytes.contains(&0))
            .ok_or("set SKEIN_TEST_CHROMIUM to the Chromium executable")?;
        let chromium = std::path::absolute(Path::new(OsStr::from_bytes(chromium)))
            .map_err(|why| format!("Chromium path: {why}"))?;
        // Chromium gets the kit's fixed locale and a private home, not the caller's environment.
        let environment = vec![("HOME".into(), profile.as_os_str().to_owned())];
        let pump = RawPump::start(chromium, profile, environment, Token::new(0), true)?;
        Ok(Self { pump, serial: 0 })
    }

    fn command(&mut self, command: Command, trouble: &mut Vec<Observation>) -> Reply {
        self.serial += 1;
        let op = Token::new(self.serial);
        let (request, waiting) = match command {
            Command::Start { .. } | Command::Stop => unreachable!("lifecycle requests are handled by the host"),
            Command::Person { person } => (Request::Person { person, downloads: None }, Waiting::Person),
            Command::Open { person, page, url, files } => {
                (Request::Page { person, page, url, requests: false, files }, Waiting::Open(page))
            }
            Command::Viewport { page, width, height } => {
                (Request::Viewport { page, op, width, height }, Waiting::Operation(op))
            }
            Command::Elements { page, selector } => (Request::Elements { page, op, selector }, Waiting::Operation(op)),
            Command::Style { page, node } => (Request::Style { page, op, node }, Waiting::Operation(op)),
            Command::Rect { page, node } => (Request::Rect { page, op, node }, Waiting::Operation(op)),
            Command::Close { page } => (Request::Close { entity: page }, Waiting::Close(page)),
        };
        self.pump.request(request);
        self.wait(waiting, trouble)
    }

    fn wait(&mut self, waiting: Waiting, trouble: &mut Vec<Observation>) -> Reply {
        let until = self.pump.now().saturating_add(WAIT);
        let idle = matches!(waiting, Waiting::Person);
        loop {
            let (reports, events) = self.pump.poll(Some(until), idle);
            let mut reply = None;
            for event in events {
                if let Some(next) = waiting.take(event, trouble)
                    && !matches!(reply, Some(Reply::Failed(_)))
                {
                    reply = Some(next);
                }
            }
            if let Some(why) = problem(reports) {
                return Reply::Failed(why);
            }
            if let Some(reply) = reply {
                return reply;
            }
            if idle && !self.pump.pending() {
                return Reply::Done;
            }
            if self.pump.now() >= until {
                return Reply::Failed("Chromium did not answer".into());
            }
        }
    }

    fn stop(&mut self, trouble: &mut Vec<Observation>) -> Reply {
        self.pump.close();
        let until = self.pump.now().saturating_add(CLOSE);
        let mut deadline = Some(until);
        let mut failure = None;
        while !self.pump.is_empty() {
            let (reports, events) = self.pump.poll(deadline, false);
            for event in events {
                observe(event, trouble);
            }
            for report in reports {
                if let Report::Trouble { why, .. } = report {
                    failure = Some(why);
                }
            }
            if deadline.is_some() && self.pump.now() >= until && !self.pump.is_empty() {
                self.pump.force();
                deadline = None;
            }
        }
        failure.map_or(Reply::Done, Reply::Failed)
    }
}

fn problem(reports: Vec<Report>) -> Option<String> {
    reports.into_iter().find_map(|report| match report {
        Report::Trouble { why, .. } => Some(why),
        Report::BrowserClosed => Some("Chromium closed".into()),
        _ => None,
    })
}

fn observe(event: Event, trouble: &mut Vec<Observation>) {
    if let Event::Trouble { page, trouble: kind, text, url } = event {
        trouble.push(Observation { page, kind, text, url });
    }
}
