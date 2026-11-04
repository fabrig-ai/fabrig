//! A project's policy beside deployment settings (jig/docs/design/domain/engine.md, 17.2 and 17.6).
use crate::settings_support::*;
use jig_core_world::{
    client_peer::ClientPeer,
    world::{Boot, Setup, World},
};
use jig_domain_core::{self as core, documents, reads, settings};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

const HISTORY: u32 = 64;
const PROJECT: u32 = 1;
const SPEND: u64 = 100;
const CHANGED_SPEND: u64 = 120;
const INSTRUCTIONS: &[u8] = b"Set up the project and its charter together.";

fn story_configuration() -> (root::Config, root::Limits) {
    let (mut config, mut limits) = configuration();
    // The policy's owner pools spend 100; leave room to raise its period alone.
    config.core.defaults.rules.period_spend = 200;
    limits.core.settings.history = HISTORY;
    (config, limits)
}

fn setup() -> (World, u64) {
    let (config, limits) = story_configuration();
    setup_with(config, &limits)
}

fn setup_with(config: root::Config, limits: &root::Limits) -> (World, u64) {
    let mut world = World::new(Setup {
        seed: SEED,
        configuration: Some(config),
        limits: *limits,
        boot: Boot::Empty,
        ..Setup::default()
    });
    world.client_peer = Some(ClientPeer::default());
    let owner = sign_in(&mut world);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: PROJECT,
        holdings: Box::new([people::Holding { person: OWNER, role: people::Role::Owner }]),
    })));
    world.send(root::Event::Core(core::Event::Period { project: PROJECT, period: 1, budget: SPEND, next: None }));
    (world, owner)
}

fn items(world: &World, project: u32) -> Box<[settings::Item]> {
    Box::new([run_charter(world, INSTRUCTIONS), project_policy_item(world, project, CHANGED_SPEND)])
}

fn policy_set(world: &mut World, owner: u64, project: u32) {
    let document = project_document(world, owner, project);
    assert_eq!(document.policy.period_spend, CHANGED_SPEND);
    assert_eq!(document.current_period.expect("open period").numbers.budget, CHANGED_SPEND);
}

#[test]
fn an_owner_of_both_sets_charters_and_a_project_policy_in_one_change() {
    let (mut world, owner) = setup();
    let before = configuration().0.core.seed_projects[0].policy.clone();
    let items = items(&world, PROJECT);
    let commits = world.commits.len();
    changed(&change(&mut world, owner, 1, 1, items.clone()), 2);
    assert_eq!(world.commits.len(), commits + 1, "the charter, policy and revision share one decision");
    assert_eq!(inventory(&mut world, owner).newest, 2);
    policy_set(&mut world, owner, PROJECT);
    assert_eq!(world.domain.settings().charter(1).expect("charter").policy.instructions.as_ref(), INSTRUCTIONS);
    let revision = revision(&world, 2, false);
    assert_eq!(revision.items.len(), items.len());
    for (recorded, item) in revision.items.iter().zip(&items) {
        assert_eq!((recorded.setting, &recorded.edit), (item.setting, &item.edit));
    }
    assert_eq!(revision.items[1].replaced, settings::Replaced::Value(settings::Value::Policy(Box::new(before))));
}

#[test]
fn a_deployment_only_owner_is_refused_a_project_policy_item() {
    let (mut world, owner) = setup();
    let (_, deployment_owner) = party(&mut world, b"deployment owner", Some((0, people::Role::Owner)));
    let before = project_document(&mut world, owner, PROJECT);
    let settings = settings_rows(&world);
    let stored = project_rows(&world);
    let items = items(&world, PROJECT);
    assert!(matches!(
        change(&mut world, deployment_owner, 1, 1, items),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role { .. }))
    ));
    assert_eq!(settings_rows(&world), settings);
    assert_eq!(project_rows(&world), stored);
    assert_eq!(project_document(&mut world, owner, PROJECT), before);
}

