//! Adversarial proposals of deployment settings (`domain/engine.md`, 17.6).
use crate::settings_support as support;
use jig_core_world::fixtures;
use jig_core_world::fixtures::{PARTY as OWNER, sign_in as signin};
use jig_core_world::peers;
use jig_core_world::reads;
use jig_core_world::world::Cut;
use jig_core_world::world::Setup;
use jig_core_world::world::World;
use jig_domain_core::{self as core, settings as s};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

const SEED: u64 = 6001;

const DEPLOYMENT: u32 = 0;
// The small profile's settings bound; one further party exercises refusal.
const PROPOSAL_LIMIT: u32 = 4;
const PARTIES: u32 = PROPOSAL_LIMIT + 1;
const CHARTER: u32 = 1;

fn configuration() -> (root::Config, root::Limits) {
    let (mut config, mut limits) = peers::fixture(SEED, 0);
    config.core.owners = Box::new([
        config.core.owners[0].clone(),
        people::InitialOwner { project: DEPLOYMENT, identity: config.core.owners[0].identity.clone() },
    ]);
    limits.core.people.initial_owners = u32::try_from(config.core.owners.len()).expect("two configured roles");
    limits.core.people.people = PARTIES;
    limits.core.people.sign_ins = PARTIES;
    limits.core.people.holdings = PARTIES;
    limits.core.settings.proposals = PROPOSAL_LIMIT;
    peers::journal(&mut limits);
    (config, limits)
}

fn world() -> World {
    let (config, limits) = configuration();
    let mut world = World::new(Setup { seed: SEED, configuration: Some(config), limits, ..Setup::default() });
    world.reads = Some(reads::Parties::new(world.store.applied));
    world
}

fn charter_item(words: &[u8]) -> s::Item {
    let mut charter = configuration().0.core.defaults.charters[0].clone();
    charter.policy.instructions = words.into();
    s::Item { setting: s::Setting::Charter(CHARTER), edit: s::Edit::Set(s::Value::Charter(Box::new(charter))) }
}

fn proposal_event(sign_in: u64, key: u8, against: u32, items: Box<[s::Item]>) -> root::Event {
    root::Event::Core(core::Event::ProposeSettings {
        reply_to: ReplyTo::new(Token::new(30_000 + u64::from(key))),
        sign_in,
        key: [key; 16],
        digest: [key; 32],
        change: Box::new(s::Proposed { against, items, reason: b"QA proposal".as_slice().into() }),
    })
}

fn propose(world: &mut World, sign_in: u64, key: u8, against: u32, items: Box<[s::Item]>) -> u64 {
    world.send(proposal_event(sign_in, key, against, items));
    match world.people_answers.last() {
        Some(people::Reply::Outcome(people::Outcome::Proposed { proposal, .. })) => *proposal,
        reply => panic!("settings proposal key {key}: {reply:?}"),
    }
}

fn decide(world: &mut World, sign_in: u64, key: u8, proposer: u64, proposal: u64, decision: people::ProposalDecision) {
    fixtures::ask_as(
        world,
        sign_in,
        key,
        people::Ask::DecideProposal {
            project: DEPLOYMENT,
            proposer: people::Proposer::Party(proposer),
            proposal,
            running: match decision {
                people::ProposalDecision::Accept => Some(people::Running::Stays),
                people::ProposalDecision::Reject { .. } | people::ProposalDecision::Pass => None,
            },
            decision,
        },
    );
}

fn proposal_document(world: &mut World, sign_in: u64, proposer: u64, proposal: u64) -> core::documents::Proposal {
    world.sign_in = Some(sign_in);
    reads::ask(world, 1, core::reads::Read::Proposal { proposer: people::Proposer::Party(proposer), proposal });
    let core::reads::Answer::Document(doc) = reads::latest(world) else {
        panic!("proposal read: {:?}", reads::latest(world))
    };
    let core::documents::Document::Proposal(doc) = &**doc else { panic!("proposal document: {doc:?}") };
    *doc.clone()
}

fn change(world: &mut World, sign_in: u64, key: u8, against: u32, items: Box<[s::Item]>) {
    world.send(root::Event::Core(core::Event::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(31_000 + u64::from(key))),
        sign_in,
        key: [key; 16],
        digest: [key; 32],
        change: Box::new(s::Change { against, items, running: people::Running::Stays }),
    }));
}

fn settings_document(world: &mut World, sign_in: u64) -> core::documents::Settings {
    world.sign_in = Some(sign_in);
    reads::ask(world, 2, core::reads::Read::Settings);
    let core::reads::Answer::Document(doc) = reads::latest(world) else {
        panic!("settings read: {:?}", reads::latest(world))
    };
    let core::documents::Document::Settings(doc) = &**doc else { panic!("settings document: {doc:?}") };
    *doc.clone()
}

