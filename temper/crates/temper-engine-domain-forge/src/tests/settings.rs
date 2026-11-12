//! Values of the forge's kinds and procedure waits
//! (temper/docs/design/protocol/configuration/settings.md, 2 and 3).
use crate::settings::{
    Bounds, Held, Item, Kind, LandingRule, Refusal, RuleApproval, RuleGate, TemplateGoal, Value, judge, requires,
};
use crate::{GateLast, GatePattern, JudgeFreshness, Requires};
use alloc::boxed::Box;
use skein_lib::List;

// Tiny admission bounds: two gates and roles, five base segments, eight bytes each.
const BOUNDS: Bounds = Bounds {
    landing_rules: 4,
    gates: 2,
    roles: 2,
    charters: 3,
    segments: 5,
    segment_bytes: 8,
    templates: 4,
    follows: 2,
    dependencies: 4,
    follow_paths: 2,
    name_bytes: 32,
};
const PROJECT: u32 = 1;
const FIRST_RULE: u32 = 1;
const GATE: RuleGate = RuleGate { number: 1, blocking: true, freshness: JudgeFreshness::Exact };
const APPROVAL: RuleApproval = RuleApproval { role: 0, people: 1, freshness: JudgeFreshness::Clean };
const BOTH: Requires = Requires { ci: true, up_to_date: true };
const NEITHER: Requires = Requires { ci: false, up_to_date: false };
const CI: Requires = Requires { ci: true, up_to_date: false };

fn rule(pattern: GatePattern, asks: Requires) -> LandingRule {
    LandingRule {
        sets: Box::new([]),
        pattern,
        enforce: true,
        ci: asks.ci,
        up_to_date: asks.up_to_date,
        gates: Box::new([]),
        approvals: Box::new([]),
    }
}

fn branch_pattern(repository: &[u8], last: GateLast) -> GatePattern {
    GatePattern {
        segments: Box::new([
            Box::from(b"forge".as_slice()),
            Box::from(b"host".as_slice()),
            Box::from(b"owner".as_slice()),
            Box::from(repository),
            Box::from(b"branch".as_slice()),
        ]),
        last,
    }
}

fn bounded_item() -> Item {
    let pattern = branch_pattern(b"12345678", GateLast::Exact(Box::from(b"12345678".as_slice())));
    let mut rule = rule(pattern, BOTH);
    rule.gates = Box::new([GATE, RuleGate { number: 2, blocking: false, freshness: JudgeFreshness::Clean }]);
    rule.approvals = Box::new([APPROVAL, RuleApproval { role: 1, freshness: JudgeFreshness::Exact, ..APPROVAL }]);
    Item { kind: Kind::DeploymentRules.number(), scope: 0, number: FIRST_RULE, value: Some(Value::Rule(rule)) }
}

fn with_rule(rule: LandingRule) -> Item {
    Item { value: Some(Value::Rule(rule)), ..bounded_item() }
}

fn held(scope: u32, number: u32, rule: LandingRule) -> Held {
    Held { scope, number, rule }
}

fn main_rule() -> LandingRule {
    rule(branch_pattern(b"name", GateLast::Exact(Box::from(b"main".as_slice()))), BOTH)
}

fn goal_rule() -> LandingRule {
    rule(branch_pattern(b"name", GateLast::Open(Box::from(b"temper".as_slice()))), NEITHER)
}

fn main_name() -> Box<[u8]> {
    b"main".as_slice().into()
}

fn goal_name() -> Box<[u8]> {
    b"temper/1/g1".as_slice().into()
}

fn repository() -> crate::Repository {
    crate::Repository {
        host: b"host".as_slice().into(),
        owner: b"owner".as_slice().into(),
        name: b"name".as_slice().into(),
        ..super::adopted()
    }
}

