//! Table tests of path comparisons and settings overlays.
mod derivation;
#[path = "tests/invariants.rs"]
mod invariants;
use super::*;
use alloc::boxed::Box;
use temper_engine_domain_forge::settings::{
    Held, LandingRule, ProjectWays, RuleApproval, RuleGate, TemplateGoal, Value,
};
use temper_engine_domain_forge::{GateLast, GatePattern, JudgeFreshness};
use temper_protocol_reaches::{Classes, Way};

const PROJECT: u32 = 9;
const OTHER: u32 = 10;

fn rule(last: GateLast, ci: bool, up_to_date: bool) -> LandingRule {
    LandingRule {
        pattern: GatePattern { segments: Box::new([b"branch".as_slice().into()]), last },
        enforce: true,
        ci,
        up_to_date,
        gates: Box::default(),
        approvals: Box::default(),
    }
}

fn approval_rule() -> LandingRule {
    let mut rule = rule(GateLast::Open(Box::default()), true, true);
    rule.approvals = Box::new([RuleApproval { role: 0, people: 1, freshness: JudgeFreshness::Clean }]);
    rule
}

fn review_rule() -> LandingRule {
    let mut rule = rule(GateLast::Open(Box::default()), true, true);
    rule.gates = Box::new([RuleGate { number: 1, blocking: true, freshness: JudgeFreshness::Clean }]);
    rule
}

fn stock_ways() -> [StockWay; 3] {
    let approve = approval_rule();
    let review = review_rule();
    [
        StockWay {
            name: b"You approve".as_slice().into(),
            headline: b"You approve every change".as_slice().into(),
            line: b"An agent makes each change; nothing reaches main until you approve it.".as_slice().into(),
            rules: Box::new([(1, approve)]),
            ways: None,
            classes: Some(Classes::stock(Way::YouApprove)),
            delegation: false,
            triager: None,
            roles: Box::default(),
        },
        StockWay {
            name: b"Agents review".as_slice().into(),
            headline: b"Agents review every change, and code lands it".as_slice().into(),
            line: b"An agent makes each change, another reviews it, and code lands it.".as_slice().into(),
            rules: Box::new([(1, review)]),
            ways: None,
            classes: Some(Classes::stock(Way::AgentsReview)),
            delegation: false,
            triager: None,
            roles: Box::default(),
        },
        StockWay {
            name: b"Draft first".as_slice().into(),
            headline: b"Draft first, then agents build it on its own branch".as_slice().into(),
            line: b"You and an agent write a design draft; agents build it on its own branch, and it lands whole."
                .as_slice()
                .into(),
            rules: Box::new([
                (1, rule(GateLast::Exact(b"main".as_slice().into()), true, true)),
                (2, rule(GateLast::Open(b"temper".as_slice().into()), false, false)),
            ]),
            ways: Some(Laid { drafts: true, goals: true, way: 4 }),
            classes: Some(Classes::stock(Way::DraftFirst)),
            delegation: true,
            triager: Some(temper_engine_domain_forge::settings::AreaOwner::Charter(14)),
            roles: Box::default(),
        },
    ]
}

fn project_ways() -> ProjectWays {
    ProjectWays { drafts: false, goals: false, changes: true, way: 8 }
}

