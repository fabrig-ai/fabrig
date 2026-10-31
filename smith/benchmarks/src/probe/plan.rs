//! Frozen request planning consumes authored suites and verified arms.
//! No agent state is read; benchmark requests and settled public evidence are inputs.
//! Contract: smith/docs/design/benchmarks.md, sections 5.1 and 11.5.

use super::{Order, Planned, ProbeAgent, Source, gating};
use crate::arm::ArmRef;
use crate::guard::Guard;
use crate::{Agent, Identity, Measure, ModelIdentity, Suite};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

fn write(guard: &Guard, path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(guard.write_path(path)?)
        .map_err(|error| error.to_string())?;
    file.write_all(bytes).map_err(|error| error.to_string())
}

fn utc() -> Result<(String, String), String> {
    let wall = skein_shell::Clock::new().now().wall;
    let time = time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(wall.as_nanos()))
        .map_err(|error| error.to_string())?;
    let started = time
        .replace_nanosecond(0)
        .map_err(|error| error.to_string())?
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|error| error.to_string())?;
    Ok((started.replace(['-', ':'], ""), started))
}

fn cpus() -> Result<Vec<u32>, String> {
    let status = fs::read_to_string("/proc/self/status").map_err(|error| error.to_string())?;
    let affinity =
        status.lines().find_map(|line| line.strip_prefix("Cpus_allowed_list:")).ok_or("missing CPU affinity")?;
    let mut cpus = Vec::new();
    for range in affinity.trim().split(',') {
        let (first, last) = range.split_once('-').unwrap_or((range, range));
        cpus.extend(
            first.parse::<u32>().map_err(|error| error.to_string())?
                ..=last.parse::<u32>().map_err(|error| error.to_string())?,
        );
    }
    Ok(cpus)
}

pub(super) fn task(entries: &[crate::CatalogueTask], name: &str) -> Result<crate::CatalogueTask, String> {
    for entry in entries {
        let mut selected = entry.clone();
        if entry.name == name {
            return Ok(selected);
        }
        if let Some(variant) = name.strip_prefix(&format!("{}/", entry.name)) {
            selected.name = name.into();
            selected.task = entry.task.with_variant(variant)?;
            return Ok(selected);
        }
    }
    Err(format!("unknown task: {name}"))
}

// Fixed machine and directory observations supplied to request construction.
struct RequestInputs<'a> {
    repository: &'a Path,
    logins: PathBuf,
    host: String,
    cpus: Vec<u32>,
}

#[derive(Clone, Copy)]
struct RunIdentity<'a> {
    name: &'a str,
    started: &'a str,
    face: Option<crate::SmithFace>,
    harness_commit: &'a str,
}

fn request(
    inputs: &RequestInputs<'_>,
    entry: &crate::CatalogueTask,
    suite: &Suite,
    agent: &crate::SuiteAgent,
    arm: &ArmRef,
    run: RunIdentity<'_>,
    attempt: usize,
) -> Result<crate::run::Request, String> {
    let root = inputs.repository.join("benchmarks");
    let task = &entry.task;
    let pin_file = root.join("agents").join(agent.agent.directory()).join(format!("{}.pin.toml", agent.config));
    let pin = crate::read_configuration(&pin_file).map_err(|error| error.to_string())?;
    let models_file = root.join("agents/models.toml");
    let models = crate::read_model_tiers(&models_file).map_err(|error| error.to_string())?;
    let model = models.lookup(&models_file, &suite.tier, agent.provider).map_err(|error| error.to_string())?;
    let (program, version, digest) = match arm {
        ArmRef::Built(build) => (build.directory.join("smith"), build.arm.commit.clone(), build.arm.sha256.clone()),
        ArmRef::Installed { program } => {
            (program.clone(), pin.pin.version.clone(), crate::arm::binary_digest(program)?)
        }
    };
    Ok(crate::run::Request {
        task: entry.manifest.clone(),
        program,
        arguments: Vec::new(),
        configuration: Some(pin_file),
        logins: Some(inputs.logins.clone()),
        identity: Identity {
            run: run.name.into(),
            attempt: format!("{attempt:02}"),
            arm: if agent.config == "smith-claude-code" {
                format!("smith-claude-code-{}", &digest[..8])
            } else {
                format!("{}-{}-{}", agent.agent.directory(), agent.config, &digest[..8])
            },
            task: entry.name.clone(),
            task_version: task.version,
            seed_sha256: task.seed_sha256.clone(),
            prompt_sha256: format!("{:x}", Sha256::digest(task.prompt.as_bytes())),
            agent: agent.agent,
            agent_version: version,
            binary_sha256: digest,
            smith_face: if agent.agent == Agent::Smith { run.face } else { None },
            configuration_sha256: pin.sha256,
            user_configuration_sha256: Measure::unavailable("user configuration is isolated"),
            provider: agent.provider,
            model: ModelIdentity {
                requested_model: task
                    .requested_model(agent.agent, model.model.as_deref().ok_or("unresolved tier")?)?
                    .into(),
                requested_effort: task.effort.clone().or_else(|| model.effort.clone()).ok_or("unresolved effort")?,
                resolved_model: Measure::unavailable("not run"),
                resolved_window: Measure::unavailable("not run"),
                resolved_effort: Measure::unavailable("not run"),
            },
            harness_commit: run.harness_commit.into(),
            host: inputs.host.clone(),
            cpus: inputs.cpus.clone(),
            started_utc: run.started.into(),
        },
        teardown_ms: 10_000,
        grace_ms: 1_000,
        settlement_ms: 3_000,
        setup_deadline_seconds: 900,
        max_stdout_bytes: 2 * 1024 * 1024,
        max_stderr_bytes: 2 * 1024 * 1024,
        disk_estimate_bytes: 16 * 1024 * 1024,
        force_walk: true,
    })
}