#[test]
fn the_last_proposal_slot_is_usable() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    fill_proposals(&mut world, owner);
    assert_eq!(settings_document(&mut world, owner).pending.len(), PROPOSAL_LIMIT);
}

#[test]
fn one_past_the_proposal_limit_is_refused() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    fill_proposals(&mut world, owner);
    let fifth = fixtures::second_party(&mut world, b"fifth", people::Role::Member);
    world.send(proposal_event(fifth, 24, 1, Box::new([charter_item(b"one too many")])));
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Limit {
            which: people::LimitKind::SettingsProposals
        })))
    );
    assert_eq!(settings_document(&mut world, owner).pending.len(), PROPOSAL_LIMIT);
}

fn proposal_state(world: &World, number: u64) -> tasks::PersonProposalState {
    world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Tasks(
                tasks::Stored::PersonProposal(row) | tasks::Stored::DecidedProposal(row),
            )) if row.number == number => Some(row.state.clone()),
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("durable party proposal")
}

fn withdraw(world: &mut World, sign_in: u64, key: u8, proposal: u64) {
    fixtures::ask_as(world, sign_in, key, people::Ask::Withdraw { project: DEPLOYMENT, proposal });
}

#[test]
fn acceptance_withdraws_siblings_and_records_its_origin() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let chosen = propose(&mut world, owner, 40, 1, Box::new([charter_item(b"chosen")]));
    let member = fixtures::second_party(&mut world, b"member", people::Role::Member);
    let sibling = propose(&mut world, member, 41, 1, Box::new([charter_item(b"sibling")]));
    decide(&mut world, owner, 42, OWNER, chosen, people::ProposalDecision::Accept);
    assert!(matches!(proposal_state(&world, chosen), tasks::PersonProposalState::SettingsAccepted { .. }));
    assert!(matches!(proposal_state(&world, sibling), tasks::PersonProposalState::Withdrawn { .. }));
    assert_eq!(settings_document(&mut world, owner).newest, 2);
    let revision = settings_revision(&world, 2);
    assert_eq!(revision.head.by, s::Decider::Party(OWNER));
    let origin = revision.head.proposal.expect("proposal origin");
    assert_eq!(origin.0, people::Proposer::Party(OWNER));
    assert_eq!(u64::from(origin.1), chosen);
    assert!(!revision.head.moved);
}

#[test]
fn withdrawal_wins_over_a_later_acceptance() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let number = propose(&mut world, owner, 50, 1, Box::new([charter_item(b"withdraw me")]));
    withdraw(&mut world, owner, 51, number);
    decide(&mut world, owner, 52, OWNER, number, people::ProposalDecision::Accept);
    assert!(matches!(proposal_state(&world, number), tasks::PersonProposalState::Withdrawn { .. }));
    assert_eq!(settings_document(&mut world, owner).newest, 1);
}

fn settings_edge_propose(world: &mut World, signed: u64, proposer: people::Proposer, position: u32) -> u64 {
    let against = world.domain.settings().newest();
    let instructions =
        if position == 1 { b"First instructions".as_slice() } else { b"Replacement instructions".as_slice() };
    let items = Box::new([support::run_charter(world, instructions)]);
    match proposer {
        people::Proposer::Party(_) => support::proposed(&support::propose(
            world,
            signed,
            u8::try_from(position).expect("request key"),
            against,
            items,
        )),
        people::Proposer::Task(task) => {
            let attempt = world.assigned.as_ref().expect("proposing run").attempt;
            world.send(root::Event::Core(core::Event::NamedAction {
                to: ReplyTo::new(Token::new(SEED)),
                key: core::CallKey { task, attempt, completion: position, position: 0 },
                action: core::NamedAction::ProposeSettings {
                    against,
                    items,
                    reason: b"Review these instructions".as_slice().into(),
                },
            }));
            let core::CallPart::Proposed { proposal, .. } = &world.answers.last().expect("proposal answer").2 else {
                panic!("task settings proposal: {:?}", world.answers.last())
            };
            *proposal
        }
    }
}

