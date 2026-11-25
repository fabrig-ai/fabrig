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
    for verdict in ["yes", "no", "restart", "unknown"] {
        let mut w1 = opening("w1");
        let mut w2 = opening("w2");
        w1.at = Wall::from_nanos(1_794_968_442_000_000_000);
        w2.at = w1.at;
        if verdict == "no" {
            w1.output = 0;
        } else if verdict == "restart" {
            w1.jobs = without_close(w1.jobs);
        }
        let mut report = super::tests::report(Event::Rehearsing {
            owner: Token::new(1),
            wires: if verdict == "unknown" { None } else { Some(kept()) },
            openings: Box::new([KeptOpening::Current(w1), KeptOpening::Current(w2)]),
            workers: Some(Box::new([])),
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
            } else if verdict == "restart" {
                assert!(text.contains("worker w1 must restart: job kind 4 (close): it runs none\n"), "{text}");
                assert!(text.ends_with("workers may stay: no\n"), "{text}");
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
        openings: Box::new([opening("w1"), opening("w2")].map(KeptOpening::Current)),
        workers: Some(Box::new([])),
        linked: 12,
        copied: 3,
    })
}

pub(super) fn configured() -> Workers {
    let jig = jig_protocol_worker_wire::spoken();
    let application =
        temper_protocol_worker_wire::application(&temper_protocol_worker_wire::v1::CEILINGS).expect("application");
    Workers {
        defaults: List::from_boxed(Box::new([kept_worker(1, "w1", false), kept_worker(2, "w2", false)])),
        room: 4096,
        limits: jig_protocol_worker_wire::CEILINGS,
        spoken: List::from_boxed(jig.iter().chain(&application.spoken).cloned().collect()),
        roots: application.roots,
        jobs: temper_engine_service::limits::jobs(&jig_protocol_config::stock()),
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
                layout: wire.layout,
                text: wire.text.as_bytes().into(),
            })
            .collect(),
        output: configured.room,
        terms: table
            .kinds
            .iter()
            .filter(|kind| kind.direction == skein_channel::Direction::FromResponder)
            .map(|kind| jig_protocol_store::OpenedTerm { kind: kind.kind, largest: kind.largest })
            .collect(),
        jobs: configured.jobs,
        at: Wall::from_nanos(1_794_564_000_123_456_789),
    }
}

fn kept_worker(number: u32, name: &str, withdrawn: bool) -> KeptWorker {
    KeptWorker { number, name: name.as_bytes().into(), withdrawn }
}

fn verdict(wires: Option<Box<[KeptWire]>>, openings: Box<[WorkerOpening]>) -> String {
    verdict_with_workers(
        wires,
        openings.into_iter().map(KeptOpening::Current).collect(),
        Some(Box::new([])),
        configured().defaults,
    )
}

