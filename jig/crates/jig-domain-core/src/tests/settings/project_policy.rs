//! Whole project policies changed and proposed (jig/docs/design/domain/engine.md, 17.2, 17.6).
use crate::settings as s;
use crate::tests::*;

const OWNER: u64 = 1;
const OTHER: u64 = 2;
const SIGN_IN: u64 = 10;
const OTHER_SIGN_IN: u64 = 11;
const PROJECT: u32 = 1;
const POLICY: u32 = 1;
const PERIOD: u64 = 1;
const PERIOD_BUDGET: u64 = 1000;
const POOL_BUDGET: u64 = 500;
const NEW_PERIOD_BUDGET: u64 = 900;
const NEW_POOL_BUDGET: u64 = 400;
// One keyed answer and a revision, plus the two funding rows and proposal updates.
const OUTPUTS: u32 = 128;

fn project_core(project_owner: u64, deployment_owners: &[u64]) -> (Core, Limits) {
    project_core_with_limits(project_owner, deployment_owners, &limits())
}
fn project_core_with_limits(project_owner: u64, deployment_owners: &[u64], limits: &Limits) -> (Core, Limits) {
    let limits = *limits;
    let (owner, _) = core();
    let mut configured = config(41);
    configured.seed_projects[0].policy.permissions = Box::default();
    configured.defaults.policies = Map::with_capacity(1);
    configured.defaults.policies.insert(POLICY, configured.seed_projects[0].policy.clone()).unwrap();
    let mut core = seeded_core(configured, &limits).unwrap();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    let mut holdings = List::with_capacity(limits.people.holdings);
    for &person in deployment_owners {
        holdings.push(people::Holding { person, role: people::Role::Owner }).unwrap();
    }
    let mut other_identity = owner.people.identity(OWNER).unwrap().clone();
    other_identity.key.subject = b"other".as_slice().into();
    for record in [
        people::Stored::Person { number: OWNER, identity: owner.people.identity(OWNER).unwrap().clone() },
        people::Stored::Person { number: OTHER, identity: other_identity },
        people::Stored::SignIn {
            number: SIGN_IN,
            person: OWNER,
            expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()),
        },
        people::Stored::SignIn {
            number: OTHER_SIGN_IN,
            person: OTHER,
            expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()),
        },
        people::Stored::Roles { project: people::DEPLOYMENT, holdings: holdings.into_boxed() },
        people::Stored::Roles {
            project: PROJECT,
            holdings: Box::new([people::Holding { person: project_owner, role: people::Role::Owner }]),
        },
    ] {
        people::step(&mut core.people, &env, people::Event::Restore { record }, &mut Queue::uncapped());
    }
    finish_people(&mut core.people, &env);
    send_task(&mut core, &limits, tasks::Event::Check);
    send_task(&mut core, &limits, tasks::Event::Restored);
    send_task(
        &mut core,
        &limits,
        tasks::Event::OpenPeriod {
            reply_to: ReplyTo::new(Token::new(80)),
            project: PROJECT,
            period: PERIOD,
            budget: PERIOD_BUDGET,
        },
    );
    send_task(
        &mut core,
        &limits,
        tasks::Event::CarvePool {
            reply_to: ReplyTo::new(Token::new(81)),
            project: PROJECT,
            person: project_owner,
            period: PERIOD,
            budget: POOL_BUDGET,
        },
    );
    (core, limits)
}
fn fixture() -> (Core, Limits) {
    project_core(OWNER, &[OWNER, OTHER])
}
fn replacement(core: &Core) -> ConfiguredPolicy {
    let mut value = core.configured_policy(PROJECT).unwrap();
    value.policy.period_spend = NEW_PERIOD_BUDGET;
    value.policy.roles[0].period_spend = NEW_POOL_BUDGET;
    value
}
fn item(value: ConfiguredPolicy) -> s::Item {
    s::Item {
        setting: s::Setting::ProjectPolicy { project: PROJECT },
        edit: s::Edit::Set(s::Value::Policy(Box::new(value))),
    }
}
fn send(core: &mut Core, limits: &Limits, event: Event) -> Box<[Request]> {
    let mut out = step(core, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits }, event);
    let mut result = List::with_capacity(OUTPUTS);
    while let Some(request) = out.pop() {
        result.push(request).unwrap();
    }
    core.reclaim();
    result.into_boxed()
}
fn change(core: &mut Core, limits: &Limits, items: Box<[s::Item]>) -> Box<[Request]> {
    let against = core.current.newest();
    send(
        core,
        limits,
        Event::ChangeSettings {
            reply_to: ReplyTo::new(Token::new(91)),
            sign_in: SIGN_IN,
            key: [1; 16],
            digest: [1; 32],
            change: Box::new(s::Change { against, items, running: s::Running::Stays }),
        },
    )
}
fn propose(core: &mut Core, limits: &Limits, items: Box<[s::Item]>) -> Box<[Request]> {
    propose_as(core, limits, SIGN_IN, items)
}
fn propose_as(core: &mut Core, limits: &Limits, sign_in: u64, items: Box<[s::Item]>) -> Box<[Request]> {
    let against = core.current.newest();
    send(
        core,
        limits,
        Event::ProposeSettings {
            reply_to: ReplyTo::new(Token::new(92)),
            sign_in,
            key: [2; 16],
            digest: [2; 32],
            change: Box::new(s::Proposed { against, reason: b"review".as_slice().into(), items }),
        },
    )
}
fn ask(core: &mut Core, limits: &Limits, sign_in: u64, key: u8, ask: people::Ask) -> Box<[Request]> {
    send(
        core,
        limits,
        Event::People(people::Event::Ask {
            reply_to: ReplyTo::new(Token::new(93)),
            sign_in,
            key: [key; 16],
            models: Box::default(),
            ask,
        }),
    )
}
fn outcome(output: &[Request]) -> people::Outcome {
    for request in output {
        if let Request::Held(held) = request
            && let Held::PeopleReply { reply: people::Reply::Outcome(outcome), .. } = held.as_ref()
        {
            return outcome.clone();
        }
    }
    panic!("keyed outcome: {output:?}")
}
fn proposal(output: &[Request]) -> u64 {
    let people::Outcome::Proposed { proposal, .. } = outcome(output) else { panic!("proposal: {output:?}") };
    proposal
}
fn decision(number: u64) -> people::Ask {
    people::Ask::DecideProposal {
        project: people::DEPLOYMENT,
        proposer: Proposer::Party(OWNER),
        proposal: number,
        decision: people::ProposalDecision::Accept,
        running: Some(people::Running::Stays),
    }
}
fn malformed(setting: s::Setting, fault: s::Fault) -> people::Outcome {
    people::Outcome::Refused(people::Refusal::NotWellFormed {
        at: 0,
        why: people::Malformed::Settings { setting, fault },
    })
}
fn writes(requests: &[Request]) -> u32 {
    let mut count = 0_u32;
    for request in requests {
        if let Request::Write(_) = request {
            count = count.checked_add(1).unwrap();
        }
    }
    count
}

