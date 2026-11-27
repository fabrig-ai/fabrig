//! Cross-tree dependency and watch admission (jig/docs/design/domain/tasks/messages.md, 5).
use super::stories::{finish, plan_world, running};
use jig_core_world::fixtures::{
    CONNECTOR, PARTY, PROJECT, ask_as, chat_for, delegate, live_row, member, second_party, two_projects,
};
use jig_core_world::world::{Setup, World};
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use jig_test_profile::fixtures::unchanged;
use skein_lib::{Duration, ReplyTo, Token};

const OTHER_PROJECT: u32 = 2;
const CHARTER: tasks::Executor = tasks::Executor::Agent { charter: 1 };
const ROOT_KEY: u8 = 48;
const CALL: u64 = 12_200;

fn root_task(world: &mut World, project: u32, sign_in: u64, role: people::Role, words: &[u8]) -> u64 {
    let request = chat_for(world, project, role.number(), words.into());
    request_root(world, sign_in, request)
}

fn request_root(world: &mut World, sign_in: u64, request: people::Ask) -> u64 {
    ask_as(world, sign_in, ROOT_KEY, request);
    let Some(people::Reply::Outcome(people::Outcome::Created { first, last, .. })) = world.people_answers.last() else {
        panic!("root was not created: {:?}", world.people_answers.last());
    };
    assert_eq!(first, last);
    *first
}

fn other_root(world: &mut World, project: u32) -> u64 {
    root_task(world, project, world.sign_in.expect("signed in"), people::Role::Owner, b"other root")
}

fn key(run: (u64, u64), completion: u32) -> core::CallKey {
    core::CallKey { task: run.0, attempt: run.1, completion, position: 0 }
}

fn call(world: &mut World, run: (u64, u64), completion: u32, action: core::NamedAction) -> core::CallPart {
    let key = key(run, completion);
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(CALL + u64::from(completion))),
        key,
        action,
    }));
    answer(world, key)
}

fn answer(world: &World, key: core::CallKey) -> core::CallPart {
    let (_, answered, part) = world.answers.last().expect("call answer");
    assert_eq!(*answered, key);
    part.clone()
}

fn dependent(world: &World, target: u64) -> core::Delegate {
    let mut child = member(world, CHARTER, b"wait for the other task");
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    child.dependencies = Box::new([core::Dependency::Existing(target)]);
    child
}

fn refused_delegate(world: &mut World, run: (u64, u64), completion: u32, target: u64) -> tasks::Problem {
    let key = key(run, completion);
    world.send(root::Event::Core(core::Event::Delegate {
        to: ReplyTo::new(Token::new(CALL + u64::from(completion))),
        key,
        batch: Box::new([dependent(world, target)]),
    }));
    let core::CallPart::DelegationRefused(problem) = answer(world, key) else {
        panic!("expected a dependency refusal: {:?}", world.answers.last());
    };
    problem
}