#[test]
#[expect(clippy::too_many_lines, reason = "the way rows pin the owned kinds and the settings left alone")]
fn items_set_the_ways_rules_and_preserve_other_visibility() {
    let [approve, review, draft] = stock_ways();
    let mut no_classes = approve.clone();
    no_classes.classes = None;
    let own = project_ways();
    let approval = approval_rule();
    let reviewed = review_rule();
    for (way, ways, expected) in [
        (
            &approve,
            None,
            Box::<[(u32, u32, Edit)]>::from([
                (2, 1, Edit::Set(Value::Rule(approval.clone()))),
                (2, 2, Edit::Remove),
                (2, 3, Edit::Remove),
                (2, 4, Edit::Remove),
                (8, 1, Edit::Set(Value::Classes(Classes::stock(Way::YouApprove)))),
            ]),
        ),
        (
            &approve,
            Some(&own),
            Box::from([
                (2, 1, Edit::Set(Value::Rule(approval))),
                (2, 2, Edit::Remove),
                (2, 3, Edit::Remove),
                (2, 4, Edit::Remove),
                (8, 1, Edit::Set(Value::Classes(Classes::stock(Way::YouApprove)))),
            ]),
        ),
        (
            &review,
            None,
            Box::from([
                (2, 1, Edit::Set(Value::Rule(reviewed.clone()))),
                (2, 2, Edit::Remove),
                (2, 3, Edit::Remove),
                (2, 4, Edit::Remove),
                (8, 1, Edit::Set(Value::Classes(Classes::stock(Way::AgentsReview)))),
            ]),
        ),
        (
            &review,
            Some(&own),
            Box::from([
                (2, 1, Edit::Set(Value::Rule(reviewed))),
                (2, 2, Edit::Remove),
                (2, 3, Edit::Remove),
                (2, 4, Edit::Remove),
                (8, 1, Edit::Set(Value::Classes(Classes::stock(Way::AgentsReview)))),
            ]),
        ),
        (
            &draft,
            None,
            Box::from([
                (2, 1, Edit::Set(Value::Rule(rule(GateLast::Exact(b"main".as_slice().into()), true, true)))),
                (2, 2, Edit::Set(Value::Rule(rule(GateLast::Open(b"temper".as_slice().into()), false, false)))),
                (2, 3, Edit::Remove),
                (2, 4, Edit::Remove),
                (7, 1, Edit::Set(Value::Ways(ProjectWays { drafts: true, goals: true, changes: false, way: 4 }))),
                (8, 1, Edit::Set(Value::Classes(Classes::stock(Way::DraftFirst)))),
                (
                    13,
                    1,
                    Edit::Set(Value::ProjectTriager(temper_engine_domain_forge::settings::ProjectTriager {
                        owner: temper_engine_domain_forge::settings::AreaOwner::Charter(14),
                    })),
                ),
            ]),
        ),
        (
            &draft,
            Some(&own),
            Box::from([
                (2, 1, Edit::Set(Value::Rule(rule(GateLast::Exact(b"main".as_slice().into()), true, true)))),
                (2, 2, Edit::Set(Value::Rule(rule(GateLast::Open(b"temper".as_slice().into()), false, false)))),
                (2, 3, Edit::Remove),
                (2, 4, Edit::Remove),
                (7, 1, Edit::Set(Value::Ways(ProjectWays { drafts: true, goals: true, changes: true, way: 4 }))),
                (8, 1, Edit::Set(Value::Classes(Classes::stock(Way::DraftFirst)))),
                (
                    13,
                    1,
                    Edit::Set(Value::ProjectTriager(temper_engine_domain_forge::settings::ProjectTriager {
                        owner: temper_engine_domain_forge::settings::AreaOwner::Charter(14),
                    })),
                ),
            ]),
        ),
        (
            &no_classes,
            None,
            Box::from([
                (2, 1, Edit::Set(Value::Rule(approval_rule()))),
                (2, 2, Edit::Remove),
                (2, 3, Edit::Remove),
                (2, 4, Edit::Remove),
            ]),
        ),
    ] {
        let mut expected_items = skein_lib::List::uncapped();
        for (kind, number, edit) in expected {
            expected_items.push(Item { kind, scope: PROJECT, number, edit }).expect("expected items");
        }
        let got = items(way, PROJECT, ways);
        for item in &got {
            assert_ne!(item.kind, 12, "a way never touches project areas");
        }
        assert_eq!(got, expected_items.into_boxed());
    }
}

fn held(scope: u32, number: u32, ci: bool) -> Held {
    Held { scope, number, rule: rule(GateLast::Open(Box::default()), ci, false) }
}