#[test]
fn an_owner_of_both_sets_charters_and_a_project_policy_in_one_change() {
    let (mut core, limits) = fixture();
    let value = replacement(&core);
    let mut charter = core.current.charter(1).unwrap().clone();
    charter.policy.instructions = b"new instructions".as_slice().into();
    let items = Box::new([
        s::Item { setting: s::Setting::Charter(1), edit: s::Edit::Set(s::Value::Charter(Box::new(charter.clone()))) },
        item(value.clone()),
    ]);
    let output = change(&mut core, &limits, items);
    assert_eq!(outcome(&output), people::Outcome::SettingsChanged { revision: 2 });
    assert_eq!(core.current.charter(1), Some(&charter));
    assert_eq!(core.configured_policy(PROJECT), Some(value.clone()));
    assert_eq!(core.authority.policy(PROJECT), Some(&value.policy));
    assert_eq!(
        core.tasks.funding(tasks::Funder::Period { project: PROJECT, period: PERIOD }).unwrap().numbers.budget,
        NEW_PERIOD_BUDGET
    );
    assert_eq!(
        core.tasks
            .funding(tasks::Funder::Pool { project: PROJECT, person: OWNER, period: PERIOD })
            .unwrap()
            .numbers
            .budget,
        NEW_POOL_BUDGET
    );
    assert_eq!(core.current.value(s::Setting::ProjectPolicy { project: PROJECT }), None);
    assert_eq!(core.current.changes.len(), 1);
    let mut saved = false;
    for request in &output {
        if let Request::Write(Write::Save(Record::People(people::Stored::Policy { project: PROJECT, .. }))) = request {
            saved = true;
        }
    }
    assert!(saved);
}

