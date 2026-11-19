//! The four written jobs' page contract (temper/docs/design/protocol/store.md, 4).
#![expect(clippy::disallowed_macros, clippy::disallowed_types, reason = "ordinary Rust job fixtures")]
use super::fixtures::{self, layouts};
use crate::{jobs, records};
use jig_protocol_store::{JobAnswer, JobPage, JobRoom, JobRow, JobWrite};
use temper_engine_domain_forge_tiers as t;
use temper_engine_domain_forge_vocabulary as d;

pub(super) fn tier() -> d::TierRow {
    let mut tier = fixtures::tier_row();
    tier.record.run = None;
    tier.probe = None;
    tier.record.failing = t::Failures::Tests(Box::new([Box::from(b"t::a".as_slice())]));
    tier.record.flakes = Box::new([t::Flake { test: b"flake".as_slice().into(), count: 2 }]);
    tier.result = None;
    tier
}
fn input(row: JobRow, variant: &str) -> JobPage {
    JobPage {
        rows: Box::new([row]),
        end: false,
        layout: Some(layouts::layout(layouts::FLOOR, variant)),
        room: JobRoom { writes: 16, bytes: 1_048_576 },
    }
}
fn page(job: &str, input: &JobPage) -> Result<JobAnswer, jig_protocol_store::JobRefusal> {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    jobs::page(job, input, &jobs::Present::default(), &bounds)
}
fn put(answer: &JobAnswer) -> (&[u8], &[u8]) {
    let [JobWrite::Put { key, value }] = &*answer.writes else { panic!("one put: {answer:?}") };
    (key, value)
}

#[test]
fn short_ids_keep_the_binary_encoding_after_the_floor_plan() {
    let mut floor = layouts::Floor::new();
    let tier = d::Stored::Tier(tier());
    let bytes = records::encode(&tier);
    // Bound widths do not change the encoded count or bytes' length.
    let wide = skein_codec::schema::parse(layouts::wide().as_bytes()).unwrap();
    let mut value = skein_codec::value::Value::with_room(100_000);
    value.decode(&wide, wide.find(b"Row").unwrap(), bytes.clone()).unwrap();
    assert_eq!(value.bytes(), bytes.as_ref());
    let input = input(floor.row(&tier, "tier"), "tier");
    let answer = page("test-ids-tier", &input).unwrap();
    assert_eq!(put(&answer), (input.rows[0].key.as_ref(), bytes.as_ref()));
    assert_eq!(answer.through, None);
    assert_eq!(answer.counts.as_ref(), [0, 0, 0]);
}

pub(super) fn long_tier(floor: &mut layouts::Floor) -> (JobRow, d::TierRow) {
    long_tier_as(floor, false)
}
fn long_tier_as(floor: &mut layouts::Floor, suspect: bool) -> (JobRow, d::TierRow) {
    let mut tier = tier();
    let a: Box<[u8]> = Box::new([b'a'; 10]);
    let b: Box<[u8]> = Box::new([b'b'; 10]);
    tier.record.failing = t::Failures::Tests(Box::new([a.clone(), b.clone()]));
    tier.record.flakes = Box::new([
        t::Flake { test: Box::new([b'x'; 10]), count: 1 },
        t::Flake { test: Box::new([b'y'; 10]), count: 2 },
        t::Flake { test: b"z".as_slice().into(), count: 3 },
    ]);
    let mut posting = fixtures::tier_posting(&tier.record.failing);
    posting.entry = None;
    posting.landings = Box::new([d::TierPostedLanding {
        commit: [1; 32],
        blame: if suspect {
            t::Blame::Suspect(t::Failures::Tests(Box::new([b])))
        } else {
            t::Blame::Owns(t::Failures::Tests(Box::new([b])))
        },
        change: None,
    }]);
    tier.posting = Some(Box::new(posting));
    let mut row = floor.row(&d::Stored::Tier(tier.clone()), "tier");
    row.value = layouts::replace_id(&row.value, &[b'b'; 10], &[b'b'; 2000]);
    row.value = layouts::replace_id(&row.value, &[b'y'; 10], &[b'y'; 3000]);
    tier.record.failing = t::Failures::Many;
    tier.record.flakes = Box::new([tier.record.flakes[0].clone(), tier.record.flakes[2].clone()]);
    tier.posting.as_mut().unwrap().landings[0].blame =
        if suspect { t::Blame::Suspect(t::Failures::Many) } else { t::Blame::Owns(t::Failures::Many) };
    (row, tier)
}