#[test]
fn ended_settings_decisions_keep_the_first_choice_and_pending_replacement() {
    // (task proposer, how it ended, running) -> retained choice, or unknown after replacement.
    for (task_proposer, decision, running, choice) in [
        (false, None, people::Running::Stays, Some(people::ProposalChoice::Withdrawn)),
        (true, Some(people::ProposalDecision::Accept), people::Running::Stays, Some(people::ProposalChoice::Accepted)),
        (
            true,
            Some(people::ProposalDecision::Reject { reason: b"Keep the default".as_slice().into() }),
            people::Running::Moves,
            Some(people::ProposalChoice::Rejected),
        ),
        (true, None, people::Running::Stays, None),
        (true, None, people::Running::Moves, None),
    ] {
        let (mut world, signed, task) = support::proposing_run(true);
        let proposer = if task_proposer { people::Proposer::Task(task) } else { people::Proposer::Party(OWNER) };
        let proposal = settings_edge_propose(&mut world, signed, proposer, 1);
        if let Some(decision) = decision {
            let running =
                if decision == people::ProposalDecision::Accept { Some(people::Running::Stays) } else { None };
            support::ask(
                &mut world,
                signed,
                150,
                people::Ask::DecideProposal { project: DEPLOYMENT, proposer, proposal, decision, running },
            );
        } else if !task_proposer {
            support::ask(&mut world, signed, 150, people::Ask::Withdraw { project: DEPLOYMENT, proposal });
        }
        let through = fixtures::live_row(&world, task).inbox.iter().last().map(|word| word.number);
        world.turn(1, 0, through, b"Read the first proposal decision");
        let replacement = settings_edge_propose(&mut world, signed, proposer, 2);
        let decision_key = root::Key::Core(core::Key::Core(core::CoreKey::ProposalDecision { proposer, proposal }));
        let decision_before = world.store.rows.get(&decision_key).cloned();
        let at = match &decision_before {
            Some(root::Record::Core(core::Record::Core(core::CoreRecord::ProposalDecision(row)))) => row.at,
            _ => world.env().wall,
        };
        let settings = support::inventory(&mut world, signed);
        let ended_read = core::reads::Read::Proposal { proposer, proposal };
        let ended = support::read(&mut world, signed, ended_read.clone());
        let pending_read = core::reads::Read::Proposal { proposer, proposal: replacement };
        let pending = support::read(&mut world, signed, pending_read.clone());
        let task_before = fixtures::live_row(&world, task).clone();
        let reply = support::ask(
            &mut world,
            signed,
            151,
            people::Ask::DecideProposal {
                project: DEPLOYMENT,
                proposer,
                proposal,
                decision: people::ProposalDecision::Accept,
                running: Some(running),
            },
        );
        assert_eq!(support::inventory(&mut world, signed), settings);
        assert_eq!(support::read(&mut world, signed, ended_read), ended);
        assert_eq!(support::read(&mut world, signed, pending_read), pending);
        assert_eq!(fixtures::live_row(&world, task), &task_before);
        assert_eq!(world.store.rows.get(&decision_key), decision_before.as_ref());
        assert_eq!(
            reply,
            people::Reply::Outcome(match choice {
                Some(choice) =>
                    people::Outcome::ProposalDecided { proposer, proposal, by: OWNER, at, choice, repeated: true },
                None => people::Outcome::Refused(people::Refusal::Unknown),
            }),
            "{proposer:?}, {choice:?}, {running:?}"
        );
        let through = fixtures::live_row(&world, task).inbox.iter().last().map(|word| word.number);
        world.turn(2, 0, through, b"Reviewed the proposal decision");
        world.park_for_message(0);
        world.close();
    }
}

#[test]
fn missing_proposals_are_unknown_with_or_without_running() {
    for (proposer, running) in [
        (people::Proposer::Task(1), None),
        (people::Proposer::Task(1), Some(people::Running::Stays)),
        (people::Proposer::Task(1), Some(people::Running::Moves)),
        (people::Proposer::Party(OWNER), None),
        (people::Proposer::Party(OWNER), Some(people::Running::Stays)),
        (people::Proposer::Party(OWNER), Some(people::Running::Moves)),
    ] {
        let mut world = world();
        let owner = world.sign_in.expect("owner sign-in");
        let pending = propose(&mut world, owner, 53, 1, Box::new([charter_item(b"still waiting")]));
        let settings = settings_document(&mut world, owner);
        let state = proposal_state(&world, pending);
        fixtures::ask_as(
            &mut world,
            owner,
            54,
            people::Ask::DecideProposal {
                project: DEPLOYMENT,
                proposer,
                proposal: u64::MAX,
                decision: people::ProposalDecision::Accept,
                running,
            },
        );
        assert_eq!(
            world.people_answers.last(),
            Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Unknown))),
            "{proposer:?}, {running:?}"
        );
        assert_eq!(settings_document(&mut world, owner), settings);
        assert_eq!(proposal_state(&world, pending), state);
    }
}

#[test]
fn rejection_wins_over_a_later_acceptance() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let number = propose(&mut world, owner, 60, 1, Box::new([charter_item(b"reject me")]));
    decide(
        &mut world,
        owner,
        61,
        OWNER,
        number,
        people::ProposalDecision::Reject { reason: b"too broad".as_slice().into() },
    );
    decide(&mut world, owner, 62, OWNER, number, people::ProposalDecision::Accept);
    let tasks::PersonProposalState::Rejected { reason, .. } = proposal_state(&world, number) else {
        panic!("rejection retained")
    };
    assert_eq!(reason.as_ref(), b"too broad");
    assert_eq!(settings_document(&mut world, owner).newest, 1);
}

