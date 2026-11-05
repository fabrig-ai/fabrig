//! Optional owner captures (smith/docs/design/protocol/agent.status.md).
//! Keeps selected charter facts and rendered inputs, never grants or environments.
//! `keep_dumps` enables capture; `pop_dump` drains starts, requests, then programs.

use alloc::boxed::Box;
use skein_lib::{Duration, List, Queue};
use smith_domain::run::{Budget, charter};
use smith_protocol_program::Given;

use super::{Service, Stages, program};

/// One thing a run sent or was given, for its asking owner.
#[derive(Debug)]
pub struct Dump {
    pub identity: Option<[u8; 16]>,
    pub activation: u64,
    pub what: Dumped,
}

/// One start, rendered model request or program input.
#[derive(Debug)]
pub enum Dumped {
    /// Selected facts supplied by the Start's charter.
    Started(Started),
    /// One unchanged request body, before compression.
    Request { provider: skein_llm::Provider, body: Box<[u8]> },
    /// One input or initialization line of a program.
    Program(Given),
}

/// The model name and effort supplied by the host.
#[derive(Debug)]
pub struct Model {
    pub name: Box<[u8]>,
    pub effort: Option<Box<[u8]>>,
}

/// Owned charter facts needed by start.txt, without instructions or credentials.
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent host policy facts copied for the dump, not lifecycle flags"
)]
#[derive(Debug)]
pub struct Started {
    pub mode: charter::Mode,
    pub resume: bool,
    pub waiting: Duration,
    pub budget: Budget,
    pub reserve: charter::Reserve,
    pub main: Model,
    pub models: Box<[Model]>,
    pub tools: charter::Tools,
    pub agents: bool,
    pub wait: bool,
    pub deliver: bool,
    pub host_tools: Box<[Box<[u8]>]>,
    pub titles: Box<[Box<[u8]>]>,
}

#[derive(Debug)]
pub(super) struct Kept {
    identity: Option<[u8; 16]>,
    activation: u64,
    starts: Queue<Dump>,
}

impl Service {
    /// Asks for later LLM and program captures, including future children.
    pub fn keep_dumps(&mut self) {
        if self.stages.dumps.is_none() {
            self.stages.dumps = Some(Kept { identity: None, activation: 0, starts: Queue::uncapped() });
            self.stages.llm.keep_bodies();
            program::keep_given(&mut self.stages);
        }
    }

    /// Takes starts before requests, then program inputs.
    pub fn pop_dump(&mut self) -> Option<Dump> {
        let kept = self.stages.dumps.as_mut()?;
        if let Some(start) = kept.starts.pop() {
            return Some(start);
        }
        let what = match self.stages.llm.pop_body() {
            Some(body) => Dumped::Request { provider: body.provider, body: body.bytes },
            None => Dumped::Program(program::pop_given(&mut self.stages)?),
        };
        let kept = self.stages.dumps.as_ref().expect("capture enabled");
        Some(Dump { identity: kept.identity, activation: kept.activation, what })
    }
}

pub(super) fn started(stages: &mut Stages, identity: Option<[u8; 16]>, activation: u64, charter: &charter::Charter) {
    let Some(kept) = &mut stages.dumps else { return };
    kept.identity = identity;
    kept.activation = activation;
    let mut models = List::with_capacity(u32::try_from(charter.models.len()).expect("admitted model count"));
    for model in &charter.models {
        models.push(Model { name: model.model.clone(), effort: model.effort.clone() }).expect("model count");
    }
    let mut host_tools = List::with_capacity(u32::try_from(charter.grants.host_tools.len()).expect("admitted tools"));
    for tool in &charter.grants.host_tools {
        host_tools.push(tool.name.clone()).expect("tool count");
    }
    let mut titles = List::with_capacity(u32::try_from(charter.brief.sections.len()).expect("admitted sections"));
    for section in &charter.brief.sections {
        titles.push(section.title.clone()).expect("section count");
    }
    kept.starts.push(Dump {
        identity,
        activation,
        what: Dumped::Started(Started {
            mode: charter.mode,
            resume: charter.resume,
            waiting: charter.waiting,
            budget: charter.budget,
            reserve: charter.reserve,
            main: Model { name: charter.llm.model.clone(), effort: charter.llm.effort.clone() },
            models: models.into_boxed(),
            tools: charter.grants.tools,
            agents: charter.grants.agents,
            wait: charter.grants.wait,
            deliver: charter.grants.deliver.is_some(),
            host_tools: host_tools.into_boxed(),
            titles: titles.into_boxed(),
        }),
    });
}
