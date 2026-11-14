//! One report renderer for shells and worlds (protocol/binaries.md, upgrade).
use jig_protocol_store::{BackRefusal, Event, KeptMap, KeptWire, Owner, Undecoded, WorkerOpening};
#[cfg(test)]
#[path = "report/tests.rs"]
mod decode_tests;
use jig_protocol_engine_workers::{Stated, Statement, Unfit, judge};
use skein_lib::{List, Wall};
use std::fmt::Write;
use temper_engine_service::upgrade::{Report, Workers};
use temper_shell::directories::Directories;

#[must_use]
pub const fn owner(of: Owner) -> &'static str {
    match of {
        Owner::Skein => "skein",
        Owner::Smith => "smith",
        Owner::Jig => "jig",
        Owner::Application => "temper",
    }
}

/// Store progress, then the start's outcome and worker compatibility.
#[must_use]
pub fn lines(binary: &str, report: &Report, directories: &Directories, deployment: &[u8]) -> (String, u8) {
    let mut text = String::new();
    let mut kept = None;
    let mut openings = &[][..];
    let mut rehearsal = false;
    let mut fresh = false;
    let mut passed = report.restored;
    for event in &report.events {
        match event {
            Event::Rehearsing { wires, fresh: marked, openings: stated, .. } => {
                rehearsal = true;
                kept = wires.as_deref();
                fresh = *marked;
                openings = stated;
            }
            Event::Decoded { records, .. } => {
                writeln!(text, "{binary} upgrade: decoded {records} records").expect("string write");
            }
            Event::Undecodable { family, version, at, .. } => {
                passed = false;
                write!(text, "{binary} upgrade: record not decodable: {family} {version}, ").expect("string write");
                place(&mut text, at);
            }
            Event::Patched { of, patch, counts, .. } => {
                writeln!(
                    text,
                    "{binary} upgrade: patched {} {patch}: {} records in {} commits; time unavailable",
                    owner(*of),
                    counts.records,
                    counts.commits
                )
                .expect("string write");
            }
            Event::Back { to, .. } => {
                passed = true;
                writeln!(text, "{binary} upgrade: generation {to} active again; start the old binary")
                    .expect("string write");
            }
            Event::BackRefused { why, .. } => {
                let words = match why {
                    BackRefusal::Committed { .. } => {
                        "the patched generation is past its first commit; going back is restoring a backup"
                    }
                    BackRefusal::NoPrevious => "no previous generation; restore a backup to go back",
                    BackRefusal::Busy => "a process holds the deployment; stop it before going back",
                    BackRefusal::Unreadable => {
                        "the deployment could not be read; check its directory and permissions, or restore a backup"
                    }
                };
                writeln!(text, "{binary} upgrade: {words}").expect("string write");
            }
            Event::Closed { removed: false, .. } => {
                passed = false;
                writeln!(text, "{binary} upgrade: scratch cleanup failed; check the cache directory")
                    .expect("string write");
            }
            // Upgrade progress does not consume the census (jig protocol/store/README.md, 6.3).
            Event::Census { .. }
            | Event::Closed { removed: true, .. }
            | Event::Opened { .. }
            | Event::OpenRefused { .. }
            | Event::Switched { .. }
            | Event::Site { .. }
            | Event::Committed { .. }
            | Event::Failed { .. }
            | Event::FrameTooLarge { .. }
            | Event::Page { .. }
            | Event::PageRefused { .. }
            | Event::Loaded { .. }
            | Event::Read { .. }
            | Event::Got { .. }
            | Event::Turns { .. }
            | Event::Unread { .. }
            | Event::Corrupt { .. }
            | Event::SecretsNotReady { .. }
            | Event::Kept { .. }
            | Event::Stale { .. }
            | Event::Erased { .. }
            | Event::Carried { .. }
            | Event::Secret { .. }
            | Event::SecretsLoaded { .. } => {}
        }
    }
    writeln!(text, "{binary} upgrade: whole rehearsal/tool time: {} ns", report.elapsed.as_nanos())
        .expect("string write");
    if let Some(failure) = &report.open_failure {
        writeln!(text, "{binary} upgrade: store opening failed: {failure:?}").expect("string write");
    }
    if let Some(end) = &report.end {
        let (refusal, _) = crate::lines::ending_with_binary(binary, Some(end), false, directories, deployment, &[]);
        text.push_str(&refusal);
    } else if report.restored {
        writeln!(text, "{binary} upgrade: patches and restores pass; what follows the switch is not rehearsed")
            .expect("string write");
    }
    if rehearsal || report.end.is_some() {
        workers(&mut text, binary, kept, fresh, openings, &report.workers);
    }
    (text, u8::from(!passed || !report.done))
}

fn place(text: &mut String, at: &Undecoded) {
    match at {
        Undecoded::Record { map, key } => {
            let map = match map {
                KeptMap::Store => "store",
                KeptMap::Secrets => "secrets",
            };
            write!(text, "{map} key ").expect("string write");
            for byte in key {
                write!(text, "{byte:02x}").expect("string write");
            }
            text.push('\n');
        }
        Undecoded::Frame { transcript, offset } => {
            writeln!(text, "transcript {transcript} at {offset}").expect("string write");
        }
    }
}

// The table's worker-facing entries are precisely jig's four and temper's worker wire.
fn worker_wire(name: &str) -> bool {
    name == temper_protocol_worker_wire::wire::FAMILY
        || jig_protocol_worker_wire::spoken().iter().any(|wire| wire.name == name)
}

