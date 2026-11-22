//! Referee journal files and CLI reads (redeploy.md, 6).
use serde_json::{Value, json};
use temper_redeploy_tool::{
    cli::{Action, execute},
    fake::{Deployment, WIRES, description},
    record::{Before, Journal, Outage, Record, Window},
    referee::judge,
    testing::referee::{BEGAN, SECOND, activity, activity_answer, answer, empty, lines, record, row, task},
};

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
            "window 31.0 s".into(),
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
fn a_record_without_told_is_read_by_a_later_referee_as_no_worker_told() {
    let d = description(1);
    let mut hosts = Deployment::new(d.clone());
    empty(&mut hosts);
    let mut record = record(1);
    record.window = Some(Window { from: BEGAN, to: BEGAN + 900 * SECOND, worker: "w1".into() });
    record.openings[0].at = BEGAN + 900 * SECOND;
    let mut value = serde_json::to_value(record).unwrap();
    assert_eq!(value["told"], Value::Null);
    value.as_object_mut().unwrap().remove("told").unwrap();
    let scratch = skein_scratch::Scratch::new("redeploy-before-told");
    let path = scratch.path().join("record.json");
    let bytes = serde_json::to_vec(&value).unwrap();
    std::fs::write(&path, &bytes).unwrap();
    let printed = scratch.path().join("printed");
    assert_eq!(
        execute(
            &d,
            &Action::Referee { record: path.clone() },
            &mut hosts,
            Box::new(std::fs::File::create(&printed).unwrap())
        )
        .unwrap(),
        1
    );
    let printed = std::fs::read_to_string(printed).unwrap();
    assert!(printed.contains("temper-redeploy referee: worker w1 back in 900.0 s past its grace 300.0 s\n"));
    assert!(printed.ends_with("temper-redeploy referee: fail: 1\n"));
    assert_eq!(std::fs::read(path).unwrap(), bytes, "referee does not rewrite the older record");
}

#[test]
fn referee_cli_returns_the_verdict_without_rewriting_the_record() {
    let scratch = skein_scratch::Scratch::new("referee-cli");
    let path = scratch.path().join("record.json");
    let d = description(1);
    let mut hosts = Deployment::new(d.clone());
    empty(&mut hosts);
    for (record, expected) in [(record(1), 0), (record(0), 1)] {
        let mut value = serde_json::to_value(&record).unwrap();
        value.as_object_mut().unwrap().remove("checkpoint").unwrap();
        value.as_object_mut().unwrap().remove("verdict").unwrap();
        let mut bytes = serde_json::to_vec_pretty(&value).unwrap();
        bytes.push(b'\n');
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(
            execute(&d, &Action::Referee { record: path.clone() }, &mut hosts, Box::new(std::io::sink())).unwrap(),
            expected
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
}