#[test]
fn only_the_proposer_can_withdraw_a_pending_proposal() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let member = fixtures::second_party(&mut world, b"member", people::Role::Member);
    let number = propose(&mut world, member, 70, 1, Box::new([charter_item(b"member proposal")]));
    withdraw(&mut world, owner, 71, number);
    assert!(matches!(proposal_state(&world, number), tasks::PersonProposalState::Pending { .. }));
    assert!(matches!(world.people_answers.last(), Some(people::Reply::Outcome(people::Outcome::Refused(_)))));
    withdraw(&mut world, member, 72, number);
    assert!(matches!(proposal_state(&world, number), tasks::PersonProposalState::Withdrawn { .. }));
}

#[test]
fn a_project_owner_cannot_accept_deployment_settings() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let project_owner = fixtures::second_party(&mut world, b"project owner", people::Role::Owner);
    let number = propose(&mut world, project_owner, 80, 1, Box::new([charter_item(b"project owner proposal")]));
    let proposer = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(row))) if row.number == number => {
                Some(row.proposer)
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("durable proposer");
    decide(&mut world, project_owner, 81, proposer, number, people::ProposalDecision::Accept);
    assert!(matches!(proposal_state(&world, number), tasks::PersonProposalState::Pending { .. }));
    assert!(matches!(world.people_answers.last(), Some(people::Reply::Outcome(people::Outcome::Refused(_)))));
    decide(&mut world, owner, 82, proposer, number, people::ProposalDecision::Accept);
    assert_eq!(settings_document(&mut world, owner).newest, 2);
}

#[test]
fn withdrawal_returns_a_full_tables_proposal_slot() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let numbers = fill_proposals(&mut world, owner);
    withdraw(&mut world, owner, 94, numbers[0]);
    propose(&mut world, owner, 95, 1, Box::new([charter_item(b"replacement")]));
    assert_eq!(settings_document(&mut world, owner).pending.len(), PROPOSAL_LIMIT);
}

#[test]
fn replaying_a_proposal_key_never_consumes_another_slot() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let first = propose(&mut world, owner, 100, 1, Box::new([charter_item(b"same request")]));
    let committed = world.store.applied;
    let replay = propose(&mut world, owner, 100, 1, Box::new([charter_item(b"same request")]));
    assert_eq!(replay, first);
    assert_eq!(world.store.applied, committed);
    assert_eq!(settings_document(&mut world, owner).pending.len(), 1);
}

fn fill_proposals(world: &mut World, owner: u64) -> Vec<u64> {
    let mut numbers = vec![propose(world, owner, 20, 1, Box::new([charter_item(b"bounded proposal")]))];
    for (key, subject) in [(21, b"second".as_slice()), (22, b"third".as_slice()), (23, b"fourth".as_slice())] {
        let sign_in = fixtures::second_party(world, subject, people::Role::Member);
        numbers.push(propose(world, sign_in, key, 1, Box::new([charter_item(b"bounded proposal")])));
    }
    numbers
}

#[test]
fn an_uncommitted_proposal_disappears_after_a_crash() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let replies = world.people_answers.len();
    let commit = world.store.applied + 1;
    world.cut = Some(Cut::Submitted(commit));
    world.send(proposal_event(owner, 110, 1, Box::new([charter_item(b"not durable yet")])));
    assert_eq!(world.reached_cut, Some(Cut::Submitted(commit)));
    assert_eq!(world.people_answers.len(), replies, "no answer before the commit");
    world.restart_with(configuration().0);
    assert!(settings_document(&mut world, owner).pending.is_empty());
    propose(&mut world, owner, 110, 1, Box::new([charter_item(b"not durable yet")]));
    assert_eq!(settings_document(&mut world, owner).pending.len(), 1);
}

#[test]
fn a_durable_unanswered_proposal_replays_after_a_crash() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let replies = world.people_answers.len();
    let commit = world.store.applied + 1;
    world.cut = Some(Cut::Durable(commit));
    world.send(proposal_event(owner, 111, 1, Box::new([charter_item(b"durable unanswered")])));
    assert_eq!(world.reached_cut, Some(Cut::Durable(commit)));
    assert_eq!(world.people_answers.len(), replies);
    world.restart_with(configuration().0);
    let committed = world.store.applied;
    propose(&mut world, owner, 111, 1, Box::new([charter_item(b"durable unanswered")]));
    assert_eq!(world.store.applied, committed, "the restored key only replays");
    assert_eq!(settings_document(&mut world, owner).pending.len(), 1);
}

