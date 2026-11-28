//! A look request against the landing's print and stylesheets (temper/docs/design/client/journey-writer-requests.md, 1.1).

use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    fmt::Write as _,
    os::unix::ffi::OsStrExt as _,
    path::{Path, PathBuf},
};

use temper_client_walk_lines::layout::{TUI, WEB};

use crate::{
    host::{Answer, Ask, Host, Program, Ran, Stream},
    look_refresh::valid_name,
    table::BUNDLE,
};

pub(crate) mod css;
pub(crate) mod pages;
pub(crate) mod request;

pub(crate) fn run(arguments: &[Box<[u8]>], host: &mut Host) -> u8 {
    let request = match request::read(arguments, host) {
        Ok(request) => request,
        Err(why) => return diagnostic(host, 2, &why),
    };
    match compare(&request, host) {
        Ok(report) => {
            host.out(report.rows.as_bytes());
            host.err(report.details.as_bytes());
            u8::from(report.differs)
        }
        Err(why) => diagnostic(host, 3, &why),
    }
}

fn diagnostic(host: &mut Host, code: u8, why: &str) -> u8 {
    host.err(format!("look-compare: {why}\n").as_bytes());
    code
}

fn compare(request: &request::Request, host: &mut Host) -> Result<Report, String> {
    let root = git_path(host, &["rev-parse", "--show-toplevel"])?;
    let scratch = git_path(host, &["rev-parse", "--path-format=absolute", "--git-path", "look-compare"])?;
    done(host, Ask::Remove(scratch.clone()))?;
    done(host, Ask::Directory(scratch.clone()))?;
    let changes = request.changes(host, &root)?;
    let print = scratch.join("print");
    let journeys = printable(host, &root, &changes.journeys)?;
    let stopped = print_journeys(host, &root, &print, &journeys)?;
    let mut report = Report::new();
    for (journey, change) in &changes.journeys {
        let item = code(&journey.item());
        match change {
            Change::HandMade => report.row(&item, "not compared: hand-made"),
            Change::Pages(_) if !journeys.contains(&journey) => report.problem(&item, "missing: no walk"),
            Change::Pages(_) if stopped.contains_key(&journey.name) => {
                report.problem(&item, &format!("stopped: {}", stopped[&journey.name]));
            }
            Change::Pages(files) => {
                for file in files {
                    pages::compare(host, request, &scratch, &print, journey, file, &mut report)?;
                }
            }
        }
    }
    for side in changes.overlays {
        css::compare(host, request, &root, side, &mut report)?;
    }
    report.row("each state the message names in words", "to fill, a row each");
    Ok(report)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Side {
    Web,
    Tui,
}

impl Side {
    pub(crate) fn root(self) -> &'static str {
        match self {
            Self::Web => WEB,
            Self::Tui => TUI,
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Tui => "tui",
        }
    }

    pub(crate) fn numbered_page(self, file: &str) -> bool {
        let number =
            file.strip_suffix(".html").or_else(|| if self == Self::Tui { file.strip_suffix(".txt") } else { None });
        number.is_some_and(|number| number.len() >= 2 && number.bytes().all(|byte| byte.is_ascii_digit()))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Journey {
    pub(crate) side: Side,
    pub(crate) name: String,
}

impl Journey {
    pub(crate) fn path(&self) -> String {
        format!("{}/{}", self.side.root(), self.name)
    }

    fn item(&self) -> String {
        format!("{}/journeys/{}", self.side.name(), self.name)
    }
}

pub(crate) enum Change {
    HandMade,
    Pages(BTreeSet<String>),
}

pub(crate) fn version_two(bytes: &[u8]) -> bool {
    bytes.split(|&byte| byte == b'\n').next() == Some(b"walk 2")
}

fn printable<'a>(
    host: &mut Host,
    root: &Path,
    changes: &'a BTreeMap<Journey, Change>,
) -> Result<Vec<&'a Journey>, String> {
    let mut journeys = Vec::new();
    for (journey, change) in changes {
        if let Change::Pages(_) = change
            && read(host, &root.join(journey.path()).join("walk"))?.is_some_and(|bytes| version_two(&bytes))
        {
            if journey.side == Side::Tui {
                let web = root.join(WEB).join(&journey.name).join("walk");
                match host.ask(Ask::Exists(web)) {
                    Answer::Is(false) => (),
                    Answer::Is(true) => {
                        return Err(format!(
                            "{} is shadowed by a walk in web/journeys/{}; rename one directory",
                            journey.item(),
                            journey.name
                        ));
                    }
                    answer => return Err(effect_error(answer)),
                }
            }
            journeys.push(journey);
        }
    }
    Ok(journeys)
}

