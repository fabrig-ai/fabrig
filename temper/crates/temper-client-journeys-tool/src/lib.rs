//! The journeys: the client's pages recorded, replayed and printed (client/web/journeys.md).

use std::{
    fs, io,
    path::{Path, PathBuf},
};
use temper_client_shell_web::journey::{self, Stale};

mod run;
mod table;
pub use run::main;
pub use table::TABLE;

type EnvironmentEntry = (Box<[u8]>, Box<[u8]>);

pub mod drive;
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
    Name(String),
    OtherClient { name: String },
    Moved { name: String },
    TuiProof { name: String, refusal: temper_client_shell_tui::journey::Refusal },
    TuiFound { name: String, found: temper_client_shell_tui::journey::Found, file: Option<String> },
    Undecoded { name: String, line: u32 },
    Found { name: String, line: u32, record: &'static str, place: journey::Place, what: journey::What },
    FoundDrawn { name: String, turn: u32, what: journey::What },
    FoundPrinted { name: String, page: String, what: journey::What },
    Proof { name: String, refusal: journey::Refusal },
    Unreadable { name: String, unreadable: journey::Unreadable },
    Io { path: PathBuf },
}

/// Counts of the originals and replacements (client/web/journey-redaction.md).
#[derive(Debug)]
pub struct Added {
    pub link: u32,
    pub stored: u32,
    pub typed: u32,
    pub given: u32,
    pub places: u32,
    pub approval: u32,
    pub credential: u32,
    pub bearer: u32,
    pub build: u32,
    pub port: u32,
    pub code: u32,
    pub placeholder_places: u32,
    pub stood: u32,
}

/// Takes a recording as NAME, redacts, proves, normalises and scans it in memory, then
/// replaces its recording and pages (client/web/journey-redaction.md and client/web/journeys.md, 3.7 and 4.3–5.1).
/// # Errors
/// Refuses an invalid name, unreadable or unsafe bytes, a failed proof or io.
pub fn add(journeys: &Path, name: &str, recording: &[u8], given: &[Box<[u8]>]) -> Result<Added, Refusal> {
    if !named(name) {
        return Err(Refusal::Name(name.to_owned()));
    }
    add_to(journeys, &journeys.with_file_name("..").join("tui/journeys"), name, recording, given)
}

