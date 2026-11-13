//! A line-log referee independent of the redeploy's bookkeeping (redeploy.md, 8).
use super::super::redeploy::{SECOND, accounts, grace, run};
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
}
impl Audited {
    fn before(&self, part: Part, line: &str) {
        if line.starts_with("swap ") {
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
        if answer.code == Some(0) {
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
    for seed in 1..=96_u64 {
        for mode in [Mode::Engine, Mode::Workers, Mode::Together] {
            for verdict in 0..5 {
                let mut random = seed * 71 + verdict;
                let count = usize::try_from(next(&mut random) % 4 + 1).unwrap();
                counts.insert(count);
                let d = description(count);
                let mut deployment = Deployment::new(d.clone());
                deployment.stage_all(NEW);
                for index in 0..count {
                    grace(&mut deployment, index, (next(&mut random) % 4 + 2) * 1000);
                    let opening = match next(&mut random) % 12 {
                        0 => Opening::Never,
                        1 => Opening::Refused { nanoseconds: SECOND, text: "wire version cannot agree".into() },
                        _ => Opening::After { nanoseconds: next(&mut random) % 3 * SECOND, wires: WIRES.into() },
                    };
                    deployment.opening(index, opening);
                }
                let (code, verdict_line) = match verdict {
                    0 => (0, "workers may stay: yes"),
                    1 => (0, "workers may stay: no"),
                    2 => (0, "workers may stay: unknown; no wires kept"),
                    3 => (0, "no verdict"),
                    4 => (1, "check refuses the build"),
                    _ => unreachable!(),
                };
                let patched = next(&mut random) % 3 == 0;
                deployment.check_answer(
                    code,
                    &format!("{}{verdict_line}", if patched { "temper-engine upgrade: patched store\n" } else { "" }),
                );
                accounts(&mut deployment, false, true);
                accounts(&mut deployment, true, false);
                let failure = fail(&mut deployment, &mut random);
                let context = format!("seed {seed}, {mode:?}, verdict {verdict}, count {count}, failure {failure}");
                let mut hosts = Audited {
                    deployment,
                    turn: mode == Mode::Workers,
                    draining: mode == Mode::Together,
                    swapped: BTreeSet::new(),
                    started: BTreeSet::new(),
                    context,
                };
                let (outcome, record) = run(&d, &mut hosts, mode);
                outcomes.insert(format!("{outcome:?}"));
                match outcome {
                    Outcome::Refused => {
                        assert!(hosts.swapped.is_empty(), "refused after a swap: {}", hosts.context);
                        for part in std::iter::once(Part::Engine).chain((0..count).map(Part::Worker)) {
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
                        assert_eq!(record.openings.len(), count, "{}", hosts.context);
                        assert!(record.openings.iter().all(|opening| !opening.refused), "{}", hosts.context);
                    }
                    Outcome::Failed => {
                        assert!(
                            record.lines.last().unwrap().line.starts_with("temper-redeploy run: failed at "),
                            "{}",
                            hosts.context
                        );
                    }
                }
                assert!(
                    hosts.deployment.lines.iter().filter(|line| line.line.starts_with("check ")).count() <= 1,
                    "{}",
                    hosts.context
                );
            }
        }
    }
    assert_eq!(counts, [1, 2, 3, 4].into());
    assert_eq!(outcomes, ["Done".into(), "Failed".into(), "Refused".into()].into());
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
        9 => deployment.fail_line(Part::Engine, &format!("backup '{NEW}-19700101T000001Z'"), 1),
        10 => deployment.fail_line(Part::Engine, "ctl engine 'account' 'list' '--json' </dev/null", 1),
        11 => deployment.documents[worker] = output(1, ""),
        12 | 13 => {}
        _ => unreachable!(),
    }
    failure
}
fn next(random: &mut u64) -> u64 {
    *random ^= *random << 13;
    *random ^= *random >> 7;
    *random ^= *random << 17;
    *random
}
