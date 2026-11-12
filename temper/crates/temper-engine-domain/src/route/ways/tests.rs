//! Gather the core and forge facts before deriving any words (temper/docs/design/domain/forge/change/branch-path.md).
use super::*;
use crate::tests::resume_fixture;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Env, Queue};

const PROJECT: u32 = 1;
const READER: u64 = 1;
const COORDINATOR: u32 = 1;

fn fixture() -> (Domain, Env<crate::Limits>) {
    roster_fixture(false)
}

fn roster_fixture(service: bool) -> (Domain, Env<crate::Limits>) {
    let mut config = resume_fixture::config(7);
    let model = config.core.settings.models.first_mut().unwrap();
    model.efforts = Box::new([b"low".as_slice().into(), b"high".as_slice().into()]);
    model.effort = Some(b"low".as_slice().into());
    let charter = config.core.defaults.charters.first_mut().unwrap();
    charter.policy.model.efforts = model.efforts.clone();
    charter.policy.model.effort = Some(b"high".as_slice().into());
    let policy = &mut config.core.seed_projects.first_mut().unwrap().policy.policy;
    let mut member = policy.roles.first().unwrap().clone();
    member.number = people::Role::Member.number();
    policy.roles = Box::new([member, policy.roles.first().unwrap().clone()]);
    config.core.defaults.templates = Box::new([goal_template()]);
    let mut later = config.core.defaults.templates[0].clone();
    later.number = 6;
    later.executor = TemplateExecutor::Chosen;
    config.core.defaults.templates = Box::new([later, config.core.defaults.templates[0].clone()]);
    let mut records = List::uncapped();
    records
        .push(crate::Record::Core(core::Record::People(people::Stored::Person {
            number: READER,
            identity: people::Identity { key: forge::user_key(1, 7), login: Box::new([]), name: Box::new([]) },
        })))
        .unwrap();
    let mut holdings = List::uncapped();
    holdings
        .push(people::Holding {
            person: READER,
            role: if service { people::Role::Owner } else { people::Role::Member },
        })
        .unwrap();
    if service {
        records
            .push(crate::Record::Core(core::Record::People(people::Stored::Person {
                number: 2,
                identity: people::Identity {
                    key: people::IdentityKey { provider: 1, subject: b"service".as_slice().into() },
                    login: Box::new([]),
                    name: Box::new([]),
                },
            })))
            .unwrap();
        holdings.push(people::Holding { person: 2, role: people::Role::Owner }).unwrap();
    }
    records
        .push(crate::Record::Core(core::Record::People(people::Stored::Roles {
            project: PROJECT,
            holdings: holdings.into_boxed(),
        })))
        .unwrap();
    let mut limits = resume_fixture::limits();
    limits.documents.roles = 2;
    limits.authority.roles = 2; // Owner and member.
    resume_fixture::root_with_records(config, &limits, records.as_slice())
}

fn goal_template() -> core::Template {
    core::Template {
        number: 4,
        name: b"goal".as_slice().into(),
        executor: TemplateExecutor::Charter(COORDINATOR),
        contract: jig_domain_tasks::Contract::Report { words: 32 },
        shallower: 0,
        budget: people::TemplateBudget::AsAllowed { cap: None },
        deadline: people::TemplateDeadline::None,
        tracked: Some(1),
        flags: jig_domain_tasks::Flags::PLAIN,
        label: 0,
        grace: None,
        llm: None,
        resources: Box::new([people::Symbol { connector: 0, symbol: super::super::forge_route::GOAL_BRANCH_SYMBOL }]),
    }
}

fn repository(home: bool, number: u32, name: &[u8]) -> forge::Repository {
    forge::Repository {
        contexts: Box::new([]),
        workflows: [b"checks.yml".as_slice(), b"ci.yml", b"fuzzy.yml"].map(Box::from).into(),
        project: PROJECT,
        home,
        provider: skein_forge::api::Repository { forge: 1, repository: number },
        host: b"forge.example".as_slice().into(),
        owner: b"org".as_slice().into(),
        name: name.into(),
        prefix: Box::new([]),
        role: forge::Role::Owned,
        ci: true,
        checks: Box::new([3]),
        kinds: forge::Kinds {
            read: true,
            push: true,
            open: true,
            land: true,
            review: true,
            status: true,
            comment: true,
            issue: true,
            branch: true,
        },
        protection: forge::Protection::Absent,
        settings: skein_forge::api::Settings {
            default_branch: b"trunk".as_slice().into(),
            merge: true,
            squash: false,
            rebase: false,
        },
    }
}

