//! The ordered client record stream (hearth/docs/design/client/README.md, 3.2 and 3.3).
use crate::{
    Asked,
    builder::vocabulary::{Answer, Progress},
    records::{self, limits::Limits, printer, records::Record},
    render::vocabulary::Outcome,
    resource::Resource,
    translate::answers,
};
use alloc::boxed::Box;
use core::mem;
use skein_io as io;
use skein_lib::{
    Env, Queue, Token,
    stream::{Down, Up},
};
#[derive(Debug)]
enum Stream {
    Taking,
    Printing,
    Ending { outcome: Outcome },
    Finished { outcome: Outcome },
    Broken { outcome: Option<Outcome> },
}
impl Stream {
    fn outcome(&self) -> Option<Outcome> {
        match self {
            Self::Taking | Self::Printing => None,
            Self::Ending { outcome, .. } | Self::Finished { outcome } => Some(*outcome),
            Self::Broken { outcome } => *outcome,
        }
    }
    fn finished(&self) -> bool {
        match self {
            Self::Taking | Self::Printing | Self::Ending { .. } => false,
            Self::Finished { .. } | Self::Broken { .. } => true,
        }
    }
    fn broken(&self) -> bool {
        match self {
            Self::Taking | Self::Printing | Self::Ending { .. } | Self::Finished { .. } => false,
            Self::Broken { .. } => true,
        }
    }
    fn end(&mut self, outcome: Outcome) {
        *self = match mem::replace(self, Self::Broken { outcome: None }) {
            Self::Taking | Self::Printing | Self::Ending { .. } => Self::Ending { outcome },
            Self::Finished { .. } => Self::Finished { outcome },
            Self::Broken { .. } => Self::Broken { outcome: Some(outcome) },
        };
    }
    fn printing(&mut self, printing: bool) {
        *self = match mem::replace(self, Self::Broken { outcome: None }) {
            Self::Taking | Self::Printing => {
                if printing {
                    Self::Printing
                } else {
                    Self::Taking
                }
            }
            stream @ (Self::Ending { .. } | Self::Finished { .. } | Self::Broken { .. }) => stream,
        };
    }
}
#[derive(Debug)]
pub(crate) struct Client {
    printer: printer::Printer,
    events: Queue<printer::Event>,
    requests: Queue<Down>,
    output: Resource,
    stream: Stream,
    refused: bool,
    /// Why a build ended on a step, when it did.
    failed: Option<records::build::Why>,
}
impl Client {
    pub fn new(form: records::form::Form, limits: Limits) -> Self {
        Self {
            printer: printer::Printer::new(form, &limits),
            events: Queue::uncapped(),
            requests: Queue::uncapped(),
            output: Resource::Awaited,
            stream: Stream::Taking,
            refused: false,
            failed: None,
        }
    }
    pub fn bound(&self) -> bool {
        self.output != Resource::Awaited
    }
    pub fn bind(&mut self, output: Token) {
        assert!(self.output.token().is_none(), "one client output");
        self.output = Resource::Open(output);
    }
    pub fn owns(&self, event: &io::Event) -> bool {
        match event {
            io::Event::Stream { owner, .. } | io::Event::Failed { owner, .. } | io::Event::Closed { owner } => {
                self.output.token() == Some(*owner)
            }
            io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Output { .. }
            | io::Event::Usage { .. }
            | io::Event::Window { .. }
            | io::Event::Resized
            | io::Event::Shutdown { .. } => false,
        }
    }
    pub fn up(&mut self, env: &Env<Limits>, event: io::Event) {
        match event {
            io::Event::Closed { .. } => {
                self.output = match mem::replace(&mut self.output, Resource::Closed) {
                    Resource::Awaited | Resource::Open(_) => Resource::Awaited,
                    Resource::Closing(_) | Resource::Closed => Resource::Closed,
                };
            }
            io::Event::Stream { up: Up::Failed(_), .. } | io::Event::Failed { .. } => self.fail(),
            io::Event::Stream { up, .. } => {
                if !self.stream.broken() {
                    printer::up(&mut self.printer, env, up, &mut self.events, &mut self.requests);
                }
            }
            io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Output { .. }
            | io::Event::Usage { .. }
            | io::Event::Window { .. }
            | io::Event::Resized
            | io::Event::Shutdown { .. } => unreachable!("client output events"),
        }
    }
    fn fail(&mut self) {
        self.stream = Stream::Broken { outcome: self.stream.outcome() };
        for _ in 0..self.requests.len() {
            drop(self.requests.pop());
        }
        for _ in 0..self.events.len() {
            self.events.pop();
        }
    }
    pub fn takes(&self) -> bool {
        let taking = match self.stream {
            Stream::Taking | Stream::Broken { outcome: None } => true,
            Stream::Printing
            | Stream::Ending { .. }
            | Stream::Finished { .. }
            | Stream::Broken { outcome: Some(_) } => false,
        };
        taking && self.requests.is_empty() && self.events.is_empty()
    }
    /// A notice of standard error's (hearth/docs/design/client/README.md, 3.2).
    pub fn progress(&mut self, env: &Env<Limits>, progress: Progress, store: &[u8]) {
        assert!(self.takes(), "printer takes one record at a time");
        self.print(env, crate::translate::store::progress(progress, store));
    }
    pub fn answer(&mut self, env: &Env<Limits>, answer: Answer, store: &[u8]) {
        assert!(self.takes(), "printer takes one record at a time");
        let record = match answer {
            Answer::Spec(spec) => Record::Spec(Box::new(answers::spec(spec))),
            Answer::Error(mut error) => {
                if let crate::render::errors::Kind::Store(fault) = &mut error.kind {
                    fault.store = skein_lib::bytes::copy_of(store);
                }
                Record::Error(crate::translate::errors::error(error))
            }
            Answer::Change(change) => Record::Change(super::translate::changes::change(change)),
            Answer::Layer { layer, key, state } => Record::Layer(answers::layer(layer, key, state)),
            Answer::Failed { step, why } => {
                let failed = answers::failed(step, why);
                self.failed = Some(failed.why);
                Record::Failed(failed)
            }
            Answer::Full { limit, most } => Record::Error(answers::full(limit, most, store)),
        };
        self.print(env, record);
    }
    pub fn reply(
        &mut self,
        env: &Env<Limits>,
        to: skein_lib::ReplyTo,
        outcome: Outcome,
        diff: Option<crate::render::diff::Tally>,
        left_out: u32,
    ) {
        assert!(self.takes(), "printer takes one record at a time");
        assert_eq!(to.into_token(), Token::new(0), "the initial call's reply");
        self.stream.end(outcome);
        let outcome = self.outcome(outcome);
        let diff = super::translate::changes::optional_tally(diff);
        self.print(env, Record::Terminal(records::records::Terminal { outcome, diff, left_out }));
    }
    /// The terminal's outcome: a refused record fails the answer, and an
    /// unavailable step makes a complete one unavailable (hearth/docs/design/client/README.md, 3.3).
    fn outcome(&self, outcome: Outcome) -> records::outcome::Outcome {
        let outcome = answers::outcome(if self.refused { outcome.worst(Outcome::Failed) } else { outcome });
        match (self.failed, outcome) {
            (Some(records::build::Why::Unavailable), records::outcome::Outcome::Done) => {
                records::outcome::Outcome::Unavailable
            }
            (Some(records::build::Why::Unavailable) | None, _) => outcome,
        }
    }
    fn print(&mut self, env: &Env<Limits>, record: Record) {
        if !self.stream.broken() {
            self.stream.printing(true);
            printer::down(
                &mut self.printer,
                env,
                printer::Request::Print(record),
                &mut self.events,
                &mut self.requests,
            );
        }
    }
    pub fn finish(&mut self, env: &Env<Limits>) {
        self.stream.end(Outcome::Done);
        if !self.stream.broken() {
            printer::down(&mut self.printer, env, printer::Request::Finish, &mut self.events, &mut self.requests);
        }
    }
    pub fn stop(&mut self) {
        self.stream = if self.stream.broken() {
            Stream::Broken { outcome: Some(Outcome::TryAgain) }
        } else {
            Stream::Finished { outcome: Outcome::TryAgain }
        };
        self.requests.push(Down::Finish);
    }
    pub fn work_pending(&self) -> bool {
        !self.requests.is_empty() || !self.events.is_empty() || (self.output.is_open() && self.stream.finished())
    }
    pub fn is_empty(&self) -> bool {
        self.stream.finished() && self.output.token().is_none() && self.stream.outcome().is_some()
    }
    pub fn code(&self) -> Option<u8> {
        if !self.is_empty() {
            return None;
        }
        if self.stream.broken() {
            return Some(75);
        }
        Some(records::outcome::code(self.outcome(self.stream.outcome().expect("terminal outcome"))))
    }
    pub fn resume(&mut self, env: &Env<Limits>, below: &mut Queue<Asked>) {
        if let Some(down) = self.requests.pop() {
            if down == Down::Finish && !self.stream.broken() {
                self.stream = Stream::Finished { outcome: self.stream.outcome().expect("terminal before Finish") };
            }
            below.push(Asked::Io(io::Request::Stream { stream: self.output.token().expect("bound output"), down }));
            return;
        }
        if let Some(event) = self.events.pop() {
            match event {
                printer::Event::Printed => {}
                printer::Event::Refused(_) => self.refused = true,
                printer::Event::Failed(_) => {
                    self.fail();
                    return;
                }
                printer::Event::Closed => unreachable!("printer closes with its output"),
            }
            self.stream.printing(false);
            if self.stream.outcome().is_some() {
                printer::down(&mut self.printer, env, printer::Request::Finish, &mut self.events, &mut self.requests);
            }
        } else if self.stream.finished()
            && let Some(entity) = self.output.request_close()
        {
            below.push(Asked::Io(io::Request::Close { entity }));
        }
    }
}