#[test]
fn judge_accepts_rules_within_their_bounds() {
    let base = bounded_item();
    let rows = [
        ("deployment first", base.clone(), Ok(())),
        (
            "project sets without classes",
            Item { kind: 2, scope: PROJECT, ..with_rule(LandingRule { sets: Box::new([1, 3]), ..main_rule() }) },
            Ok(()),
        ),
        (
            "project all sets in supplied order",
            Item {
                kind: 2,
                scope: PROJECT,
                ..with_rule(LandingRule { sets: Box::new([6, 5, 4, 3, 2, 1]), ..main_rule() })
            },
            Ok(()),
        ),
        (
            "project branch exact",
            Item {
                kind: 2,
                scope: PROJECT,
                ..with_rule(rule(
                    GatePattern {
                        segments: Box::new([Box::from(b"branch".as_slice())]),
                        last: GateLast::Exact(b"main".as_slice().into()),
                    },
                    BOTH,
                ))
            },
            Ok(()),
        ),
        (
            "project branch open",
            Item {
                kind: 2,
                scope: PROJECT,
                ..with_rule(rule(
                    GatePattern {
                        segments: Box::new([Box::from(b"branch".as_slice())]),
                        last: GateLast::Open(Box::new([])),
                    },
                    BOTH,
                ))
            },
            Ok(()),
        ),
        ("deployment last", Item { number: BOUNDS.landing_rules, ..base.clone() }, Ok(())),
        ("project first", Item { kind: Kind::ProjectRules.number(), scope: PROJECT, ..base.clone() }, Ok(())),
        (
            "project last",
            Item { kind: Kind::ProjectRules.number(), scope: PROJECT, number: BOUNDS.landing_rules, ..base.clone() },
            Ok(()),
        ),
        ("remove deployment", Item { value: None, ..base.clone() }, Ok(())),
        ("remove project", Item { kind: 2, scope: PROJECT, value: None, ..base }, Ok(())),
    ];
    for (label, item, expected) in rows {
        assert_eq!(judge(&item, &BOUNDS), expected, "{label}");
    }
    for (number, expected) in [
        (0, None),
        (1, Some(Kind::DeploymentRules)),
        (2, Some(Kind::ProjectRules)),
        (3, Some(Kind::Charters)),
        (4, None),
        (5, None),
        (11, Some(Kind::TemplateGoal)),
        (u32::MAX, None),
    ] {
        assert_eq!(Kind::of(number), expected, "kind {number}");
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "each admission fault is a row of the required single table test")]
fn judge_refuses_each_fault_by_name() {
    let base = bounded_item();
    let Some(Value::Rule(valid)) = base.value.clone() else { panic!("rule fixture") };
    let mut wrong_first = valid.pattern.clone();
    wrong_first.segments[0] = Box::from(b"branch".as_slice());
    let mut long_segment = valid.pattern.clone();
    long_segment.segments[1] = Box::from(b"123456789".as_slice());
    let mut extra_segments = List::with_capacity(BOUNDS.segments + 1);
    for segment in &valid.pattern.segments {
        extra_segments.push(segment.clone()).expect("base segments fit fixture");
    }
    extra_segments.push(Box::from(b"extra".as_slice())).expect("one past bound fits fixture");
    let rows = [
        ("kind 4", Item { kind: 4, ..base.clone() }, Err(Refusal::Kind)),
        (
            "deployment set",
            with_rule(LandingRule { sets: Box::new([1]), ..valid.clone() }),
            Err(Refusal::DeploymentSets),
        ),
        (
            "project set twice",
            Item { kind: 2, scope: PROJECT, ..with_rule(LandingRule { sets: Box::new([2, 2]), ..valid.clone() }) },
            Err(Refusal::SetTwice),
        ),
        (
            "project set zero",
            Item { kind: 2, scope: PROJECT, ..with_rule(LandingRule { sets: Box::new([0]), ..valid.clone() }) },
            Err(Refusal::SetNumber),
        ),
        (
            "project set seven",
            Item { kind: 2, scope: PROJECT, ..with_rule(LandingRule { sets: Box::new([7]), ..valid.clone() }) },
            Err(Refusal::SetNumber),
        ),
        (
            "project sets past bound",
            Item {
                kind: 2,
                scope: PROJECT,
                ..with_rule(LandingRule { sets: Box::new([1, 2, 3, 4, 5, 6, 1]), ..valid.clone() })
            },
            Err(Refusal::Sets),
        ),
        (
            "deployment branch exact",
            with_rule(rule(
                GatePattern {
                    segments: Box::new([Box::from(b"branch".as_slice())]),
                    last: GateLast::Exact(b"main".as_slice().into()),
                },
                BOTH,
            )),
            Err(Refusal::Pattern),
        ),
        (
            "deployment branch open",
            with_rule(rule(
                GatePattern {
                    segments: Box::new([Box::from(b"branch".as_slice())]),
                    last: GateLast::Open(Box::new([])),
                },
                BOTH,
            )),
            Err(Refusal::Pattern),
        ),
        ("deployment scope", Item { scope: 2, ..base.clone() }, Err(Refusal::Scope)),
        ("project scope", Item { kind: 2, ..base.clone() }, Err(Refusal::Scope)),
        ("number zero", Item { number: 0, ..base.clone() }, Err(Refusal::Number)),
        ("number past bound", Item { number: BOUNDS.landing_rules + 1, ..base.clone() }, Err(Refusal::Number)),
        ("first segment", with_rule(LandingRule { pattern: wrong_first, ..valid.clone() }), Err(Refusal::Pattern)),
        (
            "empty pattern",
            with_rule(LandingRule {
                pattern: GatePattern { segments: Box::new([]), last: valid.pattern.last.clone() },
                ..valid.clone()
            }),
            Err(Refusal::Pattern),
        ),
        (
            "segments past bound",
            with_rule(LandingRule {
                pattern: GatePattern { segments: extra_segments.into_boxed(), last: valid.pattern.last.clone() },
                ..valid.clone()
            }),
            Err(Refusal::Pattern),
        ),
        ("segment bytes", with_rule(LandingRule { pattern: long_segment, ..valid.clone() }), Err(Refusal::Pattern)),
        (
            "exact last bytes",
            with_rule(LandingRule {
                pattern: GatePattern {
                    last: GateLast::Exact(Box::from(b"123456789".as_slice())),
                    ..valid.pattern.clone()
                },
                ..valid.clone()
            }),
            Err(Refusal::Pattern),
        ),
        (
            "open last bytes",
            with_rule(LandingRule {
                pattern: GatePattern {
                    last: GateLast::Open(Box::from(b"123456789".as_slice())),
                    ..valid.pattern.clone()
                },
                ..valid.clone()
            }),
            Err(Refusal::Pattern),
        ),
        (
            "gates past bound",
            with_rule(LandingRule {
                gates: Box::new([GATE, RuleGate { number: 2, ..GATE }, RuleGate { number: 3, ..GATE }]),
                ..valid.clone()
            }),
            Err(Refusal::Gates),
        ),
        ("gate twice", with_rule(LandingRule { gates: Box::new([GATE, GATE]), ..valid.clone() }), Err(Refusal::Gates)),
        (
            "approval no people",
            with_rule(LandingRule { approvals: Box::new([RuleApproval { people: 0, ..APPROVAL }]), ..valid.clone() }),
            Err(Refusal::Approvals),
        ),
        (
            "role at bound",
            with_rule(LandingRule {
                approvals: Box::new([RuleApproval { role: BOUNDS.roles, ..APPROVAL }]),
                ..valid.clone()
            }),
            Err(Refusal::Approvals),
        ),
        (
            "role twice",
            with_rule(LandingRule { approvals: Box::new([APPROVAL, APPROVAL]), ..valid.clone() }),
            Err(Refusal::Approvals),
        ),
        (
            "approvals past bound",
            with_rule(LandingRule {
                approvals: Box::new([
                    APPROVAL,
                    RuleApproval { role: 1, ..APPROVAL },
                    RuleApproval { role: 2, ..APPROVAL },
                ]),
                ..valid
            }),
            Err(Refusal::Approvals),
        ),
        ("removal kind", Item { kind: 4, value: None, ..base.clone() }, Err(Refusal::Kind)),
        ("removal scope", Item { scope: PROJECT, value: None, ..base.clone() }, Err(Refusal::Scope)),
        ("removal number", Item { number: 0, value: None, ..base }, Err(Refusal::Number)),
    ];
    for (label, item, expected) in rows {
        assert_eq!(judge(&item, &BOUNDS), expected, "{label}");
    }
}

