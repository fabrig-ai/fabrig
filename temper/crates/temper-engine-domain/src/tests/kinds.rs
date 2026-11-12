//! The root's typed settings routes (temper/docs/design/domain/routing.md, 6;
//! temper/docs/design/protocol/configuration/settings.md, 2).
use super::*;
use crate::{Domain, KindValue, Limits as RootLimits, Work};
use jig_domain_authority as authority;
use jig_domain_core::{Event, effects::connector, settings::application};
use skein_lib::Env;
use temper_engine_domain_forge::{self as forge, settings};

const PROJECT: u32 = 1;
const RULE: u32 = 1;
const PROPOSAL: u64 = 7;
const WITHDRAWN_PROPOSAL: u64 = 8;
const CHARTERS: u32 = 8;
const OWNER: Token = Token::new(19);
const RESTORE_OWNER: Token = Token::new(20);
// The typed fixture carries the encoding's entrance-measured size.
const RULE_BYTES: u32 = 32;
// A charter's encoded u32 occupies four bytes.
const CHARTER_BYTES: u32 = 4;

fn rule(ci: bool) -> settings::SizedValue {
    settings::SizedValue {
        size: RULE_BYTES,
        value: settings::Value::Rule(settings::LandingRule {
            sets: Box::new([]),
            pattern: forge::GatePattern {
                segments: Box::new([Box::from(b"forge".as_slice())]),
                last: forge::GateLast::Open(Box::default()),
            },
            enforce: true,
            ci,
            up_to_date: true,
            gates: Box::default(),
            approvals: Box::default(),
        }),
    }
}

fn charter(number: u32) -> settings::SizedValue {
    settings::SizedValue { value: settings::Value::Charter(number), size: CHARTER_BYTES }
}

fn root() -> (Domain, Env<RootLimits>) {
    let mut limits = resume_fixture::limits();
    limits.tasks.charters = CHARTERS;
    let (mut domain, env) = resume_fixture::root_with_config_and_limits(resume_fixture::config(7), &limits);
    domain.forge.give_settings_defaults(Box::new([
        settings::Stock { kind: 1, scope: 0, number: RULE, value: rule(true) },
        settings::Stock { kind: 3, scope: 0, number: 1, value: charter(2) },
        settings::Stock { kind: 3, scope: 0, number: 2, value: charter(4) },
    ]));
    (domain, env)
}

fn project_rule(edit: application::Edit) -> application::Item {
    application::Item { kind: 2, scope: PROJECT, number: RULE, edit }
}

fn values(value: settings::SizedValue) -> Box<[KindValue]> {
    Box::new([KindValue { kind: 2, scope: PROJECT, number: RULE, value }])
}

fn ask(domain: &mut Domain, env: &Env<RootLimits>, decision: &mut Decision, ask: connector::Ask) {
    let mut requests = Queue::uncapped();
    requests.push(jig_domain_core::Request::Ask {
        connector: domain.config.forge_connector,
        ask: jig_domain_core::Ask::Effect(ask),
    });
    crate::route::route_core_requests(domain, env, decision, requests);
}

fn judge(
    domain: &mut Domain,
    env: &Env<RootLimits>,
    decision: &mut Decision,
    proposal: Option<u64>,
    items: Box<[application::Item]>,
) {
    ask(domain, env, decision, connector::Ask::JudgeSettings { owner: OWNER, revision: 2, proposal, items });
}

fn answer(domain: &mut Domain) -> Box<[connector::Judged]> {
    let Some(Work::Core(Event::EffectConnector(connector::Event::SettingsJudged { owner, judged }))) =
        domain.work.pop()
    else {
        panic!("the same step returns SettingsJudged to the core");
    };
    assert_eq!(owner, OWNER);
    assert!(domain.work.is_empty());
    judged
}

fn restored(kind: u32, scope: u32, number: u32, edit: application::RestoredEdit) -> application::Restored {
    application::Restored { kind, scope, number, edit }
}

fn restored_answer(domain: &mut Domain, expected_owner: Token) -> Box<[connector::Judged]> {
    let Some(Work::Core(Event::EffectConnector(connector::Event::SettingsRestored { owner, judged }))) =
        domain.work.pop()
    else {
        panic!("the same step returns SettingsRestored to the core");
    };
    assert_eq!(owner, expected_owner);
    judged
}

fn committed_writes(domain: &mut Domain, env: &Env<RootLimits>, decision: Decision) -> Box<[Write]> {
    let mut out = Queue::uncapped();
    crate::domain::close(domain, env, decision, &mut out);
    let Some(Output::Commit { writes, .. }) = crate::decision::commit(&mut domain.journal, &env.limits.journal) else {
        panic!("the settings rows commit in the root's decision");
    };
    writes
}

fn saved(slot: settings::SettingSlot, edit: settings::Edit) -> Write {
    Write::Save(Record::Forge {
        row: Box::new(forge::Stored::Setting(settings::SettingRow {
            key: settings::SettingKey { kind: 2, scope: PROJECT, number: RULE, slot },
            edit,
        })),
    })
}

#[test]
fn a_judge_ask_is_answered_in_the_same_step() {
    let (mut domain, env) = root();
    domain.kinds = values(rule(false));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    judge(&mut domain, &env, &mut decision, None, Box::new([project_rule(application::Edit::Set(Box::default()))]));
    assert_eq!(
        answer(&mut domain).as_ref(),
        &[connector::Judged::Accept { size: RULE_BYTES, charters: Box::default() }]
    );
    crate::domain::close(&mut domain, &env, decision, &mut Queue::uncapped());
    assert!(domain.kinds.is_empty(), "typed values end with the decision");
}

#[test]
fn a_refusal_carries_the_forges_words() {
    let (mut domain, env) = root();
    domain.kinds = Box::new([KindValue { kind: 3, scope: 0, number: 1, value: charter(0) }]);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    judge(
        &mut domain,
        &env,
        &mut decision,
        None,
        Box::new([application::Item { kind: 3, scope: 0, number: 1, edit: application::Edit::Set(Box::default()) }]),
    );
    assert_eq!(answer(&mut domain).as_ref(), &[connector::Judged::Refuse { why: settings::Refusal::Charter.words() }]);
}

#[test]
fn a_set_without_its_typed_value_is_refused_as_another_kind() {
    let (mut domain, env) = root();
    // None of kind, scope and number can substitute for another item's value.
    domain.kinds = Box::new([
        KindValue { kind: 1, scope: PROJECT, number: RULE, value: rule(false) },
        KindValue { kind: 2, scope: PROJECT + 1, number: RULE, value: rule(false) },
        KindValue { kind: 2, scope: PROJECT, number: RULE + 1, value: rule(false) },
    ]);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    judge(
        &mut domain,
        &env,
        &mut decision,
        None,
        Box::new([project_rule(application::Edit::Set(b"opaque bytes".as_slice().into()))]),
    );
    assert_eq!(answer(&mut domain).as_ref(), &[connector::Judged::Refuse { why: settings::Refusal::Value.words() }]);
}

#[test]
fn a_keep_ask_writes_the_forges_rows() {
    let (mut domain, env) = root();
    domain.kinds = values(rule(false));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    ask(
        &mut domain,
        &env,
        &mut decision,
        connector::Ask::KeepSettings {
            revision: 2,
            proposal: None,
            items: Box::new([project_rule(application::Edit::Set(Box::default()))]),
        },
    );
    let writes = committed_writes(&mut domain, &env, decision);
    assert!(writes.contains(&saved(settings::SettingSlot::Change, settings::Edit::Set(rule(false)))));
    assert_eq!(domain.forge.setting(2, PROJECT, RULE), Some(&rule(false)));
    assert!(domain.kinds.is_empty());
}

