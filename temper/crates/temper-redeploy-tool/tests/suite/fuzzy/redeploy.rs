//! A line-log referee independent of the redeploy's bookkeeping (redeploy.md, 8).
use super::super::redeploy::{SECOND, accounts, run};
use std::collections::BTreeSet;
use temper_redeploy_tool::{
    fake::{Deployment, NEW, OLD, Opening, WIRES, description, output},
    hosts::{Hosts, Output, Part, fill},
    redeploy::{Mode, Outcome},
};

struct Audited {
    deployment: Deployment,
    turn: bool,
    draining: bool,
    swapped: BTreeSet<Part>,
    started: BTreeSet<Part>,
    context: String,
    checked: bool,
    serving_checkpoint: Option<String>,
    settled_checkpoint: Option<String>,
    last_checkpoint: Option<String>,
}
impl Audited {
    fn before(&self, part: Part, line: &str) {
        if line.starts_with("check ") {
            assert!(
                !self.deployment.lines.iter().any(|call| call.line.starts_with("stop ")),
                "check after a stop: {}",
                self.context
            );
            assert!(self.deployment.engine.running, "check without the serving engine: {}", self.context);
            let checkpoint = self.serving_checkpoint.as_ref().expect("act named the check's checkpoint");
            assert_eq!(line, format!("check '/builds/{NEW}/temper-engine' '{checkpoint}'"), "{}", self.context);
        }
        if line.starts_with("stop ") {
            assert!(self.checked, "stop before the check: {}", self.context);
        }
        if line.starts_with("checkpoint ") {
            assert!(!self.deployment.engine.running, "checkpoint while engine runs: {}", self.context);
            assert_eq!(line, "checkpoint '/bin/temper-engine'", "{}", self.context);
        }
        if line.starts_with("verdict ") {
            let checkpoint = self.settled_checkpoint.as_ref().expect("stop checkpoint before verdict");
            assert_eq!(line, format!("verdict '/builds/{NEW}/temper-engine' '{checkpoint}'"), "{}", self.context);
        }
        if line.starts_with("swap ") {
            assert!(self.settled_checkpoint.is_some(), "swap without this stop's checkpoint: {}", self.context);
            assert!(!self.deployment.machine(part).running, "swapping a running part: {} {part:?}", self.context);
        }
        if part == Part::Engine && line == "stop engine" && self.draining {
            assert!(
                self.deployment.workers.iter().all(|worker| !worker.running),
                "engine stopped before the drain: {}",
                self.context
            );
        }
    }
    fn after(&mut self, part: Part, line: &str, answer: &Output) {
        if line.starts_with("check ") {
            self.checked = answer.code == Some(0);
        }
        if line.starts_with("stop ") && part == Part::Engine {
            self.settled_checkpoint = None;
        }
        if answer.code == Some(0) {
            if line == "ctl engine 'deployment' 'checkpoint' '--json' </dev/null" {
                self.serving_checkpoint = serde_json::from_str::<serde_json::Value>(&answer.stdout)
                    .ok()
                    .and_then(|value| value.get("name")?.as_str().map(str::to_owned));
            }
            if line.starts_with("checkpoint ") {
                let name = answer
                    .stdout
                    .strip_prefix("temper-engine checkpoint: ")
                    .and_then(|line| line.split_whitespace().next())
                    .expect("fake checkpoint line");
                self.settled_checkpoint = Some(name.into());
                self.last_checkpoint = Some(name.into());
            }
            if line.starts_with("swap ") {
                self.swapped.insert(part);
            }
            if line.starts_with("start ") {
                self.started.insert(part);
            }
        }
        if self.turn && !self.draining {
            assert!(
                self.deployment.workers.iter().filter(|worker| !worker.running).count() <= 1,
                "two workers stopped in turn: {}",
                self.context
            );
        }
    }
}
impl Hosts for Audited {
    fn run(&mut self, part: Part, line: &str) -> Output {
        self.before(part, line);
        let answer = self.deployment.run(part, line);
        self.after(part, line, &answer);
        answer
    }
    fn run_each(&mut self, lines: &[(Part, String)]) -> Vec<Output> {
        assert!(self.deployment.engine.running, "drain without a live engine: {}", self.context);
        self.draining = true;
        for (part, line) in lines {
            self.before(*part, line);
        }
        let answers = self.deployment.run_each(lines);
        for ((part, line), answer) in lines.iter().zip(&answers) {
            self.after(*part, line, answer);
        }
        answers
    }
    fn now(&mut self) -> u64 {
        self.deployment.now()
    }
    fn sleep(&mut self, ns: u64) {
        self.deployment.sleep(ns);
    }
}
#[test]
fn deployments_keep_swap_drain_refusal_and_done_invariants_seeded() {
    let mut outcomes = BTreeSet::new();
    let mut counts = BTreeSet::new();
    let mut verdicts = BTreeSet::new();
    let mut read_again = BTreeSet::new();
    for seed in 1..=96_u64 {
        for mode in [Mode::Engine, Mode::Workers, Mode::Together] {
            for verdict in 0..5 {
                let mut random = seed * 71 + verdict;
                let count = usize::try_from(next(&mut random) % 4 + 1).unwrap();
                counts.insert(count);
                let d = description(count);
                let mut deployment = Deployment::new(d.clone());
                deployment.stage_all(NEW);
                let mut stop = Vec::new();
                for (index, worker) in d.workers.iter().enumerate() {
                    if next(&mut random) % 2 == 0 {
                        let wait = (next(&mut random) % 5 + 3) * 100_000;
                        stop.push(format!(
                            "temper-engine serve: worker {} told the engine restarts: wait {wait} ms",
                            worker.name
                        ));
                    } else {
                        stop.push(format!("temper-engine serve: worker {} not told the engine restarts: jig-worker-wire 14; it keeps its grace", worker.name));
                    }
                    let opening = match next(&mut random) % 12 {
                        0 => Opening::Never,
                        1 => Opening::Refused { nanoseconds: SECOND, text: "wire version cannot agree".into() },
                        _ => Opening::After { nanoseconds: next(&mut random) % 3 * SECOND, wires: WIRES.into() },
                    };
                    deployment.opening(index, opening);
                }
                deployment.stop_logs.push_back(stop);
                let (code, verdict_line) = verdict_answer(verdict);
                let stop_verdict = (seed + verdict) % 5;
                verdicts.insert(format!("{mode:?}/{verdict}/{stop_verdict}"));
                let (stop_code, stop_line) = verdict_answer(stop_verdict);
                deployment.verdict_answer(stop_code, stop_line);
                let ran_job = next(&mut random) % 3 == 0;
                deployment.check_answer(
                    code,
                    &format!("{}{verdict_line}", if ran_job { "temper-engine upgrade: ran temper deleted-repository-forge-rows: 812 rows in 2 commits; time unavailable\n" } else { "" }),
                );
                accounts(&mut deployment, false, true);
                accounts(&mut deployment, true, false);
                let failure = fail(&mut deployment, &mut random);
                let context = format!(
                    "seed {seed}, {mode:?}, verdict {verdict}, stop verdict {stop_verdict}, count {count}, failure {failure}"
                );
                let mut hosts = Audited {
                    deployment,
                    turn: mode == Mode::Workers,
                    draining: mode == Mode::Together,
                    swapped: BTreeSet::new(),
                    started: BTreeSet::new(),
                    context,
                    checked: false,
                    serving_checkpoint: None,
                    settled_checkpoint: None,
                    last_checkpoint: None,
                };
                let (outcome, record) = run(&d, &mut hosts, mode);
                outcomes.insert(format!("{outcome:?}"));
                if record.verdict.is_some() {
                    read_again.insert(stop_verdict);
                }
                assert_eq!(
                    record.backup, hosts.last_checkpoint,
                    "backup is the last stop's checkpoint: {}",
                    hosts.context
                );
                audit_outcome(&hosts, outcome, &record);
                assert!(
                    hosts.deployment.lines.iter().filter(|line| line.line.starts_with("check ")).count() <= 1,
                    "{}",
                    hosts.context
                );
            }
        }
    }
    assert_eq!(verdicts.len(), 75, "each mode and each pair of scripted verdicts");
    assert_eq!(read_again, [0, 1, 2, 3, 4].into(), "each verdict actually read again at a stop");
    assert_eq!(counts, [1, 2, 3, 4].into());
    assert_eq!(outcomes, ["Done".into(), "Failed".into(), "Refused".into()].into());
}
#[test]
fn told_deployments_follow_the_opening_bound_and_per_worker_verdicts_seeded() {
    use temper_redeploy_tool::referee::judgement::{Readings, evaluate};
    let mut coverage = [false; 4];
    for seed in 1..=512_u64 {
        let mut random = seed;
        let count = usize::try_from(next(&mut random) % 4 + 1).unwrap();
        let d = description(count);
        let mut deployment = Deployment::new(d.clone());
        deployment.stage_all(NEW);
        let mut given = Vec::new();
        let mut delays = Vec::new();
        let mut stop = Vec::new();
        for (index, worker) in d.workers.iter().enumerate() {
            let grace_seconds = 300;
            let told = next(&mut random) % 2 == 0;
            let wait = (next(&mut random) % 9) * 100;
            let name = &worker.name;
            stop.push(if told {
                format!("temper-engine serve: worker {name} told the engine restarts: wait {} ms", wait * 1000)
            } else {
                format!("temper-engine serve: worker {name} not told the engine restarts: jig-worker-wire 14; it keeps its grace")
            });
            let wait = if told { grace_seconds.max(wait) } else { grace_seconds };
            given.push(wait);
            let delay = (next(&mut random) % 10) * 100;
            delays.push(delay);
            coverage[usize::from(told)] = true;
            deployment.opening(index, Opening::After { nanoseconds: delay * SECOND, wires: WIRES.into() });
        }
        deployment.stop_logs.push_back(stop);
        let bound = *given.iter().max().unwrap();
        let last = *delays.iter().max().unwrap();
        let (outcome, record) = run(&d, &mut deployment, Mode::Engine);
        assert_eq!(outcome, if last <= bound { Outcome::Done } else { Outcome::Failed }, "seed {seed}");
        assert_eq!(deployment.at, (1 + last.min(bound)) * SECOND, "seed {seed}");
        let failures = delays.iter().zip(&given).filter(|(delay, wait)| *delay >= *wait).count();
        let result = evaluate(&d, &record, &Readings::default());
        assert_eq!(result.failures, failures, "seed {seed}: {given:?}, {delays:?}: {result:?}");
        coverage[2 + usize::from(failures == 0)] = true;
    }
    assert!(coverage.into_iter().all(|seen| seen), "told and not told, both referee verdicts");
}