#[test]
fn an_uncommitted_acceptance_restores_the_pending_review() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let number = propose(&mut world, owner, 112, 1, Box::new([charter_item(b"atomic acceptance")]));
    let before = proposal_document(&mut world, owner, OWNER, number);
    let replies = world.people_answers.len();
    let commit = world.store.applied + 1;
    world.cut = Some(Cut::Submitted(commit));
    decide(&mut world, owner, 113, OWNER, number, people::ProposalDecision::Accept);
    assert_eq!(world.reached_cut, Some(Cut::Submitted(commit)));
    assert_eq!(world.people_answers.len(), replies);
    world.restart_with(configuration().0);
    assert_eq!(proposal_document(&mut world, owner, OWNER, number), before);
    assert_eq!(settings_document(&mut world, owner).newest, 1);
    decide(&mut world, owner, 113, OWNER, number, people::ProposalDecision::Accept);
    assert_eq!(settings_document(&mut world, owner).newest, 2);
}

#[test]
fn durable_acceptance_and_sibling_withdrawal_restart_together() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let member = fixtures::second_party(&mut world, b"member", people::Role::Member);
    let chosen = propose(&mut world, owner, 114, 1, Box::new([charter_item(b"durable acceptance")]));
    let sibling = propose(&mut world, member, 115, 1, Box::new([charter_item(b"stale sibling")]));
    let replies = world.people_answers.len();
    let commit = world.store.applied + 1;
    world.cut = Some(Cut::Durable(commit));
    decide(&mut world, owner, 116, OWNER, chosen, people::ProposalDecision::Accept);
    assert_eq!(world.reached_cut, Some(Cut::Durable(commit)));
    assert_eq!(world.people_answers.len(), replies);
    world.restart_with(configuration().0);
    assert!(matches!(proposal_state(&world, chosen), tasks::PersonProposalState::SettingsAccepted { revision: 2, .. }));
    assert!(matches!(proposal_state(&world, sibling), tasks::PersonProposalState::Withdrawn { .. }));
    let committed = world.store.applied;
    decide(&mut world, owner, 116, OWNER, chosen, people::ProposalDecision::Accept);
    assert_eq!(world.store.applied, committed, "acceptance replay makes no revision");
    let settings = settings_document(&mut world, owner);
    assert_eq!(settings.newest, 2);
    assert!(settings.pending.is_empty());
}

#[test]
fn a_failed_acceptance_commit_exposes_no_success() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let number = propose(&mut world, owner, 117, 1, Box::new([charter_item(b"failed commit")]));
    let replies = world.people_answers.len();
    world.store.fault(jig_core_world::Fault::Fail { commit: world.store.applied + 1 });
    decide(&mut world, owner, 118, OWNER, number, people::ProposalDecision::Accept);
    assert!(world.stopped);
    assert_eq!(world.people_answers.len(), replies, "a failed decision has no success reply");
    world.restart_with(configuration().0);
    assert!(matches!(proposal_state(&world, number), tasks::PersonProposalState::Pending { .. }));
    assert_eq!(settings_document(&mut world, owner).newest, 1);
}

#[test]
fn a_release_revision_withdraws_restored_proposals() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let member = fixtures::second_party(&mut world, b"member", people::Role::Member);
    let number = propose(&mut world, member, 120, 1, Box::new([charter_item(b"before release")]));
    let proposer = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(row))) if row.number == number => {
                Some(row.proposer)
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("durable proposer");
    let mut release = configuration().0;
    release.core.defaults.charters[0].policy.instructions = b"release default".as_slice().into();
    world.restart_with(release);
    assert!(matches!(proposal_state(&world, number), tasks::PersonProposalState::Withdrawn { .. }));
    let settings = settings_document(&mut world, owner);
    assert_eq!(settings.newest, 2);
    assert!(settings.pending.is_empty());
    assert!(world.store.rows.values().any(|row| matches!(row,
        root::Record::Core(core::Record::Tasks(tasks::Stored::Update(update))) if update.party == proposer && update.task == 0
    )), "the proposer has its durable notice");
}

#[test]
fn a_masked_release_keeps_a_pending_proposal() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    change(&mut world, owner, 121, 1, Box::new([charter_item(b"stored charter")]));
    let number = propose(&mut world, owner, 122, 2, Box::new([charter_item(b"after stored change")]));
    let before = proposal_document(&mut world, owner, OWNER, number);
    let mut release = configuration().0;
    release.core.defaults.charters[0].policy.instructions = b"masked release default".as_slice().into();
    world.restart_with(release);
    assert_eq!(settings_document(&mut world, owner).newest, 2);
    assert_eq!(proposal_document(&mut world, owner, OWNER, number), before);
}

