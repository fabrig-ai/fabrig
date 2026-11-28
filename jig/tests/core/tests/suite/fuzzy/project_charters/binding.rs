//! Compare the effective executor and prepared words with an independent binding model.
use super::*;
use jig_domain_contract as contract;
use jig_domain_people as people;

const REQUESTED: tasks::Executor = tasks::Executor::Agent { charter: 1 };
const NAME: &[u8] = b"implementer";
const OTHER: &[u8] = b"careful";

#[derive(Clone, Copy, Debug)]
enum Making {
    Party,
    Agent,
    Procedure,
    Recurring,
    Proposal,
}

fn world(seed: u64) -> World {
    let (mut config, mut limits) = jig_core_world::faults::plan_fixture(seed);
    let requested = &mut config.core.defaults.charters[0];
    requested.name = Some(NAME.into());
    requested.policy.resume = false;
    let mut other = requested.clone();
    other.number = 2;
    other.name = Some(OTHER.into());
    other.policy.instructions = b"Deployment alt.".as_slice().into();
    config.core.defaults.charters = Box::new([requested.clone(), other]);
    for uses in [
        &mut config.core.defaults.rules.ceiling.delegation.kinds,
        &mut config.core.seed_projects[0].policy.policy.ceiling.delegation.kinds,
        &mut config.core.seed_projects[0].policy.policy.roles[0].authority.delegation.kinds,
    ] {
        let mut kinds = uses.to_vec();
        kinds.push(jig_domain_authority::Executor::Charter(2));
        *uses = kinds.into_boxed_slice();
    }
    if !config.core.owners.iter().any(|owner| owner.project == 0) {
        let mut deployment = config.core.owners[0].clone();
        deployment.project = 0;
        let mut owners = config.core.owners.into_vec();
        owners.push(deployment);
        config.core.owners = owners.into_boxed_slice();
        limits.core.people.initial_owners = limits.core.people.initial_owners.max(2);
    }
    limits.core.tasks.charters = 2;
    limits.core.run_bytes = 4096;
    limits.core.tasks.executor_kinds = limits.core.tasks.executor_kinds.max(3);
    limits.core.authority.executors = limits.core.authority.executors.max(3);
    limits.core.documents.executors = limits.core.documents.executors.max(3);
    World::new(Setup { seed, configuration: Some(config), limits, ..Setup::default() })
}

fn new(template: &tasks::TaskRecord, number: u64, executor: tasks::Executor) -> tasks::New {
    let funder = tasks::Funder::Period { project: PROJECT, period: 1 };
    let mut authority = template.authority.clone();
    authority.budget.spend = 15;
    if executor == REQUESTED {
        authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    }
    tasks::New {
        number,
        project: PROJECT,
        label: 0,
        flags: tasks::Flags::PLAIN,
        executor,
        charter_scope: tasks::CharterScope::Project,
        llm: None,
        spec: template.spec.clone(),
        contract: tasks::Contract::Report { words: 128 },
        authority,
        deadline: tasks::Deadline::Named,
        numbers: tasks::Numbers { budget: 15, spent: 0, spent_below: 0, reserved: 0 },
        funder,
        dependencies: Box::new([]),
        holdings: Box::new([]),
        wake: template.wake,
        recurring: None,
        tracked: None,
    }
}

fn create(world: &mut World, making: Making) -> u64 {
    let parent = world.task();
    let template = fixtures::live_row(world, parent).clone();
    let mut delegate = fixtures::member(world, REQUESTED, b"delegate");
    delegate.authority.budget.spend = 10;
    delegate.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    match making {
        Making::Party => {
            let people::Ask::Create { mut batch, .. } = world.initial_ask.clone() else { panic!("initial batch") };
            batch[0].flags.conversation = false;
            batch[0].budget = people::Budget::Named;
            batch[0].authority.spend = 15;
            batch[0].authority.delegation = people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
            fixtures::ask(world, 81, people::Ask::Create { project: PROJECT, batch });
        }
        Making::Agent => {
            let key = world.call_key(1);
            return fixtures::delegate(world, (key.task, key.attempt), key.completion, delegate);
        }
        Making::Procedure => {
            world.hold_procedures = true;
            let number = core::fresh(&mut world.domain.core_mut().counters, core::Family::Task).expect("task number");
            world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Make {
                reads_project: false,
                creator: tasks::Party::Deployment { project: PROJECT },
                maker: None,
                places: Box::new([]),
                reply_to: ReplyTo::new(Token::new(750)),
                batch: Box::new([new(&template, number, tasks::Executor::Procedure { connector: 1, code: 1 })]),
            })));
            let (step, connector, code, _) = *world.procedure_inputs.last().expect("procedure offered");
            world.send(root::Event::connector_up(
                connector,
                contract::Up::Procedures(contract::procedures::Up::Step {
                    task: number,
                    step,
                    code: u32::from(code),
                    decision: contract::procedures::ProcedureStep {
                        batch: Box::new([delegate]),
                        ..contract::procedures::ProcedureStep::new(contract::procedures::StepEnd::Wait)
                    },
                }),
            ));
        }
        Making::Recurring => {
            let mut authority = template.authority.clone();
            authority.budget.spend = 30;
            world.send(root::Event::Core(core::Event::StartRecurring {
                project: PROJECT,
                authority,
                template: tasks::RecurringTemplate {
                    key: 1,
                    batch: Box::new([new(&template, 1, REQUESTED)]),
                    overlap: tasks::RecurringOverlap::Skip,
                },
            }));
        }
        Making::Proposal => {
            world.send(root::Event::Core(core::Event::NamedAction {
                to: ReplyTo::new(Token::new(751)),
                key: world.call_key(1),
                action: core::NamedAction::ProposeBatch {
                    batch: Box::new([delegate]),
                    reason: b"make this yours".as_slice().into(),
                    as_holder: true,
                },
            }));
            let core::CallPart::Proposed { proposal, .. } = world.answers.last().expect("proposal answer").2 else {
                panic!("proposal not offered: {:?}", world.answers.last());
            };
            fixtures::ask(
                world,
                82,
                people::Ask::DecideProposal {
                    project: PROJECT,
                    proposer: people::Proposer::Task(parent),
                    proposal,
                    decision: people::ProposalDecision::Accept,
                    running: None,
                },
            );
        }
    }
    world.assignments.last().expect("one new assignment").0
}

