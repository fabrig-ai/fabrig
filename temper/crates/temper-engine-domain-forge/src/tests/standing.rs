//! Held landing rules (temper/docs/design/protocol/configuration.md, 13.3;
//! temper/docs/design/domain/forge.md, 8.7).
use super::*;
use crate::settings::{Asked, Edit, LandingRule, RuleApproval, RuleGate, SizedValue, Value};
use fixtures::{CHANGE, HEAD, opening_row};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as policy;

fn rule() -> LandingRule {
    let repository = adopted();
    LandingRule {
        pattern: GatePattern {
            segments: Box::new([
                Box::from(b"forge".as_slice()),
                repository.host,
                repository.owner,
                repository.name,
                Box::from(b"branch".as_slice()),
            ]),
            last: GateLast::Exact(Box::from(b"main".as_slice())),
        },
        enforce: true,
        ci: false,
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([]),
    }
}

fn keep(d: &mut Domain, scope: u32, number: u32, rule: LandingRule) {
    d.keep_settings(
        &env(),
        None,
        &[Asked {
            kind: if scope == 0 { 1 } else { 2 },
            scope,
            number,
            edit: Edit::Set(SizedValue { value: Value::Rule(rule), size: 128 }),
        }],
        &mut Queue::uncapped(),
    );
}

fn fixture() -> (Domain, ChangeEvidence) {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let row = opening_row();
    let evidence = ChangeEvidence {
        prior: row.change.clone(),
        head: Some(HEAD),
        base_tip: fixtures::BASE,
        contains_base: change::Status::Passed,
        ci: change::Status::Passed,
        gates: Box::new([change::GateReport { number: 1, head: HEAD, status: change::Status::Passed }]),
        reviews: Box::new([]),
        reviews_complete: true,
    };
    outputs(&mut d, Event::Restore { record: Stored::Change(row) });
    (d, evidence)
}

fn reviewers() -> [Reviewer; 1] {
    [Reviewer { person: 7, role: 1, head: HEAD, verdict: skein_forge::api::Verdict::Approve }]
}

fn judged(d: &Domain, requirement: u16, evidence: &ChangeEvidence, reviewers: &[Reviewer]) -> Option<JudgeVerdict> {
    d.judge_landing(5, requirement, 0, CHANGE, HEAD, evidence, reviewers)
}

fn landing(d: &mut Domain) {
    let member = policy::Member {
        change: CHANGE,
        repository: policy::Repository { forge: REPO.forge, repository: REPO.repository },
        pull: 1,
        head: Some(HEAD),
        base: Some(fixtures::BASE),
        candidate: Some(HEAD),
        gates: Box::new([]),
        asked: None,
        landed: None,
    };
    d.landing
        .rows
        .insert(
            61,
            LandingRow {
                landing: policy::Landing {
                    task: 61,
                    project: 5,
                    phase: policy::Phase::Preparing,
                    members: Box::new([member]),
                    preparation: 0,
                    stale_proofs: 0,
                },
                asks: Box::new([None]),
                entries: Box::new([]),
            },
        )
        .expect("landing room");
}

#[test]
fn the_landing_rules_meet_wait_and_refuse_as_their_criteria() {
    use JudgeVerdict::{Met, Refuse, Wait};
    use change::Status::{Failed, Passed, Pending};
    let (mut d, mut evidence) = fixture();
    let mut required = rule();
    required.ci = true;
    required.gates = Box::new([RuleGate { number: 1, blocking: true, freshness: JudgeFreshness::Exact }]);
    required.approvals = Box::new([RuleApproval { role: 1, people: 1, freshness: JudgeFreshness::Exact }]);
    keep(&mut d, 0, 1, required);
    for (ci, gate, approval, complete, expected) in [
        (Passed, Passed, skein_forge::api::Verdict::Approve, true, Met),
        (Pending, Passed, skein_forge::api::Verdict::Approve, true, Wait),
        (Passed, Failed, skein_forge::api::Verdict::Approve, true, Refuse),
        (Pending, Failed, skein_forge::api::Verdict::Approve, true, Refuse),
        (Passed, Pending, skein_forge::api::Verdict::RequestChanges, true, Refuse),
        (Passed, Passed, skein_forge::api::Verdict::Approve, false, Wait),
    ] {
        evidence.ci = ci;
        evidence.gates[0].status = gate;
        evidence.reviews_complete = complete;
        let reviewers = [Reviewer { verdict: approval, ..reviewers()[0] }];
        assert_eq!(judged(&d, 5, &evidence, &reviewers), Some(expected), "{ci:?}, {gate:?}, {approval:?}");
    }
    evidence.ci = Passed;
    evidence.gates[0].status = Passed;
    evidence.reviews_complete = true;
    evidence.head = None;
    assert_eq!(judged(&d, 5, &evidence, &reviewers()), Some(Wait));
    evidence.head = Some(HEAD);
    evidence.gates[0].head = fixtures::BASE;
    assert_eq!(judged(&d, 5, &evidence, &reviewers()), Some(Wait), "exact gates reject old heads");
    d.changes.get_mut(&CHANGE).expect("change").change.clean = Box::new([fixtures::BASE]);
    let mut clean = rule();
    clean.gates = Box::new([RuleGate { number: 1, blocking: true, freshness: JudgeFreshness::Clean }]);
    clean.approvals = Box::new([RuleApproval { role: 1, people: 1, freshness: JudgeFreshness::Clean }]);
    keep(&mut d, 0, 1, clean);
    let old_reviewers = [Reviewer { head: fixtures::BASE, ..reviewers()[0] }];
    assert_eq!(judged(&d, 5, &evidence, &old_reviewers), Some(Met));

    landing(&mut d);
    for (status, expected) in [(Passed, Met), (Pending, Wait), (Failed, Refuse)] {
        evidence.gates[0].status = status;
        assert_eq!(d.judge_landing(5, 5, 0, 61, HEAD, &evidence, &old_reviewers), Some(expected));
    }
    let mut ci = rule();
    ci.ci = true;
    keep(&mut d, 0, 1, ci);
    evidence.ci = Failed;
    assert_eq!(d.judge_landing(5, 5, 0, 61, HEAD, &evidence, &[]), Some(Met), "landing CI judges its receipts");
}