#[test]
fn a_project_policy_change_records_the_policy_it_replaced() {
    let (mut core, limits) = fixture();
    let before = core.configured_policy(PROJECT).unwrap();
    let value = replacement(&core);
    let output = change(&mut core, &limits, Box::new([item(value.clone())]));
    let mut saved = None;
    for request in &output {
        if let Request::Write(Write::Save(Record::Core(CoreRecord::Settings(s::Record::Revision {
            history: false,
            revision,
        })))) = request
        {
            saved = Some(revision);
        }
    }
    let revision = saved.expect("revision saved");
    assert_eq!(
        revision.items.as_ref(),
        &[s::RevisionItem {
            setting: s::Setting::ProjectPolicy { project: PROJECT },
            edit: s::Edit::Set(s::Value::Policy(Box::new(value))),
            replaced: s::Replaced::Value(s::Value::Policy(Box::new(before)))
        }]
    );
    assert!(core.current.changes.is_empty());
    assert!(core.current.applications.is_empty());
}

#[test]
fn a_deployment_only_owner_is_refused_a_project_policy_item() {
    let (mut core, limits) = project_core(OTHER, &[OWNER]);
    let value = replacement(&core);
    let output = change(&mut core, &limits, Box::new([item(value)]));
    assert_eq!(outcome(&output), people::Outcome::Refused(people::Refusal::Role { held: None }));
    assert_eq!(core.current.newest(), 1);
}

#[test]
fn a_project_policy_removed_reset_or_twice_is_refused_its_shape() {
    for edit in [s::Edit::Removed, s::Edit::Reset] {
        let (mut core, limits) = fixture();
        let setting = s::Setting::ProjectPolicy { project: PROJECT };
        let output = change(&mut core, &limits, Box::new([s::Item { setting, edit }]));
        assert_eq!(outcome(&output), malformed(setting, s::Fault::Shape));
    }
    let (mut core, limits) = fixture();
    let item = item(replacement(&core));
    let output = change(&mut core, &limits, Box::new([item.clone(), item.clone()]));
    assert_eq!(outcome(&output), malformed(item.setting, s::Fault::Shape));
}

#[test]
fn a_project_policy_past_its_bounds_is_refused_as_an_owners_edit_is() {
    let (mut core, limits) = fixture();
    let mut value = replacement(&core);
    value.policy.period_spend = PERIOD_BUDGET + 1;
    let output = change(&mut core, &limits, Box::new([item(value)]));
    assert_eq!(outcome(&output), malformed(s::Setting::ProjectPolicy { project: PROJECT }, s::Fault::Rules));
    assert_eq!(core.authority.policy(PROJECT).unwrap().period_spend, PERIOD_BUDGET);
}

#[test]
fn the_same_project_policy_alone_is_empty() {
    let (mut core, limits) = fixture();
    let value = core.configured_policy(PROJECT).unwrap();
    let output = change(&mut core, &limits, Box::new([item(value)]));
    assert_eq!(
        outcome(&output),
        people::Outcome::Refused(people::Refusal::NotWellFormed { at: 0, why: people::Malformed::SettingsUnchanged })
    );
    assert_eq!(core.current.newest(), 1);
}