#[test]
fn random_project_sets_and_settings_bind_each_kind_of_creation_and_compose_its_run() {
    cases(
        this_test!("random_project_sets_and_settings_bind_each_kind_of_creation_and_compose_its_run"),
        4400..4408,
        |seed| {
            let mut rng = Rng::new(seed);
            for making in [Making::Party, Making::Agent, Making::Procedure, Making::Recurring, Making::Proposal] {
                let mut world = world(seed);
                let parent = world.task();
                let target = u32::try_from(1 + rng.below(2)).unwrap();
                let deployment = world.domain.core().charter(target).unwrap().policy.instructions.clone();
                let added = if rng.below(2) == 0 { String::new() } else { format!("Overlay {}.", rng.below(999)) };
                let against = world.domain.settings().newest();
                if !added.is_empty() {
                    let signed = world.sign_in.expect("signed in owner");
                    world.send(root::Event::Core(core::Event::ChangeSettings {
                        reply_to: ReplyTo::new(Token::new(752)),
                        sign_in: signed,
                        key: [83; 16],
                        digest: [83; 32],
                        change: Box::new(settings::Change {
                            against,
                            items: Box::new([settings::Item {
                                setting: settings::Setting::Charter(target),
                                edit: settings::Edit::Set(settings::Value::Overlay(Box::new(settings::Overlay {
                                    instructions: added.as_bytes().into(),
                                    ..settings::Overlay::empty()
                                }))),
                            }]),
                            running: settings::Running::Stays,
                        }),
                    }));
                    assert_ne!(world.domain.settings().newest(), against, "settings edit: {making:?}");
                }
                let mut set = set(1 + rng.below(20));
                let project = if rng.below(2) == 0 { String::new() } else { format!("Set {}.", rng.below(999)) };
                let first = format!("First {}.", rng.below(999));
                let words = &mut set.charters[0];
                words.settings = if target == 1 { NAME.into() } else { OTHER.into() };
                words.instructions = project.as_bytes().into();
                words.parts = Box::new([
                    core::CharterPart { on: tasks::Reasons::FIRST, instructions: first.as_bytes().into() },
                    core::CharterPart { on: tasks::Reasons::QUESTION, instructions: b"Not first.".as_slice().into() },
                ]);
                let revision = set.revision;
                world.send(root::Event::Core(core::Event::ProjectCharters(set)));
                let made = create(&mut world, making);
                assert_ne!(
                    made,
                    parent,
                    "{making:?}: replies {:?}; live {:?}",
                    world.people_answers.last(),
                    world.domain.core().tasks.view_tasks()
                );
                let row = fixtures::live_row(&world, made);
                assert_eq!(row.executor, tasks::Executor::Agent { charter: target }, "{making:?}");
                assert_eq!(row.scope.as_deref(), Some(NAME));
                assert_eq!(row.charters, revision);
                assert_eq!(row.settings.task, world.domain.settings().newest());
                let run = world.assigned.as_ref().expect("new run");
                assert_eq!(run.task, made);
                let base = [project.as_str(), added.as_str()]
                    .into_iter()
                    .filter(|section| !section.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n\n");
                assert_eq!(run.run.policy.instructions.as_ref(), format!("{base}\n\n{first}").as_bytes(), "{making:?}");
                assert_ne!(run.run.policy.instructions, deployment);
            }
            Ok::<(), Infallible>(())
        },
    )
    .unwrap_or_else(|case| panic!("{case}"));
}