#[test]
fn long_ids_make_many_drop_only_long_flakes_and_remap_posted_indices() {
    let mut floor = layouts::Floor::new();
    for suspect in [false, true] {
        let (row, tier) = long_tier_as(&mut floor, suspect);
        let key = row.key.clone();
        let answer = page("test-ids-tier", &input(row, "tier")).unwrap();
        assert!(
            put(&answer) == (key.as_ref(), records::encode(&d::Stored::Tier(tier)).as_ref()),
            "narrowed tier differs from the binary encoding"
        );
        assert_eq!(answer.through, None);
        assert_eq!(answer.counts.as_ref(), [1, 1, 1]);
    }
}

#[test]
fn the_id_boundary_keeps_1024_and_narrows_1025() {
    let mut floor = layouts::Floor::new();
    for length in [1024, 1025] {
        let mut tier = tier();
        let mut row = floor.row(&d::Stored::Tier(tier.clone()), "tier");
        row.value = layouts::replace_id(&row.value, b"t::a", &vec![b'a'; length]);
        row.value = layouts::replace_id(&row.value, b"flake", &vec![b'f'; length]);
        let answer = page("test-ids-tier", &input(row, "tier")).unwrap();
        if length == 1024 {
            tier.record.failing = t::Failures::Tests(Box::new([vec![b'a'; length].into()]));
            tier.record.flakes[0].test = vec![b'f'; length].into();
            assert_eq!(answer.counts.as_ref(), [0, 0, 0]);
        } else {
            tier.record.failing = t::Failures::Many;
            tier.record.flakes = Box::new([]);
            assert_eq!(answer.counts.as_ref(), [1, 1, 0]);
        }
        assert_eq!(put(&answer).1, records::encode(&d::Stored::Tier(tier)).as_ref());
    }
}

pub(super) fn receipt() -> d::Receipt {
    d::Receipt {
        report: temper_engine_domain_forge::ReceiptReport::Unset,
        base: None,
        jobs: Box::new([]),
        logs: Box::new([]),
        failures: Some(t::Failures::Tests(Box::new([b"receipt-id".as_slice().into()]))),
        passed: false,
        run: d::ReceiptRun { id: 1, number: 1, url: Box::new([]) },
        candidate: [1; 32],
        ended: skein_lib::Wall::EPOCH,
    }
}
pub(super) fn receipt_row() -> d::Stored {
    d::Stored::Receipt {
        key: d::ReceiptKey {
            project: 1,
            repository: fixtures::REPO,
            check: d::Check::Gate(b"ci".as_slice().into()),
            key: [1; 32],
        },
        receipt: receipt(),
    }
}
pub(super) fn verification() -> d::Stored {
    d::Stored::Verification(d::Verification {
        repository: fixtures::REPO,
        commit: [1; 32],
        changes: Box::new([]),
        tiers: Box::new([d::TierVerification {
            tier: b"ci".as_slice().into(),
            repaired: None,
            verdict: Some(d::TierVerdict {
                run: 1,
                blame: t::Blame::Suspect(t::Failures::Tests(Box::new([b"verify-id".as_slice().into()]))),
            }),
        }]),
    })
}
#[test]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the fixture holds only the three value types with settled failures"
)]
fn receipt_result_goal_and_verification_failures_narrow_independently() {
    let mut floor = layouts::Floor::new();
    let mut tier = tier();
    tier.result = Some(receipt());
    tier.goals = Box::new([d::TierGoal {
        goal: 2,
        branch: b"g/2".as_slice().into(),
        tip: None,
        ran: None,
        failing: t::Failures::Tests(Box::new([b"goal-id".as_slice().into()])),
        started: None,
    }]);
    let cases = [
        ("receipt", receipt_row(), b"receipt-id".as_slice(), 1),
        ("verification", verification(), b"verify-id".as_slice(), 1),
        ("tier", d::Stored::Tier(tier), b"receipt-id".as_slice(), 2),
    ];
    for (variant, mut expected, marker, many) in cases {
        let mut row = floor.row(&expected, variant);
        row.value = layouts::replace_id(&row.value, marker, &[b'!'; 2000]);
        match &mut expected {
            d::Stored::Receipt { receipt, .. } => receipt.failures = Some(t::Failures::Many),
            d::Stored::Verification(v) => {
                v.tiers[0].verdict.as_mut().unwrap().blame = t::Blame::Suspect(t::Failures::Many);
            }
            d::Stored::Tier(tier) => {
                row.value = layouts::replace_id(&row.value, b"goal-id", &[b'?'; 2000]);
                tier.result.as_mut().unwrap().failures = Some(t::Failures::Many);
                tier.goals[0].failing = t::Failures::Many;
            }
            _ => unreachable!(),
        }
        let answer = page(&format!("test-ids-{variant}"), &input(row, variant)).unwrap();
        assert_eq!(put(&answer).1, records::encode(&expected).as_ref());
        assert_eq!(answer.through, None);
        assert_eq!(answer.counts[0], many);
    }
}

