//! One git child, held until its exit and all its resources close (hearth/docs/design/domain/render.md, 2.2).
use crate::{Asked, git_machine::invocation, resource::Resource};
use alloc::boxed::Box;
use skein_io::{
    self as io,
    kernel::{self, Fd, Pipe, Spawn, Way},
};
use skein_lib::{Queue, Token, bytes, stream::Down};
#[derive(Debug)]
pub(crate) struct Child {
    pub owner: Token,
    pub process: Resource,
    pub output: Resource,
    pub error: Resource,
    pub tail: crate::git_machine::tail::Tail,
    pub output_state: Intake,
    pub error_state: Intake,
    pub command: crate::render::errors::Command,
    pub fault: Option<crate::render::errors::Ended>,
    pub input: Resource,
    pub exit: Option<invocation::Exit>,
    pub failed: bool,
    pub stage: Stage,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Stage {
    Live,
    Closing,
}
impl Child {
    pub fn spawn(
        owner: Token,
        root: Fd,
        command: invocation::Command<'_>,
        limits: &crate::git_machine::limits::Limits,
        below: &mut Queue<Asked>,
    ) -> Self {
        let pipes: Box<[Pipe]> = match command {
            invocation::Command::CatFile { .. } => Box::new([
                Pipe { child: 1, way: Way::Out, parent: None },
                Pipe { child: 0, way: Way::In, parent: None },
                Pipe { child: 2, way: Way::Out, parent: None },
            ]),
            invocation::Command::RevParse { .. }
            | invocation::Command::LsTree { .. }
            | invocation::Command::Diff { .. } => Box::new([
                Pipe { child: 1, way: Way::Out, parent: None },
                Pipe { child: 2, way: Way::Out, parent: None },
            ]),
        };
        let named = match command {
            invocation::Command::RevParse { .. } => crate::render::errors::Command::RevParse,
            invocation::Command::LsTree { .. } => crate::render::errors::Command::LsTree,
            invocation::Command::CatFile { .. } => crate::render::errors::Command::CatFile,
            invocation::Command::Diff { .. } => crate::render::errors::Command::Diff,
        };
        below.push(Asked::Io(io::Request::Spawn {
            owner,
            spawn: Spawn {
                program: bytes::copy_of(invocation::GIT),
                args: invocation::arguments(command),
                env: invocation::environment(),
                root,
                dir: Box::new([]),
                pipes,
            },
        }));
        Self {
            owner,
            process: Resource::Awaited,
            output: Resource::Awaited,
            error: Resource::Awaited,
            tail: crate::git_machine::tail::Tail::new(limits),
            output_state: Intake::Waiting,
            error_state: Intake::Demand,
            command: named,
            fault: None,
            input: Resource::Awaited,
            exit: None,
            failed: false,
            stage: Stage::Live,
        }
    }
    pub fn spawned(&mut self, child: Token, pipes: &[Token]) {
        self.process = Resource::Open(child);
        let resources: &mut [&mut Resource] = match self.command {
            crate::render::errors::Command::CatFile => &mut [&mut self.output, &mut self.input, &mut self.error],
            crate::render::errors::Command::RevParse
            | crate::render::errors::Command::LsTree
            | crate::render::errors::Command::Diff => &mut [&mut self.output, &mut self.error],
            crate::render::errors::Command::Mmdebstrap => unreachable!("git command"),
        };
        for (held, &pipe) in resources.iter_mut().zip(pipes) {
            **held = Resource::Open(pipe);
        }
    }
    pub fn refused(&mut self) {
        self.failed = true;
        self.error_state = Intake::Ended;
        self.output_state = Intake::Ended;
    }
    pub fn exited(&mut self, exit: kernel::Exit) {
        self.exit = Some(match exit {
            kernel::Exit::Code(code) => invocation::Exit::Code(code),
            kernel::Exit::Signal(signal) => invocation::Exit::Signal(signal),
        });
    }
    pub fn closed(&mut self, owner: Token) {
        if owner == self.owner {
            self.process = Resource::Closed;
        }
        if self.output.token() == Some(owner) {
            self.output = Resource::Closed;
        }
        if self.error.token() == Some(owner) {
            self.error = Resource::Closed;
        }
        if self.input.token() == Some(owner) {
            self.input = Resource::Closed;
        }
    }
    pub fn done(&self) -> bool {
        self.process == Resource::Closed
            && self.output.token().is_none()
            && self.input.token().is_none()
            && self.error.token().is_none()
    }
    pub fn stream(&mut self, down: Down, below: &mut Queue<Asked>) {
        if let Down::Demand { read, .. } = &down
            && self.output_state != Intake::Ended
            && self.output_state != Intake::Draining
        {
            self.output_state =
                if *read == skein_lib::stream::Read::Nothing { Intake::Waiting } else { Intake::Reading };
        }
        below.push(Asked::Io(io::Request::Stream { stream: self.output.token().expect("spawned stdout"), down }));
    }
    pub fn resume(&mut self, below: &mut Queue<Asked>) -> bool {
        if self.stage != Stage::Closing {
            return false;
        }
        for held in [&mut self.output, &mut self.error, &mut self.input, &mut self.process] {
            if let Some(entity) = held.request_close() {
                below.push(Asked::Io(io::Request::Close { entity }));
                return true;
            }
        }
        false
    }
    pub fn work_pending(&self) -> bool {
        self.stage == Stage::Closing
            && (self.output.is_open() || self.error.is_open() || self.input.is_open() || self.process.is_open())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Intake {
    Waiting,
    Reading,
    Demand,
    Draining,
    Ended,
}