#[test]
fn a_proposal_is_kept_judged_from_its_kept_values_and_dropped() {
    let (mut domain, env) = root();
    domain.kinds = values(rule(false));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    // Set, Remove and Reset stay in the connector's item order.
    let items = Box::new([
        project_rule(application::Edit::Set(Box::default())),
        application::Item { kind: 1, scope: 0, number: RULE, edit: application::Edit::Remove },
        application::Item { kind: 3, scope: 0, number: 1, edit: application::Edit::Reset },
    ]);
    ask(
        &mut domain,
        &env,
        &mut decision,
        connector::Ask::KeepProposalSettings { proposal: PROPOSAL, items: items.clone() },
    );
    let writes = committed_writes(&mut domain, &env, decision);
    assert!(writes.contains(&saved(settings::SettingSlot::Proposal(PROPOSAL), settings::Edit::Set(rule(false)))));
    // The next decision has no entrance values; the forge owns the proposal's.
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    judge(&mut domain, &env, &mut decision, Some(PROPOSAL), items.clone());
    assert_eq!(
        answer(&mut domain).as_ref(),
        &[
            connector::Judged::Accept { size: RULE_BYTES, charters: Box::default() },
            connector::Judged::Accept { size: 0, charters: Box::default() },
            connector::Judged::Same,
        ]
    );
    ask(
        &mut domain,
        &env,
        &mut decision,
        connector::Ask::KeepSettings { revision: 2, proposal: Some(PROPOSAL), items },
    );
    let writes = committed_writes(&mut domain, &env, decision);
    assert!(writes.contains(&saved(settings::SettingSlot::Change, settings::Edit::Set(rule(false)))));
    assert!(writes.contains(&Write::Erase(Key::Forge(forge::Key::Setting(settings::SettingKey {
        kind: 2,
        scope: PROJECT,
        number: RULE,
        slot: settings::SettingSlot::Proposal(PROPOSAL),
    })))));
    assert!(domain.forge.proposal_settings(PROPOSAL).is_empty());
    domain.kinds = values(rule(true));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    ask(
        &mut domain,
        &env,
        &mut decision,
        connector::Ask::KeepProposalSettings {
            proposal: WITHDRAWN_PROPOSAL,
            items: Box::new([project_rule(application::Edit::Set(Box::default()))]),
        },
    );
    committed_writes(&mut domain, &env, decision);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    ask(&mut domain, &env, &mut decision, connector::Ask::DropProposalSettings { proposal: WITHDRAWN_PROPOSAL });
    let writes = committed_writes(&mut domain, &env, decision);
    assert!(writes.contains(&Write::Erase(Key::Forge(forge::Key::Setting(settings::SettingKey {
        kind: 2,
        scope: PROJECT,
        number: RULE,
        slot: settings::SettingSlot::Proposal(WITHDRAWN_PROPOSAL),
    })))));
    assert!(domain.forge.proposal_settings(WITHDRAWN_PROPOSAL).is_empty());
    assert_eq!(domain.forge.setting(2, PROJECT, RULE), Some(&rule(false)));
}

#[test]
fn a_restore_ask_reaches_the_forge_and_its_answer_reaches_the_core() {
    let (mut domain, env) = root();
    let forge_env = crate::translate::environment_forge(&env);
    let previous = [
        settings::Asked { kind: 2, scope: PROJECT, number: RULE, edit: settings::Edit::Set(rule(false)) },
        settings::Asked { kind: 3, scope: 0, number: 1, edit: settings::Edit::Set(charter(6)) },
    ];
    let current = [
        settings::Asked { edit: settings::Edit::Set(rule(true)), ..previous[0].clone() },
        settings::Asked { edit: settings::Edit::Set(charter(2)), ..previous[1].clone() },
    ];
    domain.forge.keep_settings(&forge_env, 2, None, &previous, &mut Queue::uncapped());
    domain.forge.keep_settings(&forge_env, 3, None, &current, &mut Queue::uncapped());
    domain.kinds = values(rule(true));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    ask(
        &mut domain,
        &env,
        &mut decision,
        connector::Ask::RestoreSettings {
            owner: RESTORE_OWNER,
            proposal: PROPOSAL,
            items: Box::new([
                restored(3, 0, 1, application::RestoredEdit::Set { revision: 3 }),
                restored(2, PROJECT, RULE + 1, application::RestoredEdit::Set { revision: 3 }),
                restored(2, PROJECT, RULE, application::RestoredEdit::Set { revision: 3 }),
                restored(1, 0, RULE, application::RestoredEdit::Remove),
                restored(3, 0, 2, application::RestoredEdit::Reset),
            ]),
        },
    );
    let judged = restored_answer(&mut domain, RESTORE_OWNER);
    assert_eq!(
        judged.as_ref(),
        &[
            connector::Judged::Accept { size: CHARTER_BYTES, charters: Box::new([6]) },
            connector::Judged::Refuse { why: settings::Refusal::Replaced.words() },
            connector::Judged::Accept { size: RULE_BYTES, charters: Box::default() },
            connector::Judged::Accept { size: 0, charters: Box::default() },
            connector::Judged::Accept { size: CHARTER_BYTES, charters: Box::new([4]) },
        ]
    );
    let writes = committed_writes(&mut domain, &env, decision);
    assert!(writes.contains(&saved(settings::SettingSlot::Proposal(PROPOSAL), settings::Edit::Set(rule(false)))));
    let kept = domain.forge.proposal_settings(PROPOSAL);
    assert_eq!(kept.len(), 4);
    for (key, edit) in &kept {
        assert!(writes.contains(&Write::Save(Record::Forge {
            row: Box::new(forge::Stored::Setting(settings::SettingRow { key: *key, edit: edit.clone() })),
        })));
    }
    assert_eq!(domain.forge.setting(2, PROJECT, RULE), Some(&rule(true)));
    assert_eq!(domain.forge.charters(), forge::ForgeCharters { change: 2, review: 4 });
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    judge(
        &mut domain,
        &env,
        &mut decision,
        Some(PROPOSAL),
        Box::new([project_rule(application::Edit::Set(Box::default()))]),
    );
    assert_eq!(
        answer(&mut domain).as_ref(),
        &[connector::Judged::Accept { size: RULE_BYTES, charters: Box::default() }]
    );
}

#[test]
fn a_forget_ask_erases_only_older_replaced_rows_in_the_roots_decision() {
    let (mut domain, env) = root();
    let forge_env = crate::translate::environment_forge(&env);
    for (revision, value) in [(1, rule(true)), (2, rule(false)), (3, rule(true))] {
        domain.forge.keep_settings(
            &forge_env,
            revision,
            None,
            &[settings::Asked { kind: 2, scope: PROJECT, number: RULE, edit: settings::Edit::Set(value) }],
            &mut Queue::uncapped(),
        );
    }
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    ask(&mut domain, &env, &mut decision, connector::Ask::ForgetReplaced { before: 3 });
    let writes = committed_writes(&mut domain, &env, decision);
    let setting_key =
        settings::SettingKey { kind: 2, scope: PROJECT, number: RULE, slot: settings::SettingSlot::Replaced(2) };
    assert!(writes.contains(&Write::Erase(Key::Forge(forge::Key::Setting(setting_key)))));
    assert!(!writes.contains(&Write::Erase(Key::Forge(forge::Key::Setting(settings::SettingKey {
        slot: settings::SettingSlot::Replaced(3),
        ..setting_key
    })))));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    ask(
        &mut domain,
        &env,
        &mut decision,
        connector::Ask::RestoreSettings {
            owner: OWNER,
            proposal: PROPOSAL,
            items: Box::new([
                restored(2, PROJECT, RULE, application::RestoredEdit::Set { revision: 2 }),
                restored(2, PROJECT, RULE, application::RestoredEdit::Set { revision: 3 }),
            ]),
        },
    );
    let judged = restored_answer(&mut domain, OWNER);
    assert_eq!(
        judged.as_ref(),
        &[
            connector::Judged::Refuse { why: settings::Refusal::Replaced.words() },
            connector::Judged::Accept { size: RULE_BYTES, charters: Box::default() },
        ]
    );
    assert_eq!(domain.forge.setting(2, PROJECT, RULE), Some(&rule(true)));
}