#[test]
fn judge_reads_the_forge_charters() {
    let goal = Item { kind: Kind::TemplateGoal.number(), scope: 0, number: 1, value: None };
    for (design, close_out) in [(false, false), (false, true), (true, false), (true, true)] {
        for number in [1, BOUNDS.templates] {
            let item = Item { number, value: Some(Value::Goal(TemplateGoal { design, close_out })), ..goal.clone() };
            assert_eq!(judge(&item, &BOUNDS), Ok(()), "{item:?}");
        }
    }
    for (label, item, expected) in [
        ("goal project scope", Item { scope: PROJECT, ..goal.clone() }, Err(Refusal::Scope)),
        ("goal number zero", Item { number: 0, ..goal.clone() }, Err(Refusal::Number)),
        ("goal number past templates", Item { number: BOUNDS.templates + 1, ..goal.clone() }, Err(Refusal::Number)),
        ("goal wrong value", Item { value: Some(Value::Charter(1)), ..goal.clone() }, Err(Refusal::Value)),
        ("goal removal or reset", goal, Ok(())),
    ] {
        assert_eq!(judge(&item, &BOUNDS), expected, "{label}");
    }
    let base = Item { kind: Kind::Charters.number(), scope: 0, number: 1, value: Some(Value::Charter(1)) };
    for number in [1, 2] {
        let rows = [
            ("first charter", base.clone(), Ok(())),
            ("last charter", Item { value: Some(Value::Charter(BOUNDS.charters)), ..base.clone() }, Ok(())),
            ("charter zero", Item { value: Some(Value::Charter(0)), ..base.clone() }, Err(Refusal::Charter)),
            (
                "charter past bound",
                Item { value: Some(Value::Charter(BOUNDS.charters + 1)), ..base.clone() },
                Err(Refusal::Charter),
            ),
            ("remove charter", Item { value: None, ..base.clone() }, Err(Refusal::Charter)),
            ("scope", Item { scope: 1, ..base.clone() }, Err(Refusal::Scope)),
        ];
        for (label, item, expected) in rows {
            assert_eq!(judge(&Item { number, ..item }, &BOUNDS), expected, "{label}, number {number}");
        }
    }
    let rows = [
        ("number zero", Item { number: 0, ..base.clone() }, Refusal::Number),
        ("number three", Item { number: 3, ..base.clone() }, Refusal::Number),
        ("scope before number", Item { scope: 1, number: 0, ..base.clone() }, Refusal::Scope),
        ("number before removal", Item { number: 3, value: None, ..base.clone() }, Refusal::Number),
        ("number before value", Item { number: 0, value: Some(Value::Rule(main_rule())), ..base }, Refusal::Number),
    ];
    for (label, item, refusal) in rows {
        assert_eq!(judge(&item, &BOUNDS), Err(refusal), "{label}");
    }
}

