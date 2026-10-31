//! Lazy child startup and private entropy-derived names (smith/docs/design/protocol/programs.md, 3, 5.1).
//! Keeps no login; the existing status reader classifies subscription use.

use super::Complete;
use super::Owner;
use super::{Backend, Below, State};
use crate::{Command, Ended, Launch, STATUS, Status, configuration_directory, environment, mcp};
use alloc::boxed::Box;
use skein_io::{self as io, kernel};
use skein_lib::{Decimal, Env, List, Queue, Secret, sha256};
use smith_domain::llm;
use smith_protocol_llm::{self as protocol, ToDomain};

pub(super) struct Names {
    pub mcp: Secret,
    pub run: [u8; 32],
}

impl Names {
    pub fn new(entropy: &Secret) -> Self {
        let mut mcp_hash = sha256::Sha256::new();
        mcp_hash.update(entropy.expose());
        mcp_hash.update(b"mcp");
        let mcp = Secret::new(Box::from(mcp_hash.finish().as_slice()));
        let mut run_hash = sha256::Sha256::new();
        run_hash.update(entropy.expose());
        run_hash.update(b"run");
        let run_hex = mcp::secret_text(&run_hash.finish());
        let mut run = [0; 32];
        for (target, source) in run.iter_mut().zip(run_hex) {
            *target = source;
        }
        Self { mcp, run }
    }
}

impl Backend {
    pub(super) fn start(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        request: Complete,
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        let (owner, name) = (request.owner, request.grant);
        if self.startup.phase() != State::Idle {
            self.current.claim(owner, up);
            self.fail(env, llm::Failure::Invalid, b"smith: a program conversation is already started", up, below);
            return;
        }
        self.begin_prompt(request.observation.is_some());
        self.current.claim(owner, up);
        let valid = match &self.grant {
            Some(grant) => match name {
                Some(name) => {
                    name.account == grant.name.account
                        && name.generation <= grant.name.generation
                        && env.now < grant.lapses
                }
                None => false,
            },
            None => false,
        };
        if !valid {
            self.fail(env, llm::Failure::SignedOut, b"smith: the program source grant is unavailable", up, below);
            return;
        }
        if self.files.prepared() {
            self.current.retain(request, up);
            self.startup.prepare(&mut self.io);
            self.prepare_invocation(env, up, below);
            return;
        }
        let root = match self.settings.workspace.or(self.settings.directory) {
            Some(root) if self.settings.directory.is_some() => root,
            Some(_) | None => {
                self.fail(env, llm::Failure::SignedOut, b"smith: the program source is not signed in", up, below);
                return;
            }
        };
        let Ok(configuration) =
            configuration_directory(&self.settings.program, &self.grant.as_ref().expect("validated grant").source)
        else {
            self.fail(env, llm::Failure::Invalid, b"smith: invalid program source", up, below);
            return;
        };
        let Ok(environment) = environment(&self.settings.environment, &configuration) else {
            self.fail(env, llm::Failure::Program { exit: None }, b"smith: invalid program environment", up, below);
            return;
        };
        self.current.retain(request, up);
        let mut arguments = List::with_capacity(2);
        for argument in STATUS {
            arguments.push(Box::from(argument)).expect("status arguments");
        }
        let check = Command::new(
            self.token(Owner::Status),
            Launch {
                program: self.settings.program.claude.clone(),
                arguments: arguments.into_boxed(),
                environment,
                root: root.descriptor,
                directory: Box::default(),
            },
            env.now.saturating_add(self.settings.startup),
            self.settings.receiving.max_failure_bytes,
            &mut self.io,
        );
        self.startup.checking(check, &mut self.io);
    }

    pub(super) fn progress(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        self.consume_stdout(env, up, below);
        self.checked(env, up, below);
        self.server_events(env, up, below);
        self.switch_progress(env, up, below);
        if self.stopping.intent() == super::stopping::Intent::YieldRequested {
            self.yield_for_wind_down(env, up, below);
        }
        if self.startup.phase() == State::Ending && self.process.child().is_none() && !self.process.spawning() {
            self.startup.close_server(&mut self.io);
        }
        if self.startup.phase() == State::Ending
            && self.startup.check().is_none()
            && self.process.settled()
            && (self.startup.server().is_none() || self.startup.server_ended())
        {
            self.cleanup(env.now, below);
        }
    }