fn at_connector_restore(declared: bool) -> (Domain, Env<RootLimits>) {
    use jig_domain_core::{
        settings::{Kind, KindScope},
        start::{Action, Phase},
    };
    let limits = resume_fixture::limits();
    let mut config = resume_fixture::config(7);
    config.kinds_stock = Box::new([
        settings::Stock { kind: 1, scope: 0, number: RULE, value: rule(true) },
        settings::Stock { kind: 3, scope: 0, number: 1, value: charter(1) },
    ]);
    if declared {
        config.core.kinds = Box::new([
            Kind {
                number: 1,
                scope: KindScope::Deployment,
                connector: 0,
                value_bytes: RULE_BYTES,
                per_scope: limits.forge.landing_rules,
                charters: 0,
            },
            Kind {
                number: 3,
                scope: KindScope::Deployment,
                connector: 0,
                value_bytes: CHARTER_BYTES,
                per_scope: 2,
                charters: 1,
            },
        ]);
    }
    let mut domain = Domain::new(config, &limits).unwrap();
    let env = Env { now: skein_lib::Time::ZERO, wall: Wall::EPOCH, limits };
    domain.start_wall = Some(env.wall);
    domain.core_header_loaded = true;
    domain.forge.restore_begin(env.wall);
    let core_env = crate::limits::environment_core(&env);
    let mut action = domain.core.start_begin(false, env.wall);
    for _ in 0_u32..20 {
        let mut out = Queue::uncapped();
        action = match action {
            Action::Step(Phase::RestoreConnector { connector: 0 }) => return (domain, env),
            Action::Step(phase) => domain.core.start_done(&core_env, phase, &mut out),
            Action::Commit(continuation) => domain.core.start_resume(&core_env, continuation, &mut out),
            Action::Idle | Action::Refused(_) => panic!("start stopped before the forge restore"),
        };
    }
    panic!("start did not reach the forge restore");
}

#[test]
fn a_start_gives_the_forge_its_defaults() {
    use jig_domain_core::start::Phase;
    let (mut domain, env) = at_connector_restore(true);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    for (kind, value) in [(1, rule(true)), (3, charter(2))] {
        crate::restart::restore_page_row(
            &mut domain,
            &env,
            Record::Forge {
                row: Box::new(forge::Stored::Setting(settings::SettingRow {
                    key: settings::SettingKey { kind, scope: 0, number: 1, slot: settings::SettingSlot::Default },
                    edit: settings::Edit::Set(value),
                })),
            },
        );
        crate::route::route_into(&mut domain, &env, &mut decision);
    }
    assert_eq!(domain.forge.setting(3, 0, 1), Some(&charter(2)), "the stored default reached the forge");
    crate::restart::restore_connector_done(&mut domain);
    assert_eq!(domain.core.start_phase(), Some(Phase::RestoreConnector { connector: 0 }));
    let Some(Work::Core(Event::EffectConnector(connector::Event::SettingsDefaults {
        connector: 0,
        defaults,
        dropped: _,
    }))) = domain.work.pop()
    else {
        panic!("the core receives the defaults before the restore completes");
    };
    assert_eq!(
        defaults.as_ref(),
        &[
            connector::DefaultState {
                kind: 1,
                scope: 0,
                number: RULE,
                size: RULE_BYTES,
                charters: Box::default(),
                differs: false
            },
            connector::DefaultState {
                kind: 3,
                scope: 0,
                number: 1,
                size: CHARTER_BYTES,
                charters: Box::new([1]),
                differs: true
            },
        ]
    );
    let Some(Work::StartDone(Phase::RestoreConnector { connector: 0 })) = domain.work.pop() else {
        panic!("the forge restore completes after its defaults");
    };
    assert!(domain.work.is_empty());
    assert!(domain.kinds_stock.is_none(), "defaults are given once at this start");
    assert_eq!(domain.forge.setting(3, 0, 1), Some(&charter(1)));
    crate::domain::close(&mut domain, &env, decision, &mut Queue::uncapped());
}

#[test]
fn save_settings_defaults_writes_the_forges_rows() {
    let (mut domain, env) = root();
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    ask(&mut domain, &env, &mut decision, connector::Ask::SaveSettingsDefaults { revision: Some(2) });
    crate::route::route_into(&mut domain, &env, &mut decision);
    let writes = committed_writes(&mut domain, &env, decision);
    for (kind, number, value) in [(1, RULE, rule(true)), (3, 1, charter(2)), (3, 2, charter(4))] {
        assert!(writes.contains(&Write::Save(Record::Forge {
            row: Box::new(forge::Stored::Setting(settings::SettingRow {
                key: settings::SettingKey { kind, scope: 0, number, slot: settings::SettingSlot::Default },
                edit: settings::Edit::Set(value),
            })),
        })));
    }
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    ask(&mut domain, &env, &mut decision, connector::Ask::SaveSettingsDefaults { revision: None });
    assert!(domain.work.is_empty(), "matching defaults need no further rows, even without a revision");
    crate::domain::close(&mut domain, &env, decision, &mut Queue::uncapped());
}

#[test]
fn no_kinds_declared_gives_the_forge_defaults_without_a_core_event() {
    use jig_domain_core::start::Phase;
    let (mut domain, _) = at_connector_restore(false);
    assert!(domain.kinds_stock.is_none(), "undeclared stock goes to the forge during construction");
    crate::restart::restore_connector_done(&mut domain);
    let Some(Work::StartDone(Phase::RestoreConnector { connector: 0 })) = domain.work.pop() else {
        panic!("the forge restore completes after its defaults");
    };
    assert!(domain.work.is_empty(), "no SettingsDefaults precedes the restore completion");
    assert_eq!(domain.forge.setting(1, 0, RULE), Some(&rule(true)));
}

#[test]
fn the_rules_default_carries_the_standing_requirements() {
    let limits = resume_fixture::limits();
    let mut config = resume_fixture::config(7);
    let base = config.core.defaults.rules.requirements.clone();
    // Supplying one standing requirement already must not duplicate it.
    let standing = crate::route::policy_translate::standing_requirements(0);
    let mut supplied = List::uncapped();
    for requirement in &base {
        supplied.push(requirement.clone()).unwrap();
    }
    supplied.push(standing[0].clone()).unwrap();
    config.core.defaults.rules.requirements = supplied.into_boxed();
    let (domain, _) = resume_fixture::root_with_config_and_limits(config, &limits);
    let requirements = &domain.core.current.rules().requirements;
    assert_eq!(requirements.len(), base.len() + 2);
    for (number, guard, must_be_guarded) in [
        (5, authority::Guard::Guarded, true),
        (6, authority::Guard::Observed { freshness: skein_lib::Duration::ZERO }, false),
    ] {
        let mut count: u32 = 0;
        let mut found = None;
        for requirement in requirements {
            if requirement.judge.requirement == number {
                count += 1;
                found = Some(requirement);
            }
        }
        assert_eq!(count, 1);
        let requirement = found.unwrap();
        assert_eq!(requirement.connector, 0);
        assert_eq!(requirement.kind, 4);
        assert_eq!(requirement.pattern.segments.as_ref(), &[Box::from(b"forge".as_slice())]);
        assert_eq!(requirement.pattern.last, authority::Last::Open(Box::new([])));
        assert_eq!(requirement.judge, authority::Judge { connector: 0, requirement: number, parameters: 0 });
        assert_eq!(requirement.guard, guard);
        assert_eq!(requirement.must_be_guarded, must_be_guarded);
    }
}