#[test]
fn losing_ownership_refuses_acceptance_and_keeps_the_proposal() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let member = fixtures::second_party(&mut world, b"member", people::Role::Member);
    let number = propose(&mut world, member, 123, 1, Box::new([charter_item(b"waiting for owner")]));
    let proposer = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(row))) if row.number == number => {
                Some(row.proposer)
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("durable proposer");
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: DEPLOYMENT,
        holdings: Box::new([]),
    })));
    decide(&mut world, owner, 124, proposer, number, people::ProposalDecision::Accept);
    assert!(matches!(world.people_answers.last(), Some(people::Reply::Outcome(people::Outcome::Refused(_)))));
    assert!(matches!(proposal_state(&world, number), tasks::PersonProposalState::Pending { .. }));
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: DEPLOYMENT,
        holdings: Box::new([people::Holding { person: OWNER, role: people::Role::Owner }]),
    })));
    decide(&mut world, owner, 125, proposer, number, people::ProposalDecision::Accept);
    assert_eq!(settings_document(&mut world, owner).newest, 2);
}

#[test]
fn a_proposal_without_an_owner_is_refused_at_admission() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: DEPLOYMENT,
        holdings: Box::new([]),
    })));
    world.send(proposal_event(owner, 126, 1, Box::new([charter_item(b"nobody can accept")])));
    assert!(matches!(
        world.people_answers.last(),
        Some(people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Authority {
            proposable: Some(people::Accepter::Nobody),
            ..
        })))
    ));
    assert!(
        !world
            .store
            .rows
            .values()
            .any(|row| matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(_)))))
    );
}

#[test]
fn an_observers_role_is_enough_to_propose_settings() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let observer = fixtures::second_party(&mut world, b"observer", people::Role::Observer);
    let number = propose(&mut world, observer, 127, 1, Box::new([charter_item(b"observer suggestion")]));
    let proposer = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(row))) if row.number == number => {
                Some(row.proposer)
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("durable proposer");
    let review = proposal_document(&mut world, owner, proposer, number);
    assert_eq!(review.project, DEPLOYMENT);
    assert!(matches!(proposal_state(&world, number), tasks::PersonProposalState::Pending { .. }));
}

#[test]
fn review_resolves_reset_removal_and_addition_against_its_revision() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    change(&mut world, owner, 130, 1, Box::new([charter_item(b"stored override")]));
    let mut added = configuration().0.core.defaults.templates[0].clone();
    added.number = 3;
    added.name = b"added template".as_slice().into();
    let added_value = s::Value::Template(Box::new(added));
    let number = propose(
        &mut world,
        owner,
        131,
        2,
        Box::new([
            s::Item { setting: s::Setting::Charter(CHARTER), edit: s::Edit::Reset },
            s::Item { setting: s::Setting::Template(2), edit: s::Edit::Removed },
            s::Item { setting: s::Setting::Template(3), edit: s::Edit::Set(added_value.clone()) },
        ]),
    );
    let doc = proposal_document(&mut world, owner, OWNER, number);
    let core::documents::ProposalAction::Settings { against, review: Some(review), .. } = doc.action else {
        panic!("pending review")
    };
    assert_eq!(against, 2);
    assert_eq!(review.newest, 2);
    assert_eq!(review.items.len(), 3);
    let reset = review.items.iter().find(|item| item.setting == s::Setting::Charter(CHARTER)).expect("reset item");
    let s::Edit::Set(stored) = charter_item(b"stored override").edit else { unreachable!("set fixture") };
    assert_eq!(reset.now, Some(stored));
    assert_eq!(reset.origin.map(|(_, revision)| revision), Some(2));
    assert_eq!(reset.would_be, Some(s::Value::Charter(Box::new(configuration().0.core.defaults.charters[0].clone()))));
    assert_eq!(reset.edit, core::reads::RevisionEdit::Reset);
    let removed = review.items.iter().find(|item| item.setting == s::Setting::Template(2)).expect("removal item");
    assert_eq!(removed.now, Some(s::Value::Template(Box::new(configuration().0.core.defaults.templates[1].clone()))));
    assert_eq!(removed.origin.map(|(_, revision)| revision), Some(1));
    assert_eq!(removed.would_be, None);
    assert_eq!(removed.edit, core::reads::RevisionEdit::Removed);
    let addition = review.items.iter().find(|item| item.setting == s::Setting::Template(3)).expect("addition item");
    assert_eq!(addition.now, None);
    assert_eq!(addition.origin, None);
    assert_eq!(addition.would_be, Some(added_value));
}

#[test]
fn a_stale_proposal_reports_the_newest_revision() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    change(&mut world, owner, 132, 1, Box::new([charter_item(b"revision two")]));
    world.send(proposal_event(owner, 133, 1, Box::new([charter_item(b"obsolete suggestion")])));
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Stale { newest: 2 })))
    );
    assert!(settings_document(&mut world, owner).pending.is_empty());
}

