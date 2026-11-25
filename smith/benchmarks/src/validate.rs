//! Validate hidden graders on private seeds and reference overlays, without an
//! agent or credentials (../../docs/design/benchmarks/README.md, sections 3.1 and 5.2, and ../../docs/design/benchmarks/isolation.md, section 1).

use std::path::{Path, PathBuf};

use skein_lib::Duration;
use skein_shell::Clock;
use skein_world::end_to_end::Command;

use crate::files::{Files, Kind, Native};
use crate::grade::{self, CommandReceipt, Entry, GradeEnd, Snapshot};
use crate::guard::Guard;
use crate::isolate::{Machine, Private};
use crate::process::Bounds;
use crate::{CatalogueTask, Fault, input, io};

/// A task's grader terminals, supplied to the validation command.
#[derive(Debug)]
pub struct Validation {
    pub task: String,
    pub seed: GradeEnd,
    pub reference: GradeEnd,
}

impl Validation {
    /// A useful grader rejects its seed and accepts its reference solution.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.seed == GradeEnd::Failed && self.reference == GradeEnd::Passed
    }
}

/// Check the credentials probe against accepted, unchanged and refused pair controls.
/// No login or provider is used by these controls.
pub fn credentials(tasks: &Path) -> Result<(), Fault> {
    let file = tasks.join("probes/credentials/task.toml");
    let design = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/design");
    let task = crate::check_task(&file, &design)?;
    crate::batch::validate_credentials(&task)
}

/// Run named graded tasks, or every graded task, in private command homes.
pub fn validate(tasks: &Path, names: &[String]) -> Result<Vec<Validation>, Fault> {
    let design = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/design");
    let catalogue = crate::catalogue(tasks, &design)?;
    let guard = Guard::from_environment()?;
    let machine = Machine::from_environment()?;
    let state = crate::arm::state_root(&guard)?;
    let scratch = state.join("scratch");
    Native.create_dir_all(&guard.write_path(&scratch)?, 0o700)?;
    let root = crate::isolate::create_directory(&guard, &scratch.join(format!("validate-{}", std::process::id())))?;
    let result = validate_catalogue(&catalogue, names, &guard, &machine, &root);
    crate::isolate::remove_owned_tree(&guard, &root)?;
    result
}

fn validate_catalogue(
    tasks: &[CatalogueTask],
    names: &[String],
    guard: &Guard,
    machine: &Machine,
    root: &Path,
) -> Result<Vec<Validation>, Fault> {
    let clock = Clock::new();
    let world = Candidates { guard, machine, bundles: &|| Ok(crate::arm::state_root(guard)?.join("bundles")) };
    validate_catalogue_with(&Native, tasks, names, &world, root, &mut |command, bounds| {
        grade::run_command(&clock, command, bounds)
    })
}

struct Candidates<'a> {
    guard: &'a Guard,
    machine: &'a Machine,
    bundles: &'a dyn Fn() -> Result<PathBuf, Fault>,
}

fn validate_catalogue_with(
    files: &dyn Files,
    tasks: &[CatalogueTask],
    names: &[String],
    world: &Candidates<'_>,
    root: &Path,
    run: &mut impl FnMut(Command, Bounds) -> Result<CommandReceipt, Fault>,
) -> Result<Vec<Validation>, Fault> {
    for name in names {
        if !tasks.iter().any(|entry| entry.name == *name && !entry.task.grade.is_empty()) {
            return Err(input(format!("unknown graded task {name}")));
        }
    }
    let mut results = Vec::new();
    for (index, entry) in tasks.iter().enumerate() {
        if entry.task.grade.is_empty() || (!names.is_empty() && !names.contains(&entry.name)) {
            continue;
        }
        let seed = candidate(files, world, entry, &root.join(format!("{index}-seed")), false, run)?;
        let reference = candidate(files, world, entry, &root.join(format!("{index}-reference")), true, run)?;
        results.push(Validation { task: entry.name.clone(), seed, reference });
    }
    Ok(results)
}

fn overlay(files: &dyn Files, guard: &Guard, source: &Path, destination: &Path) -> Result<(), Fault> {
    for (relative, entry) in grade::snapshot_all_with(files, source)? {
        let target = guard.write_path_with(files, &destination.join(&relative))?;
        match entry {
            Entry::Directory => {
                if files.metadata(&target).is_err() {
                    crate::isolate::create_directory_with(files, guard, &target)?;
                }
            }
            Entry::File { .. } => {
                files.copy(&source.join(relative), &target)?;
            }
            Entry::Link { .. } | Entry::Special => return Err(input("reference contains a link or special node")),
        }
    }
    Ok(())
}