#[test]
fn a_worlds_rule_reaches_the_forge_as_a_default() {
    let limits = resume_fixture::limits();
    let mut config = resume_fixture::config(7);
    let deployment = Box::new([crate::LandingRule {
        sets: Box::new([]),
        connector: 0,
        kind: 4,
        pattern: authority::Pattern {
            segments: Box::new([Box::from(b"forge".as_slice())]),
            last: authority::Last::Open(Box::new([])),
        },
        enforce: true,
        ci: false,
        up_to_date: true,
        gates: Box::new([crate::Gate { number: 1, blocking: true, freshness: crate::Freshness::Clean }]),
        approvals: Box::new([crate::Approval { role: 0, people: 1, freshness: crate::Freshness::Exact }]),
    }]);
    let mut projects = skein_lib::Map::with_capacity(1);
    let mut project = deployment.clone();
    project[0].sets = Box::new([3, 1]);
    projects.insert(PROJECT, project.as_slice().into()).unwrap();
    config.kinds_stock = crate::landing_stock(deployment.as_slice(), &projects, 1, 4);
    for item in &mut config.kinds_stock {
        if item.kind == 3 {
            item.value.size = CHARTER_BYTES;
        }
    }
    // Restored defaults must not replace this start's world defaults.
    let records = [Record::Forge {
        row: Box::new(forge::Stored::Setting(settings::SettingRow {
            key: settings::SettingKey { kind: 1, scope: 0, number: 1, slot: settings::SettingSlot::Default },
            edit: settings::Edit::Set(rule(true)),
        })),
    }];
    let (domain, _) = resume_fixture::root_with_records(config, &limits, &records);
    let expected = settings::SizedValue {
        size: 0,
        value: settings::Value::Rule(settings::LandingRule {
            sets: Box::new([]),
            pattern: forge::GatePattern {
                segments: Box::new([Box::from(b"forge".as_slice())]),
                last: forge::GateLast::Open(Box::new([])),
            },
            enforce: true,
            ci: false,
            up_to_date: true,
            gates: Box::new([settings::RuleGate {
                number: 1,
                blocking: true,
                freshness: forge::JudgeFreshness::Clean,
            }]),
            approvals: Box::new([settings::RuleApproval {
                role: 0,
                people: 1,
                freshness: forge::JudgeFreshness::Exact,
            }]),
        }),
    };
    assert_eq!(domain.forge.setting(1, 0, 1), Some(&expected));
    let settings::Value::Rule(mut project) = expected.value.clone() else { panic!("rule fixture") };
    project.sets = Box::new([3, 1]);
    let expected_project = settings::SizedValue { size: 0, value: settings::Value::Rule(project) };
    assert_eq!(domain.forge.setting(2, PROJECT, 1), Some(&expected_project));
    assert_eq!(
        domain.forge.setting(3, 0, 1),
        Some(&settings::SizedValue { size: CHARTER_BYTES, value: settings::Value::Charter(1) })
    );
    assert_eq!(
        domain.forge.setting(3, 0, 2),
        Some(&settings::SizedValue { size: CHARTER_BYTES, value: settings::Value::Charter(4) })
    );
}

const READ_SIGN_IN: u64 = 10;
const READ_REPLY: skein_lib::ReplyTo = skein_lib::ReplyTo::new(Token::new(93));
fn reader(role: Option<jig_domain_people::Role>) -> (Domain, Env<RootLimits>) {
    reader_in(role, jig_domain_people::DEPLOYMENT)
}
fn reader_in(role: Option<jig_domain_people::Role>, project: u32) -> (Domain, Env<RootLimits>) {
    use jig_domain_people as people;
    let (mut domain, env) = root();
    domain.core.people = people::Domain::new(&env.limits.people, Box::new([]), 1, Box::new([]));
    domain.core.people.start_restore(env.wall);
    let mut out = Queue::uncapped();
    let people_env = env.with(env.limits.people);
    for record in [
        people::Stored::Person {
            number: 1,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: b"reader".as_slice().into() },
                login: Box::new([]),
                name: Box::new([]),
            },
        },
        people::Stored::SignIn { number: READ_SIGN_IN, person: 1, expires: Wall::from_nanos(100) },
        people::Stored::Roles {
            project,
            holdings: match role {
                Some(role) => Box::new([people::Holding { person: 1, role }]),
                None => Box::new([]),
            },
        },
    ] {
        people::step(&mut domain.core.people, &people_env, people::Event::Restore { record }, &mut out);
    }
    assert!(domain.core.people.start_link());
    domain.core.people.start_sweep();
    assert!(domain.core.people.start_complete(&people_env));
    (domain, env)
}
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the fixture handles store acknowledgements and one party read answer"
)]
fn read_forge(
    domain: &mut Domain,
    env: &Env<RootLimits>,
    sign_in: u64,
    scope: jig_domain_people::ApplicationScope,
    query: crate::ForgeRead,
) -> Delivery {
    crate::try_step(
        domain,
        env,
        crate::Event::Party(crate::PartyInput::ForgeRead { reply: READ_REPLY, sign_in, scope, query }),
    )
    .unwrap();
    let mut out = Queue::uncapped();
    for _ in 0_u32..32 {
        crate::release(domain, env, &mut out);
        while let Some(request) = out.pop() {
            match request {
                crate::Request::Party(crate::PartyRequest::Deliver(delivery)) => return delivery,
                crate::Request::Store(crate::StoreRequest::Read { load, .. }) => {
                    crate::try_step(
                        domain,
                        env,
                        crate::Event::Store(crate::StoreInput::ReadLoaded {
                            load,
                            result: jig_domain_core::reads::Loaded::Page { rows: Box::new([]), next: None },
                        }),
                    )
                    .unwrap();
                }
                crate::Request::Store(crate::StoreRequest::Commit { number, .. }) => {
                    crate::try_step(domain, env, crate::Event::Store(crate::StoreInput::Committed { number, fill: 0 }))
                        .unwrap();
                }
                crate::Request::ToChild(released) => {
                    crate::try_step(domain, env, crate::Event::Released(released)).unwrap();
                }
                other => panic!("unexpected read output: {other:?}"),
            }
        }
    }
    panic!("one read answer");
}
#[test]
fn forge_setting_reads_follow_kept_removed_and_reset_values() {
    use jig_domain_people::{ApplicationScope, Role};
    let (mut domain, env) = reader(Some(Role::Observer));
    for (kind, default, changed) in [
        (1, rule(true), rule(false)),
        (
            11,
            settings::SizedValue {
                value: settings::Value::Goal(settings::TemplateGoal { design: false, close_out: false }),
                size: 2,
            },
            settings::SizedValue {
                value: settings::Value::Goal(settings::TemplateGoal { design: true, close_out: true }),
                size: 2,
            },
        ),
    ] {
        domain.forge.give_settings_defaults(Box::new([settings::Stock {
            kind,
            scope: 0,
            number: RULE,
            value: default.clone(),
        }]));
        let query = crate::ForgeRead::Setting { kind, scope: 0, number: RULE };
        for edit in [settings::Edit::Set(changed), settings::Edit::Remove, settings::Edit::Reset] {
            let mut out = Queue::uncapped();
            domain.forge.keep_settings(
                &crate::translate::environment_forge(&env),
                2,
                None,
                &[settings::Asked { kind, scope: 0, number: RULE, edit: edit.clone() }],
                &mut out,
            );
            let expected = match edit {
                settings::Edit::Set(value) => Some(value),
                settings::Edit::Remove => None,
                settings::Edit::Reset => Some(default.clone()),
            };
            assert_eq!(
                read_forge(&mut domain, &env, READ_SIGN_IN, ApplicationScope::Deployment, query.clone()),
                Delivery::ForgeRead { reply: READ_REPLY, answer: crate::ForgeReadAnswer::Setting(expected) }
            );
        }
        assert_eq!(
            read_forge(
                &mut domain,
                &env,
                READ_SIGN_IN,
                ApplicationScope::Deployment,
                crate::ForgeRead::Setting { kind, scope: 0, number: RULE + 1 }
            ),
            Delivery::ForgeRead { reply: READ_REPLY, answer: crate::ForgeReadAnswer::Setting(None) }
        );
    }
}

