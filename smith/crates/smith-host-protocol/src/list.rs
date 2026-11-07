//! Reading an endpoint's list through the agent (smith/docs/design/protocol/hosts.md, 2;
//! smith/docs/design/protocol/agent.md, 6.1). Keeps one child command and a single answer
//! right; holds the credential until it is written. `new`, `from_io` and `fire` drive
//! the command, and `answer` translates its settled record into the host's vocabulary.

use alloc::boxed::Box;
use skein_io::{self as io, kernel};
use skein_lib::{List, Queue, Reader, Secret, Time, Token, Writer};
use smith_channel::{self as wire, CEILINGS, CredentialValue as Credential};
use smith_host_domain::{ListFailure, ListOutcome, ListedModel};
use smith_protocol_program::{self as program, Command, Ended};

/// The agent binding supplied by the host; the half prefixes its arguments with `--list`.
#[derive(Clone, Debug)]
pub struct ListLaunch {
    pub program: Box<[u8]>,
    pub arguments: Box<[Box<[u8]>]>,
    pub environment: Box<[Box<[u8]>]>,
    pub root: kernel::Fd,
    pub directory: Box<[u8]>,
}

/// One host list request, answered once after the child and all its pipes settle.
#[derive(Debug)]
pub struct ListRead {
    command: Command,
    answered: bool,
    cancelled: bool,
}

impl ListRead {
    /// Spawn the agent and write one credential-bearing ask on stdin alone.
    pub fn new(
        io_owner: Token,
        launch: ListLaunch,
        endpoint: u32,
        credential: Credential,
        deadline: Time,
        below: &mut Queue<io::Request>,
    ) -> Result<Self, program::Error> {
        let ask = wire::ListAsk::new(&CEILINGS, wire::ListAskParts { endpoint, credential: Some(credential) })
            .expect("validated channel credential");
        let mut writer = Writer::new(usize::try_from(ask.measure()).expect("bounded list ask"));
        ask.encode(&mut writer).expect("measured list ask");
        let input = Secret::new(writer.finish());
        let mut arguments = List::with_capacity(
            u32::try_from(launch.arguments.len()).expect("agent arguments fit u32").checked_add(1).expect("list flag"),
        );
        arguments.push(Box::from(b"--list".as_slice())).expect("list flag slot");
        for argument in launch.arguments {
            arguments.push(argument).expect("agent argument slot");
        }
        let launch = program::Launch {
            program: launch.program,
            arguments: arguments.into_boxed(),
            environment: launch.environment,
            root: launch.root,
            directory: launch.directory,
        };
        let output_bytes = u32::try_from(wire::ListAnswer::encoded_bound(&CEILINGS).expect("list answer bound"))
            .expect("list answer fits u32");
        Ok(Self {
            command: Command::with_input(io_owner, launch, input, output_bytes, deadline, 0, below),
            answered: false,
            cancelled: false,
        })
    }

    /// Whether this I/O owner belongs to the list command.
    #[must_use]
    pub fn owns(&self, owner: Token) -> bool {
        self.command.owns(owner)
    }

    /// Drive one child or pipe event, retaining closure evidence.
    pub fn from_io(&mut self, event: io::Event, below: &mut Queue<io::Request>) {
        self.command.from_io(event, below);
    }

    /// Enforce the host deadline while the child is active.
    pub fn fire(&mut self, now: Time, below: &mut Queue<io::Request>) {
        self.command.fire(now, below);
    }

    /// Withdraw the answer right and settle the command's resources.
    pub fn cancel(&mut self, below: &mut Queue<io::Request>) {
        self.cancelled = true;
        self.command.cancel(below);
    }

    /// The command's next host deadline, absent while settling or after closure.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.command.next_deadline()
    }

    /// Take one settled answer; a cancellation leaves no answer for the host.
    pub fn answer(&mut self) -> Option<ListOutcome> {
        if self.answered || self.cancelled {
            return None;
        }
        let ended = self.command.ended()?;
        let answer = translate(ended);
        self.answered = true;
        Some(answer)
    }
}

fn translate(ended: Ended<'_>) -> ListOutcome {
    let stdout = match ended {
        Ended::Deadline { .. } => return ListOutcome::Failed(ListFailure::Deadline),
        Ended::Unspawned { .. } => return ListOutcome::Failed(ListFailure::Agent),
        Ended::Exited { exit, stdout, .. } => match exit {
            kernel::Exit::Code(0) => match stdout {
                Some(stdout) => stdout,
                None => return ListOutcome::Failed(ListFailure::Agent),
            },
            kernel::Exit::Code(_) | kernel::Exit::Signal(_) => return ListOutcome::Failed(ListFailure::Agent),
        },
    };
    let Ok(answer) = wire::ListAnswer::decode(&CEILINGS, &mut Reader::new(stdout)) else {
        return ListOutcome::Failed(ListFailure::Agent);
    };
    match answer {
        wire::ListAnswer::Listed(rows) => {
            let mut models = List::with_capacity(rows.rows().len());
            for row in rows.into_parts().rows.into_boxed() {
                let row = row.into_parts();
                models
                    .push(ListedModel {
                        name: row.name,
                        shown: row.shown,
                        window: row.window,
                        output: row.output,
                        efforts: row.efforts.into_boxed(),
                    })
                    .expect("decoded model count");
            }
            ListOutcome::Listed(models.into_boxed())
        }
        wire::ListAnswer::Failed(failure) => ListOutcome::Failed(match failure.into_parts().failure {
            wire::ListFailed::Refused(refused) => ListFailure::Refused { status: refused.status() },
            wire::ListFailed::Unreachable => ListFailure::Unreachable,
            wire::ListFailed::Unreadable => ListFailure::Unreadable,
            wire::ListFailed::Deadline => ListFailure::Deadline,
        }),
    }
}