    fn checked(&mut self, env: &Env<protocol::ComponentLimits>, up: &mut Queue<ToDomain>, below: &mut Queue<Below>) {
        let (status, exit, detail) = match match self.startup.check() {
            Some(check) => check.ended(),
            None => None,
        } {
            Some(Ended::Exited { exit: kernel::Exit::Code(code), stdout, detail }) => (
                match stdout {
                    Some(bytes) => crate::status(code, bytes),
                    None => Status::Unreadable,
                },
                Some(u32::from(code)),
                Box::<[u8]>::from(detail),
            ),
            Some(Ended::Exited { exit: kernel::Exit::Signal(signal), detail, .. }) => {
                (Status::Unreadable, Some(128_u32.checked_add(signal).expect("signal code")), detail.into())
            }
            Some(Ended::Unspawned { detail } | Ended::Deadline { detail }) => (Status::Unreadable, None, detail.into()),
            None => return,
        };
        self.startup.checked(&mut self.io);
        if self.startup.phase() == State::Ending {
            return;
        }
        match status {
            Status::Usable => {
                self.startup.prepare(&mut self.io);
                self.open_runs(env.now, below);
            }
            Status::NotSignedIn | Status::Otherwise(_) => self.fail(
                env,
                llm::Failure::SignedOut,
                b"smith: the program source is not signed in through a subscription",
                up,
                below,
            ),
            Status::Unreadable => self.fail(env, llm::Failure::Program { exit }, &detail, up, below),
        }
    }

    pub(super) fn server_events(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        for _ in 0_u32..2 {
            let event = match self.startup.server_mut() {
                Some(server) => server.up(),
                None => None,
            };
            match event {
                Some(mcp::Up::Listening { port }) => {
                    if self.startup.phase() == State::Starting {
                        self.store_instructions(env.now, port, below);
                    }
                }
                Some(mcp::Up::Connected) => {}
                Some(prompt @ mcp::Up::Prompt { .. }) => self.permission(prompt),
                Some(call @ mcp::Up::Call { .. }) => {
                    self.calls.push(call).expect("server bounds admitted calls");
                    self.match_calls();
                }
                Some(mcp::Up::Ended(_)) => {
                    self.mcp_ended(env, up, below);
                }
                None => break,
            }
        }
    }

    pub(super) fn launch(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        let Some(launch) = self.launch_spec() else {
            self.fail(
                env,
                llm::Failure::Program { exit: None },
                b"smith: the program command exceeds its bounds",
                up,
                below,
            );
            return;
        };
        if !self.compaction() && !self.startup.resuming() {
            self.first_prompt_measure(env.now);
        }
        self.io.push(io::Request::Spawn { owner: self.token(Owner::Program), spawn: launch });
        self.process.spawn();
        self.startup.launch(env.now.saturating_add(self.settings.startup), &mut self.io);
    }

    pub(super) fn mcp_ended(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        // Relay EOF can precede waitpid. Let the child and stderr settle
        // before choosing its program fault (smith/docs/design/protocol/programs.md, 5.6).
        let waiting = self.startup.phase() == State::Running && self.process.child().is_some();
        let until =
            if waiting && !self.startup.switching() { Some(env.now.saturating_add(self.settings.grace)) } else { None };
        self.startup.relay_ended(until, &mut self.io);
        if !self.startup.switching()
            && !waiting
            && self.startup.phase() != State::Ending
            && self.startup.phase() != State::Ended
        {
            self.fail_program(env, b"smith: the MCP server ended", up, below);
        }
    }

