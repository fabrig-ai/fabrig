//! Walk files and root-first chains (temper/docs/design/client/journey-walks.md, 3.1; 2.2–2.3).
use crate::{
    line::{Line, Record, Web},
    named,
};
use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, Read},
    path::{Path, PathBuf},
};
use temper_client_shell_web::journey::read_fields;

#[derive(Debug, PartialEq, Eq)]
pub struct Roots {
    pub web: PathBuf,
    pub tui: PathBuf,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Walk {
    pub chain: Vec<Journey>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Journey {
    pub name: String,
    pub lines: Vec<Numbered>,
    pub scripts: Option<Box<[u8]>>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Numbered {
    pub number: u32,
    pub line: Line,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Unreadable {
    pub journey: String,
    pub line: u32,
    pub why: String,
}
impl Unreadable {
    fn new(journey: &str, line: u32, why: impl Into<String>) -> Self {
        Self { journey: journey.into(), line, why: why.into() }
    }
}

/// Read a journey and at most eight ancestors, root first.
pub fn read(roots: &Roots, name: &str) -> Result<Walk, Unreadable> {
    check_name(name).map_err(|why| Unreadable::new(name, 0, why))?;
    let mut chain: Vec<Journey> = vec![];
    let mut next = name.to_owned();
    loop {
        let after_error = |why: String| {
            chain.last().map_or_else(
                || Unreadable::new(&next, 0, &why),
                |child| Unreadable::new(&child.name, 2, format!("after {next:?}: {why}")),
            )
        };
        if chain.iter().any(|journey| journey.name == next) {
            return Err(after_error("closes a cycle".into()));
        }
        if chain.len() > 8 {
            return Err(after_error("is a ninth ancestor".into()));
        }
        let (directory, file) = open(roots, &next).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                after_error("missing journey".into())
            } else {
                Unreadable::new(&next, 0, error.to_string())
            }
        })?;
        let mut bytes = vec![];
        file.take(65537).read_to_end(&mut bytes).map_err(|error| Unreadable::new(&next, 0, error.to_string()))?;
        if bytes.len() > 65536 {
            return Err(Unreadable::new(&next, 0, "more than 64 KiB"));
        }
        if bytes.split(|byte| *byte == b'\n').next() != Some(b"walk 2") {
            return Err(if chain.is_empty() {
                Unreadable::new(&next, 1, "expected walk 2")
            } else {
                after_error("expected walk 2".into())
            });
        }
        let (journey, after) = journey(&directory, &next, &bytes)?;
        chain.push(journey);
        let Some(after) = after else { break };
        next = after;
    }
    chain.reverse();
    Ok(Walk { chain })
}

fn open(roots: &Roots, name: &str) -> io::Result<(PathBuf, File)> {
    let web = roots.web.join(name);
    match File::open(web.join("walk")) {
        Ok(file) => Ok((web, file)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let tui = roots.tui.join(name);
            File::open(tui.join("walk")).map(|file| (tui, file))
        }
        Err(error) => Err(error),
    }
}

fn check_name(name: &str) -> Result<(), &'static str> {
    if named(name) { Ok(()) } else { Err("name must be [a-z][a-z0-9-]*") }
}

fn journey(directory: &Path, name: &str, bytes: &[u8]) -> Result<(Journey, Option<String>), Unreadable> {
    let mut after = None;
    let mut lines = vec![];
    for (index, source) in bytes.strip_suffix(b"\n").unwrap_or(bytes).split(|byte| *byte == b'\n').enumerate().skip(1) {
        let number = u32::try_from(index + 1).expect("64 KiB file's line number fits u32");
        let refuse = |why| Unreadable::new(name, number, why);
        if number == 2 && source.starts_with(b"after ") {
            let mut fields = read_fields(source).map_err(|why| refuse(why.to_owned()))?;
            fields.word().map_err(|why| refuse(why.to_owned()))?;
            let parent = fields.word().map_err(|why| refuse(format!("expected bare parent: {why}")))?;
            let parent = std::str::from_utf8(&parent).map_err(|_| refuse("parent is not an ASCII name".into()))?;
            check_name(parent).map_err(|why| refuse(why.into()))?;
            if fields.field().is_some() {
                return Err(refuse("extra field after parent".into()));
            }
            after = Some(parent.into());
            continue;
        }
        let line = Line::parse(source).map_err(refuse)?;
        let forbidden = match line.record() {
            Record::Empty => continue,
            Record::Web(Web::Act { .. } | Web::See(_) | Web::Reload(_) | Web::Mark) => None,
            Record::Web(Web::Look(_)) => Some("web look"),
            Record::Web(Web::Requests) => Some("web requests"),
            Record::Ctl(_) => Some("ctl"),
            Record::Forge(_) => Some("forge"),
            Record::Deployment(_) => Some("deployment"),
            Record::End => Some("end"),
            Record::Tui(_) => Some("tui"),
        };
        if let Some(forbidden) = forbidden {
            return Err(refuse(format!("{forbidden} is not a journey's walk line")));
        }
        lines.push(Numbered { number, line });
    }
    let scripts = match fs::read(directory.join("scripts.json")) {
        Ok(bytes) => Some(bytes.into_boxed_slice()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(Unreadable::new(name, 0, format!("scripts.json: {error}"))),
    };
    Ok((Journey { name: name.into(), lines, scripts }, after))
}

/// Whether a directory's walk starts with exactly `walk 2`.
pub fn moved(directory: &Path) -> io::Result<bool> {
    let file = match File::open(directory.join("walk")) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    let mut header = vec![];
    BufReader::new(file.take(65537)).read_until(b'\n', &mut header)?;
    Ok(header.strip_suffix(b"\n").unwrap_or(&header) == b"walk 2")
}

#[cfg(test)]
mod tests;
