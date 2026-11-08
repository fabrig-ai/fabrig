//! A branch's path (temper/docs/design/domain/forge.md, 8.13).
#![expect(clippy::disallowed_methods, reason = "ordinary tests build and inspect bounded fixture paths")]
use super::invariants::bounds;
use super::*;
use Actor::{Agent as A, Bar as B, Code as C, Person as P};
use temper_engine_domain_forge::Follows;
use temper_engine_domain_forge_change::Way;

pub(crate) fn inputs(way: usize, ci: bool, tiers: bool, by: Way, prove: bool) -> Inputs {
    // Configuration 13.10's stock table, laid through the surface a read uses.
    let ways = stock_ways();
    let items = items(ways.get(way).expect("stock way"), PROJECT, None);
    let ways = laid_ways(None, PROJECT, &items);
    Inputs {
        branch: b"main".as_slice().into(),
        repository: Some(Repository {
            host: b"forge".as_slice().into(),
            owner: b"owner".as_slice().into(),
            name: b"name".as_slice().into(),
            ci,
            checks: Box::default(),
            gates: if prove { Box::new([b"unit".as_slice().into()]) } else { Box::default() },
            tiers: if tiers {
                Box::new([
                    Tier { name: b"slow".as_slice().into(), follows: Follows::Landing },
                    Tier { name: b"goal".as_slice().into(), follows: Follows::Goals },
                ])
            } else {
                Box::default()
            },
        }),
        rules: laid(&[], &[], PROJECT, &items),
        start: if ways.goals && ways.way == 4 { Start::Goal { charter: 5 } } else { Start::Chat },
        producer: 1,
        reviewer: 2,
        by,
        roles: Box::new([
            Role { number: 0, name: b"owner".as_slice().into(), yours: true },
            Role { number: 1, name: b"maintainer".as_slice().into(), yours: false },
        ]),
        agents: [1, 2, 5]
            .map(|charter| Agent {
                charter,
                model: Some(b"Model".as_slice().into()),
                effort: Some(b"high".as_slice().into()),
                usable: true,
                can: can(true, false, false, false).into(),
                asked: b"Review the change.".as_slice().into(),
            })
            .into(),
    }
}

fn expected(path: &Path, words: &[(Actor, &str)], row: &str) {
    let drawn: Box<[_]> = path
        .stops
        .iter()
        .map(|stop| (stop.actor.clone(), core::str::from_utf8(&stop.words).expect("stop words")))
        .collect();
    assert_eq!(drawn.as_ref(), words, "{row}");
}

