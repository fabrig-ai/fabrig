//! The binaries' shared --check report words (protocol/binaries.md).
use super::*;
use skein_lib::{Duration, Queue, Token};

#[test]
fn check_lines_name_decoded_count_and_undecodable_places() {
    let directories =
        temper_shell::directories::resolve(&[], &[(b"HOME".as_slice().into(), b"/home/test".as_slice().into())], b"/")
            .expect("directories")
            .0;
    for binary in ["temper", "temper-box", "temper-engine"] {
        let mut report = Report {
            checkpoint: None,
            workers: configured(),
            events: Queue::uncapped(),
            elapsed: Duration::ZERO,
            end: None,
            restored: true,
            catching_up: Box::new([]),
            done: true,
            open_failure: None,
        };
        report.events.push(Event::Decoded { owner: Token::new(1), records: 17 });
        let (text, code) = lines(binary, &report, &directories, b"/state");
        assert_eq!(code, 0);
        assert!(text.starts_with(&format!("{binary} upgrade: decoded 17 records\n")), "{text}");
        for (at, place) in [
            (Undecoded::Record { map: KeptMap::Store, key: Box::new([0, 0xab, 0xff]) }, "store key 00abff"),
            (Undecoded::Record { map: KeptMap::Secrets, key: Box::new([2, 0, 1]) }, "secrets key 020001"),
            (Undecoded::Frame { transcript: "7".into(), offset: 256 }, "transcript 7 at 256"),
        ] {
            let mut report = Report {
                checkpoint: None,
                workers: configured(),
                events: Queue::uncapped(),
                elapsed: Duration::ZERO,
                end: None,
                restored: false,
                catching_up: Box::new([]),
                done: true,
                open_failure: None,
            };
            report.events.push(Event::Undecodable {
                owner: Token::new(1),
                family: "temper-store".into(),
                version: 11,
                at,
            });
            report.events.push(Event::Closed { owner: Token::new(1), removed: true });
            let (text, code) = lines(binary, &report, &directories, b"/state");
            assert_eq!(code, 1);
            assert!(
                text.starts_with(&format!("{binary} upgrade: record not decodable: temper-store 11, {place}\n")),
                "{text}"
            );
            assert!(!text.contains("registry commit, jobs and restores pass"));
        }
    }
}

#[test]
fn opening_report_names_elapsed_milliseconds_and_each_forges_remaining_work() {
    use temper_engine_domain_forge::CatchingUp;
    let mut report = rehearsal();
    report.restored = true;
    report.elapsed = Duration::from_nanos(42_999_999);
    report.catching_up = Box::new([
        (
            1,
            Some(CatchingUp {
                fresh: 3,
                gathers: 2,
                ci: 1,
                goal_files: 1,
                lost_writers: 1,
                goal_branches: 6,
                uncertain: 2,
            }),
        ),
        (
            2,
            Some(CatchingUp {
                fresh: 0,
                gathers: 0,
                ci: 0,
                goal_files: 0,
                lost_writers: 0,
                goal_branches: 0,
                uncertain: 1,
            }),
        ),
        (
            3,
            Some(CatchingUp {
                fresh: 0,
                gathers: 0,
                ci: 0,
                goal_files: 0,
                lost_writers: 0,
                goal_branches: 0,
                uncertain: 0,
            }),
        ),
        (4, None),
    ]);
    let (text, code) = lines("temper-box", &report, &directories(), b"/checkpoint");
    assert_eq!(code, 0);
    assert_eq!(
        text,
        format!(
            "temper-box upgrade: whole rehearsal/tool time: 42999999 ns\n\
         temper-box upgrade: registry commit, jobs and restores pass; opened in 42 ms\n\
         temper-box upgrade: forge 1 would catch up: 3 fresh reads, 2 landing reads, 1 CI read, 1 goal-file read, 1 lost-writer read, 6 goal-branch reads, 2 uncertain writes\n\
         temper-box upgrade: forge 2 would catch up: 1 uncertain write\n\
         temper-box upgrade: forge 3 would catch up: nothing\n\
         temper-box upgrade: forge 4 would catch up: nothing\n{COMPATIBLE}"
        )
    );
    report.restored = false;
    let (text, code) = lines("temper-box", &report, &directories(), b"/checkpoint");
    assert_eq!(code, 1);
    assert!(!text.contains("opened in") && !text.contains("would catch up"), "{text}");
}

