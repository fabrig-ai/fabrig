//! Party and run settings proposals through the real store and commit barrier (domain/engine/settings.md, 6).

use crate::settings_support as support;
use jig_core_world::client_peer::ClientPeer;
use jig_core_world::fixtures;
use jig_core_world::fixtures::accepted;
use jig_core_world::peers;
use jig_core_world::world::Setup;
use jig_core_world::world::World;
use jig_domain_core::{self as core, documents, reads, settings};
use jig_domain_link::{Hello, Hosting, Phase};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::length::len_u64;
use support::*;

fn project_proposals_configuration() -> (root::Config, root::Limits) {
    let (mut config, mut limits) = run_configuration();
    limits.core.authority.projects = 3;
    limits.core.people.projects = 4;
    peers::journal(&mut limits);
    config.core.kinds = Box::new([settings::Kind {
        number: 1,
        scope: settings::KindScope::Project,
        connector: 1,
        value_bytes: 64,
        per_scope: 2,
        charters: 1,
    }]);
    config.first.settings = Some(jig_test_connector::SettingsKind {
        kind: 1,
        project: true,
        value_bytes: 64,
        per_scope: 2,
        charters: true,
        defaults: Box::new([]),
    });
    let policy = config.core.seed_projects[0].policy.clone();
    config.core.seed_projects = Box::new([
        core::SeedProject { number: 1, name: "one".into(), policy: policy.clone() },
        core::SeedProject { number: 2, name: "two".into(), policy: policy.clone() },
        core::SeedProject { number: 3, name: "three".into(), policy },
    ]);
    (config, limits)
}

fn project_proposals_world() -> (World, u64) {
    let (config, limits) = project_proposals_configuration();
    let mut world = World::new(Setup {
        configuration: Some(config),
        limits,
        boot: jig_core_world::world::Boot::Empty,
        ..Setup::default()
    });
    world.client_peer = Some(ClientPeer::default());
    let signed = sign_in(&mut world);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 2,
        holdings: Box::new([people::Holding { person: OWNER, role: people::Role::Owner }]),
    })));
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 3,
        holdings: Box::new([people::Holding { person: OWNER, role: people::Role::Member }]),
    })));
    (world, signed)
}

fn application_item(scope: u32) -> settings::Item {
    settings::Item {
        setting: settings::Setting::Application { kind: 1, scope, number: 1 },
        edit: settings::Edit::Set(settings::Value::Application(b"on".as_slice().into())),
    }
}

fn proposal_inbox(world: &mut World) -> documents::Inbox {
    let env = world.env();
    let documents::Document::Inbox(inbox) = documents::make(
        world.domain.core_mut(),
        &env.limits.core,
        OWNER,
        &documents::Subject::Inbox { party: OWNER },
        &documents::Window { rows: 32, after: None },
        env.wall,
    )
    .expect("inbox document") else {
        panic!("inbox")
    };
    inbox
}

fn assert_proposal_projects(world: &mut World, signed: u64, proposer: people::Proposer, number: u64, projects: &[u32]) {
    let inbox = proposal_inbox(world);
    let row = inbox
        .needs
        .iter()
        .find(|entry| entry.object == documents::InboxObject::Proposal { proposer, number })
        .expect("one proposal row");
    assert_eq!(row.project, Some(0));
    assert_eq!(row.projects.as_slice(), projects);
    assert_eq!(inbox.needs.len(), 1);
    assert_eq!(inbox.cached_needs, 1);
    for project in [None, Some(0), Some(1), Some(2), Some(3)] {
        let filter = reads::Filter { project, goal: None };
        let reads::Answer::Page(page) =
            read(world, signed, reads::Read::Inbox { group: reads::Group::Proposals, filter, from: None })
        else {
            panic!("filtered proposals")
        };
        let expected = project.is_none_or(|project| project == 0 || projects.contains(&project));
        assert_eq!(page.rows.len(), u32::from(expected), "project {project:?}");
        if let Some(reads::PageRow::Inbox(entry)) = page.rows.get(0) {
            assert_eq!(entry.object, row.object);
            assert_eq!(entry.projects, row.projects);
        }
    }
}

#[test]
fn a_proposal_of_one_projects_item_shows_in_that_project_alone() {
    let (mut world, signed) = project_proposals_world();
    let number = proposed(&propose(&mut world, signed, 1, 1, Box::new([application_item(1)])));
    assert_proposal_projects(&mut world, signed, people::Proposer::Party(OWNER), number, &[1]);
    assert_eq!(proposal_document(&mut world, signed, OWNER, number).projects.as_slice(), &[1]);
    accepted(&decide(
        &mut world,
        signed,
        2,
        OWNER,
        number,
        people::ProposalDecision::Accept,
        Some(people::Running::Stays),
    ));
    assert!(proposal_inbox(&mut world).needs.is_empty());
}

#[test]
fn a_proposal_touching_two_projects_shows_in_both_and_counts_once() {
    let mut items: Vec<_> = (1..=35).rev().map(application_item).collect();
    items.push(application_item(0));
    items.push(application_item(1));
    assert_eq!(settings::touched_projects(&items).as_slice(), (1..=32).collect::<Vec<_>>());
    for policy in [false, true] {
        let (mut world, signed) = project_proposals_world();
        let second = if policy { project_policy_item(&world, 2, 100) } else { application_item(2) };
        let number = proposed(&propose(&mut world, signed, 1, 1, Box::new([second, application_item(1)])));
        assert_proposal_projects(&mut world, signed, people::Proposer::Party(OWNER), number, &[1, 2]);
        assert_eq!(proposal_document(&mut world, signed, OWNER, number).projects.as_slice(), &[1, 2]);
    }
}

#[test]
fn a_deployments_proposal_shows_in_each_project_its_reader_owns() {
    for empty in [false, true] {
        let (mut world, signed) = project_proposals_world();
        let items = Box::new([run_charter(&world, b"Review")]);
        let number = proposed(&propose(&mut world, signed, 1, 1, items));
        let signed = if empty {
            // Empty sets are restored to exercise projection independently of no-op admission.
            let root::Record::Core(core::Record::Core(core::CoreRecord::Settings(settings::Record::Proposal {
                value,
                ..
            }))) = world.store.rows.get_mut(&proposal_set_key(OWNER, number)).expect("stored set")
            else {
                panic!("settings set")
            };
            value.items = Box::new([]);
            world.restart_with(project_proposals_configuration().0);
            assert!(world.domain.ready());
            sign_in(&mut world)
        } else {
            signed
        };
        assert_proposal_projects(&mut world, signed, people::Proposer::Party(OWNER), number, &[1, 2]);
        assert!(proposal_document(&mut world, signed, OWNER, number).projects.is_empty());
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one run story covers scopes, restart and decisions")]
fn a_runs_proposal_belongs_as_a_partys_does() {
    for case in 0..5 {
        let (mut world, mut signed) = project_proposals_world();
        world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
        world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Hello {
            worker: 0,
            channel: skein_lib::Token::new(7),
            hello: Hello {
                jobs: Box::default(),
                bounds: jig_test_profile::fixtures::bounds(),
                stop_bound: skein_lib::Duration::from_millis(1),
                slots: 1,
                workstreams: Box::new([]),
                hosting: Box::new([]),
            },
        })));
        let mut goal = fixtures::chat(&world, 1, b"Review settings".as_slice().into());
        let people::Ask::Create { batch, .. } = &mut goal else { panic!("goal creation") };
        batch[0].flags.conversation = false;
        batch[0].flags.listed = true;
        batch[0].budget = people::Budget::Named;
        batch[0].authority.spend = 40;
        batch[0].authority.tools = 2047;
        let people::Reply::Outcome(people::Outcome::Created { first: task, .. }) = ask(&mut world, signed, 101, goal)
        else {
            panic!("proposing run")
        };
        let (items, projects): (Box<[settings::Item]>, &[u32]) = match case {
            0 => (Box::new([application_item(1)]), &[1]),
            1 => (Box::new([application_item(2), application_item(1)]), &[1, 2]),
            2 => (Box::new([project_policy_item(&world, 2, 100), application_item(1)]), &[1, 2]),
            _ => (Box::new([run_charter(&world, b"Review")]), &[]),
        };
        let key = core::CallKey {
            task,
            attempt: world.assigned.as_ref().expect("assigned run").attempt,
            completion: 1,
            position: 0,
        };
        world.send(root::Event::Core(core::Event::NamedAction {
            to: skein_lib::ReplyTo::new(skein_lib::Token::new(SEED)),
            key,
            action: core::NamedAction::ProposeSettings { against: 1, items, reason: b"Review".as_slice().into() },
        }));
        let number = run_proposed(&world.answers.last().expect("run proposal answer").2);
        if case == 4 {
            let root::Record::Core(core::Record::Core(core::CoreRecord::Settings(settings::Record::Proposal {
                value,
                ..
            }))) = world.store.rows.get_mut(&task_set(task, number)).expect("stored run set")
            else {
                panic!("settings set")
            };
            value.items = Box::new([]);
            world.restart_with(project_proposals_configuration().0);
            assert!(world.domain.ready());
            signed = sign_in(&mut world);
        }
        assert_proposal_projects(
            &mut world,
            signed,
            people::Proposer::Task(task),
            number,
            if projects.is_empty() { &[1, 2] } else { projects },
        );
        let reads::Answer::Document(document) = read(
            &mut world,
            signed,
            reads::Read::Proposal { proposer: people::Proposer::Task(task), proposal: number },
        ) else {
            panic!("run proposal")
        };
        let documents::Document::Proposal(document) = *document else { panic!("proposal") };
        assert_eq!(document.projects.as_slice(), projects);
        if case == 4 {
            let reply = run_decide(
                &mut world,
                signed,
                task,
                number,
                people::ProposalDecision::Reject { reason: b"No items".as_slice().into() },
                None,
            );
            assert!(matches!(
                reply,
                people::Reply::Outcome(people::Outcome::ProposalDecided {
                    choice: people::ProposalChoice::Rejected,
                    ..
                })
            ));
        } else {
            accepted(&run_decide(
                &mut world,
                signed,
                task,
                number,
                people::ProposalDecision::Accept,
                Some(people::Running::Stays),
            ));
        }
        assert!(proposal_inbox(&mut world).needs.is_empty());
        world.broken();
    }
}

