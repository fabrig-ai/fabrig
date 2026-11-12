//! A branch's path (temper/docs/design/domain/forge/change/branch-path.md).
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
        sets: Box::default(),
        branch: b"main".as_slice().into(),
        repository: Some(Repository {
            host: b"forge".as_slice().into(),
            owner: b"owner".as_slice().into(),
            name: b"name".as_slice().into(),
            ci,
            checks: Box::default(),
            contexts: Box::default(),
            workflows: Box::default(),
            gates: if prove { Box::new([b"unit".as_slice().into()]) } else { Box::default() },
            tiers: if tiers {
                Box::new([
                    Tier { name: b"slow".as_slice().into(), follows: Follows::Landing, description: Box::default() },
                    Tier { name: b"goal".as_slice().into(), follows: Follows::Goals, description: Box::default() },
                ])
            } else {
                Box::default()
            },
        }),
        rules: laid(&[], &[], PROJECT, &items),
        start: if ways.goals && ways.way == 4 {
            Start::Goal { charter: 5, design: false, close_out: false }
        } else {
            Start::Chat
        },
        producer: 1,
        reviewer: 2,
        by,
        merge_job: false,
        roles: Box::new([
            Role { number: 0, name: b"owner".as_slice().into(), yours: true, holders: 1 },
            Role { number: 1, name: b"maintainer".as_slice().into(), yours: false, holders: 0 },
        ]),
        agents: [1, 2, 5]
            .map(|charter| Agent {
                charter,
                model: Some(b"Model".as_slice().into()),
                effort: Some(b"high".as_slice().into()),
                usable: true,
                can: can(true, false, false, false, false).into(),
                asked: b"Review the change.".as_slice().into(),
            })
            .into(),
    }
}

fn expected(path: &Path, words: &[(Actor, &str)], row: &str) {
    let drawn: Box<[_]> = path
        .stops
        .iter()
        .map(|stop| {
            let words = core::str::from_utf8(&stop.words).expect("stop words");
            let over = core::str::from_utf8(&stop.over).expect("over words");
            (stop.actor.clone(), words, stop.needed, over)
        })
        .collect();
    let expected: Box<[_]> = words
        .iter()
        .map(|(actor, words)| {
            let (step, over) = words.split_once(';').unwrap_or((words, ""));
            (actor.clone(), step, ["repair", "conflict", "resolve", "catch up"].contains(&step), over)
        })
        .collect();
    assert_eq!(drawn, expected, "{row}");
}

