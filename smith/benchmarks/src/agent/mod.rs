//! Agent identities and pinned configuration records; no login is read here.
//! `read_configuration` checks the pin and hashes its metadata and exact
//! configuration bytes (../../../docs/design/benchmarks/README.md, sections 5.4, 6 and 10).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::Refusal;
use crate::arm::ArmRef;
use crate::process::ProcessResult;
use crate::run::{Archive, Work};
use crate::{
    AttemptResult, CacheObservation, CheckVerdict, End, Exit, Fault, FaultKind, Forced, Identity, Measure, Task, input,
    io,
};

pub mod codex;

pub mod smith;

mod person;

pub use person::{Close, Cue, Script, Trigger};

pub(crate) use person::Person;

/// The frozen inputs an adapter prepares inside one private attempt.
#[derive(Debug)]
pub struct Context<'a> {
    pub guard: &'a crate::guard::Guard,
    pub private: &'a crate::isolate::Private,
    pub pin: &'a PinnedConfiguration,
    /// The pin's file, resolving its relative configuration document.
    pub pin_file: &'a Path,
    pub task: &'a crate::Task,
    pub variant: Option<&'a crate::Variant>,
    pub model: &'a crate::ModelChoice,
    pub arm: &'a crate::arm::ArmRef,
    /// Observed outside the attempt timer from the selected executable.
    pub installed_version: &'a str,
    /// Harness account directory: smith credentials or the source Codex home.
    pub home: &'a Path,
    pub wall: skein_lib::Wall,
    /// Hard deadline plus the grace and settlement spans; adapter adds teardown.
    pub lifetime: skein_lib::Duration,
    /// The runner's exact allow-listed environment, also used by Smith tools.
    pub environment: &'a [(std::ffi::OsString, std::ffi::OsString)],
}

/// The adapter's prepared invocation; unsupported agents refuse before this value.
#[derive(Debug)]
pub enum PreparedAgent {
    /// Codex's pinned standalone command, in an attempt-owned home.
    Codex(codex::PreparedCodex),
    /// Smith's verified built arm and native outside face.
    Smith(smith::Prepared),
}

/// The invocation supplied to the common process supervisor.
#[derive(Debug)]
pub struct Invocation {
    pub program: PathBuf,
    pub arguments: Vec<std::ffi::OsString>,
    pub environment: Vec<(std::ffi::OsString, std::ffi::OsString)>,
    pub mode: skein_world::end_to_end::Mode,
    pub teardown: skein_lib::Duration,
}

/// A neutral stream observation for the runner's clock and scripted person.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Seen {
    /// A provider response or reported turn ended.
    Response,
    /// A named tool call started.
    ToolCall { tool: String },
    /// A child thread was opened.
    Child,
    /// The accepted answer ends task wall time and begins teardown.
    Answer,
}

/// An observer retains its selected stream and any preparation behind it.
#[derive(Debug)]
pub enum Observing {
    /// Codex's root JSON stream and subsequent read-only collection.
    Codex(codex::Observing),
    /// Smith's native trace, read as the process writes it.
    Smith(smith::Observing),
}

/// Borrowed captures and their bound for the adapter's trace archives.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Traces<'a> {
    pub(crate) trace: &'a [u8],
    pub(crate) stdout: &'a [u8],
    pub(crate) maximum: usize,
}

/// The settled tree and authored checks an adapter reads (../../../docs/design/benchmarks/README.md, 6).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ended<'a> {
    pub(crate) exit: Exit,
    pub(crate) forced: Forced,
    pub(crate) task: &'a Task,
    pub(crate) workspace: &'a Path,
}

