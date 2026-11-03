//! Landing-rule values and procedure waits
//! (temper/docs/design/protocol/configuration.md, 13.2 and 13.3).
use crate::settings::{Bounds, Held, Item, Kind, LandingRule, Refusal, RuleApproval, RuleGate, judge, requires};
use crate::{GateLast, GatePattern, JudgeFreshness, Requires};
use alloc::boxed::Box;
use skein_lib::List;

// Tiny admission bounds: two gates and roles, five base segments, eight bytes each.
const BOUNDS: Bounds = Bounds { landing_rules: 4, gates: 2, roles: 2, segments: 5, segment_bytes: 8 };
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
    Item { kind: Kind::DeploymentRules.number(), scope: 0, number: FIRST_RULE, rule: Some(rule) }
}

fn with_rule(rule: LandingRule) -> Item {
    Item { rule: Some(rule), ..bounded_item() }
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
        ("remove deployment", Item { rule: None, ..base.clone() }, Ok(())),
        ("remove project", Item { kind: 2, scope: PROJECT, rule: None, ..base }, Ok(())),
    ];
    for (label, item, expected) in rows {
        assert_eq!(judge(&item, &BOUNDS), expected, "{label}");
    }
    for (number, expected) in [
        (0, None),
        (1, Some(Kind::DeploymentRules)),
        (2, Some(Kind::ProjectRules)),
        (3, None),
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
    let valid = base.rule.clone().expect("rule fixture");
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
        ("kind 3", Item { kind: 3, ..base.clone() }, Err(Refusal::Kind)),
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
        ("removal kind", Item { kind: 3, rule: None, ..base.clone() }, Err(Refusal::Kind)),
        ("removal scope", Item { scope: PROJECT, rule: None, ..base.clone() }, Err(Refusal::Scope)),
        ("removal number", Item { number: 0, rule: None, ..base }, Err(Refusal::Number)),
    ];
    for (label, item, expected) in rows {
        assert_eq!(judge(&item, &BOUNDS), expected, "{label}");
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
