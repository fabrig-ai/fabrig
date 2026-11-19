//! Redeploy paths on the deployment's clock (redeploy.md, 4, 8).
use std::{cell::RefCell, io::Write, rc::Rc};
use temper_redeploy_tool::{
    fake::{Deployment, NEW, OLD, Opening, WIRES, description, output},
    hosts::{Hosts, Output, Part, fill},
    record::{Journal, Record},
    redeploy::{Mode, Outcome, redeploy},
};

pub(super) const SECOND: u64 = 1_000_000_000;
const CHECKPOINT: &str = "00000000000000000001-0123456789abcdef";
const BACKUP: &str = "00000000000000000002-0123456789abcdef";
const SECOND_BACKUP: &str = "00000000000000000003-0123456789abcdef";
const CHECKPOINT_CALL: &str = "ctl engine 'deployment' 'checkpoint' '--json' </dev/null";
const BACKUP_CALL: &str = "checkpoint '/bin/temper-engine'";
#[derive(Clone, Default)]
struct Printed(Rc<RefCell<Vec<u8>>>);
impl Write for Printed {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(super) fn run(
    d: &temper_redeploy_tool::description::Description,
    hosts: &mut dyn Hosts,
    mode: Mode,
) -> (Outcome, Record) {
    let printed = Printed::default();
    let mut journal = Journal::new(Record::new(NEW, mode), None, Box::new(printed.clone()));
    let outcome = redeploy(d, NEW, mode, &mut journal, hosts);
    let text = String::from_utf8(printed.0.borrow().clone()).expect("journal prints UTF-8");
    assert_eq!(
        text.lines().collect::<Vec<_>>(),
        journal.record.lines.iter().map(|line| line.line.as_str()).collect::<Vec<_>>()
    );
    assert!(journal.record.lines.iter().all(|line| line.line.starts_with("temper-redeploy run: ")));
    (outcome, journal.record)
}
fn staged(count: usize) -> Deployment {
    let mut hosts = Deployment::new(description(count));
    hosts.stage_all(NEW);
    hosts
}
fn execute(hosts: &mut Deployment, mode: Mode) -> (Outcome, Record) {
    run(&hosts.description.clone(), hosts, mode)
}
fn all_old(hosts: &Deployment) {
    for machine in std::iter::once(&hosts.engine).chain(&hosts.workers) {
        assert!(machine.running);
        assert!(machine.installed.values().all(|build| build == OLD));
    }
    assert!(hosts.lines.iter().all(|call| !call.line.starts_with("swap ")));
}
fn position(hosts: &Deployment, text: &str) -> usize {
    hosts.lines.iter().position(|call| call.line == text).unwrap_or_else(|| panic!("missing {text}: {:?}", hosts.lines))
}
fn last(record: &Record) -> &str {
    &record.lines.last().expect("outcome was printed").line
}

#[test]
fn the_worker_grace_journal_needs_no_document_read() {
    let mut hosts = staged(2);
    hosts.documents.fill(output(1, "not a readable worker document"));
    let (outcome, record) = execute(&mut hosts, Mode::Engine);
    assert_eq!(outcome, Outcome::Done);
    assert_eq!(record.graces.iter().map(|grace| grace.grace_ms).collect::<Vec<_>>(), [300_000, 300_000]);
    assert!(hosts.lines.iter().all(|call| !call.line.starts_with("document ")));
}

#[test]
fn engine_alone_keeps_workers_and_records_the_first_opening_window() {
    let mut hosts = staged(2);
    hosts.opening(0, Opening::After { nanoseconds: 3 * SECOND, wires: WIRES.into() });
    hosts.opening(1, Opening::After { nanoseconds: 5 * SECOND, wires: WIRES.into() });
    let (outcome, record) = execute(&mut hosts, Mode::Engine);
    assert_eq!(outcome, Outcome::Done);
    assert_eq!(hosts.engine.installed.values().collect::<Vec<_>>(), [&NEW, &NEW]);
    for binary in ["temper-engine", "temper"] {
        assert_eq!(hosts.engine.staged[&format!("/builds/{OLD}/{binary}")], OLD);
    }
    for call in hosts.lines.iter().filter(|call| call.part != Part::Engine) {
        assert!(call.line.ends_with("'--version'") || call.line.starts_with("document "), "{call:?}");
    }
    assert!(hosts.workers.iter().all(|worker| worker.running && worker.installed.values().all(|build| build == OLD)));
    assert_eq!(
        record.previous.iter().map(|part| (part.part.as_str(), part.commit.as_str())).collect::<Vec<_>>(),
        [("engine", OLD), ("w1", OLD), ("w2", OLD)]
    );
    assert_eq!(record.graces.iter().map(|grace| grace.grace_ms).collect::<Vec<_>>(), [300_000, 300_000]);
    let window = record.window.unwrap();
    assert_eq!((window.from, window.to, window.worker.as_str()), (SECOND, 4 * SECOND, "w1"));
    assert_eq!(
        record.openings.iter().map(|opening| (opening.worker.as_str(), opening.at)).collect::<Vec<_>>(),
        [("w1", 4 * SECOND), ("w2", 6 * SECOND)]
    );
    assert!(record.openings.iter().all(|opening| opening.line.ends_with(WIRES) && !opening.refused));
    let check = record.check.unwrap();
    assert_eq!(check.code, Some(0));
    assert_eq!(check.lines, ["temper-engine upgrade: workers may stay: yes"]);
    assert_eq!(hosts.lines.iter().filter(|call| call.line.starts_with("check ")).count(), 1);
    assert_eq!(record.outages, []);
    assert_eq!(record.lines.last().unwrap().line, "temper-redeploy run: done");
}

#[test]
fn settled_stop_keeps_who_was_told_and_waits_the_largest_wait_given() {
    let mut d = description(3);
    for (worker, name) in d.workers.iter_mut().zip(["a", "b", "c"]) {
        worker.name = name.into();
    }
    let mut hosts = Deployment::new(d);
    hosts.stage_all(NEW);
    for index in 0..3 {
        hosts.opening(index, Opening::After { nanoseconds: 900 * SECOND, wires: WIRES.into() });
    }
    hosts.stop_logs.push_back(vec![
        "Nov 16 host temper-engine serve: worker a told the engine restarts: wait 1200000 ms".into(),
        "temper-engine serve: worker b not told the engine restarts: jig-worker-wire 13; it keeps its grace".into(),
        "temper-engine serve: worker outside told the engine restarts: wait 9999999 ms".into(),
    ]);
    let (outcome, record) = execute(&mut hosts, Mode::Engine);
    assert_eq!(outcome, Outcome::Done);
    assert_eq!(hosts.at, 901 * SECOND);
    assert_eq!(
        serde_json::to_value(&record.told).unwrap(),
        serde_json::json!([
            {"worker": "a", "wait_ms": 1_200_000},
            {"worker": "b", "wait_ms": null},
            {"worker": "c", "wait_ms": null}
        ])
    );
    for line in [
        "worker a told: restart wait 1200.0 s",
        "worker b keeps its grace: not told",
        "worker c keeps its grace: not told",
    ] {
        assert!(record.lines.iter().any(|kept| kept.line == format!("temper-redeploy run: {line}")), "{line}");
    }
    assert!(record.openings.iter().all(|opening| opening.at == 901 * SECOND));
    let stop = position(&hosts, "stop engine");
    assert_eq!(hosts.lines[stop + 1].line, "log engine '0'");
    assert_eq!(hosts.lines[stop + 2].line, BACKUP_CALL);
}

struct SlowStop(Deployment);
impl Hosts for SlowStop {
    fn run(&mut self, part: Part, line: &str) -> Output {
        let answer = self.0.run(part, line);
        if part == Part::Engine && line == "stop engine" {
            self.0.sleep(4 * SECOND);
        }
        answer
    }
    fn run_each(&mut self, lines: &[(Part, String)]) -> Vec<Output> {
        self.0.run_each(lines)
    }
    fn now(&mut self) -> u64 {
        self.0.now()
    }
    fn sleep(&mut self, ns: u64) {
        self.0.sleep(ns);
    }
}

#[test]
fn only_the_windows_stop_reads_told_since_before_the_stop_began() {
    let mut hosts = staged(2);
    hosts.at = 10 * SECOND;
    hosts.log.push((8 * SECOND, "temper-engine serve: worker w2 told the engine restarts: wait 99999 ms".into()));
    hosts.stop_logs.push_back(vec!["temper-engine serve: worker w1 told the engine restarts: wait 1200000 ms".into()]);
    hosts.stop_logs.push_back(vec!["temper-engine serve: worker w2 told the engine restarts: wait 77777 ms".into()]);
    hosts.verdict_answer(0, "workers may stay: yes");
    hosts.verdict_answer(0, "workers may stay: no");
    let d = hosts.description.clone();
    let mut hosts = SlowStop(hosts);
    let (outcome, record) = run(&d, &mut hosts, Mode::Engine);
    assert_eq!(outcome, Outcome::Done);
    assert_eq!(record.told_wait_ms("w1"), Some(1_200_000));
    assert_eq!(record.told_wait_ms("w2"), None);
    let first = position(&hosts.0, "stop engine");
    assert_eq!((hosts.0.lines[first + 1].at, hosts.0.lines[first + 1].line.as_str()), (14 * SECOND, "log engine '9'"));
    let second = hosts.0.lines.iter().rposition(|call| call.line == "stop engine").unwrap();
    assert!(!hosts.0.lines[second + 1].line.starts_with("log "));
    assert_eq!(record.lines.iter().filter(|line| line.line.contains("keeps its grace: not told")).count(), 1);
}

#[test]
fn unreadable_stop_log_refuses_and_restores_before_any_checkpoint_or_swap() {
    let mut hosts = staged(2);
    hosts.fail_line(Part::Engine, "log engine '0'", 1);
    let d = hosts.description.clone();
    let mut hosts = SlowStop(hosts);
    let (outcome, record) = run(&d, &mut hosts, Mode::Engine);
    assert_eq!(outcome, Outcome::Refused);
    all_old(&hosts.0);
    assert_eq!(record.told, None);
    assert!(record.check.is_some());
    assert!(hosts.0.lines.iter().all(|call| !call.line.starts_with("checkpoint ")));
    assert!(last(&record).contains("refused: who was told: engine log"));
    assert_eq!(record.openings.len(), 2);
}

#[test]
fn openings_stop_at_the_longer_of_told_wait_and_grace() {
    for (wait, bound) in [(100_000, 300), (400_000, 400)] {
        let mut hosts = staged(1);
        hosts
            .stop_logs
            .push_back(vec![format!("temper-engine serve: worker w1 told the engine restarts: wait {wait} ms")]);
        hosts.opening(0, Opening::After { nanoseconds: (bound + 1) * SECOND, wires: WIRES.into() });
        let (outcome, record) = execute(&mut hosts, Mode::Engine);
        assert_eq!(outcome, Outcome::Failed);
        assert_eq!(hosts.at, (bound + 1) * SECOND);
        assert_eq!(record.openings, []);
        assert!(last(&record).contains("workers did not open within the largest wait given: w1"));
    }
}

#[test]
fn a_refusing_check_stops_nothing_in_any_mode() {
    for mode in [Mode::Engine, Mode::Workers, Mode::Together] {
        let mut hosts = staged(2);
        hosts.check_answer(1, "temper-engine upgrade: cannot start\n");
        let (outcome, record) = execute(&mut hosts, mode);
        assert_eq!(outcome, Outcome::Refused);
        all_old(&hosts);
        assert!(hosts.lines.iter().all(|call| !call.line.starts_with("stop ") && !call.line.starts_with("start ")));
        assert_eq!(record.window, None);
        assert_eq!(record.backup, None);
        assert_eq!(record.verdict, None);
        assert_eq!(record.openings.len(), 0);
        assert_eq!(record.checkpoint.as_deref(), Some(CHECKPOINT));
        assert_eq!(record.check.as_ref().unwrap().code, Some(1));
        assert!(last(&record).starts_with("temper-redeploy run: refused: check:"));
    }
}

#[test]
fn no_unknown_and_absent_check_verdict_drain_all_without_an_old_engine_restart() {
    for verdict in [
        "workers may stay: no",
        "workers may stay: unknown; the copy keeps no wires",
        "no verdict",
        "workers may stay: yes\nworkers may stay: no",
    ] {
        let mut hosts = staged(2);
        hosts.check_answer(0, verdict);
        hosts.verdict_answer(2, "unexpected verdict");
        hosts.opening(0, Opening::After { nanoseconds: SECOND, wires: WIRES.into() });
        let (outcome, record) = execute(&mut hosts, Mode::Engine);
        assert_eq!(outcome, Outcome::Done, "{verdict}");
        assert_eq!(hosts.lines.iter().filter(|call| call.line.starts_with("check ")).count(), 1);
        assert_eq!(hosts.lines.iter().filter(|call| call.line == "stop engine").count(), 1);
        assert_eq!(hosts.batches, [vec![(Part::Worker(0), "stop w1".into()), (Part::Worker(1), "stop w2".into())]]);
        assert!(position(&hosts, "stop w2") < position(&hosts, "stop engine"));
        assert!(position(&hosts, "stop engine") < position(&hosts, "start engine"));
        assert_eq!(hosts.lines.iter().filter(|call| call.line == "start engine").count(), 1);
        assert_eq!(hosts.lines[position(&hosts, "stop w1")].at, SECOND);
        assert_eq!(record.window, None);
        assert_eq!(record.verdict, None);
        assert!(hosts.lines.iter().all(|call| !call.line.starts_with("verdict ")));
        assert_eq!(record.backup.as_deref(), Some(BACKUP));
        assert_eq!(record.outages.len(), 1);
        assert_eq!(record.outages[0].what, "every part");
        assert_eq!((record.outages[0].from, record.outages[0].to), (SECOND, 2 * SECOND));
        for machine in std::iter::once(&hosts.engine).chain(&hosts.workers) {
            assert!(machine.running && machine.installed.values().all(|build| build == NEW));
        }
        assert_eq!(record.openings.len(), 2, "only the final start's openings");
    }
}

pub(super) fn accounts(hosts: &mut Deployment, connected: bool, awaiting: bool) {
    hosts.ctl_answer(
        "account list --json",
        output(
            0,
            &serde_json::json!({"accounts": {"deployment_facts": {"program_hosts": [
                {"name": "other", "connected": true, "awaiting_logins": false},
                {"name": "w1", "connected": connected, "awaiting_logins": awaiting},
                {"name": "w2", "connected": connected, "awaiting_logins": awaiting},
                {"name": "w3", "connected": connected, "awaiting_logins": awaiting},
                {"name": "w4", "connected": connected, "awaiting_logins": awaiting}
            ]}}})
            .to_string(),
        ),
    );
}
#[test]
fn workers_in_turn_wait_for_each_opening_and_its_own_connected_logins() {
    let mut hosts = staged(2);
    hosts.opening(0, Opening::After { nanoseconds: SECOND, wires: WIRES.into() });
    hosts.opening(1, Opening::After { nanoseconds: 2 * SECOND, wires: WIRES.into() });
    // Both predicates, and the named entry rather than another ready host, must hold.
    accounts(&mut hosts, false, false);
    accounts(&mut hosts, true, true);
    accounts(&mut hosts, true, false);
    accounts(&mut hosts, false, false);
    accounts(&mut hosts, true, false);
    let (outcome, record) = execute(&mut hosts, Mode::Workers);
    assert_eq!(outcome, Outcome::Done);
    let ctl = "ctl engine 'account' 'list' '--json' </dev/null";
    let calls: Vec<_> = hosts.lines.iter().filter(|call| call.line == ctl).collect();
    assert_eq!(
        calls.iter().map(|call| call.at).collect::<Vec<_>>(),
        [4 * SECOND, 5 * SECOND, 6 * SECOND, 8 * SECOND, 9 * SECOND]
    );
    assert_eq!(hosts.lines[position(&hosts, "stop w2")].at, 6 * SECOND);
    assert!(position(&hosts, ctl) < position(&hosts, "stop w2"));
    assert_eq!(
        record.outages.iter().map(|outage| (outage.what.as_str(), outage.from, outage.to)).collect::<Vec<_>>(),
        [("w1", 3 * SECOND, 6 * SECOND), ("w2", 6 * SECOND, 9 * SECOND)]
    );
    assert_eq!(
        record.openings.iter().map(|opening| (opening.worker.as_str(), opening.at)).collect::<Vec<_>>(),
        [("w1", 4 * SECOND), ("w2", 8 * SECOND)]
    );
    assert_eq!(hosts.batches, Vec::<Vec<(Part, String)>>::new());
    for machine in std::iter::once(&hosts.engine).chain(&hosts.workers) {
        assert!(machine.running && machine.installed.values().all(|build| build == NEW));
    }
}

#[test]
fn together_drains_in_one_batch_before_the_engine_and_starts_engine_first() {
    let mut hosts = staged(2);
    hosts.check_answer(0, "workers may stay: yes");
    hosts.verdict_answer(2, "unexpected verdict");
    hosts.opening(1, Opening::After { nanoseconds: 2 * SECOND, wires: WIRES.into() });
    let (outcome, record) = execute(&mut hosts, Mode::Together);
    assert_eq!(outcome, Outcome::Done);
    assert_eq!(hosts.batches.len(), 1);
    assert_eq!(hosts.batches[0].len(), 2);
    assert!(position(&hosts, "stop w2") < position(&hosts, "stop engine"));
    assert!(
        position(&hosts, &format!("check '/builds/{NEW}/temper-engine' '{CHECKPOINT}'")) < position(&hosts, "stop w1")
    );
    assert_eq!(record.verdict, None);
    assert!(hosts.lines.iter().all(|call| !call.line.starts_with("verdict ")));
    assert_eq!(record.backup.as_deref(), Some(BACKUP));
    assert!(position(&hosts, "start engine") < position(&hosts, "start w1"));
    assert!(position(&hosts, "start w1") < position(&hosts, "start w2"));
    assert!(record.window.is_none());
    assert_eq!(
        (record.outages[0].what.as_str(), record.outages[0].from, record.outages[0].to),
        ("every part", SECOND, 3 * SECOND)
    );
}

#[test]
fn check_and_verdict_read_their_own_checkpoints_with_the_right_binaries_and_order() {
    for mode in [Mode::Engine, Mode::Workers] {
        let mut hosts = staged(1);
        let checkpoint = "00000000000000000042-0123456789abcdef";
        hosts.ctl_answer("deployment checkpoint --json", output(0, &format!("{{\"name\":\"{checkpoint}\"}}")));
        hosts.verdict_answer(0, "serving only\nworkers may stay: yes");
        hosts.verdict_answer(0, "workers may stay: no\nworkers may stay: yes\n");
        accounts(&mut hosts, true, false);
        let (outcome, record) = execute(&mut hosts, mode);
        assert_eq!(outcome, Outcome::Done);
        assert_eq!(record.checkpoint.as_deref(), Some(checkpoint));
        // The scripted ctl answer took no generated name: the stopped checkpoint is the first.
        assert_eq!(record.backup.as_deref(), Some(CHECKPOINT));
        let check = format!("check '/builds/{NEW}/temper-engine' '{checkpoint}'");
        let serving = format!("verdict '/builds/{NEW}/temper-engine' '{checkpoint}'");
        let verdict = format!("verdict '/builds/{NEW}/temper-engine' '{CHECKPOINT}'");
        let calls: Vec<_> = hosts
            .lines
            .iter()
            .filter(|call| {
                call.line == CHECKPOINT_CALL
                    || call.line.starts_with("check ")
                    || call.line.starts_with("stop ")
                    || call.line.starts_with("log ")
                    || call.line.starts_with("checkpoint ")
                    || call.line.starts_with("verdict ")
                    || call.line.starts_with("swap ")
            })
            .map(|call| call.line.as_str())
            .collect();
        assert_eq!(
            &calls[..7],
            [CHECKPOINT_CALL, &check, &serving, "stop engine", "log engine '0'", BACKUP_CALL, &verdict]
        );
        assert!(calls[7].starts_with("swap "));
        let verdict = record.verdict.as_ref().unwrap();
        assert_eq!(verdict.code, Some(0));
        assert_eq!(verdict.lines, ["workers may stay: no", "workers may stay: yes"]);
        for line in [
            format!("check checkpoint {checkpoint}"),
            format!("stop checkpoint {CHECKPOINT}; backup"),
            "check workers may stay: yes".into(),
            "serving only".into(),
            "serving verdict exit Some(0)".into(),
            "serving verdict workers may stay: yes".into(),
            "verdict workers may stay: yes".into(),
        ] {
            assert!(record.lines.iter().any(|kept| kept.line == format!("temper-redeploy run: {line}")), "{line}");
        }
    }
}

#[test]
fn a_failed_serving_verdict_refuses_without_stopping_or_swapping() {
    for mode in [Mode::Engine, Mode::Workers] {
        let mut hosts = staged(2);
        hosts.verdict.push_back(Output {
            code: Some(2),
            stdout: String::new(),
            stderr: "temper-engine: no command upgrade --workers; temper-engine --help lists them\n".into(),
        });
        let scratch = skein_scratch::Scratch::new("redeploy-serving-verdict");
        let path = scratch.path().join("record.json");
        let printed = Printed::default();
        let exit = temper_redeploy_tool::cli::execute_with_steps(
            &hosts.description.clone(),
            &temper_redeploy_tool::cli::Action::Run { commit: NEW.into(), mode, record: path.clone() },
            &mut hosts,
            Box::new(printed.clone()),
            temper_redeploy_tool::cli::Steps {
                before: |_, _| Ok(temper_redeploy_tool::record::Before::default()),
                redeploy,
                judge: |_, _, _| true,
            },
        )
        .unwrap();
        assert_eq!(exit, 1);
        let record: Record = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(record.outcome, Some(Outcome::Refused));
        assert_eq!(
            String::from_utf8(printed.0.borrow().clone()).unwrap().lines().collect::<Vec<_>>(),
            record.lines.iter().map(|line| line.line.as_str()).collect::<Vec<_>>()
        );
        all_old(&hosts);
        assert!(hosts.lines.iter().all(|call| !call.line.starts_with("stop ")
            && !call.line.starts_with("start ")
            && !call.line.starts_with("checkpoint ")));
        assert_eq!(hosts.lines.last().unwrap().line, format!("verdict '/builds/{NEW}/temper-engine' '{CHECKPOINT}'"));
        assert_eq!(record.verdict, None);
        assert_eq!(record.backup, None);
        assert_eq!(record.window, None);
        assert_eq!(record.openings, []);
        let lines: Vec<_> =
            record.lines.iter().filter_map(|line| line.line.strip_prefix("temper-redeploy run: ")).collect();
        let start = lines
            .iter()
            .position(|line| *line == "temper-engine: no command upgrade --workers; temper-engine --help lists them")
            .unwrap();
        assert_eq!(
            &lines[start..],
            [
                "temper-engine: no command upgrade --workers; temper-engine --help lists them",
                "serving verdict exit Some(2)",
                "serving verdict: a build without upgrade --workers is redeployed with --together",
                "serving verdict failed: exit Some(2): temper-engine: no command upgrade --workers; temper-engine --help lists them ",
                "refused: serving verdict: exit Some(2): temper-engine: no command upgrade --workers; temper-engine --help lists them ",
            ]
        );
    }
}

#[test]
fn a_current_builds_serving_refusal_points_at_its_lines_without_a_together_hint() {
    for mode in [Mode::Engine, Mode::Workers] {
        let mut hosts = staged(2);
        hosts.verdict.push_back(Output {
            code: Some(1),
            stdout: String::new(),
            stderr: "temper-engine upgrade: checkpoint is in use\n".into(),
        });
        let (outcome, record) = execute(&mut hosts, mode);
        assert_eq!(outcome, Outcome::Refused);
        all_old(&hosts);
        assert!(hosts.lines.iter().all(|call| !call.line.starts_with("stop ")
            && !call.line.starts_with("start ")
            && !call.line.starts_with("checkpoint ")));
        assert_eq!(record.verdict, None);
        assert_eq!(record.backup, None);
        let lines: Vec<_> = record.lines.iter().map(|line| line.line.as_str()).collect();
        assert!(lines.iter().all(|line| !line.contains("--together")));
        let hint = lines
            .iter()
            .position(|line| {
                *line == "temper-redeploy run: serving verdict: refused by the staged build; its lines above say why"
            })
            .expect("the current build's refusal points at its lines");
        assert_eq!(lines[hint - 1], "temper-redeploy run: serving verdict exit Some(1)");
        assert!(hint < lines.len() - 1);
        assert_eq!(
            last(&record),
            "temper-redeploy run: refused: serving verdict: exit Some(1): temper-engine upgrade: checkpoint is in use "
        );
    }
}

#[test]
fn each_successful_verdict_notes_the_last_rehearsal_count_between_exit_and_decision() {
    for (stdout, stderr) in [
        ("temper-engine upgrade: rehearsal linked 12 files, copied 3\nworkers may stay: yes", ""),
        (
            "temper-engine upgrade: rehearsal linked 99 files, copied 88\nworkers may stay: yes",
            "temper-engine upgrade: rehearsal linked 12 files, copied 3",
        ),
    ] {
        let mut hosts = staged(1);
        hosts.check_answer(0, "temper-engine upgrade: rehearsal linked 99 files, copied 88\nworkers may stay: yes");
        hosts.verdict.push_back(Output { code: Some(0), stdout: stdout.into(), stderr: stderr.into() });
        let (outcome, record) = execute(&mut hosts, Mode::Engine);
        assert_eq!(outcome, Outcome::Done);
        let lines: Vec<_> = record.lines.iter().map(|line| line.line.as_str()).collect();
        for step in ["serving verdict", "verdict"] {
            let exit =
                lines.iter().position(|line| *line == format!("temper-redeploy run: {step} exit Some(0)")).unwrap();
            assert_eq!(lines[exit + 1], format!("temper-redeploy run: {step} rehearsal linked 12 files, copied 3"));
            assert_eq!(lines[exit + 2], format!("temper-redeploy run: {step} workers may stay: yes"));
        }
        assert!(!lines.iter().any(|line| line.starts_with("temper-redeploy run: check rehearsal")));
        let verdict = record.verdict.as_ref().unwrap();
        assert_eq!(verdict.code, Some(0));
        assert_eq!(verdict.lines, stdout.lines().chain(stderr.lines()).collect::<Vec<_>>());
    }
}

#[test]
fn missing_or_malformed_rehearsal_counts_are_unknown_without_refusing() {
    for count in [
        "",
        "temper-engine upgrade: rehearsal linked 12 files, copied three",
        "temper-engine upgrade: rehearsal linked +12 files, copied 3",
        "temper-engine upgrade: rehearsal linked 12 files, copied -3",
        "temper-engine upgrade: rehearsal linked  files, copied 3",
        "temper-engine upgrade: rehearsal linked 12 files, copied ",
        "temper-engine upgrade: rehearsal linked 12 files, copied 3 extra",
        "temper-engine upgrade: rehearsal linked 12 files, copied 3\ntemper-engine upgrade: rehearsal linked malformed",
    ] {
        let mut hosts = staged(1);
        hosts.verdict_answer(0, &format!("{count}\nworkers may stay: yes"));
        let (outcome, record) = execute(&mut hosts, Mode::Engine);
        assert_eq!(outcome, Outcome::Done, "{count}");
        let lines: Vec<_> = record.lines.iter().map(|line| line.line.as_str()).collect();
        for step in ["serving verdict", "verdict"] {
            let exit =
                lines.iter().position(|line| *line == format!("temper-redeploy run: {step} exit Some(0)")).unwrap();
            assert_eq!(
                lines[exit + 1],
                format!(
                    "temper-redeploy run: {step} rehearsal linked: unknown; the staged build may copy the whole checkpoint at the stop"
                ),
                "{count}"
            );
            assert_eq!(lines[exit + 2], format!("temper-redeploy run: {step} workers may stay: yes"));
        }
    }
}

#[test]
fn no_unknown_and_absent_serving_verdict_drain_before_the_only_engine_stop() {
    for verdict in [
        "workers may stay: no",
        "workers may stay: unknown; no wires",
        "",
        "workers may stay: yes\nworkers may stay: no",
    ] {
        for mode in [Mode::Engine, Mode::Workers] {
            let mut hosts = staged(2);
            hosts.verdict_answer(0, verdict);
            let (outcome, record) = execute(&mut hosts, mode);
            assert_eq!(outcome, Outcome::Done, "{mode:?}: {verdict}");
            assert_eq!(hosts.lines.iter().filter(|call| call.line.starts_with("verdict ")).count(), 1);
            assert_eq!(hosts.lines.iter().filter(|call| call.line == "stop engine").count(), 1);
            assert!(
                position(&hosts, &format!("verdict '/builds/{NEW}/temper-engine' '{CHECKPOINT}'"))
                    < position(&hosts, "stop w1")
            );
            assert!(position(&hosts, "stop w1") < position(&hosts, "stop engine"));
            assert!(position(&hosts, "stop w2") < position(&hosts, "stop engine"));
            assert_eq!(hosts.lines.iter().filter(|call| call.line == "start engine").count(), 1);
            assert_eq!(record.verdict, None);
            assert_eq!(record.backup.as_deref(), Some(BACKUP));
            assert_eq!(record.window, None);
            assert_eq!(record.outages.len(), 1);
            assert!(record.lines.iter().any(|line| line.line
                == "temper-redeploy run: serving verdict workers may stay: no or unknown; every part together"));
            for machine in std::iter::once(&hosts.engine).chain(&hosts.workers) {
                assert!(machine.running && machine.installed.values().all(|build| build == NEW));
            }
        }
    }
}

#[test]
fn a_refused_or_malformed_serving_checkpoint_stops_nothing() {
    let mut answers = vec![output(1, "refused"), output(0, "not JSON"), output(0, "{}"), output(0, "{\"name\":42}")];
    for name in [
        "42-0123456789abcdef",
        "00000000000000000042-0123456789abcdeg",
        "00000000000000000042_0123456789abcdef",
        "00000000000000000042-0123456789abcde",
        "0000000000000000004x-0123456789abcdef",
        "00000000000000000042-0123456789abcdef/../bad",
    ] {
        answers.push(output(0, &serde_json::json!({"name": name}).to_string()));
    }
    answers.push(Output { code: None, stdout: String::new(), stderr: "could not spawn ctl".into() });
    for answer in answers {
        let mut hosts = staged(2);
        hosts.ctl_answer("deployment checkpoint --json", answer);
        let (outcome, record) = execute(&mut hosts, Mode::Together);
        assert_eq!(outcome, Outcome::Refused);
        all_old(&hosts);
        assert_eq!(hosts.lines.last().unwrap().line, CHECKPOINT_CALL);
        assert_eq!(record.checkpoint, None);
        assert_eq!(record.check, None);
        assert!(last(&record).contains("refused: check checkpoint:"));
    }
}

#[test]
fn a_changed_stop_verdict_restores_waits_and_drains_with_a_new_backup() {
    for verdict in [
        "workers may stay: no",
        "workers may stay: unknown; no wires",
        "",
        "workers may stay: yes\nworkers may stay: no",
    ] {
        let mut hosts = staged(2);
        hosts.verdict_answer(0, "workers may stay: yes");
        hosts.verdict_answer(0, verdict);
        hosts.opening(0, Opening::After { nanoseconds: 2 * SECOND, wires: WIRES.into() });
        hosts
            .stop_logs
            .push_back(vec!["temper-engine serve: worker w1 told the engine restarts: wait 1200000 ms".into()]);
        hosts
            .stop_logs
            .push_back(vec!["temper-engine serve: worker w2 told the engine restarts: wait 700000 ms".into()]);
        let (outcome, record) = execute(&mut hosts, Mode::Engine);
        assert_eq!(outcome, Outcome::Done, "{verdict}");
        assert_eq!(hosts.lines.iter().filter(|call| call.line.starts_with("check ")).count(), 1);
        assert_eq!(hosts.lines.iter().filter(|call| call.line.starts_with("verdict ")).count(), 2);
        assert_eq!(hosts.lines.iter().filter(|call| call.line == BACKUP_CALL).count(), 2);
        assert_eq!(record.backup.as_deref(), Some(SECOND_BACKUP));
        assert_eq!(record.told_wait_ms("w1"), Some(1_200_000));
        assert_eq!(record.told_wait_ms("w2"), None);
        assert_eq!(record.window.as_ref().unwrap().from, SECOND);
        assert_eq!(record.window.as_ref().unwrap().to, SECOND);
        let first_stop = position(&hosts, "stop engine");
        let old_start = position(&hosts, "start engine");
        let drain = position(&hosts, "stop w1");
        let second_stop = hosts.lines.iter().rposition(|call| call.line == "stop engine").unwrap();
        assert!(first_stop < old_start && old_start < drain && drain < second_stop);
        assert!(position(&hosts, "stop w2") < second_stop);
        assert_eq!(hosts.lines[drain].at, 3 * SECOND, "old openings awaited before drain");
        assert_eq!(hosts.lines[second_stop + 1].line, BACKUP_CALL, "who was told stays the first stop's");
        assert_eq!(record.verdict.as_ref().unwrap().lines, verdict.lines().collect::<Vec<_>>());
        for backup in [BACKUP, SECOND_BACKUP] {
            assert!(
                record
                    .lines
                    .iter()
                    .any(|line| line.line == format!("temper-redeploy run: stop checkpoint {backup}; backup"))
            );
        }
        assert!(
            record
                .lines
                .iter()
                .any(|line| line.line.ends_with("verdict workers may stay: no or unknown; every part together"))
        );
        assert_eq!((record.outages[0].from, record.outages[0].to), (3 * SECOND, 5 * SECOND));
        for machine in std::iter::once(&hosts.engine).chain(&hosts.workers) {
            assert!(machine.running && machine.installed.values().all(|build| build == NEW));
        }
    }
}

#[test]
fn a_failed_stop_checkpoint_or_verdict_restores_the_old_engine_before_refusing() {
    for (step, answer) in [
        ("checkpoint", output(1, "temper-engine checkpoint: refused: files")),
        ("checkpoint", output(0, "no checkpoint line")),
        ("checkpoint", output(0, "temper-engine checkpoint: 42-not-a-name at commit 42")),
        ("checkpoint", Output { code: None, stdout: String::new(), stderr: "could not spawn checkpoint".into() }),
        ("verdict", output(1, "cannot read the checkpoint")),
        ("verdict", Output { code: None, stdout: String::new(), stderr: "could not spawn verdict".into() }),
    ] {
        let mut hosts = staged(2);
        hosts.opening(0, Opening::After { nanoseconds: 2 * SECOND, wires: WIRES.into() });
        if step == "checkpoint" {
            hosts.failures.insert((Part::Engine, BACKUP_CALL.into()), answer);
        } else {
            hosts.verdict_answer(0, "workers may stay: yes");
            hosts.verdict.push_back(answer);
        }
        let (outcome, record) = execute(&mut hosts, Mode::Engine);
        assert_eq!(outcome, Outcome::Refused, "{step}: {}", last(&record));
        all_old(&hosts);
        assert!(position(&hosts, "stop engine") < position(&hosts, "start engine"));
        assert_eq!(record.openings.len(), 2);
        assert_eq!(hosts.at, 3 * SECOND, "old openings awaited");
        if step == "verdict" {
            assert_eq!(record.backup.as_deref(), Some(BACKUP));
            assert_ne!(record.verdict.as_ref().unwrap().code, Some(0));
        } else {
            assert_eq!(record.backup, None);
            assert_eq!(record.verdict, None);
        }
        assert!(
            last(&record)
                .contains(&format!("refused: {}:", if step == "checkpoint" { "stop checkpoint" } else { step }))
        );
    }
}

#[test]
fn an_opening_refusal_is_kept_and_failed_with_recovery_information() {
    let mut hosts = staged(2);
    hosts.check_answer(0, "temper-engine upgrade: ran temper deleted-repository-forge-rows: 812 rows in 2 commits; time unavailable\nworkers may stay: yes");
    hosts.opening(1, Opening::Refused { nanoseconds: SECOND, text: "temper-worker-wire too old".into() });
    let (outcome, record) = execute(&mut hosts, Mode::Engine);
    assert_eq!(outcome, Outcome::Failed);
    assert_eq!(hosts.at, 2 * SECOND, "a refusal ends the wait immediately");
    assert!(record.openings.iter().any(|opening| opening.worker == "w2"
        && opening.refused
        && opening.line.ends_with("wires: temper-worker-wire too old")));
    assert!(last(&record).starts_with("temper-redeploy run: failed at openings:"));
    for word in ["w2", "swapped: engine", &format!("backup: {BACKUP}"), &format!("previous: engine={OLD}")] {
        assert!(last(&record).contains(word), "{} lacks {word}", last(&record));
    }
}

#[test]
fn an_unopened_worker_waits_for_the_constant_grace() {
    let mut hosts = staged(2);

    hosts.opening(0, Opening::Never);
    hosts.opening(1, Opening::After { nanoseconds: 2 * SECOND, wires: WIRES.into() });
    let (outcome, record) = execute(&mut hosts, Mode::Engine);
    assert_eq!(outcome, Outcome::Failed);
    assert_eq!(hosts.at, 301 * SECOND);
    assert_eq!(record.openings.len(), 1);
    assert!(last(&record).contains("w1"));
}

#[test]
fn failures_before_a_swap_restart_all_stopped_parts_without_installing_anything() {
    for failure in ["drain", "engine stop", "checkpoint"] {
        let mut hosts = staged(2);
        let mode = match failure {
            "drain" => {
                hosts.fail_line(Part::Worker(0), "stop w1", 1);
                Mode::Together
            }
            "engine stop" => {
                hosts.fail_line(Part::Engine, "stop engine", 1);
                Mode::Together
            }
            "checkpoint" => {
                hosts.fail_line(Part::Engine, BACKUP_CALL, 1);
                Mode::Engine
            }
            _ => unreachable!(),
        };
        let (outcome, record) = execute(&mut hosts, mode);
        assert_eq!(outcome, Outcome::Refused, "{failure}: {}", last(&record));
        all_old(&hosts);
        assert!(last(&record).starts_with("temper-redeploy run: refused: "));
        assert_eq!(record.openings.len(), 2);
        let failure_at = record.lines.iter().position(|line| line.line.contains(" failed: ")).unwrap();
        let restart_at = record.lines.iter().position(|line| line.line.contains(" started at ")).unwrap();
        assert!(failure_at < restart_at, "the failed step is journaled before recovery");
        if mode == Mode::Together {
            assert_eq!(record.outages.len(), 1);
            assert_eq!(record.outages[0].what, "every part");
            assert_eq!((record.outages[0].from, record.outages[0].to), (SECOND, SECOND));
            assert!(position(&hosts, "start w1") < position(&hosts, "start w2"));
            if failure == "engine stop" {
                assert!(position(&hosts, "start engine") < position(&hosts, "start w1"));
            }
        } else {
            assert!(record.window.is_some());
        }
    }
}

#[test]
fn unreadable_versions_refuse_before_stopping() {
    for failure in ["version", "inconsistent"] {
        let mut hosts = staged(2);
        match failure {
            "version" => {
                hosts.versions.insert((Part::Worker(1), "/bin/temper-worker".into()), output(0, "invalid"));
            }
            "inconsistent" => {
                hosts
                    .versions
                    .insert((Part::Engine, "/bin/temper".into()), output(0, &format!("temper 0.1.0 ({NEW})")));
            }
            _ => unreachable!(),
        }
        let (outcome, record) = execute(&mut hosts, Mode::Together);
        assert_eq!(outcome, Outcome::Refused, "{failure}");
        all_old(&hosts);
        assert!(hosts.lines.iter().all(|call| !call.line.starts_with("stop ") && !call.line.starts_with("start ")));
        assert!(last(&record).contains("refused:"));
    }
}

#[test]
fn a_failed_start_after_a_swap_stops_in_place_before_the_next_worker() {
    let mut hosts = staged(2);
    hosts.fail_line(Part::Worker(0), "start w1", 1);
    hosts.failures.get_mut(&(Part::Worker(0), "start w1".into())).unwrap().stderr =
        "first failure line\nsecond failure line".into();
    let (outcome, record) = execute(&mut hosts, Mode::Workers);
    assert_eq!(outcome, Outcome::Failed);
    assert!(!hosts.workers[0].running);
    assert!(hosts.workers[0].installed.values().all(|build| build == NEW));
    assert!(hosts.workers[1].running && hosts.workers[1].installed.values().all(|build| build == OLD));
    assert_eq!(hosts.lines.last().unwrap().line, "start w1", "no recovery host commands after a swap");
    assert_eq!(
        hosts.lines.iter().filter(|call| call.line == "start w1").count(),
        1,
        "no retry or recovery after a swap"
    );
    for word in ["failed at w1 start", "swapped: engine, w1", &format!("backup: {BACKUP}"), &format!("w1={OLD}")] {
        assert!(last(&record).contains(word), "{} lacks {word}", last(&record));
    }
}

#[test]
fn preserving_previous_binaries_skips_only_a_matching_build_and_swaps_even_at_commit() {
    let mut hosts = staged(1);
    hosts.engine.staged.insert(format!("/builds/{OLD}/temper-engine"), OLD.into());
    hosts.engine.staged.insert(format!("/builds/{OLD}/temper"), "000000000000".into());
    let (outcome, _) = execute(&mut hosts, Mode::Engine);
    assert_eq!(outcome, Outcome::Done);
    let copies: Vec<_> = hosts.lines.iter().filter(|call| call.line.contains("'cp'")).collect();
    assert_eq!(copies.len(), 1);
    assert!(copies[0].line.contains("'/bin/temper'"));
    assert_eq!(hosts.engine.staged[&format!("/builds/{OLD}/temper")], OLD);
    hosts.lines.clear();
    let (outcome, record) = execute(&mut hosts, Mode::Engine);
    assert_eq!(outcome, Outcome::Done);
    assert_eq!(record.previous[0].commit, NEW);
    assert_eq!(hosts.lines.iter().filter(|call| call.line.starts_with("swap ")).count(), 2);
}

#[test]
fn an_old_identical_opening_in_the_overlap_does_not_open_a_restarted_worker() {
    let mut hosts = staged(2);

    hosts.opening_rounds.insert(0, [Opening::default(), Opening::Never].into());
    let (outcome, record) = execute(&mut hosts, Mode::Workers);
    assert_eq!(outcome, Outcome::Failed);
    assert_eq!(hosts.at, 301 * SECOND);
    assert!(record.openings.iter().all(|opening| opening.worker != "w1"));
    assert!(hosts.lines.iter().all(|call| call.line != "stop w2"));
    assert!(last(&record).contains("w1"));
}

#[test]
fn logins_share_the_opening_deadline_and_fail_before_the_next_worker() {
    let mut hosts = staged(2);

    hosts.opening(0, Opening::After { nanoseconds: 2 * SECOND, wires: WIRES.into() });
    accounts(&mut hosts, false, false);
    let (outcome, record) = execute(&mut hosts, Mode::Workers);
    assert_eq!(outcome, Outcome::Failed);
    assert_eq!(hosts.at, 303 * SECOND, "opening and logins share the constant grace from w1's start at 3 s");
    assert!(hosts.lines.iter().all(|call| call.line != "stop w2"));
    assert!(last(&record).contains("failed at w1 logins"));
}

#[test]
fn recovery_starts_all_stopped_parts_even_when_its_log_cannot_be_read() {
    let mut hosts = staged(2);
    hosts.fail_line(Part::Engine, "log engine '0'", 1);
    let (outcome, record) = execute(&mut hosts, Mode::Together);
    assert_eq!(outcome, Outcome::Failed);
    all_old(&hosts);
    assert!(position(&hosts, "start engine") < position(&hosts, "start w1"));
    assert!(position(&hosts, "start w1") < position(&hosts, "start w2"));
    assert!(last(&record).contains("recovery at openings: engine log"));
}

#[test]
fn a_second_binary_swap_failure_is_failed_not_rolled_back() {
    let mut hosts = staged(1);
    let swap =
        fill(&hosts.description.engine.swap, &[("from", &format!("/builds/{NEW}/temper")), ("to", "/bin/temper")]);
    hosts.fail_line(Part::Engine, &swap, 1);
    let (outcome, record) = execute(&mut hosts, Mode::Engine);
    assert_eq!(outcome, Outcome::Failed);
    assert!(!hosts.engine.running);
    assert_eq!(hosts.engine.installed["temper-engine"], NEW);
    assert_eq!(hosts.engine.installed["temper"], OLD);
    assert_eq!(hosts.lines.last().unwrap().line, swap);
    assert!(last(&record).contains("failed at engine swap temper:"));
    assert!(last(&record).contains("swapped: engine"));
}

#[test]
fn openings_and_logins_cannot_succeed_at_a_poll_past_the_bound() {
    for mode in [Mode::Engine, Mode::Workers] {
        let mut hosts = staged(1);
        hosts.sleep_lag = 1_001_000_000;

        match mode {
            Mode::Engine => {
                hosts.opening(0, Opening::After { nanoseconds: 300 * SECOND + 100_000_000, wires: WIRES.into() });
            }
            Mode::Workers => {
                hosts.opening_rounds.insert(
                    0,
                    [Opening::default(), Opening::After { nanoseconds: 298 * SECOND, wires: WIRES.into() }].into(),
                );
                accounts(&mut hosts, false, false);
                accounts(&mut hosts, true, false);
            }
            Mode::Together => unreachable!(),
        }
        let (outcome, record) = execute(&mut hosts, mode);
        assert_eq!(outcome, Outcome::Failed, "{mode:?}: an answer after the grace cannot succeed");
        assert_eq!(hosts.at, 301 * SECOND + 150_000_000);
        match mode {
            Mode::Engine => {
                assert_eq!(record.openings, []);
                assert!(last(&record).contains("failed at openings"));
            }
            Mode::Workers => {
                assert_eq!(
                    hosts.lines.iter().filter(|call| call.line.starts_with("ctl engine 'account'")).count(),
                    1,
                    "no poll beyond the bound"
                );
                assert!(last(&record).contains("failed at w1 logins"));
            }
            Mode::Together => unreachable!(),
        }
    }
}
