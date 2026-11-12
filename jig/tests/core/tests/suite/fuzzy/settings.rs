//! Random settings decisions agree with their durable rows through cold restarts.

use crate::settings_support as support;
use jig_core_world::fixtures;
use jig_domain_core::{self as core, settings};
use jig_domain_link::Hello;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, Rng, length::len_u64};
use support::*;

#[test]
fn a_standard_profile_s_whole_room_of_answers_stays_under_the_heap_ceiling_and_restarts() {
    // Standard's whole room holds four times Small's retained answers under the heap ceiling.
    support::whole_room_of_answers(jig_protocol_config::Profile::Standard);
}

fn agree(world: &jig_core_world::world::World) {
    let current = world.domain.settings();
    let mut expected = std::collections::BTreeMap::new();
    let mut changes = std::collections::BTreeMap::new();
    for row in world.store.rows.values() {
        if let root::Record::Core(core::Record::Core(core::CoreRecord::Settings(row))) = row {
            match row {
                settings::Record::Application { .. } => unreachable!("an application's record: apps-store"),
                settings::Record::Default { setting, default } => {
                    expected.insert(*setting, default.value.clone());
                }
                settings::Record::Changed { setting, changed } => {
                    changes.insert(*setting, changed.value.clone());
                }
                settings::Record::Revision { .. }
                | settings::Record::Extent { .. }
                | settings::Record::Charter { .. }
                | settings::Record::Proposal { .. } => {}
            }
        }
    }
    let defaults = expected.clone();
    for (setting, value) in changes {
        match value {
            Some(value) => {
                expected.insert(setting, value);
            }
            None => {
                expected.remove(&setting);
            }
        }
    }
    assert_eq!(current.offered().as_slice(), expected.keys().copied().collect::<Vec<_>>());
    for (setting, value) in expected {
        assert_eq!(current.value(setting), Some(&value));
        if let settings::Value::Overlay(overlay) = &value {
            let settings::Setting::Charter(number) = setting else { panic!("overlay setting") };
            let settings::Value::Charter(default) = &defaults[&setting] else { panic!("overlay default") };
            let made = settings::overlaid(default, overlay);
            let catalogue: Vec<_> = current
                .offered()
                .iter()
                .filter_map(|setting| match current.value(*setting) {
                    Some(settings::Value::Model(model)) => Some(*model.clone()),
                    _ => None,
                })
                .collect();
            let resolved = settings::charters::resolved(&made, &catalogue, &world.env().limits.core)
                .expect("offered overlay resolves");
            assert_eq!(current.charter(number), Some(&resolved), "default with stored overlay");
        }
    }
    if let Some(assigned) = &world.assigned
        && let Some(row) = world.domain.core().tasks.task(assigned.task)
        && row.settings.run == Some(assigned.run.revision)
    {
        let tasks::Executor::Agent { charter } = row.executor else { panic!("agent assignment") };
        let version =
            world.domain.core().charter_at(charter, assigned.run.revision).expect("assigned revision is held");
        assert_eq!(assigned.run.policy.instructions, version.policy.instructions, "run reads its revision's version");
    }
    assert_eq!(revision(world, current.newest(), false).head, *current.newest_head());
    let mut holders = std::collections::BTreeMap::<u32, u32>::new();
    for row in world.store.rows.values() {
        if let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task))) = row {
            assert!(
                world.store.rows.contains_key(&key(settings::Key::Held(task.settings.task))),
                "live task {} lost revision {}",
                task.number,
                task.settings.task
            );
            *holders.entry(task.settings.task).or_default() += 1;
            if let Some(run) = task.settings.run {
                assert!(world.store.rows.contains_key(&key(settings::Key::Held(run))), "live run lost revision {run}");
                if run != task.settings.task {
                    *holders.entry(run).or_default() += 1;
                }
            }
        }
    }
    for row in world.store.rows.values() {
        if let root::Record::Core(core::Record::Core(core::CoreRecord::Settings(settings::Record::Revision {
            history: false,
            revision,
        }))) = row
        {
            assert!(
                revision.head.number == current.newest() || holders.contains_key(&revision.head.number),
                "held revision has no holder: {}",
                revision.head.number
            );
        }
    }
}
fn overlay_item(words: &[u8]) -> settings::Item {
    settings::Item {
        setting: settings::Setting::Charter(1),
        edit: settings::Edit::Set(settings::Value::Overlay(Box::new(settings::Overlay {
            instructions: words.into(),
            shell: Some(false),
            ..settings::Overlay::empty()
        }))),
    }
}
fn restart_default(world: &mut jig_core_world::world::World, rng: &mut Rng) {
    let mut config = configuration().0;
    config.core.defaults.charters[0].policy.instructions =
        format!("Release {}", rng.below(1_000_000)).into_bytes().into_boxed_slice();
    restart(world, config);
}