fn fail(deployment: &mut Deployment, random: &mut u64) -> u64 {
    let d = deployment.description.clone();
    let count = u64::try_from(d.workers.len()).expect("fixture worker count fits u64");
    let worker = usize::try_from(next(random) % count).expect("a fixture worker index fits usize");
    let failure = next(random) % 14;
    match failure {
        0 => deployment.fail_line(Part::Engine, "stop engine", 1),
        1 => deployment.fail_line(Part::Worker(worker), &d.workers[worker].stop, 1),
        2 => deployment.fail_line(Part::Engine, "start engine", 1),
        3 => deployment.fail_line(Part::Worker(worker), &d.workers[worker].start, 1),
        4 | 5 => {
            let binary = if failure == 4 { "temper-engine" } else { "temper" };
            let line = fill(
                &d.engine.swap,
                &[("from", &format!("/builds/{NEW}/{binary}")), ("to", &format!("/bin/{binary}"))],
            );
            deployment.fail_line(Part::Engine, &line, 1);
        }
        6 => {
            let line = fill(
                &d.workers[worker].swap,
                &[("from", &format!("/builds/{NEW}/temper-agent")), ("to", "/bin/temper-agent")],
            );
            deployment.fail_line(Part::Worker(worker), &line, 1);
        }
        7 => deployment.check = Output { code: None, stdout: String::new(), stderr: "cannot spawn".into() },
        8 => deployment.fail_line(Part::Engine, "log engine '0'", 1),
        9 => deployment.fail_line(Part::Engine, "checkpoint '/bin/temper-engine'", 1),
        10 => deployment.fail_line(Part::Engine, "ctl engine 'account' 'list' '--json' </dev/null", 1),
        11 => deployment.ctl_answer("deployment checkpoint --json", output(1, "checkpoint refused")),
        12 => deployment.ctl_answer("deployment checkpoint --json", output(0, "{\"name\":\"not-a-checkpoint\"}")),
        13 => {}
        _ => unreachable!(),
    }
    failure
}
fn audit_outcome(hosts: &Audited, outcome: Outcome, record: &temper_redeploy_tool::record::Record) {
    match outcome {
        Outcome::Refused => {
            assert!(hosts.swapped.is_empty(), "refused after a swap: {}", hosts.context);
            for part in std::iter::once(Part::Engine).chain((0..hosts.deployment.workers.len()).map(Part::Worker)) {
                let machine = hosts.deployment.machine(part);
                assert!(
                    machine.running && machine.installed.values().all(|build| build == OLD),
                    "refusal left {part:?} changed: {}",
                    hosts.context
                );
            }
        }
        Outcome::Done => {
            for part in &hosts.started {
                let machine = hosts.deployment.machine(*part);
                assert!(
                    machine.running && machine.installed.values().all(|build| build == NEW),
                    "done left {part:?} old: {}",
                    hosts.context
                );
            }
            assert_eq!(record.openings.len(), hosts.deployment.workers.len(), "{}", hosts.context);
            assert!(record.openings.iter().all(|opening| !opening.refused), "{}", hosts.context);
        }
        Outcome::Failed => {
            assert!(
                record.lines.last().expect("outcome printed").line.starts_with("temper-redeploy run: failed at "),
                "{}",
                hosts.context
            );
        }
    }
}
fn verdict_answer(verdict: u64) -> (i32, &'static str) {
    match verdict {
        0 => (0, "workers may stay: yes"),
        1 => (0, "workers may stay: no"),
        2 => (0, "workers may stay: unknown; no wires kept"),
        3 => (0, "no verdict"),
        4 => (1, "cannot read the checkpoint"),
        _ => unreachable!(),
    }
}
fn next(random: &mut u64) -> u64 {
    *random ^= *random << 13;
    *random ^= *random >> 7;
    *random ^= *random << 17;
    *random
}
