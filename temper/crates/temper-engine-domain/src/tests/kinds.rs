//! The root's typed settings routes (temper/docs/design/domain/root.md, 5.6;
//! temper/docs/design/protocol/configuration.md, 13.2).
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
// The typed fixture carries the encoding's entrance-measured size.
const RULE_BYTES: u32 = 32;
// A charter's encoded u32 occupies four bytes.
const CHARTER_BYTES: u32 = 4;

fn rule(ci: bool) -> settings::SizedValue {
    settings::SizedValue {
        size: RULE_BYTES,
        value: settings::Value::Rule(settings::LandingRule {
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
    assert_eq!(
        answer(&mut domain).as_ref(),
        &[connector::Judged::Refuse { why: settings::Refusal::Charter.words().into() }]
    );
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
    assert_eq!(
        answer(&mut domain).as_ref(),
        &[connector::Judged::Refuse { why: settings::Refusal::Value.words().into() }]
    );
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
    let Some(Work::Core(Event::EffectConnector(connector::Event::SettingsDefaults { connector: 0, defaults }))) =
        domain.work.pop()
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
fn a_document_rule_reaches_the_forge_as_a_default() {
    let limits = resume_fixture::limits();
    let mut config = resume_fixture::config(7);
    config.landing_rules.deployment = Box::new([crate::LandingRule {
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
    config.landing_rules.projects.insert(PROJECT, config.landing_rules.deployment.clone()).unwrap();
    for item in &mut config.kinds_stock {
        item.value.size = CHARTER_BYTES;
    }
    // Restored defaults must not replace this start's document defaults.
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
    assert_eq!(domain.forge.setting(2, PROJECT, 1), Some(&expected));
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
    use jig_domain_people as people;
    let (mut domain, env) = root();
    domain.core.people = people::Domain::new(&env.limits.people, Box::new([]), 1, Box::new([]));
    domain.core.people.start_restore(env.wall);
    let mut out = Queue::uncapped();
    let people_env = Env { now: env.now, wall: env.wall, limits: env.limits.people };
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
            project: people::DEPLOYMENT,
            holdings: match role {
                Some(role) => Box::new([people::Holding { person: 1, role }]),
                None => Box::new([]),
            },
        },
    ] {
        people::step(&mut domain.core.people, &people_env, people::Event::Restore { record }, &mut out);
    }
    assert!(domain.core.people.start_link(&people_env));
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
    let query = crate::ForgeRead::Setting { kind: 1, scope: 0, number: RULE };
    for edit in [settings::Edit::Set(rule(false)), settings::Edit::Remove, settings::Edit::Reset] {
        let mut out = Queue::uncapped();
        domain.forge.keep_settings(
            &crate::translate::environment_forge(&env),
            None,
            &[settings::Asked { kind: 1, scope: 0, number: RULE, edit: edit.clone() }],
            &mut out,
        );
        let expected = match edit {
            settings::Edit::Set(value) => Some(value),
            settings::Edit::Remove => None,
            settings::Edit::Reset => Some(rule(true)),
        };
        assert_eq!(
            read_forge(&mut domain, &env, READ_SIGN_IN, ApplicationScope::Deployment, query),
            Delivery::ForgeRead { reply: READ_REPLY, answer: crate::ForgeReadAnswer::Setting(expected) }
        );
    }
    assert_eq!(
        read_forge(
            &mut domain,
            &env,
            READ_SIGN_IN,
            ApplicationScope::Deployment,
            crate::ForgeRead::Setting { kind: 1, scope: 0, number: RULE + 1 }
        ),
        Delivery::ForgeRead { reply: READ_REPLY, answer: crate::ForgeReadAnswer::Setting(None) }
    );
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
        Delivery::ForgeRead { reply: READ_REPLY, answer: crate::ForgeReadAnswer::ProposalItems(expected) }
    );
    assert_eq!(
        read_forge(
            &mut domain,
            &env,
            READ_SIGN_IN,
            ApplicationScope::Deployment,
            crate::ForgeRead::ProposalItems { key: PROPOSAL + 10 }
        ),
        Delivery::ForgeRead { reply: READ_REPLY, answer: crate::ForgeReadAnswer::ProposalItems(Box::new([])) }
    );
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
                read_forge(&mut domain, &env, sign_in, scope, query),
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
