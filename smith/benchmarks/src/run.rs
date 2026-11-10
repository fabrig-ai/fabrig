//! One prepared attempt, retaining scoped evidence and private archives only.
//! Setup and grading run outside the measured invocation. The shared supervisor
//! owns every process tree; no process discovery or resource formula lives here.
//! Entry point `attempt` follows ../../docs/design/benchmarks/README.md, sections 6, 8 and 10.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use skein_lib::{Duration, Time};
use skein_shell::Clock;
use skein_world::end_to_end::{Command, Method, PeakScope};

use crate::guard::Guard;
use crate::isolate::SetupStep::{Configuration, Scratch, Seed, Space, Warm};
use crate::isolate::{Machine, Private, SetupStep};
use crate::process::{Bounds, Capture, ProcessResult, execute};
use crate::{
    Artifact, AttemptResult, Bound, End, Exit, Fault, FaultKind, Forced, Health, Identity, Input, Io, Measure, Outcome,
    ResourceScope, Resources, Spend, SpendBasis, Task, Timing, bound, credential, format, input, io, mismatch,
    refuse_if,
};

mod outcomes;

mod cache;

mod passive;

/// The directory the scope gate lists for every real run (../../docs/design/benchmarks/README.md, section 10).
pub const MACHINE_TEMPORARY: &str = "/tmp";

fn machine_temporary() -> PathBuf {
    MACHINE_TEMPORARY.into()
}

/// An adapter's prepared pipe invocation and author-supplied harness policies.
/// Arguments already name the planned scratch paths; the runner owns its cwd.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub task: PathBuf,
    /// The directory whose user-owned entries the scope gate lists before and
    /// after the attempt: `/tmp` for a real run, and for a document naming none.
    #[serde(default = "machine_temporary")]
    pub shared_temporary: PathBuf,
    pub identity: Identity,
    pub program: PathBuf,
    pub arguments: Vec<String>,
    /// The harness's read-only login; each adapter selects its directory from it.
    #[serde(rename = "codex_home")]
    pub logins: Option<PathBuf>,
    /// The committed adapter pin; absent only for offline stream replays.
    #[serde(default)]
    pub configuration: Option<PathBuf>,
    pub teardown_ms: u64,
    pub grace_ms: u64,
    pub settlement_ms: u64,
    pub setup_deadline_seconds: u64,
    pub max_stdout_bytes: usize,
    pub max_stderr_bytes: usize,
    /// Explicit required free bytes, including writable caches and raw archives.
    pub disk_estimate_bytes: u64,
    pub force_walk: bool,
}

impl Request {
    fn bounds(&self, hard: Duration, teardown: Duration) -> Bounds {
        Bounds {
            hard,
            teardown,
            grace: Duration::from_millis(self.grace_ms),
            settlement: Duration::from_millis(self.settlement_ms),
            stdout: self.max_stdout_bytes,
            stderr: self.max_stderr_bytes,
            force_walk: self.force_walk,
        }
    }
}

/// The recorded result and whether preparation left the suite incomplete.
#[derive(Debug)]
pub struct Recorded {
    pub result: AttemptResult,
    pub directory: PathBuf,
    pub incomplete: Incomplete,
}

/// Whether preparation prevented this attempt from contributing to a complete suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Incomplete {
    /// The prepared product invocation ran and has its measured terminal.
    No,
    /// Setup refused the attempt at this named preparation step.
    Setup(SetupStep),
}

fn milliseconds(span: Duration) -> u64 {
    span.as_millis()
}
fn elapsed(start: Time, end: Time) -> u64 {
    Duration::from_nanos(end.as_nanos().checked_sub(start.as_nanos()).expect("one monotonic attempt clock")).as_millis()
}

fn blank(identity: Identity) -> AttemptResult {
    let missing = "inference did not start";
    AttemptResult {
        version: crate::RESULT_VERSION,
        identity,
        timing: Timing {
            process_wall_ms: Measure::unavailable(missing),
            task_wall_ms: Measure::unavailable(missing),
            teardown_ms: Measure::unavailable(missing),
            grading_wall_ms: Measure::unavailable("no grader ran"),
            first_byte_ms: Measure::unavailable("not exposed by the selected stream"),
            largest_gap_ms: Measure::unavailable("not exposed by the selected stream"),
            longest_completion_ms: Measure::unavailable("not exposed by the selected stream"),
            agent_t_ms: Measure::unavailable("not reported by the agent"),
            record: crate::TimeRecord::unavailable(missing),
        },
        tokens: Vec::new(),
        counts: crate::Counts {
            responses: Measure::unavailable(missing),
            provider_attempts: Measure::unavailable(missing),
            retries: Measure::unavailable(missing),
            completions: Measure::unavailable(missing),
            tool_calls: BTreeMap::new(),
            tool_failures: BTreeMap::new(),
            children: Measure::unavailable(missing),
            conversations: Measure::unavailable(missing),
            compactions: Measure::unavailable(missing),
            message_terminals: BTreeMap::new(),
        },
        delegation: crate::Delegation::unavailable("inference did not start; delegation not observed"),
        resources: Resources {
            cpu_scope: ResourceScope::Unavailable,
            cpu_ms: Measure::unavailable(missing),
            peak_memory_scope: ResourceScope::Unavailable,
            peak_rss_bytes: Measure::unavailable(missing),
            own_peak_rss_bytes: Measure::unavailable(missing),
        },
        spend: Spend {
            priced: Measure::unavailable(missing),
            basis: SpendBasis::Priced,
            usd: Measure::unavailable("adapter has not supplied scoped usage and prices"),
        },
        outcome: Outcome {
            deciding: Vec::new(),
            end: End::Refused { setup: "preparation incomplete".into() },
            exit: Measure::unavailable(missing),
            forced: Forced::No,
            warnings: Vec::new(),
            checks: Vec::new(),
            grade: None,
            protected_files_changed: Measure::unavailable(missing),
            writes_outside_workspace: Measure::unavailable("calls have not been observed"),
        },
        health: Health { loss_per_sink: BTreeMap::new(), headroom: BTreeMap::new(), passive_checks: Vec::new() },
        artifacts: Vec::new(),
        cache: None,
    }
}

