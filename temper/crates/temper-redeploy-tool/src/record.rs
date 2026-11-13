//! The whole-file journal and its JSON record (temper/docs/design/redeploy.md, 5, 7).
use crate::redeploy::{Mode, Outcome};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Before {
    pub projects: Vec<u64>,
    pub cut: bool,
    pub tasks: Vec<TaskBefore>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskBefore {
    pub task: u64,
    pub project: u64,
    pub label: u64,
    pub phase: String,
    pub held: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Previous {
    pub part: String,
    pub commit: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grace {
    pub worker: String,
    pub grace_ms: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Line {
    pub at: u64,
    pub line: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Check {
    pub code: Option<i32>,
    pub lines: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Window {
    pub from: u64,
    pub to: u64,
    pub worker: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outage {
    pub what: String,
    pub from: u64,
    pub to: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Opening {
    pub worker: String,
    pub at: u64,
    pub line: String,
    pub refused: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub commit: String,
    pub mode: Mode,
    pub began: u64,
    pub previous: Vec<Previous>,
    pub graces: Vec<Grace>,
    pub before: Before,
    pub lines: Vec<Line>,
    pub check: Option<Check>,
    pub backup: Option<String>,
    pub window: Option<Window>,
    pub outages: Vec<Outage>,
    pub openings: Vec<Opening>,
    pub outcome: Option<Outcome>,
    pub ended: Option<u64>,
}
impl Record {
    #[must_use]
    pub fn new(commit: &str, mode: Mode) -> Self {
        Self {
            commit: commit.into(),
            mode,
            began: 0,
            previous: Vec::new(),
            graces: Vec::new(),
            before: Before::default(),
            lines: Vec::new(),
            check: None,
            backup: None,
            window: None,
            outages: Vec::new(),
            openings: Vec::new(),
            outcome: None,
            ended: None,
        }
    }
}

pub struct Journal {
    pub record: Record,
    file: Option<PathBuf>,
    output: Box<dyn Write>,
}
impl Journal {
    #[must_use]
    pub fn new(record: Record, file: Option<PathBuf>, output: Box<dyn Write>) -> Self {
        Self { record, file, output }
    }
    pub fn line(&mut self, at: u64, line: &str) -> Result<(), String> {
        writeln!(self.output, "{line}").and_then(|()| self.output.flush()).map_err(|why| format!("output: {why}"))?;
        self.record.lines.push(Line { at, line: line.into() });
        self.save()
    }
    pub fn save(&mut self) -> Result<(), String> {
        let Some(file) = &self.file else {
            return Ok(());
        };
        let bytes = serde_json::to_vec_pretty(&self.record).map_err(|why| format!("record: {why}"))?;
        // The temporary is a sibling, so rename is atomic on the record's filesystem.
        let mut temporary = file.as_os_str().to_os_string();
        temporary.push(".tmp");
        let temporary = PathBuf::from(temporary);
        let result = (|| {
            let mut output = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
            output.write_all(&bytes)?;
            output.sync_all()?;
            drop(output);
            fs::rename(&temporary, file)
        })();
        result.map_err(|why| format!("record {}: {why}", file.display()))
    }
}
