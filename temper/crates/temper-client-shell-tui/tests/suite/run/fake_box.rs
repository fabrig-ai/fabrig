//! Children hosted by the simulated kernel, with the shipped descriptor directions.
use super::*;
use std::collections::VecDeque;

/// The login's end is the world's, including after a caught termination signal.
/// Inherit's descriptor copying is skein's rule; this world's terminal is above 2.
pub(super) struct Login {
    pid: Pid,
    signals: Fd,
    exit: Option<Exit>,
    complete: Queue<Complete>,
    pub(super) finished: bool,
}
impl Login {
    pub(super) fn new(sim: &mut Sim, parent: Pid, pidfd: Fd) -> Self {
        let (pid, pipes) = sim.bind_service(parent, pidfd);
        assert!(pipes.is_empty(), "this simulated parent's terminal descriptors are above 2");
        Self {
            pid,
            signals: sim.open_signal_source(pid),
            exit: None,
            complete: Queue::with_capacity(1),
            finished: false,
        }
    }
    pub(super) fn finish(&mut self, sim: &mut Sim, exit: Exit) {
        self.exit = Some(exit);
        let mut submit = Queue::with_capacity(1);
        submit.push(Submit { op: Token::new(1), kind: Op::Close { fd: self.signals } });
        sim.submit(self.pid, &mut submit);
    }
    pub(super) fn iterate(&mut self, sim: &mut Sim) {
        if self.finished {
            return;
        }
        sim.reap(self.pid, &mut self.complete);
        if let Some(complete) = self.complete.pop() {
            assert_eq!(complete.result, Ok(io::kernel::Done::Nothing));
            sim.finish_service(self.pid, self.exit.take().expect("asked end"));
            sim.assert_no_open_fds(self.pid);
            sim.assert_quiescent(self.pid);
            self.finished = true;
        }
    }
    pub(super) fn pending(&self, sim: &Sim) -> bool {
        !self.finished && (sim.ready(self.pid) > 0 || sim.deferred(self.pid))
    }
}

struct Output {
    number: u32,
    token: Token,
    queued: VecDeque<Box<[u8]>>,
    waiting: bool,
    ending: bool,
    closing: bool,
}