#[test]
fn judge_refuses_a_value_of_another_kind() {
    let rows = [
        Item { kind: 1, scope: 0, number: 1, value: Some(Value::Charter(1)) },
        Item { kind: 2, scope: PROJECT, number: 1, value: Some(Value::Charter(1)) },
        Item { kind: 3, scope: 0, number: 1, value: Some(Value::Rule(main_rule())) },
    ];
    for item in rows {
        assert_eq!(judge(&item, &BOUNDS), Err(Refusal::Value), "{item:?}");
    }
}

#[test]
fn refusal_words_are_distinct() {
    let refusals = [
        Refusal::Kind,
        Refusal::Scope,
        Refusal::Number,
        Refusal::Pattern,
        Refusal::Gates,
        Refusal::Approvals,
        Refusal::Sets,
        Refusal::SetNumber,
        Refusal::SetTwice,
        Refusal::DeploymentSets,
        Refusal::Charter,
        Refusal::Value,
        Refusal::Words,
    ];
    for (at, refusal) in refusals.iter().enumerate() {
        assert!(!refusal.words().is_empty(), "{refusal:?}");
        for prior in refusals.iter().take(at) {
            assert_ne!(refusal.words(), prior.words(), "{refusal:?} and {prior:?}");
        }
    }
}

#[test]
fn requires_is_the_union_of_covering_rules() {
    let rules = [held(PROJECT, FIRST_RULE, main_rule()), held(PROJECT, BOUNDS.landing_rules, goal_rule())];
    let deployment = held(
        0,
        FIRST_RULE,
        rule(
            GatePattern { segments: Box::new([Box::from(b"forge".as_slice())]), last: GateLast::Open(Box::new([])) },
            CI,
        ),
    );
    let rows = [
        ("main", Box::<[Held]>::from(rules.clone()), main_name(), BOTH),
        ("goal", Box::from(rules.clone()), goal_name(), NEITHER),
        ("deployment adds CI", Box::new([rules[0].clone(), rules[1].clone(), deployment]), goal_name(), CI),
        (
            "covering rules add up to date",
            Box::new([
                held(0, FIRST_RULE, LandingRule { ci: false, up_to_date: true, ..main_rule() }),
                held(PROJECT, FIRST_RULE, LandingRule { ci: true, up_to_date: false, ..main_rule() }),
            ]),
            main_name(),
            BOTH,
        ),
    ];
    for (label, held, name, expected) in rows {
        assert_eq!(requires(&held, &repository(), &name, crate::Touched::UNKNOWN), expected, "{label}");
    }
}