#[test]
fn laid_keeps_deployment_order_and_overlays_project_numbers() {
    for (rules, stock, edits, expected) in [
        (
            Box::from([held(PROJECT, 2, false)]),
            Box::from([]),
            Box::from([Item {
                kind: 2,
                scope: PROJECT,
                number: 1,
                edit: Edit::Set(Value::Rule(rule(GateLast::Open(Box::default()), true, false))),
            }]),
            Box::from([held(PROJECT, 1, true), held(PROJECT, 2, false)]),
        ),
        (
            Box::from([held(PROJECT, 1, false)]),
            Box::from([held(0, 1, true)]),
            Box::from([Item { kind: 2, scope: PROJECT, number: 1, edit: Edit::Remove }]),
            Box::from([held(PROJECT, 1, true)]),
        ),
        (
            Box::from([held(PROJECT, 1, false)]),
            Box::from([]),
            Box::from([Item { kind: 2, scope: PROJECT, number: 1, edit: Edit::Remove }]),
            Box::from([]),
        ),
        (
            Box::from([held(0, 2, true), held(0, 1, false), held(PROJECT, 2, false), held(OTHER, 1, true)]),
            Box::from([]),
            Box::from([Item { kind: 2, scope: OTHER, number: 2, edit: Edit::Remove }]),
            Box::from([held(0, 2, true), held(0, 1, false), held(PROJECT, 2, false)]),
        ),
        (
            Box::from([held(PROJECT, 1, false)]),
            Box::from([]),
            Box::from([Item {
                kind: 2,
                scope: PROJECT,
                number: 1,
                edit: Edit::Set(Value::Rule(rule(GateLast::Open(Box::default()), true, false))),
            }]),
            Box::from([held(PROJECT, 1, true)]),
        ),
    ] {
        assert_eq!(laid(&rules, &stock, PROJECT, &edits), expected);
    }
}

#[test]
fn laid_ways_sets_removes_and_ignores_other_settings() {
    let own = project_ways();
    let stock = ProjectWays { drafts: true, goals: false, changes: false, way: 0 };
    for (ways, edits, expected) in [
        (None, Box::from([]), stock),
        (Some(&own), Box::from([]), own),
        (Some(&own), Box::from([Item { kind: 7, scope: PROJECT, number: 1, edit: Edit::Remove }]), stock),
        (None, Box::from([Item { kind: 7, scope: PROJECT, number: 1, edit: Edit::Set(Value::Ways(own)) }]), own),
        (
            Some(&own),
            Box::from([
                Item { kind: 7, scope: OTHER, number: 1, edit: Edit::Remove },
                Item { kind: 7, scope: PROJECT, number: 2, edit: Edit::Remove },
                Item { kind: 2, scope: PROJECT, number: 1, edit: Edit::Remove },
            ]),
            own,
        ),
    ] {
        assert_eq!(laid_ways(ways, PROJECT, &edits), expected);
    }
    let both = TemplateGoal { design: true, close_out: true };
    let neither = TemplateGoal { design: false, close_out: false };
    for (name, held, edits, expected) in [
        ("none", None, Box::from([]), neither),
        ("held", Some(both), Box::from([]), both),
        ("set", None, Box::from([Item { kind: 11, scope: 0, number: 4, edit: Edit::Set(Value::Goal(both)) }]), both),
        ("remove", Some(both), Box::from([Item { kind: 11, scope: 0, number: 4, edit: Edit::Remove }]), neither),
        ("another template", Some(both), Box::from([Item { kind: 11, scope: 0, number: 2, edit: Edit::Remove }]), both),
        ("another kind", Some(both), Box::from([Item { kind: 7, scope: 0, number: 4, edit: Edit::Remove }]), both),
        (
            "another scope",
            Some(both),
            Box::from([Item { kind: 11, scope: PROJECT, number: 4, edit: Edit::Remove }]),
            both,
        ),
        (
            "another value",
            Some(both),
            Box::from([Item { kind: 11, scope: 0, number: 4, edit: Edit::Set(Value::Ways(own)) }]),
            both,
        ),
    ] {
        assert_eq!(laid_goal(held, 4, &edits), expected, "{name}");
    }
}

