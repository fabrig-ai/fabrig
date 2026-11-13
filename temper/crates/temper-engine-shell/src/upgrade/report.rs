//! One report renderer for shells and worlds (protocol/binaries.md, upgrade).
use jig_protocol_store::{BackRefusal, Event, KeptMap, KeptWire, Owner, Undecoded};
#[cfg(test)]
#[path = "report/tests.rs"]
mod decode_tests;
use std::fmt::Write;
use temper_engine_service::upgrade::Report;
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
    let mut rehearsal = false;
    let mut fresh = false;
    let mut passed = report.restored;
    for event in &report.events {
        match event {
            Event::Rehearsing { wires, fresh: marked, .. } => {
                rehearsal = true;
                kept = wires.as_deref();
                fresh = *marked;
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
            Event::Closed { removed: true, .. }
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
        workers(&mut text, binary, kept, fresh);
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

fn workers(text: &mut String, binary: &str, kept: Option<&[KeptWire]>, fresh: bool) {
    let Some(kept) = kept else {
        writeln!(text, "{binary} upgrade: workers may stay: unknown; the copy keeps no wires").expect("string write");
        return;
    };
    let wires = temper_engine_service::wires();
    let equal = kept.len() == wires.as_slice().len()
        && wires.iter().all(|wire| {
            kept.iter().any(|old| old.name.as_ref() == wire.name && old.version == u32::from(wire.highest()))
        });
    writeln!(text, "{binary} upgrade: workers may stay: {}", if equal && !fresh { "yes" } else { "no" })
        .expect("string write");
    if fresh {
        writeln!(text, "{binary} upgrade: a pending patch requires fresh workers; stop the workers")
            .expect("string write");
    }
    for wire in &wires {
        match kept.iter().find(|old| old.name.as_ref() == wire.name) {
            Some(old) if old.version == u32::from(wire.highest()) => {}
            Some(old) => writeln!(
                text,
                "{binary} upgrade: wire {} differs: kept {}, binary {}",
                wire.name,
                old.version,
                wire.highest()
            )
            .expect("string write"),
            None => {
                writeln!(text, "{binary} upgrade: wire {} is not kept by the copy", wire.name).expect("string write");
            }
        }
    }
    for old in kept.iter().filter(|old| !wires.iter().any(|wire| wire.name == old.name.as_ref())) {
        writeln!(text, "{binary} upgrade: wire {} is not spoken by this binary", old.name).expect("string write");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::{Duration, Queue, Token};
    fn report(event: Event) -> Report {
        let mut events = Queue::uncapped();
        events.push(event);
        Report { events, elapsed: Duration::from_secs(1), end: None, restored: false, done: true, open_failure: None }
    }
    #[test]
    fn report_words_cover_workers_back_and_patched_starts() {
        let dirs = Directories {
            config: b"/config".as_slice().into(),
            data: b"/data".as_slice().into(),
            cache: b"/cache".as_slice().into(),
            origins: [skein_shell::Origin::Flag; 3],
        };
        let wires = || {
            temper_engine_service::wires()
                .iter()
                .map(|wire| KeptWire { name: wire.name.into(), version: u32::from(wire.highest()) })
                .collect::<Vec<_>>()
                .into_boxed_slice()
        };
        for (kept, fresh, words) in [
            (Some(wires()), false, "workers may stay: yes"),
            (Some(wires()), true, "workers may stay: no"),
            (None, false, "workers may stay: unknown"),
            (
                Some(Box::new([
                    KeptWire { name: "jig-channel".into(), version: 10 },
                    KeptWire { name: "smith-charter".into(), version: 4 },
                    KeptWire { name: "smith-turn".into(), version: 7 },
                    KeptWire { name: "temper-payload".into(), version: 5 },
                ]) as Box<[_]>),
                false,
                "wire jig-channel is not spoken by this binary",
            ),
            (
                Some(Box::new([KeptWire { name: "jig-worker-wire".into(), version: 0 }]) as Box<[_]>),
                false,
                "wire jig-worker-wire differs: kept 0, binary",
            ),
        ] {
            let mut report = report(Event::Rehearsing { owner: Token::new(1), wires: kept, fresh });
            report.events.push(Event::Decoded { owner: Token::new(1), records: 17 });
            report.restored = true;
            let (text, code) = lines("temper-engine", &report, &dirs, b"/backup");
            assert_eq!(code, 0, "{text}");
            assert!(text.contains("decoded 17 records\n"), "{text}");
            assert!(text.contains(words), "{text}");
            if words.contains("jig-channel") {
                assert!(text.contains("workers may stay: no"), "{text}");
                assert!(text.contains("wire jig-worker-wire is not kept by the copy"), "{text}");
            }
        }
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
