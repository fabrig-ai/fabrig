//! Pure findings from the record and the ctl reads (redeploy.md, 6.2–6.3).
use crate::{
    description::Description,
    record::{Before, Record, TaskBefore},
};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Readings {
    pub snapshot: Before,
    /// Each held task's printable reason, including `failures CLASS`.
    pub holds: BTreeMap<u64, String>,
    pub endings: Vec<FailedEnd>,
    /// Each entry is `WHAT: WHY`, with the ctl call as WHAT.
    pub unread: Vec<String>,
}
#[derive(Debug)]
pub struct FailedEnd {
    pub task: u64,
    pub attempt: u64,
    pub at: u64,
    pub class: String,
}
#[derive(Debug, Default)]
pub struct Judgement {
    pub lines: Vec<String>,
    pub failures: usize,
}
impl Judgement {
    fn line(&mut self, mut line: String, failed: bool) {
        // A finding stays one prefixed line even when ctl's stderr spans lines.
        if line.contains(['\r', '\n']) {
            line = line.replace(['\r', '\n'], " ");
        }
        self.lines.push(line);
        self.failures += usize::from(failed);
    }
}

pub(super) fn tasks<'a>(before: &'a Before, after: &'a Before) -> BTreeMap<u64, &'a TaskBefore> {
    before.tasks.iter().chain(&after.tasks).map(|task| (task.task, task)).collect()
}

#[must_use]
pub fn evaluate(description: &Description, record: &Record, read: &Readings) -> Judgement {
    let mut result = Judgement::default();
    openings(description, record, &mut result);
    window(record, &mut result);
    for outage in &record.outages {
        result.line(
            format!("{} was out {} s", outage.what, seconds(u128::from(outage.to.saturating_sub(outage.from)))),
            false,
        );
    }
    let tasks = tasks(&record.before, &read.snapshot);
    for end in read.endings.iter().filter(|end| end.at >= record.began) {
        let name = task_name(tasks.get(&end.task).map_or(0, |task| task.label));
        let own = if end.class == "run" { ", its own" } else { "" };
        result.line(
            format!("{name} {} attempt {} failed: {}{own}", end.task, end.attempt, end.class),
            end.class != "run",
        );
    }
    for task in &read.snapshot.tasks {
        if task.phase != "held"
            || record.before.tasks.iter().any(|prior| prior.task == task.task && prior.phase == "held")
        {
            continue;
        }
        if let Some(reason) = read.holds.get(&task.task) {
            result.line(
                format!("{} {} held: {reason}", task_name(task.label), task.task),
                task.held.as_deref().is_some_and(|class| class != "run"),
            );
        }
    }
    for why in &read.unread {
        result.line(format!("could not read: {why}"), true);
    }
    if record.before.cut || read.snapshot.cut {
        result.line("memberships cut: projects past them not read".into(), false);
    }
    let mut projects = record.before.projects.clone();
    projects.extend(&read.snapshot.projects);
    projects.sort_unstable();
    projects.dedup();
    result
        .line(format!("read: {} projects, {} tasks, since {}", projects.len(), tasks.len(), utc(record.began)), false);
    result.lines.push(if result.failures == 0 { "pass".into() } else { format!("fail: {}", result.failures) });
    result
}
fn task_name(label: u64) -> &'static str {
    if label == 2 { "goal" } else { "task" }
}
fn openings(description: &Description, record: &Record, result: &mut Judgement) {
    for worker in &description.workers {
        if let Some(opening) = record.openings.iter().find(|opening| opening.worker == worker.name) {
            let marker = if opening.refused {
                format!("serve: worker {} refused at its opening: ", worker.name)
            } else {
                format!("serve: worker {} opened: ", worker.name)
            };
            let text = opening.line.split_once(&marker).map_or(opening.line.as_str(), |(_, text)| text);
            let word = if opening.refused { "refused:" } else { "agreed" };
            result.line(format!("worker {} {word} {text}", worker.name), opening.refused);
        } else {
            result.line(format!("worker {} did not open", worker.name), true);
        }
    }
}
fn window(record: &Record, result: &mut Judgement) {
    if let Some(window) = &record.window
        && let Some(grace) = record.graces.iter().min_by_key(|grace| grace.grace_ms)
    {
        let duration = u128::from(window.to.saturating_sub(window.from));
        let grace_ns = u128::from(grace.grace_ms) * 1_000_000;
        let failed = duration >= grace_ns;
        let word = if failed { "past" } else { "within" };
        result.line(
            format!("window {} s {word} {}'s grace {} s", seconds(duration), grace.worker, seconds(grace_ns)),
            failed,
        );
    }
}
fn seconds(nanos: u128) -> String {
    let tenths = (nanos + 50_000_000) / 100_000_000;
    format!("{}.{:01}", tenths / 10, tenths % 10)
}
fn utc(nanos: u64) -> String {
    let seconds = nanos / 1_000_000_000;
    let mut days = seconds / 86_400;
    let mut year = 1970_u64;
    loop {
        let length = if leap(year) { 366 } else { 365 };
        if days < length {
            break;
        }
        days -= length;
        year += 1;
    }
    let mut month = 1;
    for length in [31, if leap(year) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31] {
        if days < length {
            break;
        }
        days -= length;
        month += 1;
    }
    format!(
        "{year:04}-{month:02}-{:02}T{:02}:{:02}:{:02}Z",
        days + 1,
        seconds / 3600 % 24,
        seconds / 60 % 60,
        seconds % 60
    )
}
fn leap(year: u64) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}