#[test]
fn a_proposal_of_a_project_policy_no_party_owns_both_of_is_refused_at_entrance() {
    let (mut core, limits) = project_core(OTHER, &[OWNER]);
    let value = replacement(&core);
    let output = propose(&mut core, &limits, Box::new([item(value)]));
    assert_eq!(
        outcome(&output),
        people::Outcome::Refused(people::Refusal::Authority {
            lacking: people::Lack::Approver,
            proposable: Some(people::Accepter::Nobody)
        })
    );
    assert!(core.settings_proposals.is_empty());
}

#[test]
fn a_proposal_of_a_project_policy_is_accepted_only_by_an_owner_of_both() {
    let (mut core, limits) = fixture();
    let value = replacement(&core);
    let number = proposal(&propose(&mut core, &limits, Box::new([item(value.clone())])));
    let refused = ask(&mut core, &limits, OTHER_SIGN_IN, 3, decision(number));
    assert_eq!(outcome(&refused), people::Outcome::Refused(people::Refusal::Role { held: None }));
    assert_eq!(core.settings_proposals.len(), 1);
    let accepted = ask(&mut core, &limits, SIGN_IN, 4, decision(number));
    let people::Outcome::ProposalDecided { proposer, proposal, .. } = outcome(&accepted) else {
        panic!("accepted proposal")
    };
    assert_eq!((proposer, proposal), (Proposer::Party(OWNER), number));
    assert_eq!(core.current.head.proposal, Some((Proposer::Party(OWNER), u32::try_from(number).unwrap())));
    assert_eq!(core.configured_policy(PROJECT), Some(value));
    assert!(core.settings_proposals.is_empty());
}

#[test]
fn deleting_a_project_withdraws_a_proposal_naming_its_policy() {
    let (mut core, limits) = fixture();
    let value = replacement(&core);
    let number = proposal(&propose(&mut core, &limits, Box::new([item(value)])));
    let output = ask(&mut core, &limits, SIGN_IN, 3, people::Ask::RemoveProject { project: PROJECT, grace: None });
    assert_eq!(outcome(&output), people::Outcome::ProjectRemoved { project: PROJECT });
    assert!(core.settings_proposals.is_empty());
    assert!(core.tasks.person_proposal(OWNER, number).is_none());
    let mut withdrawn = false;
    let mut erased = false;
    let mut notified = false;
    for request in &output {
        if let Request::Write(Write::Save(Record::Tasks(tasks::Stored::DecidedProposal(row)))) = request {
            assert_eq!(row.number, number);
            assert_eq!(row.state, tasks::PersonProposalState::Withdrawn { at: Wall::EPOCH });
            withdrawn = true;
        }
        if let Request::Write(Write::Erase(Key::Core(CoreKey::Settings(s::Key::Proposal { proposal, .. })))) = request {
            assert_eq!(u64::from(*proposal), number);
            erased = true;
        }
        if let Request::Write(Write::Save(Record::Tasks(tasks::Stored::Update(update)))) = request {
            assert_eq!(update.party, OWNER);
            notified = true;
        }
    }
    assert!(withdrawn && erased && notified);
}

#[test]
fn a_deletion_withdraws_its_policy_proposals_within_the_withdrawal_allowance() {
    // Both fixture parties propose, filling the pending-set allowance.
    const PROPOSALS: u32 = 2;
    let mut limits = limits();
    limits.settings.proposals = PROPOSALS;
    let (mut core, limits) = project_core_with_limits(OWNER, &[OWNER, OTHER], &limits);
    let value = replacement(&core);
    let mut numbers = List::with_capacity(PROPOSALS);
    for sign_in in [SIGN_IN, OTHER_SIGN_IN] {
        let output = propose_as(&mut core, &limits, sign_in, Box::new([item(value.clone())]));
        numbers.push(proposal(&output)).unwrap();
    }
    assert_eq!(core.settings_proposals.len(), limits.settings.proposals);

    // Subtract the same deletion without pending sets, counting every added write.
    let (mut baseline, _) = project_core_with_limits(OWNER, &[OWNER, OTHER], &limits);
    let deletion = people::Ask::RemoveProject { project: PROJECT, grace: None };
    let ordinary = ask(&mut baseline, &limits, SIGN_IN, 3, deletion.clone());
    let output = ask(&mut core, &limits, SIGN_IN, 3, deletion);
    let withdrawal_writes = writes(&output) - writes(&ordinary);
    assert_eq!(outcome(&output), people::Outcome::ProjectRemoved { project: PROJECT });
    assert!(core.settings_proposals.is_empty());
    for (person, number) in [OWNER, OTHER].into_iter().zip(numbers.into_boxed()) {
        assert!(core.tasks.person_proposal(person, number).is_none());
    }
    let mut withdrawn = 0;
    for request in &output {
        if let Request::Write(Write::Save(Record::Tasks(tasks::Stored::DecidedProposal(row)))) = request {
            assert_eq!(row.state, tasks::PersonProposalState::Withdrawn { at: Wall::EPOCH });
            withdrawn += 1;
        }
    }
    assert_eq!(withdrawn, PROPOSALS);
    assert!(withdrawal_writes <= s::withdrawal_writes(&limits).unwrap());
}