pub(crate) fn private_parent(guard: &Guard, directory: &Path) -> Result<(), Fault> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    let directory = guard.write_path(directory)?;
    if !directory.exists() {
        fs::DirBuilder::new().recursive(true).mode(0o700).create(&directory)?;
    }
    let metadata = fs::symlink_metadata(&directory)?;
    if !metadata.is_dir() || metadata.is_symlink() || metadata.permissions().mode() & 0o077 != 0 {
        return Err(input("attempt parents must be private directories"));
    }
    for ancestor in directory.ancestors() {
        refuse_if(ancestor.join(".git").exists(), Input, "attempt data must be outside repositories")?;
    }
    Ok(())
}

pub(crate) fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

/// Resolve the planned scratch root without materialising it.
pub fn scratch_root(state: &Path, prepared: &Request) -> Result<PathBuf, Fault> {
    if !valid_name(&prepared.identity.run) || !valid_name(&prepared.identity.attempt) {
        return Err(input("run and attempt ids must be ordinary directory names"));
    }
    Ok(state.join("scratch").join(format!("{}-{}", prepared.identity.run, prepared.identity.attempt)))
}

pub(crate) fn raw(
    clock: &Clock,
    machine: &Machine,
    private: &Private,
    arguments: &[String],
    deadline: u64,
    prepared: &Request,
) -> Result<ProcessResult, Fault> {
    raw_environment(
        clock,
        machine,
        private,
        arguments,
        prepared,
        CommandPolicy { deadline_seconds: deadline, environment: &BTreeMap::new() },
    )
}

#[derive(Clone, Copy)]
struct CommandPolicy<'a> {
    deadline_seconds: u64,
    environment: &'a BTreeMap<String, String>,
}

