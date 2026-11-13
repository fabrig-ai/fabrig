//! Command options select frozen plans and settled evidence reports.
//! No agent state is read; benchmark requests and settled public evidence are inputs.
//! Contract: smith/docs/design/benchmarks/README.md, section 5.1, and smith/docs/design/benchmarks/catalogue.md, section 5.

use super::{
    Order, ProbeAgent, Source, archived, check_parent, codex, finish_calibration, gating, judgment, plan, prepare,
    summary, verdict_name,
};
use crate::arm::ArmRef;
use crate::guard::Guard;
use crate::{Fault, Input, Judgment, Suite, format, input, refuse_if};
use std::{fs, path::Path};

/// Execute a parsed command with frozen arms, printing each settled attempt and summary.
pub fn command(command: &str, arguments: &[String]) -> Result<Judgment, Fault> {
    let (name, flags) = arguments.split_first().ok_or_else(|| input("a task or suite name is required"))?;
    let (change_inputs, flags) = if command == "suite" {
        crate::parity::change::Inputs::parse(flags).map_err(input)?
    } else {
        (crate::parity::change::Inputs::default(), flags.to_vec())
    };
    refuse_if(flags.len() % 2 != 0, Input, "options require values")?;
    let allowed: &[&str] = match command {
        "probe" => &["--agent", "--attempts", "--commit", "--arm", "--shared-temporary"],
        "calibrate" => &["--attempts", "--passes", "--control", "--commit", "--arm"],
        "fix" => &["--parent", "--commit"],
        "suite" => &["--attempts", "--seed", "--commit", "--arm", "--include", "--parent", "--parent-arm"],
        _ => return Err(input("unknown probe command")),
    };
    let mut options = std::collections::BTreeMap::new();
    let mut asked = Vec::new();
    for pair in flags.as_chunks::<2>().0 {
        if command == "suite" && pair[0] == "--include" {
            asked.push(pair[1].clone());
            continue;
        }
        if !allowed.contains(&pair[0].as_str()) || options.insert(pair[0].as_str(), pair[1].as_str()).is_some() {
            return Err(input("unknown or repeated command option"));
        }
    }
    refuse_if(options.contains_key("--commit") && options.contains_key("--arm"), Input, "choose commit or arm")?;
    if options.contains_key("--parent") && options.contains_key("--parent-arm") {
        return Err(input("choose parent commit or parent arm"));
    }
    let agent = match options.get("--agent").copied().unwrap_or("smith") {
        "smith" => ProbeAgent::Smith,
        "codex" if command == "probe" => ProbeAgent::Codex,
        "smith-claude-code" if command == "probe" => ProbeAgent::SmithClaudeCode,
        _ => return Err(input("this command requires smith (probe also accepts codex or smith-claude-code)")),
    };
    let source = options.get("--commit").map_or(Source::Head, |commit| Source::Commit((*commit).into()));
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().ok_or_else(|| input("workspace missing"))?;
    if command == "suite" && name == "smoke" {
        let guard = Guard::from_environment()?;
        let directory = crate::arm::state_root(&guard)?.join("logins/smith");
        crate::credential::logins::smith(&directory, "chatgpt")?;
    }
    if command == "suite" {
        let suite = crate::read_suite(&repository.join("benchmarks/suites").join(format!("{name}.toml")))?;
        check_parent(&suite, options.contains_key("--parent") || options.contains_key("--parent-arm"))?;
        refuse_if(!suite.change && !change_inputs.is_empty(), Input, "named run inputs require a change suite")?;
        if suite.change && !options.contains_key("--commit") && !options.contains_key("--arm") {
            return Err(input("change suite requires --commit or --arm"));
        }
    }
    if agent == ProbeAgent::Codex || !options.contains_key("--arm") {
        crate::arm::resolve_commit(repository, "HEAD")?;
    }
    let arm = if agent == ProbeAgent::Codex {
        codex(repository, "standard")?
    } else if let Some(directory) = options.get("--arm") {
        ArmRef::Built(crate::arm::read_build(&Path::new(directory).join("smith"))?)
    } else {
        archived(repository, &source)?
    };
    let seed = options.get("--seed").map_or_else(
        || Ok(skein_shell::Clock::new().now().wall.as_nanos()),
        |seed| seed.parse::<u64>().map_err(|error| input(error.to_string())),
    )?;
    println!("seed={seed}");
    let attempts = options.get("--attempts").map_or(Ok(if command == "calibrate" { 12 } else { 1 }), |value| {
        value.parse::<u32>().map_err(|error| input(error.to_string()))
    })?;
    refuse_if(attempts == 0 || attempts > 12, Input, "attempts must be between one and twelve")?;
    let order = Order {
        probe: name.clone(),
        agent,
        attempts,
        source,
        shared_temporary: super::plan::planned_temporary(options.get("--shared-temporary").copied()),
    };
    match command {
        "probe" => {
            let planned = plan(repository, &order, &arm)?;
            let batch = crate::batch::run_probe(&planned.suite, &planned.directory, seed, name, attempts)?;
            Ok(summary(&planned, &batch))
        }
        "suite" => suite_command(repository, name, &arm, &options, (&asked, &change_inputs), seed, attempts),
        "calibrate" | "fix" => comparison(command, repository, &order, &arm, &options, seed),
        _ => Err(input("unknown probe command")),
    }
}

