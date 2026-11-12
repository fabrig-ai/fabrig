//! Finding files and kept evidence (qa/findings.md, 2; qa/ledger.md, 4.11 and 5).
use crate::{FindingInput, Home, Refusal, document, refused};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    ffi::OsStr,
    fs::{self, File},
    io::Read,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

const EVIDENCE_FILES: usize = 16;
const EVIDENCE_BYTES: u64 = 16 * 1024 * 1024;

impl FindingInput {
    /// Syntax errors name the input; field refusals use ledger.md, 5's words.
    pub fn decode(bytes: &[u8], file: &str) -> Result<Self, Refusal> {
        let value: Value =
            serde_json::from_slice(bytes).map_err(|why| refused(format!("finding refused: {file}: {why}")))?;
        let decode = || {
            let object = document::object(
                &value,
                &["case", "title", "severity", "area", "setup", "lines", "expected", "seen", "evidence"],
            )?;
            let case = object
                .get("case")
                .and_then(Value::as_u64)
                .filter(|number| *number > 0)
                .ok_or_else(|| refused("no case"))?;
            let text = |name| -> Result<String, Refusal> {
                object
                    .get(name)
                    .and_then(Value::as_str)
                    .filter(|text| !text.trim().is_empty())
                    .map(String::from)
                    .ok_or_else(|| refused(format!("no {name}")))
            };
            let title = text("title")?;
            let severity = text("severity")?;
            let setup = text("setup")?;
            let lines = object
                .get("lines")
                .and_then(Value::as_array)
                .filter(|lines| !lines.is_empty())
                .ok_or_else(|| refused("no lines"))?
                .iter()
                .map(document::string)
                .collect::<Result<_, _>>()?;
            let expected = text("expected")?;
            let seen = text("seen")?;
            Ok(Self {
                case,
                title,
                severity,
                setup,
                lines,
                expected,
                seen,
                area: object.get("area").map(document::string).transpose()?,
                evidence: object.get("evidence").map(document::strings).transpose()?.unwrap_or_default(),
            })
        };
        decode().map_err(|why: Refusal| refused(format!("finding refused: {why}")))
    }
}

pub(crate) fn finding(home: &Home, session: u64, file: &[u8]) -> Result<String, Refusal> {
    let mut copied = None;
    let result = home.transact(|ledger| {
        let pass = ledger.pass_command_session(session)?.pass.expect("pass session");
        let label = String::from_utf8_lossy(file);
        let bytes = if file == b"-" {
            let mut bytes = Vec::new();
            std::io::stdin()
                .read_to_end(&mut bytes)
                .map_err(|why| refused(format!("finding refused: {label}: {why}")))?;
            bytes
        } else {
            fs::read(Path::new(OsStr::from_bytes(file)))
                .map_err(|why| refused(format!("finding refused: {label}: {why}")))?
        };
        let mut input = FindingInput::decode(&bytes, &label)?;
        let setups = setups(&home.directory.join("passes").join(pass.to_string()).join("setups"))?;
        input.check(pass, &setups)?;
        let case = ledger.pass_command_case(session, input.case)?;
        let area = home.configuration.finding_area(&case.surface);
        let sources = evidence(&input.evidence)?;
        input.evidence = sources.iter().map(|(_, name)| name.clone()).collect();
        let now = skein_shell::Clock::new().now().wall.as_secs();
        // The lock keeps this number available until the operation records it.
        let number = crate::ledger::next_number(ledger.findings.iter().map(|finding| finding.number));
        if !sources.is_empty() {
            let root = home.directory.join("evidence");
            fs::create_dir_all(&root).map_err(|why| evidence_refusal(&root, why))?;
            let directory = root.join(number.to_string());
            fs::create_dir(&directory).map_err(|why| evidence_refusal(&directory, why))?;
            copied = Some(directory.clone());
            for (source, name) in sources {
                copy(&source, &directory.join(name))?;
            }
        }
        let answer = ledger.record_finding(session, &input, &setups, area, now)?;
        Ok(format!("{answer}\n"))
    });
    if result.is_err()
        && let Some(directory) = copied
    {
        fs::remove_dir_all(&directory).map_err(|why| evidence_refusal(&directory, why))?;
    }
    result
}

fn setups(directory: &Path) -> Result<Vec<String>, Refusal> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(why) => return Err(refused(format!("{}: {why}", directory.display()))),
    };
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|why| refused(format!("{}: {why}", directory.display())))?;
        if entry.file_type().map_err(|why| refused(format!("{}: {why}", entry.path().display())))?.is_dir() {
            names.push(
                entry
                    .file_name()
                    .into_string()
                    .map_err(|_| refused(format!("{}: setup name must be UTF-8", entry.path().display())))?,
            );
        }
    }
    names.sort();
    Ok(names)
}

fn evidence(paths: &[String]) -> Result<Vec<(PathBuf, String)>, Refusal> {
    let mut names = BTreeSet::new();
    let mut sources = Vec::new();
    for path in paths {
        let source = Path::new(path);
        if sources.len() == EVIDENCE_FILES {
            return Err(evidence_refusal(source, "more than 16 files"));
        }
        let name =
            source.file_name().and_then(OsStr::to_str).ok_or_else(|| evidence_refusal(source, "no file name"))?;
        if !names.insert(name.to_owned()) {
            return Err(evidence_refusal(source, format!("duplicate file name {name}")));
        }
        let metadata = fs::metadata(source).map_err(|why| evidence_refusal(source, why))?;
        check_file(source, &metadata)?;
        sources.push((source.to_owned(), name.to_owned()));
    }
    Ok(sources)
}
fn check_file(path: &Path, metadata: &fs::Metadata) -> Result<(), Refusal> {
    if !metadata.is_file() {
        return Err(evidence_refusal(path, "not a regular file"));
    }
    if metadata.len() > EVIDENCE_BYTES {
        return Err(evidence_refusal(path, "file exceeds 16 MiB"));
    }
    Ok(())
}
fn copy(source: &Path, destination: &Path) -> Result<(), Refusal> {
    let file = File::open(source).map_err(|why| evidence_refusal(source, why))?;
    check_file(source, &file.metadata().map_err(|why| evidence_refusal(source, why))?)?;
    let mut output = File::create(destination).map_err(|why| evidence_refusal(source, why))?;
    let length =
        std::io::copy(&mut file.take(EVIDENCE_BYTES + 1), &mut output).map_err(|why| evidence_refusal(source, why))?;
    if length > EVIDENCE_BYTES {
        return Err(evidence_refusal(source, "file exceeds 16 MiB"));
    }
    output.sync_all().map_err(|why| evidence_refusal(source, why))?;
    Ok(())
}
fn evidence_refusal(path: &Path, why: impl std::fmt::Display) -> Refusal {
    refused(format!("finding refused: evidence {}: {why}", path.display()))
}