#[test]
fn in_flight_runs_postings_and_landing_selections_refuse_by_key() {
    let mut floor = layouts::Floor::new();
    let mut tier = tier();
    tier.record.run = fixtures::tier_row().record.run;
    // A long id in a probe's finding is just as in flight as its selection.
    let mut run = floor.row(&d::Stored::Tier(tier.clone()), "tier");
    run.value = layouts::replace_id(&run.value, &[1; 256], &[b'r'; 2000]);
    let (mut posting, _) = long_tier(&mut floor);
    let wide = layouts::wide();
    // Rebuild with a pending posting, while keeping the owns index into the long failure.
    tier.record.run = None;
    tier.probe = None;
    tier.record.failing = t::Failures::Tests(Box::new([b"posted-id".as_slice().into()]));
    tier.posting = Some(Box::new(fixtures::tier_posting(&tier.record.failing)));
    posting.value = layouts::replace_id(&floor.row(&d::Stored::Tier(tier), "tier").value, b"posted-id", &[b'p'; 2000]);
    for (row, why) in [
        (run, "a tier run in flight holds a failing test's id past 1,024 bytes"),
        (posting, "a tier posting in flight would change for a failing test's id past 1,024 bytes"),
    ] {
        let key = row.key.clone();
        let refusal = page("test-ids-tier", &input(row, "tier")).unwrap_err();
        assert_eq!(refusal.key, Some(key));
        assert_eq!(&*refusal.why, why);
    }
    let mut landing = landing();
    let d::Stored::Landing(value) = &mut landing else { unreachable!() };
    value.landing.round.search = Some(temper_engine_domain_forge_landing::Search {
        green: 0,
        red: 1,
        tests: temper_engine_domain_forge_landing::SearchTests::ByTest {
            gates: Box::new([Some(Box::new([b"landing-id".as_slice().into()]))]),
            probed: Box::new([]),
        },
    });
    let encoded = super::jobs::encoded(&landing);
    let key = encoded.key.clone();
    let mut input = input(encoded, "landing");
    input.layout = Some(layouts::layout(&wide, "landing"));
    input.rows[0].value = layouts::replace_id(&input.rows[0].value, b"landing-id", &[b'l'; 2000]);
    let refusal = page("test-ids-landing", &input).unwrap_err();
    assert_eq!(refusal.key, Some(key));
    assert_eq!(&*refusal.why, "a landing's search by test holds a failing test's id past 1,024 bytes");
}
pub(super) fn landing() -> d::Stored {
    use temper_engine_domain_forge_landing as l;
    d::Stored::Landing(d::LandingRow {
        room: None,
        flake_reruns: Box::new([]),
        job: None,
        asks: Box::new([]),
        entries: Box::new([]),
        landing: l::Landing {
            last_left: None,
            searched: None,
            round: l::Round::new(b"landing".as_slice().into()),
            task: 1,
            project: 1,
            phase: l::Phase::Waiting,
            preparation: 0,
            stale_proofs: 0,
            branch: b"main".as_slice().into(),
            members: Box::new([]),
        },
    })
}