fn raw_environment(
    clock: &Clock,
    machine: &Machine,
    private: &Private,
    arguments: &[String],
    prepared: &Request,
    policy: CommandPolicy<'_>,
) -> Result<ProcessResult, Fault> {
    let CommandPolicy { deadline_seconds, environment } = policy;
    let (program, arguments) = arguments.split_first().ok_or_else(|| input("empty author command"))?;
    let command = Command {
        program: crate::isolate::program(machine, &private.workspace, Path::new(program))?,
        arguments: arguments.iter().map(Into::into).collect(),
        environment: {
            let mut values = crate::isolate::environment(machine, private, environment)?;
            values.push(("GIT_CONFIG_NOSYSTEM".into(), "1".into()));
            values.push(("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()));
            values
        },
        directory: private.workspace.clone(),
    };
    execute(
        clock,
        command,
        Box::new([]),
        Capture::Raw,
        prepared.bounds(Duration::from_secs(deadline_seconds), Duration::ZERO),
    )
}

pub(crate) fn succeeded(result: &ProcessResult) -> bool {
    result.exit == Exit::Code(0) && result.forced == Forced::No && result.error.is_none()
}

fn free_space(clock: &Clock, machine: &Machine, private: &Private, prepared: &Request) -> Result<(), Fault> {
    refuse_if(prepared.disk_estimate_bytes == 0, Input, "a positive disk estimate is required")?;
    let command = vec![
        "df".into(),
        "-Pk".into(),
        "--".into(),
        private.root.to_str().ok_or_else(|| format("scratch path must be UTF-8"))?.into(),
    ];
    let result = raw(clock, machine, private, &command, prepared.setup_deadline_seconds, prepared)?;
    refuse_if(!succeeded(&result), Input, "free-space observation failed")?;
    let output = std::str::from_utf8(&result.stdout)?;
    let bytes = output
        .lines()
        .last()
        .and_then(|line| line.split_whitespace().nth(3))
        .ok_or_else(|| input("df did not report available space"))?
        .parse::<u64>()?
        .checked_mul(1024)
        .ok_or_else(|| bound("free space overflow"))?;
    if bytes < prepared.disk_estimate_bytes {
        return Err(bound(format!(
            "insufficient disk space: {bytes} bytes available, {} required",
            prepared.disk_estimate_bytes
        )));
    }
    Ok(())
}

fn resources(process: &ProcessResult) -> Result<Resources, Fault> {
    let cpu = process.counts.user.checked_add(process.counts.system).ok_or_else(|| bound("tree CPU overflow"))?;
    let cpu_scope = match process.counts.method {
        Method::Cgroup => ResourceScope::Cgroup,
        Method::Walk => ResourceScope::PidfdWalk,
        Method::Group => ResourceScope::ProcessGroup,
    };
    let (peak_memory_scope, peak_rss_bytes) = match process.counts.peak_scope {
        PeakScope::Tree => (ResourceScope::Cgroup, Measure::Observed { value: process.counts.peak_rss_bytes }),
        PeakScope::LargestProcess { baseline_bytes } => {
            if process.counts.peak_rss_bytes > baseline_bytes {
                (
                    ResourceScope::LargestProcess,
                    Measure::LowerBound {
                        value: process.counts.peak_rss_bytes,
                        missing: vec![
                            "pidfd walk retains the largest process peak, not simultaneous tree memory".into(),
                        ],
                    },
                )
            } else {
                (
                    ResourceScope::Unavailable,
                    Measure::unavailable("prior setup children establish the walk's unreducible peak baseline"),
                )
            }
        }
    };
    Ok(Resources {
        cpu_scope,
        cpu_ms: Measure::Observed { value: milliseconds(cpu) },
        peak_memory_scope,
        peak_rss_bytes,
        own_peak_rss_bytes: Measure::unavailable("not reported"),
    })
}

fn archive(guard: &Guard, directory: &Path, result: &mut AttemptResult, name: &str, bytes: &[u8]) -> Result<(), Fault> {
    result.artifacts.push(crate::grade::archive(guard, directory, name, bytes)?);
    Ok(())
}

/// The attempt's archive directory (../../docs/design/benchmarks/README.md, 10).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Archive<'a> {
    pub(crate) guard: &'a Guard,
    pub(crate) directory: &'a Path,
}

impl Archive<'_> {
    pub(crate) fn put(&self, result: &mut AttemptResult, name: &str, bytes: &[u8]) -> Result<(), Fault> {
        archive(self.guard, self.directory, result, name, bytes)
    }
}