impl Agent {
    /// Why this agent cannot run yet, if it cannot.
    #[must_use]
    pub fn refusal(self) -> Option<&'static str> {
        match self {
            Self::Codex | Self::Smith => None,
            Self::ClaudeCode => Some("Claude Code belongs to the next comparison pass"),
        }
    }

    /// The login directory this adapter reads, from the harness's read-only one.
    pub fn home(self, source: &Path) -> Result<PathBuf, Fault> {
        match self {
            Self::Codex | Self::ClaudeCode => Ok(source.to_path_buf()),
            Self::Smith => smith::home(source),
        }
    }

    /// Whether a pinned request's authored arguments are this adapter's to pass on.
    #[must_use]
    pub fn takes_arguments(self, arguments: &[String]) -> bool {
        match self {
            Self::Codex | Self::ClaudeCode => false,
            Self::Smith => smith::takes_arguments(arguments),
        }
    }

    /// The stream carries the event checks a task may decide on.
    #[must_use]
    pub fn reports_events(self) -> bool {
        match self {
            Self::Codex | Self::ClaudeCode => false,
            Self::Smith => true,
        }
    }

    /// The arm and version this attempt's executable is, checked against its identity.
    pub(crate) fn source_arm(self, work: Work<'_>, identity: &Identity) -> Result<(ArmRef, String), Fault> {
        match self {
            Self::Codex => codex::source_arm(work, identity),
            Self::Smith => smith::source_arm(work, identity),
            Self::ClaudeCode => Err(input(self.refusal().expect("unsupported agent"))),
        }
    }

    /// Prepare, with the adapter's own setup check where it has one.
    pub(crate) fn prepare_checked(
        self,
        context: &Context<'_>,
        arguments: &[String],
    ) -> Result<(PreparedAgent, Option<ProcessResult>), Fault> {
        match self {
            Self::Codex | Self::ClaudeCode => self.prepare(context).map(|prepared| (prepared, None)),
            Self::Smith => smith::prepare_checked(context, arguments)
                .map(|(prepared, receipt)| (PreparedAgent::Smith(prepared), receipt)),
        }
    }

    /// Prepare a supported adapter; missing faces or versions refuse as setup.
    pub fn prepare(self, context: &Context<'_>) -> Result<PreparedAgent, Fault> {
        match self {
            Self::Codex => codex::prepare(context).map(PreparedAgent::Codex),
            Self::Smith => smith::prepare(context).map(PreparedAgent::Smith),
            Self::ClaudeCode => Err(crate::Fault {
                kind: FaultKind::Setup(crate::isolate::SetupStep::Configuration),
                detail: "Claude Code belongs to the next comparison pass".into(),
            }),
        }
    }
}

impl PreparedAgent {
    /// Archive what the attempt was prepared with.
    pub(crate) fn archive(&self, archive: Archive<'_>, result: &mut AttemptResult) -> Result<(), Fault> {
        match self {
            Self::Codex(prepared) => prepared.archive(archive, result),
            Self::Smith(prepared) => prepared.archive(archive, result),
        }
    }

    /// Refuse an identity that differs from what was prepared.
    pub fn check_identity(&self, identity: &Identity) -> Result<(), Fault> {
        match self {
            Self::Codex(_) => Ok(()),
            Self::Smith(_) => smith::Prepared::check_identity(identity),
        }
    }

    /// Build the command with its pinned configuration overrides.
    #[must_use]
    pub fn command(&self) -> Invocation {
        match self {
            Self::Codex(prepared) => prepared.command(),
            Self::Smith(prepared) => prepared.command(),
        }
    }

    /// Move preparation into its matching observer before execution.
    pub fn observe(self) -> Result<Observing, Fault> {
        match self {
            Self::Codex(prepared) => Ok(Observing::Codex(codex::Observing {
                prepared: Some(Box::new(prepared)),
                observer: codex::Observer::default(),
                collected: None,
                collection_error: None,
            })),
            Self::Smith(prepared) => smith::Observing::new(prepared).map(Observing::Smith),
        }
    }
}

impl Observing {
    /// The stream has ended: faults a closing read finds.
    pub fn finish(&mut self) -> Result<(), Fault> {
        match self {
            Self::Codex(_) => Ok(()),
            Self::Smith(observing) => observing.finish(),
        }
    }

    /// A refusal at startup, read from the stream and standard error.
    #[must_use]
    pub fn refused(&self, task: &Task, exit: Exit, forced: Forced, stderr: &[u8]) -> Option<End> {
        match self {
            Self::Codex(_) => None,
            Self::Smith(observing) => observing.refused(task, exit, forced, stderr),
        }
    }

    /// Archive the traces this adapter keeps beside its stream.
    pub(crate) fn archive_traces(
        &mut self,
        archive: Archive<'_>,
        result: &mut AttemptResult,
        process: Traces<'_>,
    ) -> Result<(), Fault> {
        match self {
            Self::Codex(_) => Ok(()),
            Self::Smith(observing) => observing.archive_traces(archive, result, process.maximum),
        }
    }

    /// Archive the main trace after the standard streams (../../../docs/design/benchmarks/README.md, 10).
    pub(crate) fn archive_trace(
        &self,
        archive: Archive<'_>,
        result: &mut AttemptResult,
        process: Traces<'_>,
    ) -> Result<(), Fault> {
        match self {
            Self::Codex(_) => codex::Observing::archive_trace(archive, result, process.trace),
            Self::Smith(observing) => observing.archive_trace(archive, result, process),
        }
    }