#[test]
fn rules_that_do_not_enforce_hold_nothing() {
    let (mut d, mut evidence) = fixture();
    let mut advice = rule();
    advice.enforce = false;
    advice.ci = true;
    advice.up_to_date = true;
    advice.gates = Box::new([RuleGate { number: 1, blocking: true, freshness: JudgeFreshness::Exact }]);
    advice.approvals = Box::new([RuleApproval { role: 1, people: 1, freshness: JudgeFreshness::Exact }]);
    keep(&mut d, 0, 1, advice);
    evidence.ci = change::Status::Failed;
    evidence.contains_base = change::Status::Failed;
    evidence.gates[0].status = change::Status::Failed;
    for requirement in [5, 6] {
        assert_eq!(judged(&d, requirement, &evidence, &[]), Some(JudgeVerdict::Met));
    }
}

#[test]
fn a_branch_no_enforcing_rule_covers_meets_both() {
    let (mut d, mut evidence) = fixture();
    evidence.head = None;
    for requirement in [5, 6] {
        assert_eq!(judged(&d, requirement, &evidence, &[]), Some(JudgeVerdict::Met));
    }
    let mut other = rule();
    other.pattern.last = GateLast::Exact(Box::from(b"elsewhere".as_slice()));
    other.ci = true;
    other.up_to_date = true;
    keep(&mut d, 0, 1, other);
    landing(&mut d);
    for requirement in [5, 6] {
        assert_eq!(judged(&d, requirement, &evidence, &[]), Some(JudgeVerdict::Met));
        assert_eq!(d.judge_landing(5, requirement, 0, 61, HEAD, &evidence, &[]), Some(JudgeVerdict::Met));
        assert_eq!(d.judge_landing(5, requirement, 0, 61, [9; 32], &evidence, &[]), None);
    }
}

#[test]
fn up_to_date_by_the_landing_rules() {
    let (mut d, mut evidence) = fixture();
    let mut required = rule();
    required.up_to_date = true;
    keep(&mut d, 0, 1, required);
    landing(&mut d);
    for (status, expected) in [
        (change::Status::Failed, JudgeVerdict::Refuse),
        (change::Status::Unknown, JudgeVerdict::Wait),
        (change::Status::Passed, JudgeVerdict::Met),
    ] {
        evidence.contains_base = status;
        assert_eq!(judged(&d, 6, &evidence, &[]), Some(expected));
        assert_eq!(d.judge_landing(5, 6, 0, 61, HEAD, &evidence, &[]), Some(expected));
    }
    keep(&mut d, 0, 1, rule());
    evidence.contains_base = change::Status::Failed;
    assert_eq!(judged(&d, 6, &evidence, &[]), Some(JudgeVerdict::Met));
}

