//! One report renderer for shells and worlds (protocol/binaries.md, upgrade).
use jig_protocol_store::{BackRefusal, Event, KeptWire, Owner};
use std::fmt::Write;
use temper_service::upgrade::Report;
use temper_shell::directories::Directories;

#[must_use]
pub const fn owner(of: Owner) -> &'static str {
    match of { Owner::Skein => "skein", Owner::Smith => "smith", Owner::Jig => "jig", Owner::Application => "temper" }
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
            Event::Rehearsing { wires, fresh: marked, .. } => { rehearsal = true; kept = wires.as_deref(); fresh = *marked; }
            Event::Patched { of, patch, counts, .. } => {
                writeln!(text, "{binary} upgrade: patched {} {patch}: {} records in {} commits; time unavailable", owner(*of), counts.records, counts.commits).expect("string write");
            }
            Event::Back { to, .. } => {
                passed = true;
                writeln!(text, "{binary} upgrade: generation {to} active again; start the old binary").expect("string write");
            }
            Event::BackRefused { why, .. } => {
                let words = match why {
                    BackRefusal::Committed { .. } => "the patched generation is past its first commit; going back is restoring a backup",
                    BackRefusal::NoPrevious => "no previous generation; restore a backup to go back",
                    BackRefusal::Busy => "a process holds the deployment; stop it before going back",
                    BackRefusal::Unreadable => "the deployment could not be read; check its directory and permissions, or restore a backup",
                };
                writeln!(text, "{binary} upgrade: {words}").expect("string write");
            }
            Event::Closed { removed: false, .. } => {
                passed = false;
                writeln!(text, "{binary} upgrade: scratch cleanup failed; check the cache directory").expect("string write");
            }
            Event::Closed { removed: true, .. }
            | Event::Opened { .. } | Event::OpenRefused { .. } | Event::Switched { .. }
            | Event::Site { .. } | Event::Committed { .. } | Event::Failed { .. }
            | Event::FrameTooLarge { .. } | Event::Page { .. } | Event::PageRefused { .. }
            | Event::Loaded { .. } | Event::Read { .. } | Event::Got { .. } | Event::Turns { .. }
            | Event::Unread { .. } | Event::Corrupt { .. } | Event::SecretsNotReady { .. }
            | Event::Kept { .. } | Event::Stale { .. } | Event::Erased { .. } | Event::Carried { .. }
            | Event::Secret { .. } | Event::SecretsLoaded { .. } => {}
        }
    }
    writeln!(text, "{binary} upgrade: whole rehearsal/tool time: {} ns", report.elapsed.as_nanos()).expect("string write");
    if let Some(end) = &report.end {
        let (refusal, _) = crate::host::ending_with_binary(binary, Some(end), false, directories, deployment, &[]);
        text.push_str(&refusal);
    } else if report.restored {
        writeln!(text, "{binary} upgrade: patches and restores pass; what follows the switch is not rehearsed").expect("string write");
    }
    if rehearsal || report.end.is_some() { workers(&mut text, binary, kept, fresh); }
    (text, u8::from(!passed || !report.done))
}


fn workers(text: &mut String, binary: &str, kept: Option<&[KeptWire]>, fresh: bool) {
    let Some(kept) = kept else {
        writeln!(text, "{binary} upgrade: workers may stay: unknown; the copy keeps no wires").expect("string write");
        return;
    };
    let wires = &temper_service::store::WIRES;
    let equal = kept.len() == wires.len() && wires.iter().all(|wire| kept.iter().any(|old| old.name.as_ref() == wire.name && old.version == wire.version));
    writeln!(text, "{binary} upgrade: workers may stay: {}", if equal && !fresh { "yes" } else { "no" }).expect("string write");
    if fresh { writeln!(text, "{binary} upgrade: a pending patch requires fresh workers; stop the workers").expect("string write"); }
    for wire in wires {
        match kept.iter().find(|old| old.name.as_ref() == wire.name) {
            Some(old) if old.version == wire.version => {}
            Some(old) => writeln!(text, "{binary} upgrade: wire {} differs: kept {}, binary {}", wire.name, old.version, wire.version).expect("string write"),
            None => writeln!(text, "{binary} upgrade: wire {} is not kept by the copy", wire.name).expect("string write"),
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
        let mut events = Queue::uncapped(); events.push(event);
        Report { events, elapsed: Duration::from_secs(1), end: None, restored: false, done: true }
    }
    #[test]
    fn report_words_cover_workers_back_and_patched_starts() {
        let dirs = Directories { config: b"/config".as_slice().into(), data: b"/data".as_slice().into(), cache: b"/cache".as_slice().into(), origins: [skein_shell::Origin::Flag; 3] };
        let wires = || temper_service::store::WIRES.iter().map(|wire| KeptWire { name: wire.name.into(), version: wire.version }).collect::<Vec<_>>().into_boxed_slice();
        for (kept, fresh, words) in [
            (Some(wires()), false, "workers may stay: yes"),
            (Some(wires()), true, "workers may stay: no"),
            (None, false, "workers may stay: unknown"),
            (Some(Box::new([KeptWire { name: "jig-channel".into(), version: 0 }]) as Box<[_]>), false, "wire jig-channel differs: kept 0, binary"),
        ] {
            let mut report = report(Event::Rehearsing { owner: Token::new(1), wires: kept, fresh });
            report.restored = true;
            let (text, code) = lines("temper-engine", &report, &dirs, b"/backup");
            assert_eq!(code, 0, "{text}");
            assert!(text.contains(words), "{text}");
        }
        for (why, remedy) in [
            (BackRefusal::Committed { manifest: 1, last: 2 }, "past its first commit; going back is restoring a backup"),
            (BackRefusal::NoPrevious, "no previous generation; restore a backup"),
            (BackRefusal::Busy, "stop it before going back"),
            (BackRefusal::Unreadable, "check its directory and permissions, or restore a backup"),
        ] {
            let (text, code) = lines("temper-box", &report(Event::BackRefused { owner: Token::new(1), why }), &dirs, b"/backup");
            assert_eq!(code, 1);
            assert!(text.contains(remedy), "{text}");
        }
        for binary in ["temper-box", "temper-engine"] {
            assert_eq!(crate::host::note_line(binary, temper_service::Note::Patched {
                of: Owner::Application, patch: 7, counts: jig_protocol_store::Counts { records: 9, commits: 3 },
            }), format!("{binary} serve: patched temper 7: 9 records in 3 commits\n"));
        }
    }
}