#[test]
fn a_project_member_proposes_and_owners_review_current_and_proposed_values() {
    let mut world = start(64);
    let owner = sign_in(&mut world);
    let (member, signed) = party(&mut world, b"member", Some((1, people::Role::Member)));
    let items = charter_and_template(&world);
    let proposal = proposed(&propose(&mut world, signed, 1, 1, items));
    let document = proposal_document(&mut world, owner, member, proposal);
    let documents::ProposalAction::Settings { against, items, review: Some(review), made } = document.action else {
        panic!("reviewed settings")
    };
    assert_eq!((against, review.newest, made), (1, 1, None));
    assert_eq!(items.len(), 2);
    assert_eq!(review.items.len(), 2);
    let charter = review.items.get(0).expect("charter review");
    assert_eq!(charter.setting, settings::Setting::Charter(1));
    assert_eq!(charter.origin, Some((false, 1)));
    let Some(settings::Value::Charter(now)) = &charter.now else { panic!("current charter") };
    assert_eq!(now.policy.instructions.as_ref(), b"Answer within the granted authority.");
    let Some(settings::Value::Charter(after)) = &charter.would_be else { panic!("proposed charter") };
    assert_eq!(after.policy.instructions.as_ref(), b"Changed by review");
    let template = review.items.get(1).expect("template review");
    assert_eq!(template.edit, reads::RevisionEdit::Removed);
    assert!(template.now.is_some());
    assert!(template.would_be.is_none());
    let reads::Answer::Page(inbox) = read(
        &mut world,
        owner,
        reads::Read::Inbox {
            group: reads::Group::Proposals,
            filter: reads::Filter { project: None, goal: None },
            from: None,
        },
    ) else {
        panic!("owner inbox")
    };
    assert!(inbox.rows.iter().any(|row| matches!(row, reads::PageRow::Inbox(entry) if entry.object == documents::InboxObject::Proposal { proposer: people::Proposer::Party(member), number: proposal })));
    assert_eq!(frame(&mut world, owner).need_counts.proposals, 1);
    assert_eq!(proposal_document(&mut world, signed, member, proposal).number, proposal);
}
#[test]
fn accepting_settings_records_the_proposal_accepter_and_revision_atomically() {
    let mut world = start(64);
    let owner = sign_in(&mut world);
    let edit = template_edit(&world, 7);
    let proposal = proposed(&propose(&mut world, owner, 1, 1, Box::new([edit])));
    assert!(matches!(
        decide(&mut world, owner, 2, OWNER, proposal, people::ProposalDecision::Accept, Some(people::Running::Stays)),
        people::Reply::Outcome(people::Outcome::ProposalDecided { choice: people::ProposalChoice::Accepted, .. })
    ));
    let head = &revision(&world, 2, false).head;
    assert_eq!(
        head.proposal,
        Some((people::Proposer::Party(OWNER), u32::try_from(proposal).expect("proposal fits the settings key")))
    );
    assert_eq!(head.by, settings::Decider::Party(OWNER));
    assert!(!head.moved);
    assert!(!world.store.rows.contains_key(&proposal_set_key(OWNER, proposal)));
    assert_eq!(
        frame(&mut world, owner).projects.get(0).expect("project").templates.get(0).expect("chat template").grace,
        Some(skein_lib::Duration::from_secs(7))
    );
    let reads::Answer::Ended(history) =
        read(&mut world, owner, reads::Read::Proposal { proposer: people::Proposer::Party(OWNER), proposal })
    else {
        panic!("accepted history")
    };
    let reads::EndedValue::PersonProposal { proposal: row, .. } = history.value else { panic!("party history") };
    assert_eq!(
        row.state,
        tasks::PersonProposalState::SettingsAccepted { by: tasks::Party::Person(OWNER), revision: 2 }
    );
}
#[test]
fn rejected_settings_keep_the_reason_and_notify_the_project_member() {
    let mut world = start(64);
    let owner = sign_in(&mut world);
    let (member, signed) = party(&mut world, b"member", Some((1, people::Role::Member)));
    let item = template_edit(&world, 7);
    let proposal = proposed(&propose(&mut world, signed, 1, 1, Box::new([item])));
    decide(
        &mut world,
        owner,
        1,
        member,
        proposal,
        people::ProposalDecision::Reject { reason: b"Keep the current grace".as_slice().into() },
        None,
    );
    let reads::Answer::Ended(history) =
        read(&mut world, signed, reads::Read::Proposal { proposer: people::Proposer::Party(member), proposal })
    else {
        panic!("rejection history")
    };
    let reads::EndedValue::PersonProposal { proposal: row, .. } = history.value else { panic!("party history") };
    assert!(
        matches!(&row.state, tasks::PersonProposalState::Rejected { reason, .. } if reason.as_ref() == b"Keep the current grace")
    );
    assert!(updates(&mut world, signed).iter().any(|entry| entry.object
        == documents::InboxObject::Proposal { proposer: people::Proposer::Party(member), number: proposal }));
    assert!(!world.store.rows.contains_key(&proposal_set_key(member, proposal)));
    let item = template_edit(&world, 8);
    let withdrawn = proposed(&propose(&mut world, signed, 2, 1, Box::new([item])));
    assert!(matches!(
        ask(&mut world, signed, 3, people::Ask::Withdraw { project: 0, proposal: withdrawn }),
        people::Reply::Outcome(people::Outcome::Withdrawn { proposal }) if proposal == withdrawn
    ));
    assert!(!world.store.rows.contains_key(&proposal_set_key(member, withdrawn)));
    assert!(updates(&mut world, signed).iter().any(|entry| entry.object
        == documents::InboxObject::Proposal { proposer: people::Proposer::Party(member), number: withdrawn }));
}
#[test]
fn an_acceptance_refused_as_full_keeps_the_proposal_and_set_pending() {
    for running in [people::Running::Stays, people::Running::Moves] {
        let (config, limits) = full_version_configuration();
        let mut world = World::new(Setup {
            configuration: Some(config),
            limits,
            boot: jig_core_world::world::Boot::Empty,
            ..Setup::default()
        });
        world.client_peer = Some(ClientPeer::default());
        let owner = sign_in(&mut world);
        world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
        let chat = fixtures::chat(&world, 1, b"live task".as_slice().into());
        let people::Reply::Outcome(people::Outcome::Created { first, .. }) = ask(&mut world, owner, 1, chat) else {
            panic!("live agent")
        };
        assert!(matches!(fixtures::live_row(&world, first).executor, tasks::Executor::Agent { charter: 1 }));
        let mut items = removing_charter(&world).into_vec();
        items.push(settings::Item { setting: settings::Setting::Charter(2), edit: settings::Edit::Removed });
        assert!(
            items
                .iter()
                .any(|item| item.setting == settings::Setting::Charter(1) && item.edit == settings::Edit::Removed)
        );
        let proposal = proposed(&propose(&mut world, owner, 2, 1, items.into_boxed_slice()));
        for request in [3, 4] {
            let before_rows = world.store.rows.clone();
            let before_task = fixtures::live_row(&world, first).clone();
            let before_settings = settings_rows(&world);
            let before_review = proposal_document(&mut world, owner, OWNER, proposal).action;
            let assignments = world.assignments.clone();
            let cancellations = world.cancellations.clone();
            assert_eq!(
                decide(&mut world, owner, request, OWNER, proposal, people::ProposalDecision::Accept, Some(running)),
                people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Full {
                    revision: 1,
                    tasks: Box::new([first]),
                    more: 0
                }))
            );
            for (key, row) in &before_rows {
                let mut expected = row.clone();
                if let root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(header))) = &mut expected {
                    header.commits += 1; // The normal keyed refusal itself commits once.
                }
                assert_eq!(world.store.rows.get(key), Some(&expected), "refusal changed durable row {key:?}");
            }
            let added: Vec<_> = world.store.rows.keys().filter(|key| !before_rows.contains_key(*key)).collect();
            assert_eq!(added.len(), 1, "only the keyed refusal is added");
            assert!(matches!(added[0], root::Key::Core(core::Key::People(people::Key::Answer(_)))));
            assert_eq!(settings_rows(&world), before_settings);
            assert_eq!(fixtures::live_row(&world, first), &before_task);
            assert_eq!(world.assignments, assignments);
            assert_eq!(world.cancellations, cancellations);
            assert_eq!(proposal_document(&mut world, owner, OWNER, proposal).action, before_review);
            assert!(world.store.rows.contains_key(&proposal_set_key(OWNER, proposal)));
            assert_eq!(inventory(&mut world, owner).pending.len(), 1);
            restart(&mut world, full_version_configuration().0);
            assert_eq!(proposal_document(&mut world, owner, OWNER, proposal).action, before_review);
        }
    }
}
#[test]
fn accepting_or_changing_settings_withdraws_stale_proposals_and_proposers_hear() {
    let mut world = start(64);
    let owner = sign_in(&mut world);
    let (member, signed) = party(&mut world, b"member", Some((1, people::Role::Member)));
    let first = template_edit(&world, 7);
    let second = template_edit(&world, 8);
    let accepted = proposed(&propose(&mut world, owner, 1, 1, Box::new([first])));
    let stale = proposed(&propose(&mut world, signed, 1, 1, Box::new([second])));
    decide(&mut world, owner, 2, OWNER, accepted, people::ProposalDecision::Accept, Some(people::Running::Stays));
    let entries = updates(&mut world, signed);
    assert!(entries.iter().any(|entry| entry.object
        == documents::InboxObject::Proposal { proposer: people::Proposer::Party(member), number: stale }));
    let reads::Answer::Ended(history) =
        read(&mut world, signed, reads::Read::Proposal { proposer: people::Proposer::Party(member), proposal: stale })
    else {
        panic!("withdrawn history")
    };
    let reads::EndedValue::PersonProposal { proposal: row, decision: Some(decision) } = history.value else {
        panic!("withdrawn party proposal")
    };
    assert!(matches!(row.state, tasks::PersonProposalState::Withdrawn { .. }));
    assert_eq!(decision.choice, people::ProposalChoice::Withdrawn);
    let again = template_edit(&world, 9);
    let stale = proposed(&propose(&mut world, signed, 2, 2, Box::new([again])));
    let direct = template_edit(&world, 10);
    changed(&change(&mut world, owner, 3, 2, Box::new([direct])), 3);
    assert!(updates(&mut world, signed).iter().any(|entry| entry.object
        == documents::InboxObject::Proposal { proposer: people::Proposer::Party(member), number: stale }));
    assert!(inventory(&mut world, owner).pending.is_empty());
}
#[test]
fn pending_reviews_survive_restart_and_a_release_withdraws_them() {
    let mut world = start(64);
    let owner = sign_in(&mut world);
    let edit = template_edit(&world, 7);
    let proposal = proposed(&propose(&mut world, owner, 1, 1, Box::new([edit])));
    let before = proposal_document(&mut world, owner, OWNER, proposal).action;
    restart(&mut world, configuration().0);
    let owner = sign_in(&mut world);
    assert_eq!(proposal_document(&mut world, owner, OWNER, proposal).action, before);
    let mut config = configuration().0;
    config.core.defaults.charters[0].policy.instructions = b"A new release default".as_slice().into();
    restart(&mut world, config);
    let owner = sign_in(&mut world);
    assert_eq!(revision(&world, 2, false).head.by, settings::Decider::Release);
    assert!(!world.store.rows.contains_key(&proposal_set_key(OWNER, proposal)));
    assert!(inventory(&mut world, owner).pending.is_empty());
    assert!(updates(&mut world, owner).iter().any(|entry| entry.object
        == documents::InboxObject::Proposal { proposer: people::Proposer::Party(OWNER), number: proposal }));
    for erase_set in [true, false] {
        let mut inconsistent = start(64);
        let owner = sign_in(&mut inconsistent);
        let item = template_edit(&inconsistent, 7);
        let proposal = proposed(&propose(&mut inconsistent, owner, 1, 1, Box::new([item])));
        let missing = if erase_set {
            proposal_set_key(OWNER, proposal)
        } else {
            root::Key::Core(core::Key::Tasks(tasks::Key::PersonProposal(proposal)))
        };
        let root::Key::Core(target) = missing.clone() else { panic!("core proposal target") };
        let record = if erase_set {
            core::Key::Tasks(tasks::Key::PersonProposal(proposal))
        } else {
            let root::Key::Core(key) = proposal_set_key(OWNER, proposal) else { panic!("settings proposal key") };
            key
        };
        let removed = inconsistent.erase(missing).expect("one side of the durable pair");
        inconsistent.restart_with(configuration().0);
        assert_eq!(
            inconsistent.domain.start_failure().map(|(_, failure)| failure),
            Some(core::start::Failure::Link { record, linked: core::Linked::Record(target) })
        );
        assert_eq!(inconsistent.domain.start_phase(), Some(core::start::Phase::Link));
        assert!(!inconsistent.domain.ready(), "missing set: {erase_set}");
        inconsistent.seed(Box::new([removed]));
        inconsistent.restart_with(configuration().0);
        assert!(inconsistent.domain.ready());
    }
}
#[test]
fn the_pending_settings_bound_admits_two_and_names_the_thirds_refusal() {
    const PENDING: u32 = 2;
    let (config, mut limits) = configuration();
    limits.core.settings.proposals = PENDING;
    peers::journal(&mut limits);
    let mut world = World::new(Setup {
        configuration: Some(config),
        limits,
        boot: jig_core_world::world::Boot::Empty,
        ..Setup::default()
    });
    world.client_peer = Some(ClientPeer::default());
    let owner = sign_in(&mut world);
    let (_, first) = party(&mut world, b"first", Some((1, people::Role::Member)));
    let (_, second) = party(&mut world, b"second", Some((1, people::Role::Member)));
    let item = template_edit(&world, 7);
    proposed(&propose(&mut world, owner, 1, 1, Box::new([item.clone()])));
    proposed(&propose(&mut world, first, 1, 1, Box::new([item.clone()])));
    assert_eq!(inventory(&mut world, owner).pending.len(), PENDING);
    assert_eq!(
        propose(&mut world, second, 1, 1, Box::new([item])),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Limit {
            which: people::LimitKind::SettingsProposals
        }))
    );
}
#[test]
fn entrance_refusals_keep_nothing_and_only_deployment_owners_accept() {
    use settings::{Fault, Setting};
    let mut world = start(64);
    let owner = sign_in(&mut world);
    let (project_owner, signed) = party(&mut world, b"project owner", Some((1, people::Role::Owner)));
    let (_, outsider) = party(&mut world, b"outsider", None);
    let before = settings_rows(&world);
    let item = template_edit(&world, 7);
    let cases: [(u8, u32, Box<[settings::Item]>, people::Refusal); 3] = [
        (1, 0, Box::new([item.clone()]), people::Refusal::Stale { newest: 1 }),
        (
            2,
            1,
            Box::new([settings::Item { setting: settings::Setting::Rules, edit: settings::Edit::Reset }]),
            people::Refusal::NotWellFormed { at: 0, why: people::Malformed::SettingsUnchanged },
        ),
        (
            3,
            1,
            Box::new([settings::Item { setting: settings::Setting::Rules, edit: settings::Edit::Removed }]),
            people::Refusal::NotWellFormed {
                at: 0,
                why: people::Malformed::Settings { setting: Setting::Rules, fault: Fault::Shape, detail: None },
            },
        ),
    ];
    for (key, against, items, why) in cases {
        assert_eq!(
            propose(&mut world, owner, key, against, items),
            people::Reply::Outcome(people::Outcome::Refused(why))
        );
    }
    assert_eq!(
        propose(&mut world, outsider, 1, 1, Box::new([item.clone()])),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role { held: None }))
    );
    assert_eq!(settings_rows(&world), before);
    let proposal = proposed(&propose(&mut world, signed, 1, 1, Box::new([item])));
    assert_eq!(
        decide(
            &mut world,
            signed,
            2,
            project_owner,
            proposal,
            people::ProposalDecision::Accept,
            Some(people::Running::Stays)
        ),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role { held: None }))
    );
    assert_eq!(
        decide(&mut world, owner, 4, project_owner, proposal, people::ProposalDecision::Accept, None),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::Running
        }))
    );
    assert_eq!(inventory(&mut world, owner).pending.len(), 1);
}