fn repositories(domain: &mut Domain, env: &Env<crate::Limits>, home: bool) {
    domain.forge.restore_begin(env.wall);
    for repository in [repository(false, 1, b"first"), repository(home, 2, b"home")] {
        forge::step(
            &mut domain.forge,
            &env.with(env.limits.forge),
            forge::Event::Restore { record: forge::Stored::Repository(repository) },
            &mut Queue::uncapped(),
        );
    }
    let provider = skein_forge::api::Repository { forge: 1, repository: if home { 2 } else { 1 } };
    assert!(domain.forge.project_landing(
        PROJECT,
        temper_engine_domain_forge_change::Way::Turn,
        Box::new([forge::RepositoryGates {
            repository: provider,
            gates: Box::new([temper_engine_domain_forge_landing::Gate {
                name: b"focused".as_slice().into(),
                workflow: b"checks.yml".as_slice().into(),
                ignored: Box::new([]),
                blocking: true,
                base: false,
            }]),
            tiers: Box::new([forge::Tier {
                name: b"fuzzy".as_slice().into(),
                workflow: b"fuzzy.yml".as_slice().into(),
                ignored: Box::new([]),
                artifact: b"result".as_slice().into(),
                format: temper_engine_domain_forge_tiers::Format::Junit,
                follows: forge::Follows::Goals,
                span: None,
                description: b"the slow suites".as_slice().into(),
            }]),
        }]),
        temper_engine_domain_forge_landing::Project::DEFAULT,
        forge::AfterLanding {
            verdict: temper_engine_domain_forge_tiers::Settings {
                retries: 1,
                suspects: temper_engine_domain_forge_tiers::Suspects::TellAll
            },
            bound: skein_lib::Duration::from_secs(60)
        }
    ));
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks the project inputs and the roles those inputs run")]
fn inputs_gather_the_repository_start_and_readers_roles() {
    // Kind 11 encodes two bools (temper/docs/design/protocol/configuration/settings.md, 2).
    const GOAL_BYTES: u32 = 2;
    let both = settings::TemplateGoal { design: true, close_out: true };
    for (name, adopted, home, goal, way, merge_job, (held, laid), expected_repository, expected_start) in [
        ("no repository", false, false, false, 0, false, (&[][..], false), None, ways::Start::Chat),
        ("first adopted", true, false, false, 0, false, (&[][..], false), Some(b"first".as_slice()), ways::Start::Chat),
        ("home", true, true, false, 0, false, (&[][..], false), Some(b"home".as_slice()), ways::Start::Chat),
        (
            "another template's",
            true,
            true,
            true,
            4,
            false,
            (&[1, 2][..], false),
            Some(b"home".as_slice()),
            ways::Start::Goal { charter: COORDINATOR, design: false, close_out: false },
        ),
        (
            "goal designs",
            true,
            true,
            true,
            4,
            false,
            (&[][..], true),
            Some(b"home".as_slice()),
            ways::Start::Goal { charter: COORDINATOR, design: true, close_out: true },
        ),
        (
            "goal held",
            true,
            true,
            true,
            4,
            false,
            (&[4][..], false),
            Some(b"home".as_slice()),
            ways::Start::Goal { charter: COORDINATOR, design: true, close_out: true },
        ),
        (
            "goal plans",
            true,
            true,
            true,
            4,
            false,
            (&[][..], false),
            Some(b"home".as_slice()),
            ways::Start::Goal { charter: COORDINATOR, design: false, close_out: false },
        ),
        (
            "goal with merge job",
            true,
            true,
            true,
            4,
            true,
            (&[][..], false),
            Some(b"home".as_slice()),
            ways::Start::Goal { charter: COORDINATOR, design: false, close_out: false },
        ),
        (
            "chat with merge job",
            true,
            true,
            false,
            0,
            true,
            (&[][..], false),
            Some(b"home".as_slice()),
            ways::Start::Chat,
        ),
        (
            "default goal",
            false,
            false,
            true,
            0,
            false,
            (&[][..], false),
            None,
            ways::Start::Goal { charter: COORDINATOR, design: false, close_out: false },
        ),
        ("missing template", false, false, true, 9, false, (&[][..], false), None, ways::Start::Chat),
    ] {
        let (mut domain, env) = fixture();
        if adopted {
            repositories(&mut domain, &env, home);
        }
        if merge_job {
            assert!(domain.forge.project_merge_job(PROJECT, b"merge".as_slice().into()));
        }
        for &number in held {
            domain.forge.keep_settings(
                &env.with(env.limits.forge),
                1,
                None,
                &[settings::Asked {
                    kind: 11,
                    scope: 0,
                    number,
                    edit: settings::Edit::Set(settings::SizedValue {
                        value: settings::Value::Goal(both),
                        size: GOAL_BYTES,
                    }),
                }],
                &mut Queue::uncapped(),
            );
        }
        let mut items = List::uncapped();
        items
            .push(ways::Item {
                kind: 7,
                scope: PROJECT,
                number: 1,
                edit: ways::Edit::Set(settings::Value::Ways(settings::ProjectWays {
                    drafts: true,
                    goals: goal,
                    changes: true,
                    way,
                })),
            })
            .unwrap();
        if laid {
            items
                .push(ways::Item { kind: 11, scope: 0, number: 4, edit: ways::Edit::Set(settings::Value::Goal(both)) })
                .unwrap();
        }
        let got = inputs(&domain, PROJECT, READER, items.as_slice(), None, None);
        let repository = match &got.repository {
            Some(repository) => Some(repository.name.as_ref()),
            None => None,
        };
        assert_eq!(repository, expected_repository, "{name}");
        assert_eq!(got.branch.as_ref(), if adopted { b"trunk".as_slice() } else { b"main" }, "{name}");
        assert_eq!(got.start, expected_start, "{name}");
        assert_eq!(got.merge_job, merge_job, "{name}");
        let mut catch_up = false;
        let mut drawn_design = false;
        let mut drawn_close_out = false;
        for stop in &ways::derive(&got).stops {
            catch_up |= stop.step == ways::Step::CatchUp;
            drawn_design |= stop.words.as_ref() == b"design";
            drawn_close_out |= stop.words.as_ref() == b"close-out";
        }
        assert_eq!(catch_up, merge_job && goal, "{name}");
        let (design, close_out) = match expected_start {
            ways::Start::Chat => (false, false),
            ways::Start::Goal { design, close_out, .. } => (design, close_out),
        };
        assert_eq!(drawn_design, design, "{name}");
        assert_eq!(drawn_close_out, close_out, "{name}");
        assert_eq!(
            got.roles.as_ref(),
            [
                ways::Role { number: 0, name: b"owner".as_slice().into(), yours: false, holders: 0 },
                ways::Role { number: 2, name: b"member".as_slice().into(), yours: true, holders: 1 }
            ],
            "{name}"
        );
        assert_eq!(got.agents.as_ref(), [ways::Agent { charter: 1, model: Some(b"scripted".as_slice().into()),
            effort: Some(b"high".as_slice().into()), usable: false,
            can: b"read the code and the forge; change and push its branch; run commands; start sub-agents; never merge".as_slice().into(),
            asked: b"Complete the task within its authority.".as_slice().into() }], "{name}");
        assert_eq!(
            got.by,
            if adopted {
                temper_engine_domain_forge_change::Way::Turn
            } else {
                temper_engine_domain_forge_change::Way::Queue
            }
        );
        if let Some(repository) = got.repository {
            assert_eq!(repository.host.as_ref(), b"forge.example", "{name}");
            assert_eq!(repository.owner.as_ref(), b"org", "{name}");
            assert!(repository.ci, "{name}");
            assert_eq!(repository.checks.as_ref(), [3], "{name}");
            assert_eq!(repository.workflows.as_ref(), [Box::from(b"ci.yml".as_slice())], "{name}");
            assert_eq!(repository.gates.as_ref(), [Box::from(b"focused".as_slice())], "{name}");
            assert_eq!(
                repository.tiers.as_ref(),
                [ways::Tier {
                    name: b"fuzzy".as_slice().into(),
                    follows: forge::Follows::Goals,
                    description: b"the slow suites".as_slice().into()
                }],
                "{name}"
            );
        }
    }
    // Each row varies a role source or the words the forge holds for it.
    for (goal, review, duplicate, missing, questions_up, drafting, words, names) in [
        (
            false,
            false,
            false,
            false,
            false,
            Some(TemplateExecutor::Charter(12)),
            RoleWords::Stock,
            &["Chat", "Ask temper", "Drafter", "Coding agent", "Status agent"][..],
        ),
        (
            false,
            true,
            false,
            false,
            false,
            None,
            RoleWords::Stock,
            &["Chat", "Ask temper", "Coding agent", "Reviewer", "Status agent"][..],
        ),
        (
            true,
            true,
            false,
            false,
            false,
            Some(TemplateExecutor::Charter(12)),
            RoleWords::Stock,
            &["Chat", "Ask temper", "Drafter", "Coordinator", "Coding agent", "Reviewer", "Status agent"][..],
        ),
        (
            false,
            true,
            true,
            false,
            false,
            None,
            RoleWords::Stock,
            &["Chat", "Ask temper", "Coding agent", "Status agent"][..],
        ),
        (false, false, false, true, false, None, RoleWords::Stock, &["Chat", "Coding agent"][..]),
        (
            false,
            false,
            false,
            false,
            true,
            Some(TemplateExecutor::Chosen),
            RoleWords::Stock,
            &["Chat", "Ask temper", "Coding agent", "Status agent"][..],
        ),
        (
            false,
            false,
            false,
            false,
            false,
            Some(TemplateExecutor::Charter(1)),
            RoleWords::Stock,
            &["Chat", "Ask temper", "Coding agent", "Status agent"][..],
        ),
        (
            false,
            false,
            false,
            false,
            false,
            None,
            RoleWords::Changed,
            &["Chat", "Ask temper", "Coding agent", "Status agent"][..],
        ),
        (
            false,
            false,
            false,
            false,
            false,
            None,
            RoleWords::Removed,
            &["Chat", "Ask temper", "Coding agent", "Status agent"][..],
        ),
    ] {
        let (mut domain, env) = roles_fixture(goal, review, duplicate, missing, questions_up, drafting, words);
        let mut expected = None;
        for address in [b"roles".as_slice(), b"ROLES", b"roles @revision 3", b"roles extra"] {
            let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
            view::read(
                &mut domain,
                &env,
                &mut decision,
                skein_lib::ReplyTo::new(Token::new(70)),
                1,
                PROJECT,
                address.into(),
            );
            crate::accept_routed(&mut domain.journal, &mut domain.core.counters, &env.limits.journal, decision, false)
                .unwrap();
            let mut released = Queue::uncapped();
            crate::resume(&mut domain.journal, &mut released);
            let Some(crate::JournalOutput::Deliver(crate::Delivery::ForgeRead {
                answer: crate::ForgeReadAnswer::View(view),
                ..
            })) = released.pop()
            else {
                panic!("view answer")
            };
            if address.starts_with(b"roles ") {
                let ways::View::NotDrawn(_) = view.as_ref() else { panic!("invalid roles address: {address:?}") };
                continue;
            }
            let ways::View::Roles(roles) = view.as_ref() else { panic!("roles answer") };
            assert_eq!(roles.len(), names.len(), "goal {goal}, review {review}");
            for ((number, card), name) in roles.iter().zip(names) {
                assert_eq!(card.name.as_ref(), name.as_bytes(), "goal {goal}, review {review}");
                let who = match card.name.as_ref() {
                    b"Chat" => "For each chat",
                    b"Ask temper" => "When you ask",
                    b"Drafter" => "When you start a draft",
                    b"Coordinator" => "For each goal",
                    b"Coding agent" | b"Reviewer" => "For each change",
                    b"Status agent" => "When you look at Home after landings",
                    _ => panic!("unexpected role"),
                };
                let does = match *number {
                    1 => match words {
                        RoleWords::Stock => "talks with you and asks for the changes",
                        RoleWords::Changed => "talks about your project",
                        RoleWords::Removed => "Complete the task within its authority.",
                    },
                    3 => "Read only what the change needs.",
                    4 => "",
                    12 => "writes a draft with you, one question at a time, and starts nothing",
                    _ => "Complete the task within its authority.",
                };
                let expected_who = if does.is_empty() {
                    Box::from(who.as_bytes())
                } else {
                    skein_lib::bytes::concat(&[who.as_bytes(), b": ", does.as_bytes()])
                };
                assert_eq!(card.who.as_ref(), expected_who.as_ref(), "charter {number}");
                let can = match card.name.as_ref() {
                    b"Ask temper" => "read temper's records and the forge; propose settings; never change anything",
                    b"Status agent" => "read the facts in its task; never merge",
                    _ => {
                        "read the code and the forge; change and push its branch; run commands; start sub-agents; never merge"
                    }
                };
                assert_eq!(card.facts.len(), 3);
                for (fact, label) in card.facts.iter().zip([b"Model".as_slice(), b"Can", b"Asked to"]) {
                    assert_eq!(fact.label.as_ref(), label);
                }
                assert_eq!(card.facts[1].value.as_ref(), can.as_bytes(), "charter {number}");
                assert!(skein_lib::bytes::find(&card.facts[0].value, b"no account runs it now").is_some());
                let asked = match *number {
                    3 => "“Read only what the change needs.”",
                    4 => "“”",
                    _ => "“Complete the task within its authority.”",
                };
                assert_eq!(card.facts[2].value.as_ref(), asked.as_bytes(), "charter {number}");
            }
            if let Some(expected) = &expected {
                assert_eq!(view.as_ref(), expected);
            }
            expected = Some(view.as_ref().clone());
        }
    }
}

#[test]
fn committed_rules_and_item_overlays_feed_the_same_inputs() {
    use jig_domain_core::effects::connector;
    use jig_domain_core::settings::application;
    const RULE_BYTES: u32 = 32;
    let (mut domain, env) = fixture();
    assert!(document(&domain, PROJECT, READER).words.is_none());
    let rule = settings::LandingRule {
        sets: Box::new([]),
        pattern: forge::GatePattern {
            segments: Box::new([b"branch".as_slice().into()]),
            last: forge::GateLast::Open(Box::new([])),
        },
        enforce: true,
        ci: false,
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([]),
    };
    // The root passes typed items to the forge at this committing ask.
    domain.kinds = Box::new([crate::KindValue {
        kind: 2,
        scope: PROJECT,
        number: 1,
        value: settings::SizedValue { size: RULE_BYTES, value: settings::Value::Rule(rule.clone()) },
    }]);
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    super::super::kinds::ask(
        &mut domain,
        &env,
        &mut decision,
        0,
        connector::Ask::KeepSettings {
            revision: 2,
            proposal: None,
            items: Box::new([application::Item {
                kind: 2,
                scope: PROJECT,
                number: 1,
                edit: application::Edit::Set(Box::new([])),
            }]),
        },
    );
    let got = inputs(&domain, PROJECT, READER, &[], None, None);
    assert_eq!(got.rules.as_ref(), [settings::Held { scope: PROJECT, number: 1, rule: rule.clone() }]);
    let mut laid = rule.clone();
    laid.ci = true;
    let item =
        ways::Item { kind: 2, scope: PROJECT, number: 1, edit: ways::Edit::Set(settings::Value::Rule(laid.clone())) };
    let got = inputs(&domain, PROJECT, READER, &[item], None, None);
    assert_eq!(got.rules.as_ref(), [settings::Held { scope: PROJECT, number: 1, rule: laid }]);
    let mut stock = rule;
    stock.up_to_date = true;
    domain.forge.give_settings_defaults(Box::new([settings::Stock {
        kind: 2,
        scope: 0,
        number: 1,
        value: settings::SizedValue { size: RULE_BYTES, value: settings::Value::Rule(stock.clone()) },
    }]));
    let item = ways::Item { kind: 2, scope: PROJECT, number: 1, edit: ways::Edit::Remove };
    let got = inputs(&domain, PROJECT, READER, &[item], None, None);
    assert_eq!(got.rules.as_ref(), [settings::Held { scope: PROJECT, number: 1, rule: stock }]);
    let words = settings::ProjectWords {
        headline: b"An owner approves every change".as_slice().into(),
        lede: b"Agents make changes, then wait for an owner to approve them.".as_slice().into(),
    };
    domain.forge.keep_settings(
        &env.with(env.limits.forge),
        3,
        None,
        &[settings::Asked {
            kind: 10,
            scope: PROJECT,
            number: 1,
            edit: settings::Edit::Set(settings::SizedValue {
                size: 120,
                value: settings::Value::ProjectWords(words.clone()),
            }),
        }],
        &mut Queue::uncapped(),
    );
    assert_eq!(document(&domain, PROJECT, READER).words, Some(words));
    assert!(document(&domain, PROJECT.checked_add(1).unwrap(), READER).words.is_none());
}

#[test]
fn an_agent_uses_the_first_held_model_or_its_first_fallback() {
    for (order, offer, expected) in [
        (&[][..], &[][..], (None, false)),
        (&[0, 1][..], &[1][..], (Some(1), true)),
        (&[1, 0][..], &[0, 1][..], (Some(1), true)),
        (&[0, 1][..], &[][..], (Some(0), false)),
        (&[0, 1][..], &[2][..], (Some(0), false)),
    ] {
        let mut offered = List::uncapped();
        for &model in offer {
            offered.push(core::documents::Offered { model, accounts: List::uncapped() }).unwrap();
        }
        assert_eq!(agents::select(order, offered.as_slice()), expected, "{order:?}: {offer:?}");
    }
}

#[derive(Clone, Copy)]
enum RoleWords {
    Stock,
    Changed,
    Removed,
}

#[expect(
    clippy::fn_params_excessive_bools,
    clippy::too_many_lines,
    reason = "the view table keeps its independent role sources, instructions and held words in one fixture"
)]
fn roles_fixture(
    goal: bool,
    review: bool,
    duplicate: bool,
    missing: bool,
    questions_up: bool,
    drafting: Option<TemplateExecutor>,
    words: RoleWords,
) -> (Domain, Env<crate::Limits>) {
    let mut config = resume_fixture::config(7);
    let mut limits = resume_fixture::limits();
    limits.tasks.charters = 16;
    limits.templates = 6;
    let mut charters = List::uncapped();
    for number in 1..=12 {
        if missing && (number == 8 || number == 9) {
            continue;
        }
        let mut charter = config.core.defaults.charters[0].clone();
        charter.number = number;
        if number == 3 {
            charter.policy.instructions = b"Read only what the change needs. Then make the change.".as_slice().into();
        } else if number == 4 {
            charter.policy.instructions = b"  \n ".as_slice().into();
        }
        if number == 8 || number == 9 {
            charter.policy.inspect = false;
            charter.policy.modify = false;
            charter.policy.shell = false;
            charter.policy.agents = false;
            charter.policy.tools.0 = if number == 8 { jig_domain_core::ToolFamilies::JIG.settings } else { 0 };
        }
        charters.push(charter).unwrap();
    }
    config.core.defaults.charters = charters.into_boxed();
    let mut template = goal_template();
    let mut templates = List::uncapped();
    for (number, charter) in [(1, 1), (4, 5), (5, 8)] {
        template.number = number;
        template.executor = TemplateExecutor::Charter(charter);
        templates.push(template.clone()).unwrap();
    }
    if let Some(executor) = drafting {
        template.number = 6;
        template.executor = executor;
        templates.push(template).unwrap();
    }
    config.core.defaults.templates = templates.into_boxed();
    let (mut domain, env) = resume_fixture::root_with_records(config, &limits, &[]);
    let mut defaults = List::uncapped();
    for (kind, number, value) in [
        (3, 1, settings::Value::Charter(3)),
        (3, 2, settings::Value::Charter(if duplicate { 3 } else { 4 })),
        (7, 1, settings::Value::Ways(settings::ProjectWays { drafts: goal, goals: goal, changes: true, way: 4 })),
        (
            6,
            1,
            settings::Value::Questions(if questions_up {
                temper_engine_domain_forge_change::Route::Up
            } else {
                temper_engine_domain_forge_change::Route::None
            }),
        ),
    ] {
        defaults
            .push(settings::Stock {
                kind,
                scope: if kind == 3 { 0 } else { PROJECT },
                number,
                value: settings::SizedValue { size: 32, value },
            })
            .unwrap();
    }
    if review {
        defaults
            .push(settings::Stock {
                kind: 2,
                scope: 0,
                number: 1,
                value: settings::SizedValue {
                    size: 32,
                    value: settings::Value::Rule(settings::LandingRule {
                        sets: Box::new([]),
                        pattern: forge::GatePattern {
                            segments: Box::new([b"branch".as_slice().into()]),
                            last: forge::GateLast::Open(Box::new([])),
                        },
                        enforce: true,
                        ci: false,
                        up_to_date: false,
                        gates: Box::new([settings::RuleGate {
                            number: 2,
                            blocking: true,
                            freshness: forge::JudgeFreshness::Clean,
                        }]),
                        approvals: Box::new([]),
                    }),
                },
            })
            .unwrap();
    }
    for (number, line) in [
        (1, b"talks with you and asks for the changes".as_slice()),
        (12, b"writes a draft with you, one question at a time, and starts nothing"),
    ] {
        defaults
            .push(settings::Stock {
                kind: 9,
                scope: 0,
                number,
                value: settings::SizedValue { size: 128, value: settings::Value::CharterWords(line.into()) },
            })
            .unwrap();
    }
    domain.forge.give_settings_defaults(defaults.into_boxed());
    let edit = match words {
        RoleWords::Stock => None,
        RoleWords::Changed => Some(settings::Edit::Set(settings::SizedValue {
            size: 128,
            value: settings::Value::CharterWords(b"talks about your project".as_slice().into()),
        })),
        RoleWords::Removed => Some(settings::Edit::Remove),
    };
    if let Some(edit) = edit {
        domain.forge.keep_settings(
            &env.with(env.limits.forge),
            1,
            None,
            &[settings::Asked { kind: 9, scope: 0, number: 1, edit }],
            &mut Queue::uncapped(),
        );
    }
    people::step(
        &mut domain.core.people,
        &env.with(env.limits.people),
        people::Event::SignedIn {
            reply_to: skein_lib::ReplyTo::new(Token::new(69)),
            person: 1,
            sign_in: 1,
            identity: people::Identity { key: forge::user_key(1, 7), login: Box::new([]), name: Box::new([]) },
            kind: people::Kind::Person,
            held: None,
        },
        &mut Queue::uncapped(),
    );
    (domain, env)
}