#[test]
#[expect(clippy::too_many_lines, reason = "each path is a row in the derivation table")]
#[expect(clippy::type_complexity, reason = "rows pair inputs with ordered actor and word pairs")]
fn paths_follow_the_stock_and_branch_rules_in_order() {
    // The look request's three pages, with "verify" added to its first two
    // paths as plan.md's "What was decided" requires.
    let mut maintainer = inputs(0, false, false, Way::Queue, false);
    maintainer.rules.first_mut().unwrap().rule.approvals =
        Box::new([RuleApproval { role: 1, people: 2, freshness: JudgeFreshness::Clean }]);
    let mut advisory = inputs(1, false, false, Way::Queue, false);
    advisory.rules.first_mut().unwrap().rule.gates.first_mut().unwrap().blocking = false;
    let mut unenforced = inputs(1, true, false, Way::Queue, false);
    let rule = &mut unenforced.rules.first_mut().unwrap().rule;
    rule.enforce = false;
    rule.approvals = Box::new([RuleApproval { role: 0, people: 2, freshness: JudgeFreshness::Clean }]);
    let mut deployment = inputs(0, true, false, Way::Queue, false);
    deployment.rules.first_mut().unwrap().scope = 0;
    let mut no_repository = inputs(1, true, true, Way::Queue, true);
    no_repository.repository = None;
    let mut forge_rule = held(0, 8, true);
    forge_rule.rule.pattern =
        GatePattern { segments: Box::new([b"forge".as_slice().into()]), last: GateLast::Open(Box::default()) };
    forge_rule.rule.approvals = Box::new([RuleApproval { role: 1, people: 2, freshness: JudgeFreshness::Clean }]);
    no_repository.rules = Box::new([forge_rule, no_repository.rules.first().unwrap().clone()]);
    let mut checks = inputs(1, false, false, Way::Queue, false);
    checks.repository.as_mut().unwrap().checks = Box::new([1, 3, 3]);
    let mut duplicate = inputs(1, false, false, Way::Queue, false);
    let mut later = duplicate.rules.first().unwrap().clone();
    duplicate.rules.first_mut().unwrap().rule.enforce = false;
    later.number = 2;
    duplicate.rules = Box::new([duplicate.rules.first().unwrap().clone(), later]);
    let mut forge = inputs(1, false, false, Way::Queue, false);
    forge.rules.first_mut().unwrap().rule.pattern = GatePattern {
        segments: [b"forge".as_slice(), b"forge", b"owner", b"name", b"branch"].map(Box::from).into(),
        last: GateLast::Exact(b"main".as_slice().into()),
    };
    let mut uncovered = forge.clone();
    uncovered.branch = b"other".as_slice().into();
    let mut owner = maintainer.clone();
    owner.roles.get_mut(1).unwrap().name = b"Owner".as_slice().into();
    let rows: &[(&str, Inputs, &[(Actor, &str)])] = &[
        (
            "Agents review, CI and tiers",
            inputs(1, true, true, Way::Queue, false),
            &[
                (P, "you ask"),
                (A, "produce"),
                (C, "check"),
                (A, "review"),
                (A, "repair"),
                (C, "check"),
                (C, "update"),
                (A, "resolve"),
                (C, "merge"),
                (C, "verify"),
                (B, "main"),
            ],
        ),
        (
            "You approve, CI and tiers",
            inputs(0, true, true, Way::Queue, false),
            &[
                (P, "you ask"),
                (A, "produce"),
                (C, "check"),
                (A, "repair"),
                (C, "check"),
                (P, "you approve"),
                (C, "update"),
                (A, "resolve"),
                (C, "merge"),
                (C, "verify"),
                (B, "main"),
            ],
        ),
        (
            "Draft first, CI and tiers",
            inputs(2, true, true, Way::Queue, false),
            &[
                (P, "draft"),
                (A, "plan"),
                (A, "produce ×n"),
                (C, "merge"),
                (C, "check"),
                (A, "repair"),
                (B, "goal branch"),
                (C, "check"),
                (C, "update"),
                (A, "resolve"),
                (C, "merge"),
                (C, "verify"),
                (B, "main"),
            ],
        ),
        (
            "Agents review, no CI or tiers",
            inputs(1, false, false, Way::Queue, false),
            &[
                (P, "you ask"),
                (A, "produce"),
                (A, "review"),
                (A, "repair"),
                (C, "update"),
                (A, "resolve"),
                (C, "merge"),
                (B, "main"),
            ],
        ),
        (
            "You approve, no CI or tiers",
            inputs(0, false, false, Way::Queue, false),
            &[
                (P, "you ask"),
                (A, "produce"),
                (P, "you approve"),
                (C, "update"),
                (A, "resolve"),
                (C, "merge"),
                (B, "main"),
            ],
        ),
        (
            "Draft first, no CI or tiers",
            inputs(2, false, false, Way::Queue, false),
            &[
                (P, "draft"),
                (A, "plan"),
                (A, "produce ×n"),
                (C, "merge"),
                (B, "goal branch"),
                (C, "update"),
                (A, "resolve"),
                (C, "merge"),
                (B, "main"),
            ],
        ),
        (
            "Agents review, turn unit",
            inputs(1, true, false, Way::Turn, true),
            &[
                (P, "you ask"),
                (A, "produce"),
                (A, "review"),
                (A, "repair"),
                (C, "prepare"),
                (A, "resolve"),
                (C, "prove"),
                (C, "turn"),
                (B, "main"),
            ],
        ),
        (
            "Agents review, turn no gates",
            inputs(1, true, false, Way::Turn, false),
            &[
                (P, "you ask"),
                (A, "produce"),
                (A, "review"),
                (A, "repair"),
                (C, "prepare"),
                (A, "resolve"),
                (C, "turn"),
                (B, "main"),
            ],
        ),
        (
            "You approve, turn unit",
            inputs(0, true, false, Way::Turn, true),
            &[
                (P, "you ask"),
                (A, "produce"),
                (P, "you approve"),
                (C, "prepare"),
                (A, "resolve"),
                (C, "prove"),
                (A, "repair"),
                (C, "turn"),
                (B, "main"),
            ],
        ),
        (
            "maintainer approval",
            maintainer,
            &[
                (P, "you ask"),
                (A, "produce"),
                (P, "a maintainer approves ×2"),
                (C, "update"),
                (A, "resolve"),
                (C, "merge"),
                (B, "main"),
            ],
        ),
        (
            "vowel approval",
            owner,
            &[
                (P, "you ask"),
                (A, "produce"),
                (P, "an Owner approves ×2"),
                (C, "update"),
                (A, "resolve"),
                (C, "merge"),
                (B, "main"),
            ],
        ),
        (
            "advisory gate",
            advisory,
            &[
                (P, "you ask"),
                (A, "produce"),
                (A, "review"),
                (A, "repair"),
                (C, "update"),
                (A, "resolve"),
                (C, "merge"),
                (B, "main"),
            ],
        ),
        (
            "unenforced adds only its gate",
            unenforced,
            &[(P, "you ask"), (A, "produce"), (A, "review"), (A, "repair"), (C, "merge"), (B, "main")],
        ),
        (
            "deployment rule",
            deployment,
            &[
                (P, "you ask"),
                (A, "produce"),
                (C, "check"),
                (A, "repair"),
                (C, "check"),
                (P, "you approve"),
                (C, "update"),
                (A, "resolve"),
                (C, "merge"),
                (B, "main"),
            ],
        ),
        (
            "no repository skips forge rules",
            no_repository,
            &[
                (P, "you ask"),
                (A, "produce"),
                (A, "review"),
                (A, "repair"),
                (C, "update"),
                (A, "resolve"),
                (C, "merge"),
                (B, "main"),
            ],
        ),
        (
            "replacement checks deduplicate and replace review",
            checks,
            &[
                (P, "you ask"),
                (A, "produce"),
                (A, "check"),
                (A, "check"),
                (A, "repair"),
                (C, "update"),
                (A, "resolve"),
                (C, "merge"),
                (B, "main"),
            ],
        ),
        (
            "duplicate gate later enforced",
            duplicate,
            &[
                (P, "you ask"),
                (A, "produce"),
                (A, "review"),
                (A, "repair"),
                (C, "update"),
                (A, "resolve"),
                (C, "merge"),
                (B, "main"),
            ],
        ),
        (
            "forge exact rule",
            forge,
            &[
                (P, "you ask"),
                (A, "produce"),
                (A, "review"),
                (A, "repair"),
                (C, "update"),
                (A, "resolve"),
                (C, "merge"),
                (B, "main"),
            ],
        ),
        (
            "uncovered branch has no rule stops",
            uncovered,
            &[(P, "you ask"), (A, "produce"), (C, "merge"), (B, "other")],
        ),
        (
            "goal queue before landing turn",
            inputs(2, true, true, Way::Turn, true),
            &[
                (P, "draft"),
                (A, "plan"),
                (A, "produce ×n"),
                (C, "merge"),
                (C, "check"),
                (A, "repair"),
                (B, "goal branch"),
                (C, "prepare"),
                (A, "resolve"),
                (C, "prove"),
                (C, "turn"),
                (C, "verify"),
                (B, "main"),
            ],
        ),
    ];
    for (name, inputs, words) in rows {
        expected(&derive(inputs), words, name);
    }
}

