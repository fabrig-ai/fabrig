//! Recipes, their root-first chains and walk lines (client/recipe.md, 2–6).
use crate::line::{Line, Record, Tui, Web};
use sha2::{Digest, Sha256};
use std::{fmt, path::Path};
use temper_client_shell_web::journey::read_fields;

mod command;
mod reader;
mod walk;
pub(crate) use command::{Mode, run};
use reader::{CommitFiles, Reader};
pub use walk::{Action, Event, Walk};

pub const SETUPS: &str = "temper/docs/development/qa/setups";

/// A refusal locates a recipe's record, or a file when no record applies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub name: String,
    pub line: Option<usize>,
    pub file: Option<String>,
    pub why: String,
}
impl Refusal {
    fn file(name: &str, file: &str, why: impl Into<String>) -> Self {
        Self { name: name.into(), line: None, file: Some(file.into()), why: why.into() }
    }
    fn line(name: &str, line: usize, why: impl Into<String>) -> Self {
        Self { name: name.into(), line: Some(line), file: None, why: why.into() }
    }
}
impl fmt::Display for Refusal {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "{}", self.name)?;
        if let Some(line) = self.line {
            write!(out, ":{line}")?;
        } else if let Some(file) = &self.file {
            write!(out, "/{file}")?;
        }
        write!(out, ": {}", self.why)
    }
}
impl std::error::Error for Refusal {}

