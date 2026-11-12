//! Held landing rules (temper/docs/design/protocol/configuration/settings.md, 3;
//! temper/docs/design/domain/forge/change/gates.md, 2).
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
        2,
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
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
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
        left: None,
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
                    round: temper_engine_domain_forge_landing::Round::new(Box::from(b"temper/1/l61".as_slice())),

                    task: 61,
                    project: 5,
                    phase: policy::Phase::Preparing,
                    members: Box::new([member]),
                    preparation: 0,
                    stale_proofs: 0,
                    branch: Box::from(b"main".as_slice()),
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
fn a_refusal_names_a_persons_request_only_when_an_approval_refuses_first() {
    use JudgeVerdict::{Refuse, Wait};
    use change::Status::{Failed, Passed};
    let older = [8; 32];
    for (requested, clean, freshness, ci, gate, complete, expected, verdict) in [
        (HEAD, [].as_slice(), JudgeFreshness::Exact, Passed, Passed, true, Some(HEAD), Refuse),
        (
            fixtures::BASE,
            [fixtures::BASE].as_slice(),
            JudgeFreshness::Clean,
            Passed,
            Passed,
            true,
            Some(fixtures::BASE),
            Refuse,
        ),
        (
            older,
            [older, fixtures::BASE].as_slice(),
            JudgeFreshness::Clean,
            Passed,
            Passed,
            true,
            Some(fixtures::BASE),
            Refuse,
        ),
        (HEAD, [fixtures::BASE].as_slice(), JudgeFreshness::Clean, Passed, Passed, true, Some(HEAD), Refuse),
        (fixtures::BASE, [].as_slice(), JudgeFreshness::Clean, Passed, Passed, true, None, Wait),
        (fixtures::BASE, [fixtures::BASE].as_slice(), JudgeFreshness::Exact, Passed, Passed, true, None, Wait),
        (HEAD, [].as_slice(), JudgeFreshness::Exact, Failed, Passed, true, None, Refuse),
        (HEAD, [].as_slice(), JudgeFreshness::Exact, Passed, Failed, true, None, Refuse),
        (HEAD, [].as_slice(), JudgeFreshness::Exact, Passed, Passed, false, None, Wait),
    ] {
        let (mut d, mut evidence) = fixture();
        d.changes.get_mut(&CHANGE).expect("change").change.clean = clean.into();
        let mut required = rule();
        required.ci = true;
        required.gates = Box::new([RuleGate { number: 1, blocking: true, freshness: JudgeFreshness::Exact }]);
        required.approvals = Box::new([RuleApproval { role: 1, people: 1, freshness }]);
        keep(&mut d, 0, 1, required);
        evidence.ci = ci;
        evidence.gates[0].status = gate;
        evidence.reviews_complete = complete;
        let mut requests: Box<[Reviewer]> = Box::new([Reviewer {
            head: requested,
            verdict: skein_forge::api::Verdict::RequestChanges,
            ..reviewers()[0]
        }]);
        if requested == older || (requested == HEAD && freshness == JudgeFreshness::Clean) {
            requests = Box::new([requests[0], Reviewer { person: 8, head: fixtures::BASE, ..requests[0] }]);
        }
        assert_eq!(judged(&d, 5, &evidence, &requests), Some(verdict));
        assert_eq!(
            d.review_request(CHANGE, HEAD, &evidence, &requests),
            expected,
            "{requested:?}, {freshness:?}, {ci:?}, {gate:?}, {complete}"
        );
        landing(&mut d);
        assert_eq!(d.review_request(61, HEAD, &evidence, &requests), None, "no repair of a goal landing");
    }
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
fn a_project_branch_rule_covers_repositories_adopted_after_it() {
    let mut d = domain();
    let mut first = adopted();
    first.project = 1;
    outputs(&mut d, Event::Restore { record: Stored::Repository(first.clone()) });
    let mut required = rule();
    required.pattern.segments = Box::new([b"branch".as_slice().into()]);
    required.ci = true;
    required.gates = Box::new([RuleGate { number: 2, blocking: true, freshness: JudgeFreshness::Clean }]);
    keep(&mut d, 1, 2, required);
    let mut later = first.clone();
    later.provider.repository += 1;
    later.name = b"later".as_slice().into();
    outputs(&mut d, Event::Restore { record: Stored::Repository(later.clone()) });
    let mut other = later.clone();
    other.project = 2;
    other.provider.repository += 1;
    other.name = b"other".as_slice().into();
    outputs(&mut d, Event::Restore { record: Stored::Repository(other.clone()) });
    for (repository, branch, covered) in [
        (&first, b"main".as_slice(), true),
        (&later, b"main".as_slice(), true),
        (&later, b"dev".as_slice(), false),
        (&other, b"main".as_slice(), false),
    ] {
        let gates = d.held_gates(&env().limits, repository, branch).expect("bounded gates");
        assert_eq!(gates.len(), usize::from(covered));
        if covered {
            let gate = gates.first().expect("covered gate");
            assert_eq!(gate.number, 2);
            assert!(gate.blocking);
        }
        assert_eq!(d.held_requires(repository, branch), Requires { ci: true, up_to_date: !covered });
    }
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

#[test]
fn owner_attached_requirements_1_to_4_wait_without_a_criteria_table() {
    let (domain, evidence) = fixture();
    for requirement in 1..=4 {
        for parameters in [0, u32::MAX] {
            assert_eq!(
                domain.judge_landing(5, requirement, parameters, CHANGE, HEAD, &evidence, &reviewers()),
                Some(JudgeVerdict::Wait)
            );
            assert!(!domain.guards_landing(5, requirement, parameters));
        }
    }
}

#[test]
fn approval_panel_distinguishes_unread_incomplete_and_counted_reviews() {
    use crate::ReviewRead::{Complete, Incomplete, Unread};
    let (mut d, _) = fixture();
    let mut required = rule();
    required.approvals = Box::new([RuleApproval { role: 1, people: 1, freshness: JudgeFreshness::Clean }]);
    keep(&mut d, 0, 1, required);
    let row = d.change(CHANGE).unwrap();
    assert_eq!(d.panel_approvals(row)[0].read, Unread);
    assert_eq!(d.panel_approvals(row)[0].remaining, None);
    let approved = reviewers()[0];
    let conflict = Reviewer { verdict: skein_forge::api::Verdict::RequestChanges, ..approved };
    for (head, complete, reviews, read, remaining, count, refused) in [
        (Some(fixtures::BASE), true, [approved].as_slice(), Unread, None, 0, false),
        (Some(HEAD), false, [].as_slice(), Incomplete, Some(1), 0, false),
        (Some(HEAD), true, [].as_slice(), Complete, Some(1), 0, false),
        (Some(HEAD), true, [approved, approved].as_slice(), Complete, Some(0), 1, false),
        (Some(HEAD), true, [approved, conflict].as_slice(), Complete, Some(1), 0, true),
    ] {
        let facts =
            ReviewFacts { head, reviews_complete: complete, reviewers: Box::from(reviews), unmapped: Box::new([]) };
        assert!(d.keep_review_facts(env().wall, CHANGE, facts.clone()));
        assert!(!d.keep_review_facts(env().wall, CHANGE, facts), "unchanged reads need no duplicate panel mark");
        let line = &d.panel_approvals(d.change(CHANGE).unwrap())[0];
        assert_eq!((line.read, line.remaining, line.counted.len(), line.refused), (read, remaining, count, refused));
    }
    d.changes.get_mut(&CHANGE).unwrap().change.clean = Box::new([fixtures::BASE]);
    d.keep_review_facts(
        env().wall,
        CHANGE,
        ReviewFacts {
            head: Some(HEAD),
            reviews_complete: true,
            reviewers: Box::new([Reviewer { head: fixtures::BASE, ..approved }]),
            unmapped: Box::new([]),
        },
    );
    let lines = d.panel_approvals(d.change(CHANGE).unwrap());
    assert_eq!(lines[0].counted[0].head, fixtures::BASE, "clean freshness retains the review's head");
    assert_eq!(lines[0].remaining, Some(0));
    keep(&mut d, 0, 1, rule());
    assert!(d.panel_approvals(d.change(CHANGE).unwrap()).is_empty(), "removing a rule removes its line");
}

#[test]
fn approval_waits_are_live_changes_short_of_an_approval_at_their_last_head() {
    use change::State;
    let first = State::First { head: HEAD, ready_since: env().wall };
    let approved = reviewers()[0];
    let refused = Reviewer { verdict: skein_forge::api::Verdict::RequestChanges, ..approved };
    for (state, pull, head, complete, reviews, expected) in [
        (first.clone(), Some(1), Some(Some(HEAD)), true, [].as_slice(), Some((true, 1))),
        (State::Landing { head: HEAD }, Some(1), Some(Some(HEAD)), true, [].as_slice(), Some((true, 1))),
        (State::Queued { head: HEAD, ready_since: env().wall }, Some(1), Some(Some(HEAD)), true, [].as_slice(), None),
        (
            State::Held { was: Box::new(first.clone()), why: change::Hold::Stalled },
            Some(1),
            Some(Some(HEAD)),
            true,
            [].as_slice(),
            None,
        ),
        (first.clone(), Some(1), Some(Some(fixtures::BASE)), true, [].as_slice(), None),
        (first.clone(), Some(1), Some(None), true, [].as_slice(), None),
        (first.clone(), Some(1), None, true, [].as_slice(), None),
        (first.clone(), Some(1), Some(Some(HEAD)), false, [].as_slice(), Some((false, 1))),
        (first.clone(), Some(1), Some(Some(HEAD)), true, [approved].as_slice(), None),
        (first.clone(), Some(1), Some(Some(HEAD)), true, [refused].as_slice(), None),
        (first.clone(), None, Some(Some(HEAD)), true, [].as_slice(), None),
        (State::Ended, Some(1), Some(Some(HEAD)), true, [].as_slice(), None),
    ] {
        let (mut d, _) = fixture();
        let mut required = rule();
        required.approvals = Box::new([RuleApproval { role: 1, people: 1, freshness: JudgeFreshness::Exact }]);
        keep(&mut d, 0, 1, required.clone());
        let row = d.changes.get_mut(&CHANGE).expect("change");
        row.pull = pull;
        // Keep evidence while live so the terminal filter is exercised independently.
        if let Some(head) = head {
            d.keep_review_facts(
                env().wall,
                CHANGE,
                ReviewFacts { head, reviews_complete: complete, reviewers: Box::from(reviews), unmapped: Box::new([]) },
            );
        }
        d.changes.get_mut(&CHANGE).expect("change").change.state = state.clone();
        let waits = d.approval_waits(env().wall, &LIMITS);
        let summary = match waits.first() {
            Some(row) => Some((row.whole, row.short.len())),
            None => None,
        };
        assert_eq!(summary, expected, "{state:?}, {pull:?}, {head:?}, {complete}, {reviews:?}");
        assert_eq!(waits.len(), usize::from(expected.is_some()));
        if let Some(wait) = waits.first() {
            assert_eq!(
                (wait.task, wait.project, wait.forge, wait.pull, wait.since, wait.behind),
                (CHANGE, 5, REPO.forge, 1, env().wall, 0)
            );
            assert_eq!(wait.owner.as_ref(), b"org");
            assert_eq!(wait.name.as_ref(), b"repo");
            assert_eq!(wait.title.as_ref(), b"Small fix");
            assert_eq!(wait.short.as_ref(), d.panel_approvals(d.change(CHANGE).expect("change")).as_ref());
        }
        d.repositories.remove(&REPO);
        assert!(d.approval_waits(env().wall, &LIMITS).is_empty(), "only adopted repositories");
        d.repositories.insert(REPO, adopted()).expect("repository room");
        required.enforce = false;
        keep(&mut d, 0, 1, required);
        assert!(d.approval_waits(env().wall, &LIMITS).is_empty(), "advice holds no approval");
    }
}

#[test]
fn approval_waits_keep_the_heads_first_time_and_count_the_changes_behind() {
    let (mut d, _) = fixture();
    let mut required = rule();
    required.approvals = Box::new([RuleApproval { role: 1, people: 1, freshness: JudgeFreshness::Exact }]);
    keep(&mut d, 0, 1, required.clone());
    let row = d.changes.get_mut(&CHANGE).expect("change");
    row.pull = Some(1);
    row.pending = None;
    row.title = Box::new([b'x'; LOGGED_TITLE_BYTES + 1]);
    row.change.state = change::State::First { head: HEAD, ready_since: env().wall };
    row.change.owns_turn = true;
    let mut second = row.clone();
    second.task = 52;
    second.change.task = 52;
    second.change.owns_turn = false;
    second.change.state = change::State::Queued { head: HEAD, ready_since: env().wall };
    d.changes.insert(52, second).expect("second change");
    let facts =
        ReviewFacts { head: Some(HEAD), reviews_complete: false, reviewers: Box::new([]), unmapped: Box::new([]) };
    let a = Wall::from_nanos(10);
    let b = Wall::from_nanos(20);
    let c = Wall::from_nanos(30);
    assert!(d.keep_review_facts(a, CHANGE, facts.clone()));
    assert!(!d.keep_review_facts(b, CHANGE, facts.clone()));
    assert!(d.keep_review_facts(b, CHANGE, ReviewFacts { reviews_complete: true, ..facts.clone() }));
    let waits = d.approval_waits(env().wall, &LIMITS);
    assert_eq!((waits[0].since, waits[0].behind), (a, 1));
    assert_eq!(waits[0].title.len(), LOGGED_TITLE_BYTES);
    assert!(waits[0].title.ends_with("…".as_bytes()));
    assert_eq!(d.review_since.get(&CHANGE), Some(&a));
    let next = ReviewFacts { head: Some(fixtures::BASE), ..facts.clone() };
    assert!(d.keep_review_facts(c, CHANGE, next));
    assert_eq!(d.review_since.get(&CHANGE), Some(&c));
    assert!(d.approval_waits(env().wall, &LIMITS).is_empty(), "old facts leave the read");
    d.changes.get_mut(&CHANGE).expect("change").change.last_head = Some(fixtures::BASE);
    d.changes.get_mut(&52).expect("second").change.state = change::State::Landing { head: HEAD };
    assert!(d.keep_review_facts(b, 52, facts));
    let waits = d.approval_waits(env().wall, &LIMITS);
    assert_eq!(waits.len(), 2);
    assert_eq!([(waits[0].task, waits[0].since), (waits[1].task, waits[1].since)], [(52, b), (CHANGE, c)]);
    assert!(d.keep_review_facts(
        c,
        52,
        ReviewFacts {
            head: Some(fixtures::BASE),
            reviews_complete: true,
            reviewers: Box::new([]),
            unmapped: Box::new([])
        }
    ));
    d.changes.get_mut(&52).expect("second").change.last_head = Some(fixtures::BASE);
    let waits = d.approval_waits(env().wall, &LIMITS);
    assert_eq!(waits.len(), 2);
    assert_eq!([waits[0].task, waits[1].task], [CHANGE, 52]);
    // One met requirement is omitted while the remaining lines keep the panel's order.
    required.approvals = Box::new([
        RuleApproval { role: 1, people: 1, freshness: JudgeFreshness::Exact },
        RuleApproval { role: 2, people: 2, freshness: JudgeFreshness::Exact },
        RuleApproval { role: 3, people: 1, freshness: JudgeFreshness::Exact },
    ]);
    keep(&mut d, 0, 1, required);
    d.keep_review_facts(
        c,
        CHANGE,
        ReviewFacts {
            head: Some(fixtures::BASE),
            reviews_complete: true,
            reviewers: Box::new([Reviewer { head: fixtures::BASE, ..reviewers()[0] }]),
            unmapped: Box::new([]),
        },
    );
    let waits = d.approval_waits(env().wall, &LIMITS);
    assert_eq!(waits[0].short.len(), 2);
    assert_eq!([waits[0].short[0].role, waits[0].short[1].role], [2, 3]);
    d.keep_review_facts(
        c,
        CHANGE,
        ReviewFacts {
            head: Some(fixtures::BASE),
            reviews_complete: true,
            reviewers: Box::new([Reviewer {
                role: 2,
                head: fixtures::BASE,
                verdict: skein_forge::api::Verdict::RequestChanges,
                ..reviewers()[0]
            }]),
            unmapped: Box::new([]),
        },
    );
    assert_eq!(
        d.approval_waits(env().wall, &LIMITS).len(),
        1,
        "a refusal excludes the change even with other short approvals"
    );
    let row = d.changes.get_mut(&CHANGE).expect("change");
    row.change.state = change::State::Ended;
    let row = row.clone();
    domain::save_change(&mut d, &mut Queue::uncapped(), row);
    assert!(!d.review_facts.contains_key(&CHANGE));
    assert!(!d.review_since.contains_key(&CHANGE));
}
