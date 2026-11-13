//! Scripted ctl documents follow temper-client-tools' JSON renderers (redeploy.md, 6).
use serde_json::{Value, json};
use temper_redeploy_tool::{
    cli::{Action, execute},
    fake::{Deployment, WIRES, description, output},
    hosts::{Output, Part, ctl},
    record::{Before, Grace, Journal, Opening, Outage, Record, TaskBefore, Window},
    redeploy::Mode,
    referee::{
        before, judge,
        judgement::{Readings, evaluate},
    },
};
const BEGAN: u64 = 1_794_564_000_000_000_000;
const SECOND: u64 = 1_000_000_000;
fn answer(hosts: &mut Deployment, words: &str, value: &Value) {
    hosts.ctl_answer(words, output(0, &value.to_string()));
}
fn row(task: u64, label: u64, phase: &str) -> Value {
    json!({"chip": {"task": task, "phase": phase}, "label": label})
}
fn task(task: u64, label: u64, phase: &str, held: Option<&str>) -> TaskBefore {
    TaskBefore { task, project: 1, label, phase: phase.into(), held: held.map(str::to_owned) }
}
fn record(workers: usize) -> Record {
    let mut record = Record::new("abcdef012345", Mode::Engine);
    record.began = BEGAN;
    for worker in description(workers).workers {
        record.openings.push(Opening {
            worker: worker.name.clone(),
            at: BEGAN,
            line: format!("temper-engine serve: worker {} opened: {WIRES}", worker.name),
            refused: false,
        });
        record.graces.push(Grace { worker: worker.name, grace_ms: 300_000 });
    }
    record
}
fn memory(record: Record) -> Journal {
    Journal::new(record, None, Box::new(std::io::sink()))
}
fn lines(journal: &Journal) -> Vec<&str> {
    journal
        .record
        .lines
        .iter()
        .map(|line| line.line.strip_prefix("temper-redeploy referee: ").expect("referee prefix"))
        .collect()
}
fn empty(hosts: &mut Deployment) {
    answer(hosts, "me --json", &json!({"projects": [], "cut": false}));
}
fn activity(task: u64, at: u64, attempt: u64, end: &Value) -> Value {
    json!({"at": at, "event": {"owner": {"source": {"task": task, "line": {"run_ended": {"attempt": attempt, "end": end}}}}}})
}
fn activity_answer(hosts: &mut Deployment, task: u64, rows: &Value) {
    answer(hosts, &format!("task activity {task} --json"), &json!({"rows": rows, "before": null}));
}

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
fn pattern_judgement_pages_activity_and_prints_every_finding_through_the_journal() {
    let d = description(1);
    let mut hosts = Deployment::new(d.clone());
    hosts.at = BEGAN + 400 * SECOND;
    let mut record = record(1);
    record.before = Before {
        projects: vec![1, 2],
        cut: false,
        tasks: vec![
            task(7, 0, "running", None),
            task(8, 0, "running", None),
            task(10, 0, "held", Some("infrastructure")),
        ],
    };
    record.window = Some(Window { from: BEGAN, to: BEGAN + 31 * SECOND, worker: "w1".into() });
    record.outages.push(Outage { what: "every part".into(), from: BEGAN, to: BEGAN + 2 * SECOND });
    answer(&mut hosts, "me --json", &json!({"projects": [{"project": 1}, {"project": 2}], "cut": false}));
    answer(
        &mut hosts,
        "task list --project 1 --json",
        &json!({"tasks": {"rows": [row(8, 0, "running"), row(9, 2, "held")]}, "read_on": null}),
    );
    answer(
        &mut hosts,
        "task list --project 2 --json",
        &json!({"tasks": {"rows": [row(10, 0, "held")]}, "read_on": null}),
    );
    for task in [9, 10] {
        answer(
            &mut hosts,
            &format!("task show {task} --json"),
            &json!({"task": {"hold": {"reason": {"failures": {"class": "infrastructure"}}}}}),
        );
    }
    answer(
        &mut hosts,
        "task activity 7 --json",
        &json!({"rows": [activity(7, BEGAN + SECOND, 3, &json!({"failed": {"class": "lost"}}))], "before": {"value": "abc"}}),
    );
    // Oldest first within a page: the old row must not hide a boundary row following it.
    answer(
        &mut hosts,
        "task activity 7 --before abc --json",
        &json!({"rows": [activity(7, BEGAN - 1, 1, &json!({"failed": {"class": "lost"}})), activity(7, BEGAN, 2, &json!("refused"))], "before": {"value": "do-not-read"}}),
    );
    activity_answer(
        &mut hosts,
        8,
        &json!([
            activity(8, BEGAN - 1, 1, &json!({"failed": {"class": "lost"}})),
            activity(8, BEGAN, 2, &json!({"failed": {"class": "run"}}))
        ]),
    );
    activity_answer(
        &mut hosts,
        9,
        &json!([
            activity(9, BEGAN, 1, &json!("finished")),
            activity(9, BEGAN, 2, &json!("cancelled")),
            activity(9, BEGAN, 3, &json!("parked"))
        ]),
    );
    activity_answer(&mut hosts, 10, &json!([]));
    let scratch = skein_scratch::Scratch::new("referee-pattern");
    let printed = scratch.path().join("printed");
    let file = scratch.path().join("record");
    let mut journal = Journal::new(record, Some(file.clone()), Box::new(std::fs::File::create(&printed).unwrap()));
    assert!(!judge(&d, &mut journal, &mut hosts));
    assert_eq!(
        lines(&journal),
        vec![
            format!("worker w1 agreed {WIRES}"),
            "window 31.0 s within w1's grace 300.0 s".into(),
            "every part was out 2.0 s".into(),
            "task 7 attempt 3 failed: lost".into(),
            "task 8 attempt 2 failed: run, its own".into(),
            "goal 9 held: failures infrastructure".into(),
            "read: 2 projects, 4 tasks, since 2026-11-13T10:00:00Z".into(),
            "fail: 2".into()
        ]
    );
    let printed = std::fs::read_to_string(printed).unwrap();
    assert_eq!(
        printed,
        format!("{}\n", journal.record.lines.iter().map(|line| line.line.as_str()).collect::<Vec<_>>().join("\n"))
    );
    assert!(journal.record.lines.iter().all(|line| line.at == hosts.at));
    assert_eq!(serde_json::from_slice::<Record>(&std::fs::read(file).unwrap()).unwrap(), journal.record);
    assert!(!hosts.lines.iter().any(|call| call.line.contains("do-not-read")));
}