#[test]
fn draft_first_requires_the_closer_and_close_job_in_both_delegations() {
    use jig_domain_authority::{self as authority, Executor};
    let required =
        Box::new([Executor::Charter(5), Executor::Procedure(6), Executor::Role(0), Executor::Job(3), Executor::Job(4)]);
    let mut config = resume_fixture::config(7);
    config.core.defaults.rules.ceiling.delegation.kinds = required.clone();
    let policy = &mut config.core.seed_projects[0].policy.policy;
    policy.ceiling.delegation.kinds = required.clone();
    policy.roles[0].authority.delegation.kinds = required.clone();
    let mut limits = resume_fixture::limits();
    // Draft first's delegation occupies five executor places in each limit.
    limits.authority.executors = 5;
    // Draft first's delegation occupies five executor places in each limit.
    limits.tasks.executor_kinds = 5;
    let (mut domain, _) = resume_fixture::root_with_config_and_limits(config, &limits);
    let complete = domain.core.authority.policy(PROJECT).unwrap().clone();
    assert!(!lacks_delegation(&domain, PROJECT));
    for missing in [Executor::Procedure(6), Executor::Job(4)] {
        for ceiling in [true, false] {
            let mut policy = complete.clone();
            let mut kept = List::with_capacity(u32::try_from(required.len()).unwrap());
            for executor in required.iter() {
                if *executor != missing {
                    kept.push(*executor).unwrap();
                }
            }
            policy.roles[0].authority.delegation.kinds = kept.into_boxed();
            if ceiling {
                policy.ceiling.delegation.kinds.clone_from(&policy.roles[0].authority.delegation.kinds);
            }
            let mut out = Queue::uncapped();
            authority::step(
                &mut domain.core.authority,
                authority::Event::Policy { project: PROJECT, policy },
                &mut out,
            );
            assert!(lacks_delegation(&domain, PROJECT), "{missing:?}, ceiling {ceiling}");
        }
    }
}

