//! The TUI's in-memory admission and reprint (temper/docs/design/client/tui/journeys.md, 5 to 7).
use crate::{Added, Refusal, io_refusal, names};
use std::{fs, path::Path};
use temper_client_shell_tui::journey::{self, Line, Stale};

mod print;
pub use print::{SCRIPT, beside, pages, stylesheet};

pub(crate) fn proof_refusal(name: &str, mut refusal: journey::Refusal) -> Refusal {
    if let journey::Refusal::Stale(stale) = &mut refusal {
        stale.recorded = kind(&stale.recorded).into();
        stale.asked = kind(&stale.asked).into();
    }
    Refusal::TuiProof { name: name.to_owned(), refusal }
}

fn kind(bytes: &[u8]) -> &[u8] {
    bytes.split(u8::is_ascii_whitespace).next().unwrap_or_default()
}

pub(crate) fn add(root: &Path, name: &str, recording: &[u8], given: &[Box<[u8]>]) -> Result<Added, Refusal> {
    let redacted =
        journey::redact(recording, given).map_err(|why| proof_refusal(name, journey::Refusal::Unreadable(why)))?;
    let recording = journey::prove(&redacted.recording).map_err(|why| proof_refusal(name, why))?;
    if let Some(found) = journey::scan(&recording, &redacted.originals).map_err(|why| proof_refusal(name, why))? {
        return Err(Refusal::TuiFound { name: name.to_owned(), found, file: None });
    }
    let lines = journey::lines(&recording).map_err(|why| proof_refusal(name, journey::Refusal::Unreadable(why)))?;
    let codes = journey::codes(&lines);
    let files = printed(name, &recording)?.map_err(|why| proof_refusal(name, journey::Refusal::Stale(why)))?;
    let marks: Vec<_> =
        lines.iter().enumerate().filter(|(_, line)| matches!(line, Line::Mark(_) | Line::End(_))).collect();
    for (files, (index, line)) in files.as_chunks::<2>().0.iter().zip(marks) {
        for file in files {
            if let Some(what) = journey::scan_bytes(file.document.as_bytes(), &codes, &redacted.originals) {
                return Err(Refusal::TuiFound {
                    name: name.to_owned(),
                    found: journey::Found {
                        line: u32::try_from(index + 1).expect("line fits u32"),
                        record: line.kind(),
                        place: journey::Place::Printed,
                        what,
                    },
                    file: Some(file.name.clone()),
                });
            }
        }
    }
    let added = Added {
        link: redacted.originals.count(journey::Kind::Link),
        stored: redacted.originals.count(journey::Kind::Stored),
        typed: redacted.originals.count(journey::Kind::Typed),
        given: redacted.originals.count(journey::Kind::Given),
        approval: redacted.originals.count(journey::Kind::Approval),
        credential: redacted.originals.count(journey::Kind::Credential),
        bearer: redacted.originals.count(journey::Kind::Bearer),
        places: redacted.places,
    };
    let directory = root.join(name);
    fs::create_dir_all(&directory).map_err(|_| io_refusal(&directory))?;
    let path = directory.join("recording");
    fs::write(&path, recording).map_err(|_| io_refusal(&path))?;
    write_files(&directory, files)?;
    beside(root).map_err(|_| io_refusal(root))?;
    Ok(added)
}

/// Reprint the TUI journeys, leaving each stale journey's files untouched.
/// # Errors
/// Refuses an unreadable recording or a filesystem failure.
pub fn reprint(root: &Path, out: &Path) -> Result<Vec<(String, Stale)>, Refusal> {
    let mut stale = Vec::new();
    for name in names(root)? {
        let path = root.join(&name).join("recording");
        let recording = fs::read(&path).map_err(|_| io_refusal(&path))?;
        match printed(&name, &recording)? {
            Ok(files) => write_files(&out.join(&name), files)?,
            Err(why) => stale.push((name, why)),
        }
    }
    fs::create_dir_all(out).map_err(|_| io_refusal(out))?;
    beside(out).map_err(|_| io_refusal(out))?;
    Ok(stale)
}

fn printed(name: &str, recording: &[u8]) -> Result<Result<Vec<crate::print::Printed>, Stale>, Refusal> {
    let mut replay =
        journey::Replay::new(recording).map_err(|why| proof_refusal(name, journey::Refusal::Unreadable(why)))?;
    let scheme = journey::lines(recording)
        .map_err(|why| proof_refusal(name, journey::Refusal::Unreadable(why)))?
        .into_iter()
        .find_map(|line| if let Line::Screen { scheme, .. } = line { Some(scheme) } else { None })
        .expect("replay validated the screen fact");
    let mut marks = Vec::new();
    loop {
        match replay.next() {
            Ok(Some(mark)) => marks.push(mark),
            Ok(None) => break,
            Err(why) => return Ok(Err(why)),
        }
    }
    Ok(Ok(pages(name, temper_client_tui::theme::ThemeChoice::Auto.scheme(scheme), &marks)))
}

pub(crate) fn printed_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".html").or_else(|| name.strip_suffix(".txt")))
        .is_some_and(|number| number.len() >= 2 && number.bytes().all(|byte| byte.is_ascii_digit()))
}

fn write_files(out: &Path, files: Vec<crate::print::Printed>) -> Result<(), Refusal> {
    fs::create_dir_all(out).map_err(|_| io_refusal(out))?;
    for entry in fs::read_dir(out).map_err(|_| io_refusal(out))? {
        let entry = entry.map_err(|_| io_refusal(out))?;
        let path = entry.path();
        if printed_file(&path) && entry.file_type().map_err(|_| io_refusal(&path))?.is_file() {
            fs::remove_file(&path).map_err(|_| io_refusal(&path))?;
        }
    }
    for file in files {
        let path = out.join(file.name);
        fs::write(&path, file.document).map_err(|_| io_refusal(&path))?;
    }
    Ok(())
}
