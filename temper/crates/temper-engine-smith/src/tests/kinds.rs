//! Settings values cross the protocol boundary whole and within its bounds
//! (temper/docs/design/domain/routing.md, 6).
use temper_engine_domain_forge::{GateLast, GatePattern, JudgeFreshness, settings};

use crate::kinds::{Bounds, decode, encode, size};

const BOUNDS: Bounds = Bounds { segments: 5, segment_bytes: 12, gates: 2, approvals: 2 };

fn deployment() -> settings::LandingRule {
    settings::LandingRule {
        sets: Box::new([]),
        pattern: GatePattern {
            segments: Box::new([
                Box::from(b"forge".as_slice()),
                Box::from(b"code.example".as_slice()),
                Box::from(b"team".as_slice()),
                Box::from(b"repo".as_slice()),
                Box::from(b"branch".as_slice()),
            ]),
            last: GateLast::Exact(Box::from(b"main".as_slice())),
        },
        enforce: true,
        ci: false,
        up_to_date: true,
        gates: Box::new([
            settings::RuleGate { number: 2, blocking: true, freshness: JudgeFreshness::Exact },
            settings::RuleGate { number: 7, blocking: false, freshness: JudgeFreshness::Clean },
        ]),
        approvals: Box::new([
            settings::RuleApproval { role: 3, people: 1, freshness: JudgeFreshness::Clean },
            settings::RuleApproval { role: 4, people: 2, freshness: JudgeFreshness::Exact },
        ]),
    }
}

fn values() -> [(u32, settings::Value); 21] {
    let mut project = deployment();
    project.pattern.last = GateLast::Open(Box::from(b"topic/".as_slice()));
    project.enforce = false;
    project.ci = true;
    project.up_to_date = false;
    project.sets = Box::new([6, 5, 4, 3, 2, 1]);
    [
        (9, settings::Value::CharterWords(Box::from([b'l'; 120].as_slice()))),
        (
            10,
            settings::Value::ProjectWords(settings::ProjectWords {
                headline: Box::from([b'h'; 120].as_slice()),
                lede: Box::from([b'l'; 480].as_slice()),
            }),
        ),
        (1, settings::Value::Rule(deployment())),
        (2, settings::Value::Rule(project)),
        (3, settings::Value::Charter(23)),
        (6, settings::Value::Questions(temper_engine_domain_forge_change::Route::None)),
        (6, settings::Value::Questions(temper_engine_domain_forge_change::Route::Up)),
        (
            8,
            settings::Value::Classes(temper_protocol_reaches::Classes::stock(
                temper_protocol_reaches::Way::AgentsReview,
            )),
        ),
        (7, settings::Value::Ways(settings::ProjectWays { drafts: true, goals: false, changes: true, way: u32::MAX })),
        (
            14,
            settings::Value::ProjectAreas(settings::ProjectAreas {
                areas: Box::new([settings::Area {
                    name: b"tests".as_slice().into(),
                    words: b"C tests".as_slice().into(),
                    owner: settings::AreaOwner::Task(u64::MAX),
                }]),
            }),
        ),
        (15, settings::Value::ProjectTriager(settings::ProjectTriager { owner: settings::AreaOwner::Task(u64::MAX) })),
        (
            14,
            settings::Value::ProjectAreas(settings::ProjectAreas {
                areas: Box::new([settings::Area {
                    name: b"tests".as_slice().into(),
                    words: b"C tests".as_slice().into(),
                    owner: settings::AreaOwner::Party(u64::MAX),
                }]),
            }),
        ),
        (15, settings::Value::ProjectTriager(settings::ProjectTriager { owner: settings::AreaOwner::Party(u64::MAX) })),
        (
            14,
            settings::Value::ProjectAreas(settings::ProjectAreas {
                areas: Box::new([settings::Area {
                    name: b"tests".as_slice().into(),
                    words: b"C tests".as_slice().into(),
                    owner: settings::AreaOwner::Role(u32::MAX),
                }]),
            }),
        ),
        (15, settings::Value::ProjectTriager(settings::ProjectTriager { owner: settings::AreaOwner::Role(u32::MAX) })),
        (
            14,
            settings::Value::ProjectAreas(settings::ProjectAreas {
                areas: Box::new([settings::Area {
                    name: b"tests".as_slice().into(),
                    words: b"C tests".as_slice().into(),
                    owner: settings::AreaOwner::Charter(u32::MAX),
                }]),
            }),
        ),
        (
            15,
            settings::Value::ProjectTriager(settings::ProjectTriager { owner: settings::AreaOwner::Charter(u32::MAX) }),
        ),
        (11, settings::Value::Goal(settings::TemplateGoal { design: false, close_out: false })),
        (11, settings::Value::Goal(settings::TemplateGoal { design: true, close_out: true })),
        (11, settings::Value::Goal(settings::TemplateGoal { design: true, close_out: false })),
        (11, settings::Value::Goal(settings::TemplateGoal { design: false, close_out: true })),
    ]
}

