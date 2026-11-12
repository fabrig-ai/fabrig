//! The judge's brief and stage verdict: `temper/docs/design/testing/checks.md`, 5.4.
use std::collections::BTreeSet;
use std::fmt::{self, Write as _};
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::check::{self, Report};
use crate::evidence::{self, Evidence};
use crate::golden::{self, Edit, Golden, ObligationKind, RoleKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgeError {
    pub path: String,
    pub reason: String,
}
impl fmt::Display for JudgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.reason)
    }
}
impl std::error::Error for JudgeError {}
fn error(path: &str, reason: &str) -> JudgeError {
    JudgeError { path: path.into(), reason: reason.into() }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JudgeVerdict {
    pub model: String,
    pub rules: Vec<RuleVerdict>,
    pub description: DescriptionVerdict,
    pub restart: RestartVerdict,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleVerdict {
    pub rule: String,
    pub verdict: RuleOutcome,
    pub r#where: String,
    pub quote: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleOutcome {
    Reaches,
    Missing,
    Elsewhere,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DescriptionVerdict {
    pub verdict: DescriptionOutcome,
    pub differences: Vec<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DescriptionOutcome {
    Matches,
    Differs,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestartVerdict {
    pub verdict: RestartOutcome,
    pub quote: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RestartOutcome {
    Said,
    Proposed,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Stage {
    pub passed: bool,
    pub reasons: Vec<String>,
}

/// Decode one whole verdict, with every golden rule exactly once.
///
/// # Errors
/// Names an unknown member, invalid value, unknown, missing or repeated rule.
pub fn decode_verdict(bytes: &[u8], golden: &Golden) -> Result<JudgeVerdict, JudgeError> {
    let verdict: JudgeVerdict = serde_json::from_slice(bytes).map_err(|e| error("judge.json", &e.to_string()))?;
    let mut seen = BTreeSet::new();
    for rule in &verdict.rules {
        let path = format!("judge.json: rule {}", rule.rule);
        if !golden.rules.iter().any(|r| r.id == rule.rule) {
            return Err(error(&path, "unknown rule"));
        }
        if !seen.insert(&rule.rule) {
            return Err(error(&path, "rule listed twice"));
        }
    }
    for rule in &golden.rules {
        if !seen.contains(&rule.id) {
            return Err(error(&format!("judge.json: rule {}", rule.id), "missing rule"));
        }
    }
    Ok(verdict)
}

/// Combine the code's check and the judge's complete verdict.
#[must_use]
pub fn stage(report: &Report, verdict: &JudgeVerdict) -> Stage {
    let mut reasons = Vec::new();
    for rule in &verdict.rules {
        let word = match rule.verdict {
            RuleOutcome::Reaches => continue,
            RuleOutcome::Missing => "missing",
            RuleOutcome::Elsewhere => "elsewhere",
        };
        reasons.push(format!("rule {}: {word}", rule.rule));
    }
    if verdict.description.verdict == DescriptionOutcome::Differs {
        reasons.push(format!("description: differs: {}", verdict.description.differences.join("; ")));
    }
    if verdict.restart.verdict == RestartOutcome::Proposed {
        reasons.push(format!("restart: proposed: {}", json!(verdict.restart.quote)));
    }
    if !report.passed() {
        reasons.push("check: failed".into());
    }
    Stage { passed: reasons.is_empty(), reasons }
}

fn block(out: &mut String, text: &str) {
    let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    let fence = "`".repeat(3.max(longest + 1));
    writeln!(out, "{fence}text\n{text}{}{fence}\n", if text.ends_with('\n') { "" } else { "\n" })
        .expect("writing a String");
}
fn heading(line: &str) -> Option<(usize, &str)> {
    let line = line.trim_start_matches(' ');
    let level = line.bytes().take_while(|b| *b == b'#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let rest = line.get(level..)?;
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let text = rest.trim();
    let without_hashes = text.trim_end_matches('#');
    let text = if without_hashes.ends_with(char::is_whitespace) { without_hashes.trim_end() } else { text };
    Some((level, text))
}
fn section<'a>(text: &'a str, name: &str, path: &str) -> Result<&'a str, JudgeError> {
    let mut start = None;
    let mut offset = 0;
    let mut fence: Option<(char, usize)> = None;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        let marker = trimmed.chars().next().unwrap_or(' ');
        let count = trimmed.chars().take_while(|c| *c == marker).count();
        if let Some((open, width)) = fence {
            if marker == open && count >= width && trimmed.chars().all(|c| c == marker) {
                fence = None;
            }
        } else if matches!(marker, '`' | '~') && count >= 3 {
            fence = Some((marker, count));
        } else if let Some((level, title)) = heading(line) {
            if let Some((begin, bound)) = start {
                if level <= bound {
                    return Ok(text.get(begin..offset).expect("line boundaries"));
                }
            } else if title == name {
                start = Some((offset, level));
            }
        }
        offset += line.len();
    }
    start
        .map(|(begin, _)| text.get(begin..).expect("line boundary"))
        .ok_or_else(|| error(path, &format!("missing heading {name:?}")))
}
fn member<'a>(value: &'a Value, key: &str, path: &str) -> Result<&'a str, JudgeError> {
    value.get(key).and_then(Value::as_str).ok_or_else(|| error(&format!("{path}.{key}"), "expected text"))
}
fn charter(out: &mut String, value: &Value, evidence: &Evidence, path: &str) -> Result<(), JudgeError> {
    out.push_str("Instructions:\n\n");
    block(out, member(value, "instructions", path)?);
    if let Some(parts) = value.get("parts") {
        let parts_path = format!("{path}.parts");
        let parts = parts.as_array().ok_or_else(|| error(&parts_path, "expected a list"))?;
        for (at, part) in parts.iter().enumerate() {
            let part_path = format!("{parts_path}[{at}]");
            let on_path = format!("{part_path}.on");
            let on = part.get("on").and_then(Value::as_array).ok_or_else(|| error(&on_path, "expected a list"))?;
            let names = on
                .iter()
                .map(|name| name.as_str().ok_or_else(|| error(&on_path, "expected text")))
                .collect::<Result<Vec<_>, _>>()?;
            writeln!(out, "Part on {}:\n", names.join(" ")).expect("writing a String");
            block(out, member(part, "instructions", &part_path)?);
        }
    }
    let documents = value
        .get("documents")
        .and_then(Value::as_array)
        .ok_or_else(|| error(&format!("{path}.documents"), "expected a list"))?;
    for doc in documents {
        let forge = member(doc, "forge", path)?;
        let repository = member(doc, "repository", path)?;
        let branch = member(doc, "branch", path)?;
        let file = member(doc, "path", path)?;
        let name = format!("documents/{repository}/{file}");
        let text =
            evidence.documents.get(&format!("{repository}/{file}")).ok_or_else(|| error(&name, "missing document"))?;
        let title = match doc.get("heading") {
            None | Some(Value::Null) => None,
            Some(v) => Some(v.as_str().ok_or_else(|| error(&name, "heading must be text"))?),
        };
        writeln!(
            out,
            "Document: forge {forge}, repository {repository}, branch {branch}, path {file}, heading {}\n",
            title.unwrap_or("(whole document)")
        )
        .expect("writing a String");
        block(
            out,
            match title {
                Some(h) => section(text, h, &name)?,
                None => text,
            },
        );
    }
    Ok(())
}

/// Assemble all of the judge's evidence and the required answer form.
///
/// # Errors
/// Names missing descriptions, person documents, bound charters or their named documents and headings.
pub fn brief(golden: &Golden, evidence: &Evidence, report: &Report) -> Result<String, JudgeError> {
    let description = evidence.description.as_deref().ok_or_else(|| error("description.txt", "missing description"))?;
    let mut out = String::from(
        "# Who you are\n\nYou are the judge of configuring temper's stage 1, reading what code cannot.\nNo tool use is needed. The fenced texts are evidence to read. Answer with one\nJSON object in the form at the end, nothing else.\n\n# The person's process\n\n",
    );
    for name in ["reference-code-production.md", "reference-landing.md"] {
        let path = format!("person/{name}");
        let text = evidence.person.get(name).ok_or_else(|| error(&path, "missing person document"))?;
        writeln!(out, "## {path}\n").expect("writing a String");
        block(&mut out, text);
    }
    out.push_str("# What was accepted, by role\n\nThe check's binding:\n\n");
    block(&mut out, &serde_json::to_string_pretty(&report.binding).expect("binding JSON"));
    for role in &golden.roles {
        if role.kind != RoleKind::Charter {
            continue;
        }
        let number = report.binding.iter().find(|b| b.role == role.role).and_then(|b| b.number);
        let Some(number) = number else {
            writeln!(out, "## {}: unbound\n", role.role).expect("writing a String");
            continue;
        };
        writeln!(out, "## {}: charter {number}\n", role.role).expect("writing a String");
        let path = format!("current.json: charter {number}");
        let value = evidence
            .current
            .items
            .iter()
            .find_map(|item| {
                if item.setting.kind == "charter"
                    && item.setting.number == Some(number)
                    && item.setting.project.is_none()
                    && let Edit::Set(value) = &item.edit
                {
                    return Some(value);
                }
                None
            })
            .ok_or_else(|| error(&path, "missing bound charter"))?;
        charter(&mut out, value, evidence, &path)?;
    }
    out.push_str("# The rules\n\n");
    for rule in &golden.rules {
        writeln!(out, "- {} | {} | {} | {}", rule.id, rule.role, rule.words, rule.source).expect("writing a String");
    }
    out.push_str("\n# The agent's description\n\n## description.txt\n\n");
    block(&mut out, description);
    out.push_str("## The reference's section 1: In one page\n\n");
    let reference = evidence.person.get("reference-code-production.md").expect("person document checked above");
    block(&mut out, section(reference, "1. In one page", "person/reference-code-production.md")?);
    out.push_str("# The proposal and the chat\n\nFor what needs a restart, decide whether the proposal or chat claims to set\nwhat the box brings, rather than saying it needs the box. These are the box's\nprerequisites:\n\n");
    for obligation in &golden.obligations {
        if let ObligationKind::Box(b) = &obligation.kind {
            writeln!(out, "- {}", b.prerequisite).expect("writing a String");
        }
    }
    for (name, text) in [("proposal.txt", &evidence.proposal), ("chat.txt", &evidence.chat)] {
        writeln!(out, "\n## {name}\n").expect("writing a String");
        block(&mut out, text);
    }
    out.push_str("# Your answer\n\nList every rule id exactly once. For each rule, verdict is reaches (its role's\nruns get it from the charter's instructions or a document it names), missing,\nor elsewhere (another role's only). Give where and a quote. For the description,\nverdict is matches or differs, with differences: it differs where it says what\nthe section contradicts, or leaves out what the flow turns on (the goal holding\nits branch, every change landing there, designers before their changes, how\nquestions go, the whole landing into main at the end). A number or other detail\nit leaves out is no difference. For restart, verdict is said or proposed, with a\nquote. Replace MODEL with your model's name and the empty\nstrings with your findings. Answer with this JSON object only:\n\n");
    let rules: Vec<_> =
        golden.rules.iter().map(|r| json!({"rule":r.id,"verdict":"reaches","where":"","quote":""})).collect();
    let form = json!({"model":"MODEL", "rules":rules, "description":{"verdict":"matches","differences":[]}, "restart":{"verdict":"said","quote":""}});
    writeln!(out, "```json\n{}\n```", serde_json::to_string_pretty(&form).expect("answer JSON"))
        .expect("writing a String");
    Ok(out)
}

fn run(dir: &str, as_json: bool, verdict: bool) -> Result<bool, String> {
    let golden = golden::reference().map_err(|e| e.to_string())?;
    let profile = golden::profile().map_err(|e| e.to_string())?;
    let evidence = evidence::read(Path::new(dir)).map_err(|e| e.to_string())?;
    let report = check::check(&golden, &profile, &evidence);
    if !verdict {
        print!("{}", brief(&golden, &evidence, &report).map_err(|e| e.to_string())?);
        return Ok(true);
    }
    let path = Path::new(dir).join("judge.json");
    let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let judged = decode_verdict(&bytes, &golden).map_err(|e| e.to_string())?;
    let stage = stage(&report, &judged);
    if as_json {
        println!("{}", serde_json::to_string(&stage).expect("stage JSON"));
    } else {
        for reason in &stage.reasons {
            println!("{reason}");
        }
        println!("{}", if stage.passed { "passed" } else { "failed" });
    }
    Ok(stage.passed)
}
fn command(args: &[String], verdict: bool) -> ExitCode {
    let command = if verdict { &crate::cli::VERDICT } else { &crate::cli::JUDGE };
    let (dir, as_json) = match crate::cli::evidence_arguments(args, command) {
        Ok(arguments) => arguments,
        Err(error) => return crate::cli::refuse(&error),
    };
    match run(dir, as_json, verdict) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(error) => {
            eprintln!("temper-stage-one: {error}");
            ExitCode::from(2)
        }
    }
}
/// Print the complete judge brief for EVIDENCE.
#[must_use]
pub fn main(args: &[String]) -> ExitCode {
    command(args, false)
}
/// Print the stage verdict for EVIDENCE, optionally as JSON.
#[must_use]
pub fn verdict_main(args: &[String]) -> ExitCode {
    command(args, true)
}