fn project_rows(world: &World) -> Vec<(root::Key, root::Record)> {
    world
        .store
        .rows
        .iter()
        .filter(|(key, _)| {
            matches!(
                key,
                root::Key::Core(
                    core::Key::People(people::Key::Policy(PROJECT) | people::Key::Roles(PROJECT))
                        | core::Key::Tasks(tasks::Key::Ledger(
                            tasks::Funder::Period { project: PROJECT, .. }
                                | tasks::Funder::Pool { project: PROJECT, .. }
                        ))
                )
            )
        })
        .map(|(key, row)| (key.clone(), row.clone()))
        .collect()
}

fn accept(world: &mut World, signed: u64, request: u8, proposer: u64, proposal: u64) -> people::Reply {
    decide(world, signed, request, proposer, proposal, people::ProposalDecision::Accept, Some(people::Running::Stays))
}

fn accepted(reply: &people::Reply) {
    assert!(
        matches!(
            reply,
            people::Reply::Outcome(people::Outcome::ProposalDecided { choice: people::ProposalChoice::Accepted, .. })
        ),
        "{reply:?}"
    );
}

#[test]
fn a_proposal_naming_a_project_policy_waits_for_an_owner_of_both() {
    let (mut world, owner) = setup();
    let (member, signed) = party(&mut world, b"member", Some((PROJECT, people::Role::Member)));
    let (_, project_owner) = party(&mut world, b"project owner", Some((PROJECT, people::Role::Owner)));
    let (_, deployment_owner) = party(&mut world, b"deployment owner", Some((0, people::Role::Owner)));
    let items = items(&world, PROJECT);
    let proposal = proposed(&propose(&mut world, signed, 1, 1, items.clone()));
    let document = proposal_document(&mut world, owner, member, proposal);
    let documents::ProposalAction::Settings { against, review: Some(review), .. } = &document.action else {
        panic!("pending settings review")
    };
    assert_eq!((*against, review.newest, review.items.len()), (1, 1, 2));
    let policy = review
        .items
        .iter()
        .find(|item| item.setting == settings::Setting::ProjectPolicy { project: PROJECT })
        .expect("project policy review");
    assert_eq!(
        policy.now,
        Some(settings::Value::Policy(Box::new(configuration().0.core.seed_projects[0].policy.clone())))
    );
    let settings::Edit::Set(value) = &items[1].edit else { panic!("policy set") };
    assert_eq!(policy.would_be.as_ref(), Some(value));
    assert_eq!(policy.edit, reads::RevisionEdit::Set);
    for reviewer in [project_owner, deployment_owner] {
        assert!(matches!(
            accept(&mut world, reviewer, 1, member, proposal),
            people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role { .. }))
        ));
        assert_eq!(proposal_document(&mut world, owner, member, proposal), document);
        assert!(world.store.rows.contains_key(&proposal_set_key(member, proposal)));
        assert_eq!(inventory(&mut world, owner).newest, 1);
    }
    accepted(&accept(&mut world, owner, 1, member, proposal));
    assert_eq!(inventory(&mut world, owner).newest, 2);
    assert_eq!(revision(&world, 2, false).head.by, settings::Decider::Party(OWNER));
    policy_set(&mut world, owner, PROJECT);
}

#[test]
fn a_proposal_no_party_owns_both_of_is_refused_at_its_entrance() {
    let (mut world, owner) = setup();
    let (member, signed) = party(&mut world, b"member", Some((PROJECT, people::Role::Member)));
    let (deployment_owner, _) = party(&mut world, b"deployment owner", Some((0, people::Role::Owner)));
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 0,
        holdings: Box::new([people::Holding { person: deployment_owner, role: people::Role::Owner }]),
    })));
    let settings = settings_rows(&world);
    let before = project_document(&mut world, owner, PROJECT);
    let items = items(&world, PROJECT);
    assert!(matches!(
        propose(&mut world, signed, 1, 1, items),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Authority {
            proposable: Some(people::Accepter::Nobody),
            ..
        }))
    ));
    assert_eq!(settings_rows(&world), settings);
    assert_eq!(project_document(&mut world, owner, PROJECT), before);
    assert!(
        !world
            .store
            .rows
            .values()
            .any(|row| matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(_)))))
    );
    assert!(!world.store.rows.contains_key(&proposal_set_key(member, 1)));
}