#[test]
fn forge_proposal_reads_use_the_deployment_unique_key_and_preserve_edits() {
    use jig_domain_people::{ApplicationScope, Role};
    let (mut domain, env) = reader(Some(Role::Member));
    let items = [
        settings::Asked { kind: 2, scope: PROJECT, number: RULE, edit: settings::Edit::Set(rule(false)) },
        settings::Asked { kind: 1, scope: 0, number: RULE, edit: settings::Edit::Remove },
        settings::Asked { kind: 3, scope: 0, number: 1, edit: settings::Edit::Reset },
    ];
    domain.forge.keep_proposal_settings(PROPOSAL, &items, &mut Queue::uncapped());
    domain.forge.keep_proposal_settings(
        WITHDRAWN_PROPOSAL,
        &[settings::Asked { kind: 3, scope: 0, number: 2, edit: settings::Edit::Set(charter(6)) }],
        &mut Queue::uncapped(),
    );
    let expected = domain.forge.proposal_settings(PROPOSAL);
    assert_eq!(expected.len(), 3);
    assert_eq!(
        read_forge(
            &mut domain,
            &env,
            READ_SIGN_IN,
            ApplicationScope::Deployment,
            crate::ForgeRead::ProposalItems { key: PROPOSAL }
        ),
        Delivery::ForgeRead {
            reply: READ_REPLY,
            answer: crate::ForgeReadAnswer::ProposalItems { items: expected, owned_approval: false }
        }
    );
    assert_eq!(
        read_forge(
            &mut domain,
            &env,
            READ_SIGN_IN,
            ApplicationScope::Deployment,
            crate::ForgeRead::ProposalItems { key: PROPOSAL + 10 }
        ),
        Delivery::ForgeRead {
            reply: READ_REPLY,
            answer: crate::ForgeReadAnswer::ProposalItems { items: Box::new([]), owned_approval: false }
        }
    );
}
#[test]
fn a_proposals_items_say_whether_an_approval_covers_an_owned_forgejo() {
    use forge::Mode::{Connected, Owned};
    use jig_domain_people::{ApplicationScope, Role};
    // Kind, scope, mode, enforcing, approval, coverage, branch pattern, set, writable, expected.
    for (kind, scope, mode, enforce, approval, covers, branch, set, writable, expected) in [
        (2, PROJECT, Owned, true, true, true, false, true, true, true),
        (2, PROJECT, Connected, true, true, true, false, true, true, false),
        (2, PROJECT, Owned, false, true, true, false, true, true, false),
        (2, PROJECT, Owned, true, false, true, false, true, true, false),
        (2, PROJECT, Owned, true, true, false, false, true, true, false),
        (2, PROJECT, Owned, true, true, true, false, false, true, false),
        (2, PROJECT, Owned, true, true, true, false, true, false, false),
        (2, PROJECT + 1, Owned, true, true, true, false, true, true, false),
        (1, 0, Owned, true, true, true, false, true, true, true),
        (2, PROJECT, Owned, true, true, true, true, true, true, true),
    ] {
        let (reader, _) = reader(Some(Role::Member));
        let mut config = resume_fixture::config(7);
        config.forges.first_mut().unwrap().mode = mode;
        let repository = forge::Repository {
            contexts: Box::new([]),
            workflows: Box::default(),
            project: PROJECT,
            home: true,
            provider: skein_forge::api::Repository { forge: 1, repository: 1 },
            host: b"forge.example".as_slice().into(),
            owner: b"org".as_slice().into(),
            name: b"repo".as_slice().into(),
            prefix: Box::new([]),
            role: forge::Role::Owned,
            ci: false,
            checks: Box::new([]),
            kinds: forge::Kinds {
                read: true,
                push: writable,
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
                default_branch: b"main".as_slice().into(),
                merge: true,
                squash: false,
                rebase: false,
            },
        };
        let (mut domain, env) = resume_fixture::root_with_config(config);
        domain.core.people = reader.core.people;
        domain.forge.restore_begin(env.wall);
        forge::step(
            &mut domain.forge,
            &env.with(env.limits.forge),
            forge::Event::Restore { record: forge::Stored::Repository(repository) },
            &mut Queue::uncapped(),
        );
        let mut value = rule(false);
        let settings::Value::Rule(rule) = &mut value.value else { panic!("landing rule") };
        rule.enforce = enforce;
        if approval {
            rule.approvals =
                Box::new([settings::RuleApproval { role: 1, people: 1, freshness: forge::JudgeFreshness::Exact }]);
        }
        if branch {
            rule.pattern.segments = Box::new([b"branch".as_slice().into()]);
        }
        if !covers {
            rule.pattern.segments = Box::new([Box::from(b"elsewhere".as_slice())]);
        }
        let edit = if set { settings::Edit::Set(value) } else { settings::Edit::Remove };
        domain.forge.keep_proposal_settings(
            PROPOSAL,
            &[settings::Asked { kind, scope, number: RULE, edit }],
            &mut Queue::uncapped(),
        );
        let Delivery::ForgeRead { answer: crate::ForgeReadAnswer::ProposalItems { items, owned_approval }, .. } =
            read_forge(
                &mut domain,
                &env,
                READ_SIGN_IN,
                ApplicationScope::Deployment,
                crate::ForgeRead::ProposalItems { key: PROPOSAL },
            )
        else {
            panic!("proposal items answer")
        };
        assert_eq!(items.len(), 1);
        assert_eq!(
            owned_approval, expected,
            "kind {kind}, scope {scope}, mode {mode:?}, enforce {enforce}, approval {approval}, covers {covers}, set {set}, writable {writable}"
        );
    }
}

#[test]
fn forge_reads_refuse_the_same_missing_membership_and_sign_in_as_settings() {
    use jig_domain_core::reads::{Answer, Why};
    use jig_domain_people::{ApplicationScope, Role};
    for query in
        [crate::ForgeRead::Setting { kind: 3, scope: 0, number: 1 }, crate::ForgeRead::ProposalItems { key: PROPOSAL }]
    {
        for (role, sign_in, scope, why) in [
            (None, READ_SIGN_IN, ApplicationScope::Deployment, Why::Unknown),
            (Some(Role::Owner), READ_SIGN_IN + 1, ApplicationScope::Deployment, Why::SignedOut),
            (Some(Role::Owner), READ_SIGN_IN, ApplicationScope::Own, Why::Unknown),
            (Some(Role::Owner), READ_SIGN_IN, ApplicationScope::Project(PROJECT), Why::Unknown),
        ] {
            let (mut domain, env) = reader(role);
            assert_eq!(
                read_forge(&mut domain, &env, sign_in, scope, query.clone()),
                Delivery::Read { reply: READ_REPLY, answer: Answer::None(why) }
            );
        }
    }
}

