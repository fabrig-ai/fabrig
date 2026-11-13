//! The deployment's checked description (temper/docs/design/redeploy.md, 2).
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Description {
    pub source: String,
    pub engine: Engine,
    pub workers: Vec<Worker>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Engine {
    pub run: Vec<String>,
    pub binaries: String,
    pub builds: String,
    pub stop: String,
    pub start: String,
    pub log: String,
    pub check: String,
    pub backup: String,
    pub swap: String,
    pub ctl: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Worker {
    pub name: String,
    pub run: Vec<String>,
    pub binaries: String,
    pub builds: String,
    pub stop: String,
    pub start: String,
    pub swap: String,
    pub document: String,
}

pub fn read(text: &str) -> Result<Description, String> {
    let value: Value = serde_json::from_str(text).map_err(|why| format!("description: {why}"))?;
    let root = object(&value, "description", &["source", "engine", "workers"])?;
    string(root, "source", "source")?;
    part(&root["engine"], "engine", true)?;
    let workers =
        root["workers"].as_array().filter(|workers| !workers.is_empty()).ok_or("workers: expected a nonempty list")?;
    let mut names = BTreeSet::new();
    for (index, worker) in workers.iter().enumerate() {
        let path = format!("workers[{index}]");
        let fields = part(worker, &path, false)?;
        let name = string(fields, "name", &format!("{path}.name"))?;
        if !names.insert(name) {
            return Err(format!("{path}.name: duplicate worker {name}"));
        }
    }
    serde_json::from_value(value).map_err(|why| format!("description: {why}"))
}

fn object<'a>(value: &'a Value, path: &str, keys: &[&str]) -> Result<&'a Map<String, Value>, String> {
    let fields = value.as_object().ok_or_else(|| format!("{path}: expected an object"))?;
    for key in fields.keys() {
        if !keys.contains(&key.as_str()) {
            return Err(format!("{path}.{key}: unknown member"));
        }
    }
    for key in keys {
        if !fields.contains_key(*key) {
            return Err(format!("{path}.{key}: missing member"));
        }
    }
    Ok(fields)
}
fn string<'a>(fields: &'a Map<String, Value>, key: &str, path: &str) -> Result<&'a str, String> {
    fields[key]
        .as_str()
        .filter(|text| !text.is_empty() && !text.contains('\0'))
        .ok_or_else(|| format!("{path}: expected a nonempty string without NUL"))
}
fn part<'a>(value: &'a Value, path: &str, engine: bool) -> Result<&'a Map<String, Value>, String> {
    let keys: &[&str] = if engine {
        &["run", "binaries", "builds", "stop", "start", "log", "check", "backup", "swap", "ctl"]
    } else {
        &["name", "run", "binaries", "builds", "stop", "start", "swap", "document"]
    };
    let fields = object(value, path, keys)?;
    for key in keys {
        let member = format!("{path}.{key}");
        if *key == "run" {
            let prefix = fields[*key]
                .as_array()
                .filter(|prefix| !prefix.is_empty())
                .ok_or_else(|| format!("{member}: expected a nonempty list"))?;
            for (index, word) in prefix.iter().enumerate() {
                if !word.as_str().is_some_and(|word| !word.is_empty() && !word.contains('\0')) {
                    return Err(format!("{member}[{index}]: expected a nonempty string without NUL"));
                }
            }
        } else {
            let text = string(fields, key, &member)?;
            let required: &[&str] = match *key {
                "log" => &["since"],
                "check" => &["binary"],
                "backup" => &["name"],
                "swap" => &["from", "to"],
                _ => &[],
            };
            if !["name", "binaries", "builds"].contains(key) {
                template(text, required).map_err(|why| format!("{member}: {why}"))?;
            }
        }
    }
    Ok(fields)
}
fn template(mut text: &str, required: &[&str]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    while let Some(start) = text.find(['{', '}']) {
        text = &text[start..];
        if !text.starts_with('{') {
            return Err("unmatched }".into());
        }
        let (name, rest) = text[1..].split_once('}').ok_or("unclosed placeholder")?;
        if !required.contains(&name) {
            return Err(format!("placeholder {{{name}}} is not given"));
        }
        seen.insert(name);
        text = rest;
    }
    for name in required {
        if !seen.contains(name) {
            return Err(format!("placeholder {{{name}}} is required"));
        }
    }
    Ok(())
}