/// Dispatch a recording by its first fact after clocks, keeping names unique
/// across the two clients (temper/docs/design/client/tui/journeys.md, 6.1).
/// # Errors
/// Refuses an invalid or occupied name, unsafe recording, failed proof or io.
pub fn add_to(
    journeys: &Path,
    tui_journeys: &Path,
    name: &str,
    recording: &[u8],
    given: &[Box<[u8]>],
) -> Result<Added, Refusal> {
    if !named(name) {
        return Err(Refusal::Name(name.to_owned()));
    }
    refuse_moved(journeys, tui_journeys, name)?;
    let is_tui = recording
        .split(|byte| *byte == b'\n')
        .skip_while(|line| !line.starts_with(b"clocks "))
        .nth(1)
        .is_some_and(|line| line.starts_with(b"screen "));
    let other = if is_tui { journeys } else { tui_journeys };
    if other.join(name).exists() {
        return Err(Refusal::OtherClient { name: name.to_owned() });
    }
    if is_tui {
        return tui::add(tui_journeys, name, recording, given);
    }
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
    // A differing live print must not hide a hit in the printed document.
    let (normalised, proof) = match proof {
        Ok(lines) => (journey::normalise_lines(lines), Ok(())),
        Err(refusal) => (journey::normalise(&redacted.recording), Err(refusal)),
    };
    let normalised = normalised.map_err(|why| match why {
        journey::Unredacted::Unreadable(unreadable) => proof_refusal(name, journey::Refusal::Unreadable(unreadable)),
        journey::Unredacted::Undecoded(undecoded) => Refusal::Undecoded { name: name.to_owned(), line: undecoded.line },
    })?;
    let normalised_lines =
        journey::lines(&normalised.recording).map_err(|why| proof_refusal(name, journey::Refusal::Unreadable(why)))?;
    let replay = journey::Replay::from_lines(normalised_lines.clone()).map_err(|why| replay_refusal(name, why))?;
    let pages = printed_replay(replay).map_err(|stale| proof_refusal(name, journey::Refusal::Stale(stale)))?;
    scanned(name, &normalised_lines, &redacted.originals)?;
    for page in &pages {
        if let Some(what) = journey::scan::bytes(page.document.as_bytes(), &redacted.originals) {
            return Err(Refusal::FoundPrinted { name: name.to_owned(), page: page.name.clone(), what });
        }
    }
    proof.map_err(|refusal| proof_refusal(name, refusal))?;
    let added = Added {
        link: redacted.originals.count(journey::Kind::Link),
        stored: redacted.originals.count(journey::Kind::Stored),
        typed: redacted.originals.count(journey::Kind::Typed),
        given: redacted.originals.count(journey::Kind::Given),
        places: redacted.places,
        approval: 0,
        credential: 0,
        bearer: 0,
        build: normalised.build,
        port: normalised.port,
        code: normalised.code,
        placeholder_places: normalised.places,
        stood: normalised.stood,
    };
    let directory = journeys.join(name);
    fs::create_dir_all(&directory).map_err(|_| io_refusal(&directory))?;
    let path = directory.join("recording");
    fs::write(&path, &normalised.recording).map_err(|_| io_refusal(&path))?;
    write_pages(&directory, pages)?;
    print::beside(journeys).map_err(|_| io_refusal(journeys))?;
    Ok(added)
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

/// Reprints every journey under `journeys` (each directory holding a `recording`)
/// into `out`, in name order, and writes the files beside them. A stale journey's
/// pages are left as they were; it is returned with where it went stale.
/// # Errors
/// Refuses an unreadable committed recording or a filesystem failure.
fn reprint(journeys: &Path, out: &Path) -> Result<Vec<(String, Stale)>, Refusal> {
    let mut stale = Vec::new();
    for name in names(journeys)? {
        let path = journeys.join(&name).join("recording");
        let recording = fs::read(&path).map_err(|_| io_refusal(&path))?;
        if let Some(stopped) = print_one(&out.join(&name), &name, &recording)? {
            stale.push((name, stopped));
        }
    }
    fs::create_dir_all(out).map_err(|_| io_refusal(out))?;
    print::beside(out).map_err(|_| io_refusal(out))?;
    Ok(stale)
}

/// Written recordings with their placeholder counts, and safe stale refusals.
pub type Normalising = (Vec<(String, journey::Normalised)>, Vec<Refusal>);

/// Give both directories' recordings their placeholders after their replay holds
/// (temper/docs/design/client/journey-placeholders.md, 4.3). Stale recordings stay
/// untouched; the caller reprints both directories afterwards.
/// # Errors
/// Refuses an unreadable recording or a filesystem failure.
pub fn normalise(journeys: &Path, tui_journeys: &Path) -> Result<Normalising, Refusal> {
    let mut written = Vec::new();
    let mut stale = Vec::new();
    for (root, is_tui) in [(journeys, false), (tui_journeys, true)] {
        for name in names(root)? {
            let path = root.join(&name).join("recording");
            let recording = fs::read(&path).map_err(|_| io_refusal(&path))?;
            let normalised = if is_tui {
                temper_client_shell_tui::journey::normalise(&recording).map_err(|why| {
                    tui::proof_refusal(&name, temper_client_shell_tui::journey::Refusal::Unreadable(why))
                })?
            } else {
                journey::normalise(&recording).map_err(|why| match why {
                    journey::Unredacted::Unreadable(why) => proof_refusal(&name, journey::Refusal::Unreadable(why)),
                    journey::Unredacted::Undecoded(why) => Refusal::Undecoded { name: name.clone(), line: why.line },
                })?
            };
            if normalised.recording.as_ref() == recording {
                continue;
            }
            let stopped = if is_tui {
                tui::printed(&name, &normalised.recording)?
                    .err()
                    .map(|why| tui::proof_refusal(&name, temper_client_shell_tui::journey::Refusal::Stale(why)))
            } else {
                printed(&name, &normalised.recording)?
                    .err()
                    .map(|why| proof_refusal(&name, journey::Refusal::Stale(why)))
            };
            if let Some(why) = stopped {
                stale.push(why);
                continue;
            }
            fs::write(&path, &normalised.recording).map_err(|_| io_refusal(&path))?;
            written.push((name, normalised));
        }
    }
    Ok((written, stale))
}

fn names(journeys: &Path) -> Result<Vec<String>, Refusal> {
    let entries = match fs::read_dir(journeys) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => return Err(io_refusal(journeys)),
    };
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| io_refusal(journeys))?;
        let path = entry.path();
        if !entry.file_type().map_err(|_| io_refusal(&path))?.is_dir() {
            continue;
        }
        if moved(&path)? {
            continue;
        }
        let recording = path.join("recording");
        if recording.try_exists().map_err(|_| io_refusal(&recording))? {
            names.push(entry.file_name().into_string().map_err(|_| io_refusal(&path))?);
        }
    }
    names.sort();
    Ok(names)
}

