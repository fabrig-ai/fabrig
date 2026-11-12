//! Collect evidence through ctl reads: `temper/docs/design/testing/checks.md`, 5.3.
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use serde_json::{Value, json};

use crate::golden::{self, Edit, Item, ProjectRef, SettingKey, SettingsFile};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectArgs {
    pub temper: PathBuf,
    pub ctl: PathBuf,
    pub project: u32,
    pub revision: u32,
    pub proposal: String,
    pub chat: u32,
    pub forge: String,
    pub repository: String,
    pub evidence: PathBuf,
}

/// Captured stdout, keyed by ctl's words (including `--json`, excluding `ctl DIR`).
#[derive(Clone, Debug, Default)]
pub struct CtlOutputs {
    pub project: u32,
    pub revision: u32,
    pub by_command: BTreeMap<Vec<String>, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectError {
    pub command: String,
    pub reason: String,
}
impl fmt::Display for CollectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.command, self.reason)
    }
}
impl std::error::Error for CollectError {}
fn failure(command: &str, reason: impl fmt::Display) -> CollectError {
    CollectError { command: command.into(), reason: reason.to_string() }
}
fn words(line: &str) -> Vec<String> {
    line.split_whitespace().map(str::to_owned).collect()
}
fn revision_call(revision: u32) -> Vec<String> {
    words(&format!("revision show {revision} --json"))
}
fn project_call(project: u32) -> Vec<String> {
    words(&format!("project show {project} --json"))
}
fn setting_call(key: &SettingKey, revision: Option<u32>) -> Vec<String> {
    if let Some(ProjectRef::Number(project)) = key.project
        && key.kind == "project-policy"
    {
        return project_call(project);
    }
    let mut call = words(&format!("settings show {}", key.kind));
    if let Some(number) = key.number {
        call.push(number.to_string());
    }
    if let Some(ProjectRef::Number(project)) = key.project {
        if key.number.is_some() {
            call.push("--project".into());
        }
        call.push(project.to_string());
    }
    if let Some(revision) = revision {
        call.extend(["--revision".into(), revision.to_string()]);
    }
    call.push("--json".into());
    call
}
fn output<'a>(outputs: &'a CtlOutputs, call: &[String]) -> Result<&'a str, CollectError> {
    outputs.by_command.get(call).map(String::as_str).ok_or_else(|| failure(&call.join(" "), "missing ctl output"))
}
fn document(outputs: &CtlOutputs, call: &[String]) -> Result<Value, CollectError> {
    serde_json::from_str(output(outputs, call)?).map_err(|e| failure(&call.join(" "), e))
}
fn member<'a>(value: &'a Value, name: &str, context: &str) -> Result<&'a Value, CollectError> {
    value.get(name).ok_or_else(|| failure(context, format!("missing {name}")))
}
fn rows<'a>(value: &'a Value, name: &str, context: &str) -> Result<&'a [Value], CollectError> {
    member(value, name, context)?
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| failure(context, format!("{name} is not an array")))
}
fn key(row: &Value, context: &str) -> Result<SettingKey, CollectError> {
    let value = member(row, "setting", context)?;
    let file = json!({"items": [{"setting": value, "edit": "reset"}]});
    let decoded = golden::decode_settings(file.to_string().as_bytes()).map_err(|e| failure(context, e))?;
    Ok(decoded.items.into_iter().next().expect("one decoded item").setting)
}
fn revision_rows(outputs: &CtlOutputs) -> Result<Vec<Value>, CollectError> {
    let call = revision_call(outputs.revision);
    let doc = document(outputs, &call)?;
    if doc.pointer("/head/number").and_then(Value::as_u64) != Some(u64::from(outputs.revision)) {
        return Err(failure(&call.join(" "), "requested revision is unavailable or differs"));
    }
    Ok(rows(&doc, "items", &call.join(" "))?.to_vec())
}
fn revision_edit(row: &Value, context: &str) -> Result<Option<Edit>, CollectError> {
    if let Some(value) = row.get("set") {
        return Ok(Some(Edit::Set(value.clone())));
    }
    let edit = row.get("edit").or_else(|| row.get("application").filter(|v| v.is_string()));
    match edit.and_then(Value::as_str) {
        Some("removed") => Ok(Some(Edit::Removed)),
        Some("reset") => Ok(Some(Edit::Reset)),
        Some(_) => Err(failure(context, "unknown revision edit")),
        None if row.get("too_large").is_some() || row.pointer("/application/set").is_some() => Ok(None),
        None => Err(failure(context, "missing revision edit")),
    }
}
fn revision_reads(outputs: &CtlOutputs) -> Result<Vec<Vec<String>>, CollectError> {
    let mut calls = Vec::new();
    for row in revision_rows(outputs)? {
        let key = key(&row, "revision items")?;
        if revision_edit(&row, "revision items")?.is_none() {
            let revision = (key.kind == "charter").then_some(outputs.revision);
            calls.push(setting_call(&key, revision));
        }
    }
    Ok(calls)
}
fn inventory(outputs: &CtlOutputs) -> Result<Vec<SettingKey>, CollectError> {
    let call = words("settings show --json");
    let doc = document(outputs, &call)?;
    let mut keys = Vec::new();
    let mut seen = BTreeSet::new();
    for row in rows(&doc, "settings", "settings show --json")? {
        let key = key(row, "settings show --json")?;
        if !seen.insert(key.clone()) {
            return Err(failure("settings show --json", "duplicate setting"));
        }
        keys.push(key);
    }
    let policy =
        SettingKey { kind: "project-policy".into(), number: None, project: Some(ProjectRef::Number(outputs.project)) };
    if seen.insert(policy.clone()) {
        keys.push(policy);
    }
    Ok(keys)
}
fn project_policy(outputs: &CtlOutputs, project: u32) -> Result<Value, CollectError> {
    let call = project_call(project);
    let context = call.join(" ");
    let doc = document(outputs, &call)?;
    if doc.get("number").and_then(Value::as_u64) != Some(u64::from(project)) {
        return Err(failure(&context, "project number differs"));
    }
    let policy = member(&doc, "policy", &context)?;
    let mut roles = Vec::new();
    for role in rows(policy, "roles", &context)? {
        let authority = member(member(role, "authority", &context)?, "authority", &context)?;
        let mut mapped = role.as_object().ok_or_else(|| failure(&context, "role is not an object"))?.clone();
        mapped.insert("authority".into(), authority.clone());
        roles.push(Value::Object(mapped));
    }
    Ok(json!({"period_spend": member(policy, "period_spend", &context)?, "roles": roles}))
}
fn read_edit(outputs: &CtlOutputs, key: &SettingKey, revision: Option<u32>) -> Result<Edit, CollectError> {
    if key.kind == "project-policy"
        && let Some(ProjectRef::Number(project)) = key.project
    {
        return Ok(Edit::Set(project_policy(outputs, project)?));
    }
    let call = setting_call(key, revision);
    let context = call.join(" ");
    let doc = document(outputs, &call)?;
    if self::key(&doc, &context)? != *key {
        return Err(failure(&context, "returned setting differs"));
    }
    let value = member(&doc, "value", &context)?;
    if value.is_null() {
        return match doc.pointer("/application/follows").and_then(Value::as_str) {
            Some("removed") => Ok(Edit::Removed),
            Some("default") => Ok(Edit::Reset),
            _ => Err(failure(&context, "setting has no whole value")),
        };
    }
    Ok(Edit::Set(value.clone()))
}

