//! Redeploy paths on the deployment's clock (redeploy.md, 4, 8).
use std::{cell::RefCell, io::Write, rc::Rc};
use temper_redeploy_tool::{
    fake::{Deployment, NEW, OLD, Opening, WIRES, description, output},
    hosts::{Hosts, Output, Part, fill},
    record::{Journal, Record},
    redeploy::{Mode, Outcome, redeploy},
};

pub(super) const SECOND: u64 = 1_000_000_000;
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
pub(super) fn grace(hosts: &mut Deployment, index: usize, ms: u64) {
    hosts.documents[index] = output(
        0,
        &serde_json::json!({"workers": [
            {"name": "not-this-worker", "graces": {"grace_ms": 1}},
            {"name": hosts.description.workers[index].name, "graces": {"grace_ms": ms}}
        ]})
        .to_string(),
    );
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
fn engine_alone_keeps_workers_and_records_the_first_opening_window() {
    let mut hosts = staged(2);
    for index in 0..2 {
        grace(&mut hosts, index, 300_000);
    }
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
fn a_refusing_check_restarts_the_old_engine_and_waits_without_a_swap() {
    let mut hosts = staged(2);
    hosts.check_answer(1, "temper-engine upgrade: cannot start\n");
    hosts.opening(0, Opening::After { nanoseconds: 2 * SECOND, wires: WIRES.into() });
    let (outcome, record) = execute(&mut hosts, Mode::Engine);
    assert_eq!(outcome, Outcome::Refused);
    all_old(&hosts);
    assert!(position(&hosts, "start engine") > position(&hosts, &format!("check '/builds/{NEW}/temper-engine'")));
    assert_eq!(record.window.as_ref().unwrap().worker, "w2");
    assert_eq!(record.openings.len(), 2);
    assert_eq!(hosts.at, 3 * SECOND);
    assert_eq!(record.check.as_ref().unwrap().code, Some(1));
    assert!(last(&record).starts_with("temper-redeploy run: refused: check:"));
}

#[test]
fn no_unknown_and_absent_last_verdict_restore_then_drain_all_without_rechecking() {
    for verdict in [
        "workers may stay: no",
        "workers may stay: unknown; the copy keeps no wires",
        "no verdict",
        "workers may stay: yes\nworkers may stay: no",
    ] {
        let mut hosts = staged(2);
        hosts.check_answer(0, verdict);
        hosts.opening(0, Opening::After { nanoseconds: SECOND, wires: WIRES.into() });
        let (outcome, record) = execute(&mut hosts, Mode::Engine);
        assert_eq!(outcome, Outcome::Done, "{verdict}");
        assert_eq!(hosts.lines.iter().filter(|call| call.line.starts_with("check ")).count(), 1);
        assert_eq!(hosts.lines.iter().filter(|call| call.line == "stop engine").count(), 2);
        assert_eq!(hosts.batches, [vec![(Part::Worker(0), "stop w1".into()), (Part::Worker(1), "stop w2".into())]]);
        let old_start = position(&hosts, "start engine");
        let second_stop = hosts.lines.iter().rposition(|call| call.line == "stop engine").unwrap();
        assert!(old_start < position(&hosts, "stop w1"));
        assert!(position(&hosts, "stop w2") < second_stop);
        assert_eq!(hosts.lines[position(&hosts, "stop w1")].at, 2 * SECOND, "wait for old openings before draining");
        assert_eq!(record.window.as_ref().unwrap().from, SECOND);
        assert_eq!(record.outages.len(), 1);
        assert_eq!(record.outages[0].what, "every part");
        assert_eq!((record.outages[0].from, record.outages[0].to), (2 * SECOND, 3 * SECOND));
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
    hosts.check_answer(0, "workers may stay: no");
    hosts.opening(1, Opening::After { nanoseconds: 2 * SECOND, wires: WIRES.into() });
    let (outcome, record) = execute(&mut hosts, Mode::Together);
    assert_eq!(outcome, Outcome::Done);
    assert_eq!(hosts.batches.len(), 1);
    assert_eq!(hosts.batches[0].len(), 2);
    assert!(position(&hosts, "stop w2") < position(&hosts, "stop engine"));
    assert!(position(&hosts, "stop engine") < position(&hosts, &format!("check '/builds/{NEW}/temper-engine'")));
    assert!(position(&hosts, "start engine") < position(&hosts, "start w1"));
    assert!(position(&hosts, "start w1") < position(&hosts, "start w2"));
    assert!(record.window.is_none());
    assert_eq!(
        (record.outages[0].what.as_str(), record.outages[0].from, record.outages[0].to),
        ("every part", SECOND, 3 * SECOND)
    );
}

#[test]
fn patched_checks_back_up_at_the_seam_clock_before_any_swap() {
    let mut hosts = staged(1);
    hosts.at = 1_700_000_000 * SECOND;
    hosts
        .check_answer(0, "temper-engine upgrade: patched generation 3\ntemper-engine upgrade: workers may stay: yes\n");
    let (outcome, record) = execute(&mut hosts, Mode::Engine);
    assert_eq!(outcome, Outcome::Done);
    let backup = format!("{NEW}-20231114T221320Z");
    assert_eq!(record.backup.as_deref(), Some(backup.as_str()));
    let backup_at = position(&hosts, &format!("backup '{backup}'"));
    let swap_at = hosts.lines.iter().position(|call| call.line.starts_with("swap ")).unwrap();
    assert!(backup_at < swap_at);
}

#[test]
fn an_opening_refusal_is_kept_and_failed_with_recovery_information() {
    let mut hosts = staged(2);
    hosts.check_answer(0, "temper-engine upgrade: patched generation 3\nworkers may stay: yes");
    hosts.opening(1, Opening::Refused { nanoseconds: SECOND, text: "temper-worker-wire too old".into() });
    let (outcome, record) = execute(&mut hosts, Mode::Engine);
    assert_eq!(outcome, Outcome::Failed);
    assert_eq!(hosts.at, 2 * SECOND, "a refusal ends the wait immediately");
    assert!(record.openings.iter().any(|opening| opening.worker == "w2"
        && opening.refused
        && opening.line.ends_with("wires: temper-worker-wire too old")));
    assert!(last(&record).starts_with("temper-redeploy run: failed at openings:"));
    for word in ["w2", "swapped: engine", "backup: abcdef012345-19700101T000001Z", &format!("previous: engine={OLD}")] {
        assert!(last(&record).contains(word), "{} lacks {word}", last(&record));
    }
}

#[test]
fn an_unopened_worker_waits_for_the_largest_not_smallest_grace() {
    let mut hosts = staged(2);
    grace(&mut hosts, 0, 1000);
    grace(&mut hosts, 1, 4000);
    hosts.opening(0, Opening::Never);
    hosts.opening(1, Opening::After { nanoseconds: 2 * SECOND, wires: WIRES.into() });
    let (outcome, record) = execute(&mut hosts, Mode::Engine);
    assert_eq!(outcome, Outcome::Failed);
    assert_eq!(hosts.at, 5 * SECOND);
    assert_eq!(record.openings.len(), 1);
    assert!(last(&record).contains("w1"));
}

#[test]
fn failures_before_a_swap_restart_all_stopped_parts_without_installing_anything() {
    for failure in ["check", "drain", "engine stop", "backup"] {
        let mut hosts = staged(2);
        let mode = match failure {
            "check" => {
                hosts.check = Output { code: None, stdout: String::new(), stderr: "could not spawn check".into() };
                Mode::Engine
            }
            "drain" => {
                hosts.fail_line(Part::Worker(0), "stop w1", 1);
                Mode::Together
            }
            "engine stop" => {
                hosts.fail_line(Part::Engine, "stop engine", 1);
                Mode::Together
            }
            "backup" => {
                hosts.check_answer(0, "upgrade: patched store\nworkers may stay: yes");
                hosts.fail_line(Part::Engine, &format!("backup '{NEW}-19700101T000001Z'"), 1);
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
fn unreadable_versions_or_graces_refuse_before_stopping() {
    for failure in ["version", "inconsistent", "document", "grace"] {
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
            "document" => hosts.documents[1] = output(1, ""),
            "grace" => hosts.documents[1] = output(0, "{\"workers\": []}"),
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
    for word in ["failed at w1 start", "swapped: engine, w1", "backup: none", &format!("w1={OLD}")] {
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
    grace(&mut hosts, 0, 2000);
    hosts.opening_rounds.insert(0, [Opening::default(), Opening::Never].into());
    let (outcome, record) = execute(&mut hosts, Mode::Workers);
    assert_eq!(outcome, Outcome::Failed);
    assert_eq!(hosts.at, 3 * SECOND);
    assert!(record.openings.iter().all(|opening| opening.worker != "w1"));
    assert!(hosts.lines.iter().all(|call| call.line != "stop w2"));
    assert!(last(&record).contains("w1"));
}

#[test]
fn logins_share_the_opening_deadline_and_fail_before_the_next_worker() {
    let mut hosts = staged(2);
    grace(&mut hosts, 0, 3000);
    hosts.opening(0, Opening::After { nanoseconds: 2 * SECOND, wires: WIRES.into() });
    accounts(&mut hosts, false, false);
    let (outcome, record) = execute(&mut hosts, Mode::Workers);
    assert_eq!(outcome, Outcome::Failed);
    assert_eq!(hosts.at, 6 * SECOND, "opening and logins share w1's three seconds from its start at 3 s");
    assert!(hosts.lines.iter().all(|call| call.line != "stop w2"));
    assert!(last(&record).contains("failed at w1 logins"));
}

#[test]
fn recovery_starts_all_stopped_parts_even_when_its_log_cannot_be_read() {
    let mut hosts = staged(2);
    hosts.check_answer(1, "check refused");
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
        grace(&mut hosts, 0, 1500);
        match mode {
            Mode::Engine => hosts.opening(0, Opening::After { nanoseconds: 2 * SECOND, wires: WIRES.into() }),
            Mode::Workers => {
                hosts.opening_rounds.insert(
                    0,
                    [Opening::default(), Opening::After { nanoseconds: SECOND, wires: WIRES.into() }].into(),
                );
                accounts(&mut hosts, false, false);
                accounts(&mut hosts, true, false);
            }
            Mode::Together => unreachable!(),
        }
        let (outcome, record) = execute(&mut hosts, mode);
        assert_eq!(outcome, Outcome::Failed, "{mode:?}: an answer after the grace cannot succeed");
        assert_eq!(hosts.at, 3 * SECOND);
        match mode {
            Mode::Engine => {
                assert_eq!(record.openings, []);
                assert!(last(&record).contains("failed at openings"));
            }
            Mode::Workers => {
                assert_eq!(
                    hosts.lines.iter().filter(|call| call.line.starts_with("ctl engine ")).count(),
                    1,
                    "no poll beyond the bound"
                );
                assert!(last(&record).contains("failed at w1 logins"));
            }
            Mode::Together => unreachable!(),
        }
    }
}
