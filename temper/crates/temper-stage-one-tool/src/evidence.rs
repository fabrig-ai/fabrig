//! Evidence files: `temper/docs/design/testing/checks.md`, 5.3.
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::golden::{self, SettingsFile};

#[derive(Clone, Debug, PartialEq)]
pub struct Evidence {
    pub facts: BoxFacts,
    pub revision: Value,
    pub items: SettingsFile,
    pub current: SettingsFile,
    pub proposal: String,
    pub chat: String,
    pub confirmed: Vec<Confirmation>,
    pub description: Option<String>,
    pub documents: BTreeMap<String, String>,
    pub person: BTreeMap<String, String>,
    pub judge: Option<Value>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoxFacts {
    pub forge: String,
    pub repository: String,
    pub project: u32,
    pub revision: u32,
    pub proposal: String,
    pub chat: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Confirmation {
    pub id: String,
    pub present: bool,
    pub seen: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Confirmed {
    r#box: Vec<Confirmation>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceError {
    pub file: PathBuf,
    pub reason: String,
}
impl fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.file.display(), self.reason)
    }
}
impl std::error::Error for EvidenceError {}
fn failure(file: &Path, reason: impl fmt::Display) -> EvidenceError {
    EvidenceError { file: file.into(), reason: reason.to_string() }
}
fn read_text(path: &Path) -> Result<String, EvidenceError> {
    fs::read_to_string(path).map_err(|e| failure(path, e))
}
fn optional_text(path: &Path) -> Result<Option<String>, EvidenceError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(failure(path, e)),
    }
}
fn read_json(path: &Path) -> Result<Value, EvidenceError> {
    serde_json::from_str(&read_text(path)?).map_err(|e| failure(path, e))
}
fn read_settings(path: &Path) -> Result<SettingsFile, EvidenceError> {
    golden::decode_settings(read_text(path)?.as_bytes()).map_err(|e| failure(path, e))
}
/// Read the required evidence and any optional description, documents or judge verdict.
///
/// # Errors
/// Names the file that cannot be read or decoded.
pub fn read(dir: &Path) -> Result<Evidence, EvidenceError> {
    let path = dir.join("box.json");
    let facts = serde_json::from_str(&read_text(&path)?).map_err(|e| failure(&path, e))?;
    let revision = read_json(&dir.join("revision.json"))?;
    let items = read_settings(&dir.join("items.json"))?;
    let current = read_settings(&dir.join("current.json"))?;
    let proposal = read_text(&dir.join("proposal.txt"))?;
    let chat = read_text(&dir.join("chat.txt"))?;
    let path = dir.join("confirmed.json");
    let confirmed: Confirmed = serde_json::from_str(&read_text(&path)?).map_err(|e| failure(&path, e))?;
    let description = optional_text(&dir.join("description.txt"))?;
    let mut documents = BTreeMap::new();
    read_documents(&dir.join("documents"), &dir.join("documents"), &mut documents, true)?;
    let mut person = BTreeMap::new();
    read_documents(&dir.join("person"), &dir.join("person"), &mut person, false)?;
    let path = dir.join("judge.json");
    let judge = optional_text(&path)?.map(|s| serde_json::from_str(&s).map_err(|e| failure(&path, e))).transpose()?;
    Ok(Evidence {
        facts,
        revision,
        items,
        current,
        proposal,
        chat,
        confirmed: confirmed.r#box,
        description,
        documents,
        person,
        judge,
    })
}
fn read_documents(
    base: &Path,
    dir: &Path,
    out: &mut BTreeMap<String, String>,
    recursive: bool,
) -> Result<(), EvidenceError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound && base == dir => return Ok(()),
        Err(e) => return Err(failure(dir, e)),
    };
    for entry in entries {
        let entry = entry.map_err(|e| failure(dir, e))?;
        let path = entry.path();
        let kind = entry.file_type().map_err(|e| failure(&path, e))?;
        if kind.is_symlink() {
            return Err(failure(&path, "expected an evidence file, not a symbolic link"));
        }
        if kind.is_dir() && recursive {
            read_documents(base, &path, out, true)?;
        } else if kind.is_file() && (recursive || path.extension().is_some_and(|s| s == "md")) {
            let relative = path.strip_prefix(base).expect("entry below document root");
            let name = relative.to_str().ok_or_else(|| failure(&path, "file name is not UTF-8"))?;
            out.insert(name.to_owned(), read_text(&path)?);
        }
    }
    Ok(())
}