#[test]
fn acceptance_can_move_tasks_without_changing_a_live_run() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let assigned = world.assigned.as_ref().expect("live run");
    let (task, attempt) = (assigned.task, assigned.attempt);
    let assignments = world.assignments.len();
    let number = propose(&mut world, owner, 134, 1, Box::new([charter_item(b"the next run")]));
    fixtures::ask_as(
        &mut world,
        owner,
        135,
        people::Ask::DecideProposal {
            project: DEPLOYMENT,
            proposer: people::Proposer::Party(OWNER),
            proposal: number,
            decision: people::ProposalDecision::Accept,
            running: Some(people::Running::Moves),
        },
    );
    assert_eq!(settings_document(&mut world, owner).newest, 2);
    assert!(settings_revision(&world, 2).head.moved, "acceptance records the owner's Moves choice");
    assert_eq!(fixtures::live_row(&world, task).settings.task, 2);
    let assigned = world.assigned.as_ref().expect("same live run");
    assert_eq!((assigned.task, assigned.attempt), (task, attempt));
    assert_eq!(world.assignments.len(), assignments);
    assert!(world.cancellations.is_empty(), "a move takes effect at the next run");
}

#[test]
fn a_party_without_any_role_cannot_propose_settings() {
    let mut world = world();
    let owner = world.sign_in.expect("owner sign-in");
    let outsider = fixtures::second_party(&mut world, b"outsider", people::Role::Observer);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 1,
        holdings: Box::new([people::Holding { person: OWNER, role: people::Role::Owner }]),
    })));
    world.send(proposal_event(outsider, 136, 1, Box::new([charter_item(b"unauthorized suggestion")])));
    assert!(matches!(
        world.people_answers.last(),
        Some(people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role { held: None })))
    ));
    assert!(settings_document(&mut world, owner).pending.is_empty());
}

#[test]
fn acceptance_moves_work_created_after_a_real_first_start() {
    let (config, limits) = configuration();
    let mut world = World::new(Setup {
        configuration: Some(config),
        limits,
        boot: jig_core_world::world::Boot::Empty,
        ..Setup::default()
    });
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 1000, next: None }));
    world.send(signin(Token::new(800), 0, &7_u64.to_be_bytes(), b"owner", b"Owner"));
    let owner = world.sign_in.expect("owner sign-in");
    world.reads = Some(reads::Parties::new(world.store.applied));
    let task = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) => Some(row.number),
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("work created by the signed-in owner");
    let number = propose(&mut world, owner, 137, 1, Box::new([charter_item(b"next revision")]));
    fixtures::ask_as(
        &mut world,
        owner,
        138,
        people::Ask::DecideProposal {
            project: DEPLOYMENT,
            proposer: people::Proposer::Party(OWNER),
            proposal: number,
            decision: people::ProposalDecision::Accept,
            running: Some(people::Running::Moves),
        },
    );
    assert_eq!(settings_document(&mut world, owner).newest, 2);
    assert!(settings_revision(&world, 2).head.moved, "acceptance records the owner's Moves choice");
    assert_eq!(fixtures::live_row(&world, task).settings.task, 2);
}

#[test]
fn settings_proposal_profiles_keep_the_advertised_bounds() {
    assert_eq!(jig_protocol_config::profile(jig_protocol_config::Profile::Small).core.settings.proposals, 4);
    assert_eq!(jig_protocol_config::profile(jig_protocol_config::Profile::Standard).core.settings.proposals, 8);
}

fn settings_revision(world: &World, number: u32) -> &s::Revision {
    world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Core(core::CoreRecord::Settings(s::Record::Revision {
                revision,
                ..
            }))) if revision.head.number == number => Some(revision),
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("durable settings revision")
}

// The catalogue includes the model the proposed charter names (engine.md, 17.11).
fn model_proposal_configuration() -> (root::Config, root::Limits, core::Model) {
    let (mut config, limits) = configuration();
    let mut model = config.core.settings.models[0].clone();
    model.name = b"pending-only".as_slice().into();
    config.core.settings.models = Box::new([config.core.settings.models[0].clone(), model.clone()]);

    (config, limits, model)
}

fn model_proposal_items(model: core::Model) -> Box<[s::Item]> {
    let mut charter = configuration().0.core.defaults.charters[0].clone();
    charter.policy.model = model;
    charter.policy.instructions = b"proposed model and instructions".as_slice().into();
    let mut template = jig_test_profile::fixtures::templates()[0].clone();
    template.name = b"proposed chat name".as_slice().into();
    Box::new([
        s::Item {
            setting: s::Setting::Template(template.number),
            edit: s::Edit::Set(s::Value::Template(Box::new(template))),
        },
        s::Item { setting: s::Setting::Charter(CHARTER), edit: s::Edit::Set(s::Value::Charter(Box::new(charter))) },
    ])
}