#[test]
fn path_sets_and_people_word_the_document_and_rule_card() {
    use temper_protocol_reaches::{Classes, PathSet};
    let (mut domain, env) = roster_fixture(true);
    let classes = Classes {
        items: Box::new([]),
        sets: Box::new([PathSet {
            number: 1,
            name: b"protocol core".as_slice().into(),
            patterns: [b"protocol/**".as_slice(), b"src/wire.rs"].map(Box::from).into(),
        }]),
    };
    let rule = settings::LandingRule {
        sets: Box::new([1]),
        pattern: forge::GatePattern {
            segments: Box::new([b"branch".as_slice().into()]),
            last: forge::GateLast::Exact(b"main".as_slice().into()),
        },
        enforce: true,
        ci: false,
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([settings::RuleApproval { role: 0, people: 1, freshness: forge::JudgeFreshness::Clean }]),
    };
    domain.forge.give_settings_defaults(
        [(2, settings::Value::Rule(rule)), (8, settings::Value::Classes(classes.clone()))]
            .map(|(kind, value)| settings::Stock {
                kind,
                scope: if kind == 2 { 0 } else { PROJECT },
                number: 1,
                value: settings::SizedValue { size: 64, value },
            })
            .into(),
    );
    let got = inputs(&domain, PROJECT, READER, &[], None, None);
    assert_eq!(got.sets, classes.sets);
    assert_eq!(got.roles.first().unwrap().holders, 1, "a service holds none");
    assert!(got.roles.first().unwrap().yours);
    assert!(!inputs(&domain, PROJECT, 2, &[], None, None).roles.first().unwrap().yours, "a service cannot sign");
    let doc = document(&domain, PROJECT, READER);
    assert_eq!(doc.rules.rows.len(), 1);
    assert_eq!(doc.rules.rows.first().unwrap().sign_off.as_ref(), b"you");
    let mut found = false;
    for stop in &doc.own.stops {
        if stop.step == ways::Step::Approve {
            found = true;
            assert_eq!((stop.words.as_ref(), stop.over.as_ref()), (b"you".as_slice(), b"if protocol core".as_slice()));
        }
    }
    assert!(found);
    for edit in [ways::Edit::Set(settings::Value::Classes(Classes::default())), ways::Edit::Remove] {
        let item = ways::Item { kind: 8, scope: PROJECT, number: 1, edit };
        assert!(
            inputs(&domain, PROJECT, READER, &[item], None, None).sets.is_empty(),
            "classes items lay over the project's sets"
        );
    }
    people::step(
        &mut domain.core.people,
        &env.with(env.limits.people),
        people::Event::SignedIn {
            reply_to: skein_lib::ReplyTo::new(Token::new(69)),
            person: READER,
            sign_in: 1,
            identity: people::Identity { key: forge::user_key(1, 7), login: Box::new([]), name: Box::new([]) },
            kind: people::Kind::Person,
            held: None,
        },
        &mut Queue::uncapped(),
    );
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    view::read(
        &mut domain,
        &env,
        &mut decision,
        skein_lib::ReplyTo::new(Token::new(70)),
        1,
        PROJECT,
        b"rule 2/1".as_slice().into(),
    );
    crate::accept_routed(&mut domain.journal, &mut domain.core.counters, &env.limits.journal, decision, false).unwrap();
    let mut released = Queue::uncapped();
    crate::resume(&mut domain.journal, &mut released);
    let Some(crate::JournalOutput::Deliver(crate::Delivery::ForgeRead {
        answer: crate::ForgeReadAnswer::View(view),
        ..
    })) = released.pop()
    else {
        panic!("rule answer")
    };
    let ways::View::Rule { card, .. } = view.as_ref() else { panic!("rule card") };
    assert_eq!(card.facts.first().unwrap().value.as_ref(), b"the branch main, where a change touches protocol core");
    assert_eq!(card.facts.last().unwrap().value.as_ref(), b"you approve");
}