#[test]
fn workers_lines_count_files_before_the_checks_verdict_for_yes_no_and_unknown() {
    let dirs = directories();
    for verdict in ["yes", "no", "unknown"] {
        let mut w1 = opening("w1");
        let mut w2 = opening("w2");
        w1.at = Wall::from_nanos(1_794_968_442_000_000_000);
        w2.at = w1.at;
        if verdict == "no" {
            w1.output = 0;
        }
        let mut report = super::tests::report(Event::Rehearsing {
            owner: Token::new(1),
            wires: if verdict == "unknown" { None } else { Some(kept()) },
            openings: Box::new([w1, w2]),
            linked: 12,
            copied: 3,
        });
        report.restored = true;
        report.events.push(Event::Decoded { owner: Token::new(1), records: 123 });
        report.events.push(Event::Ran {
            owner: Token::new(1),
            of: Owner::Application,
            job: "fixture-job".into(),
            rows: 2,
            commits: 1,
            counts: Box::new([]),
        });
        report.events.push(Event::Closed { owner: Token::new(1), removed: true });
        for binary in ["temper-engine", "temper-box"] {
            let (check, check_code) = lines(binary, &report, &dirs, b"/checkpoint");
            assert_eq!(check_code, 0);
            assert!(!check.contains("upgrade: rehearsal linked "), "{check}");
            let counts = format!("{binary} upgrade: rehearsal linked 12 files, copied 3\n");
            let expected = check
                .split_inclusive('\n')
                .filter(|line| {
                    !line.contains("decoded ")
                        && !line.contains("ran temper ")
                        && !line.contains("whole rehearsal/tool time")
                        && !line.contains("registry commit")
                })
                .collect::<String>();
            let (text, code) = workers_lines(binary, &report, &dirs, b"/checkpoint");
            assert_eq!(text, format!("{counts}{expected}"));
            assert_eq!(code, 0, "{verdict} changes no exit code");
            if verdict == "yes" {
                assert_eq!(
                    text,
                    format!(
                        "{counts}{binary} upgrade: worker w1 may stay: opened 2026-11-18T02:20:42Z\n{binary} upgrade: worker w2 may stay: opened 2026-11-18T02:20:42Z\n{binary} upgrade: workers may stay: yes\n"
                    )
                );
            } else if verdict == "unknown" {
                assert_eq!(
                    text,
                    format!("{counts}{binary} upgrade: workers may stay: unknown; the copy keeps no wires\n")
                );
            } else {
                assert!(text.contains("worker w1 may not stay: limits:"), "{text}");
                assert!(text.ends_with("workers may stay: no\n"), "{text}");
            }
        }
    }
}

#[test]
fn workers_opening_refusals_keep_the_checks_words_and_fail() {
    let dirs = directories();
    for why in [jig_protocol_store::Generations::Busy, jig_protocol_store::Generations::Unfinished] {
        let why = jig_protocol_store::OpenRefusal::Generations { why };
        let mut report = super::tests::report(Event::OpenRefused { owner: Token::new(1), why: why.clone() });
        report.events.push(Event::Closed { owner: Token::new(1), removed: true });
        report.end = Some(temper_engine_service::End::Stopped(temper_engine_service::Stop::StoreOpen { why }));
        for binary in ["temper-engine", "temper-box"] {
            let (check, check_code) = lines(binary, &report, &dirs, b"/checkpoint");
            let (text, code) = workers_lines(binary, &report, &dirs, b"/checkpoint");
            assert_eq!(check_code, 1);
            assert_eq!(code, 1);
            assert_eq!(text, check.split_once('\n').expect("check timing line").1);
            assert!(text.contains("/checkpoint"), "{text}");
            assert!(!text.contains("upgrade: rehearsal linked "), "{text}");
        }
    }
}

