//! Values of the forge's kinds and procedure waits
//! (temper/docs/design/protocol/configuration.md, 13.2 and 13.3).
use crate::settings::{Bounds, Held, Item, Kind, LandingRule, Refusal, RuleApproval, RuleGate, Value, judge, requires};
use crate::{GateLast, GatePattern, JudgeFreshness, Requires};
use alloc::boxed::Box;
use skein_lib::List;

// Tiny admission bounds: two gates and roles, five base segments, eight bytes each.
const BOUNDS: Bounds = Bounds { landing_rules: 4, gates: 2, roles: 2, charters: 3, segments: 5, segment_bytes: 8 };
const PROJECT: u32 = 1;
const FIRST_RULE: u32 = 1;
const GATE: RuleGate = RuleGate { number: 1, blocking: true, freshness: JudgeFreshness::Exact };
const APPROVAL: RuleApproval = RuleApproval { role: 0, people: 1, freshness: JudgeFreshness::Clean };
const BOTH: Requires = Requires { ci: true, up_to_date: true };
const NEITHER: Requires = Requires { ci: false, up_to_date: false };
const CI: Requires = Requires { ci: true, up_to_date: false };

fn rule(pattern: GatePattern, asks: Requires) -> LandingRule {
    LandingRule {
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

fn main_name() -> Box<[Box<[u8]>]> {
    branch_name(&[b"main"])
}

fn goal_name() -> Box<[Box<[u8]>]> {
    branch_name(&[b"temper", b"1", b"g1"])
}

fn branch_name(branch: &[&[u8]]) -> Box<[Box<[u8]>]> {
    // Five base segments and up to three segments of the goal branch.
    const NAME_SEGMENTS: u32 = 8;
    let pattern = branch_pattern(b"name", GateLast::Exact(Box::new([])));
    let mut name = List::with_capacity(NAME_SEGMENTS);
    for segment in pattern.segments {
        name.push(segment).expect("base fits name fixture");
    }
    for segment in branch {
        name.push(Box::from(*segment)).expect("branch fits name fixture");
    }
    name.into_boxed()
}

#[test]
fn judge_accepts_rules_within_their_bounds() {
    let base = bounded_item();
    let rows = [
        ("deployment first", base.clone(), Ok(())),
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
        Refusal::Charter,
        Refusal::Value,
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
        assert_eq!(requires(&held, &name), expected, "{label}");
    }
}

#[test]
fn requires_asks_both_where_no_rule_covers() {
    let other = rule(branch_pattern(b"other", GateLast::Open(Box::new([]))), NEITHER);
    let rows: [(&str, Box<[Held]>, Requires); 2] =
        [("no rules", Box::new([]), BOTH), ("another repository", Box::new([held(PROJECT, FIRST_RULE, other)]), BOTH)];
    for (label, held, expected) in rows {
        assert_eq!(requires(&held, &main_name()), expected, "{label}");
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
        assert_eq!(requires(&[held(PROJECT, FIRST_RULE, rule)], &main_name()), expected, "{label}");
    }
}