    /// The tree has ended: health, this adapter's checks, and its credential proof.
    pub(crate) fn settle(
        &mut self,
        archive: Archive<'_>,
        result: &mut AttemptResult,
        ended: Ended<'_>,
    ) -> Result<(), Fault> {
        match self {
            Self::Codex(observing) => observing.settle(archive, result),
            Self::Smith(observing) => {
                observing.settle(result, ended);
                Ok(())
            }
        }
    }

    /// What the agent persisted (../../../docs/design/benchmarks/README.md, 6, collect).
    pub(crate) fn collect(&mut self, archive: Archive<'_>, result: &mut AttemptResult) -> Result<(), Fault> {
        match self {
            Self::Codex(observing) => observing.collect_result(archive, result),
            Self::Smith(observing) => observing.collect(result),
        }
    }

    /// Cache reuse over this attempt's responses.
    #[must_use]
    pub fn cache_reuse(&self, threshold: f64, miss_limit: u32) -> Measure<CacheObservation> {
        match self {
            Self::Codex(observing) => observing.cache_reuse(threshold, miss_limit),
            Self::Smith(observing) => observing.events().cache_reuse(threshold, miss_limit),
        }
    }

    /// The check this adapter leaves to a whole triplet, where a cache measure exists.
    #[must_use]
    pub fn cache_check(&self) -> Option<CheckVerdict> {
        match self {
            Self::Codex(_) => Some(codex::Observing::cache_check()),
            Self::Smith(_) => None,
        }
    }

    /// The accepted answers come from a program account, not a provider.
    #[must_use]
    pub fn program_source(&self) -> bool {
        match self {
            Self::Codex(_) => false,
            Self::Smith(observing) => observing.prepared.program_source(),
        }
    }

    /// An offline stream replay: Codex's JSON lines, with no preparation behind them.
    #[must_use]
    pub fn replay() -> Observing {
        Self::Codex(codex::Observing {
            prepared: None,
            observer: codex::Observer::default(),
            collected: None,
            collection_error: None,
        })
    }

    /// When the person's input closes unless the task says.
    #[must_use]
    pub fn close(&self) -> Close {
        match self {
            Self::Codex(_) => Close::AfterPrompt,
            Self::Smith(observing) => match observing.prepared.face {
                smith::events::Shape::Exec => Close::AfterPrompt,
                smith::events::Shape::Chat => Close::AfterAnswer,
            },
        }
    }

    /// The native trace followed as the process writes it, opened.
    pub fn trace_file(&self) -> Result<Option<std::fs::File>, Fault> {
        match self {
            Self::Codex(_) => Ok(None),
            Self::Smith(observing) => match observing.prepared.face {
                smith::events::Shape::Exec => Ok(None),
                smith::events::Shape::Chat => {
                    let path =
                        observing.prepared.trace.as_ref().ok_or_else(|| input("Smith Chat requires a trace path"))?;
                    std::fs::File::open(path).map(Some).map_err(|error| io(format!("trace open: {error}")))
                }
            },
        }
    }

    /// The paths written outside the workspace, as far as this stream shows them.
    #[must_use]
    pub fn writes_outside(&self, workspace: &Path) -> crate::Measure<Vec<PathBuf>> {
        match self {
            Self::Codex(observing) => observing.writes_outside(workspace),
            Self::Smith(_) => crate::Measure::unavailable("event stream does not expose every shell filesystem effect"),
        }
    }

    /// The stream's own counts and tokens, before anything is collected.
    pub fn stream_measures(&self, result: &mut crate::AttemptResult) -> Result<(), Fault> {
        match self {
            Self::Codex(observing) => observing.observer.stream_measures(result),
            Self::Smith(_) => codex::Observer::default().stream_measures(result),
        }
    }

    /// Feed one complete line; known malformed records fault before classification.
    pub fn feed_line(&mut self, line: &str) -> Result<Vec<Seen>, Fault> {
        match self {
            Self::Codex(observing) => observing.feed_line(line),
            Self::Smith(observing) => observing.feed_line(line),
        }
    }

    /// The root final message, where the selected stream retains it.
    #[must_use]
    pub fn answer(&self) -> Option<String> {
        match self {
            Self::Codex(observing) => observing.observer.answer().map(str::to_owned),
            Self::Smith(observing) => observing
                .observer
                .sessions()
                .last()?
                .records
                .iter()
                .rev()
                .find_map(|record| {
                    if let smith_events::Event::RunCompleted(value) = &record.event {
                        Some(
                            value
                                .result
                                .as_ref()
                                .and_then(|result| result.text.as_deref())
                                .and_then(|text| std::str::from_utf8(text).ok())
                                .map(str::to_owned),
                        )
                    } else {
                        None
                    }
                })
                .flatten(),
        }
    }