fn durable_settings(world: &World) -> Vec<root::Record> {
    world
        .store
        .rows
        .values()
        .filter(|row| matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Settings(_)))))
        .cloned()
        .collect()
}

#[test]
fn held_versions_past_the_limit_keep_the_pending_set_across_restart() {
    // engine.md, 17.2, 17.4, 17.6 and 17.8: a model change can keep
    // an incompatible task on its version; a second held version is full.
    let (config, mut limits, model) = model_proposal_configuration();
    limits.core.settings.versions = 1;
    peers::journal(&mut limits);
    let mut world = World::new(Setup { seed: SEED, configuration: Some(config), limits, ..Setup::default() });
    world.reads = Some(reads::Parties::new(world.store.applied));
    let owner = world.sign_in.expect("owner sign-in");
    let task = world.assigned.as_ref().expect("live holder").task;
    fixtures::ask_as(
        &mut world,
        owner,
        151,
        people::Ask::Amend {
            project: 1,
            task,
            revision: 0,
            release: None,
            amendment: people::Amendment {
                llm: Some(people::LlmChoice {
                    account: None,
                    main: people::ModelEffort { model: 0, effort: Some(0) },
                    subagents: Box::new([]),
                }),
                ..jig_test_profile::fixtures::person_unchanged(b"keep the current model")
            },
        },
    );
    let task_before = fixtures::live_row(&world, task).clone();
    let policy_before = world.assigned.as_ref().expect("live run").run.policy.clone();
    let first = propose(&mut world, owner, 152, 1, model_proposal_items(model.clone()));
    decide(&mut world, owner, 153, OWNER, first, people::ProposalDecision::Accept);
    assert!(matches!(
        world.people_answers.last(),
        Some(people::Reply::Outcome(people::Outcome::ProposalDecided { choice: people::ProposalChoice::Accepted, .. }))
    ));
    assert_eq!(settings_document(&mut world, owner).newest, 2);
    assert_eq!(fixtures::live_row(&world, task), &task_before, "acceptance keeps the old task's version");
    assert_eq!(world.assigned.as_ref().expect("same live run").run.policy, policy_before);
    fixtures::ask_as(&mut world, owner, 154, people::Ask::MoveToNewest { revision: 1 });
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::MovedToNewest { newest: 2, moved: 0, stayed: 1 }))
    );
    assert_eq!(fixtures::live_row(&world, task), &task_before, "the newest disallows the old LLM choice");

    let chat = fixtures::chat(&world, 1, b"hold revision two".as_slice().into());
    fixtures::ask_as(&mut world, owner, 155, chat);
    let Some(people::Reply::Outcome(people::Outcome::Created { first: second, .. })) = world.people_answers.last()
    else {
        panic!("second live holder: {:?}", world.people_answers.last())
    };
    let second = *second;
    let second_before = fixtures::live_row(&world, second).clone();
    assert_eq!(second_before.settings.task, 2);
    let mut items = model_proposal_items(model);
    let s::Edit::Set(s::Value::Template(template)) = &mut items[0].edit else { unreachable!("template item") };
    template.name = b"third proposed chat name".as_slice().into();
    let s::Edit::Set(s::Value::Charter(charter)) = &mut items[1].edit else { unreachable!("charter item") };
    charter.policy.instructions = b"a third charter version".as_slice().into();
    let number = propose(&mut world, owner, 156, 2, items);
    let proposal_before = proposal_document(&mut world, owner, OWNER, number);
    let settings_before = settings_document(&mut world, owner);
    let rows_before = durable_settings(&world);
    decide(&mut world, owner, 157, OWNER, number, people::ProposalDecision::Accept);
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Full {
            revision: 1,
            tasks: Box::new([task]),
            more: 0,
        })))
    );
    assert_eq!(proposal_document(&mut world, owner, OWNER, number), proposal_before);
    assert_eq!(settings_document(&mut world, owner), settings_before);
    assert_eq!(durable_settings(&world), rows_before, "full commits no item of the set");
    assert_eq!(fixtures::live_row(&world, task), &task_before, "full does not move a holder");
    assert!(matches!(proposal_state(&world, number), tasks::PersonProposalState::Pending { .. }));
    assert_eq!(fixtures::live_row(&world, second), &second_before, "full keeps the second holder too");
    world.restart_with(model_proposal_configuration().0);
    assert_eq!(fixtures::live_row(&world, task).settings.task, 1);
    assert_eq!(fixtures::live_row(&world, second).settings.task, 2);
    assert!(matches!(proposal_state(&world, number), tasks::PersonProposalState::Pending { .. }));
    assert_eq!(proposal_document(&mut world, owner, OWNER, number), proposal_before);
    assert_eq!(settings_document(&mut world, owner), settings_before);
}