#[test]
fn workers_lines_exclude_whole_rehearsal_progress() {
    let mut report = rehearsal();
    report.restored = true;
    report.events.push(Event::Decoded { owner: Token::new(1), records: 17 });
    report.events.push(Event::Ran {
        owner: Token::new(1),
        of: Owner::Application,
        job: "fixture-job".into(),
        rows: 2,
        commits: 1,
        counts: Box::new([]),
    });
    let dirs = directories();
    assert_eq!(workers_lines("temper-box", &report, &dirs, b"/checkpoint"), (format!("{COUNTS}{COMPATIBLE}"), 0));
    assert_eq!(
        lines("temper-box", &report, &dirs, b"/checkpoint"),
        (
            format!(
                "temper-box upgrade: decoded 17 records\n\
                 temper-box upgrade: ran temper fixture-job: 2 rows in 1 commits; time unavailable\n\
                 temper-box upgrade: whole rehearsal/tool time: 1000000000 ns\n\
                 temper-box upgrade: registry commit, jobs and restores pass; opened in 1000 ms\n\
                 {COMPATIBLE}"
            ),
            0,
        )
    );
}

#[test]
fn workers_lines_fail_without_rehearsal() {
    let mut report = super::tests::report(Event::Decoded { owner: Token::new(1), records: 17 });
    report.restored = true;
    assert_eq!(workers_lines("temper-box", &report, &directories(), b"/checkpoint"), (String::new(), 1));
}

#[test]
fn workers_event_refusals_keep_the_checks_words_and_fail() {
    let dirs = directories();
    for (event, refusal) in [
        (
            Event::Undecodable {
                owner: Token::new(1),
                family: "temper-store".into(),
                version: 11,
                at: Undecoded::Record { map: KeptMap::Store, key: Box::new([0xab]) },
            },
            "temper-box upgrade: record not decodable: temper-store 11, store key ab\n",
        ),
        (
            Event::Closed { owner: Token::new(1), removed: false },
            "temper-box upgrade: scratch cleanup failed; check the cache directory\n",
        ),
    ] {
        let mut report = rehearsal();
        report.events.push(Event::Decoded { owner: Token::new(1), records: 17 });
        report.events.push(event);
        let (check, check_code) = lines("temper-box", &report, &dirs, b"/checkpoint");
        assert_eq!(check_code, 1);
        assert_eq!(
            check,
            format!(
                "temper-box upgrade: decoded 17 records\n{refusal}\
                 temper-box upgrade: whole rehearsal/tool time: 1000000000 ns\n{COMPATIBLE}"
            )
        );
        assert_eq!(
            workers_lines("temper-box", &report, &dirs, b"/checkpoint"),
            (format!("{refusal}{COUNTS}{COMPATIBLE}"), 1)
        );
    }
}

#[test]
fn workers_open_failures_keep_the_checks_words_and_fail() {
    let mut report = rehearsal();
    report.open_failure =
        Some(jig_protocol_store::restore::OpenFailure::Refused(jig_protocol_store::OpenRefusal::Generations {
            why: jig_protocol_store::Generations::Busy,
        }));
    let dirs = directories();
    let refusal = "temper-box upgrade: store opening failed: Refused(Generations { why: Busy })\n";
    assert_eq!(
        lines("temper-box", &report, &dirs, b"/checkpoint"),
        (format!("temper-box upgrade: whole rehearsal/tool time: 1000000000 ns\n{refusal}{COMPATIBLE}"), 1)
    );
    assert_eq!(
        workers_lines("temper-box", &report, &dirs, b"/checkpoint"),
        (format!("{refusal}{COUNTS}{COMPATIBLE}"), 1)
    );
}

