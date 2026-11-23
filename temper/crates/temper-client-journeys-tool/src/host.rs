//! Machine requests and answers for the journeys' runs (temper/docs/design/client/contracts.md, 1).

use std::path::PathBuf;

use crate::EnvironmentEntry;

mod real;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Ask {
    Run(Program),
    Directory(PathBuf),
    Read(PathBuf),
    Write(PathBuf, Box<[u8]>),
    Remove(PathBuf),
    Exists(PathBuf),
    Alive(u32),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Program {
    #[expect(clippy::struct_field_names, reason = "Program::program is the shared journey host contract")]
    pub(crate) program: Box<[u8]>,
    pub(crate) arguments: Vec<Box<[u8]>>,
    pub(crate) environment: Vec<EnvironmentEntry>,
    pub(crate) directory: Option<PathBuf>,
    pub(crate) stdout: Stream,
    pub(crate) stderr: Stream,
}

impl Program {
    pub(crate) fn new(program: &[u8], arguments: &[&[u8]]) -> Self {
        Self {
            program: program.into(),
            arguments: arguments.iter().map(|word| Box::from(*word)).collect(),
            environment: Vec::new(),
            directory: None,
            stdout: Stream::Keep,
            stderr: Stream::Err,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stream {
    Keep,
    Out,
    Err,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Answer {
    Ran(Ran),
    Bytes(Box<[u8]>),
    Is(bool),
    Done,
    Missing,
    Failed(String),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Ran {
    pub(crate) code: Option<i32>,
    pub(crate) stdout: Box<[u8]>,
    pub(crate) stderr: Box<[u8]>,
}

impl Ran {
    pub(crate) fn new(code: Option<i32>, stdout: &[u8]) -> Self {
        Self { code, stdout: stdout.into(), stderr: Box::default() }
    }
}

pub(crate) struct Host {
    kind: Kind,
}

enum Kind {
    Real(Box<[EnvironmentEntry]>),
    #[cfg(test)]
    Canned(Canned),
}

impl Host {
    pub(crate) fn real(environment: Box<[EnvironmentEntry]>) -> Self {
        Self { kind: Kind::Real(environment) }
    }

    pub(crate) fn ask(&mut self, ask: Ask) -> Answer {
        match &mut self.kind {
            Kind::Real(environment) => real::ask(environment, ask),
            #[cfg(test)]
            Kind::Canned(canned) => canned.ask(&ask),
        }
    }

    pub(crate) fn out(&mut self, bytes: &[u8]) {
        match &mut self.kind {
            Kind::Real(_) => real::out(bytes).expect("write the run's stdout"),
            #[cfg(test)]
            Kind::Canned(canned) => canned.gathered.stdout.extend_from_slice(bytes),
        }
    }

    pub(crate) fn err(&mut self, bytes: &[u8]) {
        match &mut self.kind {
            Kind::Real(_) => real::err(bytes).expect("write the run's stderr"),
            #[cfg(test)]
            Kind::Canned(canned) => canned.gathered.stderr.extend_from_slice(bytes),
        }
    }
}

#[cfg(test)]
pub(crate) struct Step {
    pub(crate) ask: Ask,
    pub(crate) answer: Answer,
}

#[cfg(test)]
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Gathered {
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
}

#[cfg(test)]
struct Canned {
    steps: std::vec::IntoIter<Step>,
    index: usize,
    gathered: Gathered,
}

#[cfg(test)]
impl Canned {
    fn ask(&mut self, ask: &Ask) -> Answer {
        let index = self.index;
        let answer = match self.steps.next() {
            Some(step) if &step.ask == ask => step.answer,
            Some(Step { ask: expected, .. }) => panic!("host step {index}: expected {expected:?}, got {ask:?}"),
            None => panic!("host step {index}: no request expected, got {ask:?}"),
        };
        self.index += 1;
        if let (Ask::Run(program), Answer::Ran(ran)) = (ask, &answer) {
            self.gather(program.stdout, &ran.stdout);
            self.gather(program.stderr, &ran.stderr);
        }
        answer
    }

    fn gather(&mut self, stream: Stream, bytes: &[u8]) {
        match stream {
            Stream::Keep => (),
            Stream::Out => self.gathered.stdout.extend_from_slice(bytes),
            Stream::Err => self.gathered.stderr.extend_from_slice(bytes),
        }
    }
}

#[cfg(test)]
impl Host {
    pub(crate) fn canned(steps: Vec<Step>) -> Self {
        Self { kind: Kind::Canned(Canned { steps: steps.into_iter(), index: 0, gathered: Gathered::default() }) }
    }

    pub(crate) fn finish(self) -> Gathered {
        let Kind::Canned(mut canned) = self.kind else { panic!("only a canned host gathers output") };
        if let Some(step) = canned.steps.next() {
            panic!("host step {} not taken: {:?}", canned.index, step.ask);
        }
        canned.gathered
    }
}

#[cfg(test)]
mod tests;