fn find(path: &Path, step: Step) -> &Stop {
    path.stops.iter().find(|stop| stop.step == step).expect("named stop")
}

fn facts(values: &[(&str, &str)]) -> Box<[Fact]> {
    values
        .iter()
        .map(|(label, value)| Fact { label: label.as_bytes().into(), value: value.as_bytes().into() })
        .collect()
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks cards and their bounds")]
fn cards_open_the_setting_and_stay_within_the_bounds() {
    let review = inputs(1, true, true, Way::Queue, false);
    let approved = inputs(0, false, false, Way::Queue, false);
    let mut deployment = review.clone();
    deployment.rules.first_mut().unwrap().scope = 0;
    let mut advisory = review.clone();
    advisory.rules.first_mut().unwrap().rule.enforce = false;
    let mut duplicate = advisory.clone();
    duplicate.rules = Box::new([advisory.rules.first().unwrap().clone(), review.rules.first().unwrap().clone()]);
    let mut missing = review.clone();
    missing.agents = Box::default();
    let mut no_account = review.clone();
    no_account.agents.get_mut(1).unwrap().model = None;
    no_account.agents.get_mut(1).unwrap().effort = None;
    no_account.agents.get_mut(1).unwrap().usable = false;
    let turn = inputs(0, true, false, Way::Turn, true);
    let rows = [
        (
            &review,
            Step::Review,
            Opens::Gate(1),
            "an agent · holds a change until it approves",
            "the reviewer",
            facts(&[
                ("Model", "Model · high"),
                ("Can", "read the code and the forge; never merge or update"),
                ("Asked to", "“Review the change.”"),
            ]),
        ),
        (
            &review,
            Step::Merge,
            Opens::Queue,
            "code · lands the change",
            "the queue",
            facts(&[("Does", "merges exactly the commit its checks saw")]),
        ),
        (
            &approved,
            Step::Approve,
            Opens::Rule { kind: 2, scope: PROJECT, number: 1 },
            "a person · decides whether it lands",
            "owner",
            facts(&[("Role", "owner"), ("Asks", "1 of them")]),
        ),
        (&review, Step::Branch, Opens::Nothing, "a branch · where the change lands", "main", facts(&[])),
        (
            &deployment,
            Step::Check,
            Opens::Rule { kind: 1, scope: 0, number: 1 },
            "code · holds a change until CI passes",
            "CI",
            facts(&[("Checks", "the repository's CI, at the change's head")]),
        ),
        (
            &advisory,
            Step::Review,
            Opens::Gate(1),
            "an agent · advises; holds nothing",
            "the reviewer",
            facts(&[
                ("Model", "Model · high"),
                ("Can", "read the code and the forge; never merge or update"),
                ("Asked to", "“Review the change.”"),
            ]),
        ),
        (
            &duplicate,
            Step::Review,
            Opens::Gate(1),
            "an agent · holds a change until it approves",
            "the reviewer",
            facts(&[
                ("Model", "Model · high"),
                ("Can", "read the code and the forge; never merge or update"),
                ("Asked to", "“Review the change.”"),
            ]),
        ),
        (
            &missing,
            Step::Review,
            Opens::Gate(1),
            "an agent · holds a change until it approves",
            "the reviewer",
            facts(&[("Charter", "charter 2")]),
        ),
        (
            &no_account,
            Step::Review,
            Opens::Gate(1),
            "an agent · holds a change until it approves",
            "the reviewer",
            facts(&[
                ("Model", "no model · no account runs it now"),
                ("Can", "read the code and the forge; never merge or update"),
                ("Asked to", "“Review the change.”"),
            ]),
        ),
        (
            &turn,
            Step::Prove,
            Opens::Turn,
            "code · runs the test gates at the candidate",
            "the turn",
            facts(&[("Gates", "unit")]),
        ),
        (
            &review,
            Step::Verify,
            Opens::Tier(b"slow".as_slice().into()),
            "code · checks what landed",
            "slow",
            facts(&[("Runs", "the tier slow after each landing")]),
        ),
    ];
    for (inputs, step, opens, who, name, facts) in rows {
        let path = derive(inputs);
        let stop = find(&path, step);
        assert_eq!(stop.opens, opens);
        assert_eq!(stop.card, Card { who: who.as_bytes().into(), name: name.as_bytes().into(), facts });
    }
    for length in [0, WORDS, VALUE, VALUE + 7] {
        let mut inputs = inputs(0, true, true, Way::Queue, false);
        inputs.branch = "é".repeat(length).into_bytes().into();
        for role in &mut inputs.roles {
            role.name = inputs.branch.clone();
        }
        for agent in &mut inputs.agents {
            agent.model = Some(inputs.branch.clone());
            agent.effort = Some(inputs.branch.clone());
            agent.can = inputs.branch.clone();
            agent.asked = inputs.branch.clone();
        }
        let rule = &mut inputs.rules.first_mut().unwrap().rule;
        rule.gates =
            (0..40).map(|number| RuleGate { number, blocking: true, freshness: JudgeFreshness::Exact }).collect();
        let path = derive(&inputs);
        bounds(&path);
        if length > 0 {
            assert_eq!(path.stops.len(), STOPS);
            assert_eq!(path.stops.last().unwrap().words.as_ref(), skein_lib::bytes::utf8_prefix(&inputs.branch, WORDS));
        }
    }
    let mut long_model = inputs(1, true, true, Way::Queue, false);
    let agent = long_model.agents.first_mut().unwrap();
    agent.model = Some(("x".to_owned() + &"€".repeat(128)).into_bytes().into());
    agent.effort = None;
    let path = derive(&long_model);
    let model = find(&path, Step::Produce).card.facts.first().unwrap();
    assert_eq!(model.value.as_ref(), ("x".to_owned() + &"€".repeat(42)).as_bytes());
    let mut invalid = inputs(1, true, true, Way::Queue, false);
    invalid.branch = [0x80; 400].into();
    invalid.agents.get_mut(1).unwrap().asked = [0x80; 400].into();
    bounds(&derive(&invalid));
}