#[test]
fn openings_window_boundaries_outages_and_other_holds_are_judged() {
    let d = description(2);
    let mut hosts = Deployment::new(d.clone());
    empty(&mut hosts);
    let mut record = record(2);
    // The window is compared to the smallest grace, not the opening's worker.
    record.graces[1].grace_ms = 400_000;
    for (duration, failed, word) in [(31, false, "within"), (300, true, "past"), (301, true, "past")] {
        record.window = Some(Window { from: BEGAN, to: BEGAN + duration * SECOND, worker: "w2".into() });
        let mut journal = memory(record.clone());
        assert_eq!(judge(&d, &mut journal, &mut hosts), !failed);
        assert!(lines(&journal).contains(&format!("window {duration}.0 s {word} w1's grace 300.0 s").as_str()));
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
fn referee_cli_returns_the_verdict_without_rewriting_the_record() {
    let scratch = skein_scratch::Scratch::new("referee-cli");
    let path = scratch.path().join("record.json");
    let d = description(1);
    let mut hosts = Deployment::new(d.clone());
    empty(&mut hosts);
    for (record, expected) in [(record(1), 0), (record(0), 1)] {
        let mut bytes = serde_json::to_vec_pretty(&record).unwrap();
        bytes.push(b'\n');
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(
            execute(&d, &Action::Referee { record: path.clone() }, &mut hosts, Box::new(std::io::sink())).unwrap(),
            expected
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
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