fn seed_commit(
    clock: &Clock,
    guard: &Guard,
    machine: &Machine,
    private: &Private,
    prepared: &Request,
) -> Result<(), Fault> {
    let path = guard.write_path(&private.workspace.join(".git"))?;
    refuse_if(path.exists(), Input, "seed must not contain git metadata")?;
    for arguments in [
        vec!["init", "--quiet"],
        vec!["add", "--", "."],
        vec!["commit", "--quiet", "--allow-empty", "-m", "Frozen task seed"],
    ] {
        let mut command: Vec<String> = [
            "git",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.autocrlf=false",
            "-c",
            "gc.auto=0",
            "-c",
            "maintenance.auto=false",
            "-c",
            "user.name=Smith benchmark",
            "-c",
            "user.email=smith-bench@example.invalid",
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        command.extend(arguments.into_iter().map(String::from));
        let result = raw(clock, machine, private, &command, prepared.setup_deadline_seconds, prepared)?;
        refuse_if(!succeeded(&result), Bound, "bounded seed git setup failed")?;
    }
    Ok(())
}

fn copy_cache(
    clock: &Clock,
    guard: &Guard,
    machine: &Machine,
    private: &Private,
    prepared: &Request,
) -> Result<(), Fault> {
    for (source, destination) in [(&private.warm_cargo, &private.cargo_home), (&private.warm_target, &private.target)] {
        // A source has been warmed by this attempt's own setup, never borrowed
        // from a user's Cargo home. Validate before cp can follow anything.
        let files = crate::grade::snapshot_all(source)?;
        if files
            .values()
            .any(|entry| !matches!(entry, crate::grade::Entry::File { .. } | crate::grade::Entry::Directory))
        {
            return Err(input("warm caches contain a link or special node"));
        }
        guard.write_path(destination)?;
        let command = vec![
            "cp".into(),
            "--recursive".into(),
            "--reflink=auto".into(),
            "--no-preserve=ownership".into(),
            "--".into(),
            source.join(".").to_str().ok_or_else(|| format("cache path must be UTF-8"))?.into(),
            destination.to_str().ok_or_else(|| format("cache path must be UTF-8"))?.into(),
        ];
        let result = raw(clock, machine, private, &command, prepared.setup_deadline_seconds, prepared)?;
        refuse_if(!succeeded(&result), Input, "private writable cache copy failed")?;
        crate::isolate::restrict_tree(guard, destination)?;
    }
    Ok(())
}

fn task_for(prepared: &Request) -> Result<Task, Fault> {
    let mut task = crate::check_task(&prepared.task, &Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/design"))?;
    let kind = match task.kind {
        crate::Kind::Probe => "probes",
        crate::Kind::Fixture => "fixtures",
        crate::Kind::Repository => "repository",
    };
    let base = format!("{kind}/{}", task.id);
    if prepared.identity.task != base {
        let name = prepared
            .identity
            .task
            .strip_prefix(&format!("{base}/"))
            .ok_or_else(|| mismatch("prepared task identity differs"))?;
        task = task.with_variant(name)?;
    }
    Ok(task)
}

fn copy_seed(
    guard: &Guard,
    source: &Path,
    digest: &str,
    root: &Path,
) -> Result<(Private, crate::grade::Snapshot), Fault> {
    let private = crate::isolate::prepare(guard, source, digest, root)?;
    let seed = crate::grade::snapshot(&private.workspace).map_err(Fault::at(Seed))?;
    Ok((private, seed))
}

fn prepare(
    clock: &Clock,
    guard: &Guard,
    machine: &Machine,
    task: &Task,
    task_directory: &Path,
    root: &Path,
    prepared: &Request,
) -> Result<(Private, String, crate::grade::Snapshot), Fault> {
    validate_prepared(task, prepared).map_err(Fault::at(Configuration))?;
    let bundles = crate::arm::state_root(guard).map_err(Fault::at(Seed))?.join("bundles");
    let (seed, digest) = crate::seed_source(task, task_directory, &bundles)?;
    let (private, snapshot) = copy_seed(guard, &seed, &digest, root)?;
    free_space(clock, machine, &private, prepared).map_err(Fault::at(Space))?;
    crate::isolate::environment(machine, &private, &task.environment).map_err(Fault::at(Configuration))?;
    crate::isolate::login_profile(guard, machine, &private).map_err(Fault::at(Configuration))?;
    seed_commit(clock, guard, machine, &private, prepared).map_err(Fault::at(Seed))?;
    let revision = seed_revision(clock, machine, &private, prepared).map_err(Fault::at(Seed))?;
    warm(clock, guard, machine, &private, task, prepared).map_err(Fault::at(Warm))?;
    Ok((private, revision, snapshot))
}

fn validate_prepared(task: &Task, prepared: &Request) -> Result<(), Fault> {
    if task.version != prepared.identity.task_version || task.seed_sha256 != prepared.identity.seed_sha256 {
        return Err(mismatch("prepared identity differs from the frozen task"));
    }
    if task.requested_model(prepared.identity.agent, &prepared.identity.model.requested_model)?
        != prepared.identity.model.requested_model
    {
        return Err(mismatch("prepared model differs from the authored model override"));
    }
    let prompt = task.prompt.as_bytes().to_vec();
    if prompt.is_empty()
        || prompt.len() > 4088
        || !prompt.ends_with(b"\n")
        || prompt[..prompt.len() - 1].contains(&b'\n')
    {
        return Err(input("prompt must be one paragraph ending in a newline, under 4088 bytes"));
    }
    if format!("{:x}", Sha256::digest(&prompt)) != prepared.identity.prompt_sha256 {
        return Err(mismatch("prompt digest differs"));
    }
    if crate::arm::binary_digest(&prepared.program)? != prepared.identity.binary_sha256 {
        return Err(mismatch("prepared binary digest differs"));
    }
    if let Some(reason) = prepared.identity.agent.refusal() {
        return Err(input(reason));
    }
    if prepared.setup_deadline_seconds == 0 || task.deadline_seconds == 0 {
        return Err(input("setup and task require positive author deadlines"));
    }
    for seconds in std::iter::once(prepared.setup_deadline_seconds)
        .chain(std::iter::once(task.deadline_seconds))
        .chain(task.grade.iter().map(|grade| grade.deadline_seconds))
    {
        if seconds == 0 || seconds.checked_mul(1_000_000_000).is_none() {
            return Err(bound("author deadline is outside the clock's range"));
        }
    }
    for milliseconds in [prepared.teardown_ms, prepared.grace_ms, prepared.settlement_ms] {
        if milliseconds.checked_mul(1_000_000).is_none() {
            return Err(bound("supervision policy is outside the clock's range"));
        }
    }
    Ok(())
}

fn seed_revision(clock: &Clock, machine: &Machine, private: &Private, prepared: &Request) -> Result<String, Fault> {
    let commit = raw(
        clock,
        machine,
        private,
        &["git".into(), "rev-parse".into(), "HEAD".into()],
        prepared.setup_deadline_seconds,
        prepared,
    )?;
    refuse_if(!succeeded(&commit), Io, "cannot retain frozen seed commit")?;
    Ok(String::from_utf8(commit.stdout)?.trim().to_owned())
}

fn warm(
    clock: &Clock,
    guard: &Guard,
    machine: &Machine,
    private: &Private,
    task: &Task,
    prepared: &Request,
) -> Result<(), Fault> {
    let mut warming = private.clone();
    warming.cargo_home.clone_from(&private.warm_cargo);
    warming.target.clone_from(&private.warm_target);
    for command in &task.setup {
        let result = raw_environment(
            clock,
            machine,
            &warming,
            command,
            prepared,
            CommandPolicy { deadline_seconds: prepared.setup_deadline_seconds, environment: &task.environment },
        )?;
        if !succeeded(&result) {
            return Err(io(format!("setup command failed: {:?}, {:?}", result.exit, result.error)));
        }
    }
    copy_cache(clock, guard, machine, private, prepared)?;
    if !task.grade.is_empty() {
        let mut grading = crate::isolate::empty(guard, &private.root.join("grading"))?;
        crate::isolate::login_profile(guard, machine, &grading)?;
        grading.warm_cargo.clone_from(&private.warm_cargo);
        grading.warm_target.clone_from(&private.warm_target);
        copy_cache(clock, guard, machine, &grading, prepared)?;
    }
    Ok(())
}

fn observe(
    guard: &Guard,
    directory: &Path,
    workspace: &Path,
    result: &mut AttemptResult,
    process: &mut ProcessResult,
    task: &Task,
    maximum: usize,
) -> Result<crate::grade::Ended, Fault> {
    process.observing.as_ref().ok_or_else(|| input("an attempt has its observer"))?;
    result.identity.started_utc =
        time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(process.start_wall.as_nanos()))?
            .format(&time::format_description::well_known::Rfc3339)?;
    let archives = Archive { guard, directory };
    let traces = crate::agent::Traces { trace: &process.trace, stdout: &process.stdout, maximum };
    let observing = process.observing.as_mut().expect("attempt observer checked above");
    observing.archive_traces(archives, result, traces)?;
    observing.finish()?;
    let classification = observing.classify(process.exit, process.forced);
    result.outcome.end = if process.error.is_none() {
        observing.refused(task, process.exit, process.forced, &process.stderr)
    } else {
        None
    }
    .unwrap_or(classification.end);
    result.outcome.warnings = classification.warnings;
    result.outcome.exit = Measure::Observed { value: process.exit };
    result.outcome.forced = process.forced;
    if let Some(error) = &process.error {
        result.outcome.end = End::HarnessError { what: error.clone() };
    }
    result.timing.process_wall_ms = Measure::Observed { value: elapsed(process.start, process.end) };
    if let Some(answered) = process.answered {
        result.timing.task_wall_ms = Measure::Observed { value: elapsed(process.start, answered) };
        result.timing.teardown_ms = Measure::Observed { value: elapsed(answered, process.end) };
    }
    archive(guard, directory, result, "stdout.log", &process.stdout)?;
    archive(guard, directory, result, "stderr.log", &process.stderr)?;
    observing.archive_trace(archives, result, traces)?;
    let credential_refresh = Measure::unavailable("the credentials pair has not settled");
    observing.settle(
        archives,
        result,
        crate::agent::Ended { exit: process.exit, forced: process.forced, task, workspace },
    )?;
    result.resources = resources(process)?;
    let observing = process.observing.as_ref().expect("attempt observer checked above");
    observing.stream_measures(result)?;
    process.observing.as_mut().expect("attempt observer checked above").collect(archives, result)?;
    result.cache = cache::observation(process, task);
    if result.cache.is_some()
        && let Some(check) = process.observing.as_ref().expect("attempt observer checked above").cache_check()
    {
        result.outcome.checks.push(check);
    }
    result.outcome.writes_outside_workspace =
        process.observing.as_ref().expect("attempt observer checked above").writes_outside(workspace);
    result.health.passive_checks.push(passive::scope(&result.outcome.writes_outside_workspace));
    result.health.passive_checks.push(passive::tree(process));
    Ok(settled_outcome(process, &result.outcome, credential_refresh))
}

fn settled_outcome(
    process: &ProcessResult,
    outcome: &Outcome,
    credential_refresh: Measure<bool>,
) -> crate::grade::Ended {
    let answer = if matches!(outcome.end, End::Completed) {
        process.observing.as_ref().and_then(|observing| observing.answer())
    } else {
        None
    };
    let last_line = if process.error.is_none() {
        std::str::from_utf8(&process.stderr).ok().and_then(|text| text.lines().last()).map(str::to_owned)
    } else {
        None
    };
    crate::grade::Ended { end: outcome.end.clone(), exit: outcome.exit.clone(), last_line, answer, credential_refresh }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Work<'a> {
    pub(crate) clock: &'a Clock,
    pub(crate) guard: &'a Guard,
    pub(crate) machine: &'a Machine,
    pub(crate) private: &'a Private,
    pub(crate) task: &'a Task,
    pub(crate) prepared: &'a Request,
    pub(crate) directory: &'a Path,
}

fn credential_lifetime(work: Work<'_>) -> Result<Duration, Fault> {
    Duration::from_secs(work.task.deadline_seconds)
        .checked_add(Duration::from_millis(work.prepared.grace_ms))
        .and_then(|span| span.checked_add(Duration::from_millis(work.prepared.settlement_ms)))
        .ok_or_else(|| bound("credential lifetime overflow"))
}

type Invocation = (Command, Capture, Duration, Option<ProcessResult>);

fn invocation(work: Work<'_>, result: &mut AttemptResult) -> Result<Invocation, Fault> {
    let mut environment = crate::isolate::environment(work.machine, work.private, &work.task.environment)?;
    let Some(pin_file) = &work.prepared.configuration else {
        if work.prepared.logins.is_some() {
            return Err(input("credentialed attempts require a committed adapter configuration"));
        }
        return Ok((
            Command {
                program: work.prepared.program.clone(),
                arguments: work.prepared.arguments.iter().map(Into::into).collect(),
                environment,
                directory: work.private.workspace.clone(),
            },
            Capture::Agent(Box::new(crate::agent::Observing::replay())),
            Duration::from_millis(work.prepared.teardown_ms),
            None,
        ));
    };
    if !work.prepared.arguments.is_empty() && !result.identity.agent.takes_arguments(&work.prepared.arguments) {
        return Err(input("pinned adapters render their own arguments"));
    }
    if crate::arm::binary_digest(&work.prepared.program)? != result.identity.binary_sha256 {
        return Err(mismatch("installed executable changed since the attempt was planned"));
    }
    let pin = crate::read_configuration(pin_file)?;
    if pin.sha256 != result.identity.configuration_sha256 || pin.pin.agent != result.identity.agent {
        return Err(mismatch("adapter configuration differs from the planned identity"));
    }
    let (arm, version) = result.identity.agent.source_arm(work, &result.identity)?;
    let source = work
        .prepared
        .logins
        .as_deref()
        .ok_or_else(|| credential("the adapter requires its read-only source login directory"))?;
    let home = result.identity.agent.home(source)?;
    let model = crate::ModelChoice {
        model: Some(result.identity.model.requested_model.clone()),
        effort: Some(result.identity.model.requested_effort.clone()),
    };
    let context = crate::agent::Context {
        guard: work.guard,
        private: work.private,
        pin: &pin,
        pin_file,
        task: work.task,
        variant: None,
        model: &model,
        arm: &arm,
        installed_version: &version,
        home: &home,
        wall: work.clock.now().wall,
        lifetime: credential_lifetime(work)?,
        environment: &environment,
    };
    let (prepared, receipt) = result.identity.agent.prepare_checked(&context, &work.prepared.arguments)?;
    prepared.check_identity(&result.identity)?;
    prepared.archive(Archive { guard: work.guard, directory: work.directory }, result)?;
    if let Some(receipt) = &receipt
        && !work.task.event.contains(&crate::EventCheck::NoRun)
    {
        return Err(input(format!("the adapter's check refused setup with {:?}", receipt.exit)));
    }
    let mut invocation = prepared.command();
    crate::batch::settings_arguments(&mut invocation.arguments, &work.prepared.arguments);
    environment.extend(invocation.environment);
    Ok((
        Command {
            program: invocation.program,
            arguments: invocation.arguments,
            environment,
            directory: work.private.workspace.clone(),
        },
        Capture::Agent(Box::new(prepared.observe()?)),
        invocation.teardown,
        receipt,
    ))
}

fn afterwards(
    work: Work<'_>,
    seed: &crate::grade::Snapshot,
    revision: String,
    result: &mut AttemptResult,
    ended: &crate::grade::Ended,
) -> Result<(), Fault> {
    let Work { clock, guard, machine, private, task, prepared, directory } = work;
    let after = crate::grade::snapshot(&private.workspace)?;
    let file_checks: Vec<_> = task.outcome.iter().filter(|check| !outcomes::neutral(check)).cloned().collect();
    result.outcome.checks.extend(crate::grade::checks(&private.workspace, seed, &after, &file_checks, ended));
    let changed: Vec<_> =
        task.protected.iter().filter(|path| crate::grade::changed(seed, &after, path)).cloned().collect();
    result.outcome.protected_files_changed = Measure::Observed { value: changed };
    let git_files = crate::grade::snapshot_all(&private.workspace.join(".git"))?;
    if git_files
        .values()
        .any(|entry| !matches!(entry, crate::grade::Entry::File { .. } | crate::grade::Entry::Directory))
    {
        return Err(input("agent replaced git metadata with a link or special file"));
    }
    let include = raw(
        clock,
        machine,
        private,
        &[
            "git".into(),
            "-c".into(),
            "core.hooksPath=/dev/null".into(),
            "add".into(),
            "--intent-to-add".into(),
            "--".into(),
            ".".into(),
        ],
        prepared.setup_deadline_seconds,
        prepared,
    )?;
    refuse_if(!succeeded(&include), Input, "cannot include untracked files in seed diff")?;
    let diff = raw(
        clock,
        machine,
        private,
        &[
            "git".into(),
            "-c".into(),
            "core.hooksPath=/dev/null".into(),
            "diff".into(),
            "--no-ext-diff".into(),
            "--no-textconv".into(),
            "--binary".into(),
            revision,
            "--".into(),
        ],
        prepared.setup_deadline_seconds,
        prepared,
    )?;
    refuse_if(!succeeded(&diff), Input, "seed diff collection failed")?;
    archive(guard, directory, result, "diff.patch", &diff.stdout)?;
    if !task.grade.is_empty() {
        let grading_private = Private::paths(&private.root.join("grading"));
        let grading_root = &grading_private.root;
        let grading = &grading_private.workspace;
        crate::isolate::copy_into(guard, &private.workspace, grading)?;
        let grader = prepared.task.parent().ok_or_else(|| input("task needs a directory"))?.join("grader");
        if grader.exists() {
            crate::isolate::copy_directory(guard, &grader, &grading_root.join("grader"))?;
        }
        let grade_start = clock.now().now;
        let mut passed = true;
        for (index, grade) in task.grade.iter().enumerate() {
            let judgment = crate::grade::command(
                clock,
                machine,
                &grading_private,
                grading,
                grade,
                prepared.bounds(Duration::ZERO, Duration::ZERO),
                &task.environment,
            )?;
            archive(guard, directory, result, &format!("grade-{index}-stdout.log"), &judgment.stdout)?;
            archive(guard, directory, result, &format!("grade-{index}-stderr.log"), &judgment.stderr)?;
            match judgment.end {
                crate::grade::GradeEnd::Passed => {}
                crate::grade::GradeEnd::Failed => passed = false,
                crate::grade::GradeEnd::Fault { reason } => {
                    result.outcome.grade = Some(Measure::unavailable(reason.clone()));
                    result.timing.grading_wall_ms = Measure::Observed { value: elapsed(grade_start, clock.now().now) };
                    return Err(io(reason));
                }
            }
        }
        result.outcome.grade = Some(Measure::Observed { value: passed });
        result.timing.grading_wall_ms = Measure::Observed { value: elapsed(grade_start, clock.now().now) };
    }
    Ok(())
}

/// Run exactly one prepared invocation in this process and archive its result.
/// Setup failures are recorded as refusals and make the enclosing suite incomplete.
pub fn attempt(guard: &Guard, machine: &Machine, state: &Path, prepared: &Request) -> Result<Recorded, Fault> {
    let root = scratch_root(state, prepared)?;
    let temporary_before = crate::isolate::temporary_entries(&prepared.shared_temporary, &root);
    private_parent(guard, root.parent().expect("scratch parent"))?;
    let archive_root = state.join("runs").join(&prepared.identity.run);
    private_parent(guard, &archive_root)?;
    let directory = crate::isolate::create_directory(guard, &archive_root.join(&prepared.identity.attempt))?;
    let clock = Clock::new();
    let existed = root.exists();
    let mut result = blank(prepared.identity.clone());
    let mut inference = false;
    let mut neutral = outcomes::Neutral::default();
    let work = if existed {
        Err(Fault::new(FaultKind::Setup(Scratch), "attempt scratch root already exists; setup refused"))
    } else {
        attempt_work(
            Archive { guard, directory: &directory },
            machine,
            (&clock, &root),
            prepared,
            &mut result,
            &mut inference,
            &mut neutral,
        )
    };
    let incomplete = match work {
        Ok(()) => Incomplete::No,
        Err(error) => {
            let incomplete = if inference {
                Incomplete::No
            } else {
                match error.kind {
                    FaultKind::Setup(step) => Incomplete::Setup(step),
                    _ => unreachable!("preparation calls label their faults"),
                }
            };
            result.outcome.end =
                if inference { End::HarnessError { what: error.detail } } else { End::Refused { setup: error.detail } };
            incomplete
        }
    };
    if !existed && root.exists() {
        let deletion = crate::isolate::remove_owned_tree(guard, &root);
        if let Err(error) = deletion {
            result.outcome.end = End::HarnessError { what: format!("owned workspace deletion failed: {error}") };
        }
    }
    passive::temporary(
        &mut result,
        &prepared.shared_temporary,
        crate::isolate::temporary_changes(temporary_before, &prepared.shared_temporary, &root),
    );
    neutral.finish(&mut result);
    let bytes = crate::render_result(&result)?;
    let _: Artifact = crate::grade::archive(guard, &directory, "result.json", bytes.as_bytes())?;
    Ok(Recorded { result, directory, incomplete })
}

fn archive_task(
    archive: Archive<'_>,
    result: &mut AttemptResult,
    prepared: &Request,
    task: &Task,
) -> Result<(), Fault> {
    archive.put(result, "prepared.json", &serde_json::to_vec_pretty(prepared)?)?;
    archive.put(result, "task.toml", &fs::read(&prepared.task)?)?;
    archive.put(result, "prompt.txt", task.prompt.as_bytes())
}

fn attempt_work(
    archive: Archive<'_>,
    machine: &Machine,
    scratch: (&Clock, &Path),
    prepared: &Request,
    result: &mut AttemptResult,
    inference: &mut bool,
    neutral: &mut outcomes::Neutral,
) -> Result<(), Fault> {
    let Archive { guard, directory } = archive;
    let (clock, root) = scratch;
    let task = task_for(prepared).map_err(Fault::at(Seed))?;
    neutral.task(&task, prepared.identity.agent);
    archive_task(archive, result, prepared, &task).map_err(Fault::at(Seed))?;
    let task_directory =
        prepared.task.parent().ok_or_else(|| Fault::new(FaultKind::Setup(Seed), "task needs a directory"))?;
    let (private, revision, seed) = prepare(clock, guard, machine, &task, task_directory, root, prepared)?;
    let work = Work { clock, guard, machine, private: &private, task: &task, prepared, directory };
    let (command, capture, teardown, receipt) = invocation(work, result).map_err(Fault::at(Configuration))?;
    let script = script(&task, &capture);
    let mut process = if let Some(mut receipt) = receipt {
        let Capture::Agent(observing) = capture else {
            return Err(Fault::new(FaultKind::Setup(Configuration), "a checked adapter needs its observer"));
        };
        receipt.observing = Some(observing);
        receipt
    } else {
        crate::process::execute_script(
            clock,
            command,
            script,
            capture,
            prepared.bounds(Duration::from_secs(task.deadline_seconds), teardown),
        )
        .map_err(Fault::at(Configuration))?
    };
    *inference = true;
    let observed =
        observe(guard, directory, &private.workspace, result, &mut process, &task, prepared.max_stdout_bytes);
    let ended = neutral.observed(observed, &process, &result.outcome)?;
    afterwards(work, &seed, revision, result, ended)
}

/// Put a task's authored lines behind its prompt, preserving pipe input closure.
#[must_use]
pub fn script(task: &Task, capture: &Capture) -> crate::agent::Script {
    let default = match capture {
        Capture::Agent(observing) => observing.close(),
        Capture::Raw => crate::agent::Close::AfterPrompt,
    };
    let mut cues = vec![crate::agent::Cue { when: crate::agent::Trigger::Start, text: task.prompt.clone() }];
    cues.extend(task.person.clone());
    crate::agent::Script { cues, close: task.input_end.unwrap_or(default) }
}

/// Archive a dispatcher-observed failure without inventing agent measurements.
pub(crate) fn recover(
    guard: &Guard,
    state: &Path,
    prepared: &Request,
    process: &ProcessResult,
    reason: String,
) -> Result<AttemptResult, Fault> {
    let directory = state.join("runs").join(&prepared.identity.run).join(&prepared.identity.attempt);
    private_parent(guard, &directory)?;
    let mut result = blank(prepared.identity.clone());
    result.outcome.end = End::HarnessError { what: reason };
    result.outcome.forced = process.forced;
    // Keep the digest of any complete archive the interrupted child wrote.
    // The dispatcher still supplies no agent measurements from its own tree.
    for (path, entry) in crate::grade::snapshot_all(&directory)? {
        match entry {
            crate::grade::Entry::File { sha256, .. } => result.artifacts.push(Artifact { path, sha256 }),
            crate::grade::Entry::Directory => {}
            crate::grade::Entry::Link { .. } | crate::grade::Entry::Special => {
                return Err(input("interrupted attempt archive contains a link or special node"));
            }
        }
    }
    result.health.passive_checks.push(crate::CheckVerdict {
        check: "dispatcher-tree-settled".into(),
        passed: Measure::Observed { value: true },
        evidence: vec![format!("{:?}", process.tree)],
    });
    archive(guard, &directory, &mut result, "dispatcher-stdout.log", &process.stdout)?;
    archive(guard, &directory, &mut result, "dispatcher-stderr.log", &process.stderr)?;
    let bytes = crate::render_result(&result)?;
    crate::grade::archive(guard, &directory, "result.json", bytes.as_bytes())?;
    Ok(result)
}

/// Read an adapter-produced request strictly, without reading any login.
pub fn read_prepared(path: &Path) -> Result<Request, Fault> {
    let text = fs::read_to_string(path)?;
    let mut decoder = serde_json::Deserializer::from_str(&text);
    let request = serde_path_to_error::deserialize(&mut decoder)
        .map_err(|error| format(format!("{}: {error}", path.display())))?;
    decoder.end()?;
    Ok(request)
}

/// Execute one already-prepared request in this fresh harness process.
pub fn attempt_file(path: &Path) -> Result<Recorded, Fault> {
    let guard = Guard::from_environment()?;
    let machine = Machine::from_environment()?;
    let state = crate::arm::state_root(&guard)?;
    attempt(&guard, &machine, &state, &read_prepared(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_seed_snapshot_comes_from_the_copied_source() {
        let scratch = skein_scratch::Scratch::new("bench-run");
        let root = scratch.path().to_path_buf().join(format!("smith-bench-copied-source-{}", std::process::id()));
        fs::create_dir(&root).expect("fixture root");
        let guard = Guard::new(&root.join("home"), &root.join("config"), &root.join("state"), &[]).expect("guard");
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/recorded/bundles/tiny");
        let digest = crate::seed_digest(&source).expect("source digest");
        let (private, snapshot) = copy_seed(&guard, &source, &digest, &root.join("attempt")).expect("copied seed");
        assert_eq!(snapshot, crate::grade::snapshot(&source).expect("bundle snapshot"));
        assert!(snapshot.contains_key(Path::new("README")));
        assert!(!snapshot.contains_key(Path::new("bundle.toml")));
        fs::write(private.workspace.join("Cargo.toml"), "setup-created").expect("setup edit");
        assert!(!snapshot.contains_key(Path::new("Cargo.toml")));
    }
}