#[test]
fn forge_reads_wait_for_the_preceding_settings_commit() {
    use jig_domain_people::{ApplicationScope, Role};
    let (mut domain, env) = reader(Some(Role::Observer));
    domain.kinds = values(rule(false));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    ask(
        &mut domain,
        &env,
        &mut decision,
        connector::Ask::KeepSettings {
            revision: 2,
            proposal: None,
            items: Box::new([project_rule(application::Edit::Set(Box::default()))]),
        },
    );
    let mut out = Queue::uncapped();
    crate::domain::close(&mut domain, &env, decision, &mut out);
    crate::domain::send_commit(&mut domain, &mut out);
    let Some(crate::Request::Store(crate::StoreRequest::Commit { number, .. })) = out.pop() else {
        panic!("the settings change waits for its commit")
    };
    crate::try_step(
        &mut domain,
        &env,
        crate::Event::Party(crate::PartyInput::ForgeRead {
            reply: READ_REPLY,
            sign_in: READ_SIGN_IN,
            scope: ApplicationScope::Deployment,
            query: crate::ForgeRead::Setting { kind: 2, scope: PROJECT, number: RULE },
        }),
    )
    .unwrap();
    crate::release(&mut domain, &env, &mut out);
    assert!(out.is_empty(), "a read cannot escape before the settings commit");
    crate::try_step(&mut domain, &env, crate::Event::Store(crate::StoreInput::Committed { number, fill: 0 })).unwrap();
    crate::release(&mut domain, &env, &mut out);
    crate::release(&mut domain, &env, &mut out);
    assert_eq!(
        out.pop(),
        Some(crate::Request::Party(crate::PartyRequest::Deliver(Delivery::ForgeRead {
            reply: READ_REPLY,
            answer: crate::ForgeReadAnswer::Setting(Some(rule(false))),
        })))
    );
    assert!(out.is_empty());
}

fn ways(way: u32) -> settings::SizedValue {
    settings::SizedValue {
        size: 7,
        value: settings::Value::Ways(settings::ProjectWays { drafts: true, goals: false, changes: true, way }),
    }
}

fn root_with_way() -> (Domain, Env<RootLimits>) {
    use jig_domain_people as people;
    use jig_domain_tasks as tasks;
    let mut config = resume_fixture::config(7);
    config.core.defaults.templates = Box::new([jig_domain_core::Template {
        number: 1,
        name: b"way".as_slice().into(),
        executor: people::TemplateExecutor::Chosen,
        contract: tasks::Contract::Report { words: 32 },
        shallower: 1,
        budget: people::TemplateBudget::Given,
        deadline: people::TemplateDeadline::Given,
        tracked: None,
        flags: tasks::Flags {
            ended_by: tasks::EndedBy::Result,
            conversation: false,
            listed: true,
            retention: tasks::Retention::Kept,
        },
        label: 1,
        grace: None,
        llm: None,
        resources: Box::new([]),
    }]);
    resume_fixture::root_with_config_and_limits(config, &resume_fixture::limits())
}

#[test]
fn a_restore_refuses_a_way_the_project_does_not_offer() {
    let (mut domain, env) = root_with_way();
    let mut out = Queue::uncapped();
    domain.forge.keep_settings(
        &crate::translate::environment_forge(&env),
        2,
        None,
        &[settings::Asked { kind: 7, scope: PROJECT, number: 1, edit: settings::Edit::Set(ways(2)) }],
        &mut out,
    );
    domain.forge.keep_settings(
        &crate::translate::environment_forge(&env),
        3,
        None,
        &[settings::Asked { kind: 7, scope: PROJECT, number: 1, edit: settings::Edit::Remove }],
        &mut out,
    );
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    ask(
        &mut domain,
        &env,
        &mut decision,
        connector::Ask::RestoreSettings {
            owner: RESTORE_OWNER,
            proposal: PROPOSAL,
            items: Box::new([restored(7, PROJECT, 1, application::RestoredEdit::Set { revision: 3 })]),
        },
    );
    assert_eq!(
        restored_answer(&mut domain, RESTORE_OWNER).as_ref(),
        &[connector::Judged::Refuse { why: settings::Refusal::Way.words() }]
    );
    assert!(domain.forge.proposal_settings(PROPOSAL).is_empty());
    crate::domain::close(&mut domain, &env, decision, &mut Queue::uncapped());
}

#[test]
fn a_project_reads_its_own_kinds_with_a_role_there() {
    use jig_domain_core::reads::{Answer, Why};
    use jig_domain_people::{ApplicationScope, DEPLOYMENT, Role};
    for kind in [2, 6, 7, 8, 10, 12, 13, 14, 15] {
        for (role, membership, scope, query_scope, good) in [
            (Some(Role::Member), PROJECT, ApplicationScope::Project(PROJECT), PROJECT, true),
            (None, PROJECT, ApplicationScope::Project(PROJECT), PROJECT, false),
            (Some(Role::Member), PROJECT, ApplicationScope::Project(PROJECT + 1), PROJECT + 1, false),
            (Some(Role::Member), PROJECT, ApplicationScope::Project(PROJECT), PROJECT + 1, false),
            (Some(Role::Member), DEPLOYMENT, ApplicationScope::Project(PROJECT), PROJECT, false),
            (Some(Role::Observer), DEPLOYMENT, ApplicationScope::Deployment, PROJECT, true),
        ] {
            let (mut domain, env) = reader_in(role, membership);
            let value = match kind {
                2 => rule(false),
                6 => settings::SizedValue {
                    value: settings::Value::Questions(temper_engine_domain_forge_change::Route::Up),
                    size: 1,
                },
                7 => ways(0),
                13 => settings::SizedValue {
                    value: settings::Value::Dependency(settings::Dependency { project: 2 }),
                    size: 4,
                },
                12 => settings::SizedValue { value: settings::Value::Follow(follow_value()), size: 128 },
                8 => settings::project_stock(8).expect("project classes stock"),
                10 => settings::SizedValue {
                    value: settings::Value::ProjectWords(settings::ProjectWords {
                        headline: b"Our way".as_slice().into(),
                        lede: b"We review changes.".as_slice().into(),
                    }),
                    size: 33,
                },
                14 => settings::SizedValue {
                    value: settings::Value::ProjectAreas(settings::ProjectAreas { areas: Box::new([]) }),
                    size: 4,
                },
                15 => settings::SizedValue {
                    value: settings::Value::ProjectTriager(settings::ProjectTriager {
                        owner: settings::AreaOwner::Role(0),
                    }),
                    size: 5,
                },
                _ => unreachable!("table kind"),
            };
            domain.forge.keep_settings(
                &crate::translate::environment_forge(&env),
                2,
                None,
                &[settings::Asked { kind, scope: query_scope, number: 1, edit: settings::Edit::Set(value.clone()) }],
                &mut Queue::uncapped(),
            );
            for (query, read_value) in [
                (crate::ForgeRead::Setting { kind, scope: query_scope, number: 1 }, Some(value)),
                (crate::ForgeRead::RevisionSetting { kind, scope: query_scope, number: 1, revision: 1 }, None),
            ] {
                let answer = read_forge(&mut domain, &env, READ_SIGN_IN, scope, query);
                let expected = if good {
                    Delivery::ForgeRead { reply: READ_REPLY, answer: crate::ForgeReadAnswer::Setting(read_value) }
                } else {
                    Delivery::Read { reply: READ_REPLY, answer: Answer::None(Why::Unknown) }
                };
                assert_eq!(answer, expected, "kind {kind}, membership {membership}, scope {scope:?}");
            }
        }
    }
    for query in [
        crate::ForgeRead::Setting { kind: 1, scope: PROJECT, number: 1 },
        crate::ForgeRead::Setting { kind: 3, scope: PROJECT, number: 1 },
        crate::ForgeRead::Setting { kind: 11, scope: PROJECT, number: 1 },
        crate::ForgeRead::RevisionSetting { kind: 1, scope: PROJECT, number: 1, revision: 1 },
        crate::ForgeRead::RevisionSetting { kind: 3, scope: PROJECT, number: 1, revision: 1 },
        crate::ForgeRead::RevisionSetting { kind: 6, scope: PROJECT + 1, number: 1, revision: 1 },
        crate::ForgeRead::RevisionSetting { kind: 10, scope: PROJECT + 1, number: 1, revision: 1 },
        crate::ForgeRead::ProposalItems { key: PROPOSAL },
    ] {
        let (mut domain, env) = reader_in(Some(Role::Owner), PROJECT);
        assert_eq!(
            read_forge(&mut domain, &env, READ_SIGN_IN, ApplicationScope::Project(PROJECT), query),
            Delivery::Read { reply: READ_REPLY, answer: Answer::None(Why::Unknown) }
        );
    }
}