#[test]
fn deleting_a_project_withdraws_a_proposal_naming_its_policy() {
    const SECOND: u32 = 2;
    let (mut config, mut limits) = story_configuration();
    config.core.grants = Box::new([people::ActGrant { connector: 1, may: Box::new([people::ActKind::RemoveProject]) }]);
    limits.core.people.act_grants = 1;
    let (mut world, owner) = setup_with(config, &limits);
    assert_eq!(
        ask(&mut world, owner, 1, people::Ask::MakeProject { name: "Second project".into(), policy: 1 }),
        people::Reply::Outcome(people::Outcome::ProjectMade { project: SECOND })
    );
    let (member, signed) = party(&mut world, b"member", Some((PROJECT, people::Role::Member)));
    let removed_items = items(&world, SECOND);
    let removed = proposed(&propose(&mut world, signed, 1, 1, removed_items));
    let kept_items = Box::new([project_policy_item(&world, PROJECT, CHANGED_SPEND)]);
    let kept = proposed(&propose(&mut world, owner, 2, 1, kept_items));
    let review = proposal_document(&mut world, owner, OWNER, kept);
    let closing = world.commits.len();
    world.send(root::Event::Core(core::Event::Act {
        reply_to: ReplyTo::new(Token::new(SEED)),
        connector: 1,
        key: [3; 16],
        act: Box::new(people::Act::RemoveProject { project: SECOND, grace: None }),
        carried: None,
    }));
    let decision = &world.commits[closing];
    assert!(decision.iter().any(|write| matches!(
        write,
        root::Write::Save(root::Record::Core(core::Record::People(people::Stored::Acted {
            outcome: people::Outcome::ProjectRemoved { project: SECOND },
            ..
        })))
    )));
    assert!(decision.iter().any(|write| matches!(write,
        root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::DecidedProposal(row))))
        if row.number == removed && matches!(row.state, tasks::PersonProposalState::Withdrawn { .. }))));
    assert!(decision.iter().any(|write| matches!(write, root::Write::Erase(key)
        if *key == proposal_set_key(member, removed))));
    assert!(!world.store.rows.contains_key(&proposal_set_key(member, removed)));
    assert!(updates(&mut world, signed).iter().any(|entry| entry.object
        == documents::InboxObject::Proposal { proposer: people::Proposer::Party(member), number: removed }));
    let reads::Answer::Ended(history) = read(
        &mut world,
        signed,
        reads::Read::Proposal { proposer: people::Proposer::Party(member), proposal: removed },
    ) else {
        panic!("withdrawn proposal")
    };
    let reads::EndedValue::PersonProposal { proposal, decision: Some(decision) } = history.value else {
        panic!("withdrawal decision")
    };
    assert!(matches!(proposal.state, tasks::PersonProposalState::Withdrawn { .. }));
    assert_eq!(decision.choice, people::ProposalChoice::Withdrawn);
    assert_eq!(proposal_document(&mut world, owner, OWNER, kept), review);
    assert!(world.store.rows.contains_key(&proposal_set_key(OWNER, kept)));
    assert_eq!(inventory(&mut world, owner).newest, 1);
    world.advance(skein_lib::Duration::from_secs(6));
    assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::Deletion(SECOND)))));
}

#[test]
fn a_restart_keeps_a_pending_project_policy_proposal() {
    let (mut world, owner) = setup();
    let (member, signed) = party(&mut world, b"member", Some((PROJECT, people::Role::Member)));
    let items = items(&world, PROJECT);
    let proposal = proposed(&propose(&mut world, signed, 1, 1, items));
    let before = proposal_document(&mut world, owner, member, proposal);
    restart(&mut world, story_configuration().0);
    let owner = sign_in(&mut world);
    assert_eq!(proposal_document(&mut world, owner, member, proposal), before);
    assert!(world.store.rows.contains_key(&proposal_set_key(member, proposal)));
    accepted(&accept(&mut world, owner, 1, member, proposal));
    assert_eq!(inventory(&mut world, owner).newest, 2);
    policy_set(&mut world, owner, PROJECT);
}
