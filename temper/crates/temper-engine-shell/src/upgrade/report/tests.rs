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
fn workers_lines_are_the_checks_verdict_alone_for_yes_no_and_unknown() {
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
            assert_eq!(text, expected);
            assert_eq!(code, 0, "{verdict} changes no exit code");
            if verdict == "yes" {
                assert_eq!(
                    text,
                    format!(
                        "{binary} upgrade: worker w1 may stay: opened 2026-11-18T02:20:42Z\n{binary} upgrade: worker w2 may stay: opened 2026-11-18T02:20:42Z\n{binary} upgrade: workers may stay: yes\n"
                    )
                );
            } else if verdict == "unknown" {
                assert_eq!(text, format!("{binary} upgrade: workers may stay: unknown; the copy keeps no wires\n"));
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
        let mut report =
            super::tests::report(Event::Rehearsing { owner: Token::new(1), wires: None, openings: Box::new([]) });
        report.events.push(Event::OpenRefused { owner: Token::new(1), why: why.clone() });
        report.events.push(Event::Closed { owner: Token::new(1), removed: true });
        report.end = Some(temper_engine_service::End::Stopped(temper_engine_service::Stop::StoreOpen { why }));
        for binary in ["temper-engine", "temper-box"] {
            let (check, check_code) = lines(binary, &report, &dirs, b"/checkpoint");
            let (text, code) = workers_lines(binary, &report, &dirs, b"/checkpoint");
            assert_eq!(check_code, 1);
            assert_eq!(code, 1);
            assert_eq!(text, check.split_once('\n').expect("check timing line").1);
            assert!(text.contains("/checkpoint"), "{text}");
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
    assert_eq!(workers_lines("temper-box", &report, &dirs, b"/checkpoint"), (COMPATIBLE.into(), 0));
    assert_eq!(
        lines("temper-box", &report, &dirs, b"/checkpoint"),
        (
            format!(
                "temper-box upgrade: decoded 17 records\n\
                 temper-box upgrade: ran temper fixture-job: 2 rows in 1 commits; time unavailable\n\
                 temper-box upgrade: whole rehearsal/tool time: 1000000000 ns\n\
                 temper-box upgrade: registry commit, jobs and restores pass; what follows the switch is not rehearsed\n\
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
        assert_eq!(workers_lines("temper-box", &report, &dirs, b"/checkpoint"), (format!("{refusal}{COMPATIBLE}"), 1));
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
    assert_eq!(workers_lines("temper-box", &report, &dirs, b"/checkpoint"), (format!("{refusal}{COMPATIBLE}"), 1));
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
    })
}

// A single sending kind keeps the report tests about routing judge's verdict, not schema derivation.
pub(super) fn configured() -> Workers {
    Workers {
        room: 4096,
        schema: skein_channel::Schema {
            magic: *b"test",
            versions: List::from_boxed(Box::new([skein_channel::Version {
                version: jig_protocol_worker_wire::wire::VERSION,
                kinds: List::from_boxed(Box::new([skein_channel::Kind {
                    kind: 0x0180,
                    direction: skein_channel::Direction::FromResponder,
                    largest: 8,
                }])),
            }])),
        },
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
    WorkerOpening {
        worker: name.into(),
        wires: temper_engine_service::wires()
            .iter()
            .filter(|wire| worker_wire(wire.name))
            .map(|wire| jig_protocol_store::OpenedWire {
                name: wire.name.into(),
                lowest: wire.lowest,
                layouts: wire.layouts.clone().into_boxed(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        output: 4096,
        terms: Box::new([jig_protocol_store::OpenedTerm { kind: 0x0180, largest: 8 }]),
        at: Wall::from_nanos(1_794_564_000_123_456_789),
    }
}

fn verdict(wires: Option<Box<[KeptWire]>>, openings: Box<[WorkerOpening]>) -> String {
    let mut report = super::tests::report(Event::Rehearsing { owner: Token::new(1), wires, openings });
    report.workers = configured();
    report.restored = true;
    let dirs =
        temper_shell::directories::resolve(&[], &[(b"HOME".as_slice().into(), b"/home/test".as_slice().into())], b"/")
            .expect("directories")
            .0;
    let (text, code) = lines("temper-box", &report, &dirs, b"/state");
    assert_eq!(code, 0, "worker compatibility changes no exit code");
    text.strip_prefix("temper-box upgrade: whole rehearsal/tool time: 1000000000 ns\ntemper-box upgrade: registry commit, jobs and restores pass; what follows the switch is not rehearsed\n").expect("other rehearsal lines unchanged").into()
}

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
    let charter = temper_engine_service::wires()
        .into_boxed()
        .into_iter()
        .find(|wire| wire.name == "smith-charter")
        .expect("charter");
    let version = charter.highest();
    let mut hex = String::new();
    for byte in charter.layout(version).expect("current layout").iter().take(8) {
        write!(hex, "{byte:02x}").expect("string write");
    }
    let mut layout = opening("w2");
    *layout
        .wires
        .iter_mut()
        .find(|wire| wire.name.as_ref() == "smith-charter")
        .expect("charter")
        .layouts
        .last_mut()
        .expect("current layout") = [0xcd; 32];
    let mut disjoint = opening("w2");
    disjoint.wires.iter_mut().find(|wire| wire.name.as_ref() == "smith-charter").expect("charter").lowest =
        version.checked_add(1).expect("test version");
    let mut output = opening("w1");
    output.output = 4095;
    let mut terms = opening("w2");
    terms.terms[0].largest = 7;
    for (opening, name, other, reason) in [
        (
            layout,
            "w2",
            "w1",
            format!(
                "wires: wire smith-charter version {version}: the engine's layout {hex}, the worker's cdcdcdcdcdcdcdcd"
            ),
        ),
        (
            disjoint,
            "w2",
            "w1",
            format!(
                "wires: wire smith-charter: the engine speaks {}..{version}, the worker {}..{}",
                charter.lowest,
                version + 1,
                version + u16::try_from(charter.layouts.len()).expect("range")
            ),
        ),
        (
            output,
            "w1",
            "w2",
            "limits: engine io.output 4096 bytes differs from worker io.output 4095 bytes; change io.output to agree"
                .into(),
        ),
        (terms, "w2", "w1", "limits: 384".into()),
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