fn moved(directory: &Path) -> Result<bool, Refusal> {
    temper_client_walk_lines::walk::moved(directory).map_err(|_| io_refusal(&directory.join("walk")))
}

fn refuse_moved(web: &Path, tui: &Path, name: &str) -> Result<(), Refusal> {
    for root in [web, tui] {
        if moved(&root.join(name))? {
            return Err(Refusal::Moved { name: name.to_owned() });
        }
    }
    Ok(())
}

fn print_one(out: &Path, name: &str, recording: &[u8]) -> Result<Option<Stale>, Refusal> {
    match printed(name, recording)? {
        Ok(pages) => {
            write_pages(out, pages)?;
            Ok(None)
        }
        Err(stale) => Ok(Some(stale)),
    }
}

fn printed(name: &str, recording: &[u8]) -> Result<Result<Vec<print::Printed>, Stale>, Refusal> {
    let replay = journey::Replay::new(recording).map_err(|why| replay_refusal(name, why))?;
    Ok(printed_replay(replay))
}

fn replay_refusal(name: &str, refusal: journey::Refusal) -> Refusal {
    match refusal {
        journey::Refusal::Unreadable(unreadable) => Refusal::Unreadable { name: name.to_owned(), unreadable },
        refusal @ (journey::Refusal::Stale(_)
        | journey::Refusal::Unproved { .. }
        | journey::Refusal::Differs { .. }) => Refusal::Proof { name: name.to_owned(), refusal },
    }
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

fn write_pages(out: &Path, pages: Vec<print::Printed>) -> Result<(), Refusal> {
    fs::create_dir_all(out).map_err(|_| io_refusal(out))?;
    for entry in fs::read_dir(out).map_err(|_| io_refusal(out))? {
        let entry = entry.map_err(|_| io_refusal(out))?;
        let path = entry.path();
        if printed_page(&path) && entry.file_type().map_err(|_| io_refusal(&path))?.is_file() {
            fs::remove_file(&path).map_err(|_| io_refusal(&path))?;
        }
    }
    for page in pages {
        let path = out.join(page.name);
        fs::write(&path, page.document).map_err(|_| io_refusal(&path))?;
    }
    Ok(())
}

fn printed_page(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".html"))
        .is_some_and(|number| number.len() >= 2 && number.bytes().all(|byte| byte.is_ascii_digit()))
}

fn io_refusal(path: &Path) -> Refusal {
    Refusal::Io { path: path.to_owned() }
}

#[cfg(test)]
mod tests;