#[test]
fn a_task_waits_on_another_tree_its_requester_reads() {
    let (mut configuration, mut limits) = jig_core_world::faults::plan_fixture(12_201);
    limits.core.authority.roles = 2;
    let roles = &mut configuration.core.seed_projects[0].policy.policy.roles;
    let mut member = roles[0].clone();
    member.number = people::Role::Member.number();
    *roles = Box::new([roles[0].clone(), member]);
    let mut world = World::new(Setup { seed: 12_201, configuration: Some(configuration), limits, ..Setup::default() });
    let prerequisite = world.assignments[0].0;
    let signed = second_party(&mut world, b"member who delegates", people::Role::Member);
    let task = root_task(&mut world, PROJECT, signed, people::Role::Member, b"wait on the other root");
    let requester = running(&world, task);
    let tasks::Party::Person(person) = live_row(&world, task).requester else { panic!("person's root") };
    assert_eq!(world.domain.core().people.role(person, PROJECT), Some(people::Role::Member));
    assert_eq!(live_row(&world, prerequisite).requester, tasks::Party::Person(PARTY));
    assert_eq!(live_row(&world, prerequisite).root, prerequisite);
    assert_ne!(live_row(&world, requester.0).root, prerequisite);
    assert!(live_row(&world, requester.0).references.is_empty());

    let child = dependent(&world, prerequisite);
    let child = delegate(&mut world, requester, 1, child);
    assert_eq!(&*live_row(&world, child).waiting_on, &[prerequisite]);
    assert_eq!(live_row(&world, child).phase, tasks::Phase::Waiting);
    let kind = tasks::SubscriptionKind::Task { target: prerequisite, held: false, result: true };
    let core::CallPart::Subscribed { subscription } =
        call(&mut world, requester, 2, core::NamedAction::Subscribe { kind })
    else {
        panic!("watch was not admitted: {:?}", world.answers.last());
    };
    assert!(live_row(&world, requester.0).references.is_empty());

    let upstream = running(&world, prerequisite);
    finish(&mut world, upstream, b"done");
    assert!(live_row(&world, child).waiting_on.is_empty());
    assert_ne!(live_row(&world, child).phase, tasks::Phase::Waiting);
    let notice = tasks::MessageKind::Notice { subscription, target: prerequisite, state: tasks::NoticeState::Done };
    assert!(live_row(&world, requester.0).inbox.iter().any(|word| word.kind == notice));
}

const LOST_READING_SEED: u64 = 12_212;

fn watch_fixture() -> (root::Config, root::Limits) {
    let (mut configuration, mut limits) = jig_core_world::faults::plan_fixture(LOST_READING_SEED);
    limits.core.authority.roles = 3;
    let roles = &mut configuration.core.seed_projects[0].policy.policy.roles;
    let mut member = roles[0].clone();
    member.number = people::Role::Member.number();
    let mut maintainer = roles[0].clone();
    maintainer.number = people::Role::Maintainer.number();
    *roles = Box::new([roles[0].clone(), maintainer, member]);
    (configuration, limits)
}

fn watch(world: &mut World, run: (u64, u64), completion: u32, target: u64, held: bool) -> u64 {
    let kind = tasks::SubscriptionKind::Task { target, held, result: true };
    let core::CallPart::Subscribed { subscription } =
        call(world, run, completion, core::NamedAction::Subscribe { kind })
    else {
        panic!("watch was not admitted: {:?}", world.answers.last());
    };
    subscription
}

fn set_roles(world: &mut World, owner: u64, key: u8, holdings: Box<[people::Holding]>) {
    ask_as(world, owner, key, people::Ask::SetRoles { project: PROJECT, holdings });
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::RolesSet { project: PROJECT }))
    );
}

fn watch_word(row: &tasks::TaskRecord, subscription: u64) -> &tasks::Word {
    let mut notices = row.inbox.iter().filter(
        |word| matches!(word.kind, tasks::MessageKind::Notice { subscription: number, .. } if number == subscription),
    );
    let word = notices.next().expect("watch notice in inbox");
    assert!(notices.next().is_none(), "one merged slot for this watch");
    word
}

