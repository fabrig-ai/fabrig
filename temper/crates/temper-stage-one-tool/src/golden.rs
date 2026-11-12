//! The golden and settings files: `temper/docs/design/testing/checks.md`, 5.
use std::collections::BTreeSet;
use std::fmt;
use std::fmt::Write as _;
use std::process::ExitCode;

use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Golden {
    pub placeholders: Placeholders,
    pub roles: Vec<Role>,
    /// Kinds of setting no obligation asks for, and a proposal may hold; none when left out.
    pub free: Vec<String>,
    pub obligations: Vec<Obligation>,
    pub rules: Vec<Rule>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Placeholders {
    pub forge: String,
    pub repository: String,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Role {
    pub role: String,
    pub kind: RoleKind,
    pub profile: u32,
    pub anchor: Option<Anchor>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RoleKind {
    Charter,
    Template,
    LandingRule,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Anchor {
    Executes(String),
    ForgeCharter(u32),
    Number(u32),
    Resource(ResourceAnchor),
    Branch(BranchAnchor),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResourceAnchor {
    GoalBranch,
    None,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BranchAnchor {
    Exact(String),
    Open(String),
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Obligation {
    pub id: String,
    pub row: String,
    #[serde(flatten)]
    pub kind: ObligationKind,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ObligationKind {
    Setting(SettingObligation),
    Box(BoxObligation),
    Gap(GapObligation),
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SettingObligation {
    pub subject: Subject,
    pub member: String,
    pub test: Test,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Subject {
    Role(String),
    Setting(SettingKey),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Test {
    Equal,
    AtLeast,
    Bits,
    Lacks,
    Contains,
    Set,
    Judged,
    Unchanged,
    Only,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BoxObligation {
    pub prerequisite: String,
    pub confirm: Confirm,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Confirm {
    Ctl,
    Forge,
    Document,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GapObligation {
    pub status: String,
    pub names: Vec<Vec<String>>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Rule {
    pub id: String,
    pub role: String,
    pub words: String,
    pub source: String,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct SettingKey {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectRef>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProjectRef {
    Box,
    Number(u32),
}
impl Serialize for ProjectRef {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Box => serializer.serialize_str("box"),
            Self::Number(n) => serializer.serialize_u32(*n),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct SettingsFile {
    pub items: Vec<Item>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub setting: SettingKey,
    pub edit: Edit,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    Set(Value),
    Removed,
    Reset,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoldenError {
    pub path: String,
    pub reason: String,
}
impl fmt::Display for GoldenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.reason)
    }
}
impl std::error::Error for GoldenError {}

pub(crate) fn error(path: &str, reason: &str) -> GoldenError {
    GoldenError { path: path.into(), reason: reason.into() }
}
pub(crate) fn json(bytes: &[u8]) -> Result<Value, GoldenError> {
    serde_json::from_slice(bytes).map_err(|e| error("$", &e.to_string()))
}
pub(crate) fn object<'a>(v: &'a Value, path: &str, allowed: &[&str]) -> Result<&'a Map<String, Value>, GoldenError> {
    let map = v.as_object().ok_or_else(|| error(path, "expected an object"))?;
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(error(&format!("{path}.{key}"), "unknown member"));
        }
    }
    Ok(map)
}
pub(crate) fn required<'a>(map: &'a Map<String, Value>, key: &str, path: &str) -> Result<&'a Value, GoldenError> {
    map.get(key).ok_or_else(|| error(&format!("{path}.{key}"), "missing member"))
}
pub(crate) fn string(v: &Value, path: &str) -> Result<String, GoldenError> {
    v.as_str().map(str::to_owned).ok_or_else(|| error(path, "expected a string"))
}
pub(crate) fn text(map: &Map<String, Value>, key: &str, path: &str) -> Result<String, GoldenError> {
    string(required(map, key, path)?, &format!("{path}.{key}"))
}
pub(crate) fn number(v: &Value, path: &str) -> Result<u32, GoldenError> {
    v.as_u64().and_then(|n| u32::try_from(n).ok()).ok_or_else(|| error(path, "expected a u32"))
}
pub(crate) fn array<'a>(v: &'a Value, path: &str) -> Result<&'a [Value], GoldenError> {
    v.as_array().map(Vec::as_slice).ok_or_else(|| error(path, "expected an array"))
}
fn field_array<'a>(map: &'a Map<String, Value>, key: &str, path: &str) -> Result<&'a [Value], GoldenError> {
    array(required(map, key, path)?, &format!("{path}.{key}"))
}
fn role_kind(v: &str, path: &str) -> Result<RoleKind, GoldenError> {
    match v {
        "charter" => Ok(RoleKind::Charter),
        "template" => Ok(RoleKind::Template),
        "landing-rule" => Ok(RoleKind::LandingRule),
        _ => Err(error(path, "unknown role kind")),
    }
}
fn anchor(v: &Value, path: &str) -> Result<Option<Anchor>, GoldenError> {
    if v.is_null() {
        return Ok(None);
    }
    let map = object(v, path, &["executes", "forge-charter", "number", "resource", "branch"])?;
    if map.len() != 1 {
        return Err(error(path, "expected one anchor"));
    }
    let (key, v) = map.iter().next().expect("one anchor");
    let at = format!("{path}.{key}");
    Ok(Some(match key.as_str() {
        "executes" => Anchor::Executes(string(v, &at)?),
        "forge-charter" => Anchor::ForgeCharter(number(v, &at)?),
        "number" => Anchor::Number(number(v, &at)?),
        "resource" => Anchor::Resource(match string(v, &at)?.as_str() {
            "goal-branch" => ResourceAnchor::GoalBranch,
            "none" => ResourceAnchor::None,
            _ => return Err(error(&at, "unknown resource anchor")),
        }),
        "branch" => {
            let branch = object(v, &at, &["exact", "open"])?;
            if branch.len() != 1 {
                return Err(error(&at, "expected one branch anchor"));
            }
            if branch.contains_key("exact") {
                Anchor::Branch(BranchAnchor::Exact(text(branch, "exact", &at)?))
            } else {
                Anchor::Branch(BranchAnchor::Open(text(branch, "open", &at)?))
            }
        }
        _ => unreachable!("anchor members checked"),
    }))
}
fn setting_key(v: &Value, path: &str, subject: bool) -> Result<SettingKey, GoldenError> {
    let map = object(v, path, &["kind", "number", "project"])?;
    let kind = text(map, "kind", path)?;
    let setting_number = map.get("number").map(|v| number(v, &format!("{path}.number"))).transpose()?;
    let project = map
        .get("project")
        .map(|v| {
            if v.as_str() == Some("box") {
                Ok(ProjectRef::Box)
            } else {
                number(v, &format!("{path}.project")).map(ProjectRef::Number)
            }
        })
        .transpose()?;
    match kind.as_str() {
        "rules" if setting_number.is_none() && project.is_none() => {}
        "project-policy" if setting_number.is_none() && project.is_some() => {}
        "project-ways" | "project-questions"
            if setting_number.is_none() && project.is_some() && project != Some(ProjectRef::Number(0)) => {}
        "landing-rule" if subject || setting_number.is_some() => {}
        "charter" | "template" | "template-goal" | "policy" | "forge-charter"
            if project.is_none() && (subject || setting_number.is_some()) => {}
        _ => return Err(error(path, "invalid setting key")),
    }
    Ok(SettingKey { kind, number: setting_number, project })
}
fn subject(v: &Value, path: &str) -> Result<Subject, GoldenError> {
    let map = object(v, path, &["role", "setting"])?;
    if map.len() != 1 {
        return Err(error(path, "expected one subject"));
    }
    if map.contains_key("role") {
        Ok(Subject::Role(text(map, "role", path)?))
    } else {
        Ok(Subject::Setting(setting_key(required(map, "setting", path)?, &format!("{path}.setting"), true)?))
    }
}
fn test(v: &str, path: &str) -> Result<Test, GoldenError> {
    match v {
        "equal" => Ok(Test::Equal),
        "at-least" => Ok(Test::AtLeast),
        "bits" => Ok(Test::Bits),
        "lacks" => Ok(Test::Lacks),
        "contains" => Ok(Test::Contains),
        "set" => Ok(Test::Set),
        "judged" => Ok(Test::Judged),
        "unchanged" => Ok(Test::Unchanged),
        "only" => Ok(Test::Only),
        _ => Err(error(path, "unknown setting test")),
    }
}
fn obligation(v: &Value, path: &str) -> Result<Obligation, GoldenError> {
    let kind = v
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| error(&format!("{path}.kind"), "expected an obligation kind"))?;
    let allowed: &[&str] = match kind {
        "setting" => &["id", "row", "kind", "subject", "member", "test", "value"],
        "box" => &["id", "row", "kind", "prerequisite", "confirm"],
        "gap" => &["id", "row", "kind", "status", "names"],
        _ => return Err(error(&format!("{path}.kind"), "unknown obligation kind")),
    };
    let map = object(v, path, allowed)?;
    let id = text(map, "id", path)?;
    let row = text(map, "row", path)?;
    let kind = match kind {
        "setting" => ObligationKind::Setting(SettingObligation {
            subject: subject(required(map, "subject", path)?, &format!("{path}.subject"))?,
            member: text(map, "member", path)?,
            test: test(&text(map, "test", path)?, &format!("{path}.test"))?,
            value: map.get("value").cloned(),
        }),
        "box" => ObligationKind::Box(BoxObligation {
            prerequisite: text(map, "prerequisite", path)?,
            confirm: match text(map, "confirm", path)?.as_str() {
                "ctl" => Confirm::Ctl,
                "forge" => Confirm::Forge,
                "document" => Confirm::Document,
                _ => return Err(error(&format!("{path}.confirm"), "unknown confirmation")),
            },
        }),
        "gap" => {
            let mut names = Vec::new();
            for (i, alternative) in field_array(map, "names", path)?.iter().enumerate() {
                let at = format!("{path}.names[{i}]");
                let mut words = Vec::new();
                for (j, word) in array(alternative, &at)?.iter().enumerate() {
                    words.push(string(word, &format!("{at}[{j}]"))?);
                }
                names.push(words);
            }
            ObligationKind::Gap(GapObligation { status: text(map, "status", path)?, names })
        }
        _ => unreachable!("obligation kind checked"),
    };
    Ok(Obligation { id, row, kind })
}

/// Decode a golden, refusing unknown members and unresolved role or value paths.
///
/// # Errors
/// Returns the JSON path and reason of the first invalid member.
pub fn decode(bytes: &[u8]) -> Result<Golden, GoldenError> {
    let v = json(bytes)?;
    let map = object(&v, "$", &["golden", "placeholders", "roles", "free", "obligations", "rules"])?;
    text(map, "golden", "$")?;
    let placeholders = object(required(map, "placeholders", "$")?, "$.placeholders", &["forge", "repository"])?;
    let placeholders = Placeholders {
        forge: text(placeholders, "forge", "$.placeholders")?,
        repository: text(placeholders, "repository", "$.placeholders")?,
    };
    let mut roles = Vec::new();
    let mut names = BTreeSet::new();
    for (i, v) in field_array(map, "roles", "$")?.iter().enumerate() {
        let path = format!("$.roles[{i}]");
        let map = object(v, &path, &["role", "kind", "profile", "anchor"])?;
        let role = text(map, "role", &path)?;
        if !names.insert(role.clone()) {
            return Err(error(&format!("{path}.role"), "duplicate role"));
        }
        roles.push(Role {
            role,
            kind: role_kind(&text(map, "kind", &path)?, &format!("{path}.kind"))?,
            profile: number(required(map, "profile", &path)?, &format!("{path}.profile"))?,
            anchor: anchor(required(map, "anchor", &path)?, &format!("{path}.anchor"))?,
        });
    }
    validate_anchors(&roles)?;
    let mut free = Vec::new();
    if map.contains_key("free") {
        for (i, v) in field_array(map, "free", "$")?.iter().enumerate() {
            free.push(v.as_str().ok_or_else(|| error(&format!("$.free[{i}]"), "expected text"))?.to_owned());
        }
    }
    let mut obligations = Vec::new();
    let mut ids = BTreeSet::new();
    let profile = profile()?;
    for (i, v) in field_array(map, "obligations", "$")?.iter().enumerate() {
        let path = format!("$.obligations[{i}]");
        let item = obligation(v, &path)?;
        if !ids.insert(item.id.clone()) {
            return Err(error(&format!("{path}.id"), "duplicate id"));
        }
        if let ObligationKind::Setting(setting) = &item.kind {
            validate_setting(setting, &roles, &profile, &path)?;
        }
        obligations.push(item);
    }
    let mut text_rules = Vec::new();
    for (i, v) in field_array(map, "rules", "$")?.iter().enumerate() {
        let path = format!("$.rules[{i}]");
        let map = object(v, &path, &["id", "role", "words", "source"])?;
        let id = text(map, "id", &path)?;
        if !ids.insert(id.clone()) {
            return Err(error(&format!("{path}.id"), "duplicate id"));
        }
        let role = text(map, "role", &path)?;
        find_role(&roles, &role, &format!("{path}.role"))?;
        text_rules.push(Rule { id, role, words: text(map, "words", &path)?, source: text(map, "source", &path)? });
    }
    Ok(Golden { placeholders, roles, free, obligations, rules: text_rules })
}
fn find_role<'a>(roles: &'a [Role], name: &str, path: &str) -> Result<&'a Role, GoldenError> {
    roles.iter().find(|r| r.role == name).ok_or_else(|| error(path, "unknown role"))
}
fn validate_anchors(roles: &[Role]) -> Result<(), GoldenError> {
    for (i, role) in roles.iter().enumerate() {
        let path = format!("$.roles[{i}].anchor");
        let valid = match &role.anchor {
            Some(Anchor::Executes(name)) => {
                let target = find_role(roles, name, &format!("{path}.executes"))?;
                if target.kind != RoleKind::Template {
                    return Err(error(&format!("{path}.executes"), "expected a template role"));
                }
                role.kind == RoleKind::Charter
            }
            Some(Anchor::ForgeCharter(_)) => role.kind == RoleKind::Charter,
            Some(Anchor::Resource(_)) => role.kind == RoleKind::Template,
            Some(Anchor::Branch(_)) => role.kind == RoleKind::LandingRule,
            Some(Anchor::Number(_)) | None => true,
        };
        if !valid {
            return Err(error(&path, "anchor does not match role kind"));
        }
    }
    Ok(())
}
fn validate_setting(
    s: &SettingObligation,
    roles: &[Role],
    profile: &SettingsFile,
    path: &str,
) -> Result<(), GoldenError> {
    let role = match &s.subject {
        Subject::Role(name) => Some(find_role(roles, name, &format!("{path}.subject.role"))?),
        Subject::Setting(_) => None,
    };
    if let Some(value) = &s.value {
        validate_value(value, roles, &format!("{path}.value"))?;
    }
    if s.value.is_none() && matches!(s.test, Test::Equal | Test::Contains) {
        let found = role
            .and_then(|role| {
                profile
                    .items
                    .iter()
                    .find(|item| item.setting.kind == kind_name(role.kind) && item.setting.number == Some(role.profile))
            })
            .and_then(|item| match &item.edit {
                Edit::Set(v) => member(v, &s.member),
                Edit::Removed | Edit::Reset => None,
            });
        if found.is_none() {
            return Err(error(&format!("{path}.value"), "missing expected value and profile member"));
        }
    }
    Ok(())
}
fn validate_value(v: &Value, roles: &[Role], path: &str) -> Result<(), GoldenError> {
    match v {
        Value::Object(map) => {
            if let Some(Value::String(name)) = map.get("role") {
                find_role(roles, name, &format!("{path}.role"))?;
            }
            for (key, value) in map {
                validate_value(value, roles, &format!("{path}.{key}"))?;
            }
        }
        Value::Array(values) => {
            for (i, value) in values.iter().enumerate() {
                validate_value(value, roles, &format!("{path}[{i}]"))?;
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
    Ok(())
}
#[must_use]
pub(crate) fn kind_name(kind: RoleKind) -> &'static str {
    match kind {
        RoleKind::Charter => "charter",
        RoleKind::Template => "template",
        RoleKind::LandingRule => "landing-rule",
    }
}
/// Look up a dotted member, including numeric array indices; empty means the whole value.
#[must_use]
pub(crate) fn member<'a>(mut v: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() {
        return Some(v);
    }
    for word in path.split('.') {
        v = if v.is_array() { v.get(word.parse::<usize>().ok()?)? } else { v.get(word)? };
    }
    Some(v)
}
/// Read the embedded reference golden.
///
/// # Errors
/// Returns an invalid golden's path and reason.
pub fn reference() -> Result<Golden, GoldenError> {
    decode(include_bytes!("../golden/reference.json"))
}
/// Decode a settings file, retaining set values whole as JSON.
/// Evidence inventories include project policy beside the deployment settings.
///
/// # Errors
/// Returns the path of an invalid envelope, key, edit or repeated setting.
pub fn decode_settings(bytes: &[u8]) -> Result<SettingsFile, GoldenError> {
    let v = json(bytes)?;
    let map = object(&v, "$", &["items"])?;
    let mut items = Vec::new();
    let mut keys = BTreeSet::new();
    for (i, v) in field_array(map, "items", "$")?.iter().enumerate() {
        let path = format!("$.items[{i}]");
        let map = object(v, &path, &["setting", "set", "edit"])?;
        let setting = setting_key(required(map, "setting", &path)?, &format!("{path}.setting"), false)?;
        if !keys.insert(setting.clone()) {
            return Err(error(&format!("{path}.setting"), "duplicate setting"));
        }
        let edit = match (map.get("set"), map.get("edit")) {
            (Some(value), None) => Edit::Set(value.clone()),
            (None, Some(value)) => match string(value, &format!("{path}.edit"))?.as_str() {
                "removed" => Edit::Removed,
                "reset" => Edit::Reset,
                _ => return Err(error(&format!("{path}.edit"), "unknown edit")),
            },
            _ => return Err(error(&path, "expected exactly one of set or edit")),
        };
        items.push(Item { setting, edit });
    }
    Ok(SettingsFile { items })
}
/// Read the golden's embedded settings profile from `golden/profile.json`.
///
/// # Errors
/// Returns an invalid settings envelope's path and reason.
pub fn profile() -> Result<SettingsFile, GoldenError> {
    decode_settings(include_bytes!("../golden/profile.json"))
}

/// Print the obligations, or the whole golden as JSON.
#[must_use]
pub fn main(args: &[String]) -> ExitCode {
    if !(args.is_empty() || args == ["--json"]) {
        return crate::cli::refuse(&crate::TABLE.refusal(&crate::cli::OBLIGATIONS, "invalid arguments"));
    }
    let golden = match reference() {
        Ok(golden) => golden,
        Err(e) => {
            eprintln!("temper-stage-one: {e}");
            return ExitCode::from(2);
        }
    };
    if args.is_empty() {
        for obligation in &golden.obligations {
            let (kind, detail) = match &obligation.kind {
                ObligationKind::Setting(s) => {
                    ("setting", format!("{} {} {}", subject_text(&s.subject), s.member, test_name(s.test)))
                }
                ObligationKind::Box(b) => ("box", b.prerequisite.clone()),
                ObligationKind::Gap(g) => ("gap", g.status.clone()),
            };
            println!("{}  {kind}  {}: {detail}", obligation.id, obligation.row);
        }
    } else {
        print!("{}", include_str!("../golden/reference.json"));
    }
    ExitCode::SUCCESS
}
fn subject_text(subject: &Subject) -> String {
    match subject {
        Subject::Role(name) => name.clone(),
        Subject::Setting(key) => {
            let mut out = key.kind.clone();
            if let Some(project) = &key.project {
                match project {
                    ProjectRef::Box => out.push_str(" project box"),
                    ProjectRef::Number(n) => write!(out, " project {n}").expect("writing a String"),
                }
            }
            if let Some(number) = key.number {
                write!(out, " {number}").expect("writing a String");
            }
            out
        }
    }
}
fn test_name(test: Test) -> &'static str {
    match test {
        Test::Equal => "equal",
        Test::AtLeast => "at-least",
        Test::Bits => "bits",
        Test::Lacks => "lacks",
        Test::Contains => "contains",
        Test::Set => "set",
        Test::Judged => "judged",
        Test::Unchanged => "unchanged",
        Test::Only => "only",
    }
}