    /// Classify from the selected face, its accepted answer and tree exit.
    #[must_use]
    pub fn classify(&self, exit: crate::Exit, forced: crate::Forced) -> crate::Classification {
        match self {
            Self::Codex(observing) => observing.observer.classify(exit, forced),
            Self::Smith(observing) => observing.observer.classify(exit, forced),
        }
    }
}

/// The adapter selected by a suite author for each attempt.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Agent {
    /// smith, observed through the face its binary offers.
    Smith,
    /// Codex, observed through JSON lines and its named rollouts.
    Codex,
    /// Standalone Claude Code identity, refused by the benchmark adapters.
    ClaudeCode,
}

impl Agent {
    /// The directory holding this adapter's pinned configurations.
    #[must_use]
    pub const fn directory(self) -> &'static str {
        match self {
            Self::Smith => "smith",
            Self::Codex => "codex",
            Self::ClaudeCode => "claude-code",
        }
    }
}

/// The provider route selected by a suite author.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Provider {
    /// The `OpenAI` model tier table, retained under its codex name.
    Codex,
    /// The Anthropic route.
    Anthropic,
}

/// A committed configuration pin supplied to one agent adapter.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    pub name: String,
    pub agent: Agent,
    pub provider: Provider,
    pub version: String,
    /// A smith pin may require a source ancestor for its account policy.
    pub minimum_commit: Option<String>,
    /// The private package executable's digest, required for Codex.
    pub sha256: Option<String>,
    /// Codex's whole package, relative to the harness state root.
    pub package: Option<PathBuf>,
    pub configuration: PathBuf,
}

/// The validated pin and digest carried into an attempt's identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinnedConfiguration {
    pub pin: Configuration,
    pub sha256: String,
}

/// Read a committed pin without reading a user's home or credentials.
pub fn read_configuration(file: &Path) -> Result<PinnedConfiguration, Refusal> {
    let pin: Configuration = crate::formats::read_document(file)?;
    if pin.name.is_empty() || pin.version.is_empty() {
        return Err(Refusal::new(file, "name/version", "configuration name and agent version must be pinned"));
    }
    crate::check::check_path(file, "configuration", &pin.configuration)?;
    match pin.agent {
        Agent::Smith => {
            if let Some(commit) = &pin.minimum_commit
                && (!crate::formats::lowercase_hex(commit, 40))
            {
                return Err(Refusal::new(file, "minimum_commit", "expected a full lowercase source commit"));
            }
            if pin.name == "smith-claude-code" && pin.provider != Provider::Anthropic {
                return Err(Refusal::new(file, "provider", "smith-claude-code requires the anthropic tier"));
            }
        }
        Agent::Codex => {
            if pin.provider != Provider::Codex {
                return Err(Refusal::new(file, "provider", "Codex requires its codex route"));
            }
            let digest = pin
                .sha256
                .as_deref()
                .ok_or_else(|| Refusal::new(file, "sha256", "Codex requires an executable digest"))?;
            if !crate::formats::lowercase_hex(digest, 64) {
                return Err(Refusal::new(file, "sha256", "expected 64 lowercase hexadecimal digits"));
            }
            let package = pin
                .package
                .as_ref()
                .ok_or_else(|| Refusal::new(file, "package", "Codex requires a private package"))?;
            crate::check::check_path(file, "package", package)?;
        }
        Agent::ClaudeCode => {
            if pin.provider != Provider::Anthropic {
                return Err(Refusal::new(file, "provider", "Claude Code requires its anthropic route"));
            }
        }
    }
    let directory = file.parent().ok_or_else(|| Refusal::new(file, "configuration", "pin needs a directory"))?;
    let metadata = std::fs::read(file).map_err(|error| Refusal::new(file, "document", error.to_string()))?;
    let configuration = directory.join(&pin.configuration);
    let config_metadata = std::fs::symlink_metadata(&configuration)
        .map_err(|error| Refusal::new(file, "configuration", error.to_string()))?;
    if !config_metadata.is_file() || config_metadata.is_symlink() {
        return Err(Refusal::new(file, "configuration", "expected a committed regular configuration file"));
    }
    let bytes =
        std::fs::read(&configuration).map_err(|error| Refusal::new(file, "configuration", error.to_string()))?;
    let mut digest = Sha256::new();
    digest.update(skein_lib::len_u64(metadata.len()).to_be_bytes());
    digest.update(metadata);
    digest.update(skein_lib::len_u64(bytes.len()).to_be_bytes());
    digest.update(bytes);
    Ok(PinnedConfiguration { pin, sha256: format!("{:x}", digest.finalize()) })
}