fn stop() -> Stop {
    Stop {
        actor: Actor::Person,
        step: Step::Ask,
        words: b"you ask".as_slice().into(),
        opens: Opens::Nothing,
        needed: false,
        over: Box::default(),
        card: Card { who: b"person".as_slice().into(), name: b"you".as_slice().into(), facts: Box::default() },
    }
}

#[test]
fn same_compares_drawn_stops_and_ignores_cards() {
    let own = stop();
    let mut card = own.clone();
    card.card.name = b"another person".as_slice().into();
    let mut actor = own.clone();
    actor.actor = Actor::Agent;
    let mut step = own.clone();
    step.step = Step::Draft;
    let mut words = own.clone();
    words.words = b"draft".as_slice().into();
    let mut opens = own.clone();
    opens.opens = Opens::Charter(1);
    let mut needed = own.clone();
    needed.needed = true;
    let mut over = own.clone();
    over.over = b"tests".as_slice().into();
    for (other, expected) in [
        (own.clone(), true),
        (card, true),
        (actor, false),
        (step, false),
        (words, false),
        (opens, false),
        (needed, false),
        (over, false),
    ] {
        assert_eq!(own.same(&other), expected);
    }
}

fn path(stops: &[Stop]) -> Path {
    Path { repository: None, branch: b"main".as_slice().into(), stops: stops.into() }
}

#[test]
fn paths_compare_stops_in_order_and_number() {
    let ask = stop();
    let mut draft = ask.clone();
    draft.step = Step::Draft;
    let own = path(&[ask.clone(), draft.clone()]);
    let mut renamed = own.clone();
    renamed.repository = Some(b"other/repo".as_slice().into());
    renamed.branch = b"other".as_slice().into();
    for (other, expected) in
        [(renamed, true), (path(&[draft.clone(), ask.clone()]), false), (path(&[ask]), false), (path(&[]), false)]
    {
        assert_eq!(own.same(&other), expected);
    }
}

#[test]
fn current_returns_the_first_equal_path_or_none() {
    let own = path(&[stop()]);
    for (ways, expected) in [
        (Box::from([]), None),
        (Box::from([path(&[])]), None),
        (Box::from([path(&[]), own.clone(), own.clone()]), Some(1)),
        (Box::from([own.clone()]), Some(0)),
    ] {
        assert_eq!(current(&ways, &own), expected);
    }
}

#[test]
fn sentence_stops_trims_and_cuts_at_a_word_end() {
    let reviewer = "Review the change at the named head: read its pull request, the files it changes and its checks, and judge them against what the change was asked to do.";
    let long = "x".repeat(VALUE.checked_sub(1).expect("value bound")) + "érest";
    let word = "x".repeat(200);
    for (input, bound, expected) in [
        (b"  First. Next. ".as_slice(), VALUE, b"First.".as_slice()),
        (b" First.\nNext", VALUE, b"First."),
        (b"  No stop\n ", VALUE, b"No stop"),
        (b"v1.2 stays", VALUE, b"v1.2 stays"),
        (b"", VALUE, b""),
        (reviewer.as_bytes(), ASKED.checked_sub("“”".len()).expect("quote room"), reviewer.as_bytes()),
        (reviewer.as_bytes(), 64, "Review the change at the named head: read its pull request,…".as_bytes()),
        (word.as_bytes(), ASKED, ("x".repeat(181) + "…").as_bytes()),
        (long.as_bytes(), VALUE, ("x".repeat(125) + "…").as_bytes()),
        (b"long", 2, b""),
    ] {
        assert_eq!(first_sentence(input, bound).as_ref(), expected);
    }
}