#[test]
fn requires_asks_both_where_no_rule_covers() {
    let other = rule(branch_pattern(b"other", GateLast::Open(Box::new([]))), NEITHER);
    let rows: [(&str, Box<[Held]>, Requires); 2] =
        [("no rules", Box::new([]), BOTH), ("another repository", Box::new([held(PROJECT, FIRST_RULE, other)]), BOTH)];
    for (label, held, expected) in rows {
        assert_eq!(requires(&held, &repository(), &main_name(), crate::Touched::UNKNOWN), expected, "{label}");
    }
}

#[test]
fn requires_reads_rules_that_do_not_enforce() {
    let rows = [
        ("CI advice", LandingRule { enforce: false, ci: true, up_to_date: false, ..main_rule() }, CI),
        (
            "update advice",
            LandingRule { enforce: false, ci: false, up_to_date: true, ..main_rule() },
            Requires { ci: false, up_to_date: true },
        ),
        ("neither advice", LandingRule { enforce: false, ci: false, up_to_date: false, ..main_rule() }, NEITHER),
    ];
    for (label, rule, expected) in rows {
        assert_eq!(
            requires(&[held(PROJECT, FIRST_RULE, rule)], &repository(), &main_name(), crate::Touched::UNKNOWN),
            expected,
            "{label}"
        );
    }
}

