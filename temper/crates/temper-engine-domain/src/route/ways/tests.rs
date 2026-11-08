//! Gather the core and forge facts before deriving any words (temper/docs/design/domain/forge.md, 8.13).
use super::*;
use crate::tests::resume_fixture;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Env, Queue};

const PROJECT: u32 = 1;
const READER: u64 = 1;
const COORDINATOR: u32 = 1;

fn fixture() -> (Domain, Env<crate::Limits>) {
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
    config.core.defaults.templates = Box::new([core::Template {
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
    }]);
    let mut later = config.core.defaults.templates[0].clone();
    later.number = 6;
    later.executor = TemplateExecutor::Chosen;
    config.core.defaults.templates = Box::new([later, config.core.defaults.templates[0].clone()]);
    let records = [
        crate::Record::Core(core::Record::People(people::Stored::Person {
            number: READER,
            identity: people::Identity { key: forge::user_key(1, 7), login: Box::new([]), name: Box::new([]) },
        })),
        crate::Record::Core(core::Record::People(people::Stored::Roles {
            project: PROJECT,
            holdings: Box::new([people::Holding { person: READER, role: people::Role::Member }]),
        })),
    ];
    let mut limits = resume_fixture::limits();
    limits.documents.roles = 2;
    limits.authority.roles = 2; // Owner and member.
    resume_fixture::root_with_records(config, &limits, &records)
}

fn repository(home: bool, number: u32, name: &[u8]) -> forge::Repository {
    forge::Repository {
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
                blocking: true
            }]),
            tiers: Box::new([forge::Tier {
                name: b"fuzzy".as_slice().into(),
                workflow: b"fuzzy.yml".as_slice().into(),
                ignored: Box::new([]),
                artifact: b"result".as_slice().into(),
                format: temper_engine_domain_forge_tiers::Format::Junit,
                follows: forge::Follows::Goals,
                span: None
            }]),
        }]),
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
fn inputs_gather_the_repository_start_and_readers_roles() {
    for (name, adopted, home, goal, way, merge_job, expected_repository, expected_start) in [
        ("no repository", false, false, false, 0, false, None, ways::Start::Chat),
        ("first adopted", true, false, false, 0, false, Some(b"first".as_slice()), ways::Start::Chat),
        ("home", true, true, false, 0, false, Some(b"home".as_slice()), ways::Start::Chat),
        ("goal", true, true, true, 4, false, Some(b"home".as_slice()), ways::Start::Goal { charter: COORDINATOR }),
        (
            "goal with merge job",
            true,
            true,
            true,
            4,
            true,
            Some(b"home".as_slice()),
            ways::Start::Goal { charter: COORDINATOR },
        ),
        ("chat with merge job", true, true, false, 0, true, Some(b"home".as_slice()), ways::Start::Chat),
        ("default goal", false, false, true, 0, false, None, ways::Start::Goal { charter: COORDINATOR }),
        ("missing template", false, false, true, 9, false, None, ways::Start::Chat),
    ] {
        let (mut domain, env) = fixture();
        if adopted {
            repositories(&mut domain, &env, home);
        }
        if merge_job {
            assert!(domain.forge.project_merge_job(PROJECT, b"merge".as_slice().into()));
        }
        let items = [ways::Item {
            kind: 7,
            scope: PROJECT,
            number: 1,
            edit: ways::Edit::Set(settings::Value::Ways(settings::ProjectWays {
                drafts: true,
                goals: goal,
                changes: true,
                away: false,
                way,
            })),
        }];
        let got = inputs(&domain, PROJECT, READER, &items, None, None);
        let repository = match &got.repository {
            Some(repository) => Some(repository.name.as_ref()),
            None => None,
        };
        assert_eq!(repository, expected_repository, "{name}");
        assert_eq!(got.branch.as_ref(), if adopted { b"trunk".as_slice() } else { b"main" }, "{name}");
        assert_eq!(got.start, expected_start, "{name}");
        assert_eq!(got.merge_job, merge_job, "{name}");
        let mut catch_up = false;
        for stop in &ways::derive(&got).stops {
            catch_up |= stop.step == ways::Step::CatchUp;
        }
        assert_eq!(catch_up, merge_job && goal, "{name}");
        assert_eq!(
            got.roles.as_ref(),
            [
                ways::Role { number: 0, name: b"owner".as_slice().into(), yours: false },
                ways::Role { number: 2, name: b"member".as_slice().into(), yours: true }
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
            assert_eq!(repository.gates.as_ref(), [Box::from(b"focused".as_slice())], "{name}");
            assert_eq!(
                repository.tiers.as_ref(),
                [ways::Tier { name: b"fuzzy".as_slice().into(), follows: forge::Follows::Goals }],
                "{name}"
            );
        }
    }
}

#[test]
fn committed_rules_and_item_overlays_feed_the_same_inputs() {
    use jig_domain_core::effects::connector;
    use jig_domain_core::settings::application;
    const RULE_BYTES: u32 = 32;
    let (mut domain, env) = fixture();
    let rule = settings::LandingRule {
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
    super::super::kinds::ask(
        &mut domain,
        &env,
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