fn task_requests(
    repository: &Path,
    entry: &crate::CatalogueTask,
    suite: &Suite,
    arms: (&ArmRef, &[(String, ArmRef)]),
    run: RunIdentity<'_>,
    repetitions: u32,
    requests: &mut Vec<crate::run::Request>,
) -> Result<(), String> {
    let inputs = RequestInputs {
        repository,
        logins: crate::arm::state_root(&Guard::from_environment()?)?.join("logins/codex"),
        host: fs::read_to_string("/proc/sys/kernel/hostname").map_err(|error| error.to_string())?.trim().into(),
        cpus: cpus()?,
    };
    task_requests_in(&inputs, entry, suite, arms, run, repetitions, requests)
}

fn task_requests_in(
    inputs: &RequestInputs<'_>,
    entry: &crate::CatalogueTask,
    suite: &Suite,
    arms: (&ArmRef, &[(String, ArmRef)]),
    run: RunIdentity<'_>,
    repetitions: u32,
    requests: &mut Vec<crate::run::Request>,
) -> Result<(), String> {
    let (arm, binaries) = arms;
    for agent in &suite.agents {
        if !agent.tasks.is_empty() && !agent.tasks.contains(&entry.name) {
            continue;
        }
        let installed;
        let selected = if agent.agent == Agent::Codex && matches!(arm, ArmRef::Built(_)) {
            installed = codex(inputs.repository, &agent.config)?;
            &installed
        } else {
            arm
        };
        let mut executions = Vec::new();
        if suite.change {
            for (name, binary) in binaries {
                executions.push((Some(name), binary, Vec::new()));
            }
        }
        for comparison in &suite.arms {
            match comparison {
                crate::Arm::Binary { name, .. } => {
                    let (_, binary) = binaries
                        .iter()
                        .find(|(built_name, _)| built_name == name)
                        .ok_or("binary arm has no prepared build")?;
                    executions.push((Some(name), binary, Vec::new()));
                }
                crate::Arm::Override { name, smith } => {
                    executions.push((Some(name), selected, crate::suite::override_arguments(smith)));
                }
                crate::Arm::Agent { .. } => {}
            }
        }
        if executions.is_empty() {
            executions.push((None, selected, Vec::new()));
        }
        for (name, program, arguments) in executions {
            for _ in 0..repetitions {
                let mut request = request(inputs, entry, suite, agent, program, run, requests.len() + 1)?;
                if let Some(name) = name {
                    request.identity.arm.clone_from(name);
                    request.arguments.clone_from(&arguments);
                }
                requests.push(request);
            }
        }
    }
    Ok(())
}