#[test]
fn checking_an_empty_deployment_names_the_missing_generation() {
    use jig_protocol_store::{Generations, OpenRefusal};
    use temper_engine_service::{End, Stop};

    let why = OpenRefusal::Generations { why: Generations::Unreadable };
    let mut report = super::tests::report(Event::OpenRefused { owner: Token::new(1), why: why.clone() });
    report.end = Some(End::Stopped(Stop::StoreOpen { why }));
    assert_eq!(
        lines("temper-engine", &report, &directories(), b"/state"),
        (concat!(
            "temper-engine upgrade: whole rehearsal/tool time: 1000000000 ns\n",
            "temper-engine serve: The `active` file in /state does not name a generation, and this start has changed nothing in it. ",
            "Restore /state from a backup, or write `active` again to name a generation under `generations/` that stands, and start again.\n",
            "temper-engine upgrade: workers may stay: unknown; the copy keeps no wires\n",
        ).into(), 1)
    );
}

#[test]
fn going_back_on_an_empty_deployment_names_the_unreadable_directory() {
    let report = super::tests::report(Event::BackRefused { owner: Token::new(1), why: BackRefusal::Unreadable });
    for binary in ["temper-engine", "temper-box"] {
        assert_eq!(
            lines(binary, &report, &directories(), b"/state"),
            (
                format!(
                    "{binary} upgrade: the deployment could not be read; check its directory and permissions, or restore a backup\n\
                 {binary} upgrade: whole rehearsal/tool time: 1000000000 ns\n"
                ),
                1
            )
        );
    }
}

#[test]
fn checking_and_starting_name_the_same_restore_refusals() {
    use jig_domain_core::{
        HeldLimit, Key,
        settings::Setting,
        start::{Failure, LiveSet, Numbered, Phase},
    };
    use jig_domain_people::{Key::Person, RestoreBound::IdentityBytes};
    use temper_engine_service::{End, Stop};

    // The simulated fixtures hold an eight-byte identity at a seven-byte limit,
    // and one version above the standard profile's sixteen.
    const HELD_IDENTITY: u64 = 8;
    const IDENTITY_LIMIT: u64 = 7;
    const PERSON: u64 = 1;
    const STOCK_VERSIONS: u32 = 16;
    const HELD_VERSIONS: u32 = STOCK_VERSIONS + 1;
    const FIRST_PAST_OWN: u32 = 65;
    const FIRST_STOCK_CHARTER: u32 = 1;
    let dirs = directories();
    for (failure, words) in [
        (
            Failure::LiveSet {
                set: LiveSet::PeopleBound(IdentityBytes),
                key: Key::People(Person(PERSON)),
                held: HELD_IDENTITY,
                limit: IDENTITY_LIMIT,
            },
            "PeopleBound(IdentityBytes), key People(Person(1)), held 8, limit 7",
        ),
        (
            Failure::Numbers {
                settings: Box::new([Numbered { setting: Setting::Charter(FIRST_PAST_OWN), changed: true }]),
            },
            "settings out of their numbers (the stock's 1 to 32, a deployment's own 33 to 64): changed charter 65",
        ),
        (
            Failure::Taken { settings: Box::new([Setting::Charter(FIRST_STOCK_CHARTER)]) },
            "the stock's new settings take numbers a deployment's own hold (the stock's 1 to 32, a deployment's own 33 to 64): charter 1; move each to 33 to 64 on the build that runs now, then start this one",
        ),
        (
            Failure::HeldPastLimit { which: HeldLimit::Versions, held: HELD_VERSIONS, limit: STOCK_VERSIONS },
            "Versions, held 17, limit 16",
        ),
    ] {
        let expected = format!("temper-engine serve: restart refused during core restore: {words}\n");
        let mut report = rehearsal();
        report.end = Some(End::Stopped(Stop::StartRefused { phase: Some(Phase::LoadCore), failure }));
        assert_eq!(
            crate::lines::ending_with_binary("temper-engine", report.end.as_ref(), false, &dirs, b"/state", &[]),
            (expected.clone(), 1)
        );
        let (text, exit) = lines("temper-engine", &report, &dirs, b"/state");
        assert_eq!(exit, 1);
        assert_eq!(
            text.split_inclusive('\n').filter(|line| line.contains("restart refused")).collect::<String>(),
            expected
        );
        assert!(!text.contains("registry commit, jobs and restores pass"), "{text}");
    }
}

