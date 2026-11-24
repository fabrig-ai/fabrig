//! One report renderer for shells and worlds (protocol/binaries.md, upgrade).
use jig_protocol_store::{
    BackRefusal, Event, KeptMap, KeptOpening, KeptWire, KeptWorker, Owner, Undecoded, WorkerOpening,
};
#[cfg(test)]
#[path = "report/tests.rs"]
mod decode_tests;
use jig_protocol_engine_workers::{Stated, Statement, Unfit, judge};
use skein_lib::{List, Wall};
use std::{collections::BTreeMap, fmt::Write};
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

/// A job's declared counts, including the separator when it counts anything.
pub(crate) fn job_counts(counts: &[jig_protocol_store::JobCount]) -> String {
    let mut text = String::new();
    let mut separator = "; ";
    for count in counts {
        write!(text, "{separator}{} {}", count.name, count.count).expect("string write");
        separator = ", ";
    }
    text
}

/// Store progress, then the start's outcome and worker compatibility.
#[must_use]
pub fn lines(binary: &str, report: &Report, directories: &Directories, deployment: &[u8]) -> (String, u8) {
    let mut text = String::new();
    let mut passed = report.restored;
    for event in &report.events {
        passed = event_refusal(&mut text, binary, event, passed);
        if let Event::Decoded { records, .. } = event {
            writeln!(text, "{binary} upgrade: decoded {records} records").expect("string write");
        } else if let Event::Ran { of, job, rows, commits, counts, .. } = event {
            writeln!(
                text,
                "{binary} upgrade: ran {} {job}: {rows} rows in {commits} commits{}; time unavailable",
                owner(*of),
                job_counts(counts)
            )
            .expect("string write");
        }
    }
    writeln!(text, "{binary} upgrade: whole rehearsal/tool time: {} ns", report.elapsed.as_nanos())
        .expect("string write");
    refusal(&mut text, binary, report, directories, deployment);
    if report.end.is_none() && report.restored {
        writeln!(
            text,
            "{binary} upgrade: registry commit, jobs and restores pass; opened in {} ms",
            report.elapsed.as_millis()
        )
        .expect("string write");
        for (forge, state) in &report.catching_up {
            writeln!(text, "{binary} upgrade: forge {forge} would catch up: {}", catching_up(*state))
                .expect("string write");
        }
    }
    verdict(&mut text, binary, report, false);
    (text, u8::from(!passed || !report.done))
}

fn catching_up(state: Option<temper_engine_domain_forge::CatchingUp>) -> String {
    let mut text = String::new();
    if let Some(state) = state {
        for (count, singular, plural) in [
            (state.fresh, "fresh read", "fresh reads"),
            (state.gathers, "landing read", "landing reads"),
            (state.ci, "CI read", "CI reads"),
            (state.goal_files, "goal-file read", "goal-file reads"),
            (state.lost_writers, "lost-writer read", "lost-writer reads"),
            (state.goal_branches, "goal-branch read", "goal-branch reads"),
            (state.uncertain, "uncertain write", "uncertain writes"),
        ] {
            if count != 0 {
                if !text.is_empty() {
                    text.push_str(", ");
                }
                write!(text, "{count} {}", if count == 1 { singular } else { plural }).expect("string write");
            }
        }
    }
    if text.is_empty() {
        text.push_str("nothing");
    }
    text
}

/// Rehearsal file counts and the workers' verdict, or the whole check's opening refusal.
#[must_use]
pub fn workers_lines(binary: &str, report: &Report, directories: &Directories, deployment: &[u8]) -> (String, u8) {
    let mut text = String::new();
    let mut passed = true;
    for event in &report.events {
        passed = event_refusal(&mut text, binary, event, passed);
    }
    passed &= refusal(&mut text, binary, report, directories, deployment);
    let rehearsal = verdict(&mut text, binary, report, true);
    (text, u8::from(!passed || !report.done || !rehearsal))
}