fn workers(
    text: &mut String,
    binary: &str,
    kept: Option<&[KeptWire]>,
    fresh: bool,
    openings: &[WorkerOpening],
    configured: &Workers,
) {
    if fresh {
        writeln!(text, "{binary} upgrade: a pending patch requires fresh workers; stop the workers")
            .expect("string write");
        writeln!(text, "{binary} upgrade: workers may stay: no").expect("string write");
        return;
    }
    let Some(kept) = kept else {
        writeln!(text, "{binary} upgrade: workers may stay: unknown; the copy keeps no wires").expect("string write");
        return;
    };
    let (spoken, mut allowed) = engine_wires(text, binary, kept);
    for name in &configured.names {
        let name = String::from_utf8_lossy(name);
        let Some(opening) = openings.iter().find(|opening| opening.worker.as_ref() == name) else {
            allowed = false;
            writeln!(text, "{binary} upgrade: worker {name} unknown: no opening kept since the engine's last start")
                .expect("string write");
            continue;
        };
        match judge(&spoken, configured.room, &configured.schema, &statement(opening)) {
            Ok(_) => writeln!(text, "{binary} upgrade: worker {name} may stay: opened {}", at(opening.at))
                .expect("string write"),
            Err(unfit) => {
                allowed = false;
                let (kind, words) = match unfit {
                    Unfit::Wires(words) => ("wires", words),
                    Unfit::Limits(words) => ("limits", words),
                };
                writeln!(
                    text,
                    "{binary} upgrade: worker {name} may not stay: {kind}: {}",
                    String::from_utf8_lossy(&words)
                )
                .expect("string write");
            }
        }
    }
    writeln!(text, "{binary} upgrade: workers may stay: {}", if allowed { "yes" } else { "no" }).expect("string write");
}

fn engine_wires(text: &mut String, binary: &str, kept: &[KeptWire]) -> (List<jig_protocol_worker_wire::Spoken>, bool) {
    let mut allowed = true;
    let mut spoken = List::with_capacity(temper_engine_service::wires().len());
    for wire in temper_engine_service::wires().into_boxed() {
        if worker_wire(wire.name) {
            spoken.push(wire).expect("table capacity");
            continue;
        }
        match kept.iter().find(|old| old.name.as_ref() == wire.name) {
            Some(old) if (u32::from(wire.lowest)..=u32::from(wire.highest())).contains(&old.version) => {}
            Some(old) => {
                allowed = false;
                writeln!(
                    text,
                    "{binary} upgrade: wire {}: kept {}, this binary speaks {}..{}",
                    wire.name,
                    old.version,
                    wire.lowest,
                    wire.highest()
                )
                .expect("string write");
            }
            None => {
                allowed = false;
                writeln!(text, "{binary} upgrade: wire {} is not kept by the copy", wire.name).expect("string write");
            }
        }
    }
    (spoken, allowed)
}

fn statement(opening: &WorkerOpening) -> Statement {
    Statement {
        wires: List::from_boxed(
            opening
                .wires
                .iter()
                .map(|wire| Stated {
                    name: wire.name.clone(),
                    lowest: wire.lowest,
                    layouts: List::from_boxed(wire.layouts.clone()),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
        output: opening.output,
        terms: List::from_boxed(
            opening
                .terms
                .iter()
                .map(|term| skein_channel::Term { kind: term.kind, largest: term.largest })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
    }
}

fn at(wall: Wall) -> String {
    let date = wall.civil();
    let seconds = wall.as_secs() % 86_400;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        date.year,
        date.month,
        date.day,
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::{Duration, Queue, Token};
    pub(super) fn report(event: Event) -> Report {
        let mut events = Queue::uncapped();
        events.push(event);
        Report {
            workers: decode_tests::configured(&[]),
            events,
            elapsed: Duration::from_secs(1),
            end: None,
            restored: false,
            done: true,
            open_failure: None,
        }
    }
    #[test]
    fn report_words_cover_back_and_patched_starts() {
        let dirs = Directories {
            config: b"/config".as_slice().into(),
            data: b"/data".as_slice().into(),
            cache: b"/cache".as_slice().into(),
            origins: [skein_shell::Origin::Flag; 3],
        };
        for (why, remedy) in [
            (
                BackRefusal::Committed { manifest: 1, last: 2 },
                "past its first commit; going back is restoring a backup",
            ),
            (BackRefusal::NoPrevious, "no previous generation; restore a backup"),
            (BackRefusal::Busy, "stop it before going back"),
            (BackRefusal::Unreadable, "check its directory and permissions, or restore a backup"),
        ] {
            let (text, code) =
                lines("temper-box", &report(Event::BackRefused { owner: Token::new(1), why }), &dirs, b"/backup");
            assert_eq!(code, 1);
            assert!(text.contains(remedy), "{text}");
        }
        for (binary, shown) in
            [("temper-box", "temper-box"), ("temper-engine", "temper-engine"), ("temper", "temper-box")]
        {
            assert_eq!(
                crate::lines::note_line(
                    binary,
                    temper_engine_service::Note::Patched {
                        of: Owner::Application,
                        patch: 7,
                        counts: jig_protocol_store::Counts { records: 9, commits: 3 },
                    }
                ),
                format!("{shown} serve: patched temper 7: 9 records in 3 commits\n")
            );
        }
    }
}