fn narrowed(file: &Path, order: &Order) -> Result<toml::Value, String> {
    let mut value: toml::Value = crate::formats::read_document(file).map_err(|error| error.to_string())?;
    let fields = value.as_table_mut().ok_or("suite must be a TOML table")?;
    fields.insert("tasks".into(), toml::Value::Array(vec![toml::Value::String(order.probe.clone())]));
    fields.insert("select".into(), toml::Value::Array(vec![]));
    fields.insert("arm".into(), toml::Value::Array(vec![]));
    fields.insert("repetitions".into(), toml::Value::Integer(i64::from(order.attempts)));
    fields.insert("rerun_failed".into(), toml::Value::Integer(i64::from(order.attempts == 1)));
    if order.probe == "probes/credentials" && order.agent != ProbeAgent::Smith {
        return Err("the credentials probe requires smith's chatgpt account".into());
    }
    if order.agent == ProbeAgent::SmithClaudeCode {
        let program = toml::from_str::<toml::Value>(
            "[[agents]]\nagent='smith'\nprovider='anthropic'\nconfig='smith-claude-code'",
        )
        .map_err(|error| error.to_string())?;
        fields.insert("agents".into(), program["agents"].clone());
    } else if order.agent == ProbeAgent::Codex {
        let codex = toml::from_str::<toml::Value>("[[agents]]\nagent='codex'\nprovider='codex'\nconfig='standard'")
            .map_err(|error| error.to_string())?;
        fields.insert("agents".into(), codex["agents"].clone());
    } else {
        let agents = fields.get_mut("agents").and_then(toml::Value::as_array_mut).ok_or("suite agents")?;
        agents.retain(|agent| {
            agent["agent"].as_str() == Some("smith")
                && agent["tasks"].as_array().is_none_or(|tasks| {
                    tasks.is_empty() || tasks.iter().any(|task| task.as_str() == Some(&order.probe))
                })
        });
        for agent in agents {
            agent["tasks"] = toml::Value::Array(vec![toml::Value::String(order.probe.clone())]);
        }
    }
    Ok(value)
}

/// Prepare a probe's authored pin and tier, without invoking a compiler or agent.
pub fn plan(repository: &Path, order: &Order, arm: &ArmRef) -> Result<Planned, String> {
    if !order.probe.starts_with("probes/") || order.attempts == 0 {
        return Err("invalid probe order".into());
    }
    let value = narrowed(&repository.join("benchmarks/suites/probes.toml"), order)?;
    prepare(repository, &order.probe, &value, (arm, None), true, &[])
}

pub(super) fn prepare(
    repository: &Path,
    name: &str,
    value: &toml::Value,
    arms: (&ArmRef, Option<&ArmRef>),
    logical: bool,
    asked: &[String],
) -> Result<Planned, String> {
    let harness_commit = crate::arm::commit(repository, "HEAD")?;
    let suite: Suite = value.clone().try_into().map_err(|error| error.to_string())?;
    let (arm, parent_arm) = arms;
    check_parent(&suite, parent_arm.is_some())?;
    let (stamp, started) = utc()?;
    let source = match arm {
        ArmRef::Built(build) => build.arm.commit.clone(),
        ArmRef::Installed { program } => crate::arm::binary_digest(program)?,
    };
    let run = format!("{}-{}-{stamp}", name.replace('/', "-"), &source[..8]);
    let guard = Guard::from_environment()?;
    let parent = crate::arm::state_root(&guard)?.join("requests");
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(guard.write_path(&parent)?)
        .map_err(|error| error.to_string())?;
    let directory = parent.join(&run);
    if directory.exists() {
        return Err("run already exists; wait until the next UTC second".into());
    }
    crate::isolate::create_directory(&guard, &directory)?;
    let suite_file = parent.join(format!("{run}.toml"));
    write(&guard, &suite_file, toml::to_string(value).map_err(|error| error.to_string())?.as_bytes())?;
    let root = repository.join("benchmarks");
    let entries = if logical {
        let base = name.split('/').take(2).collect::<Vec<_>>().join("/");
        if base.split('/').any(|part| part.is_empty() || part == ".." || part == ".") {
            return Err("invalid task name".into());
        }
        let manifest = root.join("tasks").join(&base).join("task.toml");
        vec![crate::CatalogueTask {
            name: base,
            task: crate::check_task(&manifest, &root.join("../docs/design")).map_err(|error| error.to_string())?,
            manifest,
        }]
    } else {
        crate::catalogue(&root.join("tasks"), &root.join("../docs/design")).map_err(|error| error.to_string())?
    };
    let models = crate::read_model_tiers(&root.join("agents/models.toml")).map_err(|error| error.to_string())?;
    let names = crate::validate_suite(&suite_file, &suite, &entries, &root.join("agents"), &models)
        .map_err(|error| error.to_string())?;
    let names = crate::include(&suite_file, &suite, &names, asked).map_err(|error| error.to_string())?;
    let names = if name == "smoke" { gating::smoke_tasks(&entries, &names)? } else { names };
    let face = arm_face(repository, arm)?;
    let mut binaries = Vec::new();
    if let Some(parent_arm) = parent_arm {
        arm_face(repository, parent_arm)?;
        binaries.push(("change".into(), arm.clone()));
        binaries.push(("parent".into(), parent_arm.clone()));
    }
    for comparison in &suite.arms {
        if let crate::Arm::Binary { name, commit } = comparison {
            let binary = archived(repository, &Source::Commit(commit.clone()))?;
            arm_face(repository, &binary)?;
            binaries.push((name.clone(), binary));
        }
    }
    let mut requests = Vec::new();
    for name in names {
        let entry = task(&entries, &name)?;
        let task = &entry.task;
        let repetitions = suite.task_repetitions(&name, task.repetitions, logical)?;
        task_requests(
            repository,
            &entry,
            &suite,
            (arm, &binaries),
            RunIdentity { name: &run, started: &started, face, harness_commit: &harness_commit },
            repetitions,
            &mut requests,
        )?;
    }
    write_requests(&guard, &directory, &requests)?;
    Ok(Planned { run, suite: suite_file, directory, requests })
}

