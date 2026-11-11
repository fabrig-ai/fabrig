//! The box's front is a child beside its steps (temper/docs/design/protocol/setup/box.md, 6).
use super::{
    Duration, Exit, Front, Input, LINE_BYTES, List, Outcome, Pipe, Queue, Read, Request, Secret, Service, Sink, Tail,
    Time, Token, Up, kernel, lines, pipe, spawn_arguments, spawn_environment,
};
use crate::{lines::Output, logs::Piece};
use alloc::boxed::Box;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Stop {
    Running,
    First(Time),
    Second(Time),
    Closing,
}
#[derive(Debug)]
#[expect(
    clippy::partial_pub_fields,
    reason = "The host observes ownership and the tail; stream writers and stop timers stay local."
)]
pub(super) struct Child {
    pub owner: Option<Token>,
    pub process: Option<Token>,
    pub pipes: [Pipe; 2],
    link: Option<Sink>,
    pub(super) lines: Option<Sink>,
    lines_closing: bool,
    url: Option<Secret>,
    pub tail: Tail,
    pub ended: bool,
    failure: Option<(super::SourceEnd, skein_lib::Wall)>,
    stop: Stop,
    grace: Duration,
}
impl Child {
    pub fn new(grace: Duration, tail_lines: u32) -> Self {
        Self {
            owner: None,
            process: None,
            pipes: [Pipe::new(false), Pipe::new(false)],
            link: None,
            lines: None,
            lines_closing: false,
            url: None,
            tail: Tail::new(tail_lines),
            ended: false,
            failure: None,
            stop: Stop::Running,
            grace,
        }
    }
    pub fn spawned(&mut self, child: Token, pipes: &[Token], now: Time, requests: &mut Queue<Request>) {
        self.process = Some(child);
        for (pipe, token) in self.pipes.iter_mut().zip(pipes) {
            pipe.token = Some(*token);
        }
        let mut link = Sink::new(*pipes.get(2).expect("front fd 3"), 1);
        link.push(self.url.take().expect("front URL").into_exposed()).expect("one local link");
        self.link = Some(link);
        self.lines = Some(Sink::new(*pipes.get(3).expect("front fd 4"), crate::front::HELD));
        match self.stop {
            Stop::Running => {}
            Stop::First(_) => self.signal(now, false, requests),
            Stop::Second(_) | Stop::Closing => {
                self.stop = Stop::Running;
                self.close(requests);
            }
        }
    }
    pub fn closed(&mut self, owner: Token, wall: skein_lib::Wall) {
        if self.owner == Some(owner) {
            self.owner = None;
            self.process = None;
            self.url = None;
        }
        for sink in [&mut self.link, &mut self.lines] {
            if let Some(state) = sink
                && state.token == owner
            {
                state.closed();
                *sink = None;
            }
        }
        for pipe in &mut self.pipes {
            if pipe.token == Some(owner) {
                pipe.token = None;
                pipe.reading = false;
                pipe.lines.end(wall);
            }
        }
    }
    pub fn stream(&mut self, owner: Token, up: Up, wall: skein_lib::Wall, requests: &mut Queue<Request>) -> bool {
        for sink in [&mut self.link, &mut self.lines] {
            if let Some(sink) = sink
                && sink.token == owner
            {
                sink.up(up, requests);
                return true;
            }
        }
        for pipe in &mut self.pipes {
            if pipe.token == Some(owner) {
                pipe.reading = false;
                match up {
                    Up::Bytes(bytes) => pipe.lines.push(bytes, wall),
                    Up::End | Up::Failed(_) => {
                        pipe.lines.end(wall);
                        requests.push(Request::Close { entity: owner });
                    }
                    Up::Room => {}
                }
                return true;
            }
        }
        false
    }
    pub fn owns_stream(&self, owner: Token) -> bool {
        for sink in [&self.link, &self.lines] {
            if let Some(sink) = sink
                && sink.token == owner
            {
                return true;
            }
        }
        for pipe in &self.pipes {
            if pipe.token == Some(owner) {
                return true;
            }
        }
        false
    }
    fn signal(&mut self, now: Time, second: bool, requests: &mut Queue<Request>) {
        if let Some(child) = self.process {
            requests.push(Request::Signal { child, signal: kernel::Signal::Terminate, to: kernel::Target::Child });
        }
        let at = now.saturating_add(self.grace);
        self.stop = if second { Stop::Second(at) } else { Stop::First(at) };
    }
    fn close(&mut self, requests: &mut Queue<Request>) {
        if self.stop != Stop::Closing {
            if let Some(entity) = self.process {
                requests.push(Request::Close { entity });
            }
            self.stop = Stop::Closing;
        }
    }
    pub fn terminate(&mut self, now: Time, requests: &mut Queue<Request>) {
        if self.ended {
            return;
        }
        match self.stop {
            Stop::Running => self.signal(now, false, requests),
            Stop::First(_) | Stop::Second(_) => self.close(requests),
            Stop::Closing => {}
        }
    }
    pub fn deadline(&self) -> Option<Time> {
        if self.ended || self.process.is_none() {
            return None;
        }
        match self.stop {
            Stop::First(at) | Stop::Second(at) => Some(at),
            Stop::Running | Stop::Closing => None,
        }
    }
    pub fn fire(&mut self, now: Time, requests: &mut Queue<Request>) {
        if let Some(at) = self.deadline()
            && at <= now
        {
            match self.stop {
                Stop::First(_) => self.signal(now, true, requests),
                Stop::Second(_) => self.close(requests),
                Stop::Running | Stop::Closing => {}
            }
        }
    }
    pub fn settled(&self) -> bool {
        if !self.ended
            || self.failure.is_some()
            || self.owner.is_some()
            || self.process.is_some()
            || self.link.is_some()
            || self.lines.is_some()
        {
            return false;
        }
        for pipe in &self.pipes {
            if !pipe.settled() {
                return false;
            }
        }
        true
    }
    pub fn pending(&self, takes_log: bool) -> bool {
        if let Some(sink) = &self.link
            && sink.pending()
        {
            return true;
        }
        if let Some(sink) = &self.lines
            && !self.lines_closing
            && (sink.pending() || self.ended)
        {
            return true;
        }
        for pipe in &self.pipes {
            if takes_log && (pipe.lines.ready() || pipe.token.is_some() && !pipe.reading && pipe.lines.wants_read()) {
                return true;
            }
        }
        false
    }
}
impl Service {
    /// True after this box's front has ended; false for a box with no front.
    #[must_use]
    pub fn front_ended(&self) -> bool {
        match &self.front {
            Some(front) => front.ended,
            None => false,
        }
    }
    pub(super) fn front_reporting(&self) -> bool {
        match &self.front {
            Some(front) => front.failure.is_some(),
            None => false,
        }
    }
    pub(super) fn front_failure_ready(&self) -> bool {
        let Some(front) = &self.front else {
            return false;
        };
        if front.failure.is_none()
            || !self.terminal.takes(1) && !self.terminal.failed()
            || self.logs.step() == self.logs.front_step()
        {
            return false;
        }
        for pipe in &front.pipes {
            if !pipe.settled() {
                return false;
            }
        }
        true
    }
    pub(super) fn front_failure_work(&mut self) {
        if !self.front_failure_ready() {
            return;
        }
        let front = self.front.as_mut().expect("front failure");
        let (end, wall) = front.failure.take().expect("ready front failure");
        let words = match end {
            super::SourceEnd::Exited(exit) => pipe::exited(b"tui", exit),
            super::SourceEnd::NotStarted => pipe::not_started(b"tui"),
        };
        let mut item = self.formatter.notice(&lines::joined(&[&words, b"\n"]), wall);
        if let super::SourceEnd::Exited(_) = end {
            let step = self.logs.front_step().expect("front log");
            let length = item.bytes.len();
            front.tail.append(&mut item, &self.logs.path(step));
            let path = b"    Log: ";
            // A started front names its log even when it wrote no tail.
            if item.bytes.len() == length {
                item.bytes = lines::joined(&[&item.bytes, path, &self.logs.path(step), b"\n"]);
            }
        }
        let words = item.bytes.get(10..).expect("failure time prefix");
        self.say(self.formatter.notice(words, wall));
    }
    pub(super) fn front_running(&self) -> bool {
        match &self.front {
            Some(front) => !front.ended,
            None => false,
        }
    }
    pub(super) fn front_line(&mut self, line: crate::front::Line) {
        if self.front_running()
            && let Some(front) = &mut self.front
            && let Some(sink) = &mut front.lines
        {
            // This sink drops at its entrance; it never holds the box's work.
            drop(sink.push(line.bytes()));
        }
    }
    pub(super) fn box_log_ready(&self) -> bool {
        self.logs.takes()
            && match &self.front {
                Some(front) => front.process.is_some() || front.ended,
                None => true,
            }
    }
    pub(super) fn spawn_front(&mut self, front: Front) {
        self.serial = self.serial.checked_add(1).expect("spawn serial");
        let owner = Token::new(self.serial);
        let state = self.front.as_mut().expect("front state");
        state.owner = Some(owner);
        state.url = Some(front.link);
        let mut pipes = List::with_capacity(4);
        for (child, way) in [(1, kernel::Way::Out), (2, kernel::Way::Out), (3, kernel::Way::In), (4, kernel::Way::In)] {
            pipes.push(kernel::Pipe { child, way, parent: None }).expect("front's four pipes");
        }
        self.requests.push(Request::Spawn {
            owner,
            spawn: kernel::Spawn {
                program: self.settings.program.clone(),
                args: spawn_arguments(&self.settings, &front.arguments),
                env: spawn_environment(&self.settings.environment, 0),
                root: self.settings.root,
                dir: Box::new([]),
                pipes: pipes.into_boxed(),
            },
        });
    }
    pub(super) fn front_end(&mut self, exit: Option<Exit>) {
        let front = self.front.as_mut().expect("front's end");
        if front.ended {
            return;
        }
        front.ended = true;
        if exit != Some(Exit::Code(0)) {
            front.failure = Some((
                match exit {
                    Some(exit) => super::SourceEnd::Exited(exit),
                    None => super::SourceEnd::NotStarted,
                },
                self.env.wall,
            ));
        }
        front.close(&mut self.requests);
        self.failed |= exit != Some(Exit::Code(0));
        let words = match exit {
            Some(exit) => pipe::exited(b"tui", exit),
            None => pipe::not_started(b"tui"),
        };
        self.box_lines.push(self.formatter.notice(&lines::joined(&[&words, b"\n"]), self.env.wall));
        if self.machine.outcome().is_none() {
            self.input(Input::Terminate);
        }
    }
    pub(super) fn terminate(&mut self) {
        if let Some(front) = &mut self.front {
            front.terminate(self.env.now, &mut self.requests);
        } else {
            self.input(Input::Terminate);
            return;
        }
        match self.machine.outcome() {
            Some(Outcome::GaveUp { .. }) => {}
            Some(Outcome::Stopped) | None => self.input(Input::Terminate),
        }
    }
    pub(super) fn front_work(&mut self) {
        if self.requests.room() < 8 {
            return;
        }
        let Some(front) = &mut self.front else {
            return;
        };
        if let Some(sink) = &mut front.link {
            sink.tick(true, &mut self.requests);
        }
        if let Some(sink) = &mut front.lines
            && !front.lines_closing
        {
            if front.ended {
                self.requests.push(Request::Close { entity: sink.token });
                front.lines_closing = true;
            } else {
                sink.tick(false, &mut self.requests);
            }
        }
        let step = self.logs.front_step().expect("front's log");
        for pipe in &mut front.pipes {
            if self.logs.takes()
                && let Some(output) = pipe.lines.take(b"tui")
            {
                match output {
                    Output::Piece { bytes, wall } => self.logs.put(Piece { step, bytes, wall, log_only: false }),
                    Output::Line { .. } | Output::Legacy { .. } | Output::Refused { .. } => {
                        unreachable!("front diagnostics")
                    }
                }
            }
            if self.logs.takes()
                && pipe.lines.wants_read()
                && !pipe.reading
                && let Some(stream) = pipe.token
            {
                self.requests.push(Request::Stream {
                    stream,
                    down: skein_lib::stream::Down::Demand { read: Read::Arrived { max: LINE_BYTES }, room: 0 },
                });
                pipe.reading = true;
            }
        }
    }
}