fn verdict_with_workers(
    wires: Option<Box<[KeptWire]>>,
    openings: Box<[KeptOpening]>,
    workers: Option<Box<[KeptWorker]>>,
    defaults: List<KeptWorker>,
) -> String {
    let mut report = super::tests::report(Event::Rehearsing {
        owner: Token::new(1),
        wires,
        openings,
        workers,
        linked: 12,
        copied: 3,
    });
    report.workers.defaults = defaults;
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
const W2_STAYS: &str = "temper-box upgrade: worker w2 may stay: opened 2026-11-13T10:00:00Z\n";

#[test]
fn compatible_openings_follow_worker_number_not_opening_order() {
    let mut wires = kept().into_vec();
    // Only the engine's own contracts are read from Wires, not a worker's wires.
    wires.retain(|wire| wire.name.as_ref() != "smith-charter");
    wires.push(KeptWire { name: "retired".into(), version: 999 });
    assert_eq!(
        verdict(Some(wires.into_boxed_slice()), Box::new([opening("w2"), opening("unconfigured"), opening("w1")])),
        COMPATIBLE
    );
}

#[test]
fn an_opening_with_derived_wire_plans_may_stay() {
    let mut derived = opening("w2");
    let wire = derived.wires.iter_mut().find(|wire| wire.name.as_ref() == "jig-worker-wire").expect("channel");
    // Each side has a defaulted field and an optional kind the other lacks, with roots unchanged.
    let text = jig_protocol_worker_wire::SCHEMA
        .replace("    fresh: bool = false\n", "")
        .replace("value: FactValue }", "value: FactValue worker_note: u32 = 19 }")
        .replace("enum FromWorker {", "record WorkerOnly { value: u8 }\nenum FromWorker {")
        .replace("paused: Paused, part: Part", "paused: Paused, part: Part, worker_only: WorkerOnly")
        .replace(", part_busy: PartBusy", "");
    let schema = skein_codec::schema::parse(text.as_bytes()).expect("derived text");
    let layout = schema.fingerprints().family().digest();
    assert_ne!(wire.layout, layout);
    wire.layout = layout;
    wire.text = text.into_bytes().into_boxed_slice();
    // The responder's remaining kinds keep their tags and byte bounds.
    derived.terms = derived
        .terms
        .into_iter()
        .filter(|term| term.kind != jig_protocol_worker_wire::Kind::PartBusy.number())
        .collect();
    assert_eq!(verdict(Some(kept()), Box::new([opening("w1"), derived])), COMPATIBLE);
}

fn without_close(jobs: Box<[jig_domain_link::JobBound]>) -> Box<[jig_domain_link::JobBound]> {
    jobs.into_iter().filter(|job| job.kind.0 != 4).collect()
}

#[test]
fn workers_must_restart_if_a_job_kind_or_bound_falls_short() {
    let close = temper_engine_service::limits::close_bound();
    for (offered, why) in [
        (None, "it runs none".into()),
        (
            Some(jig_domain_link::JobBound { spec: close.spec - 1, ..close }),
            format!("its spec takes {} bytes, this binary's {} bytes", close.spec - 1, close.spec),
        ),
        (
            Some(jig_domain_link::JobBound { result: close.result - 1, ..close }),
            format!("its result takes {} bytes, this binary's {} bytes", close.result - 1, close.result),
        ),
        (
            Some(jig_domain_link::JobBound { deadline: Duration::from_secs(300), ..close }),
            format!("its deadline is 300 s, this binary's {} s", close.deadline.as_millis() / 1000),
        ),
        (
            Some(jig_domain_link::JobBound { deadline: Duration::from_millis(300_001), ..close }),
            format!("its deadline is 300001 ms, this binary's {} s", close.deadline.as_millis() / 1000),
        ),
    ] {
        let mut w1 = opening("w1");
        w1.jobs = without_close(w1.jobs).into_iter().chain(offered).collect();
        assert_eq!(
            verdict(Some(kept()), Box::new([w1, opening("w2")])),
            format!("temper-box upgrade: worker w1 must restart: job kind 4 (close): {why}\n{W2_STAYS}{NO}")
        );
    }
}

fn drop_worker_fields(opening: &mut WorkerOpening, fields: &[&str]) {
    let wire = opening.wires.iter_mut().find(|wire| wire.name.as_ref() == "temper-worker-wire").expect("worker wire");
    let mut text = str::from_utf8(&wire.text).expect("schema text").to_owned();
    for field in fields {
        assert_eq!(text.matches(field).count(), 1, "{field}");
        text = text.replace(field, "");
    }
    let layout = skein_codec::schema::parse(text.as_bytes()).expect("older text").fingerprints().family().digest();
    assert_ne!(wire.layout, layout);
    wire.layout = layout;
    wire.text = text.into_bytes().into_boxed_slice();
}

fn worker_layout_hex() -> String {
    let layout = temper_protocol_worker_wire::wire::layout();
    let prefix = layout[..8].try_into().expect("eight layout bytes");
    format!("{:016x}", u64::from_be_bytes(prefix))
}

#[test]
fn a_worker_must_restart_if_its_layout_cannot_keep_a_job_spec() {
    let mut w1 = opening("w1");
    drop_worker_fields(&mut w1, &[" request: bytes 1048576"]);
    let hex = worker_layout_hex();
    assert_eq!(
        verdict(Some(kept()), Box::new([w1, opening("w2")])),
        format!(
            "temper-box upgrade: worker w1 must restart: the worker's layout cannot keep temper-worker-wire JobSpec.request: field, sent in jobs of kinds 1 (prepare), 3 (merge) and 4 (close); this binary's temper-worker-wire is at {hex}\n{W2_STAYS}{NO}"
        )
    );
}

#[test]
fn all_job_shortfalls_come_before_layout_gaps() {
    let mut w1 = opening("w1");
    w1.jobs = Box::new([]);
    drop_worker_fields(&mut w1, &[" request: bytes 1048576", " identity: u32"]);
    let hex = worker_layout_hex();
    assert_eq!(
        verdict(Some(kept()), Box::new([w1, opening("w2")])),
        format!(
            "temper-box upgrade: worker w1 must restart: job kind 1 (prepare): it runs none\n\
             temper-box upgrade: worker w1 must restart: job kind 3 (merge): it runs none\n\
             temper-box upgrade: worker w1 must restart: job kind 4 (close): it runs none\n\
             temper-box upgrade: worker w1 must restart: the worker's layout cannot keep temper-worker-wire Repository.identity: field, sent in assignments and jobs of kinds 1 (prepare), 3 (merge) and 4 (close); this binary's temper-worker-wire is at {hex}\n\
             temper-box upgrade: worker w1 must restart: the worker's layout cannot keep temper-worker-wire JobSpec.request: field, sent in jobs of kinds 1 (prepare), 3 (merge) and 4 (close); this binary's temper-worker-wire is at {hex}\n{W2_STAYS}{NO}"
        )
    );
}

#[test]
fn worker_refusals_keep_the_openings_wire_and_limits_words() {
    let mut layout = opening("w2");
    let charter = layout.wires.iter_mut().find(|wire| wire.name.as_ref() == "smith-charter").expect("charter");
    charter.layout = [0xcd; 32];
    charter.text = b"not a schema".as_slice().into();
    let mut no_plan = opening("w2");
    let charter = no_plan.wires.iter_mut().find(|wire| wire.name.as_ref() == "smith-charter").expect("charter");
    let text =
        str::from_utf8(&charter.text).expect("schema text").replace("instructions: text 262144", "instructions: u32");
    charter.layout =
        skein_codec::schema::parse(text.as_bytes()).expect("retyped schema").fingerprints().family().digest();
    charter.text = text.into_bytes().into_boxed_slice();
    let mut missing = opening("w2");
    missing.wires = missing.wires.into_iter().filter(|wire| wire.name.as_ref() != "smith-charter").collect();
    let mut output = opening("w1");
    output.output = 4095;
    drop_worker_fields(&mut output, &[" request: bytes 1048576"]);
    let mut terms = opening("w2");
    terms.terms[0].largest = 7;
    for (mut opening, name, other, reason) in [
        (layout, "w2", "w1", "wires: wire smith-charter: the worker's text does not parse at line 1"),
        (
            no_plan,
            "w2",
            "w1",
            "wires: wire smith-charter Charter: no plan to the worker: retyped at Charter.instructions",
        ),
        (missing, "w2", "w1", "wires: wire smith-charter: the worker states none"),
        (
            output,
            "w1",
            "w2",
            "limits: engine io.output 4096 bytes differs from worker io.output 4095 bytes; change io.output to agree",
        ),
        (terms, "w2", "w1", "limits: 384"),
    ] {
        opening.jobs = Box::new([]);
        let first = format!("temper-box upgrade: worker {name} may not stay: {reason}\n");
        let second = format!("temper-box upgrade: worker {other} may stay: opened 2026-11-13T10:00:00Z\n");
        let expected = if name == "w1" { format!("{first}{second}{NO}") } else { format!("{second}{first}{NO}") };
        assert_eq!(verdict(Some(kept()), Box::new([opening, self::opening(other)])), expected);
    }
}

#[test]
fn earlier_and_missing_openings_have_distinct_unknown_lines() {
    assert_eq!(
        verdict_with_workers(
            Some(kept()),
            Box::new([KeptOpening::BeforeLayouts { worker: "w1".into() }]),
            Some(Box::new([])),
            configured().defaults,
        ),
        format!(
            "temper-box upgrade: worker w1 unknown: opened before layouts\ntemper-box upgrade: worker w2 unknown: no opening kept since the engine's last start\n{NO}"
        )
    );
}

#[test]
fn an_opening_before_jobs_were_kept_cannot_stay() {
    assert_eq!(
        verdict_with_workers(
            Some(kept()),
            Box::new([KeptOpening::BeforeJobs { worker: "w1".into() }, KeptOpening::Current(opening("w2"))]),
            Some(Box::new([])),
            configured().defaults,
        ),
        format!(
            "temper-box upgrade: worker w1 unknown: opened before its job kinds were kept\ntemper-box upgrade: worker w2 may stay: opened 2026-11-13T10:00:00Z\n{NO}"
        )
    );
}

#[test]
fn stored_worker_settings_overlay_defaults_by_number_and_hide_withdrawals() {
    let defaults = List::from_boxed(Box::new([kept_worker(1, "local", false), kept_worker(3, "retired", false)]));
    let changes =
        Box::new([kept_worker(1, "local", true), kept_worker(2, "w2", false), kept_worker(3, "retired", true)]);
    assert_eq!(
        verdict_with_workers(
            Some(kept()),
            Box::new([opening("local"), opening("retired"), opening("w2")].map(KeptOpening::Current)),
            Some(changes),
            defaults,
        ),
        "temper-box upgrade: worker w2 may stay: opened 2026-11-13T10:00:00Z\ntemper-box upgrade: workers may stay: yes\n"
    );
}

#[test]
fn unreadable_worker_settings_make_the_verdict_unknown() {
    assert_eq!(
        verdict_with_workers(Some(kept()), Box::new([]), None, configured().defaults),
        "temper-box upgrade: workers may stay: unknown; the copy's workers cannot be read\n"
    );
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
