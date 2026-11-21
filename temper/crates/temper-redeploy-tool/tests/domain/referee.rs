//! Referee judgements on fake hosts and values (redeploy.md, 6).
use serde_json::json;
use temper_redeploy_tool::{
    fake::{Deployment, WIRES, description, output},
    hosts::{Output, Part, ctl},
    record::{Before, Journal, Opening, Record, TaskBefore, Told, Window},
    referee::{
        before, judge,
        judgement::{Readings, evaluate},
    },
    testing::referee::{BEGAN, SECOND, activity, activity_answer, answer, empty, lines, memory, record, row, task},
};

#[test]
fn snapshot_follows_list_words_and_reads_only_held_tasks() {
    let d = description(0);
    let mut hosts = Deployment::new(d.clone());
    answer(&mut hosts, "me --json", &json!({"projects": [{"project": 1}, {"project": 2}], "cut": false}));
    answer(
        &mut hosts,
        "task list --project 1 --json",
        &json!({"tasks": {"rows": [row(7, 0, "running"), row(8, 2, "waiting")]}, "read_on": "task list --project 1 --after abc --json"}),
    );
    answer(
        &mut hosts,
        "task list --project 1 --after abc --json",
        &json!({"live": [row(9, 2, "active")], "read_on": null}),
    );
    answer(
        &mut hosts,
        "task list --project 2 --json",
        &json!({"tasks": {"rows": [row(10, 0, "held")]}, "read_on": null}),
    );
    answer(
        &mut hosts,
        "task show 10 --json",
        &json!({"task": {"hold": {"reason": {"failures": {"class": "infrastructure"}}}}}),
    );
    assert_eq!(
        before(&d, &mut hosts).unwrap(),
        Before {
            projects: vec![1, 2],
            cut: false,
            tasks: vec![
                task(7, 0, "running", None),
                task(8, 2, "waiting", None),
                task(9, 2, "active", None),
                TaskBefore { project: 2, ..task(10, 0, "held", Some("infrastructure")) }
            ]
        }
    );
    assert_eq!(hosts.lines.len(), 5);
    assert!(hosts.lines.iter().all(|call| call.part == Part::Engine
        && call.line.ends_with("'--json' </dev/null")
        && call.line.matches("'--json'").count() == 1));
    hosts.ctl_answer(
        "account list --json",
        Output { code: None, stdout: String::new(), stderr: "could not spawn".into() },
    );
    let why = ctl(&d, &["account", "list"], &mut hosts).unwrap_err();
    assert!(why.contains("account list") && why.contains("could not spawn"));
    assert_eq!(hosts.lines.last().unwrap().line, "ctl engine 'account' 'list' '--json' </dev/null");
}

#[test]
fn openings_window_boundaries_outages_and_other_holds_are_judged() {
    let d = description(2);
    let mut hosts = Deployment::new(d.clone());
    empty(&mut hosts);
    let mut record = record(2);
    record.graces[1].grace_ms = 400_000;
    for (duration, failed, word) in [(31, false, "within"), (400, true, "past"), (401, true, "past")] {
        record.window = Some(Window { from: BEGAN, to: BEGAN + duration * SECOND, worker: "w2".into() });
        record.openings[1].at = BEGAN + duration * SECOND;
        let mut journal = memory(record.clone());
        assert_eq!(judge(&d, &mut journal, &mut hosts), !failed);
        assert!(lines(&journal).contains(&format!("window {duration}.0 s").as_str()));
        assert!(
            lines(&journal).contains(&format!("worker w2 back in {duration}.0 s {word} its grace 400.0 s").as_str())
        );
        assert!(
            !lines(&journal).iter().any(|line| line.starts_with("worker w1 back")),
            "w1's opening predates the start ridden"
        );
    }
    record.window = None;
    record.openings.pop();
    let mut journal = memory(record.clone());
    assert!(!judge(&d, &mut journal, &mut hosts));
    assert!(lines(&journal).contains(&"worker w2 did not open"));
    record.openings.push(Opening {
        worker: "w2".into(),
        at: BEGAN,
        line: "temper-engine serve: worker w2 refused at its opening: wires: too old".into(),
        refused: true,
    });
    let mut journal = memory(record);
    assert!(!judge(&d, &mut journal, &mut hosts));
    assert!(lines(&journal).contains(&"worker w2 refused: wires: too old"));
    assert_eq!(lines(&journal).last(), Some(&"fail: 1"));
    let d = description(0);
    let mut hosts = Deployment::new(d.clone());
    answer(&mut hosts, "me --json", &json!({"projects": [{"project": 1}], "cut": true}));
    answer(
        &mut hosts,
        "task list --project 1 --json",
        &json!({"tasks": {"rows": [row(10, 0, "held"), row(11, 2, "held"), row(12, 0, "held")]}, "read_on": null}),
    );
    for (task, reason) in
        [(10, json!({"dependency": {"task": 9}})), (11, json!("budget")), (12, json!({"failures": {"class": "run"}}))]
    {
        answer(&mut hosts, &format!("task show {task} --json"), &json!({"task": {"hold": {"reason": reason}}}));
        activity_answer(&mut hosts, task, &json!([]));
    }
    let mut journal = memory(record_without_workers());
    assert!(judge(&d, &mut journal, &mut hosts));
    assert_eq!(
        lines(&journal),
        [
            "task 10 held: dependency",
            "goal 11 held: budget",
            "task 12 held: failures run",
            "memberships cut: projects past them not read",
            "read: 1 projects, 3 tasks, since 2026-11-13T10:00:00Z",
            "pass"
        ]
    );
}