#[test]
fn settings_proposal_reads_hide_pending_and_retained_values_from_project_only_parties() {
    let mut world = start(64);
    let owner = sign_in(&mut world);
    let (member, signed) = party(&mut world, b"proposer", Some((1, people::Role::Member)));
    let (_, outsider) = party(&mut world, b"project-owner", Some((1, people::Role::Owner)));
    let item = template_edit(&world, 7);
    let proposal = proposed(&propose(&mut world, signed, 1, 1, Box::new([item])));
    let point = reads::Read::Proposal { proposer: people::Proposer::Party(member), proposal };
    assert_eq!(read(&mut world, outsider, point.clone()), reads::Answer::None(reads::Why::Unknown));
    decide(
        &mut world,
        owner,
        1,
        member,
        proposal,
        people::ProposalDecision::Reject { reason: b"Keep the default".as_slice().into() },
        None,
    );
    assert_eq!(read(&mut world, outsider, point.clone()), reads::Answer::None(reads::Why::Unknown));
    assert!(matches!(read(&mut world, owner, point.clone()), reads::Answer::Ended(_)));
    assert!(matches!(read(&mut world, signed, point.clone()), reads::Answer::Ended(_)));
    assert_eq!(
        decide(&mut world, owner, 2, member, proposal, people::ProposalDecision::Accept, None),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::Running
        }))
    );
    restart(&mut world, configuration().0);
    let (_, outsider) = party(&mut world, b"project-owner", None);
    assert_eq!(read(&mut world, outsider, point), reads::Answer::None(reads::Why::Unknown));
}

#[test]
fn ordinary_party_proposal_history_stays_private_to_its_proposer() {
    let mut world = start(64);
    let owner = sign_in(&mut world);
    let (_, other) = party(&mut world, b"other-owner", Some((1, people::Role::Owner)));
    let people::Ask::Create { project, mut batch } = fixtures::chat(&world, 1, b"proposed chat".as_slice().into())
    else {
        panic!("chat batch")
    };
    batch[0].budget = people::Budget::Named;
    batch[0].authority.spend = 20;
    let people::Reply::Outcome(people::Outcome::Proposed { proposal, .. }) = ask(
        &mut world,
        owner,
        1,
        people::Ask::Propose { project, action: people::Action::Batch(batch), reason: b"Review".as_slice().into() },
    ) else {
        panic!("ordinary proposal")
    };
    ask(
        &mut world,
        owner,
        2,
        people::Ask::DecideProposal {
            project,
            proposer: people::Proposer::Party(OWNER),
            proposal,
            decision: people::ProposalDecision::Reject { reason: b"Keep it".as_slice().into() },
            running: None,
        },
    );
    let point = reads::Read::Proposal { proposer: people::Proposer::Party(OWNER), proposal };
    assert_eq!(read(&mut world, other, point.clone()), reads::Answer::None(reads::Why::Unknown));
    assert!(matches!(read(&mut world, owner, point), reads::Answer::Ended(_)));
    assert_eq!(
        ask(
            &mut world,
            owner,
            3,
            people::Ask::DecideProposal {
                project,
                proposer: people::Proposer::Party(OWNER),
                proposal,
                decision: people::ProposalDecision::Accept,
                running: Some(people::Running::Stays),
            }
        ),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::Running
        }))
    );
}

fn full_version_configuration() -> (root::Config, root::Limits) {
    let (mut config, mut limits) = configuration();
    let mut second = config.core.defaults.charters[0].clone();
    second.number = 2;
    config.core.defaults.charters = Box::new([config.core.defaults.charters[0].clone(), second]);
    limits.core.settings.versions = 1;
    peers::journal(&mut limits);
    (config, limits)
}

