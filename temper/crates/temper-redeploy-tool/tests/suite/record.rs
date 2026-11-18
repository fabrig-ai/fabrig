//! The public JSON members and atomic whole journal (redeploy.md, 5).
use serde_json::{Value, json};
use std::io::Read;
use temper_redeploy_tool::record::{Journal, Record};

#[test]
fn the_record_round_trips_and_each_printed_line_replaces_the_whole_file() {
    let scratch = skein_scratch::Scratch::new("redeploy-record");
    let file = scratch.path().join("record.json");
    let printed = scratch.path().join("stdout");
    let value = json!({
        "commit": "abcdef012345", "mode": "workers", "began": 123,
        "previous": [{"part": "engine", "commit": "0123456789ab"}],
        "graces": [{"worker": "w1", "grace_ms": 30000}],
        "told": [{"worker": "w1", "wait_ms": 1_200_000}],
        "before": {"projects": [2], "cut": false, "tasks": [{"task": 3, "project": 2, "label": 2, "phase": "held", "held": "lost"}]},
        "lines": [], "checkpoint": "00000000000000000041-0123456789abcdef",
        "check": {"code": 0, "lines": ["workers may stay: yes"]},
        "verdict": {"code": 0, "lines": ["workers may stay: yes"]},
        "backup": "00000000000000000042-0123456789abcdef",
        "window": {"from": 123, "to": 124, "worker": "w1"},
        "outages": [{"what": "w1", "from": 124, "to": 125}],
        "openings": [{"worker": "w1", "at": 124, "line": "worker w1 opened: a-wire 1", "refused": false}],
        "outcome": "done", "ended": 126
    });
    let record: Record = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(&record).unwrap(), value);
    let mut journal = Journal::new(record, Some(file.clone()), Box::new(std::fs::File::create(&printed).unwrap()));
    journal.save().unwrap();
    let mut old = std::fs::File::open(&file).unwrap();
    journal.line(127, "temper-redeploy run: done").unwrap();
    let mut previous = String::new();
    old.read_to_string(&mut previous).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&previous).unwrap(), value, "an open reader retains the old whole record");
    let disk: Record = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    assert_eq!(disk, journal.record);
    assert_eq!(disk.lines[0].at, 127);
    assert_eq!(std::fs::read_to_string(printed).unwrap(), "temper-redeploy run: done\n");
    assert!(!scratch.path().join("record.json.tmp").exists());
    let mut memory = Journal::new(disk, None, Box::new(std::io::sink()));
    memory.line(128, "temper-redeploy referee: pass").unwrap();
    assert_eq!(memory.record.lines.len(), 2);
    for mode in ["engine", "together"] {
        let mut value = value.clone();
        value["mode"] = json!(mode);
        value["outcome"] = Value::Null;
        let record: Record = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(record).unwrap(), value);
    }
}

#[test]
fn records_before_checkpoints_read_the_new_fields_as_null() {
    let mut value =
        serde_json::to_value(Record::new("abcdef012345", temper_redeploy_tool::redeploy::Mode::Engine)).unwrap();
    value.as_object_mut().unwrap().remove("checkpoint").unwrap();
    value.as_object_mut().unwrap().remove("verdict").unwrap();
    let record: Record = serde_json::from_value(value).unwrap();
    assert_eq!(record.checkpoint, None);
    assert_eq!(record.verdict, None);
    let value = serde_json::to_value(record).unwrap();
    assert_eq!(value["checkpoint"], Value::Null);
    assert_eq!(value["verdict"], Value::Null);
}