fn candidate(
    files: &dyn Files,
    world: &Candidates<'_>,
    entry: &CatalogueTask,
    root: &Path,
    reference: bool,
    run: &mut impl FnMut(Command, Bounds) -> Result<CommandReceipt, Fault>,
) -> Result<GradeEnd, Fault> {
    let task_root = entry.manifest.parent().ok_or_else(|| input("task has no directory"))?;
    let bundles = (world.bundles)()?;
    let (seed, digest) = crate::seed_source_with(files, &entry.task, task_root, &bundles)?;
    let guard = world.guard;
    let private = crate::isolate::prepare_with(files, guard, &seed, &digest, root)?;
    let before = grade::snapshot_with(files, &private.workspace)?;
    if reference {
        let reference = task_root.join("reference");
        let patch = reference.join("solution.patch");
        if files.metadata(&patch).is_ok_and(|facts| facts.kind == Kind::File) {
            apply_patch(files, world, &private, &patch, run)?;
        } else {
            overlay(files, guard, &reference, &private.workspace)?;
        }
    }
    let grader = task_root.join("grader");
    if files.metadata(&grader).is_ok() {
        crate::isolate::copy_directory_with(files, guard, &grader, &root.join("grader"))?;
    }
    let bounds = Bounds {
        hard: Duration::ZERO,
        teardown: Duration::ZERO,
        grace: Duration::from_secs(1),
        settlement: Duration::from_secs(3),
        stdout: 0,
        stderr: 0,
        force_walk: true,
    };
    let mut end = GradeEnd::Passed;
    for command in &entry.task.setup {
        let setup = crate::Grade {
            command: command.clone(),
            deadline_seconds: entry.task.deadline_seconds,
            max_output_bytes: 4 * 1024 * 1024,
        };
        let environment = &entry.task.environment;
        let result =
            grade::command_with(files, world.machine, &private, &private.workspace, &setup, bounds, environment, run)?;
        if result.end != GradeEnd::Passed {
            return Err(io(format!(
                "validation setup failed: {:?}: {}",
                result.end,
                String::from_utf8_lossy(&result.stderr)
            )));
        }
    }
    for (index, grade) in entry.task.grade.iter().enumerate() {
        let result = grade::command_with(
            files,
            world.machine,
            &private,
            &private.workspace,
            grade,
            bounds,
            &entry.task.environment,
            run,
        )?;
        if entry.task.kind == crate::Kind::Repository {
            eprintln!(
                "{} {} grade {index}:\n{}{}",
                entry.name,
                if reference { "reference" } else { "seed" },
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
        }
        if reference && result.end != GradeEnd::Passed {
            eprintln!(
                "{} reference grade {index}:\n{}{}",
                entry.name,
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
        }
        match result.end {
            GradeEnd::Failed if end == GradeEnd::Passed => end = GradeEnd::Failed,
            GradeEnd::Passed | GradeEnd::Failed => {}
            GradeEnd::Fault { reason } => end = GradeEnd::Fault { reason },
        }
    }
    if reference {
        reference_checks(files, &entry.task, &private.workspace, &before)?;
    }
    Ok(end)
}

fn reference_checks(
    files: &dyn Files,
    task: &crate::Task,
    settled: &Path,
    before: &Snapshot,
) -> Result<(), crate::Refusal> {
    let after = grade::snapshot_with(files, settled)
        .map_err(|reason| crate::Refusal::new(settled, "reference", reason.detail))?;
    for path in &task.protected {
        if crate::grade::changed(before, &after, path) {
            return Err(crate::Refusal::new(settled, "protected", format!("reference changes {}", path.display())));
        }
    }
    let ended = crate::grade::Ended {
        end: crate::End::Completed,
        exit: crate::Measure::Observed { value: crate::Exit::Code(0) },
        last_line: None,
        answer: None,
        credential_refresh: crate::Measure::Unavailable { reason: "reference runs without credentials".into() },
    };
    for check in grade::checks_with(files, settled, before, &after, &task.outcome, &ended) {
        if check.passed != (crate::Measure::Observed { value: true }) {
            return Err(crate::Refusal::new(settled, "outcome", format!("reference fails {}", check.check)));
        }
    }
    Ok(())
}

fn apply_patch(
    files: &dyn Files,
    world: &Candidates<'_>,
    private: &Private,
    patch: &Path,
    run: &mut impl FnMut(Command, Bounds) -> Result<CommandReceipt, Fault>,
) -> Result<(), Fault> {
    let target = private.root.join("solution.patch");
    files.copy(patch, &world.guard.write_path_with(files, &target)?)?;
    let grade = crate::Grade {
        command: vec!["git".into(), "apply".into(), "../solution.patch".into()],
        deadline_seconds: 10,
        max_output_bytes: 1024 * 1024,
    };
    let bounds = Bounds {
        hard: Duration::ZERO,
        teardown: Duration::ZERO,
        grace: Duration::from_secs(1),
        settlement: Duration::from_secs(3),
        stdout: 0,
        stderr: 0,
        force_walk: true,
    };
    let result = grade::command_with(
        files,
        world.machine,
        private,
        &private.workspace,
        &grade,
        bounds,
        &std::collections::BTreeMap::default(),
        run,
    )?;
    if result.end != GradeEnd::Passed {
        return Err(io(format!("reference patch failed: {}", String::from_utf8_lossy(&result.stderr))));
    }
    Ok(())
}

pub(crate) mod testing {
    use super::{
        Bounds, Candidates, CatalogueTask, Command, CommandReceipt, Fault, Files, Guard, Machine, Path, Validation,
    };

    /// Private scratch and bundle roots and the machine paths supplied to validation replay.
    #[derive(Debug)]
    pub struct ReplayPaths<'a> {
        pub guard: &'a Guard,
        pub machine: &'a Machine,
        pub scratch: &'a Path,
        pub bundles: &'a Path,
    }

    /// Validate real catalogue entries with filesystem and command receipts supplied by the test.
    pub fn validate_catalogue_with(
        files: &dyn Files,
        entries: &[CatalogueTask],
        names: &[String],
        paths: &ReplayPaths<'_>,
        mut run: impl FnMut(Command, Bounds) -> Result<CommandReceipt, Fault>,
    ) -> Result<Vec<Validation>, Fault> {
        let world = Candidates { guard: paths.guard, machine: paths.machine, bundles: &|| Ok(paths.bundles.into()) };
        super::validate_catalogue_with(files, entries, names, &world, paths.scratch, &mut run)
    }

    pub fn validate_catalogue(
        entries: &[CatalogueTask],
        names: &[String],
        guard: &Guard,
        machine: &Machine,
        root: &Path,
    ) -> Result<Vec<Validation>, Fault> {
        super::validate_catalogue(entries, names, guard, machine, root)
    }
}
