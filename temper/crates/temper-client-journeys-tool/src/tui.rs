//! The TUI's in-memory proof and print (temper/docs/design/client/tui/journeys.md, 5 to 7).
use crate::{Build, Printed, Refusal, io_refusal, write_assets};
use std::{ffi::OsStr, fs, path::Path};
use temper_client_shell_tui::journey::{self, Line, Stale};

pub use temper_client_walk_lines::tui_print::{SCRIPT, beside, pages, stylesheet};

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

pub(crate) fn print(
    out: &Path,
    name: &str,
    file: &OsStr,
    recording: &[u8],
    given: &[Box<[u8]>],
    build: &Build,
) -> Result<Printed, Refusal> {
    let lines = journey::lines(recording).map_err(|why| proof_refusal(name, journey::Refusal::Unreadable(why)))?;
    if let Some(version) =
        lines.iter().find_map(|line| if let Line::Version(version) = line { Some(version) } else { None })
        && version.as_ref() != build.temper.as_bytes()
    {
        return Err(Refusal::Layout {
            file: file.to_string_lossy().into_owned(),
            recorded: format!("by temper {}", String::from_utf8_lossy(version)),
            read: format!("this is temper {}", build.temper),
        });
    }
    let redacted =
        journey::redact(recording, given).map_err(|why| proof_refusal(name, journey::Refusal::Unreadable(why)))?;
    if let Some(found) =
        journey::scan(&redacted.recording, &redacted.originals).map_err(|why| proof_refusal(name, why))?
    {
        return Err(Refusal::TuiFound { name: name.to_owned(), found, file: None });
    }
    let proved = journey::prove(&redacted.recording).map_err(|why| proof_refusal(name, why))?;
    let recording = proved.as_ref();
    let files = printed(name, recording)?.map_err(|why| proof_refusal(name, journey::Refusal::Stale(why)))?;
    let lines = journey::lines(recording).map_err(|why| proof_refusal(name, journey::Refusal::Unreadable(why)))?;
    let codes = journey::codes(&lines);
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
    let printed = Printed {
        name: name.to_owned(),
        link: redacted.originals.count(journey::Kind::Link),
        stored: redacted.originals.count(journey::Kind::Stored),
        typed: redacted.originals.count(journey::Kind::Typed),
        given: redacted.originals.count(journey::Kind::Given),
        approval: redacted.originals.count(journey::Kind::Approval),
        credential: redacted.originals.count(journey::Kind::Credential),
        bearer: redacted.originals.count(journey::Kind::Bearer),
        places: redacted.places,
    };
    let directory = out.join(temper_client_walk_lines::layout::TUI).join(name);
    write_files(&directory, files)?;
    write_assets(out, temper_client_walk_lines::walk::Client::Tui)?;
    Ok(printed)
}

pub(crate) fn printed(name: &str, recording: &[u8]) -> Result<Result<Vec<crate::print::Printed>, Stale>, Refusal> {
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
