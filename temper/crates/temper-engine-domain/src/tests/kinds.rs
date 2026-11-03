//! The root's typed settings routes (temper/docs/design/domain/root.md, 5.6;
//! temper/docs/design/protocol/configuration.md, 13.2).
use super::*;
use crate::{Domain, KindValue, Limits as RootLimits, Work};
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
