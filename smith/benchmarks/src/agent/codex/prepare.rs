//! Codex preparation retains pinned overrides and private credential paths.
//! It never writes to the source login. The command's settings are
//! explicit; its observer retains only root stream evidence until collection.
//! Contracts: ../../../../docs/design/benchmarks/README.md, sections 6, 8.2, 10 and 13.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::agent::{Context, Invocation, Seen};
use crate::isolate::SetupStep;
use crate::{Agent, Fault, FaultKind, Input, Mismatch, ModelChoice, Provider, input, mismatch, refuse_if, stream};

use super::{Item, Observation, Observer};

/// The pinned standalone Codex invocation, supplied to the common supervisor.
#[derive(Debug)]
pub struct PreparedCodex {
    pub overrides: Vec<(String, String)>,
    pub credential: PathBuf,
    pub expires: skein_lib::Wall,
    pub borrowed: Option<crate::credential::Borrowed>,
    pub home: PathBuf,
    pub final_message: PathBuf,
    pub model: ModelChoice,
    pub program: PathBuf,
    pub workspace: PathBuf,
    pub teardown: skein_lib::Duration,
}

/// The root stream observer and any pinned preparation for this attempt.
#[derive(Debug)]
pub struct Observing {
    /// None for an offline stream replay, which has nothing to collect.
    pub prepared: Option<Box<PreparedCodex>>,
    pub observer: Observer,
    pub collected: Option<super::Rollouts>,
    pub collection_error: Option<String>,
}

/// Prepare a version-matched Codex command without touching its existing login.
pub fn prepare(context: &Context<'_>) -> Result<PreparedCodex, Fault> {
    let mut prepared = configure(context).map_err(Fault::at(SetupStep::Configuration))?;
    let borrowed = crate::credential::codex(
        context.guard,
        context.home,
        &prepared.home,
        context.wall,
        context.lifetime.checked_add(prepared.teardown).ok_or_else(|| Fault {
            kind: FaultKind::Setup(SetupStep::Configuration),
            detail: "credential teardown lifetime overflow".into(),
        })?,
    )?;
    prepared.credential.clone_from(&borrowed.credential);
    prepared.expires = borrowed.expires;
    prepared.borrowed = Some(borrowed);
    Ok(prepared)
}

fn configure(context: &Context<'_>) -> Result<PreparedCodex, Fault> {
    refuse_if(!context.task.person.is_empty(), Input, "Codex does not accept later person lines")?;
    if context.pin.pin.agent != Agent::Codex || context.pin.pin.provider != Provider::Codex {
        return Err(input("Codex requires its own codex configuration pin"));
    }
    let installed =
        context.installed_version.trim().strip_prefix("codex-cli ").unwrap_or(context.installed_version.trim());
    if installed != context.pin.pin.version {
        return Err(mismatch(format!("installed Codex {installed} differs from pinned {}", context.pin.pin.version)));
    }
    if context.model.model.as_ref().is_none_or(String::is_empty)
        || context.model.effort.as_ref().is_none_or(String::is_empty)
    {
        return Err(input("Codex model and effort must be explicitly selected"));
    }
    let program = match context.arm {
        crate::arm::ArmRef::Installed { program } => program.clone(),
        crate::arm::ArmRef::Built(_) => {
            return Err(input("Codex requires its installed executable, not a smith build"));
        }
    };
    let reread = crate::read_configuration(context.pin_file)?;
    refuse_if(reread != *context.pin, Mismatch, "Codex configuration changed since the attempt was planned")?;
    let digest = crate::arm::binary_digest(&program)?;
    if Some(digest.as_str()) != context.pin.pin.sha256.as_deref() {
        return Err(mismatch(format!("Codex executable digest {digest} differs from its pin")));
    }
    let overrides_path = context
        .pin_file
        .parent()
        .ok_or_else(|| input("configuration pin needs a directory"))?
        .join(&context.pin.pin.configuration);
    let bytes = std::fs::read(&overrides_path)?;
    let text = std::str::from_utf8(&bytes)?;
    let document: std::collections::BTreeMap<String, toml::Value> = toml::from_str(text)?;
    let milliseconds = document
        .get("harness")
        .and_then(|value| value.get("teardown_ms"))
        .and_then(toml::Value::as_integer)
        .ok_or_else(|| input("Codex pin requires harness.teardown_ms"))?;
    let nanos = u64::try_from(milliseconds)
        .ok()
        .and_then(|value| value.checked_mul(1_000_000))
        .filter(|value| *value > 0)
        .ok_or_else(|| input("Codex teardown must fit a positive duration"))?;
    let teardown = skein_lib::Duration::from_nanos(nanos);
    let mut overrides: Vec<_> =
        document.into_iter().filter(|(key, _)| key != "harness").map(|(key, value)| (key, value.to_string())).collect();
    let workspace = crate::isolate::working_directory(context.guard, context.private, context.task)?;
    // Codex splits dotted override paths literally; quote the dynamic path
    // within a top-level TOML table value, including dots in the workspace.
    let project = toml::Table::from_iter([("trust_level".into(), toml::Value::String("trusted".into()))]);
    let projects = toml::Table::from_iter([(workspace.display().to_string(), toml::Value::Table(project))]);
    overrides.push(("projects".into(), toml::Value::Table(projects).to_string()));
    let writable = [&context.private.cargo_home, &context.private.target, &context.private.temporary]
        .into_iter()
        .map(|path| toml::Value::String(path.display().to_string()))
        .collect();
    overrides.push(("sandbox_workspace_write.writable_roots".into(), toml::Value::Array(writable).to_string()));
    overrides.push((
        "model_reasoning_effort".into(),
        toml::Value::String(context.model.effort.clone().expect("validated effort")).to_string(),
    ));
    Ok(PreparedCodex {
        overrides,
        credential: PathBuf::new(),
        expires: skein_lib::Wall::EPOCH,
        borrowed: None,
        home: context.private.root.join("codex"),
        final_message: context.private.root.join("final.txt"),
        model: context.model.clone(),
        program,
        workspace,
        teardown,
    })
}