#[test]
fn a_review_shows_the_project_policy_now_and_as_it_would_be() {
    let (mut core, limits) = fixture();
    let value = replacement(&core);
    let number = proposal(&propose(&mut core, &limits, Box::new([item(value.clone())])));
    let mut changed = core.authority.policy(PROJECT).unwrap().clone();
    changed.period_spend = NEW_PERIOD_BUDGET;
    authority::step(
        &mut core.authority,
        authority::Event::Policy { project: PROJECT, policy: changed },
        &mut Queue::uncapped(),
    );
    let document = make(
        &core,
        &limits,
        OWNER,
        &Subject::Proposal { proposer: Proposer::Party(OWNER), number },
        &window(),
        Wall::EPOCH,
    )
    .unwrap();
    let Document::Proposal(document) = document else { panic!("proposal document") };
    let ProposalAction::Settings { review: Some(review), .. } = document.action else { panic!("settings review") };
    assert_eq!(review.items.len(), 1);
    assert_eq!(
        review.items.get(0).unwrap().now,
        Some(s::Value::Policy(Box::new(core.configured_policy(PROJECT).unwrap())))
    );
    assert_eq!(review.items.get(0).unwrap().would_be, Some(s::Value::Policy(Box::new(value))));
    assert_eq!(review.items.get(0).unwrap().origin, None);
}

#[test]
fn project_zero_and_unknown_projects_keep_their_distinct_refusals() {
    for (project, expected) in [
        (people::DEPLOYMENT, malformed(s::Setting::ProjectPolicy { project: people::DEPLOYMENT }, s::Fault::Shape)),
        (PROJECT + 1, people::Outcome::Refused(people::Refusal::Unknown)),
    ] {
        let (mut core, limits) = fixture();
        let mut item = item(replacement(&core));
        item.setting = s::Setting::ProjectPolicy { project };
        assert_eq!(outcome(&change(&mut core, &limits, Box::new([item]))), expected);
        assert_eq!(core.current.newest(), 1);
    }
}

fn crowded_core() -> (Core, Limits) {
    let (mut core, limits) = fixture();
    // The other pool reserves the final hundred that the owner's original pool did not take.
    send_task(
        &mut core,
        &limits,
        tasks::Event::CarvePool {
            reply_to: ReplyTo::new(Token::new(82)),
            project: PROJECT,
            person: OTHER,
            period: PERIOD,
            budget: 100,
        },
    );
    (core, limits)
}
fn fills_period(core: &Core) -> ConfiguredPolicy {
    let mut value = replacement(core);
    value.policy.period_spend = PERIOD_BUDGET;
    value.policy.roles[0].period_spend = 900;
    value
}

#[test]
fn a_whole_policy_pool_raise_may_fill_the_open_period() {
    let (mut core, limits) = crowded_core();
    let value = fills_period(&core);
    assert_eq!(
        outcome(&change(&mut core, &limits, Box::new([item(value)]))),
        people::Outcome::SettingsChanged { revision: 2 }
    );
    let period = core.tasks.funding(tasks::Funder::Period { project: PROJECT, period: PERIOD }).unwrap();
    assert_eq!(period.numbers.reserved, PERIOD_BUDGET);
}

