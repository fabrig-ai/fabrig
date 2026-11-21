//! Scripted referee value fixtures (tools/crates/design.md, 3.2).
use crate::{
    fake::{Deployment, WIRES, description, output},
    record::{Grace, Journal, Opening, Record, TaskBefore},
    redeploy::Mode,
};
use serde_json::{Value, json};

pub const BEGAN: u64 = 1_794_564_000_000_000_000;
pub const SECOND: u64 = 1_000_000_000;
pub fn answer(hosts: &mut Deployment, words: &str, value: &Value) {
    hosts.ctl_answer(words, output(0, &value.to_string()));
}
pub fn row(task: u64, label: u64, phase: &str) -> Value {
    json!({"chip": {"task": task, "phase": phase}, "label": label})
}
pub fn task(task: u64, label: u64, phase: &str, held: Option<&str>) -> TaskBefore {
    TaskBefore { task, project: 1, label, phase: phase.into(), held: held.map(str::to_owned) }
}
pub fn record(workers: usize) -> Record {
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
pub fn memory(record: Record) -> Journal {
    Journal::new(record, None, Box::new(std::io::sink()))
}
pub fn lines(journal: &Journal) -> Vec<&str> {
    journal
        .record
        .lines
        .iter()
        .map(|line| line.line.strip_prefix("temper-redeploy referee: ").expect("referee prefix"))
        .collect()
}
pub fn empty(hosts: &mut Deployment) {
    answer(hosts, "me --json", &json!({"projects": [], "cut": false}));
}
pub fn activity(task: u64, at: u64, attempt: u64, end: &Value) -> Value {
    json!({"at": at, "event": {"owner": {"source": {"task": task, "line": {"run_ended": {"attempt": attempt, "end": end}}}}}})
}
pub fn activity_answer(hosts: &mut Deployment, task: u64, rows: &Value) {
    answer(hosts, &format!("task activity {task} --json"), &json!({"rows": rows, "before": null}));
}