#[test]
fn only_the_owner_reads_the_program_environment() {
    use jig_domain_core::reads::{Answer, Why};
    use jig_domain_people::{ApplicationScope, Role};
    let (mut domain, env) = reader(Some(Role::Owner));
    assert_eq!(
        read_forge(&mut domain, &env, READ_SIGN_IN, ApplicationScope::Deployment, crate::ForgeRead::ProgramEnvironment),
        Delivery::ForgeRead { reply: READ_REPLY, answer: crate::ForgeReadAnswer::ProgramEnvironment }
    );
    for (role, sign_in, scope, why) in [
        (Some(Role::Observer), READ_SIGN_IN, ApplicationScope::Deployment, Why::Unknown),
        (Some(Role::Owner), READ_SIGN_IN + 1, ApplicationScope::Deployment, Why::SignedOut),
        (Some(Role::Owner), READ_SIGN_IN, ApplicationScope::Project(PROJECT), Why::Unknown),
    ] {
        let (mut domain, env) = reader(role);
        assert_eq!(
            read_forge(&mut domain, &env, sign_in, scope, crate::ForgeRead::ProgramEnvironment),
            Delivery::Read { reply: READ_REPLY, answer: Answer::None(why) }
        );
    }
}

fn follow_value() -> settings::Follow {
    settings::Follow {
        source: settings::FollowSource {
            repository: skein_forge::api::Repository { forge: 1, repository: 13 },
            branch: b"main".as_slice().into(),
            paths: Box::new([b"src".as_slice().into()]),
        },
        target: settings::FollowTarget {
            repository: skein_forge::api::Repository { forge: 1, repository: 12 },
            branch: b"stable".as_slice().into(),
        },
        owner: settings::FollowOwner::Person(1),
        start: settings::FollowStart::Owner,
        producer: None,
        check: settings::FollowCheck {
            workflow: b"tests.yml".as_slice().into(),
            artifact: b"results".as_slice().into(),
            format: temper_engine_domain_forge_tiers::Format::Junit,
        },
    }
}

fn follow_repository(number: u32, role: forge::Role, push: bool) -> forge::Repository {
    forge::Repository {
        project: 2,
        contexts: Box::new([]),
        home: false,
        provider: skein_forge::api::Repository { forge: 1, repository: number },
        host: b"forge.example".as_slice().into(),
        owner: b"org".as_slice().into(),
        name: b"repo".as_slice().into(),
        prefix: Box::new([]),
        role,
        ci: true,
        checks: Box::new([]),
        kinds: forge::Kinds {
            read: true,
            push,
            open: push,
            land: push,
            review: push,
            status: push,
            comment: push,
            issue: push,
            branch: push,
        },
        protection: forge::Protection::Absent,
        settings: skein_forge::api::Settings {
            default_branch: if role == forge::Role::Context {
                b"main".as_slice().into()
            } else {
                b"stable".as_slice().into()
            },
            merge: true,
            squash: false,
            rebase: false,
        },
        workflows: Box::new([]),
    }
}

fn follow_root() -> (Domain, Env<RootLimits>) {
    let (mut domain, env) = reader_in(Some(jig_domain_people::Role::Observer), 2);
    domain.forge.give_settings_defaults(Box::new([settings::Stock {
        kind: 3,
        scope: 0,
        number: 1,
        value: charter(1),
    }]));
    domain.forge.restore_begin(env.wall);
    for repository in
        [follow_repository(12, forge::Role::Owned, true), follow_repository(13, forge::Role::Context, false)]
    {
        forge::step(
            &mut domain.forge,
            &env.with(env.limits.forge),
            forge::Event::Restore { record: forge::Stored::Repository(repository) },
            &mut Queue::uncapped(),
        );
    }
    (domain, env)
}

fn judge_follow(
    domain: &mut Domain,
    env: &Env<RootLimits>,
    number: u32,
    follow: settings::Follow,
) -> connector::Judged {
    domain.kinds = Box::new([KindValue {
        kind: 12,
        scope: 2,
        number,
        value: settings::SizedValue { value: settings::Value::Follow(follow), size: 128 },
    }]);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    judge(
        domain,
        env,
        &mut decision,
        None,
        Box::new([application::Item { kind: 12, scope: 2, number, edit: application::Edit::Set(Box::new([])) }]),
    );
    let answer = answer(domain)[0].clone();
    crate::domain::close(domain, env, decision, &mut Queue::uncapped());
    answer
}

#[test]
fn a_follow_refusal_names_the_follow_and_the_failing_member() {
    let base = follow_value();
    // Each change reaches one refusal after the forge's structural judgement.
    for (fault, expected) in [
        ("source", "follow 1: its source 1/12 is not adopted by project 2 as context"),
        ("target", "follow 1: its target 1/13 is not a repository project 2 owns and temper writes"),
        ("readonly", "follow 1: its target 1/12 is not a repository project 2 owns and temper writes"),
        ("owner", "follow 1: its owner 99 is not a person of project 2"),
        ("deployment owner", "follow 1: its owner 1 is not a person of project 2"),
        ("paths", "follow 1: more paths than forge.follow_paths"),
        ("/src", "follow 1: a malformed path or one past 256 bytes"),
        ("src/", "follow 1: a malformed path or one past 256 bytes"),
        ("a//b", "follow 1: a malformed path or one past 256 bytes"),
        ("", "follow 1: a malformed path or one past 256 bytes"),
        ("producer", "follow 1: its producer does not spell change parameter 11"),
        ("charter", "follow 1: its producer's charter 99 is not offered by the settings"),
        ("workflow empty", "follow 1: a check with an empty member or one past 256 bytes"),
        ("workflow long", "follow 1: a check with an empty member or one past 256 bytes"),
        ("artifact empty", "follow 1: a check with an empty member or one past 256 bytes"),
        ("artifact long", "follow 1: a check with an empty member or one past 256 bytes"),
        ("fixed source", "follow 1: its source and target are fixed; remove it and add another"),
        ("fixed target", "follow 1: its source and target are fixed; remove it and add another"),
        ("zero", "follow 0: a number this kind does not have"),
        ("past", "follow 3: a number this kind does not have"),
    ] {
        let (mut domain, env) = follow_root();
        let mut value = base.clone();
        let mut number = 1;
        match fault {
            "source" => value.source.repository = value.target.repository,
            "target" => value.target.repository = value.source.repository,
            "readonly" => forge::step(
                &mut domain.forge,
                &env.with(env.limits.forge),
                forge::Event::Restore {
                    record: forge::Stored::Repository(follow_repository(12, forge::Role::Owned, false)),
                },
                &mut Queue::uncapped(),
            ),
            "owner" => value.owner = settings::FollowOwner::Person(99),
            "deployment owner" => {
                domain.core.people =
                    reader_in(Some(jig_domain_people::Role::Owner), jig_domain_people::DEPLOYMENT).0.core.people;
            }
            "paths" => {
                let mut paths = List::uncapped();
                for _ in 0..=env.limits.forge.follow_paths {
                    paths.push(b"src".as_slice().into()).unwrap();
                }
                value.source.paths = paths.into_boxed();
            }
            "/src" | "src/" | "a//b" | "" => value.source.paths = Box::new([fault.as_bytes().into()]),
            "producer" => value.producer = Some(b"6;3.x;".as_slice().into()),
            "charter" => value.producer = Some(b"99;;".as_slice().into()),
            "workflow empty" => value.check.workflow = Box::new([]),
            "workflow long" => value.check.workflow = Box::new([b'w'; 257]),
            "artifact empty" => value.check.artifact = Box::new([]),
            "artifact long" => value.check.artifact = Box::new([b'a'; 257]),
            "fixed source" | "fixed target" => {
                let kept = settings::SizedValue { size: 128, value: settings::Value::Follow(base.clone()) };
                domain.forge.keep_settings(
                    &env.with(env.limits.forge),
                    2,
                    None,
                    &[settings::Asked { kind: 12, scope: 2, number: 1, edit: settings::Edit::Set(kept) }],
                    &mut Queue::uncapped(),
                );
                if fault == "fixed source" {
                    value.source.branch = b"other".as_slice().into();
                } else {
                    value.target.branch = b"other".as_slice().into();
                }
            }
            "zero" => number = 0,
            "past" => number = env.limits.forge.follows + 1,
            _ => unreachable!("fault table"),
        }
        assert_eq!(
            judge_follow(&mut domain, &env, number, value),
            connector::Judged::Refuse { why: expected.as_bytes().into() },
            "{fault}"
        );
    }
}