#[test]
fn a_projects_kinds_are_judged() {
    use crate::settings::{ProjectWays, ProjectWords};
    use temper_engine_domain_forge_change::Route;
    use temper_protocol_reaches::{self as reaches, Classes, Way};
    for (kind, expected) in [
        (6, Value::Questions(Route::None)),
        (8, Value::Classes(Classes::stock(Way::AgentsReview))),
        (7, Value::Ways(ProjectWays { drafts: true, goals: false, changes: false, way: 0 })),
    ] {
        assert_eq!(crate::settings::project_stock(kind).unwrap().value, expected, "stock {kind}");
    }
    let invalid = Classes {
        items: Box::new([reaches::Item {
            subject: reaches::Subject::Kind(reaches::Kind::Money),
            words: Box::new([]),
            class: reaches::Class::Ask,
        }]),
        sets: Box::new([]),
    };
    let values = [
        (6, Value::Questions(Route::None), Ok(())),
        (6, Value::Questions(Route::Foreman), Ok(())),
        (6, Value::Questions(Route::Up), Ok(())),
        (7, Value::Ways(ProjectWays { drafts: true, goals: false, changes: true, way: u32::MAX }), Ok(())),
        (8, Value::Classes(Classes::stock(Way::AgentsReview)), Ok(())),
        (8, Value::Classes(invalid), Err(Refusal::Classes(reaches::Refusal::Fixed { item: 1 }))),
        (
            10,
            Value::ProjectWords(ProjectWords { headline: Box::new([b'h'; 120]), lede: Box::new([b'l'; 480]) }),
            Ok(()),
        ),
    ];
    for (kind, value, set_result) in values {
        assert_eq!(Kind::of(kind).unwrap().number(), kind);
        let base = Item { kind, scope: PROJECT, number: 1, value: Some(value) };
        for (label, item, expected) in [
            ("set", base.clone(), set_result),
            ("scope", Item { scope: 0, ..base.clone() }, Err(Refusal::Scope)),
            ("zero", Item { number: 0, ..base.clone() }, Err(Refusal::Number)),
            ("two", Item { number: 2, ..base.clone() }, Err(Refusal::Number)),
            ("charter", Item { value: Some(Value::Charter(1)), ..base.clone() }, Err(Refusal::Value)),
            ("rule", Item { value: Some(Value::Rule(main_rule())), ..base.clone() }, Err(Refusal::Value)),
            ("remove or reset", Item { value: None, ..base.clone() }, Ok(())),
        ] {
            assert_eq!(judge(&item, &BOUNDS), expected, "{kind}: {label}");
        }
        let other = if kind == 6 || kind == 8 {
            Value::Ways(ProjectWays { drafts: false, goals: false, changes: false, way: 0 })
        } else {
            Value::Questions(Route::None)
        };
        assert_eq!(judge(&Item { value: Some(other), ..base }, &BOUNDS), Err(Refusal::Value));
    }
    let ways = Value::Ways(ProjectWays { drafts: true, goals: false, changes: false, way: 0 });
    for (line, expected) in [
        (Box::from([b'x'; 120].as_slice()), Ok(())),
        (Box::from([b'x'; 121].as_slice()), Err(Refusal::Words)),
        (Box::from(b"".as_slice()), Err(Refusal::Words)),
        (Box::from(b"line\nbreak".as_slice()), Err(Refusal::Words)),
        (Box::from(b"tab\t".as_slice()), Err(Refusal::Words)),
        (Box::from("é\u{0085}".as_bytes()), Err(Refusal::Words)),
        (Box::from(b"\xff".as_slice()), Err(Refusal::Words)),
        (Box::from("é".as_bytes()), Ok(())),
    ] {
        let item = Item { kind: 9, scope: 0, number: 1, value: Some(Value::CharterWords(line)) };
        assert_eq!(judge(&item, &BOUNDS), expected, "line {item:?}");
    }
    let base = Item { kind: 9, scope: 0, number: 1, value: Some(Value::CharterWords(Box::from(b"line".as_slice()))) };
    assert_eq!(Kind::of(9).unwrap().number(), 9);
    for (label, item, expected) in [
        ("scope", Item { scope: PROJECT, ..base.clone() }, Err(Refusal::Scope)),
        ("zero", Item { number: 0, ..base.clone() }, Err(Refusal::Number)),
        ("last charter", Item { number: BOUNDS.charters, ..base.clone() }, Ok(())),
        ("past charters", Item { number: BOUNDS.charters + 1, ..base.clone() }, Err(Refusal::Number)),
        ("remove or reset", Item { value: None, ..base.clone() }, Ok(())),
        ("ways", Item { value: Some(ways.clone()), ..base }, Err(Refusal::Value)),
        ("project ways", Item { kind: 10, scope: PROJECT, number: 1, value: Some(ways) }, Err(Refusal::Value)),
    ] {
        assert_eq!(judge(&item, &BOUNDS), expected, "{label}");
    }
    for (headline, lede, expected) in [
        (Box::from([b'h'; 120].as_slice()), Box::from([b'l'; 480].as_slice()), Ok(())),
        (Box::from([b'h'; 121].as_slice()), Box::from(b"lede".as_slice()), Err(Refusal::Words)),
        (Box::from(b"headline".as_slice()), Box::from([b'l'; 481].as_slice()), Err(Refusal::Words)),
    ] {
        let item = Item {
            kind: 10,
            scope: PROJECT,
            number: 1,
            value: Some(Value::ProjectWords(ProjectWords { headline, lede })),
        };
        assert_eq!(judge(&item, &BOUNDS), expected, "project words {item:?}");
    }
}