pub(super) fn suite_attempts(value: &mut toml::Value, attempts: u32) -> Result<(), Fault> {
    value
        .as_table_mut()
        .ok_or_else(|| format("suite must be a table"))?
        .insert("attempts".into(), toml::Value::Integer(i64::from(attempts)));
    Ok(())
}

fn suite_command(
    repository: &Path,
    name: &str,
    arm: &ArmRef,
    options: &std::collections::BTreeMap<&str, &str>,
    selection: (&[String], &crate::parity::change::Inputs),
    seed: u64,
    attempts: u32,
) -> Result<Judgment, Fault> {
    let (asked, inputs) = selection;
    let file = repository.join("benchmarks/suites").join(format!("{name}.toml"));
    let mut value: toml::Value = crate::formats::read_document(&file)?;
    let suite: Suite = value.clone().try_into()?;
    if options.contains_key("--attempts") {
        suite_attempts(&mut value, attempts)?;
    }
    let parent = if let Some(directory) = options.get("--parent-arm") {
        Some(ArmRef::Built(crate::arm::read_build(&Path::new(directory).join("smith"))?))
    } else if let Some(commit) = options.get("--parent") {
        Some(archived(repository, &Source::Commit((*commit).into()))?)
    } else {
        None
    };
    let planned = prepare(
        repository,
        name,
        &value,
        (arm, parent.as_ref()),
        false,
        asked,
        Path::new(crate::run::MACHINE_TEMPORARY),
    )?;
    if name == "smoke" && planned.requests.is_empty() {
        println!("summary {} attempts=0 verdict=incomplete", planned.run);
        return Ok(Judgment::Incomplete);
    }
    let small = planned.requests.iter().try_fold(false, |small, request| -> Result<bool, Fault> {
        let task = crate::read_task(&request.task)?;
        Ok(small
            || options.contains_key("--attempts")
                && attempts < suite.task_repetitions(&request.identity.task, task.repetitions, false)?)
    })?;
    let batch = crate::batch::run(&planned.suite, &planned.directory, seed)?;
    if small || suite.change {
        let guard = Guard::from_environment()?;
        let directory = crate::arm::state_root(&guard)?.join("runs").join(&planned.run);
        return if suite.change {
            change_summary(
                &batch,
                &suite,
                seed,
                &directory,
                inputs,
                small || batch.results.len() < planned.requests.len(),
            )
        } else {
            small_summary(&batch, &suite, seed, &directory)
        };
    }
    if name == "parity" {
        return parity_summary(repository, &batch, seed);
    }
    if name == "probes" {
        let verdict = gating::judgment(&batch, &gating::uncalibrated(&planned.requests)?);
        println!("summary {} attempts={} verdict={}", planned.run, batch.results.len(), verdict_name(verdict));
        return Ok(verdict);
    }
    Ok(summary(&planned, &batch))
}

pub(super) fn small_summary(
    batch: &crate::batch::Batch,
    suite: &Suite,
    seed: u64,
    directory: &Path,
) -> Result<Judgment, Fault> {
    let summary =
        crate::summarise(&batch.results, &suite.name, &suite.tier, suite.design, seed, None, suite.reference_arm())?;
    let path = directory.join("summary.json");
    fs::create_dir_all(path.parent().ok_or_else(|| input("run directory missing"))?)?;
    crate::write_summary(&path, &summary)?;
    println!("{}", path.display());
    println!("summary {} attempts={} verdict=incomplete", summary.run, batch.results.len());
    Ok(Judgment::Incomplete)
}

fn change_summary(
    batch: &crate::batch::Batch,
    suite: &Suite,
    seed: u64,
    directory: &Path,
    inputs: &crate::parity::change::Inputs,
    small: bool,
) -> Result<Judgment, Fault> {
    let summary =
        crate::summarise(&batch.results, &suite.name, &suite.tier, suite.design, seed, None, suite.reference_arm())?;
    settle_change(summary, directory, inputs, small)
}

