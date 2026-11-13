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
            workers: configured(&[]),
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
                workers: configured(&[]),
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
            assert!(!text.contains("patches and restores pass"));
        }
    }
}

// A single sending kind keeps the report tests about routing judge's verdict, not schema derivation.
pub(super) fn configured(names: &[&str]) -> Workers {
    Workers {
        names: names.iter().map(|name| name.as_bytes().into()).collect::<Vec<_>>().into_boxed_slice(),
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

fn verdict(wires: Option<Box<[KeptWire]>>, fresh: bool, openings: Box<[WorkerOpening]>, names: &[&str]) -> String {
    let mut report = super::tests::report(Event::Rehearsing { owner: Token::new(1), wires, fresh, openings });
    report.workers = configured(names);
    report.restored = true;
    let dirs =
        temper_shell::directories::resolve(&[], &[(b"HOME".as_slice().into(), b"/home/test".as_slice().into())], b"/")
            .expect("directories")
            .0;
    let (text, code) = lines("temper-box", &report, &dirs, b"/state");
    assert_eq!(code, 0, "worker compatibility changes no exit code");
    text.strip_prefix("temper-box upgrade: whole rehearsal/tool time: 1000000000 ns\ntemper-box upgrade: patches and restores pass; what follows the switch is not rehearsed\n").expect("other rehearsal lines unchanged").into()
}

const COMPATIBLE: &str = "temper-box upgrade: worker w1 may stay: opened 2026-11-13T10:00:00Z\ntemper-box upgrade: worker w2 may stay: opened 2026-11-13T10:00:00Z\ntemper-box upgrade: workers may stay: yes\n";
const NO: &str = "temper-box upgrade: workers may stay: no\n";

#[test]
fn compatible_openings_follow_configuration_order_ignoring_unconfigured_names() {
    let mut wires = kept().into_vec();
    // Only the engine's own contracts are read from Wires, not a worker's wires.
    wires.retain(|wire| wire.name.as_ref() != "smith-charter");
    wires.push(KeptWire { name: "retired".into(), version: 999 });
    assert_eq!(
        verdict(
            Some(wires.into_boxed_slice()),
            false,
            Box::new([opening("w2"), opening("unconfigured"), opening("w1")]),
            &["w1", "w2"]
        ),
        COMPATIBLE
    );
    assert_eq!(
        verdict(Some(kept()), false, Box::new([opening("unconfigured")]), &[]),
        "temper-box upgrade: workers may stay: yes\n"
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
    layout
        .wires
        .iter_mut()
        .find(|wire| wire.name.as_ref() == "smith-charter")
        .expect("charter")
        .layouts
        .fill([0xcd; 32]);
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
                version + 1
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
        // The failing worker precedes the compatible worker in configuration order.
        assert_eq!(
            verdict(Some(kept()), false, Box::new([opening, self::opening(other)]), &[name, other]),
            format!(
                "temper-box upgrade: worker {name} may not stay: {reason}\ntemper-box upgrade: worker {other} may stay: opened 2026-11-13T10:00:00Z\n{NO}"
            )
        );
    }
}

#[test]
fn a_configured_worker_without_an_opening_is_unknown_and_cannot_stay() {
    assert_eq!(
        verdict(Some(kept()), false, Box::new([opening("w1")]), &["w1", "w2"]),
        format!(
            "temper-box upgrade: worker w1 may stay: opened 2026-11-13T10:00:00Z\ntemper-box upgrade: worker w2 unknown: no opening kept since the engine's last start\n{NO}"
        )
    );
}

#[test]
fn fresh_patches_preempt_wires_and_individual_worker_judgements() {
    for wires in [None, Some(kept())] {
        assert_eq!(
            verdict(wires, true, Box::new([opening("w1")]), &["w1"]),
            format!("temper-box upgrade: a pending patch requires fresh workers; stop the workers\n{NO}")
        );
    }
}

#[test]
fn no_kept_wires_makes_the_verdict_unknown_without_judging_workers() {
    assert_eq!(
        verdict(None, false, Box::new([opening("w1")]), &["w1"]),
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
            verdict(Some(wires.into_boxed_slice()), false, Box::new([opening("w1"), opening("w2")]), &["w1", "w2"]),
            format!("temper-box upgrade: {reason}\n{}", COMPATIBLE.replace("stay: yes", "stay: no"))
        );
    }
}
