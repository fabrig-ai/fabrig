//! The check by code: `temper/docs/design/testing/checks.md`, 5.1 and 5.4.
use std::fmt::Write as _;
use std::path::Path;
use std::process::ExitCode;

use serde::Serialize;
use serde_json::{Value, json};

use crate::evidence::{self, Evidence};
use crate::golden::{
    self, Anchor, BranchAnchor, Edit, Golden, Obligation, ObligationKind, ProjectRef, ResourceAnchor, Role, SettingKey,
    SettingObligation, SettingsFile, Subject, Test,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    pub binding: Vec<Binding>,
    pub verdicts: Vec<Verdict>,
    pub extras: Vec<Extra>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Binding {
    pub role: String,
    pub number: Option<u32>,
    pub by: BoundBy,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BoundBy {
    Anchor,
    Assignment,
    Unbound,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Verdict {
    pub id: String,
    pub row: String,
    pub outcome: Outcome,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Outcome {
    Met,
    Wrong { found: Value, expected: Value },
    Missing,
    Judged,
    Unchanged,
    Named { text: TextName },
    Unnamed,
    Confirmed,
    Lacking { seen: String },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextName {
    Proposal,
    Chat,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Extra {
    pub setting: SettingKey,
    pub why: String,
}

impl Outcome {
    fn passed(&self) -> bool {
        match self {
            Self::Met | Self::Judged | Self::Unchanged | Self::Named { .. } | Self::Confirmed => true,
            Self::Wrong { .. } | Self::Missing | Self::Unnamed | Self::Lacking { .. } => false,
        }
    }
    fn text(&self) -> String {
        match self {
            Self::Met => "met".into(),
            Self::Wrong { found, expected } => format!("wrong: found {found}, expected {expected}"),
            Self::Missing => "missing".into(),
            Self::Judged => "judged".into(),
            Self::Unchanged => "unchanged".into(),
            Self::Named { text } => format!(
                "named: {}",
                match text {
                    TextName::Proposal => "proposal.txt",
                    TextName::Chat => "chat.txt",
                }
            ),
            Self::Unnamed => "unnamed".into(),
            Self::Confirmed => "confirmed".into(),
            Self::Lacking { seen } => format!("lacking: {seen}"),
        }
    }
}
impl Report {
    #[must_use]
    pub fn passed(&self) -> bool {
        self.extras.is_empty() && self.verdicts.iter().all(|v| v.outcome.passed())
    }
    #[must_use]
    pub fn text(&self) -> String {
        let mut out = String::new();
        for binding in &self.binding {
            let kind = if binding.role.ends_with("template") {
                "template"
            } else if binding.role.ends_with("rule") {
                "landing-rule"
            } else {
                "charter"
            };
            let detail = match (binding.number, binding.by) {
                (Some(n), BoundBy::Anchor) => format!("{kind} {n}, by its anchor"),
                (Some(n), BoundBy::Assignment) => format!("{kind} {n}, by assignment"),
                (None, _) | (Some(_), BoundBy::Unbound) => "unbound".into(),
            };
            writeln!(out, "{}: {detail}", binding.role).expect("writing a String");
        }
        for verdict in &self.verdicts {
            writeln!(out, "{}  {}", verdict.id, verdict.outcome.text()).expect("writing a String");
        }
        for extra in &self.extras {
            writeln!(out, "extra: {}: {}", key_text(&extra.setting), extra.why).expect("writing a String");
        }
        if self.passed() {
            writeln!(out, "passed").expect("writing a String");
        } else {
            let failed = self.verdicts.iter().filter(|v| !v.outcome.passed()).count() + self.extras.len();
            writeln!(out, "failed: {failed} of {}", self.verdicts.len() + self.extras.len()).expect("writing a String");
        }
        out
    }
    #[must_use]
    pub fn json(&self) -> Value {
        json!({"binding": self.binding, "verdicts": self.verdicts, "extras": self.extras, "passed": self.passed()})
    }
}
fn key_text(key: &SettingKey) -> String {
    let mut out = key.kind.clone();
    if let Some(project) = &key.project {
        match project {
            ProjectRef::Box => out.push_str(" project box"),
            ProjectRef::Number(n) => write!(out, " project {n}").expect("writing a String"),
        }
    }
    if let Some(n) = key.number {
        write!(out, " {n}").expect("writing a String");
    }
    out
}
fn key(kind: &str, number: u32) -> SettingKey {
    SettingKey { kind: kind.into(), number: Some(number), project: None }
}
fn value<'a>(file: &'a SettingsFile, key: &SettingKey) -> Option<&'a Value> {
    match &file.items.iter().find(|item| item.setting == *key)?.edit {
        Edit::Set(value) => Some(value),
        Edit::Removed | Edit::Reset => None,
    }
}
fn role_index(golden: &Golden, name: &str) -> Option<usize> {
    golden.roles.iter().position(|role| role.role == name)
}
fn subject_key(
    subject: &Subject,
    golden: &Golden,
    evidence: &Evidence,
    keys: &[Option<SettingKey>],
) -> Option<SettingKey> {
    match subject {
        Subject::Role(name) => keys.get(role_index(golden, name)?)?.clone(),
        Subject::Setting(key) => {
            let mut key = key.clone();
            if key.project == Some(ProjectRef::Box) {
                key.project = Some(ProjectRef::Number(evidence.facts.project));
            }
            if key.kind == "project-ways" && !evidence.items.items.iter().any(|item| item.setting == key) {
                return None;
            }
            Some(key)
        }
    }
}
fn unhex(value: &Value) -> Option<String> {
    let text = value.as_str()?;
    if text.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::new();
    for pair in text.as_bytes().chunks_exact(2) {
        let hi = char::from(pair[0]).to_digit(16)?;
        let lo = char::from(pair[1]).to_digit(16)?;
        bytes.push(u8::try_from(hi * 16 + lo).ok()?);
    }
    String::from_utf8(bytes).ok()
}
fn pattern(value: &Value) -> Option<Value> {
    let mut decoded = value.clone();
    for segment in decoded.get_mut("segments")?.as_array_mut()? {
        *segment = Value::String(unhex(segment)?);
    }
    let last = decoded.get_mut("last")?.as_object_mut()?;
    if last.len() != 1 {
        return None;
    }
    let end = last.get_mut("exact");
    if let Some(end) = end {
        *end = Value::String(unhex(end)?);
    } else {
        let end = last.get_mut("open")?;
        *end = Value::String(unhex(end)?);
    }
    Some(decoded)
}
fn substitute(value: &mut Value, golden: &Golden, evidence: &Evidence, keys: &[Option<SettingKey>]) -> Option<()> {
    match value {
        Value::Object(map) => {
            if map.len() == 1 && map.get("role").is_some_and(Value::is_string) {
                let name = map.get("role")?.as_str()?;
                *value = json!(keys.get(role_index(golden, name)?)?.as_ref()?.number?);
            } else {
                for child in map.values_mut() {
                    substitute(child, golden, evidence, keys)?;
                }
            }
        }
        Value::Array(values) => {
            for child in values {
                substitute(child, golden, evidence, keys)?;
            }
        }
        Value::String(text) => {
            if *text == golden.placeholders.forge {
                text.clone_from(&evidence.facts.forge);
            } else if *text == golden.placeholders.repository {
                text.clone_from(&evidence.facts.repository);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
    Some(())
}
/// A last open on nothing beneath a segment covers, for a branch under that segment, what
/// the segment open covers, so a rule written either way reads the same.
fn folded(mut decoded: Value) -> Option<Value> {
    if decoded.pointer("/last/open").and_then(Value::as_str) == Some("") {
        let segment = decoded.get_mut("segments")?.as_array_mut()?.pop()?;
        *decoded.pointer_mut("/last/open")? = segment;
    }
    Some(decoded)
}
fn expected_pattern(value: &Value, golden: &Golden, evidence: &Evidence) -> Option<Value> {
    let mut decoded = pattern(value)?;
    let segments = decoded.get_mut("segments")?.as_array_mut()?;
    let (owner, repository) = golden.placeholders.repository.split_once('/')?;
    let (box_owner, box_repository) = evidence.facts.repository.split_once('/')?;
    for (i, from, to) in [
        (1, golden.placeholders.forge.as_str(), evidence.facts.forge.as_str()),
        (2, owner, box_owner),
        (3, repository, box_repository),
    ] {
        if let Some(segment) = segments.get_mut(i)
            && segment.as_str() == Some(from)
        {
            *segment = json!(to);
        }
    }
    Some(decoded)
}
fn branch_matches(value: &Value, anchor: &BranchAnchor) -> bool {
    let Some(last) = golden::member(value, "pattern.last") else { return false };
    match anchor {
        BranchAnchor::Exact(text) => last.get("exact").and_then(unhex).as_ref() == Some(text),
        BranchAnchor::Open(text) => {
            let Some(open) = last.get("open").and_then(unhex) else { return false };
            let beneath = || {
                let segments = golden::member(value, "pattern.segments")?.as_array()?;
                segments.last().and_then(unhex)
            };
            open == *text || (open.is_empty() && beneath().as_ref() == Some(text))
        }
    }
}

fn anchor_key(role: &Role, golden: &Golden, evidence: &Evidence, keys: &[Option<SettingKey>]) -> Option<SettingKey> {
    let kind = golden::kind_name(role.kind);
    let found = match role.anchor.as_ref()? {
        Anchor::Number(n) => key(kind, *n),
        Anchor::Executes(name) => {
            let template = keys.get(role_index(golden, name)?)?.as_ref()?;
            let n = golden::member(value(&evidence.current, template)?, "executor.charter")?.as_u64()?;
            key(kind, u32::try_from(n).ok()?)
        }
        Anchor::ForgeCharter(n) => {
            let n = value(&evidence.current, &key("forge-charter", *n))?.as_u64()?;
            key(kind, u32::try_from(n).ok()?)
        }
        Anchor::Resource(resource) => {
            return evidence
                .current
                .items
                .iter()
                .filter(|item| item.setting.kind == kind)
                .filter(|item| {
                    let Some(resources) = value(&evidence.current, &item.setting)
                        .and_then(|v| v.get("resources"))
                        .and_then(Value::as_array)
                    else {
                        return false;
                    };
                    match resource {
                        ResourceAnchor::GoalBranch => {
                            resources.iter().any(|r| r.get("symbol").and_then(Value::as_u64) == Some(1))
                        }
                        ResourceAnchor::None => resources.is_empty(),
                    }
                })
                .min_by_key(|item| item.setting.number)
                .map(|item| item.setting.clone());
        }
        Anchor::Branch(branch) => {
            return evidence
                .current
                .items
                .iter()
                .filter(|item| {
                    item.setting.kind == kind
                        && (item.setting.project.is_none()
                            || item.setting.project == Some(ProjectRef::Number(evidence.facts.project)))
                })
                .filter(|item| value(&evidence.current, &item.setting).is_some_and(|v| branch_matches(v, branch)))
                .min_by_key(|item| (item.setting.number, item.setting.project.clone()))
                .map(|item| item.setting.clone());
        }
    };
    value(&evidence.current, &found).map(|_| found)
}

fn expected(setting: &SettingObligation, golden: &Golden, profile: &SettingsFile) -> Option<Value> {
    if let Some(value) = &setting.value {
        return Some(value.clone());
    }
    let Subject::Role(name) = &setting.subject else { return None };
    let role = golden.roles.get(role_index(golden, name)?)?;
    golden::member(value(profile, &key(golden::kind_name(role.kind), role.profile))?, &setting.member).cloned()
}
fn integer(value: &Value) -> Option<i128> {
    value.as_i64().map(i128::from).or_else(|| value.as_u64().map(i128::from))
}
fn compare(test: Test, found: &Value, expected: &Value) -> bool {
    match test {
        Test::Equal => found == expected,
        Test::AtLeast => match (integer(found), integer(expected)) {
            (Some(f), Some(e)) => f >= e,
            _ => false,
        },
        Test::Bits => match (found.as_u64(), expected.as_u64()) {
            (Some(f), Some(e)) => f & e == e,
            _ => false,
        },
        Test::Lacks => match (found.as_u64(), expected.as_u64()) {
            (Some(f), Some(e)) => f & e == 0,
            _ => false,
        },
        Test::Contains => match (found.as_array(), expected.as_array()) {
            (Some(f), Some(e)) => e.iter().all(|v| f.iter().any(|item| holds(item, v))),
            _ => false,
        },
        Test::Set | Test::Judged | Test::Unchanged | Test::Only => false,
    }
}
/// An element holds an expected one when each member the golden names is equal,
/// a member absent counting as null, as ctl prints it; a member it does not name,
/// a document's heading among them, is free.
fn holds(found: &Value, expected: &Value) -> bool {
    match (found.as_object(), expected.as_object()) {
        (Some(found), Some(expected)) => {
            expected.iter().all(|(name, value)| found.get(name).unwrap_or(&Value::Null) == value)
        }
        _ => found == expected,
    }
}
fn setting_outcome(
    setting: &SettingObligation,
    golden: &Golden,
    profile: &SettingsFile,
    evidence: &Evidence,
    keys: &[Option<SettingKey>],
) -> Outcome {
    let Some(key) = subject_key(&setting.subject, golden, evidence, keys) else { return Outcome::Missing };
    if setting.test == Test::Only {
        let unbound: Vec<_> = evidence
            .current
            .items
            .iter()
            .filter(|item| {
                item.setting.kind == key.kind
                    && matches!(item.edit, Edit::Set(_))
                    && !keys.contains(&Some(item.setting.clone()))
            })
            .map(|item| item.setting.clone())
            .collect();
        return if unbound.is_empty() {
            Outcome::Met
        } else {
            Outcome::Wrong { found: json!(unbound), expected: json!("only bound roles") }
        };
    }
    if setting.test == Test::Unchanged {
        if let Some(item) = evidence.items.items.iter().find(|item| item.setting == key) {
            let found = match &item.edit {
                Edit::Set(v) => json!({"set": v}),
                Edit::Removed => json!({"edit": "removed"}),
                Edit::Reset => json!({"edit": "reset"}),
            };
            return Outcome::Wrong { found, expected: json!("unchanged") };
        }
        return if value(&evidence.current, &key).is_some() { Outcome::Unchanged } else { Outcome::Missing };
    }
    let Some(found) = value(&evidence.current, &key).and_then(|v| golden::member(v, &setting.member)) else {
        return Outcome::Missing;
    };
    match setting.test {
        Test::Set => {
            return if found.is_null() {
                Outcome::Wrong { found: found.clone(), expected: json!("non-null") }
            } else {
                Outcome::Met
            };
        }
        Test::Judged => {
            return if found.as_str().is_some_and(|s| !s.is_empty()) { Outcome::Judged } else { Outcome::Missing };
        }
        Test::Equal | Test::AtLeast | Test::Bits | Test::Lacks | Test::Contains | Test::Unchanged | Test::Only => {}
    }
    let Some(mut expected) = expected(setting, golden, profile) else { return Outcome::Missing };
    if substitute(&mut expected, golden, evidence, keys).is_none() {
        return Outcome::Missing;
    }
    let mut found = found.clone();
    if key.kind == "landing-rule" && setting.member == "pattern" {
        let decoded_found = pattern(&found).and_then(folded);
        let decoded_expected = expected_pattern(&expected, golden, evidence);
        match (decoded_found, decoded_expected) {
            (Some(f), Some(e)) => {
                found = f;
                expected = e;
            }
            _ => return Outcome::Wrong { found, expected },
        }
    }
    if compare(setting.test, &found, &expected) { Outcome::Met } else { Outcome::Wrong { found, expected } }
}
fn outcome(
    obligation: &Obligation,
    golden: &Golden,
    profile: &SettingsFile,
    evidence: &Evidence,
    keys: &[Option<SettingKey>],
) -> Outcome {
    match &obligation.kind {
        ObligationKind::Setting(setting) => setting_outcome(setting, golden, profile, evidence, keys),
        ObligationKind::Box(_) => match evidence.confirmed.iter().find(|c| c.id == obligation.id) {
            Some(c) if c.present => Outcome::Confirmed,
            Some(c) => Outcome::Lacking { seen: c.seen.clone() },
            None => Outcome::Lacking { seen: "not confirmed".into() },
        },
        ObligationKind::Gap(gap) => {
            for (name, text) in [(TextName::Proposal, &evidence.proposal), (TextName::Chat, &evidence.chat)] {
                let text = text.to_lowercase();
                if gap.names.iter().any(|words| words.iter().all(|word| text.contains(&word.to_lowercase()))) {
                    return Outcome::Named { text: name };
                }
            }
            Outcome::Unnamed
        }
    }
}
fn assign(golden: &Golden, profile: &SettingsFile, evidence: &Evidence, keys: &mut Vec<Option<SettingKey>>) {
    let roles: Vec<_> =
        golden.roles.iter().enumerate().filter(|(_, role)| role.anchor.is_none()).map(|(i, _)| i).collect();
    let mut candidates: Vec<_> = evidence
        .items
        .items
        .iter()
        .filter(|item| {
            item.setting.kind == "charter"
                && matches!(item.edit, Edit::Set(_))
                && !keys.contains(&Some(item.setting.clone()))
        })
        .map(|item| item.setting.clone())
        .collect();
    candidates.sort_by_key(|key| key.number);
    let mut best = keys.clone();
    let mut best_score = None;
    let mut choices = vec![0; roles.len()];
    loop {
        let mut used = Vec::new();
        let mut unique = true;
        for (&i, &choice) in roles.iter().zip(&choices) {
            keys[i] = candidates.get(choice).cloned();
            if let Some(key) = &keys[i] {
                if used.contains(key) {
                    unique = false;
                }
                used.push(key.clone());
            }
        }
        if unique {
            let score =
                golden.obligations.iter().filter(|o| outcome(o, golden, profile, evidence, keys).passed()).count();
            if best_score.is_none_or(|best_score| score > best_score) {
                best_score = Some(score);
                best.clone_from(keys);
            }
        }
        let mut advanced = false;
        for choice in choices.iter_mut().rev() {
            if *choice < candidates.len() {
                *choice += 1;
                advanced = true;
                break;
            }
            *choice = 0;
        }
        if !advanced {
            break;
        }
    }
    *keys = best;
}
/// Bind the golden's roles, evaluate its obligations, and find unasked-for settings.
#[must_use]
pub fn check(golden: &Golden, profile: &SettingsFile, evidence: &Evidence) -> Report {
    let mut keys = vec![None; golden.roles.len()];
    for _ in &golden.roles {
        let mut changed = false;
        for (i, role) in golden.roles.iter().enumerate() {
            if keys[i].is_none() {
                keys[i] = anchor_key(role, golden, evidence, &keys);
                changed |= keys[i].is_some();
            }
        }
        if !changed {
            break;
        }
    }
    assign(golden, profile, evidence, &mut keys);
    let binding = golden
        .roles
        .iter()
        .zip(&keys)
        .map(|(role, key)| Binding {
            role: role.role.clone(),
            number: key.as_ref().and_then(|key| key.number),
            by: if key.is_none() {
                BoundBy::Unbound
            } else if role.anchor.is_some() {
                BoundBy::Anchor
            } else {
                BoundBy::Assignment
            },
        })
        .collect();
    let verdicts = golden
        .obligations
        .iter()
        .map(|o| Verdict {
            id: o.id.clone(),
            row: o.row.clone(),
            outcome: outcome(o, golden, profile, evidence, &keys),
        })
        .collect();
    let subjects: Vec<_> = golden
        .obligations
        .iter()
        .filter_map(|o| match &o.kind {
            ObligationKind::Setting(s) => subject_key(&s.subject, golden, evidence, &keys),
            ObligationKind::Box(_) | ObligationKind::Gap(_) => None,
        })
        .collect();
    let extras = evidence
        .items
        .items
        .iter()
        .filter(|item| !keys.contains(&Some(item.setting.clone())) && !subjects.contains(&item.setting))
        .filter(|item| !golden.free.contains(&item.setting.kind))
        .filter(|item| !(item.setting.kind == "landing-rule" && item.edit == Edit::Removed))
        .map(|item| Extra { setting: item.setting.clone(), why: "no obligation asks for it".into() })
        .collect();
    Report { binding, verdicts, extras }
}
/// Read evidence and print the check in text or JSON.
///
/// # Errors
/// Returns a usage error or the golden/evidence file that cannot be read.
pub fn run(args: &[String]) -> Result<bool, String> {
    let (dir, as_json) = crate::cli::evidence_arguments(args, &crate::cli::CHECK)?;
    run_evidence(dir, as_json)
}
fn run_evidence(dir: &str, as_json: bool) -> Result<bool, String> {
    let golden = golden::reference().map_err(|e| e.to_string())?;
    let profile = golden::profile().map_err(|e| e.to_string())?;
    let evidence = evidence::read(Path::new(dir)).map_err(|e| e.to_string())?;
    let report = check(&golden, &profile, &evidence);
    if as_json {
        println!("{}", report.json());
    } else {
        print!("{}", report.text());
    }
    Ok(report.passed())
}
#[must_use]
pub fn main(args: &[String]) -> ExitCode {
    let (dir, as_json) = match crate::cli::evidence_arguments(args, &crate::cli::CHECK) {
        Ok(arguments) => arguments,
        Err(error) => return crate::cli::refuse(&error),
    };
    match run_evidence(dir, as_json) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(error) => {
            eprintln!("temper-stage-one: {error}");
            ExitCode::from(2)
        }
    }
}