fn print_journeys(
    host: &mut Host,
    root: &Path,
    print: &Path,
    journeys: &[&Journey],
) -> Result<BTreeMap<String, String>, String> {
    let mut stopped = BTreeMap::new();
    if journeys.is_empty() {
        return Ok(stopped);
    }
    let mut program = Program::new(
        b"cargo",
        &[
            b"run",
            b"--locked",
            b"--quiet",
            b"-p",
            b"temper-client-walk",
            b"--",
            b"print",
            b"--out",
            print.as_os_str().as_bytes(),
        ],
    );
    program.directory = Some(root.to_owned());
    program.environment.push((BUNDLE.name.as_bytes().into(), b"none".as_slice().into()));
    program.arguments.extend(journeys.iter().map(|journey| journey.name.as_bytes().into()));
    let ran = child(host, program)?;
    if !matches!(ran.code, Some(0 | 1)) {
        if ran.code.is_none() && !ran.stderr.is_empty() {
            return Err(format!("the print couldn't finish: {}", String::from_utf8_lossy(&ran.stderr).trim_end()));
        }
        return Err(format!("the print couldn't finish ({})", status(ran.code)));
    }
    let mut lines = text(&ran.stdout, "print report")?.lines();
    for journey in journeys {
        let Some(line) = lines.next() else { return Err("the print omitted a journey".into()) };
        if line.strip_prefix("printed ") == Some(journey.name.as_str()) {
            continue;
        }
        if let Some(why) = line.strip_prefix(&format!("stopped {}: ", journey.name)) {
            stopped.insert(journey.name.clone(), why.to_owned());
        } else {
            return Err(format!("unexpected print line: {line}"));
        }
    }
    // The walk tool resolves the output's path before naming it.
    if !lines.next().is_some_and(|line| line.starts_with("printed into "))
        || lines.next().is_some()
        || (ran.code == Some(0)) != stopped.is_empty()
    {
        return Err("the print gave an incomplete report".into());
    }
    Ok(stopped)
}

struct Report {
    rows: String,
    details: String,
    differs: bool,
}

impl Report {
    fn new() -> Self {
        Self { rows: "| Item | Result |\n|---|---|\n".into(), details: String::new(), differs: false }
    }

    fn row(&mut self, item: &str, result: &str) {
        writeln!(self.rows, "| {} | {} |", cell(item), cell(result)).expect("write to String");
    }

    fn problem(&mut self, item: &str, result: &str) {
        self.differs = true;
        self.row(item, result);
    }

    fn detail(&mut self, item: &str, result: &str, evidence: &str) {
        self.problem(item, result);
        writeln!(self.details, "look-compare: {}", cell(item)).expect("write to String");
        self.details.push_str(evidence);
        if !evidence.ends_with('\n') {
            self.details.push('\n');
        }
    }
}

fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace(['\r', '\n'], " ")
}

fn code(text: &str) -> String {
    let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    let delimiter = "`".repeat(longest + 1);
    let space = if text.starts_with('`') || text.ends_with('`') { " " } else { "" };
    format!("{delimiter}{space}{text}{space}{delimiter}")
}

pub(crate) fn read(host: &mut Host, path: &Path) -> Result<Option<Box<[u8]>>, String> {
    match host.ask(Ask::Read(path.to_owned())) {
        Answer::Bytes(bytes) => Ok(Some(bytes)),
        Answer::Missing => Ok(None),
        answer => Err(effect_error(answer)),
    }
}

pub(crate) fn done(host: &mut Host, ask: Ask) -> Result<(), String> {
    match host.ask(ask) {
        Answer::Done => Ok(()),
        answer => Err(effect_error(answer)),
    }
}

fn effect_error(answer: Answer) -> String {
    let Answer::Failed(why) = answer else { unreachable!("unexpected host answer: {answer:?}") };
    why
}

fn child(host: &mut Host, program: Program) -> Result<Ran, String> {
    match host.ask(Ask::Run(program)) {
        Answer::Ran(ran) => Ok(ran),
        answer => Err(effect_error(answer)),
    }
}

fn git(host: &mut Host, directory: Option<&Path>, args: &[&str]) -> Result<Ran, String> {
    let mut program = Program::new(b"git", &args.iter().map(|arg| arg.as_bytes()).collect::<Vec<_>>());
    program.directory = directory.map(Path::to_owned);
    program.stderr = Stream::Keep;
    child(host, program)
}

pub(crate) fn git_ok(host: &mut Host, directory: Option<&Path>, args: &[&str]) -> Result<Box<[u8]>, String> {
    let result = git(host, directory, args)?;
    if result.code == Some(0) { Ok(result.stdout) } else { Err(git_error(&args.join(" "), &result)) }
}

fn status(code: Option<i32>) -> String {
    code.map_or_else(|| "no exit status".into(), |code| format!("exit {code}"))
}

fn git_error(command: &str, ran: &Ran) -> String {
    let stderr = String::from_utf8_lossy(&ran.stderr);
    let stderr = stderr.trim_end();
    let status = status(ran.code);
    if stderr.is_empty() { format!("git {command}: {status}") } else { format!("git {command}: {status}: {stderr}") }
}

pub(crate) fn git_path(host: &mut Host, args: &[&str]) -> Result<PathBuf, String> {
    let bytes = git_ok(host, None, args)?;
    let path = PathBuf::from(OsStr::from_bytes(bytes.strip_suffix(b"\n").unwrap_or(&bytes)));
    if path.is_absolute() { Ok(path) } else { Err(format!("git {} did not give an absolute path", args.join(" "))) }
}

pub(crate) fn text<'a>(bytes: &'a [u8], what: &str) -> Result<&'a str, String> {
    std::str::from_utf8(bytes).map_err(|_| format!("{what} is not UTF-8"))
}

#[cfg(test)]
pub(crate) mod tests;