fn directories() -> Directories {
    Directories {
        config: b"/config".as_slice().into(),
        data: b"/data".as_slice().into(),
        cache: b"/cache".as_slice().into(),
        origins: [skein_shell::Origin::Flag; 3],
    }
}

fn rehearsal() -> Report {
    super::tests::report(Event::Rehearsing {
        owner: Token::new(1),
        wires: Some(kept()),
        openings: Box::new([opening("w1"), opening("w2")]),
        linked: 12,
        copied: 3,
    })
}

pub(super) fn configured() -> Workers {
    let jig = jig_protocol_worker_wire::spoken();
    let application =
        temper_protocol_worker_wire::application(&temper_protocol_worker_wire::v1::CEILINGS).expect("application");
    Workers {
        room: 4096,
        limits: jig_protocol_worker_wire::CEILINGS,
        spoken: List::from_boxed(jig.iter().chain(&application.spoken).cloned().collect()),
        roots: application.roots,
    }
}

fn kept() -> Box<[KeptWire]> {
    temper_engine_service::wires()
        .iter()
        .map(|wire| KeptWire { name: wire.name.into(), version: u32::from(wire.highest()) })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn opening(name: &str) -> WorkerOpening {
    let configured = configured();
    let schema = jig_protocol_worker_wire::schema(&configured.limits).expect("fixture table");
    let table = schema.version(jig_protocol_worker_wire::VERSION).expect("current version");
    WorkerOpening {
        worker: name.into(),
        wires: configured
            .spoken
            .iter()
            .map(|wire| jig_protocol_store::OpenedWire {
                name: wire.name.into(),
                lowest: wire.version,
                layouts: Box::new([wire.layout]),
            })
            .collect(),
        output: configured.room,
        terms: table
            .kinds
            .iter()
            .filter(|kind| kind.direction == skein_channel::Direction::FromResponder)
            .map(|kind| jig_protocol_store::OpenedTerm { kind: kind.kind, largest: kind.largest })
            .collect(),
        at: Wall::from_nanos(1_794_564_000_123_456_789),
    }
}

fn verdict(wires: Option<Box<[KeptWire]>>, openings: Box<[WorkerOpening]>) -> String {
    let mut report =
        super::tests::report(Event::Rehearsing { owner: Token::new(1), wires, openings, linked: 12, copied: 3 });
    report.workers = configured();
    report.restored = true;
    let dirs =
        temper_shell::directories::resolve(&[], &[(b"HOME".as_slice().into(), b"/home/test".as_slice().into())], b"/")
            .expect("directories")
            .0;
    let (text, code) = lines("temper-box", &report, &dirs, b"/state");
    assert_eq!(code, 0, "worker compatibility changes no exit code");
    text.strip_prefix("temper-box upgrade: whole rehearsal/tool time: 1000000000 ns\ntemper-box upgrade: registry commit, jobs and restores pass; opened in 1000 ms\n").expect("other rehearsal lines unchanged").into()
}

const COUNTS: &str = "temper-box upgrade: rehearsal linked 12 files, copied 3\n";
const COMPATIBLE: &str = "temper-box upgrade: worker w1 may stay: opened 2026-11-13T10:00:00Z\ntemper-box upgrade: worker w2 may stay: opened 2026-11-13T10:00:00Z\ntemper-box upgrade: workers may stay: yes\n";
const NO: &str = "temper-box upgrade: workers may stay: no\n";

#[test]
fn compatible_openings_follow_the_stored_openings() {
    let mut wires = kept().into_vec();
    // Only the engine's own contracts are read from Wires, not a worker's wires.
    wires.retain(|wire| wire.name.as_ref() != "smith-charter");
    wires.push(KeptWire { name: "retired".into(), version: 999 });
    assert_eq!(verdict(Some(wires.into_boxed_slice()), Box::new([opening("w1"), opening("w2")])), COMPATIBLE);
    assert_eq!(
        verdict(Some(kept()), Box::new([opening("unconfigured")])),
        "temper-box upgrade: worker unconfigured may stay: opened 2026-11-13T10:00:00Z\ntemper-box upgrade: workers may stay: yes\n"
    );
}

#[test]
fn worker_refusals_keep_the_openings_wire_and_limits_words() {
    let mut layout = opening("w2");
    *layout
        .wires
        .iter_mut()
        .find(|wire| wire.name.as_ref() == "smith-charter")
        .expect("charter")
        .layouts
        .last_mut()
        .expect("current layout") = [0xcd; 32];
    let mut missing = opening("w2");
    missing.wires = missing.wires.into_iter().filter(|wire| wire.name.as_ref() != "smith-charter").collect();
    let mut output = opening("w1");
    output.output = 4095;
    let mut terms = opening("w2");
    terms.terms[0].largest = 7;
    for (opening, name, other, reason) in [
        (layout, "w2", "w1", "wires: wire smith-charter: the worker's text does not parse at line 1"),
        (missing, "w2", "w1", "wires: wire smith-charter: the worker states none"),
        (
            output,
            "w1",
            "w2",
            "limits: engine io.output 4096 bytes differs from worker io.output 4095 bytes; change io.output to agree",
        ),
        (terms, "w2", "w1", "limits: 384"),
    ] {
        // The failing worker precedes the compatible worker in stored opening order.
        assert_eq!(
            verdict(Some(kept()), Box::new([opening, self::opening(other)])),
            format!(
                "temper-box upgrade: worker {name} may not stay: {reason}\ntemper-box upgrade: worker {other} may stay: opened 2026-11-13T10:00:00Z\n{NO}"
            )
        );
    }
}

#[test]
fn no_kept_wires_makes_the_verdict_unknown_without_judging_workers() {
    assert_eq!(
        verdict(None, Box::new([opening("w1")])),
        "temper-box upgrade: workers may stay: unknown; the copy keeps no wires\n"
    );
}

#[test]
fn engine_contracts_must_be_kept_in_range_but_do_not_hide_worker_lines() {
    let mut changed = kept().into_vec();
    changed.iter_mut().find(|wire| wire.name.as_ref() == "temper-host-tools").expect("host tools").version = 0;
    let mut missing = kept().into_vec();
    missing.retain(|wire| wire.name.as_ref() != "temper-host-tools");
    let spoken = temper_engine_service::wires()
        .into_boxed()
        .into_iter()
        .find(|wire| wire.name == "temper-host-tools")
        .expect("host tools");
    for (wires, reason) in [
        (
            changed,
            format!("wire temper-host-tools: kept 0, this binary speaks {}..{}", spoken.lowest, spoken.highest()),
        ),
        (missing, "wire temper-host-tools is not kept by the copy".into()),
    ] {
        assert_eq!(
            verdict(Some(wires.into_boxed_slice()), Box::new([opening("w1"), opening("w2")])),
            format!("temper-box upgrade: {reason}\n{}", COMPATIBLE.replace("stay: yes", "stay: no"))
        );
    }
}