fn record_without_workers() -> Record {
    record(0)
}

#[test]
fn each_workers_return_is_judged_against_its_given_wait() {
    let d = description(2);
    let mut hosts = Deployment::new(d.clone());
    empty(&mut hosts);
    for (wait_ms, a, b, failures) in [
        (1_200_000, 900, 900, 1),
        (1_200_000, 1199, 299, 0),
        (1_200_000, 1200, 300, 2),
        (1_200_000, 1201, 301, 2),
        (100_000, 299, 299, 0),
        (100_000, 300, 300, 2),
    ] {
        let mut record = record(2);
        record.told = Some(vec![
            Told { worker: "w2".into(), wait_ms: None },
            Told { worker: "w1".into(), wait_ms: Some(wait_ms) },
        ]);
        record.window =
            Some(Window { from: BEGAN, to: BEGAN + a.min(b) * SECOND, worker: if a < b { "w1" } else { "w2" }.into() });
        record.openings[0].at = BEGAN + a * SECOND;
        record.openings[1].at = BEGAN + b * SECOND;
        let mut journal = memory(record);
        assert_eq!(judge(&d, &mut journal, &mut hosts), failures == 0);
        let restart = (wait_ms / 1000).max(300);
        let a_word = if a >= restart { "past" } else { "within" };
        let b_word = if b >= 300 { "past" } else { "within" };
        assert_eq!(
            lines(&journal),
            vec![
                format!("worker w1 agreed {WIRES}"),
                format!("worker w2 agreed {WIRES}"),
                format!("window {}.0 s", a.min(b)),
                format!("worker w1 back in {a}.0 s {a_word} its restart wait {restart}.0 s"),
                format!("worker w2 back in {b}.0 s {b_word} its grace 300.0 s"),
                "read: 0 projects, 0 tasks, since 2026-11-13T10:00:00Z".into(),
                if failures == 0 { "pass".into() } else { format!("fail: {failures}") }
            ]
        );
    }
}

#[test]
fn unread_calls_fail_without_abandoning_other_projects_or_before_tasks() {
    let d = description(0);
    let mut hosts = Deployment::new(d.clone());
    answer(&mut hosts, "me --json", &json!({"projects": [{"project": 1}, {"project": 2}], "cut": false}));
    hosts.ctl_answer(
        "task list --project 1 --json",
        Output { code: Some(9), stdout: String::new(), stderr: "offline\nretry later".into() },
    );
    answer(
        &mut hosts,
        "task list --project 2 --json",
        &json!({"tasks": {"rows": [row(9, 2, "held")]}, "read_on": null}),
    );
    hosts.ctl_answer("task show 9 --json", output(0, "not JSON"));
    assert!(before(&d, &mut hosts).unwrap_err().contains("task list --project 1"));
    let mut record = record(0);
    record.before.tasks.push(task(7, 0, "running", None));
    activity_answer(&mut hosts, 7, &json!([activity(7, BEGAN, 3, &json!({"failed": {"class": "lost"}}))]));
    hosts
        .ctl_answer("task activity 9 --json", Output { code: None, stdout: String::new(), stderr: "no engine".into() });
    let mut journal = memory(record);
    assert!(!judge(&d, &mut journal, &mut hosts));
    let lines = lines(&journal);
    assert!(lines.contains(&"task 7 attempt 3 failed: lost"));
    assert_eq!(lines.iter().filter(|line| line.starts_with("could not read: ")).count(), 3);
    assert!(lines.iter().any(
        |line| line.starts_with("could not read: task list --project 1: ") && line.contains("offline retry later")
    ));
    assert!(lines.iter().all(|line| !line.contains(['\r', '\n'])));
    assert!(lines.iter().any(|line| line.starts_with("could not read: task show 9: JSON: ")));
    assert!(
        lines.iter().any(|line| line.starts_with("could not read: task activity 9: ") && line.contains("no engine"))
    );
    assert_eq!(lines.last(), Some(&"fail: 4"));
    hosts.ctl.remove("me --json");
    hosts.ctl_answer("me --json", output(1, ""));
    let mut journal = memory(journal.record);
    journal.record.lines.clear();
    assert!(!judge(&d, &mut journal, &mut hosts));
    assert_eq!(lines_for_failures(&journal), 2); // me unread, plus before task 7's lost attempt
}

fn lines_for_failures(journal: &Journal) -> usize {
    lines(journal)
        .last()
        .expect("verdict")
        .strip_prefix("fail: ")
        .expect("failed verdict")
        .parse()
        .expect("failure count")
}

#[test]
fn summary_uses_utc_seconds_across_leap_and_century_boundaries() {
    for (nanos, expected) in [
        (0, "1970-01-01T00:00:00Z"),
        (951_782_400_999_999_999, "2000-02-29T00:00:00Z"),
        (4_107_542_400_000_000_000, "2100-03-01T00:00:00Z"),
        (BEGAN, "2026-11-13T10:00:00Z"),
    ] {
        let mut record = record(0);
        record.began = nanos;
        let result = evaluate(&description(0), &record, &Readings::default());
        assert_eq!(result.lines, [format!("read: 0 projects, 0 tasks, since {expected}"), "pass".into()]);
    }
}