#[test]
fn each_kind_round_trips_its_value() {
    assert_eq!(temper_client_protocol_wire::CLIENT_VERSION, 18);
    for (kind, value) in values() {
        let bytes = encode(&value);
        let back = decode(kind, &bytes, &BOUNDS).expect("whole value at its receiving bounds");
        assert_eq!(back, settings::SizedValue { value, size: u32::try_from(bytes.len()).expect("bounded encoding") });
    }

    // Shape decoding leaves semantic validity to the forge's judge.
    let mut rule = deployment();
    *rule.pattern.segments.first_mut().expect("pattern segments") = Box::from([0xff]);
    let number = rule.gates.first().expect("first gate").number;
    rule.gates.get_mut(1).expect("second gate").number = number;
    rule.approvals.first_mut().expect("first approval").people = 0;
    for value in [settings::Value::Rule(rule), settings::Value::Charter(0), settings::Value::Charter(u32::MAX)] {
        let kind = match &value {
            settings::Value::Rule(_) => 1,
            settings::Value::Charter(_) => 3,
            settings::Value::Questions(_) => 6,
            settings::Value::Ways(_) => 7,
            settings::Value::Classes(_) => 8,
            settings::Value::CharterWords(_) => 9,
            settings::Value::ProjectWords(_) => 10,
            settings::Value::Follow(_) => 12,
            settings::Value::Dependency(_) => 13,
            settings::Value::Goal(_) => 11,
            settings::Value::ProjectAreas(_) => 14,
            settings::Value::ProjectTriager(_) => 15,
        };
        let bytes = encode(&value);
        assert_eq!(decode(kind, &bytes, &BOUNDS).expect("shaped value").value, value);
    }
}

#[test]
fn a_value_past_its_bounds_or_cut_short_is_out_of_shape() {
    let value = settings::Value::Rule(deployment());
    let bytes = encode(&value);
    for bounds in [
        Bounds { gates: 1, ..BOUNDS },
        Bounds { approvals: 1, ..BOUNDS },
        Bounds { segments: 4, ..BOUNDS },
        Bounds { segment_bytes: 11, ..BOUNDS },
    ] {
        assert!(decode(1, &bytes, &bounds).is_none(), "past {bounds:?}");
        assert!(decode(2, &bytes, &bounds).is_none(), "past {bounds:?}");
    }
    for last in [GateLast::Exact(Box::new([b'x'; 13])), GateLast::Open(Box::new([b'x'; 13]))] {
        let mut rule = deployment();
        rule.pattern.last = last;
        let bytes = encode(&settings::Value::Rule(rule));
        assert!(decode(1, &bytes, &BOUNDS).is_none(), "last segment past its byte bound");
        assert!(decode(1, &bytes, &Bounds { segment_bytes: 13, ..BOUNDS }).is_some(), "last at its byte bound");
    }
    for (kind, value) in values() {
        let bytes = encode(&value);
        for end in 0..bytes.len() {
            let prefix = bytes.get(..end).expect("prefix within the encoding");
            assert!(decode(kind, prefix, &BOUNDS).is_none(), "kind {kind}, cut at {end}");
        }
        let mut writer = skein_lib::Writer::new(bytes.len().checked_add(1).expect("one extra byte"));
        writer.put(&bytes).expect("encoding room");
        writer.put(&[0]).expect("trailing byte room");
        let trailing = writer.finish();
        assert!(decode(kind, &trailing, &BOUNDS).is_none(), "trailing byte for kind {kind}");
    }
    for kind in [0, 4, u32::MAX] {
        assert!(decode(kind, &bytes, &BOUNDS).is_none(), "unknown kind {kind}");
    }
    assert!(decode(3, &bytes, &BOUNDS).is_none(), "a rule is not a charter");
    assert!(decode(1, &encode(&settings::Value::Charter(23)), &BOUNDS).is_none(), "a charter is not a rule");
}

#[test]
fn size_is_the_encodings_length() {
    for (_, value) in values() {
        let bytes = encode(&value);
        assert_eq!(size(&value), u32::try_from(bytes.len()).expect("bounded encoding"));
    }
}

#[test]
fn follows_round_trip_all_members_and_require_a_whole_record() {
    use settings::{Follow, FollowCheck, FollowOwner, FollowSource, FollowStart, FollowTarget, Value};
    let base = Follow {
        source: FollowSource {
            repository: skein_forge::api::Repository { forge: 1, repository: 12 },
            branch: b"main".as_slice().into(),
            paths: Box::new([b"src".as_slice().into(), b"include/vsr.h".as_slice().into()]),
        },
        target: FollowTarget {
            repository: skein_forge::api::Repository { forge: 2, repository: 13 },
            branch: b"stable".as_slice().into(),
        },
        owner: FollowOwner::Person(4),
        start: FollowStart::Owner,
        producer: Some(b"6;3.1;".as_slice().into()),
        check: FollowCheck {
            workflow: b"tests.yml".as_slice().into(),
            artifact: b"results".as_slice().into(),
            format: temper_engine_domain_forge_tiers::Format::Junit,
        },
    };
    let mut automatic = base.clone();
    automatic.start = FollowStart::Code;
    automatic.check.format = temper_engine_domain_forge_tiers::Format::Lines;
    automatic.producer = None;
    automatic.source.branch = Box::new([]);
    automatic.source.paths = Box::new([]);
    for follow in [base, automatic] {
        let value = Value::Follow(follow);
        let bytes = encode(&value);
        let measured = size(&value);
        assert_eq!(decode(12, &bytes, &BOUNDS), Some(settings::SizedValue { value, size: measured }));
        assert_eq!(usize::try_from(measured).unwrap(), bytes.len());
        for end in 0..bytes.len() {
            assert!(decode(12, &bytes[..end], &BOUNDS).is_none());
        }
        let mut trailing = skein_lib::Writer::new(bytes.len().checked_add(1).unwrap());
        trailing.put(&bytes).unwrap();
        trailing.put(&[0]).unwrap();
        assert!(decode(12, &trailing.finish(), &BOUNDS).is_none());
    }
}