#[test]
fn losing_project_reading_ends_only_the_reading_watch_and_its_last_notice_survives_restart() {
    let (configuration, limits) = watch_fixture();
    let mut world =
        World::new(Setup { seed: LOST_READING_SEED, configuration: Some(configuration), limits, ..Setup::default() });
    let target = world.assignments[0].0;
    let owner = world.sign_in.expect("owner signed in");
    let signed = second_party(&mut world, b"member who watches", people::Role::Member);
    let mut ask = chat_for(&world, PROJECT, people::Role::Member.number(), b"watch the other tree".as_slice().into());
    let people::Ask::Create { batch, .. } = &mut ask else { panic!("chat creation") };
    batch[0].wake = Some(people::WakePolicy {
        words: people::WakeRule::Never,
        notices: people::WakeRule::Batch { count: 2, age: Duration::from_secs(60) },
        news: people::WakeRule::Never,
        results: people::ResultsWake::Never,
        questions: true,
        answers: true,
        timers: true,
    });
    let task = request_root(&mut world, signed, ask);
    let tasks::Party::Person(person) = live_row(&world, task).requester else { panic!("person's root") };
    let run = running(&world, task);
    let child = member(&world, CHARTER, b"watch by reference");
    let child = delegate(&mut world, run, 1, child);
    let waiting = dependent(&world, target);
    let waiting = delegate(&mut world, run, 2, waiting);
    let by_reading = watch(&mut world, run, 3, target, true);
    let by_reference = watch(&mut world, run, 4, child, false);
    assert!(live_row(&world, task).references.is_empty());
    assert_ne!(live_row(&world, target).root, task);
    assert_eq!(live_row(&world, waiting).waiting_on.as_ref(), [target]);

    // Give the watch a pending hint so losing it must replace, not add to, the slot.
    let previous_state = tasks::NoticeState::Held { revision: 1 };
    let at = world.wall_time();
    let previous = world.domain.core_mut().notice(task, by_reading, target, previous_state, Box::new([]), at);
    world.send(root::Event::Core(core::Event::Tasks(previous)));
    let first = watch_word(live_row(&world, task), by_reading).clone();
    assert_eq!(first.kind, tasks::MessageKind::Notice { subscription: by_reading, target, state: previous_state });
    assert_eq!((first.hits, first.eligible), (1, false));

    let owner_holding = people::Holding { person: PARTY, role: people::Role::Owner };
    set_roles(
        &mut world,
        owner,
        49,
        Box::new([owner_holding, people::Holding { person, role: people::Role::Maintainer }]),
    );
    let row = live_row(&world, task);
    assert_eq!(row.subscriptions.len(), 2);
    assert_eq!(watch_word(row, by_reading), &first);

    set_roles(&mut world, owner, 50, Box::new([owner_holding]));
    let row = live_row(&world, task);
    assert!(
        matches!(row.subscriptions.as_ref(), [tasks::Subscription { number, kind: tasks::SubscriptionKind::Task { target, .. } }] if *number == by_reference && *target == child)
    );
    assert_eq!(live_row(&world, waiting).waiting_on.as_ref(), [target]);
    assert_eq!(live_row(&world, waiting).phase, tasks::Phase::Waiting);
    let last = watch_word(row, by_reading).clone();
    assert_eq!(
        last.kind,
        tasks::MessageKind::Notice { subscription: by_reading, target, state: tasks::NoticeState::Unreadable }
    );
    assert!(last.number > first.number);
    assert_eq!(
        (last.at, last.hits, last.eligible),
        (first.at, 2, true),
        "the merged notice reaches the batch threshold"
    );

    let upstream = running(&world, target);
    finish(&mut world, upstream, b"done");
    assert!(live_row(&world, waiting).waiting_on.is_empty());
    assert_ne!(live_row(&world, waiting).phase, tasks::Phase::Waiting);
    assert_eq!(watch_word(live_row(&world, task), by_reading), &last);

    world.restart_with(watch_fixture().0);
    let restored = world.domain.core().tasks.task(task).expect("restored watcher");
    assert_eq!(restored.subscriptions.len(), 1);
    assert_eq!(restored.subscriptions[0].number, by_reference);
    assert_eq!(watch_word(restored, by_reading), &last);
}

#[test]
fn a_task_cannot_depend_on_a_task_of_another_project() {
    let (configuration, mut limits) = two_projects(12_202);
    limits.core.tasks.tasks = 3;
    let mut world = World::new(Setup { seed: 12_202, configuration: Some(configuration), limits, ..Setup::default() });
    let requester = world.assignments[0];
    let other = other_root(&mut world, OTHER_PROJECT);
    assert_eq!(live_row(&world, other).project, OTHER_PROJECT);
    assert_eq!(refused_delegate(&mut world, requester, 1, other).why, tasks::Refusal::Dependencies);
}