// Event refusals stay interleaved with the whole check's progress.
fn event_refusal(text: &mut String, binary: &str, event: &Event, mut passed: bool) -> bool {
    match event {
        Event::Undecodable { family, version, at, .. } => {
            passed = false;
            write!(text, "{binary} upgrade: record not decodable: {family} {version}, ").expect("string write");
            place(text, at);
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
        // Upgrade progress does not consume the census (jig jig/docs/design/protocol/store/growth.md, 1.3).
        Event::Rehearsing { .. }
        | Event::Decoded { .. }
        | Event::Ran { .. }
        | Event::Checkpointed { .. }
        | Event::CheckpointRefused { .. }
        | Event::Restored { .. }
        | Event::RestoreRefused { .. }
        | Event::CheckpointHeld { .. }
        | Event::CheckpointRemoved { .. }
        | Event::Census { .. }
        | Event::Closed { removed: true, .. }
        | Event::Opened { .. }
        | Event::OpenRefused { .. }
        | Event::Switched { .. }
        | Event::Job { .. }
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
    passed
}

fn refusal(text: &mut String, binary: &str, report: &Report, directories: &Directories, deployment: &[u8]) -> bool {
    if let Some(failure) = &report.open_failure {
        writeln!(text, "{binary} upgrade: store opening failed: {failure:?}").expect("string write");
    }
    if let Some(end) = &report.end {
        let (refusal, _) = crate::lines::ending_with_binary(binary, Some(end), false, directories, deployment, &[]);
        text.push_str(&refusal);
    }
    report.open_failure.is_none() && report.end.is_none()
}

fn verdict(text: &mut String, binary: &str, report: &Report, count_files: bool) -> bool {
    let mut kept = None;
    let mut openings = &[][..];
    let mut counts = None;
    let mut workers_kept = None;
    for event in &report.events {
        if let Event::Rehearsing { wires, openings: stated, workers, linked, copied, .. } = event {
            counts = Some((*linked, *copied));
            kept = wires.as_deref();
            openings = stated;
            workers_kept = workers.as_deref();
        }
    }
    if count_files && let Some((linked, copied)) = counts {
        writeln!(text, "{binary} upgrade: rehearsal linked {linked} files, copied {copied}").expect("string write");
    }
    if counts.is_some() || report.end.is_some() {
        workers(text, binary, kept, openings, workers_kept, &report.workers);
    }
    counts.is_some()
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

fn workers(
    text: &mut String,
    binary: &str,
    kept: Option<&[KeptWire]>,
    openings: &[KeptOpening],
    workers_kept: Option<&[KeptWorker]>,
    configured: &Workers,
) {
    let Some(kept) = kept else {
        writeln!(text, "{binary} upgrade: workers may stay: unknown; the copy keeps no wires").expect("string write");
        return;
    };
    let mut allowed = host_tools(text, binary, kept);
    let Some(workers_kept) = workers_kept else {
        writeln!(text, "{binary} upgrade: workers may stay: unknown; the copy's workers cannot be read")
            .expect("string write");
        return;
    };
    // Settings changes replace defaults by number; a removal masks its default.
    let current: BTreeMap<_, _> = configured.defaults.iter().chain(workers_kept).map(|w| (w.number, w)).collect();
    let by_name: BTreeMap<_, _> = openings
        .iter()
        .map(|opening| {
            let name = match opening {
                KeptOpening::Current(current) => &current.worker,
                KeptOpening::BeforeLayouts { worker } => worker,
            };
            (name.as_bytes(), opening)
        })
        .collect();
    for worker in current.values().filter(|worker| !worker.withdrawn) {
        let name = String::from_utf8_lossy(&worker.name);
        let opening = match by_name.get(worker.name.as_ref()) {
            Some(KeptOpening::Current(opening)) => opening,
            Some(KeptOpening::BeforeLayouts { .. }) => {
                allowed = false;
                writeln!(text, "{binary} upgrade: worker {name} unknown: opened before layouts").expect("string write");
                continue;
            }
            None => {
                allowed = false;
                writeln!(
                    text,
                    "{binary} upgrade: worker {name} unknown: no opening kept since the engine's last start"
                )
                .expect("string write");
                continue;
            }
        };
        match judge(&configured.spoken, &configured.roots, configured.room, &configured.limits, &statement(opening)) {
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

fn host_tools(text: &mut String, binary: &str, kept: &[KeptWire]) -> bool {
    let wire = temper_engine_service::store::host_tools_wire();
    match kept.iter().find(|old| old.name.as_ref() == wire.name) {
        Some(old) if (u32::from(wire.lowest)..=u32::from(wire.highest())).contains(&old.version) => true,
        Some(old) => {
            writeln!(
                text,
                "{binary} upgrade: wire {}: kept {}, this binary speaks {}..{}",
                wire.name,
                old.version,
                wire.lowest,
                wire.highest()
            )
            .expect("string write");
            false
        }
        None => {
            writeln!(text, "{binary} upgrade: wire {} is not kept by the copy", wire.name).expect("string write");
            false
        }
    }
}

fn statement(opening: &WorkerOpening) -> Statement {
    Statement {
        wires: List::from_boxed(
            opening
                .wires
                .iter()
                .map(|wire| Stated { name: wire.name.clone(), layout: wire.layout, text: wire.text.clone() })
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
            checkpoint: None,
            workers: decode_tests::configured(),
            events,
            elapsed: Duration::from_secs(1),
            end: None,
            restored: false,
            catching_up: Box::new([]),
            done: true,
            open_failure: None,
        }
    }
    #[test]
    fn ran_report_counts_precede_time_or_leave_the_line_unchanged() {
        use jig_protocol_store::JobCount;
        let dirs = Directories {
            config: b"/config".as_slice().into(),
            data: b"/data".as_slice().into(),
            cache: b"/cache".as_slice().into(),
            origins: [skein_shell::Origin::Flag; 3],
        };
        for (counts, suffix) in [
            (vec![], ""),
            (
                vec![JobCount { name: "dropped".into(), count: 5 }, JobCount { name: "PullState".into(), count: 5 }],
                "; dropped 5, PullState 5",
            ),
        ] {
            let (text, _) = lines(
                "temper-box",
                &report(Event::Ran {
                    owner: Token::new(1),
                    of: Owner::Application,
                    job: "deleted-repository-forge-rows".into(),
                    rows: 812,
                    commits: 2,
                    counts: counts.into_boxed_slice(),
                }),
                &dirs,
                b"/backup",
            );
            assert_eq!(
                text.lines().next().unwrap(),
                format!(
                    "temper-box upgrade: ran temper deleted-repository-forge-rows: 812 rows in 2 commits{suffix}; time unavailable"
                )
            );
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
                    temper_engine_service::Note::Ran {
                        of: Owner::Application,
                        job: "fixture-job".into(),
                        rows: 9,
                        commits: 3,
                        counts: Box::new([]),
                    }
                ),
                format!("{shown} serve: ran temper fixture-job: 9 rows in 3 commits\n")
            );
        }
    }
}