#[test]
fn role_who_keeps_when_and_cuts_what_it_does_to_the_room() {
    let when = "When you look at Home after landings";
    let exact = "x".repeat(90);
    let over = "x".repeat(91);
    let multibyte = "x".repeat(86) + "é" + &"x".repeat(32);
    let words = "word ".repeat(17) + "é" + &"x".repeat(33);
    assert_eq!(multibyte.len(), 120);
    assert_eq!(words.len(), 120);
    for (when, does, expected) in [
        (
            "For each chat",
            "talks with you and asks for the changes",
            b"For each chat: talks with you and asks for the changes".as_slice().into(),
        ),
        ("For each change", "", b"For each change".as_slice().into()),
        (when, exact.as_str(), skein_lib::bytes::concat(&[when.as_bytes(), b": ", exact.as_bytes()])),
        (when, over.as_str(), skein_lib::bytes::concat(&[when.as_bytes(), b": ", ("x".repeat(87) + "…").as_bytes()])),
        (
            when,
            multibyte.as_str(),
            skein_lib::bytes::concat(&[when.as_bytes(), b": ", ("x".repeat(86) + "…").as_bytes()]),
        ),
        (
            when,
            words.as_str(),
            skein_lib::bytes::concat(&[when.as_bytes(), b": ", ("word ".repeat(16) + "word…").as_bytes()]),
        ),
    ] {
        let who = role_who(when.as_bytes(), does.as_bytes());
        assert_eq!(who.as_ref(), expected.as_ref());
        assert!(who.len() <= WHO);
        assert!(skein_lib::bytes::is_utf8(&who));
    }
}

#[test]
fn can_names_families_in_order_and_always_forbids_merging() {
    for (inspect, modify, shell, agents, settings, expected) in [
        (false, false, false, false, false, b"read the facts in its task; never merge".as_slice()),
        (
            false,
            false,
            false,
            false,
            true,
            b"read temper's records and the forge; propose settings; never change anything",
        ),
        (true, false, false, false, false, b"read the code and the forge; never merge"),
        (false, true, false, false, false, b"change and push its branch; never merge"),
        (false, false, true, false, false, b"run commands; never merge"),
        (false, false, false, true, false, b"start sub-agents; never merge"),
        (
            true,
            true,
            true,
            true,
            false,
            b"read the code and the forge; change and push its branch; run commands; start sub-agents; never merge",
        ),
    ] {
        assert_eq!(can(inspect, modify, shell, agents, settings), expected);
    }
}

#[test]
fn subjects_keep_project_bits_apart_from_other_subjects() {
    const SUBJECT: u64 = subject(0);
    assert_eq!(SUBJECT, 0x4000_0040_0000_0000);
    for (project, expected) in [(0, SUBJECT), (1, 0x4000_0040_0000_0001), (u32::MAX, 0x4000_0040_ffff_ffff)] {
        assert_eq!(subject(project), expected);
    }
}

#[test]
fn derive_keeps_the_branch_and_repository() {
    for (repository, expected) in [
        (None, None),
        (
            Some(Repository {
                host: b"forge".as_slice().into(),
                owner: b"owner".as_slice().into(),
                name: b"name".as_slice().into(),
                ci: false,
                checks: Box::default(),
                contexts: Box::default(),
                workflows: Box::default(),
                gates: Box::default(),
                tiers: Box::default(),
            }),
            Some(Box::from(b"owner/name".as_slice())),
        ),
    ] {
        let inputs = Inputs {
            branch: b"topic".as_slice().into(),
            repository,
            rules: Box::default(),
            start: Start::Chat,
            producer: 1,
            reviewer: 2,
            by: temper_engine_domain_forge_change::Way::Queue,
            merge_job: false,
            roles: Box::default(),
            agents: Box::default(),
        };
        let path = derive(&inputs);
        assert_eq!(path.branch.as_ref(), b"topic");
        assert_eq!(path.repository, expected);
        assert_eq!(path.stops.last().expect("landing bar").actor, Actor::Bar);
    }
}