#[test]
fn proposed_settings_read_whole_set_removed_and_reset_with_the_reviews_audience() {
    for edit in [reads::RevisionEdit::Set, reads::RevisionEdit::Removed, reads::RevisionEdit::Reset] {
        let mut world = start(64);
        let owner = sign_in(&mut world);
        let (member, signed) = party(&mut world, b"proposer", Some((1, people::Role::Member)));
        let (_, outsider) = party(&mut world, b"project owner", Some((1, people::Role::Owner)));
        let setting = if edit == reads::RevisionEdit::Removed {
            settings::Setting::Template(1)
        } else {
            settings::Setting::Charter(1)
        };
        let default = world.domain.settings().charter(1).expect("default").clone();
        let mut proposed_charter = default.clone();
        proposed_charter.policy.instructions = b"A charter's proposed instructions, read whole.".as_slice().into();
        let value = settings::Value::Charter(Box::new(proposed_charter));
        let (proposed_edit, expected) = match edit {
            reads::RevisionEdit::Set => (settings::Edit::Set(value.clone()), Some(value)),
            reads::RevisionEdit::Removed => (settings::Edit::Removed, None),
            reads::RevisionEdit::Reset => {
                changed(
                    &change(
                        &mut world,
                        owner,
                        1,
                        1,
                        Box::new([settings::Item { setting, edit: settings::Edit::Set(value) }]),
                    ),
                    2,
                );
                (settings::Edit::Reset, Some(settings::Value::Charter(Box::new(default))))
            }
        };
        let newest = world.domain.settings().newest();
        let proposal = proposed(&propose(
            &mut world,
            signed,
            2,
            newest,
            Box::new([settings::Item { setting, edit: proposed_edit }]),
        ));
        let point =
            reads::Read::Setting { setting, revision: 0, proposal: Some((people::Proposer::Party(member), proposal)) };
        for reader in [owner, signed] {
            let review = proposal_document(&mut world, reader, member, proposal);
            let documents::ProposalAction::Settings { review: Some(review), .. } = review.action else {
                panic!("pending review")
            };
            assert_eq!(review.items.get(0).expect("review item").would_be, expected);
            let answer = read(&mut world, reader, point.clone());
            match &expected {
                Some(value) => assert!(
                    matches!(answer, reads::Answer::Setting(document) if document.value.as_ref() == Some(value))
                ),
                None => assert_eq!(answer, reads::Answer::None(reads::Why::Unknown)),
            }
        }
        let review = reads::Read::Proposal { proposer: people::Proposer::Party(member), proposal };
        assert_eq!(read(&mut world, outsider, point.clone()), read(&mut world, outsider, review));
        assert_eq!(
            read(
                &mut world,
                owner,
                reads::Read::Setting {
                    setting,
                    revision: newest,
                    proposal: Some((people::Proposer::Party(member), proposal)),
                }
            ),
            reads::Answer::None(reads::Why::Malformed)
        );
        assert_eq!(
            read(
                &mut world,
                owner,
                reads::Read::Setting {
                    setting: settings::Setting::Template(999),
                    revision: 0,
                    proposal: Some((people::Proposer::Party(member), proposal)),
                }
            ),
            reads::Answer::None(reads::Why::Unknown)
        );
        decide(
            &mut world,
            owner,
            3,
            member,
            proposal,
            people::ProposalDecision::Reject { reason: b"Keep the current value".as_slice().into() },
            None,
        );
        assert_eq!(read(&mut world, owner, point), reads::Answer::None(reads::Why::Unknown));
    }
}

#[test]
fn accepted_and_rejected_settings_decisions_replay_and_fresh_keys_get_the_first_decision() {
    for accepted in [true, false] {
        let mut world = start(64);
        let owner = sign_in(&mut world);
        let item = template_edit(&world, 7);
        let proposal = proposed(&propose(&mut world, owner, 1, 1, Box::new([item])));
        let choice = if accepted {
            people::ProposalDecision::Accept
        } else {
            people::ProposalDecision::Reject { reason: b"Keep it".as_slice().into() }
        };
        let running = if accepted { Some(people::Running::Stays) } else { None };
        let first = decide(&mut world, owner, 2, OWNER, proposal, choice.clone(), running);
        assert!(matches!(first, people::Reply::Outcome(people::Outcome::ProposalDecided { .. })));
        let before = settings_rows(&world);
        assert_eq!(decide(&mut world, owner, 2, OWNER, proposal, choice, running), first);
        let mut repeated = first.clone();
        let people::Reply::Outcome(people::Outcome::ProposalDecided { repeated: marker, .. }) = &mut repeated else {
            panic!("decision")
        };
        *marker = true;
        assert_eq!(
            decide(
                &mut world,
                owner,
                3,
                OWNER,
                proposal,
                people::ProposalDecision::Accept,
                Some(people::Running::Stays)
            ),
            repeated
        );
        assert_eq!(
            decide(
                &mut world,
                owner,
                4,
                OWNER,
                proposal,
                people::ProposalDecision::Reject { reason: b"A new reason".as_slice().into() },
                None
            ),
            repeated
        );
        assert_eq!(settings_rows(&world), before);
    }
}

fn task_proposal_history(world: &World, task: u64) -> Vec<tasks::History> {
    world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::History(history))) if history.task == task => {
                Some(history.clone())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_runs_fresh_proposal_replaces_its_own_that_waits() {
    let (mut world, signed, task) = proposing_run(true);
    let first = run_charter(&world, b"First instructions");
    let old = run_proposed(&run_propose(&mut world, 1, Box::new([first])));
    let before = fixtures::live_row(&world, task).clone();
    let item = run_charter(&world, b"Replacement instructions");
    let fresh = run_proposed(&run_propose(&mut world, 1, Box::new([item.clone()])));
    assert_ne!(fresh, old);
    let row = fixtures::live_row(&world, task);
    assert_eq!(row.revision, before.revision + 2);
    assert_eq!(row.proposal.as_ref().expect("new pending proposal").number, fresh);
    assert_eq!(row.inbox, before.inbox, "replacement sends no decision message");
    let history = task_proposal_history(&world, task);
    let withdrawn = history.iter().find(|row| row.revision == before.revision + 1).expect("withdrawn history");
    assert_eq!(withdrawn.by, tasks::Party::Task(task));
    assert_eq!(withdrawn.change, tasks::Change::ProposalWithdrawn);
    let proposal = withdrawn.proposal.as_ref().expect("withdrawn proposal");
    assert_eq!(proposal.number, old);
    assert_eq!(proposal.state, tasks::ProposalState::Withdrawn);
    let proposed = history.iter().find(|row| row.revision == before.revision + 2).expect("new proposal history");
    assert_eq!(proposed.change, tasks::Change::Proposed);
    assert_eq!(proposed.proposal.as_ref().expect("new proposal").number, fresh);
    assert!(!world.store.rows.contains_key(&task_set(task, old)));
    let root::Record::Core(core::Record::Core(core::CoreRecord::Settings(settings::Record::Proposal {
        value, ..
    }))) = &world.store.rows[&task_set(task, fresh)]
    else {
        panic!("new set")
    };
    assert_eq!(value.items.as_ref(), &[item]);
    assert_eq!(inventory(&mut world, signed).pending.len(), 1);
    let commit = world
        .commits
        .iter()
        .find(|writes| writes.contains(&root::Write::Erase(task_set(task, old))))
        .expect("withdrawal commit");
    assert!(
        commit.iter().any(|write| matches!(write, root::Write::Save(row) if row.key() == task_set(task, fresh))),
        "withdrawal and new set commit together"
    );
    world.park_for_message(0);
    world.close();
}

#[test]
fn a_refused_fresh_proposal_leaves_its_own_waiting() {
    let (mut world, signed, task) = proposing_run(true);
    let item = run_charter(&world, b"First pending instructions");
    let old = run_proposed(&run_propose(&mut world, 1, Box::new([item])));
    for refusal in 0..4 {
        if refusal == 3 {
            let item = run_charter(&world, b"Another party's instructions");
            proposed(&propose(&mut world, signed, 104, 1, Box::new([item])));
        }
        let before = settings_rows(&world);
        let history = task_proposal_history(&world, task);
        let row = fixtures::live_row(&world, task).clone();
        let mut item = run_charter(&world, b"Refused replacement");
        let part = match refusal {
            0 => run_propose(&mut world, 0, Box::new([item])),
            1 => {
                let settings::Edit::Set(settings::Value::Charter(charter)) = &mut item.edit else { panic!("charter") };
                charter.number = 0;
                run_propose(&mut world, 1, Box::new([item]))
            }
            2 => {
                let reason =
                    review_reason(usize::try_from(run_configuration().1.core.people.words).expect("words bound") + 1);
                run_propose_reason(&mut world, 1, Box::new([item]), &reason)
            }
            3 => run_propose(&mut world, 1, Box::new([item])),
            _ => unreachable!("four refusal cases"),
        };
        match refusal {
            0 => assert_eq!(part, core::CallPart::SettingsRefused(people::Refusal::Stale { newest: 1 })),
            1 => assert!(matches!(part, core::CallPart::SettingsRefused(people::Refusal::NotWellFormed { .. }))),
            2 => assert_eq!(
                part,
                core::CallPart::SettingsRefused(people::Refusal::Limit { which: people::LimitKind::Words })
            ),
            3 => assert_eq!(
                part,
                core::CallPart::SettingsRefused(people::Refusal::Limit { which: people::LimitKind::SettingsProposals })
            ),
            _ => unreachable!("four refusal cases"),
        }
        assert_eq!(settings_rows(&world), before, "refusal {refusal} retains the set");
        assert_eq!(task_proposal_history(&world, task), history, "refusal {refusal} writes no history");
        assert_eq!(fixtures::live_row(&world, task).proposal, row.proposal);
        assert_eq!(fixtures::live_row(&world, task).revision, row.revision);
        assert_eq!(fixtures::live_row(&world, task).inbox, row.inbox);
        assert_eq!(fixtures::live_row(&world, task).proposal.as_ref().expect("old pending").number, old);
    }
    world.park_for_message(0);
    world.close();
}

#[test]
fn an_owners_decision_of_a_replaced_proposal_changes_nothing() {
    let (mut world, signed, task) = proposing_run(true);
    let item = run_charter(&world, b"First instructions");
    let old = run_proposed(&run_propose(&mut world, 1, Box::new([item])));
    let item = run_charter(&world, b"New instructions");
    let fresh = run_proposed(&run_propose(&mut world, 1, Box::new([item])));
    let before = settings_rows(&world);
    let history = task_proposal_history(&world, task);
    assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::Core(core::CoreKey::ProposalDecision {
        proposer: people::Proposer::Task(task),
        proposal: old
    }))));
    assert_eq!(
        run_decide(&mut world, signed, task, old, people::ProposalDecision::Accept, None),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Unknown))
    );
    assert_eq!(world.domain.settings().newest(), 1);
    assert_eq!(settings_rows(&world), before);
    assert_eq!(task_proposal_history(&world, task), history);
    accepted(&ask(
        &mut world,
        signed,
        103,
        people::Ask::DecideProposal {
            project: people::DEPLOYMENT,
            proposer: people::Proposer::Task(task),
            proposal: fresh,
            decision: people::ProposalDecision::Accept,
            running: Some(people::Running::Stays),
        },
    ));
    assert_eq!(world.domain.settings().newest(), 2);
    assert_eq!(
        revision(&world, 2, false).head.proposal,
        Some((people::Proposer::Task(task), u32::try_from(fresh).expect("proposal number")))
    );
    let message = run_outcome(&world, task, fresh).number;
    world.turn(1, 0, Some(message), b"The new settings were accepted");
    world.park_for_message(0);
    world.close();
}