pub(super) fn follow() -> crate::settings::Follow {
    use crate::settings::{Follow, FollowCheck, FollowOwner, FollowSource, FollowStart, FollowTarget};
    Follow {
        source: FollowSource {
            repository: skein_forge::api::Repository { forge: 1, repository: 13 },
            branch: b"main".as_slice().into(),
            paths: Box::new([b"src".as_slice().into(), b"include".as_slice().into()]),
        },
        target: FollowTarget {
            repository: skein_forge::api::Repository { forge: 1, repository: 12 },
            branch: b"main".as_slice().into(),
        },
        owner: FollowOwner::Person(4),
        start: FollowStart::Owner,
        producer: None,
        check: FollowCheck {
            workflow: b".forgejo/workflows/tests.yml".as_slice().into(),
            artifact: b"results".as_slice().into(),
            format: temper_engine_domain_forge_tiers::Format::Junit,
        },
    }
}

#[test]
fn follows_judge_numbers_paths_producers_and_checks_at_their_bounds() {
    use crate::settings::FollowStart;
    let base = follow();
    let item = Item { kind: 12, scope: PROJECT, number: 1, value: Some(Value::Follow(base.clone())) };
    assert_eq!(Kind::of(12), Some(Kind::Follow));
    assert_eq!(Kind::Follow.number(), 12);
    assert_eq!(judge(&item, &BOUNDS), Ok(()));
    assert_eq!(judge(&Item { number: BOUNDS.follows, ..item.clone() }, &BOUNDS), Ok(()));
    for number in [0, BOUNDS.follows + 1] {
        assert_eq!(judge(&Item { number, ..item.clone() }, &BOUNDS), Err(Refusal::Number));
    }
    assert_eq!(judge(&Item { scope: 0, ..item.clone() }, &BOUNDS), Err(Refusal::Scope));
    assert_eq!(judge(&Item { value: Some(Value::Charter(1)), ..item.clone() }, &BOUNDS), Err(Refusal::Value));
    assert_eq!(judge(&Item { value: None, ..item.clone() }, &BOUNDS), Ok(()));
    for (paths, expected) in [
        (Box::<[Box<[u8]>]>::default(), Ok(())),
        (Box::new([Box::<[u8]>::from([b'a'; 256])]), Ok(())),
        (Box::new([Box::<[u8]>::from([b'a'; 257])]), Err(Refusal::FollowPath)),
        (Box::new([b"a".as_slice().into(), b"b".as_slice().into(), b"c".as_slice().into()]), Err(Refusal::FollowPaths)),
    ] {
        let mut value = base.clone();
        value.source.paths = paths;
        assert_eq!(judge(&Item { value: Some(Value::Follow(value)), ..item.clone() }, &BOUNDS), expected);
    }
    for path in [b"/src".as_slice(), b"src/", b"a//b", b""] {
        let mut value = base.clone();
        value.source.paths = Box::new([path.into()]);
        assert_eq!(
            judge(&Item { value: Some(Value::Follow(value)), ..item.clone() }, &BOUNDS),
            Err(Refusal::FollowPath),
            "{path:?}"
        );
    }
    for (producer, good) in [
        (b"".as_slice(), true),
        (b";;", true),
        (b"6;3.1 0.0 2;3.0", true),
        (b"6;3.1;", true),
        (b"6", false),
        (b"0;;", false),
        (b"6;256;", false),
        (b"6;3.256;", false),
        (b"6;3  ;", false),
        (b"6;;;", false),
    ] {
        let mut value = base.clone();
        value.producer = Some(producer.into());
        value.start = FollowStart::Code;
        assert_eq!(
            judge(&Item { value: Some(Value::Follow(value)), ..item.clone() }, &BOUNDS),
            if good { Ok(()) } else { Err(Refusal::FollowProducer) },
            "{producer:?}"
        );
    }
    for workflow in [true, false] {
        for (length, good) in [(0, false), (256, true), (257, false)] {
            let mut value = base.clone();
            let member = if workflow { &mut value.check.workflow } else { &mut value.check.artifact };
            *member = vec![b'x'; length].into_boxed_slice();
            assert_eq!(
                judge(&Item { value: Some(Value::Follow(value)), ..item.clone() }, &BOUNDS),
                if good { Ok(()) } else { Err(Refusal::FollowCheck) }
            );
        }
    }
}