#[test]
fn a_whole_policy_pool_raise_past_open_room_is_refused_atomically() {
    let (mut core, limits) = crowded_core();
    let before = core.configured_policy(PROJECT).unwrap();
    let mut value = fills_period(&core);
    value.policy.roles[0].period_spend += 1;
    let mut charter = core.current.charter(1).unwrap().clone();
    charter.policy.instructions = b"must stay uncommitted".as_slice().into();
    let output = change(
        &mut core,
        &limits,
        Box::new([
            s::Item { setting: s::Setting::Charter(1), edit: s::Edit::Set(s::Value::Charter(Box::new(charter))) },
            item(value),
        ]),
    );
    assert_eq!(outcome(&output), malformed(s::Setting::ProjectPolicy { project: PROJECT }, s::Fault::Budget));
    assert_eq!(core.configured_policy(PROJECT), Some(before));
    assert!(core.current.changes.is_empty());
    assert_eq!(core.current.newest(), 1);
}

#[test]
fn a_project_policy_can_override_a_template_the_same_change_adds() {
    let (mut core, limits) = fixture();
    let template = Template {
        number: 2,
        name: b"work".as_slice().into(),
        resources: Box::default(),
        executor: people::TemplateExecutor::Chosen,
        contract: tasks::Contract::Report { words: 32 },
        shallower: 0,
        budget: people::TemplateBudget::Given,
        deadline: people::TemplateDeadline::None,
        tracked: None,
        flags: tasks::Flags::PLAIN,
        label: 0,
        grace: None,
        llm: None,
    };
    let mut value = replacement(&core);
    value.templates = Box::new([people::TemplateOverride {
        template: template.number,
        executor: None,
        budget: Some(people::TemplateBudget::Given),
        deadline: None,
        priority: None,
    }]);
    let output = change(
        &mut core,
        &limits,
        Box::new([
            item(value.clone()),
            s::Item {
                setting: s::Setting::Template(template.number),
                edit: s::Edit::Set(s::Value::Template(Box::new(template))),
            },
        ]),
    );
    assert_eq!(outcome(&output), people::Outcome::SettingsChanged { revision: 2 });
    assert_eq!(core.configured_policy(PROJECT), Some(value));
}

#[test]
fn a_policy_is_checked_against_the_rules_the_change_leaves() {
    let (mut core, limits) = fixture();
    let mut value = replacement(&core);
    value.policy.period_spend = PERIOD_BUDGET + 1;
    let mut rules = core.current.rules().clone();
    rules.period_spend = value.policy.period_spend;
    let output = change(
        &mut core,
        &limits,
        Box::new([
            item(value.clone()),
            s::Item { setting: s::Setting::Rules, edit: s::Edit::Set(s::Value::Rules(Box::new(rules))) },
        ]),
    );
    assert_eq!(outcome(&output), people::Outcome::SettingsChanged { revision: 2 });
    assert_eq!(core.configured_policy(PROJECT), Some(value));
}

#[test]
fn distinct_projects_still_allow_only_one_policy_item() {
    let (core, limits) = fixture();
    let first = item(replacement(&core));
    let mut second = first.clone();
    second.setting = s::Setting::ProjectPolicy { project: PROJECT + 1 };
    assert_eq!(
        s::change_shape(&[first, second.clone()], &limits),
        Err(s::Failure { setting: second.setting, fault: s::Fault::Shape })
    );
}

#[test]
fn a_whole_policy_checks_each_roles_models_as_an_owner_edit_does() {
    let (mut core, limits) = fixture();
    let mut value = replacement(&core);
    value.policy.ceiling.models = authority::Models::from_parts(0, false);
    let output = change(&mut core, &limits, Box::new([item(value)]));
    assert_eq!(outcome(&output), malformed(s::Setting::ProjectPolicy { project: PROJECT }, s::Fault::Reference));
    assert_eq!(core.current.newest(), 1);
}