impl PreparedCodex {
    /// Render the standalone JSON-lines command with exact pinned settings.
    #[must_use]
    pub fn command(&self) -> Invocation {
        let mut arguments: Vec<OsString> =
            ["--no-daemon", "exec", "--json", "-m"].into_iter().map(Into::into).collect();
        arguments.push(self.model.model.as_ref().expect("prepared model").into());
        for (key, value) in &self.overrides {
            arguments.extend(["-c".into(), format!("{key}={value}").into()]);
        }
        arguments.extend([
            "-C".into(),
            self.workspace.as_os_str().into(),
            "-o".into(),
            self.final_message.as_os_str().into(),
            "-".into(),
        ]);
        Invocation {
            program: self.program.clone(),
            arguments,
            environment: vec![("CODEX_HOME".into(), self.home.as_os_str().into())],
            mode: skein_world::end_to_end::Mode::Pipes,
            teardown: self.teardown,
        }
    }
}

impl Observing {
    /// Observe known root records once, returning neutral timing cues.
    pub fn feed_line(&mut self, line: &str) -> Result<Vec<Seen>, Fault> {
        let answered = self.observer.answer().is_some();
        let turns = self.observer.usages.len();
        let record = self.observer.observe_line(line).map_err(|error| stream(error.to_string()))?;
        let mut seen = Vec::new();
        match record {
            Some(Observation::TurnCompleted { .. }) => {
                if self.observer.usages.len() != turns {
                    seen.push(Seen::Response);
                }
            }
            Some(Observation::ItemStarted { item }) => match item {
                Item::CommandExecution { .. } => seen.push(Seen::ToolCall { tool: "command_execution".into() }),
                Item::FileChange { .. } => seen.push(Seen::ToolCall { tool: "file_change".into() }),
                Item::CollabToolCall { tool, .. } => {
                    if tool == "spawn_agent" {
                        seen.push(Seen::Child);
                    }
                    seen.push(Seen::ToolCall { tool });
                }
                Item::AgentMessage { .. } => {}
            },
            Some(
                Observation::ThreadStarted { .. }
                | Observation::TurnStarted
                | Observation::ItemUpdated { .. }
                | Observation::ItemCompleted { .. }
                | Observation::TurnFailed { .. }
                | Observation::Error { .. },
            )
            | None => {}
        }
        if !answered && self.observer.answer().is_some() {
            seen.push(Seen::Answer);
        }
        Ok(seen)
    }

    /// The stream-visible writes outside this attempt's workspace.
    #[must_use]
    pub(in crate::agent) fn writes_outside(&self, workspace: &Path) -> crate::Measure<Vec<PathBuf>> {
        let mut outside = Vec::new();
        for item in self.observer.completed_items() {
            match item {
                Item::FileChange { changes, .. } => {
                    for change in changes {
                        let path = lexical(workspace, Path::new(&change.path));
                        if !path.starts_with(workspace.parent().unwrap_or(workspace)) {
                            outside.push(path);
                        }
                    }
                }
                Item::AgentMessage { .. } | Item::CommandExecution { .. } | Item::CollabToolCall { .. } => {}
            }
        }
        outside.sort();
        outside.dedup();
        crate::Measure::LowerBound {
            value: outside,
            missing: vec![
                "JSON lines omit non-root calls and shell filesystem effects; complete adapter collection required"
                    .into(),
            ],
        }
    }

    /// Read only this completed attempt's native root and child records.
    pub fn collect(&mut self) -> Result<&super::Rollouts, Fault> {
        let prepared =
            self.prepared.as_ref().ok_or_else(|| stream("offline Codex replay has no preparation to collect"))?;
        if let Some(what) = &self.collection_error {
            return Err(stream(what.clone()));
        }
        if self.collected.is_none() {
            let thread = self.observer.thread().ok_or_else(|| stream("Codex stream has no named root"))?;
            let mut children = Vec::new();
            for item in self.observer.completed_items() {
                if let Item::CollabToolCall { tool, receiver_thread_ids, .. } = item
                    && tool == "spawn_agent"
                {
                    children.extend(receiver_thread_ids.iter().cloned());
                }
            }
            children.sort();
            children.dedup();
            self.collected = Some(super::Rollouts { responses: Vec::new(), children: Vec::new(), files: Vec::new() });
            if let Err(error) = super::rollouts::collect_named(
                &prepared.home,
                thread,
                &children,
                self.collected.as_mut().expect("raw collection retained"),
            ) {
                let what = error.to_string();
                self.collection_error = Some(what.clone());
                return Err(stream(what));
            }
        }
        Ok(self.collected.as_ref().expect("collected above"))
    }
}

fn lexical(base: &Path, path: &Path) -> PathBuf {
    use std::path::Component;
    let absolute = base.join(path);
    let mut normal = PathBuf::new();
    for part in absolute.components() {
        match part {
            Component::ParentDir => {
                normal.pop();
            }
            Component::CurDir => {}
            Component::RootDir | Component::Normal(_) | Component::Prefix(_) => normal.push(part),
        }
    }
    normal
}