#[test]
fn a_replaced_proposal_stays_gone_after_a_restart() {
    let (mut world, signed, task) = proposing_run(true);
    let item = run_charter(&world, b"First instructions");
    let old = run_proposed(&run_propose(&mut world, 1, Box::new([item])));
    let item = run_charter(&world, b"Durable new instructions");
    let fresh = run_proposed(&run_propose(&mut world, 1, Box::new([item])));
    let before = settings_rows(&world);
    let history = task_proposal_history(&world, task);
    restart(&mut world, run_configuration().0);
    assert!(world.domain.ready(), "restart passed Link");
    assert_eq!(settings_rows(&world), before);
    assert_eq!(task_proposal_history(&world, task), history);
    assert!(!world.store.rows.contains_key(&task_set(task, old)));
    assert!(world.store.rows.contains_key(&task_set(task, fresh)));
    assert_eq!(fixtures::live_row(&world, task).proposal.as_ref().expect("one pending").number, fresh);
    assert_eq!(inventory(&mut world, signed).pending.len(), 1);
    assert!(
        !fixtures::live_row(&world, task)
            .inbox
            .iter()
            .any(|word| matches!(word.kind, tasks::MessageKind::SettingsDecision { proposal, .. } if proposal == old))
    );
    world.close();
}

fn run_proposed(part: &core::CallPart) -> u64 {
    let core::CallPart::Proposed { proposal, holder } = part else { panic!("proposed: {part:?}") };
    assert_eq!(
        *holder,
        tasks::ProposalHolder::Policy { project: people::DEPLOYMENT, kind: tasks::ProposalKind::Settings }
    );
    *proposal
}
fn run_decide(
    world: &mut World,
    signed: u64,
    task: u64,
    proposal: u64,
    decision: people::ProposalDecision,
    running: Option<people::Running>,
) -> people::Reply {
    ask(
        world,
        signed,
        102,
        people::Ask::DecideProposal {
            project: people::DEPLOYMENT,
            proposer: people::Proposer::Task(task),
            proposal,
            decision,
            running,
        },
    )
}
fn task_set(task: u64, proposal: u64) -> root::Key {
    key(settings::Key::Proposal {
        proposer: people::Proposer::Task(task),
        proposal: u32::try_from(proposal).expect("proposal key"),
    })
}
fn run_outcome(world: &World, task: u64, proposal: u64) -> &tasks::Word {
    fixtures::live_row(world, task).inbox.iter().find(|word| matches!(word.kind, tasks::MessageKind::SettingsDecision { proposal: number, .. } if number == proposal)).expect("settings outcome in the proposer's inbox")
}
// Keep the review inline bound below the tool input bound for a hex-encoded set.
const CHARTER_REVIEW_BYTES: u32 = 128;

fn large_run_charter(world: &mut World, signed: u64, number: u32) -> settings::Item {
    let mut charter = world.domain.settings().charter(1).expect("fixture charter").clone();
    charter.number = number;
    let setting = settings::Setting::Charter(number);
    if number != 1 {
        let newest = world.domain.settings().newest();
        let item =
            settings::Item { setting, edit: settings::Edit::Set(settings::Value::Charter(Box::new(charter.clone()))) };
        changed(&change(world, signed, 107, newest, Box::new([item])), newest + 1);
    }
    let inline = usize::try_from(CHARTER_REVIEW_BYTES).expect("inline bound");
    let mut instructions = charter.policy.instructions.repeat(inline / charter.policy.instructions.len() + 1);
    instructions.extend_from_slice(b" Keep change reports short.");
    charter.policy.instructions = instructions.into_boxed_slice();
    settings::Item { setting, edit: settings::Edit::Set(settings::Value::Charter(Box::new(charter))) }
}

fn run_charter_review(world: &mut World, signed: u64, task: u64, proposal: u64, expected: &settings::Value) {
    let reads::Answer::Document(document) =
        read(world, signed, reads::Read::Proposal { proposer: people::Proposer::Task(task), proposal })
    else {
        panic!("pending run proposal")
    };
    let documents::Document::Proposal(document) = *document else { panic!("proposal review") };
    let documents::ProposalAction::Settings { review: Some(review), .. } = document.action else {
        panic!("settings review")
    };
    let settings::Value::Charter(charter) = expected else { panic!("proposed charter") };
    assert!(len_u64(charter.policy.instructions.len()) > review.value_bytes);
    let item = review.items.get(0).expect("reviewed charter");
    assert_eq!(item.setting, settings::Setting::Charter(charter.number));
    assert_eq!(item.would_be.as_ref(), Some(expected));
}

fn run_setting_readers(world: &mut World, owner: u64, point: &reads::Read, task: u64, proposal: u64) {
    for (subject, role) in [
        (b"deployment member".as_slice(), Some((people::DEPLOYMENT, people::Role::Member))),
        (b"project owner".as_slice(), Some((1, people::Role::Owner))),
        (b"outsider".as_slice(), None),
    ] {
        let (_, signed) = party(world, subject, role);
        let review = reads::Read::Proposal { proposer: people::Proposer::Task(task), proposal };
        assert_eq!(read(world, signed, review), reads::Answer::None(reads::Why::Unknown));
        assert_eq!(read(world, signed, point.clone()), reads::Answer::None(reads::Why::Unknown));
    }
    let mut wrong = point.clone();
    let reads::Read::Setting { proposal: Some((_, number)), .. } = &mut wrong else { panic!("proposal read") };
    *number += 1;
    assert_eq!(read(world, owner, wrong), reads::Answer::None(reads::Why::Unknown));
}

#[test]
fn a_runs_large_proposed_charter_is_read_whole_by_its_reviewer() {
    for (number, decision, running) in [
        (1, people::ProposalDecision::Accept, Some(people::Running::Stays)),
        (2, people::ProposalDecision::Reject { reason: b"Keep the current charter".as_slice().into() }, None),
    ] {
        let (mut world, signed, task) = proposing_run(true);
        let item = large_run_charter(&mut world, signed, number);
        let settings::Edit::Set(expected) = item.edit.clone() else { panic!("set charter") };
        let newest = world.domain.settings().newest();
        let proposal = run_proposed(&run_propose(&mut world, newest, Box::new([item])));
        let (config, mut limits) = run_configuration();
        limits.core.documents.text_bytes = CHARTER_REVIEW_BYTES;
        world.restart_with_limits(config, limits);
        run_charter_review(&mut world, signed, task, proposal, &expected);
        let point = reads::Read::Setting {
            setting: settings::Setting::Charter(number),
            revision: 0,
            proposal: Some((people::Proposer::Task(task), proposal)),
        };
        let answer = read(&mut world, signed, point.clone());
        let reads::Answer::Setting(document) = answer else { panic!("whole proposed charter: {answer:?}") };
        assert_eq!(document.value.as_ref(), Some(&expected), "complete instructions including the added sentence");
        run_setting_readers(&mut world, signed, &point, task, proposal);
        let reply = run_decide(&mut world, signed, task, proposal, decision, running);
        assert!(matches!(reply, people::Reply::Outcome(people::Outcome::ProposalDecided { .. })), "{reply:?}");
        assert_eq!(read(&mut world, signed, point), reads::Answer::None(reads::Why::Unknown));
    }
}