    pub(super) fn launch_spec(&mut self) -> Option<kernel::Spawn> {
        let prompt = &self.current.request()?.prompt;
        let outcome = &self.current.request()?.outcome;
        if !super::compact::is_compaction(prompt) {
            match &prompt.choice {
                llm::ToolChoice::Only(_) | llm::ToolChoice::None => {
                    self.stopping.intend(super::stopping::Intent::Finishing);
                }
                llm::ToolChoice::Auto => {}
            }
        }
        let schema = if self.result_contract() {
            let contract = protocol::finish_schema(outcome, self.settings.contract_bytes).ok()?;
            mcp::json::outcome_schema(&contract, self.settings.contract_bytes)?
        } else {
            Box::default()
        };
        let root = self.settings.workspace.or(self.settings.directory)?;
        let directory = if self.settings.workspace.is_some() { Box::default() } else { self.run_relative() };
        let configuration = configuration_directory(&self.settings.program, &self.grant.as_ref()?.source).ok()?;
        let instructions = self.run_path(b"instructions.md");
        let mcp_file = self.run_path(b"mcp.json");
        let mut arguments = if super::compact::is_compaction(prompt) {
            super::compact::arguments(prompt, &instructions, &self.startup.session())?
        } else {
            arguments(
                &prompt.model,
                prompt.effort.as_deref(),
                &instructions,
                &mcp_file,
                &schema,
                self.settings.max_turns,
                &self.startup.session(),
                &super::policy::tools_flag(prompt.tools),
            )?
        };
        if self.startup.resuming() {
            for argument in &mut arguments {
                if argument.as_ref() == b"--session-id" {
                    *argument = b"--resume".as_slice().into();
                }
            }
            self.resume_input()?;
        } else if super::compact::is_compaction(prompt) {
            self.compaction_input()?;
        } else {
            self.outgoing = super::stream::user_message(prompt, self.settings.input_bytes);
        }
        self.outgoing.as_ref()?;
        Some(kernel::Spawn {
            program: self.settings.program.claude.clone(),
            args: arguments,
            env: super::replies::environment(&self.settings.environment, &configuration, self.run_time)?,
            root: root.descriptor,
            dir: directory,
            pipes: Box::new([
                kernel::Pipe { child: 0, way: kernel::Way::In, parent: None },
                kernel::Pipe { child: 1, way: kernel::Way::Out, parent: None },
                kernel::Pipe { child: 2, way: kernel::Way::Out, parent: None },
            ]),
        })
    }
}

/// The program's exact argv, keeping inline schema and empty built-ins as whole arguments.
#[expect(clippy::too_many_arguments, reason = "each independent argv value remains a whole argument")]
pub(super) fn arguments(
    model: &[u8],
    effort: Option<&[u8]>,
    instructions: &[u8],
    mcp_file: &[u8],
    schema: &[u8],
    max_turns: u32,
    session: &[u8],
    tools: &[u8],
) -> Option<Box<[Box<[u8]>]>> {
    let turns = Decimal::of(u64::from(max_turns));
    let mut arguments = List::with_capacity(36);
    for flag in [
        b"-p".as_slice(),
        b"--input-format",
        b"stream-json",
        b"--output-format",
        b"stream-json",
        b"--verbose",
        b"--include-partial-messages",
        b"--setting-sources",
        b"user",
        b"--model",
    ] {
        arguments.push(Box::from(flag)).ok()?;
    }
    arguments.push(Box::from(model)).ok()?;
    if let Some(effort) = effort {
        arguments.push(Box::from(b"--effort".as_slice())).ok()?;
        arguments.push(Box::from(effort)).ok()?;
    }
    let permission = mcp::json::join(&[b"mcp__", mcp::SERVER, b"__", mcp::PROMPT_TOOL]);
    for argument in [
        b"--append-system-prompt-file".as_slice(),
        instructions,
        b"--mcp-config",
        mcp_file,
        b"--strict-mcp-config",
        b"--permission-prompt-tool",
        permission.as_ref(),
        b"--tools",
        tools,
        b"--max-turns",
        turns.as_bytes(),
        b"--session-id",
        session,
    ] {
        arguments.push(Box::from(argument)).ok()?;
    }
    if !schema.is_empty() {
        arguments.push(Box::from(b"--json-schema".as_slice())).ok()?;
        arguments.push(Box::from(schema)).ok()?;
    }
    Some(arguments.into_boxed())
}