/// Where a walk line came from. The resets belong to the walk, not a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Recipe { name: String, line: usize },
    Walk,
}
impl fmt::Display for Source {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Recipe { name, line } => write!(out, "{name}:{line}"),
            Self::Walk => out.write_str("walk:"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkLine {
    pub source: Source,
    pub line: Box<[u8]>,
}
#[derive(Debug)]
struct RecipeLine {
    number: usize,
    line: Line,
}
#[derive(Debug)]
struct Recipe {
    name: String,
    after: Option<String>,
    bytes: Vec<u8>,
    scripts: Option<Vec<u8>>,
    files: CommitFiles,
    steps: Vec<RecipeLine>,
    check: Vec<RecipeLine>,
}
/// A checked recipe and at most eight parents, stored root first.
#[derive(Debug)]
pub struct Chain {
    recipes: Vec<Recipe>,
}
impl Chain {
    pub fn read(setups: &Path, name: &str) -> Result<Self, Refusal> {
        Self::read_with(&Reader::Directory(setups), name)
    }
    fn read_with(reader: &Reader<'_>, name: &str) -> Result<Self, Refusal> {
        check_name(name).map_err(|why| Refusal::file(name, "recipe", why))?;
        let mut recipes: Vec<Recipe> = vec![];
        let mut next = name.to_owned();
        loop {
            if let Some(child) = recipes.last() {
                if recipes.iter().any(|r| r.name == next) {
                    return Err(Refusal::line(&child.name, 2, format!("after {next:?} closes a cycle")));
                }
                if recipes.len() > 8 {
                    return Err(Refusal::line(&child.name, 2, format!("after {next:?} is a ninth parent")));
                }
            }
            let recipe = Recipe::read(reader, &next).map_err(|refusal| {
                if let Some(child) = recipes.last()
                    && reader.missing(&next)
                {
                    return Refusal::line(&child.name, 2, format!("after {next:?}: {refusal}"));
                }
                refusal
            })?;
            let after = recipe.after.clone();
            recipes.push(recipe);
            let Some(after) = after else { break };
            next = after;
        }
        recipes.reverse();
        Ok(Self { recipes })
    }
    #[must_use]
    pub fn hash(&self) -> [u8; 32] {
        let mut hash = Sha256::new();
        for recipe in &self.recipes {
            for bytes in [recipe.name.as_bytes(), &recipe.bytes, recipe.scripts.as_deref().unwrap_or_default()] {
                hash.update(u64::try_from(bytes.len()).expect("file length fits u64").to_be_bytes());
                hash.update(bytes);
            }
            hash.update(u64::try_from(recipe.files.len()).expect("file count").to_be_bytes());
            for (path, bytes) in &recipe.files {
                for bytes in [path.as_ref(), bytes.as_slice()] {
                    hash.update(u64::try_from(bytes.len()).expect("file length").to_be_bytes());
                    hash.update(bytes);
                }
            }
        }
        hash.finalize().into()
    }
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.recipes.iter().map(|recipe| recipe.name.as_str())
    }
    /// Lines from a fresh box, or from a named member's restored copy.
    pub fn lines(&self, from: Option<&str>) -> Result<Vec<WalkLine>, Refusal> {
        let start = match from {
            Some(name) => self
                .recipes
                .iter()
                .position(|r| r.name == name)
                .ok_or_else(|| Refusal::file(name, "recipe", "--from is not the recipe or one of its ancestors"))?,
            None => 0,
        };
        let mut lines = vec![];
        for (index, recipe) in self.recipes.iter().enumerate().skip(start) {
            if from.is_none() || index != start {
                lines.extend(recipe.steps.iter().map(|line| recipe.walk_line(line)));
                lines.extend(
                    [br#"web reload "/""#.as_slice(), b"tui stop"]
                        .map(|line| WalkLine { source: Source::Walk, line: line.into() }),
                );
            }
            lines.extend(recipe.check.iter().map(|line| recipe.walk_line(line)));
        }
        Ok(lines)
    }
}
impl Recipe {
    fn walk_line(&self, line: &RecipeLine) -> WalkLine {
        WalkLine {
            source: Source::Recipe { name: self.name.clone(), line: line.number },
            line: line.line.print().into(),
        }
    }
    fn read(reader: &Reader<'_>, name: &str) -> Result<Self, Refusal> {
        let files = reader.files(name).map_err(|why| Refusal::file(name, "recipe", why))?;
        for file in &files {
            if file != "recipe" && file != "scripts.json" && file != "README.md" && file != "files" {
                return Err(Refusal::file(name, &file.to_string_lossy(), "unexpected file"));
            }
        }
        let bytes = reader.bytes(name, "recipe").map_err(|why| Refusal::file(name, "recipe", why))?;
        if bytes.len() > 65536 {
            return Err(Refusal::file(name, "recipe", "more than 64 KiB"));
        }
        let mut records = bytes.strip_suffix(b"\n").unwrap_or(&bytes).split(|byte| *byte == b'\n').enumerate();
        let Some((_, b"recipe 1")) = records.next() else {
            return Err(Refusal::line(name, 1, "expected recipe 1"));
        };
        let mut after = None;
        let mut steps = vec![];
        for (index, source) in records {
            let number = index + 1;
            let refuse = |why| Refusal::line(name, number, why);
            if number > 1024 {
                return Err(refuse("more than 1024 lines".into()));
            }
            if number == 2 && source.starts_with(b"after ") {
                let mut fields = read_fields(source).map_err(|why| refuse(why.into()))?;
                fields.word().map_err(|why| refuse(why.into()))?;
                let parent = fields.bytes().map_err(|why| refuse(format!("expected quoted parent: {why}")))?;
                let parent = std::str::from_utf8(&parent).map_err(|_| refuse("parent is not an ASCII name".into()))?;
                check_name(parent).map_err(|why| refuse(why.into()))?;
                if fields.field().is_some() {
                    return Err(refuse("extra field after parent".into()));
                }
                after = Some(parent.to_owned());
                continue;
            }
            let line = Line::parse(source).map_err(refuse)?;
            if !line.kept() || matches!(line.record(), Record::Tui(Tui::Stop)) {
                return Err(refuse("empty lines, looks, end and tui stop are not recipe acts".into()));
            }
            steps.push(RecipeLine { number, line });
        }
        let check_at = steps.iter().rposition(|line| !is_check(&line.line)).map_or(0, |index| index + 1);
        if steps.is_empty() || check_at == steps.len() {
            let last = bytes.strip_suffix(b"\n").unwrap_or(&bytes).split(|byte| *byte == b'\n').count().max(1);
            return Err(Refusal::line(name, last, "recipe must end with at least one web see or tui see"));
        }
        let check = steps.split_off(check_at);
        let scripts = if files.iter().any(|file| file == "scripts.json") {
            Some(reader.bytes(name, "scripts.json").map_err(|why| Refusal::file(name, "scripts.json", why))?)
        } else {
            None
        };
        let files = reader.commit_files(name)?;
        let named: std::collections::BTreeSet<_> = steps
            .iter()
            .filter_map(|line| match line.line.record() {
                Record::Forge(crate::line::Forge::Commit { paths, .. }) => Some(paths.iter().map(AsRef::as_ref)),
                _ => None,
            })
            .flatten()
            .collect();
        for path in &named {
            if !files.contains_key(*path) {
                return Err(Refusal::file(
                    name,
                    &format!("files/{}", String::from_utf8_lossy(path)),
                    "missing forge commit file",
                ));
            }
        }
        for path in files.keys() {
            if !named.contains(path.as_ref()) {
                return Err(Refusal::file(
                    name,
                    &format!("files/{}", String::from_utf8_lossy(path)),
                    "no forge commit line names this file",
                ));
            }
        }
        Ok(Self { name: name.into(), after, bytes, scripts, files, steps, check })
    }
}
fn check_name(name: &str) -> Result<(), &'static str> {
    if name.len() > 64 || !crate::named(name) {
        return Err("name must be 1–64 bytes of [a-z][a-z0-9-]*");
    }
    Ok(())
}
fn is_check(line: &Line) -> bool {
    matches!(line.record(), Record::Web(Web::See(_)) | Record::Tui(Tui::See(_)))
}
#[must_use]
pub fn hex(hash: [u8; 32]) -> String {
    use std::fmt::Write;
    let mut text = String::with_capacity(64);
    for byte in hash {
        write!(text, "{byte:02x}").expect("write to String");
    }
    text
}
#[cfg(test)]
mod tests;