#[test]
fn paging_counts_only_consumed_rows_and_refuses_an_unfittable_first_row() {
    let mut floor = layouts::Floor::new();
    let (row, _) = long_tier(&mut floor);
    let mut input = input(JobRow { key: row.key.clone(), value: row.value.clone() }, "tier");
    input.rows = Box::new([
        JobRow { key: row.key.clone(), value: row.value.clone() },
        JobRow { key: Box::new([28, 9]), value: row.value.clone() },
    ]);
    let full = page("test-ids-tier", &input).unwrap();
    let one = full.writes[0].bytes();
    for room in [JobRoom { writes: 1, bytes: u64::MAX }, JobRoom { writes: 16, bytes: one }] {
        input.room = room;
        let answer = page("test-ids-tier", &input).unwrap();
        assert_eq!(answer.writes.len(), 1);
        assert_eq!(answer.through.as_ref(), Some(&row.key));
        assert_eq!(answer.counts.as_ref(), [1, 1, 1]);
    }
    for room in [JobRoom { writes: 0, bytes: one }, JobRoom { writes: 1, bytes: one - 1 }] {
        input.room = room;
        let refusal = page("test-ids-tier", &input).unwrap_err();
        assert_eq!(refusal.key.as_ref(), Some(&row.key));
        assert_eq!(&*refusal.why, "row exceeds the test id job edge");
    }
    input.layout = None;
    assert_eq!(&*page("test-ids-tier", &input).unwrap_err().why, "test id job rows have no layout");
    input.rows = Box::new([]);
    input.end = true;
    for (job, counts) in [("landing", vec![]), ("receipt", vec![0]), ("tier", vec![0, 0, 0]), ("verification", vec![0])]
    {
        let answer = page(&format!("test-ids-{job}"), &input).unwrap();
        assert_eq!(answer.counts.as_ref(), counts);
        assert!(answer.writes.is_empty());
        assert_eq!(answer.through, None);
    }
}

#[test]
fn the_wide_text_parses_and_keeps_many_names_at_1024() {
    let wide = layouts::wide();
    skein_codec::schema::parse(wide.as_bytes()).expect("the wide text fits the codec envelope");
    assert!(wide.contains("record ShownTest { test_id: bytes 1024 }"));
    assert!(wide.contains("record FailingTest { test_id: bytes 4096 }"));
}

#[test]
fn the_binary_text_has_each_wide_substitution_once() {
    let wide = layouts::wide();
    assert_ne!(wide, layouts::BINARY);
    for fragment in [
        "record FailingTest { test_id: bytes 1024",
        "record TierFlake { test: bytes 1024",
        "pending: list 1537 TierPending",
        "landings: list 1537 TierLanding",
        "blamed: list 1537 Blame",
        "waiting: list 1537 TierGroup",
        "landings: list 1537 TierPostedLanding",
    ] {
        assert_eq!(layouts::BINARY.matches(fragment).count(), 1);
        let missing = layouts::BINARY.replacen(fragment, "missing", 1);
        let Err(why) = jobs::test_ids::wide_text(&missing) else { panic!("missing declaration was accepted") };
        assert_eq!(why.key, None);
        assert_eq!(&*why.why, format!("the binary's text has no single {fragment}"));
        let doubled = format!("{}\n{fragment}", layouts::BINARY);
        let Err(why) = jobs::test_ids::wide_text(&doubled) else { panic!("duplicate declaration was accepted") };
        assert_eq!(&*why.why, format!("the binary's text has no single {fragment}"));
    }
}