/// Assemble revision items and every listed current setting, keeping JSON values whole.
///
/// # Errors
/// Names missing or malformed ctl output and settings that cannot be read.
pub fn assemble(outputs: &CtlOutputs) -> Result<(SettingsFile, SettingsFile), CollectError> {
    let mut items = Vec::new();
    let mut seen = BTreeSet::new();
    for row in revision_rows(outputs)? {
        let setting = key(&row, "revision items")?;
        if !seen.insert(setting.clone()) {
            return Err(failure("revision items", "duplicate setting"));
        }
        let edit = match revision_edit(&row, "revision items")? {
            Some(edit) => edit,
            None => read_edit(outputs, &setting, (setting.kind == "charter").then_some(outputs.revision))?,
        };
        items.push(Item { setting, edit });
    }
    let mut current = Vec::new();
    for setting in inventory(outputs)? {
        let whole_policy = items.iter().find(|item| item.setting == setting && setting.kind == "project-policy");
        let edit = match whole_policy {
            Some(Item { edit: Edit::Set(value), .. }) => Edit::Set(value.clone()),
            Some(Item { edit: Edit::Removed | Edit::Reset, .. }) | None => read_edit(outputs, &setting, None)?,
        };
        current.push(Item { setting, edit });
    }
    Ok((SettingsFile { items }, SettingsFile { items: current }))
}
fn settings_json(file: &SettingsFile) -> Value {
    let mut items = Vec::new();
    for item in &file.items {
        items.push(match &item.edit {
            Edit::Set(value) => json!({"setting": item.setting, "set": value}),
            Edit::Removed => json!({"setting": item.setting, "edit": "removed"}),
            Edit::Reset => json!({"setting": item.setting, "edit": "reset"}),
        });
    }
    json!({"items": items})
}
fn run(args: &CollectArgs, call: &[String]) -> Result<String, CollectError> {
    let context = format!("{} ctl {} {}", args.temper.display(), args.ctl.display(), call.join(" "));
    let result = Command::new(&args.temper)
        .arg("ctl")
        .arg(&args.ctl)
        .args(call)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| failure(&context, e))?;
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(failure(&context, format!("{}: {}", result.status, stderr.lines().next().unwrap_or("no stderr"))));
    }
    String::from_utf8(result.stdout).map_err(|e| failure(&context, e))
}
fn capture(args: &CollectArgs, outputs: &mut CtlOutputs, call: Vec<String>) -> Result<(), CollectError> {
    if let std::collections::btree_map::Entry::Vacant(entry) = outputs.by_command.entry(call) {
        let text = run(args, entry.key())?;
        entry.insert(text);
    }
    Ok(())
}
fn chat(args: &CollectArgs) -> Result<String, CollectError> {
    let mut call = words(&format!("chat show {}", args.chat));
    let mut seen = BTreeSet::new();
    let mut pages = Vec::new();
    loop {
        if !seen.insert(call.clone()) {
            return Err(failure(&call.join(" "), "repeated chat cursor"));
        }
        let page = run(args, &call)?;
        let last = page.lines().last().unwrap_or("");
        let prefix = format!("read on with: chat show {} --before ", args.chat);
        let next = last.strip_prefix(&prefix);
        if let Some(cursor) = next {
            if cursor.is_empty() || cursor.split_whitespace().count() != 1 {
                return Err(failure(&call.join(" "), "invalid chat cursor"));
            }
            call = words(&format!("chat show {} --before {cursor}", args.chat));
        }
        let end = if next.is_some() || last == "no more" {
            page.trim_end_matches('\n').len() - last.len()
        } else {
            page.len()
        };
        pages.push(page[..end].to_owned());
        if next.is_none() {
            break;
        }
    }
    pages.reverse();
    Ok(pages.concat())
}
const FILES: [&str; 6] = ["box.json", "revision.json", "items.json", "current.json", "proposal.txt", "chat.txt"];
fn prepare(dir: &Path) -> Result<(), CollectError> {
    fs::create_dir_all(dir).map_err(|e| failure(&dir.display().to_string(), e))?;
    for name in FILES.into_iter().chain(["documents", "person"]) {
        let path = dir.join(name);
        match fs::symlink_metadata(&path) {
            Ok(_) => return Err(failure(&path.display().to_string(), "collect output already exists")),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(failure(&path.display().to_string(), e)),
        }
    }
    Ok(())
}
fn write_file(dir: &Path, name: &str, text: &str) -> Result<(), CollectError> {
    let path = dir.join(name);
    let context = path.display().to_string();
    let mut file = OpenOptions::new().write(true).create_new(true).open(&path).map_err(|e| failure(&context, e))?;
    file.write_all(text.as_bytes()).map_err(|e| failure(&context, e))
}