#[test]
fn follows_resolve_adopted_branches_and_allow_member_reads_but_not_outsiders() {
    use jig_domain_people::{ApplicationScope, Role};
    let (mut domain, env) = follow_root();
    let mut value = follow_value();
    value.source.branch = Box::new([]);
    value.target.branch = Box::new([]);
    value.producer = Some(Box::new([]));
    let expected = follow_value();
    let size = settings::follow_record(&expected).unwrap().measure();
    assert_eq!(
        judge_follow(&mut domain, &env, 1, value.clone()),
        connector::Judged::Accept { size, charters: Box::new([]) }
    );
    domain.kinds = Box::new([KindValue {
        kind: 12,
        scope: 2,
        number: 1,
        value: settings::SizedValue { value: settings::Value::Follow(value), size: 1 },
    }]);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    ask(
        &mut domain,
        &env,
        &mut decision,
        connector::Ask::KeepSettings {
            revision: 2,
            proposal: None,
            items: Box::new([application::Item {
                kind: 12,
                scope: 2,
                number: 1,
                edit: application::Edit::Set(Box::new([])),
            }]),
        },
    );
    committed_writes(&mut domain, &env, decision);
    assert_eq!(domain.forge.follow(2, 1), Some(&expected));
    assert_eq!(
        crate::route::see::address(b"settings follow 1", 2),
        crate::route::see::Address::Setting(jig_domain_core::settings::Setting::Application {
            kind: 12,
            scope: 2,
            number: 1
        })
    );
    for role in [Some(Role::Observer), Some(Role::Member), None] {
        let (mut reader, read_env) = reader_in(role, 2);
        reader.forge.keep_settings(
            &read_env.with(read_env.limits.forge),
            2,
            None,
            &[settings::Asked {
                kind: 12,
                scope: 2,
                number: 1,
                edit: settings::Edit::Set(settings::SizedValue {
                    value: settings::Value::Follow(expected.clone()),
                    size,
                }),
            }],
            &mut Queue::uncapped(),
        );
        let read = read_forge(
            &mut reader,
            &read_env,
            READ_SIGN_IN,
            ApplicationScope::Project(2),
            crate::ForgeRead::Setting { kind: 12, scope: 2, number: 1 },
        );
        if role.is_some() {
            assert_eq!(
                read,
                Delivery::ForgeRead {
                    reply: READ_REPLY,
                    answer: crate::ForgeReadAnswer::Setting(Some(settings::SizedValue {
                        value: settings::Value::Follow(expected.clone()),
                        size
                    }))
                }
            );
        } else {
            assert_eq!(
                read,
                Delivery::Read {
                    reply: READ_REPLY,
                    answer: jig_domain_core::reads::Answer::None(jig_domain_core::reads::Why::Unknown)
                }
            );
        }
    }
}

#[test]
fn restoring_a_retired_setting_names_it_and_keeps_current_settings() {
    for (kind, scope, number, words) in [
        (6, 2, 1, b"project-questions 2: retired stored setting".as_slice()),
        (3, 0, 3, b"forge-charter 3: retired stored setting".as_slice()),
    ] {
        let (mut domain, env) = root();
        domain.forge.restore_begin(env.wall);
        let key = settings::SettingKey { kind, scope, number, slot: settings::SettingSlot::Replaced(2) };
        let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
        crate::restart::restore_page_row(
            &mut domain,
            &env,
            Record::Forge { row: Box::new(forge::Stored::RefusedSetting(key)) },
        );
        crate::route::route_into(&mut domain, &env, &mut decision);
        let before = domain.forge.setting(3, 0, 1).cloned();
        let current = domain.core.current.newest();
        ask(
            &mut domain,
            &env,
            &mut decision,
            connector::Ask::RestoreSettings {
                owner: RESTORE_OWNER,
                proposal: PROPOSAL,
                items: Box::new([application::Restored {
                    kind,
                    scope,
                    number,
                    edit: application::RestoredEdit::Set { revision: 2 },
                }]),
            },
        );
        let Some(Work::Core(Event::EffectConnector(connector::Event::SettingsRestored { owner, judged }))) =
            domain.work.pop()
        else {
            panic!("restore judgement")
        };
        assert_eq!(owner, RESTORE_OWNER);
        assert_eq!(judged.as_ref(), &[connector::Judged::Refuse { why: words.into() }]);
        assert_eq!(domain.forge.setting(3, 0, 1).cloned(), before);
        assert_eq!(domain.core.current.newest(), current);
        assert!(domain.forge.proposal_settings(PROPOSAL).is_empty());
    }
}

#[test]
fn dependency_structural_refusals_name_the_dependency_and_restore_rechecks_the_target() {
    for (scope, number, project, reason) in [
        (0, 1, 2, "dependency 1: a scope this kind does not have"),
        (PROJECT, 0, 2, "dependency 0: a number this kind does not have"),
        (PROJECT, 5, 2, "dependency 5: a number this kind does not have"),
        (PROJECT, 1, 0, "dependency 1: a value of another kind"),
    ] {
        let (mut domain, env) = root();
        domain.kinds = Box::new([KindValue {
            kind: 13,
            scope,
            number,
            value: settings::SizedValue {
                value: settings::Value::Dependency(settings::Dependency { project }),
                size: 4,
            },
        }]);
        let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
        judge(
            &mut domain,
            &env,
            &mut decision,
            None,
            Box::new([application::Item { kind: 13, scope, number, edit: application::Edit::Set(Box::new([])) }]),
        );
        assert_eq!(answer(&mut domain).as_ref(), &[connector::Judged::Refuse { why: reason.as_bytes().into() }]);
        crate::domain::close(&mut domain, &env, decision, &mut Queue::uncapped());
    }
    let (mut domain, env) = root();
    let mut out = Queue::uncapped();
    let value =
        settings::SizedValue { value: settings::Value::Dependency(settings::Dependency { project: 99 }), size: 4 };
    domain.forge.keep_settings(
        &crate::translate::environment_forge(&env),
        2,
        None,
        &[settings::Asked { kind: 13, scope: PROJECT, number: 1, edit: settings::Edit::Set(value) }],
        &mut out,
    );
    domain.forge.keep_settings(
        &crate::translate::environment_forge(&env),
        3,
        None,
        &[settings::Asked { kind: 13, scope: PROJECT, number: 1, edit: settings::Edit::Remove }],
        &mut out,
    );
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    ask(
        &mut domain,
        &env,
        &mut decision,
        connector::Ask::RestoreSettings {
            owner: RESTORE_OWNER,
            proposal: PROPOSAL,
            items: Box::new([restored(13, PROJECT, 1, application::RestoredEdit::Set { revision: 3 })]),
        },
    );
    assert_eq!(
        restored_answer(&mut domain, RESTORE_OWNER).as_ref(),
        &[connector::Judged::Refuse {
            why: b"dependency 1: project 99 is not a project of this deployment".as_slice().into()
        }]
    );
    assert!(domain.forge.proposal_settings(PROPOSAL).is_empty());
    crate::domain::close(&mut domain, &env, decision, &mut Queue::uncapped());
}