fn live_work(world: &mut jig_core_world::world::World, signed: u64) {
    // Two non-conversation goals retain their settings: one parked, one running.
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Hello {
        worker: 0,
        channel: skein_lib::Token::new(7),
        hello: Hello {
            jobs: Box::default(),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: Duration::from_millis(1),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
    for request in [2, 3] {
        let mut goal = fixtures::chat(world, 1, b"Keep working".as_slice().into());
        let people::Ask::Create { batch, .. } = &mut goal else { panic!("goal fixture") };
        batch[0].flags.conversation = false;
        batch[0].flags.listed = true;
        batch[0].budget = people::Budget::Named;
        batch[0].authority.spend = 40;
        let answer = ask(world, signed, request, goal);
        assert!(matches!(answer, people::Reply::Outcome(people::Outcome::Created { .. })), "{answer:?}");
        assert!(world.assigned.is_some());
        if request == 2 {
            world.end(tasks::End::Parked, 0);
        }
    }
}

fn draw_change(
    world: &mut jig_core_world::world::World,
    signed: u64,
    step: u8,
    seed: u64,
    operation: u64,
    rng: &mut Rng,
    reached: &mut [bool; 8],
) {
    reached[7] |= operation == 5;
    let previous = world.domain.settings().newest();
    let stale = rng.below(4) == 0;
    reached[0] |= stale;
    let against = previous - u32::from(stale);
    if operation < 3 {
        reached[usize::try_from(operation + 1).expect("three edits")] = true;
    }
    let item = match operation {
        0 => settings::Item { setting: settings::Setting::Template(1), edit: settings::Edit::Reset },
        1 | 3 => template_edit(world, 6 + rng.below(8)),
        5 => overlay_item(format!("overlay {seed} {step}").as_bytes()),
        _ => {
            let mut charter = world.domain.settings().charter(1).expect("settings fixture").clone();
            charter.policy.instructions = format!("random settings {seed} {step}").into_bytes().into_boxed_slice();
            settings::Item {
                setting: settings::Setting::Charter(1),
                edit: settings::Edit::Set(settings::Value::Charter(Box::new(charter))),
            }
        }
    };
    let before_rows = settings_rows(world);
    let before = world.domain.settings().value(item.setting).cloned();
    let answer = if operation == 3 {
        let old_holders = world.store.rows.values().any(|row| matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task))) if task.settings.task < previous || task.settings.run.is_some_and(|run| run < previous)));
        world.send(root::Event::Core(core::Event::ChangeSettings {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(SEED)),
            sign_in: signed,
            key: [step + 10; 16],
            digest: [step + 10; 32],
            change: Box::new(settings::Change {
                against,
                items: Box::new([item.clone()]),
                running: settings::Running::Moves,
            }),
        }));
        let (_, root::Delivery::Core(core::Held::PeopleReply { reply, .. })) =
            world.client_peer.as_mut().expect("client").delivered.pop().expect("settings reply")
        else {
            panic!("settings reply")
        };
        reached[5] |= old_holders && matches!(reply, people::Reply::Outcome(people::Outcome::SettingsChanged { .. }));
        reply
    } else {
        change(world, signed, step + 10, against, Box::new([item.clone()]))
    };
    if stale {
        assert_eq!(settings_rows(world), before_rows);
        assert_eq!(
            answer,
            people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Stale { newest: previous }))
        );
    } else if world.domain.settings().value(item.setting) == before.as_ref() {
        assert_eq!(
            answer,
            people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
                at: 0,
                why: people::Malformed::SettingsUnchanged
            }))
        );
    } else {
        changed(&answer, previous + 1);
    }
    agree(world);
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the test run selects replay seeds and their count from the environment")]
fn random_small_settings_changes_and_stale_requests_survive_restarts() {
    let count =
        std::env::var("JIG_CORE_SETTINGS_SEEDS").ok().map_or(24, |value| value.parse::<u64>().expect("seed count"));
    let mut coverage = [false; 8];
    for seed in 713..713 + count {
        let result = std::panic::catch_unwind(|| {
            let mut reached = [false; 8];
            let mut rng = Rng::new(seed);
            let mut world = start(2);
            let signed = sign_in(&mut world);
            assert_eq!(revision(&world, 1, false).head.by, settings::Decider::FirstStart);
            assert!(world.store.rows.contains_key(&key(settings::Key::Default(settings::Setting::Rules))));
            assert!(matches!(
                ask(&mut world, signed, 1, people::Ask::MakeProject { name: "Random project".into(), policy: 1 }),
                people::Reply::Outcome(people::Outcome::ProjectMade { .. })
            ));
            assert!(
                world
                    .store
                    .rows
                    .values()
                    .any(|value| *value == record(settings::Record::Extent { oldest: 0, keys: 1 }))
            );
            live_work(&mut world, signed);
            agree(&world);
            for step in 0..16_u8 {
                let previous = world.domain.settings().newest();
                let operation = rng.below(6);
                if operation == 4 {
                    let held: Vec<_> = world
                        .store
                        .rows
                        .values()
                        .filter_map(|row| match row {
                            root::Record::Core(core::Record::Core(core::CoreRecord::Settings(
                                settings::Record::Revision { history: false, revision },
                            ))) => Some(revision.head.number),
                            root::Record::Core(_) | root::Record::Connector { .. } => None,
                        })
                        .collect();
                    let from = held[usize::try_from(rng.below(len_u64(held.len()))).expect("held index")];
                    reached[6] |= from != previous;
                    let reply = ask(&mut world, signed, step + 10, people::Ask::MoveToNewest { revision: from });
                    assert!(
                        matches!(reply, people::Reply::Outcome(people::Outcome::MovedToNewest { .. })),
                        "{reply:?}"
                    );
                    agree(&world);
                } else {
                    draw_change(&mut world, signed, step, seed, operation, &mut rng, &mut reached);
                }
                if rng.below(4) == 0 {
                    reached[4] = true;
                    restart_default(&mut world, &mut rng);
                    agree(&world);
                }
            }
            let me = frame(&mut world, signed);
            assert_eq!(
                me.projects.get(0).expect("settings fixture").templates.get(0).expect("settings fixture").grace,
                world.domain.settings().template(1).expect("settings fixture").grace
            );
            assert!(
                world.domain.settings().template(1).expect("settings fixture").grace.expect("settings fixture")
                    >= Duration::from_secs(5)
            );
            restart(&mut world, configuration().0);
            agree(&world);
            reached
        });
        let Ok(reached) = result else { panic!("settings replay seed {seed}") };
        for at in 0..coverage.len() {
            coverage[at] |= reached[at];
        }
    }
    assert_eq!(
        coverage, [true; 8],
        "stale, reset, template, charter, restart, Moves with old holders MoveToNewest with old holders, and overlays all reached"
    );
}

