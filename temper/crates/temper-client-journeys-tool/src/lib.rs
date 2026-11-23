//! The journeys: the client's pages recorded, replayed and printed (client/web/journeys.md).

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use temper_client_shell_web::journey::{self, Stale};
use temper_client_walk_lines::walk::Client;

#[expect(dead_code, reason = "the writer and look-refresh commands will dispatch through this host")]
mod host;
mod output;
mod run;
mod table;
pub use run::main;
pub use table::TABLE;

type EnvironmentEntry = (Box<[u8]>, Box<[u8]>);

pub mod drive;
#[doc(hidden)]
pub mod testing {
    pub use crate::drive::testing::{configuration, deployment_plan, io};
}
pub mod journey_walk;
pub use temper_client_walk_lines::{line, named, print};
pub mod recipe;
pub mod tui;
mod tui_driver;
/// Where the TUI journeys live (temper/docs/design/client/tui/journeys.md, 6.1).
pub const TUI_JOURNEYS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/design/client/tui/journeys");
/// Where the journeys live, by the manifest's path (client/web/journeys.md, 5.1).
pub const JOURNEYS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/design/client/web/journeys");

#[derive(Debug)]
pub enum Refusal {
    OutputRoot,
    Layout { file: String, recorded: String, read: String },
    TuiProof { name: String, refusal: temper_client_shell_tui::journey::Refusal },
    TuiFound { name: String, found: temper_client_shell_tui::journey::Found, file: Option<String> },
    Undecoded { name: String, line: u32 },
    Found { name: String, line: u32, record: &'static str, place: journey::Place, what: journey::What },
    FoundDrawn { name: String, turn: u32, what: journey::What },
    FoundPrinted { name: String, page: String, what: journey::What },
    Proof { name: String, refusal: journey::Refusal },
    Io { path: PathBuf },
}

/// Counts of the originals and replacements (client/web/journey-redaction.md).
#[derive(Debug)]
pub struct Printed {
    pub name: String,
    pub link: u32,
    pub stored: u32,
    pub typed: u32,
    pub given: u32,
    pub places: u32,
    pub approval: u32,
    pub credential: u32,
    pub bearer: u32,
}

/// The layouts this build can prove (draft/archive/journey-walks.md, 4.3).
pub struct Build {
    pub wire: u32,
    pub temper: String,
}

const REPOSITORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../..");

/// Resolve OUT without writing, refusing the checkout's writer-owned roots.
/// # Errors
/// Refuses a path inside either root or one that cannot be resolved.
fn destination(
    out: &Path,
    repository: &Path,
    absolute: impl FnOnce(&Path) -> std::io::Result<PathBuf>,
    mut canonicalize: impl FnMut(&Path) -> std::io::Result<PathBuf>,
) -> Result<PathBuf, Refusal> {
    use temper_client_walk_lines::layout::{self, TUI, WEB};
    let out =
        absolute(out).and_then(|path| layout::resolve_with(&path, &mut canonicalize)).map_err(|_| io_refusal(out))?;
    for root in [WEB, TUI] {
        let path = repository.join(root);
        let root = layout::resolve_with(&path, &mut canonicalize).map_err(|_| io_refusal(&path))?;
        if out.starts_with(root) {
            return Err(Refusal::OutputRoot);
        }
    }
    Ok(out)
}

fn recording_name(file: &OsStr) -> String {
    Path::new(file)
        .file_name()
        .and_then(|name| name.as_encoded_bytes().split(|byte| *byte == b'.').next())
        .and_then(|name| std::str::from_utf8(name).ok())
        .filter(|name| named(name))
        .unwrap_or("recorded")
        .to_owned()
}

struct Admitted {
    client: Client,
    printed: Printed,
    pages: Vec<print::Printed>,
}

/// Check the layout, redact, scan and prove the recording before publication.
fn admit(file: &OsStr, recording: &[u8], given: &[Box<[u8]>], build: &Build) -> Result<Admitted, Refusal> {
    let name = recording_name(file);
    let is_tui = recording
        .split(|byte| *byte == b'\n')
        .skip_while(|line| !line.starts_with(b"clocks "))
        .nth(1)
        .is_some_and(|line| line.starts_with(b"screen "));
    if is_tui {
        return tui::admit(&name, file, recording, given, build);
    }
    let lines = journey::lines(recording).map_err(|why| proof_refusal(&name, journey::Refusal::Unreadable(why)))?;
    if let Some(path) =
        lines.iter().find_map(|line| if let journey::Line::Open { path, .. } = line { Some(path) } else { None })
    {
        let version = path
            .splitn(2, |byte| *byte == b'?')
            .nth(1)
            .and_then(|query| query.split(|byte| *byte == b'&').find_map(|field| field.strip_prefix(b"version=")))
            .and_then(|value| std::str::from_utf8(value).ok())
            .and_then(|value| value.parse::<u32>().ok());
        if let Some(version) = version.filter(|version| *version != build.wire) {
            return Err(Refusal::Layout {
                file: file.to_string_lossy().into_owned(),
                recorded: format!("at jig-client-wire {version}"),
                read: format!("this build reads jig-client-wire {}", build.wire),
            });
        }
    }
    admit_web(&name, recording, given)
}

fn admit_web(name: &str, recording: &[u8], given: &[Box<[u8]>]) -> Result<Admitted, Refusal> {
    let redacted = journey::redact(recording, given).map_err(|why| match why {
        journey::Unredacted::Unreadable(unreadable) => proof_refusal(name, journey::Refusal::Unreadable(unreadable)),
        journey::Unredacted::Undecoded(undecoded) => Refusal::Undecoded { name: name.to_owned(), line: undecoded.line },
    })?;
    let redacted_lines =
        journey::lines(&redacted.recording).map_err(|why| proof_refusal(name, journey::Refusal::Unreadable(why)))?;
    scanned(name, &redacted_lines, &redacted.originals)?;
    let mut drawn = None;
    let proof = journey::Replay::from_lines(redacted_lines).and_then(|replay| {
        replay.prove_drawn(&mut |turn, page| {
            if drawn.is_none() {
                drawn = journey::scan::page(&page.html(), &redacted.originals).map(|what| Refusal::FoundDrawn {
                    name: name.to_owned(),
                    turn,
                    what,
                });
            }
        })
    });
    if let Some(found) = drawn {
        return Err(found);
    }
    let lines = proof.map_err(|why| proof_refusal(name, why))?;
    let replay = journey::Replay::from_lines(lines).map_err(|why| proof_refusal(name, why))?;
    let pages = printed_replay(replay).map_err(|stale| proof_refusal(name, journey::Refusal::Stale(stale)))?;
    for page in &pages {
        if let Some(what) = journey::scan::bytes(page.document.as_bytes(), &redacted.originals) {
            return Err(Refusal::FoundPrinted { name: name.to_owned(), page: page.name.clone(), what });
        }
    }
    let printed = Printed {
        name: name.to_owned(),
        link: redacted.originals.count(journey::Kind::Link),
        stored: redacted.originals.count(journey::Kind::Stored),
        typed: redacted.originals.count(journey::Kind::Typed),
        given: redacted.originals.count(journey::Kind::Given),
        places: redacted.places,
        approval: 0,
        credential: 0,
        bearer: 0,
    };
    Ok(Admitted { client: Client::Web, printed, pages })
}

fn scanned(name: &str, lines: &[journey::Line], originals: &journey::Originals) -> Result<(), Refusal> {
    let found = journey::scan::lines(lines, originals)
        .map_err(|why| Refusal::Undecoded { name: name.to_owned(), line: why.line })?;
    if let Some(journey::Found { line, record, place, what }) = found {
        return Err(Refusal::Found { name: name.to_owned(), line, record, place, what });
    }
    Ok(())
}

fn proof_refusal(name: &str, mut refusal: journey::Refusal) -> Refusal {
    if let journey::Refusal::Stale(stale) = &mut refusal {
        stale.recorded = stale.recorded.split_whitespace().next().unwrap_or("").to_owned();
        stale.asked = stale.asked.split_whitespace().next().unwrap_or("").to_owned();
    }
    Refusal::Proof { name: name.to_owned(), refusal }
}

fn printed_replay(mut replay: journey::Replay) -> Result<Vec<print::Printed>, Stale> {
    let mut marks = Vec::new();
    loop {
        match replay.next() {
            Ok(Some(mark)) => {
                marks.push(print::Mark {
                    html: mark.page.html(),
                    heading: print::heading(mark.tree),
                    acted: mark.acted,
                    lead: mark.lead,
                });
            }
            Ok(None) => break,
            Err(stale) => return Err(stale),
        }
    }
    Ok(print::pages(&marks))
}

fn io_refusal(path: &Path) -> Refusal {
    Refusal::Io { path: path.to_owned() }
}

#[cfg(test)]
mod tests;
