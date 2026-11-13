//! The full-screen terminal on tagged io (skein/docs/design/ui/screen.md, 8).
use crate::{ASK_BACKGROUND, Decoder, Depth, ENTER, Key, LEAVE, draw, draw_bound};
use skein_io::{
    Event, Io, Request, Window,
    kernel::{Complete, Fd, Op, Submit},
};
use skein_lib::{
    Duration, Env, List, Queue, Time, Token, Wall, bytes,
    stream::{Down, Read, Up},
    usize_of,
};
use skein_ui_screen::{Cell, Frame, Position, Rgb, Size};

const NAMESPACE: u8 = 1;
const READ: u32 = 4096;
const OUTPUT: u32 = 65_536;

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub interval: Duration,
    pub depth: Depth,
    pub most: Size,
    pub paste: u32,
    pub background: Option<Duration>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Background {
    Waiting,
    Answered(Rgb),
    Unanswered,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Open,
    Lending,
    LendingLeave,
    Lent,
    Closing,
    Leaving,
    Draining,
    Closed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Withdrawal {
    Unasked,
    Waiting,
    Quiet,
}

#[derive(Debug)]
struct Input {
    decoder: Decoder,
    keys: Queue<Key>,
    reading: bool,
    ended: bool,
}
#[derive(Debug)]
struct WindowState {
    size: Size,
    pending: bool,
    again: bool,
}
#[derive(Debug)]
pub struct Terminal {
    settings: Settings,
    window: WindowState,
    io: Io,
    limits: skein_io::Limits,
    input: Option<Token>,
    output: Option<Token>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    landed: Queue<Complete>,
    raw: Queue<Submit>,
    events: Queue<Event>,
    intake: Input,
    background: Background,
    background_deadline: Option<Time>,
    writable: bool,
    withdrawal: Withdrawal,
    last: Frame,
    newest: Frame,
    whole: bool,
    dirty: bool,
    drawn: Option<Time>,
    bytes: List<u8>,
    offset: Option<u32>,
    phase: Phase,
}

impl Terminal {
    #[must_use]
    pub const fn background(&self) -> Background {
        self.background
    }

    fn io_limits(settings: &Settings) -> Option<skein_io::Limits> {
        draw_bound(settings.most)?;
        if settings.paste == 0 || settings.interval.as_nanos() == 0 {
            return None;
        }
        Some(skein_io::Limits {
            sockets: 2,
            names: 0,
            refusals: 1,
            intake: READ,
            receive: READ,
            output: OUTPUT,
            sends: 1,
            accepts: 1,
            backlog: 1,
            close_timeout: Duration::from_secs(30),
            retry: Duration::from_millis(1),
        })
    }
    /// Includes io's independent window-request slot (skein/docs/design/runtime/io/README.md, 4).
    #[must_use]
    pub fn operations(settings: &Settings) -> Option<u32> {
        skein_io::operations(&Self::io_limits(settings)?)
    }
    #[must_use]
    pub fn new(settings: Settings, size: Size) -> Option<Terminal> {
        let limits = Self::io_limits(&settings)?;
        if size.rows == 0 || size.columns == 0 {
            return None;
        }
        Some(Self {
            settings,
            window: WindowState { size, pending: false, again: false },
            io: Io::new(&limits),
            limits,
            input: None,
            output: None,
            completions: Queue::uncapped(),
            submissions: Queue::uncapped(),
            landed: Queue::uncapped(),
            raw: Queue::uncapped(),
            events: Queue::uncapped(),
            intake: Input {
                decoder: Decoder::new(settings.paste),
                keys: Queue::with_capacity(READ.checked_add(1)?),
                reading: false,
                ended: false,
            },
            background: if settings.background.is_some() { Background::Waiting } else { Background::Unanswered },
            background_deadline: None,
            writable: false,
            withdrawal: Withdrawal::Unasked,
            last: Frame::new(settings.most),
            newest: Frame::new(settings.most),
            whole: true,
            dirty: false,
            drawn: None,
            bytes: List::with_capacity(draw_bound(settings.most)?.max(64)),
            offset: None,
            phase: Phase::Open,
        })
    }
    /// Queues ENTER before any frame. Gives back the descriptor io refused.
    pub fn adopt(&mut self, input: Fd, output: Fd) -> Result<(), Fd> {
        assert!(self.input.is_none() && self.output.is_none(), "adopt once");
        self.input = Some(self.io.adopt_read_pipe(input)?);
        self.output = Some(self.io.adopt_write_pipe(output, None)?);
        self.control(ENTER);
        if self.settings.background.is_some() {
            for byte in ASK_BACKGROUND {
                self.bytes.push(*byte).expect("ENTER and background query fit control output");
            }
            self.intake.decoder.answers();
        }
        self.demand_output(Time::ZERO);
        Ok(())
    }
    #[must_use]
    pub fn owns(op: Token) -> bool {
        op.tag() == NAMESPACE
    }
    pub fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    pub fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    pub fn keys(&mut self) -> &mut Queue<Key> {
        &mut self.intake.keys
    }
    pub fn resized(&mut self) {
        if self.phase == Phase::Open {
            self.window.again = true;
        }
    }
    #[must_use]
    pub const fn size(&self) -> Size {
        self.window.size
    }
    pub fn draw(&mut self, frame: &Frame) {
        match self.phase {
            Phase::Open | Phase::Lending | Phase::LendingLeave | Phase::Lent => {}
            Phase::Closing | Phase::Leaving | Phase::Draining | Phase::Closed => return,
        }
        let size = frame.size();
        assert!(
            size.rows <= self.settings.most.rows && size.columns <= self.settings.most.columns,
            "the host's frame fits the terminal's most"
        );
        self.newest.clear(size, Cell::BLANK);
        for row in 0..size.rows {
            for (column, cell) in frame.row(row).expect("row within frame").iter().enumerate() {
                let at = Position { row, column: u16::try_from(column).expect("column fits frame") };
                assert!(self.newest.put(at, *cell), "cell fits newest frame");
            }
        }
        self.newest.set_cursor(frame.cursor());
        self.dirty = true;
    }
    #[must_use]
    pub const fn ended(&self) -> bool {
        self.intake.ended
    }
    /// Stops drawing, takes back the read, and leaves the screen for a child.
    pub fn lend(&mut self) {
        if self.phase == Phase::Open {
            self.phase = Phase::Lending;
            self.withdrawal = Withdrawal::Unasked;
        }
    }
    #[must_use]
    pub fn lent(&self) -> bool {
        self.phase == Phase::Lent
    }
    /// Enters again, refreshes the window, and draws the next frame whole.
    pub fn resume(&mut self) {
        if self.phase == Phase::Lent {
            self.phase = Phase::Open;
            self.control(ENTER);
            self.whole = true;
            self.drawn = None;
            self.window.again = true;
        }
    }
    pub fn close(&mut self) {
        match self.phase {
            Phase::Open | Phase::Lending => self.phase = Phase::Closing,
            Phase::LendingLeave => self.phase = Phase::Leaving,
            Phase::Lent => {
                self.drain(Time::ZERO);
                self.phase = Phase::Closed;
            }
            Phase::Closing | Phase::Leaving | Phase::Draining | Phase::Closed => {}
        }
    }
    #[must_use]
    pub const fn closed(&self) -> bool {
        match self.phase {
            Phase::Closed => true,
            Phase::Open
            | Phase::Lending
            | Phase::LendingLeave
            | Phase::Lent
            | Phase::Closing
            | Phase::Leaving
            | Phase::Draining => false,
        }
    }
    fn draw_deadline(&self) -> Option<Time> {
        if self.phase == Phase::Open && self.dirty && self.writable && self.offset.is_none() {
            Some(match self.drawn {
                Some(at) => at.saturating_add(self.settings.interval),
                None => Time::ZERO,
            })
        } else {
            None
        }
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        let next = match (self.io.next_deadline(), self.draw_deadline()) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        match (next, self.background_deadline) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        !self.completions.is_empty()
            || !self.landed.is_empty()
            || !self.submissions.is_empty()
            || self.io.is_ready()
            || self.io.is_due(now)
            || (self.phase == Phase::Open
                && self.input.is_some()
                && ((!self.intake.reading && !self.intake.ended && self.intake.keys.is_empty())
                    || (self.window.again && !self.window.pending)))
            || (self.phase == Phase::Lending && self.withdrawal == Withdrawal::Unasked)
            || (self.writable
                && (self.offset.is_some()
                    || self.phase == Phase::Closing
                    || (self.phase == Phase::Lending && self.withdrawal == Withdrawal::Quiet && !self.window.pending)
                    || self.phase == Phase::LendingLeave
                    || (self.phase == Phase::Leaving && !self.window.pending)))
            || match self.draw_deadline() {
                Some(at) => now >= at,
                None => false,
            }
            || match self.background_deadline {
                Some(at) => now >= at,
                None => false,
            }
    }
    pub fn iterate(&mut self, now: Time) {
        if self.input.is_some()
            && self.background == Background::Waiting
            && self.background_deadline.is_none()
            && let Some(wait) = self.settings.background
        {
            self.background_deadline = Some(now.saturating_add(wait));
        }
        for _ in 0..self.completions.len() {
            let mut complete = self.completions.pop().expect("counted completion");
            assert!(Self::owns(complete.op), "host routes terminal completions");
            complete.op = Token::new(complete.op.untagged());
            if let Op::Cancel { target } = &mut complete.kind {
                *target = Token::new(target.untagged());
            }
            self.landed.push(complete);
        }
        skein_io::stage::up(
            &mut self.io,
            &Env { now, wall: Wall::EPOCH, limits: self.limits },
            &mut self.landed,
            &mut self.events,
            &mut self.raw,
            now,
        );
        self.events(now);
        self.answer_background(now);
        if self.phase == Phase::Lending && self.withdrawal == Withdrawal::Unasked {
            self.withdrawal = Withdrawal::Waiting;
            if let Some(input) = self.input {
                self.io.cancel_read(&Env { now, wall: Wall::EPOCH, limits: self.limits }, input, &mut self.raw);
            } else {
                self.withdrawal = Withdrawal::Quiet;
            }
        }
        if self.phase == Phase::Lending
            && let Some(input) = self.input
            && self.io.read_cancelled(input)
        {
            self.withdrawal = Withdrawal::Quiet;
            self.intake.reading = false;
        }
        self.output(now);
        if self.phase == Phase::Open
            && let Some(input) = self.input
        {
            if self.window.again && !self.window.pending {
                self.window.again = false;
                self.window.pending = true;
                self.request(now, Request::Window { owner: input, stream: input });
            }
            if !self.intake.reading && !self.intake.ended && self.intake.keys.is_empty() {
                self.intake.reading = true;
                self.request(
                    now,
                    Request::Stream {
                        stream: input,
                        down: Down::Demand { read: Read::Arrived { max: READ }, room: 0 },
                    },
                );
            }
        }
        self.io.reclaim();
        for _ in 0..self.raw.len() {
            let mut submit = self.raw.pop().expect("counted submission");
            submit.op = Token::tagged(NAMESPACE, submit.op.raw()).expect("finite terminal namespace");
            if let Op::Cancel { target } = &mut submit.kind {
                *target = Token::tagged(NAMESPACE, target.raw()).expect("finite terminal namespace");
            }
            self.submissions.push(submit);
        }
        if self.phase == Phase::Draining && self.io.is_empty() && self.submissions.is_empty() {
            self.phase = Phase::Closed;
        }
    }
    fn answer_background(&mut self, now: Time) {
        if self.background != Background::Waiting {
            return;
        }
        self.background = match self.intake.decoder.background() {
            Some(Some(rgb)) => Background::Answered(rgb),
            Some(None) => Background::Unanswered,
            None => {
                let due = match self.background_deadline {
                    Some(at) => now >= at,
                    None => false,
                };
                if self.intake.ended || due { Background::Unanswered } else { Background::Waiting }
            }
        };
        if self.background != Background::Waiting {
            self.background_deadline = None;
            self.intake.decoder.end_background_wait(&mut self.intake.keys);
        }
    }
    fn output(&mut self, now: Time) {
        if !self.writable {
            return;
        }
        if let Some(offset) = self.offset {
            self.write_piece(now, offset);
            return;
        }
        match self.phase {
            Phase::Open => {
                if let Some(at) = self.draw_deadline()
                    && now >= at
                {
                    self.bytes.clear();
                    draw(
                        if self.whole { None } else { Some(&self.last) },
                        &self.newest,
                        self.settings.depth,
                        &mut self.bytes,
                    );
                    core::mem::swap(&mut self.last, &mut self.newest);
                    self.whole = false;
                    self.dirty = false;
                    self.drawn = Some(now);
                    self.write_piece(now, 0);
                }
            }
            Phase::Lending => {
                if self.withdrawal == Withdrawal::Quiet && !self.window.pending {
                    self.control(LEAVE);
                    self.phase = Phase::LendingLeave;
                    self.write_piece(now, 0);
                }
            }
            Phase::LendingLeave => self.phase = Phase::Lent,
            Phase::Closing => {
                self.control(LEAVE);
                self.phase = Phase::Leaving;
                self.write_piece(now, 0);
            }
            Phase::Leaving => {
                if !self.window.pending {
                    self.drain(now);
                }
            }
            Phase::Lent | Phase::Draining | Phase::Closed => {}
        }
    }
    fn control(&mut self, bytes: &[u8]) {
        self.bytes.clear();
        for byte in bytes {
            self.bytes.push(*byte).expect("control sequence fits output");
        }
        self.offset = Some(0);
    }
    fn write_piece(&mut self, now: Time, offset: u32) {
        let end = offset.saturating_add(OUTPUT).min(self.bytes.len());
        let piece =
            bytes::copy_of(self.bytes.as_slice().get(usize_of(offset)..usize_of(end)).expect("output piece exists"));
        self.offset = (end < self.bytes.len()).then_some(end);
        self.request(now, Request::Stream { stream: self.output.expect("adopted output"), down: Down::Send(piece) });
        self.demand_output(now);
    }
    fn demand_output(&mut self, now: Time) {
        self.writable = false;
        self.request(
            now,
            Request::Stream {
                stream: self.output.expect("adopted output"),
                down: Down::Demand { read: Read::Nothing, room: OUTPUT },
            },
        );
    }
    fn request(&mut self, now: Time, request: Request) {
        skein_io::down(&mut self.io, &Env { now, wall: Wall::EPOCH, limits: self.limits }, request, &mut self.raw);
    }
    fn drain(&mut self, now: Time) {
        self.phase = Phase::Draining;
        self.offset = None;
        self.writable = false;
        if let Some(input) = self.input {
            self.request(now, Request::Abort { entity: input });
        }
        if let Some(output) = self.output {
            self.request(now, Request::Close { entity: output });
        }
    }
    fn events(&mut self, now: Time) {
        for _ in 0..self.events.len() {
            match self.events.pop().expect("counted event") {
                Event::Stream { owner, up } => match up {
                    Up::Bytes(bytes) => {
                        self.intake.reading = false;
                        if self.phase == Phase::Open || self.phase == Phase::Lending {
                            self.intake.decoder.read(&bytes, &mut self.intake.keys);
                        }
                    }
                    Up::Room => self.writable = true,
                    Up::End | Up::Failed(_) => {
                        if self.input == Some(owner) {
                            self.intake.ended = true;
                            self.intake.reading = false;
                        } else {
                            self.drain(now);
                        }
                    }
                },
                Event::Window { window, .. } => {
                    self.window.pending = false;
                    match window {
                        Window::Size(size) => {
                            self.window.size = Size { rows: size.rows, columns: size.columns };
                            self.whole = true;
                        }
                        Window::Gone => {}
                    }
                }
                Event::Closed { owner } => {
                    if self.input == Some(owner) {
                        self.input = None;
                    }
                    if self.output == Some(owner) {
                        self.output = None;
                    }
                }
                Event::Failed { .. } => self.close(),
                Event::Listening { .. }
                | Event::Accepted { .. }
                | Event::Connecting { .. }
                | Event::Connected { .. }
                | Event::Output { .. }
                | Event::Spawned { .. }
                | Event::Exited { .. }
                | Event::Usage { .. }
                | Event::Resized
                | Event::Shutdown { .. } => unreachable!("terminal owns pipes and a window request"),
            }
        }
    }
}