fn arm_face(repository: &Path, arm: &ArmRef) -> Result<Option<crate::SmithFace>, String> {
    match arm {
        ArmRef::Installed { .. } => Ok(None),
        ArmRef::Built(build) => {
            let faces = crate::agent::smith::read_faces(&repository.join("benchmarks/agents/smith/faces.toml"))
                .map_err(|error| error.reason)?;
            crate::agent::smith::face(&build.arm, &faces, repository).map_err(|error| error.reason)?;
            Ok(Some(crate::SmithFace::Events))
        }
    }
}

pub(super) fn codex(repository: &Path, configuration: &str) -> Result<ArmRef, String> {
    let pin = crate::read_configuration(
        &repository.join("benchmarks/agents/codex").join(format!("{configuration}.pin.toml")),
    )
    .map_err(|error| error.to_string())?;
    let guard = Guard::from_environment()?;
    let program = crate::arm::state_root(&guard)?
        .join(pin.pin.package.ok_or("Codex requires a private package")?)
        .join("bin/codex");
    Ok(ArmRef::Installed { program })
}

pub(super) fn archived(repository: &Path, source: &Source) -> Result<ArmRef, String> {
    let reference = match source {
        Source::Head => "HEAD",
        Source::Commit(commit) => commit,
    };
    let commit = crate::arm::resolve_commit(repository, reference)?;
    let guard = Guard::from_environment()?;
    let arms = crate::arm::state_root(&guard)?.join("arms");
    for entry in fs::read_dir(arms).map_err(|_| format!("no archived arm for {commit}; run build {reference} first"))? {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry.file_name().to_string_lossy().starts_with(&format!("{commit}-")) {
            return crate::arm::read_build(&entry.path().join("smith")).map(ArmRef::Built);
        }
    }
    Err(format!("no archived arm for {commit}; run build {reference} first"))
}

fn write_requests(guard: &Guard, directory: &Path, requests: &[crate::run::Request]) -> Result<(), String> {
    for request in requests {
        write(
            guard,
            &directory.join(format!("{}.json", request.identity.attempt)),
            &serde_json::to_vec_pretty(request).map_err(|error| error.to_string())?,
        )?;
    }
    Ok(())
}

pub(super) fn check_parent(suite: &Suite, has_parent: bool) -> Result<(), String> {
    if suite.change != has_parent {
        return Err(if suite.change {
            "change suite requires --parent or --parent-arm"
        } else {
            "only change suites accept a parent"
        }
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod overrides;