fn deployment_root(world: &mut World, source: u64) -> u64 {
    let number = core::fresh(&mut world.domain.core_mut().counters, core::Family::Task).expect("task number");
    let source = live_row(world, source);
    let mut authority = source.authority.clone();
    authority.budget.spend = 100;
    let batch = Box::new([tasks::New {
        number,
        project: PROJECT,
        label: source.label,
        flags: source.flags,
        executor: CHARTER,
        llm: source.llm.clone(),
        spec: source.spec.clone(),
        contract: source.contract.clone(),
        authority,
        deadline: tasks::Deadline::Named,
        numbers: tasks::Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
        funder: tasks::Funder::Period { project: PROJECT, period: 1 },
        dependencies: Box::new([]),
        holdings: Box::new([]),
        wake: source.wake,
        recurring: None,
        tracked: None,
    }]);
    let creator = tasks::Party::Deployment { project: PROJECT };
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Make {
        reads_project: false,
        maker: None,
        places: Box::new([]),
        reply_to: ReplyTo::new(Token::new(CALL)),
        creator,
        batch,
    })));
    assert_eq!(live_row(world, number).requester, creator);
    number
}

#[test]
fn a_deployments_tree_does_not_read_its_project() {
    let mut world = plan_world(12_203);
    let target = world.assignments[0].0;
    let root = deployment_root(&mut world, target);
    let requester = running(&world, root);
    assert_eq!(refused_delegate(&mut world, requester, 1, target).why, tasks::Refusal::Dependencies);
}

#[test]
fn removing_the_tree_requesters_role_removes_new_reading_reach() {
    let mut world = plan_world(12_204);
    let requester = world.assignments[0];
    let owner = second_party(&mut world, b"other owner", people::Role::Owner);
    let target = root_task(&mut world, PROJECT, owner, people::Role::Owner, b"new owner's root");
    let tasks::Party::Person(person) = live_row(&world, target).requester else { panic!("person's root") };
    assert_ne!(person, PARTY);
    ask_as(
        &mut world,
        owner,
        49,
        people::Ask::SetRoles {
            project: PROJECT,
            holdings: Box::new([people::Holding { person, role: people::Role::Owner }]),
        },
    );
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::RolesSet { project: PROJECT }))
    );
    assert_eq!(refused_delegate(&mut world, requester, 1, target).why, tasks::Refusal::Dependencies);
    let kind = tasks::SubscriptionKind::Task { target, held: false, result: true };
    assert!(matches!(
        call(&mut world, requester, 2, core::NamedAction::Subscribe { kind }),
        core::CallPart::SubscriptionRefused(tasks::Problem { why: tasks::Refusal::Reference, .. })
    ));
}

#[test]
fn readable_dependencies_cannot_close_a_circle_between_trees() {
    let mut world = plan_world(12_205);
    let first = world.assignments[0];
    let second = other_root(&mut world, PROJECT);
    let second_run = running(&world, second);
    let child = dependent(&world, first.0);
    let child = delegate(&mut world, second_run, 1, child);
    let problem = refused_delegate(&mut world, first, 1, second);
    assert_eq!(problem.why, tasks::Refusal::Circle { cut: false });
    assert_eq!(problem.blocked_by.as_deref(), Some([first.0, child + 1, second, child].as_slice()));
    assert!(live_row(&world, first.0).delegates.is_empty());
}

#[test]
fn reading_does_not_grant_a_reference_to_message_or_amend() {
    let mut world = plan_world(12_206);
    let requester = world.assignments[0];
    let target = other_root(&mut world, PROJECT);
    let message = core::NamedAction::Message {
        target,
        kind: tasks::MessageKind::Words,
        words: b"not a reference".as_slice().into(),
    };
    let reply = call(&mut world, requester, 1, message);
    assert!(
        matches!(&reply,
        core::CallPart::ToolDenied { answer: authority::Answer::Refuse, findings }
            if findings.as_ref() == [authority::Finding::Reference]),
        "{reply:?}"
    );
    assert!(matches!(
        amend(&mut world, requester, 2, target, requester.0),
        core::CallPart::ControlRefused(tasks::Problem { why: tasks::Refusal::Reference, .. })
    ));
}