#[test]
fn a_run_proposes_settings_and_an_owner_accepts_moving_its_work() {
    let (mut world, signed, task) = proposing_run(true);
    let item = run_charter(&world, b"Reviewed instructions");
    let proposal = run_proposed(&run_propose(&mut world, 1, Box::new([item])));
    assert!(world.store.rows.contains_key(&task_set(task, proposal)));
    let before_replay = settings_rows(&world);
    let key = world.answers.last().expect("typed answer").1;
    world.send(root::Event::Core(core::Event::NamedAction {
        to: skein_lib::ReplyTo::new(skein_lib::Token::new(39_001)),
        key,
        action: core::NamedAction::ProposeSettings {
            against: 1,
            items: Box::new([run_charter(&world, b"Replay cannot replace this set")]),
            reason: b"again".as_slice().into(),
        },
    }));
    assert_eq!(run_proposed(&world.answers.last().expect("typed answer").2), proposal);
    assert_eq!(settings_rows(&world), before_replay);
    let reads::Answer::Document(document) =
        read(&mut world, signed, reads::Read::Proposal { proposer: people::Proposer::Task(task), proposal })
    else {
        panic!("proposal document")
    };
    let documents::Document::Proposal(document) = *document else { panic!("proposal review") };
    let documents::ProposalAction::Settings { against, review: Some(review), .. } = document.action else {
        panic!("settings review")
    };
    assert_eq!((against, review.newest), (1, 1));
    let reviewed = review.items.get(0).expect("reviewed charter");
    assert_eq!(reviewed.origin, Some((false, 1)));
    let Some(settings::Value::Charter(now)) = &reviewed.now else { panic!("current value") };
    let Some(settings::Value::Charter(after)) = &reviewed.would_be else { panic!("proposed value") };
    assert_eq!(now.policy.instructions.as_ref(), b"Answer within the granted authority.");
    assert_eq!(after.policy.instructions.as_ref(), b"Reviewed instructions");
    let reads::Answer::Page(inbox) = read(
        &mut world,
        signed,
        reads::Read::Inbox {
            group: reads::Group::Proposals,
            filter: reads::Filter { project: None, goal: None },
            from: None,
        },
    ) else {
        panic!("owner inbox")
    };
    assert!(inbox.rows.iter().any(|row| matches!(row, reads::PageRow::Inbox(entry) if entry.object == documents::InboxObject::Proposal { proposer: people::Proposer::Task(task), number: proposal })));
    accepted(&run_decide(
        &mut world,
        signed,
        task,
        proposal,
        people::ProposalDecision::Accept,
        Some(people::Running::Moves),
    ));
    let head = &revision(&world, 2, false).head;
    assert_eq!(head.proposal, Some((people::Proposer::Task(task), u32::try_from(proposal).expect("proposal key"))));
    assert_eq!(head.by, settings::Decider::Party(OWNER));
    assert!(head.moved);
    assert_eq!(fixtures::live_row(&world, task).settings.task, 2);
    assert_eq!(
        run_outcome(&world, task, proposal).kind,
        tasks::MessageKind::SettingsDecision { proposal, outcome: tasks::SettingsOutcome::Accepted { revision: 2 } }
    );
    assert!(!world.store.rows.contains_key(&task_set(task, proposal)));
    // Engine 6 step 3 ends the engine claim, releasing its run revision (engine 17.4); task revision stays held.
    world.broken();
}
#[test]
fn a_runs_settings_proposal_rejected_reaches_its_inbox_with_the_reason() {
    let (mut world, signed, task) = proposing_run(true);
    let item = run_charter(&world, b"Later instructions");
    let proposal = run_proposed(&run_propose(&mut world, 1, Box::new([item])));
    let reason = b"Keep the current instructions";
    let reply = run_decide(
        &mut world,
        signed,
        task,
        proposal,
        people::ProposalDecision::Reject { reason: reason.as_slice().into() },
        None,
    );
    assert!(
        matches!(
            reply,
            people::Reply::Outcome(people::Outcome::ProposalDecided { choice: people::ProposalChoice::Rejected, .. })
        ),
        "{reply:?}"
    );
    let outcome = run_outcome(&world, task, proposal);
    assert_eq!(
        outcome.kind,
        tasks::MessageKind::SettingsDecision { proposal, outcome: tasks::SettingsOutcome::Rejected }
    );
    assert_eq!(outcome.words.as_ref(), reason);
    assert!(!world.store.rows.contains_key(&task_set(task, proposal)));
    assert_eq!(world.domain.settings().newest(), 1);
    // Engine 6 step 3 settles the dead engine/in-process claim: phase, retry, reserve and activity change durably.
    world.broken();
}
#[test]
fn a_runs_stale_settings_proposal_is_withdrawn_and_it_hears_the_newest() {
    let (mut world, signed, task) = proposing_run(true);
    let item = run_charter(&world, b"Proposed instructions");
    let proposal = run_proposed(&run_propose(&mut world, 1, Box::new([item])));
    let direct = run_charter(&world, b"Direct owner instructions");
    changed(&change(&mut world, signed, 103, 1, Box::new([direct])), 2);
    assert_eq!(
        run_outcome(&world, task, proposal).kind,
        tasks::MessageKind::SettingsDecision { proposal, outcome: tasks::SettingsOutcome::Stale { newest: 2 } }
    );
    assert!(!world.store.rows.contains_key(&task_set(task, proposal)));
    assert!(fixtures::live_row(&world, task).proposal.is_none());
    let item = run_charter(&world, b"Proposed again against newest");
    let next = run_proposed(&run_propose(&mut world, 2, Box::new([item])));
    assert_ne!(next, proposal);
    assert!(world.store.rows.contains_key(&task_set(task, next)));
    // Engine 6 step 3 settles the dead engine/in-process claim: phase, retry, reserve and activity change durably.
    world.broken();
}
#[test]
fn a_run_without_the_settings_family_is_denied_and_may_propose_nothing() {
    let (mut world, _, task) = proposing_run(false);
    let before = settings_rows(&world);
    let item = run_charter(&world, b"Unauthorized proposal");
    assert!(matches!(run_propose(&mut world, 1, Box::new([item])), core::CallPart::ToolDenied { .. }));
    assert_eq!(settings_rows(&world), before);
    assert!(fixtures::live_row(&world, task).proposal.is_none());
    // Engine 6 step 3 settles the dead engine/in-process claim: phase, retry, reserve and activity change durably.
    world.broken();
}
#[test]
fn a_runs_settings_proposal_refused_at_the_entrance_keeps_nothing() {
    let (mut world, signed, task) = proposing_run(true);
    let before = settings_rows(&world);
    let item = run_charter(&world, b"Stale basis");
    assert_eq!(
        run_propose(&mut world, 0, Box::new([item])),
        core::CallPart::SettingsRefused(people::Refusal::Stale { newest: 1 })
    );
    assert_eq!(settings_rows(&world), before);
    let mut item = run_charter(&world, b"Malformed proposal");
    let settings::Edit::Set(settings::Value::Charter(charter)) = &mut item.edit else { panic!("charter") };
    charter.number = 0;
    assert!(matches!(
        run_propose(&mut world, 1, Box::new([item])),
        core::CallPart::SettingsRefused(people::Refusal::NotWellFormed { .. })
    ));
    assert_eq!(settings_rows(&world), before);
    let first = run_charter(&world, b"First pending");
    proposed(&propose(&mut world, signed, 104, 1, Box::new([first])));
    let (_, member) = party(&mut world, b"another proposer", Some((1, people::Role::Member)));
    let second = run_charter(&world, b"Second pending");
    proposed(&propose(&mut world, member, 105, 1, Box::new([second])));
    let before = settings_rows(&world);
    let item = run_charter(&world, b"Past the limit");
    assert_eq!(
        run_propose(&mut world, 1, Box::new([item])),
        core::CallPart::SettingsRefused(people::Refusal::Limit { which: people::LimitKind::SettingsProposals })
    );
    assert_eq!(settings_rows(&world), before);
    assert!(fixtures::live_row(&world, task).proposal.is_none());
    // Engine 6 step 3 settles the dead engine/in-process claim: phase, retry, reserve and activity change durably.
    world.broken();
}
#[test]
fn a_runs_pending_settings_proposal_survives_a_restart_and_is_accepted() {
    let (mut world, signed, task) = proposing_run(true);
    let item = run_charter(&world, b"Durable proposed instructions");
    let proposal = run_proposed(&run_propose(&mut world, 1, Box::new([item])));
    restart(&mut world, run_configuration().0);
    assert!(world.store.rows.contains_key(&task_set(task, proposal)));
    accepted(&run_decide(
        &mut world,
        signed,
        task,
        proposal,
        people::ProposalDecision::Accept,
        Some(people::Running::Stays),
    ));
    assert_eq!(
        run_outcome(&world, task, proposal).kind,
        tasks::MessageKind::SettingsDecision { proposal, outcome: tasks::SettingsOutcome::Accepted { revision: 2 } }
    );
    assert!(!world.store.rows.contains_key(&task_set(task, proposal)));
    restart(&mut world, run_configuration().0);
    assert_eq!(
        revision(&world, 2, false).head.proposal,
        Some((people::Proposer::Task(task), u32::try_from(proposal).expect("proposal key")))
    );
    assert_eq!(fixtures::live_row(&world, task).inbox.iter().filter(|word| matches!(word.kind, tasks::MessageKind::SettingsDecision { proposal: number, .. } if number == proposal)).count(), 1);
}

#[test]
fn exhausted_stale_outcome_numbers_refuse_before_changing_settings() {
    let (mut world, signed, task) = proposing_run(true);
    let item = run_charter(&world, b"Pending instructions");
    let proposal = run_proposed(&run_propose(&mut world, 1, Box::new([item])));
    let header_key = root::Key::Core(core::Key::Core(core::CoreKey::Deployment));
    let mut header = world.store.rows[&header_key].clone();
    let root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(numbers))) = &mut header else {
        panic!("deployment header")
    };
    numbers.messages = u64::MAX;
    world.domain.core_mut().counters = core::Counters::new(*numbers);
    let before = settings_rows(&world);
    let direct = run_charter(&world, b"Direct change");
    assert_eq!(
        change(&mut world, signed, 106, 1, Box::new([direct])),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Limit {
            which: people::LimitKind::Arithmetic
        }))
    );
    assert_eq!(settings_rows(&world), before);
    assert!(fixtures::live_row(&world, task).proposal.is_some());
    assert!(world.store.rows.contains_key(&task_set(task, proposal)));
    let (mut changed, _) = run_configuration();
    changed.core.defaults.charters[0].policy.instructions = b"Release default".as_slice().into();
    world.restart_with(changed);
    assert!(!world.domain.ready(), "startup also refuses an unnumberable stale outcome");
    assert!(world.stopped);
    assert_eq!(settings_rows(&world), before);
    // The fixture deliberately exhausts message numbers and leaves start refused.
    world.broken();
}

// Synthetic reasons cover the reported sizes; they are not transcript quotations.
fn review_reason(bytes: usize) -> String {
    let mut reason = String::from("engine enforces: review every change. your agents follow: keep reports short. ");
    let effect = " Running work keeps its current instructions until an owner moves it.";
    reason.push_str(&"x".repeat(bytes - reason.len() - effect.len()));
    reason.push_str(effect);
    assert_eq!(reason.len(), bytes);
    reason
}
fn owner_reason(world: &mut World, signed: u64, proposer: people::Proposer, proposal: u64) -> documents::Text {
    let reads::Answer::Page(inbox) = read(
        world,
        signed,
        reads::Read::Inbox {
            group: reads::Group::Proposals,
            filter: reads::Filter { project: None, goal: None },
            from: None,
        },
    ) else {
        panic!("owner inbox")
    };
    assert!(inbox.rows.iter().any(|row| matches!(row, reads::PageRow::Inbox(entry)
        if entry.object == documents::InboxObject::Proposal { proposer, number: proposal })));
    let reads::Answer::Document(document) = read(world, signed, reads::Read::Proposal { proposer, proposal }) else {
        panic!("pending proposal")
    };
    let documents::Document::Proposal(value) = *document else { panic!("proposal document") };
    value.reason
}
#[test]
fn a_runs_settings_proposal_reason_is_read_whole_after_restart() {
    for bytes in [900, 649, usize::try_from(run_configuration().1.core.people.words).expect("reason bound")] {
        let (mut world, signed, task) = proposing_run(true);
        let reason = review_reason(bytes);
        let item = run_charter(&world, b"Keep reports short and review every change.");
        let proposal = run_proposed(&run_propose_reason(&mut world, 1, Box::new([item]), &reason));
        for restarted in [false, true] {
            if restarted {
                let (config, mut limits) = run_configuration();
                limits.core.documents.text_bytes = 128;
                world.restart_with_limits(config, limits);
                assert!(world.domain.ready());
            }
            assert_eq!(
                fixtures::live_row(&world, task).proposal.as_ref().expect("stored proposal").reason.as_ref(),
                reason.as_bytes()
            );
            let shown = owner_reason(&mut world, signed, people::Proposer::Task(task), proposal);
            assert_eq!(shown.bytes.len(), reason.len(), "whole settings reason");
            assert_eq!(shown.bytes.as_ref(), reason.as_bytes());
            assert!(!shown.cut);
        }
    }
}
fn party_reason_proposal(world: &mut World, signed: u64, item: settings::Item, reason: &str) -> u64 {
    world.send(root::Event::Core(core::Event::ProposeSettings {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(SEED)),
        sign_in: signed,
        key: [1; 16],
        digest: [1; 32],
        change: Box::new(settings::Proposed { against: 1, items: Box::new([item]), reason: reason.as_bytes().into() }),
    }));
    let (_, root::Delivery::Core(core::Held::PeopleReply { reply, .. })) =
        world.client_peer.as_mut().expect("client").delivered.pop().expect("proposal reply")
    else {
        panic!("proposal reply")
    };
    proposed(&reply)
}
#[test]
fn a_partys_settings_proposal_reason_is_read_whole_after_restart() {
    for bytes in [900, 649, usize::try_from(configuration().1.core.people.words).expect("reason bound")] {
        let mut world = start(64);
        let signed = sign_in(&mut world);
        let reason = review_reason(bytes);
        let item = run_charter(&world, b"Keep reports short and review every change.");
        let proposal = party_reason_proposal(&mut world, signed, item, &reason);
        for restarted in [false, true] {
            if restarted {
                let (config, mut limits) = configuration();
                limits.core.documents.text_bytes = 128;
                world.restart_with_limits(config, limits);
                assert!(world.domain.ready());
            }
            let stored = world
                .store
                .rows
                .values()
                .find_map(|row| match row {
                    root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(row)))
                        if row.proposer == OWNER && row.number == proposal =>
                    {
                        Some(row)
                    }
                    _ => None,
                })
                .expect("stored party proposal");
            assert_eq!(stored.reason.as_ref(), reason.as_bytes());
            let shown = owner_reason(&mut world, signed, people::Proposer::Party(OWNER), proposal);
            assert_eq!(shown.bytes.len(), reason.len(), "whole settings reason");
            assert_eq!(shown.bytes.as_ref(), reason.as_bytes());
            assert!(!shown.cut);
        }
    }
}