/// Run ctl's reads and write the collector's evidence, refusing existing outputs.
///
/// # Errors
/// Names the failed command or evidence path, including ctl's first stderr line.
pub fn collect(args: &CollectArgs) -> Result<(), CollectError> {
    prepare(&args.evidence)?;
    let mut outputs = CtlOutputs { project: args.project, revision: args.revision, ..CtlOutputs::default() };
    capture(args, &mut outputs, revision_call(args.revision))?;
    for call in revision_reads(&outputs)? {
        capture(args, &mut outputs, call)?;
    }
    capture(args, &mut outputs, words("settings show --json"))?;
    let revision = revision_rows(&outputs)?;
    for setting in inventory(&outputs)? {
        let mut set_policy = false;
        for row in &revision {
            if key(row, "revision items")? == setting && setting.kind == "project-policy" && row.get("set").is_some() {
                set_policy = true;
            }
        }
        if !set_policy {
            capture(args, &mut outputs, setting_call(&setting, None))?;
        }
    }
    let (items, current) = assemble(&outputs)?;
    let proposal = run(args, &words(&format!("proposal show {}", args.proposal)))?;
    let chat = chat(args)?;
    let facts = json!({"forge": args.forge, "repository": args.repository, "project": args.project, "revision": args.revision, "proposal": args.proposal, "chat": args.chat});
    write_file(&args.evidence, "revision.json", output(&outputs, &revision_call(args.revision))?)?;
    write_file(&args.evidence, "items.json", &settings_json(&items).to_string())?;
    write_file(&args.evidence, "current.json", &settings_json(&current).to_string())?;
    write_file(&args.evidence, "proposal.txt", &proposal)?;
    write_file(&args.evidence, "chat.txt", &chat)?;
    write_file(&args.evidence, "box.json", &facts.to_string())?;
    for name in ["documents", "person"] {
        let path = args.evidence.join(name);
        fs::create_dir(&path).map_err(|e| failure(&path.display().to_string(), e))?;
    }
    Ok(())
}
impl CollectArgs {
    /// Parse K5's required flags and evidence directory.
    ///
    /// # Errors
    /// Names missing, duplicate, unknown or malformed arguments.
    pub fn parse(args: &[String]) -> Result<Self, CollectError> {
        let mut flags = BTreeMap::new();
        let mut evidence = None;
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            if arg.starts_with("--") {
                if !["--temper", "--ctl", "--project", "--revision", "--proposal", "--chat", "--forge", "--repository"]
                    .contains(&arg.as_str())
                {
                    return Err(failure("collect", format!("unknown flag {arg}")));
                }
                let value = args
                    .next()
                    .filter(|value| !value.starts_with("--") && !value.is_empty())
                    .ok_or_else(|| failure("collect", format!("missing value for {arg}")))?;
                if flags.insert(arg.as_str(), value.clone()).is_some() {
                    return Err(failure("collect", format!("duplicate {arg}")));
                }
            } else if arg.starts_with('-') || evidence.replace(PathBuf::from(arg)).is_some() {
                return Err(failure("collect", "expected one EVIDENCE directory"));
            }
        }
        for flag in ["--temper", "--ctl", "--project", "--revision", "--proposal", "--chat", "--forge", "--repository"]
        {
            if !flags.contains_key(flag) {
                return Err(failure("collect", format!("missing {flag}")));
            }
        }
        Ok(Self {
            temper: PathBuf::from(&flags["--temper"]),
            ctl: PathBuf::from(&flags["--ctl"]),
            project: parse_number(&flags["--project"], "--project")?,
            revision: parse_number(&flags["--revision"], "--revision")?,
            proposal: flags["--proposal"].clone(),
            chat: parse_number(&flags["--chat"], "--chat")?,
            forge: flags["--forge"].clone(),
            repository: flags["--repository"].clone(),
            evidence: evidence.ok_or_else(|| failure("collect", "missing EVIDENCE directory"))?,
        })
    }
}
fn parse_number(text: &str, flag: &str) -> Result<u32, CollectError> {
    text.parse::<u32>()
        .ok()
        .filter(|n| *n != 0)
        .ok_or_else(|| failure("collect", format!("{flag} requires a positive u32")))
}
/// Run the collect command, reporting usage and read failures with exit 2.
#[must_use]
pub fn main(args: &[String]) -> ExitCode {
    let args = match CollectArgs::parse(args) {
        Ok(args) => args,
        Err(error) => return crate::cli::refuse(&crate::TABLE.refusal(&crate::cli::COLLECT, &error.reason)),
    };
    match collect(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("temper-stage-one: {error}");
            ExitCode::from(2)
        }
    }
}
