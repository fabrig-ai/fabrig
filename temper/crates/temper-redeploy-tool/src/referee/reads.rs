//! The ctl documents as their renderers write them (redeploy.md, 6.1).
use super::judgement::{FailedEnd, Readings};
use crate::{
    description::Description,
    hosts::{self, Hosts},
    record::TaskBefore,
};
use serde_json::Value;
use std::collections::BTreeSet;

fn document(description: &Description, hosts: &mut dyn Hosts, words: &[&str]) -> Result<Value, String> {
    let text = hosts::ctl(description, words, hosts)?;
    serde_json::from_str(&text).map_err(|why| format!("{}: JSON: {why}", words.join(" ")))
}
fn number(value: &Value, path: &str) -> Result<u64, String> {
    value.pointer(path).and_then(Value::as_u64).ok_or_else(|| format!("{path}: expected an unsigned integer"))
}
fn string<'a>(value: &'a Value, path: &str) -> Result<&'a str, String> {
    value.pointer(path).and_then(Value::as_str).ok_or_else(|| format!("{path}: expected a string"))
}
fn array<'a>(value: &'a Value, path: &str) -> Result<&'a Vec<Value>, String> {
    value.pointer(path).and_then(Value::as_array).ok_or_else(|| format!("{path}: expected rows"))
}
fn optional_string(value: &Value, path: &str) -> Result<Option<String>, String> {
    match value.pointer(path) {
        Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        _ => Err(format!("{path}: expected a string or null")),
    }
}
fn memberships(value: &Value, read: &mut Readings) -> Result<(), String> {
    read.snapshot.cut = value.get("cut").and_then(Value::as_bool).ok_or("/cut: expected a boolean")?;
    for row in array(value, "/projects")? {
        read.snapshot.projects.push(number(row, "/project")?);
    }
    Ok(())
}
pub(super) fn snapshot(description: &Description, hosts: &mut dyn Hosts) -> Readings {
    let mut read = Readings::default();
    if let Err(why) = document(description, hosts, &["me"])
        .and_then(|value| memberships(&value, &mut read).map_err(|why| format!("me: {why}")))
    {
        read.unread.push(why);
    }
    for project in read.snapshot.projects.clone() {
        let mut words = format!("task list --project {project}");
        let mut seen = BTreeSet::new();
        loop {
            if !seen.insert(words.clone()) {
                read.unread.push(format!("{words}: repeated read_on"));
                break;
            }
            let page = document(description, hosts, &words.split_whitespace().collect::<Vec<_>>())
                .and_then(|value| task_page(&value, project).map_err(|why| format!("{words}: {why}")));
            let (tasks, next) = match page {
                Ok(page) => page,
                Err(why) => {
                    read.unread.push(why);
                    break;
                }
            };
            for mut task in tasks {
                if task.phase == "held" {
                    let words = format!("task show {}", task.task);
                    match document(description, hosts, &words.split_whitespace().collect::<Vec<_>>())
                        .and_then(|value| hold(&value).map_err(|why| format!("{words}: {why}")))
                    {
                        Ok((class, reason)) => {
                            task.held = class;
                            read.holds.insert(task.task, reason);
                        }
                        Err(why) => read.unread.push(why),
                    }
                }
                read.snapshot.tasks.push(task);
            }
            match next {
                Some(next) => words = next,
                None => break,
            }
        }
    }
    read
}
fn task_page(value: &Value, project: u64) -> Result<(Vec<TaskBefore>, Option<String>), String> {
    let rows = if value.get("tasks").is_some() { array(value, "/tasks/rows")? } else { array(value, "/live")? };
    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(TaskBefore {
            task: number(row, "/chip/task")?,
            project,
            label: number(row, "/label")?,
            phase: string(row, "/chip/phase")?.into(),
            held: None,
        });
    }
    Ok((tasks, optional_string(value, "/read_on")?))
}
fn hold(value: &Value) -> Result<(Option<String>, String), String> {
    let reason = value.pointer("/task/hold/reason").ok_or("/task/hold/reason: missing")?;
    if reason.get("failures").is_some() {
        let class = string(reason, "/failures/class")?.to_owned();
        Ok((Some(class.clone()), format!("failures {class}")))
    } else if let Some(name) = reason.as_str() {
        Ok((None, name.into()))
    } else if let Some(fields) = reason.as_object().filter(|fields| fields.len() == 1) {
        Ok((None, fields.keys().next().expect("one reason tag").clone()))
    } else {
        Err("/task/hold/reason: expected a reason tag".into())
    }
}

pub(super) fn activity(description: &Description, hosts: &mut dyn Hosts, task: u64, began: u64, read: &mut Readings) {
    let mut words = format!("task activity {task}");
    let mut seen = BTreeSet::new();
    loop {
        if !seen.insert(words.clone()) {
            read.unread.push(format!("{words}: repeated before cursor"));
            break;
        }
        let page = document(description, hosts, &words.split_whitespace().collect::<Vec<_>>())
            .and_then(|value| activity_page(&value, task, began, read).map_err(|why| format!("{words}: {why}")));
        match page {
            Ok(Some(cursor)) => words = format!("task activity {task} --before {cursor}"),
            Ok(None) => break,
            Err(why) => {
                read.unread.push(why);
                break;
            }
        }
    }
}
fn activity_page(value: &Value, task: u64, began: u64, read: &mut Readings) -> Result<Option<String>, String> {
    let mut older = false;
    // Pages are read newest first, but rows within a page are chronological.
    for row in array(value, "/rows")? {
        let at = number(row, "/at")?;
        if at < began {
            older = true;
            continue;
        }
        if let Some(end) = row.pointer("/event/owner/source/line/run_ended")
            && end.pointer("/end/failed").is_some()
        {
            read.endings.push(FailedEnd {
                task,
                at,
                attempt: number(end, "/attempt")?,
                class: string(end, "/end/failed/class")?.into(),
            });
        }
    }
    if older {
        return Ok(None);
    }
    match value.get("before") {
        Some(Value::Null) => Ok(None),
        Some(cursor) => Ok(Some(string(cursor, "/value")?.into())),
        None => Err("/before: missing".into()),
    }
}