#[test]
fn a_runs_settings_reason_past_people_words_is_refused_durably() {
    let (mut world, _, task) = proposing_run(true);
    let reason = review_reason(usize::try_from(run_configuration().1.core.people.words).expect("reason bound") + 1);
    let item = run_charter(&world, b"Short reports");
    let before = settings_rows(&world);
    let part = run_propose_reason(&mut world, 1, Box::new([item]), &reason);
    assert_eq!(part, core::CallPart::SettingsRefused(people::Refusal::Limit { which: people::LimitKind::Words }));
    assert_eq!(settings_rows(&world), before);
    assert!(fixtures::live_row(&world, task).proposal.is_none());
    let rendered = jig_protocol_tools::render(&part);
    assert!(rendered.error);
    assert_eq!(
        rendered.text.as_ref(),
        b"Settings refused: reason exceeds people.words; shorten the reason and propose again"
    );
    let key = world.answers.last().expect("refusal answer").1;
    let address = root::Key::Core(core::Key::Core(core::CoreKey::Call(key)));
    let stored = world.store.rows.get(&address).expect("stored refusal").clone();
    let root::Record::Core(core::Record::Core(core::CoreRecord::Call(call))) = &stored else { panic!("call record") };
    assert_eq!(call.part, part);
    restart(&mut world, run_configuration().0);
    assert_eq!(world.store.rows.get(&address), Some(&stored));
    assert!(fixtures::live_row(&world, task).proposal.is_none());
}

#[test]
fn a_release_withdraws_a_parked_runs_proposal_and_tells_its_task() {
    let (mut world, _, task) = proposing_run(true);
    let item = run_charter(&world, b"Pending instructions");
    let proposal = run_proposed(&run_propose(&mut world, 1, Box::new([item])));
    world.park_for_message(0);
    seed_release_accounts(&mut world);
    let before = world.assignments.len();
    restart(&mut world, released_run_configuration());
    assert_eq!(revision(&world, 2, false).head.by, settings::Decider::Release);
    assert!(!world.store.rows.contains_key(&task_set(task, proposal)));
    assert_eq!(
        run_outcome(&world, task, proposal).kind,
        tasks::MessageKind::SettingsDecision { proposal, outcome: tasks::SettingsOutcome::Stale { newest: 2 } }
    );
    resume_release_host(&mut world);
    assert_eq!(world.assignments.len(), before + 1, "one next run after the start");
    let assignment = world.assigned.as_ref().expect("proposer resumes");
    assert_eq!(assignment.task, task);
    assert_eq!(
        assignment
            .messages
            .iter()
            .filter(|word| matches!(word.kind,
        tasks::MessageKind::SettingsDecision { proposal: number, .. } if number == proposal))
            .count(),
        1
    );
    restart(&mut world, released_run_configuration());
    assert_eq!(world.domain.settings().newest(), 2);
    assert_eq!(
        fixtures::live_row(&world, task)
            .inbox
            .iter()
            .filter(|word| matches!(word.kind,
        tasks::MessageKind::SettingsDecision { proposal: number, .. } if number == proposal))
            .count(),
        1
    );
    world.close();
}

fn released_run_configuration() -> root::Config {
    let mut config = run_configuration().0;
    config.core.defaults.charters[0].policy.instructions = b"Released default".as_slice().into();
    config
}

fn seed_release_accounts(world: &mut World) {
    world.seed(jig_test_profile::fixtures::account_rows(&run_configuration().0.core.settings.llm_providers));
}

fn resume_release_host(world: &mut World) {
    release_host(world, Box::new([]));
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 2, budget: 100, next: None }));
}

fn release_host(world: &mut World, hosting: Box<[Hosting]>) {
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Hello {
        worker: 0,
        channel: skein_lib::Token::new(7),
        hello: Hello {
            jobs: Box::default(),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: skein_lib::Duration::from_millis(1),
            slots: 1,
            workstreams: Box::new([]),
            hosting,
        },
    })));
}

#[test]
fn a_release_withdraws_the_proposal_of_a_run_still_claimed() {
    let (mut world, _, task) = proposing_run(true);
    let item = run_charter(&world, b"Pending instructions");
    let proposal = run_proposed(&run_propose(&mut world, 1, Box::new([item])));
    let assignment = world.assigned.as_ref().expect("claimed proposer");
    let attempt = assignment.attempt;
    let claim = fixtures::live_row(&world, task).phase.clone();
    let assignments = world.assignments.clone();
    let inbound = world.inbound.clone();
    seed_release_accounts(&mut world);
    world.hold_restore_pages = true;
    world.hold_switch = true;
    let mut config = released_run_configuration();
    config.core.workers_in_process = false;
    world.restart_with(config);
    release_host(
        &mut world,
        Box::new([Hosting {
            run: skein_lib::Token::new(task),
            attempt: skein_lib::Token::new(attempt),
            phase: Phase::Active,
        }]),
    );
    world.finish_restore_pages();
    assert_eq!(world.domain.start_phase(), Some(core::start::Phase::Switch));
    assert_eq!(world.assignments, assignments);
    assert_eq!(world.inbound, inbound, "no relay before durable Wires");
    assert!(!world.store.rows.contains_key(&task_set(task, proposal)));
    let outcome = run_outcome(&world, task, proposal).clone();
    assert_eq!(
        outcome.kind,
        tasks::MessageKind::SettingsDecision { proposal, outcome: tasks::SettingsOutcome::Stale { newest: 2 } }
    );
    world.finish_switch();
    assert!(world.domain.ready());
    assert_eq!(fixtures::live_row(&world, task).phase, claim);
    assert_eq!(world.assignments, assignments, "adoption keeps the same attempt");
    assert_eq!(world.inbound, inbound, "the restored word waits for the next turn");
    world.turn(1, 0, None, b"Read the release's decision");
    assert_eq!(world.inbound.iter().filter(|word| word.number == outcome.number).count(), 1);
    world.turn(2, 0, Some(outcome.number), b"The pending proposal is stale");
    assert!(!fixtures::live_row(&world, task).inbox.iter().any(|word| word.number == outcome.number));
    fixtures::finish_chat(&mut world, 0, b"Done after adoption");
    let mut config = released_run_configuration();
    config.core.workers_in_process = false;
    restart(&mut world, config);
    assert_eq!(world.domain.settings().newest(), 2);
    world.close();
}

#[test]
fn a_release_with_a_held_revision_withdraws_a_runs_proposal() {
    let (mut world, signed, task) = proposing_run(true);
    let item = template_edit(&world, 7);
    changed(&change(&mut world, signed, 103, 1, Box::new([item])), 2);
    assert_eq!(fixtures::live_row(&world, task).settings.task, 1);
    assert_eq!(
        world.assigned.as_ref().expect("old goal keeps its run").run.policy.instructions.as_ref(),
        b"Answer within the granted authority."
    );
    let item = run_charter(&world, b"Pending against revision two");
    let proposal = run_proposed(&run_propose(&mut world, 2, Box::new([item])));
    world.park_for_message(0);
    seed_release_accounts(&mut world);
    restart(&mut world, released_run_configuration());
    assert_eq!(revision(&world, 3, false).head.by, settings::Decider::Release);
    assert!(!world.store.rows.contains_key(&task_set(task, proposal)));
    assert_eq!(
        run_outcome(&world, task, proposal).kind,
        tasks::MessageKind::SettingsDecision { proposal, outcome: tasks::SettingsOutcome::Stale { newest: 3 } }
    );
    assert_eq!(
        inventory(&mut world, signed).held.iter().map(|row| (row.head.number, row.holders)).collect::<Vec<_>>(),
        [(1, 1), (3, 0)]
    );
    resume_release_host(&mut world);
    assert_eq!(
        world.assigned.as_ref().expect("held goal resumes").run.policy.instructions.as_ref(),
        b"Answer within the granted authority."
    );
    assert_eq!(fixtures::live_row(&world, task).settings.run, Some(1));
    let rows = settings_rows(&world);
    restart(&mut world, released_run_configuration());
    assert_eq!(world.domain.settings().newest(), 3);
    assert_eq!(settings_rows(&world), rows);
    assert_eq!(
        fixtures::live_row(&world, task)
            .inbox
            .iter()
            .filter(|word| matches!(word.kind,
        tasks::MessageKind::SettingsDecision { proposal: number, .. } if number == proposal))
            .count(),
        1
    );
    world.close();
}

