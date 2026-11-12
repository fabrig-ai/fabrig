//! Child pipes; session policy lives in `drive::Machine` (client/drive.md, 5).
use super::super::{Event, Input, Output, Process, Program};
use skein_fake_person_browser::harness::session::Pump;
use skein_io::{
    self as io,
    kernel::{Exit, Pipe, Signal, Spawn, Target, Way},
};
use skein_lib::{
    Token,
    stream::{self, Down, Up},
};
use std::{collections::VecDeque, os::unix::ffi::OsStrExt};
pub(super) struct Child {
    pub process: Process,
    pub owner: Token,
    identity: Option<Token>,
    pipes: Vec<Token>,
    input: Option<VecDeque<Box<[u8]>>>,
    output: [Vec<u8>; 2],
    fragments: [Vec<u8>; 2],
    ended: [bool; 2],
    exit: Option<i32>,
    notified: bool,
    stopping: bool,
    pub settled: bool,
}
impl Child {
    pub fn start(process: Process, owner: Token, program: Program, pump: &mut Pump) -> Self {
        let input = match program.input {
            Input::Null => None,
            Input::Bytes(bytes) => Some(bytes.chunks(32768).map(Box::from).collect()),
        };
        let mut pipes = vec![];
        if input.is_some() {
            pipes.push(Pipe { child: 0, way: Way::In, parent: None });
        }
        pipes.extend([Pipe { child: 1, way: Way::Out, parent: None }, Pipe { child: 2, way: Way::Out, parent: None }]);
        let spawn = Spawn {
            program: program.program.as_os_str().as_bytes().into(),
            args: program.args.iter().map(|arg| arg.as_bytes().into()).collect(),
            env: program
                .environment
                .iter()
                .map(|(key, value)| [key.as_bytes(), b"=", value.as_bytes()].concat().into())
                .collect(),
            root: pump.root(),
            dir: program.directory.as_os_str().as_bytes().into(),
            pipes: pipes.into(),
        };
        pump.request(io::Request::Spawn { owner, spawn });
        Self {
            process,
            owner,
            identity: None,
            pipes: vec![],
            input,
            output: [vec![], vec![]],
            fragments: [vec![], vec![]],
            ended: [false; 2],
            exit: None,
            notified: false,
            stopping: false,
            settled: false,
        }
    }
    pub fn owns(&self, owner: Token) -> bool {
        self.owner == owner || self.pipes.contains(&owner)
    }
    pub fn stop(&mut self, pump: &mut Pump, force: bool) {
        self.stopping = true;
        if let Some(child) = self.identity {
            pump.request(if force {
                io::Request::Close { entity: child }
            } else {
                io::Request::Signal { child, signal: Signal::Terminate, to: Target::Child }
            });
            if force {
                for entity in &self.pipes {
                    pump.request(io::Request::Close { entity: *entity });
                }
            }
        }
    }
    fn send(&mut self, pump: &mut Pump) {
        let Some(input) = &self.input else {
            return;
        };
        let stream = self.pipes[0];
        if let Some(bytes) = input.front() {
            pump.request(io::Request::Stream {
                stream,
                down: Down::Demand {
                    read: stream::Read::Nothing,
                    room: u32::try_from(bytes.len()).expect("input chunk fits"),
                },
            });
        } else {
            pump.request(io::Request::Stream { stream, down: Down::Finish });
        }
    }
    fn read(stream: Token, pump: &mut Pump) {
        pump.request(io::Request::Stream {
            stream,
            down: Down::Demand { read: stream::Read::Arrived { max: 32768 }, room: 0 },
        });
    }
    pub fn event(&mut self, event: io::Event, pump: &mut Pump) -> Vec<Event> {
        let mut events = vec![];
        match event {
            io::Event::Spawned { child, pipes, .. } => {
                self.identity = Some(child);
                self.pipes = pipes.into_vec();
                self.send(pump);
                for pipe in self.pipes.iter().skip(usize::from(self.input.is_some())) {
                    Self::read(*pipe, pump);
                }
                if self.stopping {
                    self.stop(pump, true);
                }
            }
            io::Event::Exited { exit, .. } => {
                self.exit = Some(match exit {
                    Exit::Code(code) => i32::from(code),
                    Exit::Signal(signal) => -i32::try_from(signal).expect("signal fits"),
                });
            }
            io::Event::Stream { owner, up } => {
                let index = self.pipes.iter().position(|pipe| *pipe == owner).expect("owned pipe");
                if self.input.is_some() && index == 0 {
                    match up {
                        Up::Room => {
                            if let Some(bytes) = self.input.as_mut().expect("input pipe").pop_front() {
                                pump.request(io::Request::Stream { stream: owner, down: Down::Send(bytes) });
                            }
                            self.send(pump);
                        }
                        Up::End | Up::Failed(_) => pump.request(io::Request::Close { entity: owner }),
                        Up::Bytes(_) => {}
                    }
                } else {
                    let at = index - usize::from(self.input.is_some());
                    match up {
                        Up::Bytes(bytes) => {
                            if self.process == Process::Box {
                                self.fragments[at].extend_from_slice(&bytes);
                                while let Some(end) = self.fragments[at].iter().position(|byte| *byte == b'\n') {
                                    events.push(Event::BoxLine {
                                        output: if at == 0 { Output::Stdout } else { Output::Stderr },
                                        line: self.fragments[at].drain(..=end).collect(),
                                    });
                                }
                            } else {
                                self.output[at].extend_from_slice(&bytes);
                            }
                            Self::read(owner, pump);
                        }
                        Up::End | Up::Failed(_) => {
                            if !self.fragments[at].is_empty() {
                                events.push(Event::BoxLine {
                                    output: if at == 0 { Output::Stdout } else { Output::Stderr },
                                    line: std::mem::take(&mut self.fragments[at]).into(),
                                });
                            }
                            self.ended[at] = true;
                            pump.request(io::Request::Close { entity: owner });
                        }
                        Up::Room => {}
                    }
                }
            }
            io::Event::Failed { error, .. } => {
                self.exit = Some(127);
                self.ended = [true; 2];
                self.output[1].extend_from_slice(format!("could not start {:?}: {error:?}", self.process).as_bytes());
            }
            io::Event::Closed { owner } if owner == self.owner => self.settled = true,
            _ => {}
        }
        if let Some(code) = self.exit
            && self.ended.iter().all(|ended| *ended)
            && !self.notified
        {
            self.notified = true;
            if let Some(child) = self.identity {
                pump.request(io::Request::Close { entity: child });
            }
            events.push(Event::Exited {
                process: self.process,
                code,
                stdout: std::mem::take(&mut self.output[0]).into(),
                stderr: std::mem::take(&mut self.output[1]).into(),
            });
        }
        events
    }
}