#[expect(clippy::struct_excessive_bools, reason = "the fake's program, close state and held stop vary independently")]
pub(super) struct Child {
    pub(super) pid: Pid,
    pub(super) boxed: bool,
    io: io::Io,
    env: Env<io::Limits>,
    complete: Queue<Complete>,
    submit: Queue<Submit>,
    told: Queue<io::Event>,
    input: Option<Token>,
    output: Vec<Output>,
    closing: bool,
    pub(super) finished: bool,
    pub(super) hold_stop: bool,
}
impl Child {
    pub(super) fn new(sim: &mut Sim, parent: Pid, pidfd: Fd, boxed: bool, hand: bool) -> Self {
        let (pid, pipes) = sim.bind_service(parent, pidfd);
        let limits = io::Limits { sockets: 6, intake: 8192, receive: 8192, output: 8192, ..io::Limits::TEST };
        let mut io = io::Io::new(&limits);
        let mut input = None;
        let mut output = Vec::new();
        for (number, fd) in pipes {
            if number == 0 {
                input = Some(io.adopt_read_pipe(fd).expect("child stdin"));
            } else {
                output.push(Output {
                    number,
                    token: io.adopt_write_pipe(fd, None).expect("child output"),
                    queued: VecDeque::new(),
                    waiting: false,
                    ending: !boxed || (number == 3 && hand),
                    closing: false,
                });
            }
        }
        let mut child = Self {
            pid,
            boxed,
            io,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            complete: Queue::with_capacity(256),
            submit: Queue::with_capacity(256),
            told: Queue::uncapped(),
            input,
            output,
            closing: !boxed,
            finished: false,
            hold_stop: false,
        };
        if boxed {
            if hand {
                child.write(3, HANDED_LINK);
            }
            child.down(io::Request::Stream {
                stream: input.expect("box life"),
                down: stream::Down::Demand { read: stream::Read::Arrived { max: 1 }, room: 0 },
            });
        } else {
            child.write(1, b"fetched temper-box 3 of 3 bytes\nfound /bin/temper-box\n");
            child.down(io::Request::Abort { entity: input.expect("fetch null stdin") });
        }
        child
    }
    fn down(&mut self, request: io::Request) {
        io::down(&mut self.io, &self.env, request, &mut self.submit);
    }
    pub(super) fn write(&mut self, number: u32, bytes: &[u8]) {
        let output = self.output.iter_mut().find(|output| output.number == number).expect("open child output");
        for chunk in bytes.chunks(8192) {
            output.queued.push_back(chunk.into());
        }
    }
    pub(super) fn end(&mut self, number: u32) {
        self.output.iter_mut().find(|output| output.number == number).expect("output to end").ending = true;
    }
    pub(super) fn close(&mut self) {
        self.closing = true;
        for output in &mut self.output {
            output.ending = true;
        }
        if let Some(entity) = self.input {
            self.down(io::Request::Abort { entity });
        }
    }
    pub(super) fn iterate(&mut self, sim: &mut Sim, now: Time) {
        if self.finished {
            return;
        }
        if !sim.service_running(self.pid) {
            sim.reap_exited_service(self.pid, &mut self.complete);
            self.complete.clear();
            sim.close_exited_service(self.pid);
            self.finished = true;
            return;
        }
        self.env.now = now;
        sim.reap(self.pid, &mut self.complete);
        io::stage::up(&mut self.io, &self.env, &mut self.complete, &mut self.told, &mut self.submit, now);
        while let Some(event) = self.told.pop() {
            match event {
                io::Event::Stream { owner, up: stream::Up::Room } => {
                    let output = self.output.iter_mut().find(|output| output.token == owner).expect("output room");
                    output.waiting = false;
                    let bytes = output.queued.pop_front().expect("asked room for queued output");
                    self.down(io::Request::Stream { stream: owner, down: stream::Down::Send(bytes) });
                }
                io::Event::Stream { up: stream::Up::End | stream::Up::Failed(_), .. } => {
                    if !self.hold_stop {
                        self.close();
                    }
                }
                io::Event::Closed { owner } => {
                    if self.input == Some(owner) {
                        self.input = None;
                    }
                    self.output.retain(|output| output.token != owner);
                }
                io::Event::Shutdown { .. } => self.close(),
                io::Event::Stream { up: stream::Up::Bytes(_), .. }
                | io::Event::Output { .. }
                | io::Event::Failed { .. }
                | io::Event::Listening { .. }
                | io::Event::Accepted { .. }
                | io::Event::Connecting { .. }
                | io::Event::Connected { .. }
                | io::Event::Spawned { .. }
                | io::Event::Exited { .. }
                | io::Event::Usage { .. }
                | io::Event::Window { .. }
                | io::Event::Resized => panic!("unexpected fake child event: {event:?}"),
            }
        }
        let mut requests = Vec::new();
        for output in &mut self.output {
            if output.closing || output.waiting {
                continue;
            }
            if let Some(bytes) = output.queued.front() {
                output.waiting = true;
                requests.push(io::Request::Stream {
                    stream: output.token,
                    down: stream::Down::Demand {
                        read: stream::Read::Nothing,
                        room: u32::try_from(bytes.len()).expect("chunk size"),
                    },
                });
            } else if output.ending {
                output.closing = true;
                requests.push(io::Request::Close { entity: output.token });
            }
        }
        for request in requests {
            self.down(request);
        }
        self.io.reclaim();
        sim.submit(self.pid, &mut self.submit);
        if self.closing && self.io.is_empty() && self.complete.is_empty() && self.submit.is_empty() {
            sim.finish_service(self.pid, Exit::Code(0));
            sim.assert_no_open_fds(self.pid);
            sim.assert_quiescent(self.pid);
            self.finished = true;
        }
    }
    pub(super) fn pending(&self, sim: &Sim) -> bool {
        !self.finished
            && (!self.complete.is_empty()
                || !self.told.is_empty()
                || !self.submit.is_empty()
                || self.io.is_ready()
                || sim.ready(self.pid) > 0
                || sim.deferred(self.pid)
                || self
                    .output
                    .iter()
                    .any(|output| !output.waiting && !output.closing && (!output.queued.is_empty() || output.ending)))
    }
}