#[test]
fn a_crash_after_the_releases_commit_leaves_the_withdrawal_done() {
    let (mut world, _, task) = proposing_run(true);
    let item = run_charter(&world, b"Pending instructions");
    let proposal = run_proposed(&run_propose(&mut world, 1, Box::new([item])));
    world.park_for_message(0);
    seed_release_accounts(&mut world);
    let assignments = world.assignments.clone();
    world.hold_switch = true;
    world.restart_with(released_run_configuration());
    assert_eq!(world.domain.start_phase(), Some(core::start::Phase::Switch));
    assert_eq!(world.assignments, assignments, "no activation before durable Wires");
    assert_eq!(revision(&world, 2, false).head.by, settings::Decider::Release);
    assert!(!world.store.rows.contains_key(&task_set(task, proposal)));
    let outcome = run_outcome(&world, task, proposal).clone();
    assert_eq!(
        outcome.kind,
        tasks::MessageKind::SettingsDecision { proposal, outcome: tasks::SettingsOutcome::Stale { newest: 2 } }
    );
    let release = world
        .commits
        .iter()
        .find(|writes| {
            writes.iter().any(|write| {
                matches!(write,
        root::Write::Save(row) if row.key() == key(settings::Key::Held(2)))
            })
        })
        .expect("release decision");
    assert!(release.contains(&root::Write::Erase(task_set(task, proposal))), "withdrawal shares the release commit");
    assert!(
        release.iter().any(|write| matches!(write,
        root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))))
            if row.number == task && row.inbox.iter().any(|word| word == &outcome))),
        "the same decision saves the proposer's word"
    );
    world.hold_switch = false;
    restart(&mut world, released_run_configuration());
    assert_eq!(world.domain.settings().newest(), 2);
    assert_eq!(run_outcome(&world, task, proposal), &outcome);
    assert_eq!(fixtures::live_row(&world, task).inbox.iter().filter(|word| word.number == outcome.number).count(), 1);
    let header = &world.store.rows[&root::Key::Core(core::Key::Core(core::CoreKey::Deployment))];
    let root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(numbers))) = header else {
        panic!("deployment header")
    };
    assert!(numbers.messages >= outcome.number);
    resume_release_host(&mut world);
    assert_eq!(world.assignments.len(), assignments.len() + 1, "the crash leaves exactly one next run due");
    assert_eq!(world.assigned.as_ref().expect("next run").task, task);
    world.turn(1, 0, Some(outcome.number), b"The proposal was withdrawn");
    world.park_for_message(0);
    world.close();
}

// One document admitted, two carried by the proposal codec's Small profile.
const DOCUMENT_LIMIT: u32 = 1;
fn document_run() -> (World, u64, u64) {
    document_run_with_limit(DOCUMENT_LIMIT)
}
fn document_run_with_limit(bound: u32) -> (World, u64, u64) {
    let mut limits = run_configuration().1;
    limits.core.tasks.charter_documents = bound;
    let (mut world, signed, task) = settings_run_with_limits(true, None, limits);
    // The fake wire accepts Small's bounds independently of the core's admission bounds.
    let (config, mut wire) = run_configuration();
    wire.core.authority.segments = jig_protocol_store::settings_v1::CEILINGS.document_name;
    wire.core.authority.segment_bytes = jig_protocol_store::settings_v1::CEILINGS.segment_bytes;
    world.settings_input_limits(wire.core, &config.core.settings.models);
    (world, signed, task)
}
fn document_item(world: &World, count: usize) -> settings::Item {
    let mut item = run_charter(world, b"Read the rules");
    let settings::Edit::Set(settings::Value::Charter(charter)) = &mut item.edit else { panic!("charter") };
    charter.documents = vec![charter_rule_document(); count].into_boxed_slice();
    item
}
#[test]
fn run_proposal_document_refusal_names_member_and_bound() {
    let (mut world, _, task) = document_run();
    let before = settings_rows(&world);
    let item = document_item(&world, 2);
    let answer = run_propose(&mut world, 1, Box::new([item]));
    assert_eq!(
        answer,
        document_refusal(people::SettingDetail {
            member: people::SettingMember::Documents,
            document: 0,
            segment: 0,
            rule: people::SettingRule::Maximum,
            supplied: 2,
            bound: DOCUMENT_LIMIT,
        })
    );
    assert_eq!(settings_rows(&world), before);
    assert!(fixtures::live_row(&world, task).proposal.is_none());
    world.broken();
}

fn document_refusal(detail: people::SettingDetail) -> core::CallPart {
    core::CallPart::SettingsRefused(people::Refusal::NotWellFormed {
        at: 0,
        why: people::Malformed::Settings {
            setting: settings::Setting::Charter(1),
            fault: settings::Fault::Shape,
            detail: Some(detail),
        },
    })
}
#[test]
fn run_proposal_documents_at_the_bound_are_admitted() {
    let (mut world, _, task) = document_run();
    let item = document_item(&world, usize::try_from(DOCUMENT_LIMIT).unwrap());
    let proposal = run_proposed(&run_propose(&mut world, 1, Box::new([item])));
    assert_eq!(fixtures::live_row(&world, task).proposal.as_ref().expect("pending proposal").number, proposal);
    world.broken();
}
#[test]
fn run_proposal_document_members_keep_the_failed_rule() {
    use people::{SettingDetail as D, SettingMember as M, SettingRule as R};
    let (_, bounds) = run_configuration();
    let document = charter_rule_document();
    let mut path = document.clone();
    path.path = Box::default();
    let mut heading = document.clone();
    heading.heading = Some(Box::default());
    let mut names = document.clone();
    names.name.segments =
        vec![b"rules".as_slice().into(); usize::try_from(bounds.core.authority.segments + 1).unwrap()].into();
    let mut segment = document.clone();
    segment.name.segments =
        Box::new([vec![b'n'; usize::try_from(bounds.core.authority.segment_bytes + 1).unwrap()].into()]);
    let mut connector = document;
    connector.connector = u16::MAX;
    for (document, member, rule, supplied, bound) in [
        (path, M::Path, R::Minimum, 0, 1),
        (heading, M::Heading, R::Minimum, 0, 1),
        (names, M::NameSegments, R::Maximum, bounds.core.authority.segments + 1, bounds.core.authority.segments),
        (
            segment,
            M::NameSegmentBytes,
            R::Maximum,
            bounds.core.authority.segment_bytes + 1,
            bounds.core.authority.segment_bytes,
        ),
        (connector, M::Connector, R::Registered, u32::from(u16::MAX), 0),
    ] {
        let (mut world, _, task) = document_run_with_limit(2);
        let before = settings_rows(&world);
        let mut item = document_item(&world, 1);
        let settings::Edit::Set(settings::Value::Charter(charter)) = &mut item.edit else { panic!("charter") };
        charter.documents = Box::new([charter_rule_document(), document]);
        assert_eq!(
            run_propose(&mut world, 1, Box::new([item])),
            document_refusal(D { member, document: 1, segment: 0, rule, supplied, bound })
        );
        assert_eq!(settings_rows(&world), before);
        assert!(fixtures::live_row(&world, task).proposal.is_none());
        world.broken();
    }
}

fn charter_rule_document() -> core::Document {
    core::Document {
        connector: 1,
        name: jig_domain_authority::Name { segments: Box::new([b"rules".as_slice().into()]) },
        path: b"RULES.md".as_slice().into(),
        heading: None,
    }
}

#[test]
fn overlays_are_reviewed_accepted_rejected_and_withdrawn_when_stale() {
    let mut world = start(64);
    let owner = sign_in(&mut world);
    let overlay = settings::Overlay { instructions: b"Proposed words".as_slice().into(), ..settings::Overlay::empty() };
    let value = settings::Value::Overlay(Box::new(overlay));
    let item = settings::Item { setting: settings::Setting::Charter(1), edit: settings::Edit::Set(value.clone()) };
    let proposal = proposed(&propose(&mut world, owner, 1, 1, Box::new([item.clone()])));
    let documents::ProposalAction::Settings { review: Some(review), .. } =
        proposal_document(&mut world, owner, OWNER, proposal).action
    else {
        panic!("settings review")
    };
    let Some(settings::Value::Charter(would_be)) = &review.items.get(0).unwrap().would_be else {
        panic!("overlaid would-be charter")
    };
    assert_eq!(would_be.policy.instructions.as_ref(), b"Answer within the granted authority.\n\nProposed words");
    decide(
        &mut world,
        owner,
        2,
        OWNER,
        proposal,
        people::ProposalDecision::Reject { reason: b"Not yet".as_slice().into() },
        None,
    );
    assert!(!world.store.rows.contains_key(&proposal_set_key(OWNER, proposal)));
    assert_eq!(world.domain.settings().newest(), 1);
    let proposal = proposed(&propose(&mut world, owner, 3, 1, Box::new([item.clone()])));
    decide(&mut world, owner, 4, OWNER, proposal, people::ProposalDecision::Accept, Some(people::Running::Stays));
    assert_eq!(world.domain.settings().value(item.setting), Some(&value));
    assert!(matches!(
        change(&mut world, owner, 7, 2, Box::new([item.clone()])),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
            why: people::Malformed::SettingsUnchanged,
            ..
        }))
    ));
    let stale_item = settings::Item {
        setting: item.setting,
        edit: settings::Edit::Set(settings::Value::Overlay(Box::new(settings::Overlay {
            instructions: b"Later overlay".as_slice().into(),
            ..settings::Overlay::empty()
        }))),
    };
    let stale = proposed(&propose(&mut world, owner, 5, 2, Box::new([stale_item])));
    let edit = template_edit(&world, 7);
    changed(&change(&mut world, owner, 6, 2, Box::new([edit])), 3);
    assert!(!world.store.rows.contains_key(&proposal_set_key(OWNER, stale)));
    let reads::Answer::Ended(history) =
        read(&mut world, owner, reads::Read::Proposal { proposer: people::Proposer::Party(OWNER), proposal: stale })
    else {
        panic!("withdrawn history")
    };
    let reads::EndedValue::PersonProposal { proposal, .. } = history.value else { panic!("party proposal") };
    assert!(matches!(proposal.state, tasks::PersonProposalState::Withdrawn { .. }));
    restart(&mut world, configuration().0);
    assert_eq!(world.domain.settings().value(item.setting), Some(&value));
}
