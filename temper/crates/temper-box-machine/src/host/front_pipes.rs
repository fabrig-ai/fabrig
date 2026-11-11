//! The parent's pipes beside the steps (temper/docs/design/protocol/setup/box.md, 6).
use super::{Front, Input, Io, Read, Request, Service, Sink, Token, Up};
use skein_lib::{Queue, stream::Down};

#[derive(Debug, PartialEq, Eq)]
enum InputState {
    Ready,
    Reading,
    Ended,
}

#[derive(Debug)]
pub(super) struct Pipes {
    input: Token,
    input_state: InputState,
    started: bool,
    link: Sink,
    url: Option<skein_lib::Secret>,
    pub(super) lines: Sink,
    closing: bool,
}
impl Pipes {
    pub fn new(io: &mut Io, front: Front) -> Self {
        let input = io.adopt_read_pipe(front.input).expect("parent input slot");
        let link = io.adopt_write_pipe(front.link_output, None).expect("parent link slot");
        let lines = io.adopt_write_pipe(front.lines, None).expect("parent lines slot");
        let link = Sink::new(link, 1);
        Self {
            input,
            input_state: InputState::Ready,
            started: false,
            link,
            url: Some(front.link),
            lines: Sink::new(lines, crate::front::HELD),
            closing: false,
        }
    }
    pub fn start(&mut self) {
        self.link.push(self.url.take().expect("local URL").into_exposed()).expect("one local URL");
        self.started = true;
    }
    pub fn link_closed(&self) -> bool {
        self.link.is_closed()
    }
    pub fn closed(&mut self, owner: Token) {
        for sink in [&mut self.link, &mut self.lines] {
            if sink.token == owner {
                sink.closed();
            }
        }
    }
    pub fn owns_stream(&self, owner: Token) -> bool {
        self.input == owner || self.link.token == owner || self.lines.token == owner
    }
    pub fn stream(&mut self, owner: Token, up: Up, requests: &mut Queue<Request>) -> bool {
        if owner == self.input {
            if self.input_state != InputState::Ended {
                self.input_state = InputState::Ready;
            }
            match up {
                Up::End | Up::Failed(_) if self.input_state != InputState::Ended => {
                    self.input_state = InputState::Ended;
                    requests.push(Request::Close { entity: self.input });
                    return true;
                }
                Up::Bytes(_) | Up::Room | Up::End | Up::Failed(_) => {}
            }
        } else if owner == self.link.token {
            self.link.up(up, requests);
        } else if owner == self.lines.token {
            self.lines.up(up, requests);
        }
        false
    }
    pub fn pending(&self) -> bool {
        !self.closing
            && (self.input_state == InputState::Ready
                || self.started && (self.link.pending() || self.link.needs_flush())
                || self.lines.pending())
    }
    pub fn close(&mut self, requests: &mut Queue<Request>) {
        // An unread lines pipe must never delay the box's exit.
        if !self.closing {
            if self.input_state != InputState::Ended {
                requests.push(Request::Close { entity: self.input });
                self.input_state = InputState::Ended;
            }
            if !self.link.is_closed() {
                requests.push(Request::Abort { entity: self.link.token });
            }
            if !self.lines.is_closed() {
                requests.push(Request::Abort { entity: self.lines.token });
            }
            self.closing = true;
        }
    }
    fn tick(&mut self, output: u32, requests: &mut Queue<Request>) {
        if self.closing {
            return;
        }
        if self.input_state == InputState::Ready {
            requests.push(Request::Stream {
                stream: self.input,
                down: Down::Demand { read: Read::Arrived { max: 1 }, room: 0 },
            });
            self.input_state = InputState::Reading;
        }
        if self.started && !self.link.is_closed() {
            self.link.flush(output, requests);
            self.link.tick(self.link.drained(), requests);
        }
        self.lines.tick(false, requests);
    }
}
impl Service {
    pub(super) fn front_running(&self) -> bool {
        self.front.is_some()
    }
    pub(super) fn front_line(&mut self, line: crate::front::Line) {
        if let Some(front) = &mut self.front {
            drop(front.lines.push(line.bytes()));
        }
    }
    pub(super) fn box_log_ready(&self) -> bool {
        self.logs.takes() && (self.front.is_none() || self.phase != super::Phase::New)
    }
    pub(super) fn terminate(&mut self) {
        self.input(Input::Terminate);
    }
    pub(super) fn front_work(&mut self) {
        if self.requests.room() >= 8
            && let Some(front) = &mut self.front
        {
            front.tick(self.env.limits.output, &mut self.requests);
        }
    }
}