pub(super) fn settle_change(
    mut summary: crate::RunSummary,
    directory: &Path,
    inputs: &crate::parity::change::Inputs,
    small: bool,
) -> Result<Judgment, Fault> {
    crate::parity::change::attach(&mut summary, inputs)?;
    let path = directory.join("summary.json");
    fs::create_dir_all(directory)?;
    crate::write_summary(&path, &summary)?;
    let code = summary.change.as_ref().expect("change section").code();
    let verdict = if small {
        Judgment::Incomplete
    } else {
        match code {
            0 => Judgment::Pass,
            1 => Judgment::Fail,
            _ => Judgment::Incomplete,
        }
    };
    println!("{}", path.display());
    println!("summary {} attempts={} verdict={}", summary.run, summary.attempts.len(), verdict_name(verdict));
    Ok(verdict)
}

fn parity_summary(repository: &Path, batch: &crate::batch::Batch, seed: u64) -> Result<Judgment, Fault> {
    let summary = crate::summarise(&batch.results, "parity", "working", crate::Design::Interleaved, seed, None, None)?;
    let directory = repository.join("benchmarks/summaries");
    fs::create_dir_all(&directory)?;
    crate::write_summary(&directory.join(format!("{}.json", summary.run)), &summary)?;
    let parity = summary.parity.expect("parity suite section");
    let verdict = match parity.verdict {
        crate::parity::Verdict::OnPar => Judgment::Pass,
        crate::parity::Verdict::NotOnPar => Judgment::Fail,
        crate::parity::Verdict::Incomplete => Judgment::Incomplete,
    };
    println!("summary {} attempts={} verdict={}", summary.run, batch.results.len(), verdict_name(verdict));
    Ok(verdict)
}

fn comparison(
    command: &str,
    repository: &Path,
    order: &Order,
    arm: &ArmRef,
    options: &std::collections::BTreeMap<&str, &str>,
    seed: u64,
) -> Result<Judgment, Fault> {
    let required = options
        .get("--passes")
        .map_or(Ok(10), |passes| passes.parse::<u32>().map_err(|error| input(error.to_string())))?;
    if command == "calibrate" && (required != 10 || required > order.attempts) {
        return Err(input("calibration requires ten passes within twelve attempts"));
    }
    let control = options.get(if command == "fix" { "--parent" } else { "--control" });
    refuse_if(command == "fix" && control.is_none(), Input, "fix requires --parent COMMIT")?;
    let planned = plan(repository, order, arm)?;
    let after_build = crate::fix::prepared(&planned.suite, &planned.directory, seed, &order.probe, order.attempts)?;
    let before = control
        .map(|commit| -> Result<_, Fault> {
            let before_arm = archived(repository, &Source::Commit((*commit).into()))?;
            if let (ArmRef::Built(before), ArmRef::Built(after)) = (&before_arm, arm)
                && before.arm.commit == after.arm.commit
            {
                return Err(input("control must differ from the changed arm"));
            }
            let ceiling = if command == "calibrate" { 2 } else { 1 };
            let one = Order {
                probe: order.probe.clone(),
                agent: ProbeAgent::Smith,
                attempts: ceiling,
                source: Source::Head,
                shared_temporary: order.shared_temporary.clone(),
            };
            let planned = plan(repository, &one, &before_arm)?;
            let before_build = crate::fix::prepared(&planned.suite, &planned.directory, seed, &order.probe, ceiling)?;
            let batch = if command == "calibrate" {
                crate::batch::run_calibration(&planned.suite, &planned.directory, seed, &order.probe, 2, 2)?
            } else {
                crate::batch::run_probe(&planned.suite, &planned.directory, seed, &order.probe, 1)?
            };
            Ok((before_build, planned, batch))
        })
        .transpose()?;
    if command == "fix" {
        let batch = crate::batch::run_probe(&planned.suite, &planned.directory, seed, &order.probe, 1)?;
        let (before_build, _, before_batch) = before.ok_or_else(|| input("fix lacks parent"))?;
        let finding = crate::fix::report(
            order.probe.clone(),
            crate::fix::collected(&before_build, &before_batch.results, &before_batch)?,
            crate::fix::collected(&after_build, &batch.results, &batch)?,
        );
        print!("{}", crate::fix::body(&finding));
        return Ok(match finding.shown {
            crate::fix::Shown::Effect => Judgment::Pass,
            crate::fix::Shown::FailsAfter | crate::fix::Shown::PassesBefore => Judgment::Fail,
            crate::fix::Shown::BeforeRefused | crate::fix::Shown::Unjudged => Judgment::Incomplete,
        });
    }
    if before.as_ref().is_some_and(|(_, _, batch)| judgment(batch) == Judgment::Pass) {
        println!("control passed both attempts; no calibration written");
        return Ok(Judgment::Fail);
    }
    finish_calibration(repository, order, &planned, before.as_ref(), required, seed)
}