#[test]
#[expect(clippy::too_many_lines, reason = "each path is a row in the derivation table")]
#[expect(clippy::type_complexity, reason = "rows pair inputs with ordered actor and word pairs")]
fn paths_follow_the_stock_and_branch_rules_in_order() {
    // The look request's three pages, with "verify" added to its first two
    // paths, since main's tier follows main whatever the way
    // (temper/docs/design/domain/forge/change/branch-path.md).
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
    let mut catch_up = inputs(2, true, true, Way::Queue, false);
    catch_up.merge_job = true;
    let mut designed = catch_up.clone();
    designed.start = Start::Goal { charter: 5, design: true, close_out: true };
    let mut close_turn = inputs(2, true, true, Way::Turn, true);
    close_turn.start = Start::Goal { charter: 5, design: false, close_out: true };
    let rows: &[(&str, Inputs, &[(Actor, &str)])] = &[
        (
            "Agents review, CI and tiers",
            inputs(1, true, true, Way::Queue, false),
            &[
                (P, "you ask"),
                (A, "produce"),
                (C, "check"),
                (A, "review"),
                (A, "repair;tests, review"),
                (C, "check"),
                (C, "conflict"),
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
                (A, "repair;tests"),
                (C, "check"),
                (P, "you approve"),
                (C, "conflict"),
                (A, "resolve"),
                (C, "merge"),
                (C, "verify"),
                (B, "main"),
            ],
        ),
        (
            "Draft first, CI and tiers",
            catch_up,
            &[
                (P, "draft"),
                (A, "plan"),
                (A, "produce ×n"),
                (C, "merge;into goal"),
                (C, "check"),
                (A, "repair;tests"),
                (C, "catch up"),
                (B, "goal branch"),
                (C, "check"),
                (C, "conflict"),
                (A, "resolve"),
                (C, "merge"),
                (C, "verify"),
                (B, "main"),
            ],
        ),
        (
            "Draft first, design and close-out, CI and tiers",
            designed,
            &[
                (P, "draft"),
                (A, "design"),
                (A, "produce ×n"),
                (C, "merge;into goal"),
                (C, "check"),
                (A, "repair;tests"),
                (C, "catch up"),
                (B, "goal branch"),
                (C, "check"),
                (C, "conflict"),
                (A, "resolve"),
                (C, "merge"),
                (A, "close-out"),
                (C, "verify"),
                (B, "main"),
            ],
        ),
        (
            "goal close-out after landing turn",
            close_turn,
            &[
                (P, "draft"),
                (A, "plan"),
                (A, "produce ×n"),
                (C, "merge;into goal"),
                (C, "check"),
                (A, "repair;tests"),
                (B, "goal branch"),
                (C, "prepare"),
                (A, "resolve"),
                (C, "prove"),
                (C, "turn"),
                (A, "close-out"),
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
                (A, "repair;review"),
                (C, "conflict"),
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
                (C, "conflict"),
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
                (C, "merge;into goal"),
                (B, "goal branch"),
                (C, "conflict"),
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
                (A, "repair;review"),
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
                (A, "repair;review"),
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
                (A, "repair;tests"),
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
                (C, "conflict"),
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
                (C, "conflict"),
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
                (A, "repair;review"),
                (C, "conflict"),
                (A, "resolve"),
                (C, "merge"),
                (B, "main"),
            ],
        ),
        (
            "unenforced adds only its gate",
            unenforced,
            &[(P, "you ask"), (A, "produce"), (A, "review"), (A, "repair;review"), (C, "merge"), (B, "main")],
        ),
        (
            "deployment rule",
            deployment,
            &[
                (P, "you ask"),
                (A, "produce"),
                (C, "check"),
                (A, "repair;tests"),
                (C, "check"),
                (P, "you approve"),
                (C, "conflict"),
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
                (A, "repair;review"),
                (C, "conflict"),
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
                (A, "repair;tests"),
                (C, "conflict"),
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
                (A, "repair;review"),
                (C, "conflict"),
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
                (A, "repair;review"),
                (C, "conflict"),
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
                (C, "merge;into goal"),
                (C, "check"),
                (A, "repair;tests"),
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
    let mut stock_review = review.clone();
    let reviewer_sentence = "Review the change at the named head: read its pull request, the files it changes and its checks, and judge them against what the change was asked to do.";
    stock_review.agents.get_mut(1).unwrap().asked = reviewer_sentence.as_bytes().into();
    let mut described = review.clone();
    described.repository.as_mut().unwrap().tiers.first_mut().unwrap().description =
        b"the slow suites".as_slice().into();
    let mut goal_described = inputs(2, true, true, Way::Queue, false);
    goal_described.repository.as_mut().unwrap().tiers.get_mut(1).unwrap().description =
        b"the gate without its slowest suites".as_slice().into();
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
    let mut one_context = deployment.clone();
    one_context.repository.as_mut().unwrap().contexts = Box::new([b"ci / test".as_slice().into()]);
    let mut three_contexts = deployment.clone();
    three_contexts.repository.as_mut().unwrap().contexts =
        [b"build".as_slice(), b"lint", b"test"].map(Box::from).into();
    let mut deployment_workflows = deployment.clone();
    deployment_workflows.repository.as_mut().unwrap().workflows = Box::new([b"ci.yml".as_slice().into()]);
    let mut three_workflows = three_contexts.clone();
    three_workflows.repository.as_mut().unwrap().workflows =
        [b"build.yml".as_slice(), b"lint.yml", b"test.yml"].map(Box::from).into();
    let mut goal = inputs(2, true, true, Way::Queue, false);
    goal.merge_job = true;
    let mut goal_conflict = goal.clone();
    goal_conflict.rules.get_mut(1).unwrap().rule.up_to_date = true;
    let mut other = review.clone();
    other.branch = b"release".as_slice().into();
    let mut designed = goal.clone();
    designed.start = Start::Goal { charter: 5, design: true, close_out: true };
    let mut missing_coordinator = designed.clone();
    missing_coordinator.agents = Box::default();
    let mut release = designed.clone();
    release.branch = b"release".as_slice().into();
    let rows = [
        (
            &designed,
            Step::Design,
            Opens::Charter(5),
            "an agent · designs the goal, then cuts it into changes",
            "the coordinator",
            facts(&[
                ("Model", "Model · high"),
                ("Can", "read the code and the forge; never merge"),
                ("Asked to", "“Review the change.”"),
                (
                    "Its order",
                    "the design is delivered first, as a change into the goal's branch; the changes start from it and merge after it",
                ),
            ]),
        ),
        (
            &designed,
            Step::CloseOut,
            Opens::Charter(5),
            "an agent · after the landing, brings what the goal leaves up to date",
            "the coordinator",
            facts(&[
                ("Model", "Model · high"),
                ("Can", "read the code and the forge; never merge"),
                ("Asked to", "“Review the change.”"),
                ("Lands", "as one change into main, by main's rules"),
            ]),
        ),
        (
            &release,
            Step::CloseOut,
            Opens::Charter(5),
            "an agent · after the landing, brings what the goal leaves up to date",
            "the coordinator",
            facts(&[
                ("Model", "Model · high"),
                ("Can", "read the code and the forge; never merge"),
                ("Asked to", "“Review the change.”"),
                ("Lands", "as one change into release, by release's rules"),
            ]),
        ),
        (
            &missing_coordinator,
            Step::Design,
            Opens::Charter(5),
            "an agent · designs the goal, then cuts it into changes",
            "the coordinator",
            facts(&[
                ("Charter", "charter 5"),
                (
                    "Its order",
                    "the design is delivered first, as a change into the goal's branch; the changes start from it and merge after it",
                ),
            ]),
        ),
        (
            &missing_coordinator,
            Step::CloseOut,
            Opens::Charter(5),
            "an agent · after the landing, brings what the goal leaves up to date",
            "the coordinator",
            facts(&[("Charter", "charter 5"), ("Lands", "as one change into main, by main's rules")]),
        ),
        (
            &goal_conflict,
            Step::Conflict,
            Opens::Queue,
            "code · temper/0/g0 has moved in a way that clashes with the change",
            "the queue",
            facts(&[
                ("When", "the change reaches the front of the queue: temper/0/g0 is merged into it, never by an agent"),
                ("Merged cleanly", "it is checked again and lands"),
                ("A clash", "it leaves the queue and goes to resolve"),
            ]),
        ),
        (
            &review,
            Step::Conflict,
            Opens::Queue,
            "code · main has moved in a way that clashes with the change",
            "the queue",
            facts(&[
                ("When", "the change reaches the front of the queue: main is merged into it, never by an agent"),
                ("Merged cleanly", "it is checked again and lands"),
                ("A clash", "it leaves the queue and goes to resolve"),
            ]),
        ),
        (
            &other,
            Step::Conflict,
            Opens::Queue,
            "code · release has moved in a way that clashes with the change",
            "the queue",
            facts(&[
                ("When", "the change reaches the front of the queue: release is merged into it, never by an agent"),
                ("Merged cleanly", "it is checked again and lands"),
                ("A clash", "it leaves the queue and goes to resolve"),
            ]),
        ),
        (
            &goal,
            Step::Conflict,
            Opens::Queue,
            "code · main has moved in a way that clashes with the goal's branch",
            "the queue",
            facts(&[
                ("When", "the goal's branch reaches the front of the queue: main is merged into it, never by an agent"),
                ("Merged cleanly", "it is checked again and lands"),
                ("A clash", "it leaves the queue and goes to resolve"),
            ]),
        ),
        (
            &approved,
            Step::Conflict,
            Opens::Queue,
            "code · main has moved in a way that clashes with the change",
            "the queue",
            facts(&[
                ("When", "the change reaches the front of the queue: main is merged into it, never by an agent"),
                ("Merged cleanly", "it lands"),
                ("A clash", "it leaves the queue and goes to resolve"),
            ]),
        ),
        (
            &goal,
            Step::CatchUp,
            Opens::Nothing,
            "code · takes main into the goal's branch",
            "the merge job",
            facts(&[
                ("Does", "merges main's tip into a branch made at the goal's tip, landing as a change into it"),
                ("When", "the coordinator's call, as main's landings touch the goal's open work"),
                ("A conflict", "goes to resolve"),
            ]),
        ),
        (
            &goal,
            Step::Merge,
            Opens::Queue,
            "code · lands each change into the goal's branch",
            "the queue",
            facts(&[("Does", "merges exactly the commit its checks saw")]),
        ),
        (
            &review,
            Step::Resolve,
            Opens::Charter(review.producer),
            "an agent · settles a conflict with its branch, then goes back through the checks",
            "the coding agent",
            facts(&[
                ("Model", "Model · high"),
                ("Can", "read the code and the forge; never merge"),
                ("Asked to", "“Review the change.”"),
            ]),
        ),
        (
            &stock_review,
            Step::Review,
            Opens::Gate { number: 1, charter: review.reviewer },
            "an agent · holds a change until it approves",
            "the reviewer",
            facts(&[
                ("Model", "Model · high"),
                ("Can", "read the code and the forge; never merge"),
                (
                    "Asked to",
                    "“Review the change at the named head: read its pull request, the files it changes and its checks, and judge them against what the change was asked to do.”",
                ),
            ]),
        ),
        (
            &review,
            Step::Review,
            Opens::Gate { number: 1, charter: review.reviewer },
            "an agent · holds a change until it approves",
            "the reviewer",
            facts(&[
                ("Model", "Model · high"),
                ("Can", "read the code and the forge; never merge"),
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
            facts(&[
                ("Role", "owner"),
                ("Asks", "1 of them"),
                ("Counts", "code counts the sign-offs at this head; a clean update keeps them"),
            ]),
        ),
        (&review, Step::Branch, Opens::Nothing, "a branch · where the change lands", "main", facts(&[])),
        (
            &deployment,
            Step::Check,
            Opens::Rule { kind: 1, scope: 0, number: 1 },
            "code · holds a change until CI passes",
            "CI",
            facts(&[("Checks", "every check the repository's CI workflows report, at the change's head")]),
        ),
        (
            &deployment_workflows,
            Step::Check,
            Opens::Rule { kind: 1, scope: 0, number: 1 },
            "code · holds a change until CI passes",
            "CI",
            facts(&[
                ("Workflows", "ci.yml"),
                ("Checks", "every check the repository's CI workflows report, at the change's head"),
            ]),
        ),
        (
            &three_workflows,
            Step::Check,
            Opens::Rule { kind: 1, scope: 0, number: 1 },
            "code · holds a change until CI passes",
            "CI",
            facts(&[
                ("Workflows", "build.yml, lint.yml and test.yml"),
                ("Checks", "the CI workflows' checks build, lint and test, at the change's head"),
            ]),
        ),
        (
            &one_context,
            Step::Check,
            Opens::Rule { kind: 1, scope: 0, number: 1 },
            "code · holds a change until CI passes",
            "CI",
            facts(&[("Checks", "the CI workflows' checks ci / test, at the change's head")]),
        ),
        (
            &three_contexts,
            Step::Check,
            Opens::Rule { kind: 1, scope: 0, number: 1 },
            "code · holds a change until CI passes",
            "CI",
            facts(&[("Checks", "the CI workflows' checks build, lint and test, at the change's head")]),
        ),
        (
            &advisory,
            Step::Review,
            Opens::Gate { number: 1, charter: review.reviewer },
            "an agent · advises; holds nothing",
            "the reviewer",
            facts(&[
                ("Model", "Model · high"),
                ("Can", "read the code and the forge; never merge"),
                ("Asked to", "“Review the change.”"),
            ]),
        ),
        (
            &duplicate,
            Step::Review,
            Opens::Gate { number: 1, charter: review.reviewer },
            "an agent · holds a change until it approves",
            "the reviewer",
            facts(&[
                ("Model", "Model · high"),
                ("Can", "read the code and the forge; never merge"),
                ("Asked to", "“Review the change.”"),
            ]),
        ),
        (
            &missing,
            Step::Review,
            Opens::Gate { number: 1, charter: review.reviewer },
            "an agent · holds a change until it approves",
            "the reviewer",
            facts(&[("Charter", "charter 2")]),
        ),
        (
            &no_account,
            Step::Review,
            Opens::Gate { number: 1, charter: review.reviewer },
            "an agent · holds a change until it approves",
            "the reviewer",
            facts(&[
                ("Model", "no model · no account runs it now"),
                ("Can", "read the code and the forge; never merge"),
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
        (
            &described,
            Step::Verify,
            Opens::Tier(b"slow".as_slice().into()),
            "code · checks what landed",
            "slow",
            facts(&[("Runs", "the tier slow after each landing"), ("Checks", "the slow suites")]),
        ),
        (
            &goal_described,
            Step::Check,
            Opens::Tier(b"goal".as_slice().into()),
            "code · checks the goal's branch",
            "goal",
            facts(&[("Runs", "the tier goal on the goal's branch"), ("Checks", "the gate without its slowest suites")]),
        ),
    ];
    for (inputs, step, opens, who, name, facts) in rows {
        let path = derive(inputs);
        bounds(&path);
        let stop = find(&path, step);
        assert_eq!(stop.opens, opens);
        assert_eq!(stop.card, Card { who: who.as_bytes().into(), name: name.as_bytes().into(), facts });
    }
    let goal_path = derive(&goal);
    let landing_merge = goal_path.stops.iter().rev().find(|stop| stop.step == Step::Merge).unwrap();
    assert_eq!(landing_merge.card.who.as_ref(), "code · lands the goal on main".as_bytes());
    for length in [0, WORDS, VALUE, VALUE + 7, ASKED, ASKED + 7] {
        let mut inputs = inputs(0, true, true, Way::Queue, false);
        inputs.branch = "é".repeat(length).into_bytes().into();
        inputs.start = Start::Goal { charter: 5, design: true, close_out: true };
        inputs.merge_job = true;
        inputs.repository.as_mut().unwrap().contexts = [(); 3].map(|()| inputs.branch.clone()).into();
        inputs.repository.as_mut().unwrap().workflows = [(); 3].map(|()| inputs.branch.clone()).into();
        for role in &mut inputs.roles {
            role.name = inputs.branch.clone();
        }
        for agent in &mut inputs.agents {
            agent.model = Some(inputs.branch.clone());
            agent.effort = Some(inputs.branch.clone());
            agent.can = inputs.branch.clone();
            agent.asked = inputs.branch.clone();
        }
        let path = derive(&inputs);
        bounds(&path);
        assert_eq!(find(&path, Step::Design).card.facts.len(), FACTS);
        assert_eq!(find(&path, Step::CloseOut).card.facts.len(), FACTS);
        let rule = &mut inputs.rules.first_mut().unwrap().rule;
        rule.gates =
            (0..40).map(|number| RuleGate { number, blocking: true, freshness: JudgeFreshness::Exact }).collect();
        let path = derive(&inputs);
        bounds(&path);
        for stop in &path.stops {
            for fact in &stop.card.facts {
                if fact.label.as_ref() == b"Asked to" {
                    assert!(fact.value.len() <= ASKED);
                    if inputs.branch.len() > ASKED - "“”".len() {
                        assert!(
                            fact.value.ends_with("…”".as_bytes()),
                            "overlong instructions carry an ellipsis inside quotes"
                        );
                    }
                } else if fact.label.as_ref() == b"Workflows" && inputs.branch.len() > VALUE {
                    assert!(fact.value.ends_with("…".as_bytes()), "overlong workflows carry an ellipsis");
                } else if fact.label.as_ref() == b"Checks" && inputs.branch.len() > VALUE {
                    assert!(fact.value.ends_with("…".as_bytes()), "overlong required checks carry an ellipsis");
                }
            }
        }
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
    assert_eq!(model.value.as_ref(), ("x".to_owned() + &"€".repeat(41) + "…").as_bytes());
    let mut invalid = inputs(1, true, true, Way::Queue, false);
    invalid.branch = [0x80; 400].into();
    invalid.agents.get_mut(1).unwrap().asked = [0x80; 400].into();
    invalid.repository.as_mut().unwrap().contexts =
        Box::new([[0x80].as_slice().into(), b"ci / test".as_slice().into()]);
    invalid.repository.as_mut().unwrap().workflows = Box::new([[0x80].as_slice().into(), b"ci.yml".as_slice().into()]);
    let path = derive(&invalid);
    assert_eq!(
        find(&path, Step::Check).card.facts,
        facts(&[("Workflows", "ci.yml"), ("Checks", "the CI workflows' checks ci / test, at the change's head")]),
        "invalid UTF-8 workflows and contexts are skipped whole"
    );
    bounds(&path);
}

fn path_rule() -> Inputs {
    let mut input = inputs(0, false, false, Way::Queue, false);
    let rule = &mut input.rules.first_mut().unwrap().rule;
    rule.ci = false;
    rule.up_to_date = false;
    rule.sets = Box::new([1]);
    input.sets = Box::new([temper_protocol_reaches::PathSet {
        number: 1,
        name: b"protocol core".as_slice().into(),
        patterns: [b"protocol/**".as_slice(), b"src/wire.rs"].map(Box::from).into(),
    }]);
    input
}

#[test]
fn a_path_only_sign_off_names_its_paths_and_counts_at_this_head() {
    for (name, over) in [("protocol core", "if protocol core"), ("vsr's protocol core", "by path")] {
        for (freshness, counts) in [
            (JudgeFreshness::Exact, "code counts the sign-offs at this head"),
            (JudgeFreshness::Clean, "code counts the sign-offs at this head; a clean update keeps them"),
        ] {
            let mut input = path_rule();
            input.sets.first_mut().unwrap().name = name.as_bytes().into();
            input.rules.first_mut().unwrap().rule.approvals.first_mut().unwrap().freshness = freshness;
            let path = derive(&input);
            bounds(&path);
            let stops: Box<[_]> = path.stops.iter().filter(|stop| stop.step == Step::Approve).collect();
            assert_eq!(stops.len(), 1);
            let stop = stops.first().unwrap();
            assert_eq!(stop.words.as_ref(), b"you");
            assert_eq!(stop.over.as_ref(), over.as_bytes());
            assert_eq!(stop.opens, Opens::Rule { kind: 2, scope: PROJECT, number: 1 });
            let facts: Box<[_]> =
                stop.card.facts.iter().map(|fact| (fact.label.as_ref(), fact.value.as_ref())).collect();
            assert_eq!(
                facts.as_ref(),
                [
                    (b"Only for".as_slice(), "protocol/** · src/wire.rs".as_bytes()),
                    (b"Role", b"owner"),
                    (b"Asks", b"1 of them"),
                    (b"Counts", counts.as_bytes()),
                ]
            );
        }
    }
    let mut input = path_rule();
    input.rules.first_mut().unwrap().rule.sets = Box::new([2, 1]);
    let path = derive(&input);
    let stop = path.stops.iter().find(|stop| stop.step == Step::Approve).unwrap();
    assert_eq!(stop.words.as_ref(), b"you approve");
    assert!(stop.over.is_empty(), "a missing set covers every change");
    assert_eq!(stop.card.facts.len(), 3);
    assert_eq!(rules_by_path(&input), PathRules::empty());
    input.sets = [
        input.sets.first().unwrap().clone(),
        temper_protocol_reaches::PathSet {
            number: 2,
            name: b"other".as_slice().into(),
            patterns: Box::new([b"other/**".as_slice().into()]),
        },
    ]
    .into();
    let path = derive(&input);
    let stop = path.stops.iter().find(|stop| stop.step == Step::Approve).unwrap();
    assert_eq!(stop.over.as_ref(), b"by path");
    assert_eq!(stop.card.facts.first().unwrap().value.as_ref(), "protocol/** · src/wire.rs · other/**".as_bytes());
    let long: Box<[u8]> = [b'a'; 80].as_slice().into();
    input.sets.first_mut().unwrap().patterns = [long.clone(), long.clone(), long.clone()].into();
    let path = derive(&input);
    bounds(&path);
    let stop = path.stops.iter().find(|stop| stop.step == Step::Approve).unwrap();
    let only_for = skein_lib::bytes::concat(&[long.as_ref(), " · …".as_bytes()]);
    assert_eq!(stop.card.facts.first().unwrap().value, only_for);
    let rules = rules_by_path(&input);
    invariants::rules_bounds(&rules);
    let paths = skein_lib::bytes::concat(&[
        long.as_ref(),
        " · ".as_bytes(),
        long.as_ref(),
        " · ".as_bytes(),
        long.as_ref(),
        " · …".as_bytes(),
    ]);
    assert_eq!(rules.rows.first().unwrap().paths, paths);
}

#[test]
fn one_role_signs_once_and_path_rules_can_ask_more() {
    let mut input = path_rule();
    input.sets.first_mut().unwrap().name = b"payments".as_slice().into();
    input.roles.first_mut().unwrap().holders = 3;
    let mut every = input.rules.first().unwrap().clone();
    every.rule.sets = Box::default();
    let mut path = every.clone();
    path.number = 2;
    path.rule.sets = Box::new([1]);
    path.rule.approvals.first_mut().unwrap().people = 2;
    input.rules = Box::new([every, path]);
    let path = derive(&input);
    let stops: Box<[_]> = path.stops.iter().filter(|stop| stop.step == Step::Approve).collect();
    assert_eq!(stops.len(), 1);
    let stop = stops.first().unwrap();
    assert_eq!(stop.words.as_ref(), b"one of you");
    assert!(stop.over.is_empty());
    assert_eq!(stop.opens, Opens::Rule { kind: 2, scope: PROJECT, number: 1 });
    let facts: Box<[_]> = stop.card.facts.iter().map(|fact| fact.value.as_ref()).collect();
    assert_eq!(
        facts.as_ref(),
        [
            b"owner: the three of you".as_slice(),
            b"1 of you; payments: 2",
            b"code counts the sign-offs at this head; a clean update keeps them"
        ]
    );
    input.roles.first_mut().unwrap().yours = false;
    input.rules = Box::new([input.rules.get(1).unwrap().clone()]);
    let path = derive(&input);
    let stop = path.stops.iter().find(|stop| stop.step == Step::Approve).unwrap();
    assert_eq!(stop.words.as_ref(), "an owner approves ×2".as_bytes());
}

#[test]
fn only_path_rules_give_checks_and_gates_their_over_words() {
    let mut input = path_rule();
    input.repository.as_mut().unwrap().ci = true;
    let rule = &mut input.rules.first_mut().unwrap().rule;
    rule.ci = true;
    rule.approvals = Box::default();
    rule.gates = Box::new([RuleGate { number: 1, blocking: true, freshness: JudgeFreshness::Clean }]);
    let path = derive(&input);
    for step in [Step::Check, Step::Review] {
        let stops: Box<[_]> = path.stops.iter().filter(|stop| stop.step == step).collect();
        assert_eq!(stops.len(), if step == Step::Check { 2 } else { 1 });
        for stop in stops {
            assert_eq!(stop.over.as_ref(), b"if protocol core");
        }
    }
    let repair = path.stops.iter().find(|stop| stop.step == Step::Repair).unwrap();
    assert_eq!(repair.over.as_ref(), b"tests, review");
    let mut every = input.rules.first().unwrap().clone();
    every.number = 2;
    every.rule.sets = Box::default();
    input.rules = Box::new([input.rules.first().unwrap().clone(), every]);
    let path = derive(&input);
    assert!(
        path.stops
            .iter()
            .filter(|stop| [Step::Check, Step::Review].contains(&stop.step))
            .all(|stop| stop.over.is_empty())
    );
}

#[test]
fn path_rows_combine_every_change_rules_and_explain_freshness() {
    let mut input = path_rule();
    let rules = rules_by_path(&input);
    invariants::rules_bounds(&rules);
    assert_eq!(
        rules.rows.as_ref(),
        [PathRule {
            paths: "protocol/** · src/wire.rs".as_bytes().into(),
            review: Box::default(),
            sign_off: b"you".as_slice().into(),
            lands: b"once you sign off".as_slice().into(),
        }]
    );
    assert_eq!(
        rules.note.as_ref(),
        b"A sign-off is for one head. A change pushed after it needs signing again; taking main cleanly keeps it."
    );
    input.roles.first_mut().unwrap().holders = 3;
    input.sets.first_mut().unwrap().name = b"payments".as_slice().into();
    let mut every = input.rules.first().unwrap().clone();
    every.number = 2;
    every.rule.sets = Box::default();
    every.rule.gates = Box::new([RuleGate { number: 1, blocking: true, freshness: JudgeFreshness::Clean }]);
    input.rules.first_mut().unwrap().rule.approvals.first_mut().unwrap().people = 2;
    input.rules = Box::new([input.rules.first().unwrap().clone(), every]);
    let rules = rules_by_path(&input);
    invariants::rules_bounds(&rules);
    let rows: Box<[_]> = rules
        .rows
        .iter()
        .map(|row| (row.paths.as_ref(), row.review.as_ref(), row.sign_off.as_ref(), row.lands.as_ref()))
        .collect();
    assert_eq!(
        rows.as_ref(),
        [
            (
                "protocol/** · src/wire.rs".as_bytes(),
                b"an agent".as_slice(),
                b"two of you".as_slice(),
                b"once two have signed this head".as_slice()
            ),
            (b"everything else", b"an agent", b"one of you", b"once signed"),
        ]
    );
    input.rules.first_mut().unwrap().rule.approvals.first_mut().unwrap().freshness = JudgeFreshness::Exact;
    assert_eq!(rules_by_path(&input).note.as_ref(), b"A sign-off is for one head.");
    input.rules.get_mut(1).unwrap().rule.approvals.first_mut().unwrap().freshness = JudgeFreshness::Exact;
    assert_eq!(
        rules_by_path(&input).note.as_ref(),
        b"A sign-off is for one head. A change pushed after it, or taking main, needs signing again."
    );
    input.rules.first_mut().unwrap().rule.sets = Box::default();
    assert_eq!(rules_by_path(&input), PathRules::empty());
}

#[test]
fn shared_owners_keep_the_current_stock_way() {
    let mut own = inputs(0, false, false, Way::Queue, false);
    own.roles.first_mut().unwrap().holders = 3;
    let own = derive(&own);
    let paths: Box<[_]> = (0..3)
        .map(|way| {
            let mut input = inputs(way, false, false, Way::Queue, false);
            input.roles.first_mut().unwrap().holders = 3;
            derive(&input)
        })
        .collect();
    assert_eq!(current(&paths, &own), Some(0));
    assert_eq!(own.stops.iter().find(|stop| stop.step == Step::Approve).unwrap().words.as_ref(), b"one of you");
    assert_eq!(
        paths.first().unwrap().stops.iter().find(|stop| stop.step == Step::Approve).unwrap().words.as_ref(),
        b"one of you"
    );
}