fn amend(world: &mut World, run: (u64, u64), completion: u32, target: u64, dependency: u64) -> core::CallPart {
    let mut amendment = unchanged(b"a new prerequisite");
    amendment.dependencies = Some(Box::new([dependency]));
    let revision = live_row(world, target).revisions.spec;
    call(world, run, completion, core::NamedAction::Amend { keep_parameters: true, target, revision, amendment })
}

#[test]
fn a_task_amends_its_delegate_to_wait_on_a_readable_live_task() {
    let mut world = plan_world(12_207);
    let requester = world.assignments[0];
    let prerequisite = other_root(&mut world, PROJECT);
    let member = member(&world, CHARTER, b"amended delegate");
    let child = delegate(&mut world, requester, 1, member);
    assert!(matches!(amend(&mut world, requester, 2, child, prerequisite), core::CallPart::Revised { .. }));
    assert_eq!(&*live_row(&world, child).waiting_on, &[prerequisite]);
}

#[test]
fn a_task_amends_its_delegate_to_depend_on_a_readable_ended_task() {
    let mut world = plan_world(12_208);
    let requester = world.assignments[0];
    let prerequisite = other_root(&mut world, PROJECT);
    let member = member(&world, CHARTER, b"amended delegate");
    let child = delegate(&mut world, requester, 1, member);
    let upstream = running(&world, prerequisite);
    finish(&mut world, upstream, b"done");
    assert!(world.domain.core().tasks.task(prerequisite).is_none());

    assert!(matches!(amend(&mut world, requester, 2, child, prerequisite), core::CallPart::Revised { .. }));
    assert_eq!(&*live_row(&world, child).dependencies, &[prerequisite]);
    assert!(live_row(&world, child).waiting_on.is_empty());
}

#[test]
fn a_task_cannot_amend_its_delegate_to_depend_on_a_failed_readable_task() {
    let mut world = plan_world(12_209);
    let requester = world.assignments[0];
    let prerequisite = other_root(&mut world, PROJECT);
    let member = member(&world, CHARTER, b"amended delegate");
    let child = delegate(&mut world, requester, 1, member);
    let (_, attempt) = running(&world, prerequisite);
    world.send(root::Event::Answer {
        channel: Token::new(7),
        task: prerequisite,
        attempt,
        read: None,
        refused: Box::new([]),
        cumulative: 0,
        end: tasks::End::Finished {
            result: tasks::TaskResult::Failure { reason: b"failed".as_slice().into() },
            cancel_delegates: false,
        },
    });
    assert!(world.domain.core().tasks.task(prerequisite).is_none());

    let problem = tasks::Problem::naming(
        Some(child),
        tasks::Refusal::DependencyEnded { how: tasks::Status::Failed },
        &[prerequisite],
    );
    assert_eq!(amend(&mut world, requester, 2, child, prerequisite), core::CallPart::ControlRefused(problem));
    assert!(live_row(&world, child).dependencies.is_empty());
}

#[test]
fn a_procedure_can_make_a_delegate_wait_on_another_readable_tree() {
    let mut world = plan_world(12_210);
    world.hold_procedures = true;
    let requester = world.assignments[0];
    let prerequisite = other_root(&mut world, PROJECT);
    let procedure = member(&world, tasks::Executor::Procedure { connector: CONNECTOR, code: 1 }, b"procedure");
    let procedure = delegate(&mut world, requester, 1, procedure);
    let (step, connector, code, context) = world.procedure_inputs.last().expect("procedure input");
    assert_eq!(context.task, procedure);
    world.send(root::Event::Core(core::Event::ProcedureStep {
        task: procedure,
        step: *step,
        connector: *connector,
        code: u32::from(*code),
        decision: core::ProcedureStep {
            batch: Box::new([dependent(&world, prerequisite)]),
            ..core::ProcedureStep::new(core::StepEnd::Wait)
        },
    }));
    let [child] = live_row(&world, procedure).delegates.as_ref() else {
        panic!("procedure did not delegate: {:?}", world.trace);
    };
    assert_eq!(&*live_row(&world, *child).waiting_on, &[prerequisite]);
}
