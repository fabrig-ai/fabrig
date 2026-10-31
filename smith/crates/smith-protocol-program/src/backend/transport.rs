//! Native child and pipe progress (smith/docs/design/protocol/programs.md, 5.5–5.7).
//! Keeps transport cursors and bounded stderr; the startup and conversation
//! handlers own admission and completion. IO reports all child and pipe ends.

use super::{Backend, Below, Owner, PIPE_BYTES, Pipes, State};
use alloc::boxed::Box;
use skein_io as io;
use skein_lib::{Env, Queue, Token, stream};
use smith_protocol_llm::{self as protocol, ToDomain};

impl Backend {
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive entry routes child and pipe terminals before the shared progress pass"
    )]
    pub(super) fn io_event(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        event: io::Event,
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        let owner = event.owner().expect("backend has no signals");
        if let Some(check) = &mut self.check
            && check.owns(owner)
        {
            check.from_io(event, &mut self.io);
            return;
        }
        if let Some(server) = &mut self.server
            && server.owns(owner)
        {
            server.from_io(env.now, event, &mut self.io);
            return;
        }
        match event {
            io::Event::Spawned { child, pipes, .. } => {
                self.process.spawned(child, Pipes::from_spawn(&pipes));
                self.interrupt.restart(&mut self.io);
                self.read_stdout();
                if let Some(pipe) = self.process.pipes().errors {
                    Self::demand(pipe, PIPE_BYTES, &mut self.io);
                }
                if self.state == State::Ending {
                    self.io.push(io::Request::Close { entity: self.process.pipes().input.expect("stdin") });
                    if self.stopping > 0
                        || match self.deadline {
                            Some(deadline) => env.now >= deadline,
                            None => false,
                        }
                    {
                        self.stopping = 0;
                        self.terminate(env.now);
                    }
                } else {
                    self.send_input();
                }
            }
            io::Event::Exited { exit, .. } => {
                self.process.exited(exit, env.now.saturating_add(self.settings.grace), &mut self.io);
            }
            io::Event::Closed { owner } => {
                self.process.closed(owner, self.token(Owner::Program));
                if self.process.settled() && self.state != State::Ending && !self.switching {
                    let detail = self.error_tail();
                    let failure = self.exit_failure();
                    self.fail(env, failure, &detail, up, below);
                }
            }
            io::Event::Failed { owner, .. } => {
                if owner == self.token(Owner::Program) {
                    self.process.failed();
                }
                self.fail_program(env, b"smith: the program could not start", up, below);
            }
            io::Event::Stream { owner, up: stream::Up::Bytes(bytes) } => {
                if self.process.pipes().errors == Some(owner) {
                    self.keep_error(&bytes);
                    Self::demand(owner, PIPE_BYTES, &mut self.io);
                } else {
                    self.stdout_demanding = false;
                    self.completion_deadline(env, false, up);
                    assert!(self.intake.is_empty(), "one outstanding stdout read");
                    for byte in bytes {
                        self.intake.push(byte).expect("stdout demand is 4096 bytes");
                    }
                    self.consume_stdout(env, up, below);
                }
            }
            io::Event::Stream { owner, up: stream::Up::End | stream::Up::Failed(_) } => {
                self.io.push(io::Request::Close { entity: owner });
            }
            io::Event::Stream { up: stream::Up::Room, .. } => {}
            io::Event::Output { up: stream::OutputUp::Settled { right, outcome }, .. } => {
                if self.state != State::Ending || self.stop == super::stop::Stop::Parking {
                    match outcome {
                        stream::OutputOutcome::Granted => {
                            if self.process.exit().is_some() || self.process.pipes().input.is_none() {
                                self.fail_program(env, b"smith: the program input closed", up, below);
                                return;
                            }
                            if right == super::INTERRUPT_RIGHT {
                                self.interrupt.send(self.process.pipes().input.expect("interrupt write"), &mut self.io);
                            } else if let Some(bytes) = &self.outgoing {
                                let start = usize::try_from(self.written).expect("input cursor");
                                let end = start
                                    .checked_add(usize::try_from(PIPE_BYTES).expect("pipe chunk fits usize"))
                                    .expect("input chunk")
                                    .min(bytes.len());
                                let chunk = Box::from(bytes.get(start..end).expect("input range"));
                                self.written = u32::try_from(end).expect("input ceiling");
                                let finished = end == bytes.len();
                                self.io.push(io::Request::Output {
                                    stream: self.process.pipes().input.expect("input write"),
                                    down: stream::OutputDown::Send { right, bytes: chunk },
                                });
                                if finished {
                                    self.outgoing = None;
                                    self.written = 0;
                                    self.remember_session();
                                    self.send_interrupt();
                                } else {
                                    self.send_input();
                                }
                            }
                        }
                        stream::OutputOutcome::Cancelled | stream::OutputOutcome::Failed(_) => {
                            self.fail_program(env, b"smith: the program input failed", up, below);
                        }
                    }
                }
            }
            io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Usage { .. }
            | io::Event::Window { .. }
            | io::Event::Resized
            | io::Event::Shutdown { .. } => unreachable!("program event route"),
        }
    }

    pub(super) fn demand(pipe: Token, max: u32, below: &mut Queue<io::Request>) {
        below.push(io::Request::Stream {
            stream: pipe,
            down: stream::Down::Demand { read: stream::Read::Arrived { max }, room: 0 },
        });
    }

    pub(super) fn read_stdout(&mut self) {
        if !self.stdout_demanding
            && self.intake.is_empty()
            && let Some(pipe) = self.process.pipes().output
        {
            Self::demand(pipe, PIPE_BYTES, &mut self.io);
            self.stdout_demanding = true;
        }
    }

    pub(super) fn send_input(&mut self) {
        if let Some(bytes) = &self.outgoing {
            self.io.push(io::Request::Output {
                stream: self.process.pipes().input.expect("spawned input"),
                down: stream::OutputDown::Room {
                    right: super::INPUT_RIGHT,
                    bytes: u32::try_from(bytes.len())
                        .expect("bounded input")
                        .checked_sub(self.written)
                        .expect("remaining input")
                        .min(PIPE_BYTES),
                },
            });
        }
    }

    fn keep_error(&mut self, bytes: &[u8]) {
        let room = self.stderr.capacity().checked_sub(self.stderr.len()).expect("bounded tail");
        if self.stderr.capacity() > 0 && bytes.len() > usize::try_from(room).expect("tail capacity fits usize") {
            self.stderr_wrapped = true;
        }
        self.stderr.put(bytes);
    }

    fn error_tail(&self) -> Box<[u8]> {
        self.stderr.to_boxed()
    }
}