#[test]
fn project_rules_add_to_the_deployments() {
    let (mut d, mut evidence) = fixture();
    let mut ci = rule();
    ci.ci = true;
    keep(&mut d, 0, 1, ci);
    let mut project = rule();
    project.gates = Box::new([RuleGate { number: 1, blocking: true, freshness: JudgeFreshness::Exact }]);
    keep(&mut d, 6, 1, project.clone());
    evidence.gates[0].status = change::Status::Failed;
    assert_eq!(judged(&d, 5, &evidence, &[]), Some(JudgeVerdict::Met), "another project's rules");
    keep(&mut d, 5, 1, project);
    assert_eq!(judged(&d, 5, &evidence, &[]), Some(JudgeVerdict::Refuse));

    landing(&mut d);
    let mut second = opening_row();
    second.task = 52;
    second.base = Box::from(b"goal".as_slice());
    let mut repository = adopted();
    repository.provider.repository = 3;
    repository.name = Box::from(b"another".as_slice());
    second.repository = repository.provider;
    d.repositories.insert(repository.provider, repository.clone()).expect("second repository");
    let mut other_rule = rule();
    other_rule.pattern.segments[3] = repository.name;
    other_rule.ci = true;
    keep(&mut d, 0, 2, other_rule);
    d.changes.insert(52, second).expect("second change");
    let row = d.landing.rows.get_mut(&61).expect("landing");
    let first = row.landing.members[0].clone();
    let mut second = first.clone();
    second.change = 52;
    second.repository.repository = 3;
    second.candidate = Some([8; 32]);
    row.landing.members = Box::new([first, second]);
    row.asks = Box::new([None, None]);
    assert_eq!(d.judge_landing(5, 5, 0, 61, HEAD, &evidence, &[]), Some(JudgeVerdict::Refuse));
    evidence.head = Some([8; 32]);
    evidence.ci = change::Status::Failed;
    assert_eq!(
        d.judge_landing(5, 5, 0, 61, [8; 32], &evidence, &[]),
        Some(JudgeVerdict::Met),
        "member's base selects its rules"
    );
}

#[test]
fn held_gates_block_only_under_enforcing_rules() {
    let mut d = domain();
    let mut gates = rule();
    gates.gates = Box::new([
        RuleGate { number: 1, blocking: true, freshness: JudgeFreshness::Clean },
        RuleGate { number: 2, blocking: false, freshness: JudgeFreshness::Exact },
    ]);
    for enforce in [true, false] {
        gates.enforce = enforce;
        keep(&mut d, 0, 1, gates.clone());
        let selected = d.held_gates(&LIMITS, &adopted(), b"main").expect("gates");
        assert_eq!(selected.len(), 2);
        assert_eq!(
            selected[0],
            change::Gate {
                number: 1,
                kind: change::GateKind::Agent,
                blocking: enforce,
                freshness: change::Freshness::Clean,
                eager: false,
            }
        );
        assert!(!selected[1].blocking);
    }
    gates.enforce = true;
    gates.gates[0].freshness = JudgeFreshness::Exact;
    keep(&mut d, 5, 1, gates);
    let selected = d.held_gates(&LIMITS, &adopted(), b"main").expect("gates");
    assert_eq!(selected.len(), 2, "shared gate selected once");
    assert!(selected[0].blocking, "project enforcement adds to deployment advice");
    assert_eq!(selected[0].freshness, change::Freshness::Exact);
    assert!(d.held_gates(&LIMITS, &adopted(), b"other").expect("uncovered").is_empty());
    assert_eq!(d.held_gates(&LIMITS, &adopted(), b"a//b"), None);
    assert_eq!(
        d.held_gates(
            &Limits { change_policy: change::Limits { gates: 1, ..LIMITS.change_policy }, ..LIMITS },
            &adopted(),
            b"main"
        ),
        None
    );
    let mut repository = adopted();
    repository.ci = false;
    repository.checks = Box::new([1]);
    assert!(d.held_gates(&LIMITS, &repository, b"other").expect("adopted check")[0].blocking);
}

#[test]
fn held_requires_reads_the_held_rules() {
    let mut d = domain();
    let repository = adopted();
    let both = Requires { ci: true, up_to_date: true };
    assert_eq!(d.held_requires(&repository, b"main"), both);
    let mut required = rule();
    required.pattern.last = GateLast::Open(Box::from(b"temper".as_slice()));
    keep(&mut d, 0, 1, required.clone());
    assert_eq!(d.held_requires(&repository, b"temper/1/g1"), Requires { ci: false, up_to_date: false });
    assert_eq!(d.held_requires(&repository, b"main"), both);
    required.enforce = false;
    required.ci = true;
    keep(&mut d, 0, 1, required);
    let mut project = rule();
    project.pattern.last = GateLast::Open(Box::from(b"temper".as_slice()));
    project.up_to_date = true;
    keep(&mut d, 6, 1, project.clone());
    assert_eq!(d.held_requires(&repository, b"temper/1/g1"), Requires { ci: true, up_to_date: false });
    keep(&mut d, 5, 1, project);
    assert_eq!(d.held_requires(&repository, b"temper/1/g1"), both);
    for branch in [b"a//b".as_slice(), b"", b"/a", b"a/"] {
        assert_eq!(d.held_requires(&repository, branch), both);
    }
    assert_eq!(d.requires(&repository, b"temper/1/g1"), both, "compiled rules remain independent");
}

#[test]
fn requirement_5_is_guarded_and_6_observed() {
    let (d, evidence) = fixture();
    for parameters in [0, 1, u32::MAX] {
        assert!(d.guards_landing(5, 5, parameters));
        assert!(!d.guards_landing(5, 6, parameters));
        for requirement in [5, 6] {
            assert_eq!(
                d.judge_landing(5, requirement, parameters, CHANGE, HEAD, &evidence, &[]),
                Some(JudgeVerdict::Met)
            );
        }
    }
}