fn pending_settings(world: &jig_core_world::world::World) -> Vec<(u64, u32, settings::Proposal)> {
    world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Core(core::CoreRecord::Settings(settings::Record::Proposal {
                proposer: people::Proposer::Party(person),
                proposal,
                value,
            }))) => Some((*person, *proposal, value.clone())),
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .collect()
}
fn proposal_sets_agree(world: &mut jig_core_world::world::World, signed: u64) {
    let sets = pending_settings(world);
    let pending = inventory(world, signed).pending;
    assert_eq!(pending.len(), u32::try_from(sets.len()).expect("bounded sets"));
    let task_rows: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::PersonProposal(row)))
                if row.action.is_settings() =>
            {
                Some((row.proposer, row.number, row.action.clone()))
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .collect();
    assert_eq!(sets.len(), task_rows.len());
    for (person, proposal, set) in sets {
        assert_eq!(set.against, world.domain.settings().newest());
        assert!(pending.iter().any(|row| row.proposer == people::Proposer::Party(person) && row.number == proposal));
        assert!(task_rows.iter().any(|(proposer, number, action)| *proposer == person
            && *number == u64::from(proposal)
            && *action
                == jig_domain_tasks::PersonAction::Settings {
                    against: set.against,
                    items: settings::proposal_heads(&set.items)
                }));
        let jig_domain_core::documents::ProposalAction::Settings { against, review: Some(review), .. } =
            proposal_document(world, signed, person, u64::from(proposal)).action
        else {
            panic!("pending settings review")
        };
        assert_eq!(against, set.against);
        assert_eq!(review.items.len(), u32::try_from(set.items.len()).expect("bounded items"));
        for (shown, item) in review.items.iter().zip(&set.items) {
            assert_eq!(shown.setting, item.setting);
            let made;
            let expected = match &item.edit {
                settings::Edit::Set(settings::Value::Overlay(overlay)) => {
                    let Some(settings::Value::Charter(default)) = world.domain.settings().default_value(item.setting)
                    else {
                        panic!("default")
                    };
                    made = settings::Value::Charter(Box::new(settings::overlaid(default, overlay)));
                    Some(&made)
                }
                settings::Edit::Set(value) => Some(value),
                settings::Edit::Removed => None,
                settings::Edit::Reset => world.domain.settings().default_value(item.setting),
            };
            assert_eq!(shown.would_be.as_ref(), expected);
        }
    }
}
#[test]
#[expect(clippy::disallowed_methods, reason = "the test run selects replay seeds and their count from the environment")]
fn random_settings_proposals_decisions_and_restarts_keep_all_three_owners_in_agreement() {
    const SEEDS: u64 = 8;
    const STEPS: u8 = 32;
    let mut coverage = [false; 6];
    let count =
        std::env::var("JIG_CORE_SETTINGS_SEEDS").ok().map_or(SEEDS, |value| value.parse::<u64>().expect("seed count"));
    let first = std::env::var("JIG_SETTINGS_PROPOSALS_SEED")
        .map_or(713, |value| value.parse::<u64>().expect("settings proposal seed"));
    for seed in first..first.checked_add(count).expect("settings seed range") {
        let result = std::panic::catch_unwind(|| {
            let mut reached = [false; 6];
            let mut rng = Rng::new(seed);
            let mut world = start(4);
            let signed = sign_in(&mut world);
            for step in 0..STEPS {
                let operation = usize::try_from(rng.below(6)).expect("six operations");
                reached[operation] = true;
                let pending = pending_settings(&world).first().cloned();
                let key = 20 + step;
                match operation {
                    0 => {
                        let item = if rng.below(2) == 0 {
                            overlay_item(format!("proposed overlay {seed} {step}").as_bytes())
                        } else {
                            template_edit(&world, 6 + rng.below(10))
                        };
                        let against = world.domain.settings().newest();
                        propose(&mut world, signed, key, against, Box::new([item]));
                    }
                    1 => {
                        if let Some((person, proposal, _)) = pending {
                            decide(
                                &mut world,
                                signed,
                                key,
                                person,
                                u64::from(proposal),
                                people::ProposalDecision::Accept,
                                Some(people::Running::Stays),
                            );
                        }
                    }
                    2 => {
                        if let Some((person, proposal, _)) = pending {
                            decide(
                                &mut world,
                                signed,
                                key,
                                person,
                                u64::from(proposal),
                                people::ProposalDecision::Reject { reason: b"random rejection".as_slice().into() },
                                None,
                            );
                        }
                    }
                    3 => {
                        if let Some((_, proposal, _)) = pending {
                            ask(
                                &mut world,
                                signed,
                                key,
                                people::Ask::Withdraw { project: 0, proposal: u64::from(proposal) },
                            );
                        }
                    }
                    4 => {
                        let item = template_edit(&world, 6 + rng.below(10));
                        let against = world.domain.settings().newest();
                        change(&mut world, signed, key, against, Box::new([item]));
                    }
                    5 => {
                        restart_default(&mut world, &mut rng);
                    }
                    _ => unreachable!(),
                }
                agree(&world);
                proposal_sets_agree(&mut world, signed);
            }
            reached
        });
        let Ok(reached) = result else { panic!("settings proposal replay seed {seed}") };
        for (covered, reached) in coverage.iter_mut().zip(reached) {
            *covered |= reached;
        }
    }
    assert_eq!(coverage, [true; 6]);
}
