//! A chat's change goes through the same named delegation admission as a raw batch.
use super::*;

fn chat_world(adopt: bool) -> (World, engine::Assignment) {
    chat_world_with_words(adopt, &[b'x'; 128])
}

pub(super) fn chat_world_with_words(adopt: bool, words: &[u8]) -> (World, engine::Assignment) {
    let mut world = World::configured(true, true);
    if adopt {
        world.adopt();
    } else {
        world.sign_in();
    }
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { panic!("create") };
    batch[0].words = words.into();
    let chat = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    (world, chat)
}

pub(super) fn change(
    world: &mut World,
    chat: &engine::Assignment,
    name: u64,
    repository: Option<&[u8]>,
    words: &[u8],
) -> engine::CallAnswer {
    change_landing(world, chat, name, repository, words, None)
}

pub(super) fn change_landing(
    world: &mut World,
    chat: &engine::Assignment,
    name: u64,
    repository: Option<&[u8]>,
    words: &[u8],
    landing: Option<engine::ChangeLanding>,
) -> engine::CallAnswer {
    let mut change = engine::ChangeCall::words(words.into(), repository.map(Box::from));
    change.landing = landing;
    let before = world.answers.len();
    let event = world.protocol.call(
        Token::new(7),
        chat.task,
        chat.attempt,
        name,
        engine::Call {
            completion: u32::try_from(name).expect("bounded call name"),
            position: 1,
            tool: engine::Tool::Change { changes: Box::new([change]) },
        },
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.answers.len() > before {
            return world.answers[before].clone();
        }
    }
    panic!("change was not answered");
}

// Two changes share the half left after the chat's 60-unit run reservation.
const PLAN_CAP: u64 = 200;
const PLAN_CALL: u64 = 93;
const PLAN_STEPS: u32 = 1000;
const CHAT_AND_FIRST_PRODUCER: usize = 2;
const CHAT_AND_BOTH_PRODUCERS: usize = 3;
const TWO_CHANGES: usize = 2;

fn plan_world(control: bool) -> (World, engine::Assignment) {
    plan_world_with_limits(control, &StoryLimits { tasks_batch: Some(2), ..StoryLimits::default() })
}

fn plan_world_with_limits(control: bool, limits: &StoryLimits) -> (World, engine::Assignment) {
    let mut world = World::configured_limits(true, true, false, PLAN_STEPS, Gate::default(), None, *limits);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { panic!("create") };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(PLAN_CAP) };
    let family = jig_domain_core::ToolFamilies::JIG.control.0;
    if control {
        batch[0].authority.tools |= family;
    } else {
        batch[0].authority.tools &= !family;
    }
    let chat = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    (world, chat)
}

fn plan_change(world: &mut World, chat: &engine::Assignment, input: &[u8]) -> Option<engine::CallAnswer> {
    let before = start_plan_change(world, chat, input);
    world.until(Until::Ready);
    world.answers.get(before).cloned()
}

fn plan_change_admission(world: &mut World, chat: &engine::Assignment, input: &[u8]) -> Option<engine::CallAnswer> {
    let errors = world.call_errors.len();
    let before = start_plan_change(world, chat, input);
    // Observe the committed admission without running every admitted procedure to idle.
    for _ in 0..200 {
        world.tick();
        if world.answers.len() > before || world.call_errors.len() > errors {
            return world.answers.get(before).cloned();
        }
    }
    panic!("change admission was not answered");
}

fn start_plan_change(world: &mut World, chat: &engine::Assignment, input: &[u8]) -> usize {
    world.until(Until::Ready);
    let before = world.answers.len();
    let input = smith_domain_run::HostInput::attested(input.into()).expect("bounded input");
    let temper_engine_smith::Decoded::Call(call) = temper_engine_smith::call_configured(
        smith_domain_run::CallName { activation: chat.attempt, completion: 1, position: 1 },
        b"change",
        &input,
        &walking::models(),
        &[],
    )
    .expect("change decodes") else {
        panic!("change call")
    };
    let event = world.protocol.call(Token::new(7), chat.task, chat.attempt, PLAN_CALL, call);
    world.send(event);
    before
}

fn stored_change(world: &World, task: u64) -> &forge_top::ChangeRow {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(task))) else {
        panic!("registered change")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    row
}

#[test]
fn a_plan_member_from_a_task_without_control_is_refused() {
    let (mut world, chat) = plan_world(false);
    let rows = world.store.rows.clone();
    plan_change(&mut world, &chat, br#"{"words":"Small fix","landing":"main"}"#);
    assert_eq!(world.call_errors.len(), 1);
    assert_eq!(world.call_errors[0].text.as_ref(), b"change's landing, branch, name, after, questions, repairs, changes, producer and llm are a plan's: this task's authority lacks the control family (amend, cancel, release).");
    assert_eq!(world.store.rows, rows, "no delegate or call record is built");
}

#[test]
fn a_branch_without_the_control_family_is_a_plans_member_refused() {
    let (mut world, chat) = plan_world(false);
    let rows = world.store.rows.clone();
    plan_change(&mut world, &chat, br#"{"words":"Small fix","branch":"mine"}"#);
    assert_eq!(world.call_errors.len(), 1);
    assert_eq!(world.call_errors[0].text.as_ref(), b"change's landing, branch, name, after, questions, repairs, changes, producer and llm are a plan's: this task's authority lacks the control family (amend, cancel, release).");
    assert_eq!(world.store.rows, rows, "no delegate or call record is built");
}

#[test]
fn a_producer_from_a_task_without_control_is_refused() {
    let (mut world, chat) = plan_world(false);
    let rows = world.store.rows.clone();
    plan_change(&mut world, &chat, br#"{"words":"Small fix","producer":6}"#);
    assert_eq!(world.call_errors.len(), 1);
    assert_eq!(world.call_errors[0].text.as_ref(), b"change's landing, branch, name, after, questions, repairs, changes, producer and llm are a plan's: this task's authority lacks the control family (amend, cancel, release).");
    assert_eq!(world.store.rows, rows, "no delegate or call record is built");
    assert_eq!(world.assigned.len(), 1);
}

#[test]
fn a_control_changes_choices_are_validated_before_delegation() {
    for (input, expected) in [
        (br#"{"words":"Small fix","producer":7}"#.as_slice(), tasks::Refusal::Executor),
        (br#"{"changes":[{"words":"First fix"},{"words":"Second fix","producer":7}]}"#, tasks::Refusal::Executor),
    ] {
        let (mut world, chat) = plan_world(true);
        let answer = plan_change(&mut world, &chat, input);
        assert!(
            matches!(&answer, Some(engine::CallAnswer::DelegationRefused(problem)) if problem.why == expected),
            "{input:?}: {answer:?}"
        );
        assert!(world.call_errors.is_empty());
        assert!(!world.store.rows.keys().any(|key| matches!(key, Key::Forge(forge_top::Key::Change(_)))));
        assert_eq!(
            world
                .store
                .rows
                .keys()
                .filter(|key| matches!(key, Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(_)))))
                .count(),
            1,
            "no delegate is built"
        );
        assert_eq!(world.assigned.len(), 1);
    }
}

#[test]
fn a_control_changes_producer_llm_is_admitted_and_saved() {
    let (mut world, chat) = plan_world(true);
    let answer = plan_change(
        &mut world,
        &chat,
        br#"{"words":"Small fix","producer":1,"llm":{"producer":{"main":{"model":"scripted"}}}}"#,
    );
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("configured choices admitted: {answer:?}") };
    assert!(world.call_errors.is_empty());
    let row = stored_change(&world, made[0].task);
    assert_eq!(row.producing_charter, 1);
    assert_eq!(
        row.producing_llm.as_ref().expect("saved producer choice").main,
        tasks::ModelEffort { model: 0, effort: None }
    );
    assert!(row.review_llm.is_none());
}

#[test]
fn two_changes_in_one_call_are_one_batch_the_second_after_the_first() {
    let (mut world, chat) = plan_world(true);
    let answer = plan_change(
        &mut world,
        &chat,
        br#"{"changes":[{"name":"a","words":"First fix"},{"words":"Second fix","after":["a"]}]}"#,
    );
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("one batch: {answer:?}") };
    assert_eq!(made.len(), TWO_CHANGES);
    assert_eq!(live(&world, made[0].task).spec.words.as_ref(), b"First fix");
    let second = live(&world, made[1].task);
    assert_eq!(second.spec.words.as_ref(), b"Second fix");
    assert_eq!(second.dependencies.as_ref(), &[made[0].task]);
    assert_eq!(live(&world, made[0].task).authority.delegation.tasks, 5, "nine descendant slots shared equally");
    assert_eq!(second.authority.delegation.tasks, 4, "one slot per change reserved before sharing descendants");
    assert_eq!(second.authority.budget.spend, 35, "half of the 140 left, shared equally");
    world.until(Until::SecondAssignment);
    world.until(Until::Ready);
    assert_eq!(world.assigned.len(), CHAT_AND_FIRST_PRODUCER);
    let producer = world.assigned[1].clone();
    assert_eq!(live(&world, producer.task).requester, tasks::Party::Task(made[0].task));
    let branch = stored_change(&world, made[0].task).branch.clone();
    land_change(&mut world, &producer, &branch);
    for _ in 0..PLAN_STEPS {
        world.tick();
        if world.assigned.len() == CHAT_AND_BOTH_PRODUCERS {
            break;
        }
    }
    assert_eq!(world.merges, 1);
    assert_eq!(world.assigned.len(), CHAT_AND_BOTH_PRODUCERS);
    assert_eq!(live(&world, world.assigned[2].task).requester, tasks::Party::Task(made[1].task));
}

#[test]
fn a_change_batch_admits_thirty_two_members_and_refuses_one_more() {
    const BATCH: u32 = 32; // temper/docs/design/protocol/configuration/settings.md, 7, A batch.
    const ONE_MORE: u32 = BATCH + 1;
    const TASK_ROOM: u32 = ONE_MORE * 2; // One change and one producer for each member.
    // Both admission rows, including their world setup, measured 102 ms CPU together.
    for (count, expected) in [
        (BATCH, Ok(usize::try_from(BATCH).unwrap())),
        (ONE_MORE, Err((authority::Answer::Refuse, Box::from([authority::Finding::Oversized])))),
    ] {
        let (mut world, chat) = plan_world_with_limits(
            true,
            &StoryLimits {
                authority_batch: Some(BATCH),
                tasks_batch: Some(ONE_MORE),
                task_delegates: Some(ONE_MORE),
                tasks: Some(TASK_ROOM + 1),
                tree_tasks: Some(TASK_ROOM + 1),
                inbox_messages: Some(ONE_MORE),
                inbox_bytes: Some(ONE_MORE * 128),
                ..StoryLimits::default()
            },
        );
        let input = serde_json::to_vec(&serde_json::json!({
            "changes": vec![serde_json::json!({"words": "Small fix"}); usize::try_from(count).unwrap()]
        }))
        .expect("change batch JSON");
        let answer = plan_change_admission(&mut world, &chat, &input);
        let outcome = match answer {
            Some(engine::CallAnswer::Delegated(made)) => Ok(made.len()),
            Some(engine::CallAnswer::DelegationDenied { answer, findings }) => Err((answer, findings)),
            other => panic!("batch of {count}: {other:?}; errors: {:?}", world.call_errors),
        };
        assert_eq!(outcome, expected, "batch of {count}");
        let admitted = outcome.unwrap_or(0);
        assert_eq!(world.answers.len(), 1, "one answer for one batch");
        assert_eq!(
            world
                .store
                .rows
                .keys()
                .filter(|key| matches!(key, Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(_)))))
                .count(),
            admitted + 1
        );
        assert_eq!(live(&world, chat.task).delegates.len(), admitted);
    }
}

#[test]
fn a_change_names_its_landing_branch() {
    let (mut world, chat) = plan_world(true);
    world.external(2, raw::Op::Git(raw::Git::Create { branch: b"release".as_slice().into(), commit: 1 }));
    let answer = plan_change(&mut world, &chat, br#"{"words":"Small fix","landing":"release"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("delegated: {answer:?}") };
    assert!(
        live(&world, made[0].task)
            .spec
            .parameters
            .contains(&tasks::Parameter::Bytes { name: 2, value: b"release".as_slice().into() })
    );
    world.until(Until::SecondAssignment);
    let branch = stored_change(&world, made[0].task).branch.clone();
    let producer = world.assigned[1].clone();
    land_change(&mut world, &producer, &branch);
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number: 1 })) else {
        panic!("opened pull")
    };
    assert_eq!(pull.base.as_ref(), b"release");
    assert_eq!(world.merges, 1);
}

#[test]
fn landing_goal_without_a_held_goal_branch_is_refused() {
    let (mut world, chat) = plan_world(true);
    let rows = world.store.rows.clone();
    plan_change(&mut world, &chat, br#"{"words":"Small fix","landing":"goal"}"#);
    assert_eq!(world.call_errors.len(), 1);
    assert_eq!(world.call_errors[0].text.as_ref(), b"landing goal: this task holds no goal branch in org/repo.");
    assert_eq!(world.store.rows, rows, "the missing goal branch makes no delegate");
}

#[test]
fn an_after_naming_no_earlier_change_is_refused() {
    let (mut world, chat) = plan_world(true);
    let answer = plan_change(
        &mut world,
        &chat,
        br#"{"changes":[{"words":"First fix","after":["b"]},{"name":"b","words":"Second fix"}]}"#,
    );
    assert!(matches!(answer, Some(engine::CallAnswer::DelegationRefused(problem))
        if problem.why == tasks::Refusal::Dependencies));
    assert_eq!(world.assigned.len(), 1);
    assert!(!world.store.rows.keys().any(|key| matches!(key, Key::Forge(forge_top::Key::Change(_)))));
}

fn live(world: &World, task: u64) -> &tasks::TaskRecord {
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task))))
    else {
        panic!("live task {task}")
    };
    row
}

#[test]
fn change_expands_the_only_repository_field_by_field() {
    let (mut world, chat) = chat_world(true);
    let parent = live(&world, chat.task).clone();
    let answer = change(&mut world, &chat, 93, None, b"Small fix");
    let engine::CallAnswer::Delegated(made) = answer else { panic!("delegated: {answer:?}") };
    let row = live(&world, made[0].task);
    assert_eq!(row.project, parent.project);
    assert_eq!(row.executor, tasks::Executor::Procedure { connector: 0, code: 2 });
    assert_eq!(
        row.spec,
        tasks::Spec {
            job: None,
            asks: false,
            words: Box::from(&b"Small fix"[..]),
            parameters: Box::new([
                tasks::Parameter::Resource {
                    name: 1,
                    connector: 0,
                    resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository)
                },
                tasks::Parameter::Bytes { name: 2, value: Box::from(&b"main"[..]) },
            ]),
            inputs: Box::new([]),
        }
    );
    assert_eq!(
        row.contract,
        tasks::Contract::Change { connector: 0, kind: 1, words: 512.min(world.env.limits.tasks.result_bytes) }
    );
    assert_eq!(row.authority.tools, parent.authority.tools);
    assert_eq!(
        row.authority.grants,
        [1, 2, 3, 4, 10, 11, 12, 17]
            .into_iter()
            .flat_map(|kind| {
                let prefix = [b"forge".as_slice(), b"forge.example", b"org"];
                [
                    (kind == 1).then(|| tasks::Grant {
                        connector: 0,
                        kind,
                        pattern: tasks::Pattern {
                            segments: prefix.map(Box::from).into(),
                            last: tasks::Last::Exact(Box::from(b"repo".as_slice())),
                        },
                    }),
                    Some(tasks::Grant {
                        connector: 0,
                        kind,
                        pattern: tasks::Pattern {
                            segments: [b"forge".as_slice(), b"forge.example", b"org", b"repo"].map(Box::from).into(),
                            last: tasks::Last::Open(Box::new([])),
                        },
                    }),
                ]
                .into_iter()
                .flatten()
            })
            .collect::<Box<[_]>>()
    );
    assert_eq!(
        row.authority.delegation.kinds.as_ref(),
        &[tasks::AuthorityExecutor::Charter(1), tasks::AuthorityExecutor::Charter(4)]
    );
    assert_eq!(row.authority.delegation.tasks, 6);
    assert_eq!(row.authority.delegation.depth, 2.min(parent.authority.delegation.depth - 1));
    assert_eq!(
        row.authority.budget.spend,
        (parent.numbers.budget - parent.numbers.spent - parent.numbers.spent_below - parent.numbers.reserved) / 2
    );
    assert_eq!(row.authority.budget.deadline, parent.authority.budget.deadline);
    assert_eq!(row.authority.notes, tasks::Scopes(0));
    assert!(row.authority.note_resources.is_empty());
    assert!(row.dependencies.is_empty());
    assert_eq!(row.wake, tasks::WakePolicy::DEFAULT);
    assert_eq!(row.label, 0);
    assert_eq!(row.flags, tasks::Flags::PLAIN);
    assert!(row.llm.is_none());
    assert_eq!(made[0].given, None, "named funding");
    world.until(Until::SecondAssignment);
    assert!(
        world.assigned[1].run.contract
            == tasks::Contract::Change { connector: 0, kind: 2, words: world.env.limits.tasks.result_bytes },
        "the same code-2 procedure starts its producer"
    );
}

#[test]
fn a_change_keeps_no_note_scope() {
    let (mut world, chat) = super::goal_tool::chat_with_note_scope(authority::Scopes(7));
    let answer = change(&mut world, &chat, PLAN_CALL, None, b"Small fix");
    let engine::CallAnswer::Delegated(made) = answer else { panic!("delegated: {answer:?}") };
    let row = live(&world, made[0].task);
    assert_eq!(row.authority.notes, tasks::Scopes(0));
    assert!(row.authority.note_resources.is_empty());
}

fn producer(words: &[u8]) -> (World, tasks::TaskRecord) {
    let (mut world, chat) = chat_world(true);
    let engine::CallAnswer::Delegated(_) = change(&mut world, &chat, 93, None, words) else { panic!("delegated") };
    world.until(Until::SecondAssignment);
    let row = live(&world, world.assigned[1].task).clone();
    (world, row)
}

#[test]
fn the_producer_hears_the_change_task_lands_what_it_delivers_when_its_bound_has_room() {
    let (world, row) = producer(b"Small fix");
    assert_eq!(
        row.spec.words.as_ref(),
        b"Small fix\n\nThe host pushes what you deliver; the change task opens the pull request, \
waits for CI and merges it. Finish with your change once it is checked. \
Report the branch and head you produced; only the change task reports landing after merge."
            .as_slice(),
        "the change's words, then the producer's scope"
    );
    // Words that leave no room for the scope beside the branches go alone.
    let bound = usize::try_from(world.env.limits.tasks.spec_bytes).expect("bound fits");
    let words = vec![b'x'; bound - b"main".len() - 64];
    let (_, row) = producer(&words);
    let branch = row
        .spec
        .parameters
        .iter()
        .find_map(|parameter| match parameter {
            tasks::Parameter::Bytes { name: 3, value } => Some(value.len()),
            _ => None,
        })
        .expect("the producer's branch");
    assert!(branch <= 64, "the branch fits beside the words: {branch}");
    assert_eq!(row.spec.words.as_ref(), words.as_slice(), "the words alone");
}

#[test]
fn change_with_no_repository_is_refused_once() {
    let (mut world, chat) = chat_world(false);
    let first = change(&mut world, &chat, 93, None, b"Small fix");
    assert!(matches!(first, engine::CallAnswer::ChangeRefused { ref repositories, .. } if repositories.is_empty()));
    let task_rows = world
        .store
        .rows
        .keys()
        .filter(|key| matches!(key, Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(_)))))
        .count();
    let again = change(&mut world, &chat, 93, None, b"Small fix");
    assert_eq!(again, first);
    assert_eq!(
        world
            .store
            .rows
            .keys()
            .filter(|key| matches!(key, Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(_)))))
            .count(),
        task_rows
    );
    let key = jig_domain_core::CallKey { task: chat.task, attempt: chat.attempt, completion: 93, position: 1 };
    assert!(world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Call(key)))));
}

#[test]
fn change_with_an_unknown_repository_names_the_available_one() {
    let (mut world, chat) = chat_world(true);
    let first = change(&mut world, &chat, 93, Some(b"org/missing"), b"Small fix");
    assert!(
        matches!(&first, engine::CallAnswer::ChangeRefused { named: Some(named), repositories, .. } if named.as_ref() == b"org/missing" && repositories.as_ref() == [Box::from(&b"org/repo"[..])])
    );
    assert_eq!(change(&mut world, &chat, 93, Some(b"org/missing"), b"Small fix"), first);
    assert_eq!(world.assigned.len(), 1);
}

#[test]
fn change_accepts_a_second_change_within_the_chats_task_allotment() {
    let (mut world, chat) = configured_chat(
        100,
        Box::new([people::Executor::Charter(1), people::Executor::Charter(4), people::Executor::Procedure(2)]),
        7,
    );
    assert!(matches!(change(&mut world, &chat, 93, None, b"First fix"), engine::CallAnswer::Delegated(_)));
    assert!(matches!(change(&mut world, &chat, 94, None, b"Second fix"), engine::CallAnswer::Delegated(_)));
}

#[test]
fn change_refuses_oversized_words_as_a_delegate_spec() {
    let (mut world, chat) = chat_world(true);
    let words = vec![b'x'; skein_lib::usize_of(world.env.limits.tasks.spec_bytes) + 1];
    assert!(
        matches!(change(&mut world, &chat, 93, None, &words), engine::CallAnswer::DelegationRefused(problem) if problem.why == tasks::Refusal::Spec)
    );
    assert_eq!(world.assigned.len(), 1);
}

fn configured_chat(cap: u64, kinds: Box<[people::Executor]>, tasks: u32) -> (World, engine::Assignment) {
    let mut world = World::configured(true, true);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { panic!("create") };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(cap) };
    batch[0].authority.delegation.kinds = kinds;
    batch[0].authority.delegation.tasks = tasks;
    let chat = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    (world, chat)
}

#[test]
fn change_without_procedure_permission_gets_delegates_executor_denial() {
    let (mut world, chat) = configured_chat(100, Box::new([people::Executor::Charter(1)]), 11);
    assert!(
        matches!(change(&mut world, &chat, 93, None, b"Small fix"), engine::CallAnswer::DelegationDenied { findings, .. } if findings.contains(&authority::Finding::Executor { source: authority::Source::Task }))
    );
    assert_eq!(world.assigned.len(), 1);
}

#[test]
fn change_without_task_capacity_gets_delegates_tasks_denial() {
    let (mut world, chat) = configured_chat(
        100,
        Box::new([people::Executor::Charter(1), people::Executor::Charter(4), people::Executor::Procedure(2)]),
        0,
    );
    assert!(
        matches!(change(&mut world, &chat, 93, None, b"Small fix"), engine::CallAnswer::DelegationDenied { findings, .. } if findings.contains(&authority::Finding::Tasks { source: authority::Source::Task }))
    );
}

#[test]
fn change_with_one_task_left_is_refused_as_the_tree_limit() {
    let (mut world, chat) = configured_chat(
        100,
        Box::new([people::Executor::Charter(1), people::Executor::Charter(4), people::Executor::Procedure(2)]),
        1,
    );
    plan_change(&mut world, &chat, br#"{"words":"Small fix"}"#);
    assert_eq!(world.call_errors.len(), 1);
    assert_eq!(world.call_errors[0].text.as_ref(), b"change batch: task tree limit; available 1 tasks, required 2 (one change task and at least one descendant per member). Fewer changes per call cannot help: even one change needs two tasks of lifetime authority.");
    assert_eq!(world.assigned.len(), 1);
}

#[test]
fn change_with_one_level_left_is_refused_as_the_depth_limit() {
    let mut world = World::configured(true, true);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { panic!("create") };
    batch[0].authority.delegation.depth = 1;
    let chat = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    assert!(
        matches!(change(&mut world, &chat, 93, None, b"Small fix"), engine::CallAnswer::DelegationRefused(problem) if problem.why == tasks::Refusal::Depth)
    );
    assert_eq!(world.assigned.len(), 1);
}

#[test]
fn change_with_half_the_remainder_at_minimum_is_refused_as_funding() {
    let (mut world, chat) = configured_chat(
        62,
        Box::new([people::Executor::Charter(1), people::Executor::Charter(4), people::Executor::Procedure(2)]),
        11,
    );
    let row = live(&world, chat.task);
    assert_eq!(
        (row.numbers.budget - row.numbers.reserved - row.numbers.spent - row.numbers.spent_below) / 2,
        1,
        "fixture leaves half at the minimum"
    );
    assert!(
        matches!(change(&mut world, &chat, 93, None, b"Small fix"), engine::CallAnswer::DelegationRefused(problem) if problem.why == tasks::Refusal::Funding)
    );
}

/// A chat whose project has two owned readable adoptions.
pub(super) fn several_repositories() -> (World, engine::Assignment) {
    let mut world = World::configured(true, true);
    world.adopt();
    world.until(Until::Ready);
    fake::repository(
        &mut world.fake,
        &world.fake_env.limits,
        raw::Setup {
            workflows: Box::new([]),
            id: 3,
            name: Box::from(&b"org/extra"[..]),
            default: Box::from(&b"main"[..]),
            tree: Box::new([]),
            labels: Box::new([]),
            checks: raw::Checks {
                cues: Box::new([]),
                contexts: Box::new([]),
                latency_min: Duration::ZERO,
                latency_max: Duration::ZERO,
                silent: 0,
                passes: 1000,
                reruns: 0,
                cue: None,
            },
            protection: None,
            hooked: false,
        },
    );
    fake::grant(&mut world.fake, b"org/extra", 1, raw::Permission::Admin);
    world.send(engine::Event::Party(engine::PartyInput::Application {
        scope: people::ApplicationScope::Project(1),
        reply_to: ReplyTo::new(Token::new(95)),
        sign_in: world.signed_in.expect("owner session"),
        key: [95; 16],
        digest: [6; 32],
        carried: None,
        request: engine::Application::Forge(forge_top::Ask::Adopt(forge_top::Adoption {
            project: 1,
            home: false,
            forge: forge_world::REPO.forge,
            host: Box::from(&b"forge.example"[..]),
            owner: Box::from(&b"org"[..]),
            name: Box::from(&b"extra"[..]),
            prefix: Box::from(&b"temper/"[..]),
            role: forge_top::Role::Owned,
            landing: Box::from(&b"main"[..]),
            ci: true,
            checks: Box::new([]),
        })),
    }));
    for _ in 0..200 {
        world.tick();
        if world.adopted.len() == 2 {
            break;
        }
    }
    assert_eq!(world.adopted.len(), 2, "second adoption: {:?}", world.adopted);
    assert!(matches!(world.adopted[1], forge_top::ApplicationOutcome::Adopted(_)), "{:?}", world.adopted);
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let ask = adoption_chat(&world);
    let chat = assigned_chat(&mut world, 92, ask);
    (world, chat)
}

#[test]
fn change_with_several_unnamed_repositories_is_refused_once() {
    let (mut world, chat) = several_repositories();
    let first = change(&mut world, &chat, 93, None, b"Small fix");
    assert!(
        matches!(&first, engine::CallAnswer::ChangeRefused { named: None, repositories, .. } if repositories.len() == 2),
        "{first:?}"
    );
    assert_eq!(change(&mut world, &chat, 93, None, b"Small fix"), first);
    assert_eq!(world.assigned.len(), 1);
}

#[test]
fn change_selects_the_named_repository_among_several() {
    let (mut world, chat) = several_repositories();
    let engine::CallAnswer::Delegated(made) = change(&mut world, &chat, 93, Some(b"org/extra"), b"Small fix") else {
        panic!("delegated")
    };
    assert_eq!(
        live(&world, made[0].task).spec.parameters[0],
        tasks::Parameter::Resource { name: 1, connector: 0, resource: forge_top::tag(forge_world::REPO.forge, 3) }
    );
}

pub(super) fn named_read(
    world: &mut World,
    chat: &engine::Assignment,
    completion: u32,
    tool: &[u8],
    input: &[u8],
) -> engine::CallAnswer {
    let before = world.answers.len();
    let input = smith_domain_run::HostInput::attested(input.into()).expect("bounded input");
    let temper_engine_smith::Decoded::Call(body) = temper_engine_smith::call(
        smith_domain_run::CallName { activation: chat.attempt, completion, position: 1 },
        tool,
        &input,
    )
    .expect("flat read") else {
        panic!("root read");
    };
    let event = world.protocol.call(Token::new(7), chat.task, chat.attempt, u64::from(completion), body);
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.answers.len() > before {
            return world.answers[before].clone();
        }
    }
    panic!("read was not answered");
}

#[test]
fn a_chat_reads_its_only_repository_by_branch_without_a_repository_name() {
    let (mut world, chat) = chat_world(true);
    let answer = named_read(&mut world, &chat, 93, b"read_branch", br#"{"branch":"main"}"#);
    assert!(matches!(answer, engine::CallAnswer::ForgeRead(ref result) if result.is_ok()), "{answer:?}");
    assert!(world.store.pending.is_empty(), "a read needs no commit");
}

#[test]
fn an_unnamed_read_with_two_repositories_names_both() {
    let (mut world, chat) = several_repositories();
    let answer = named_read(&mut world, &chat, 93, b"read_branch", br#"{"branch":"main"}"#);
    assert!(
        matches!(answer, engine::CallAnswer::ReadRepositoryRefused { named: None, ref repositories }
        if repositories.as_ref() == [Box::from(b"org/repo".as_slice()), Box::from(b"org/extra".as_slice())]),
        "{answer:?}"
    );
}

#[test]
fn an_empty_repository_name_reads_the_only_repository_and_refuses_two() {
    let (mut world, chat) = chat_world(true);
    let answer = named_read(&mut world, &chat, 93, b"read_branch", br#"{"repository":"","branch":"main"}"#);
    assert!(matches!(answer, engine::CallAnswer::ForgeRead(ref result) if result.is_ok()), "{answer:?}");
    let (mut world, chat) = several_repositories();
    let answer = named_read(&mut world, &chat, 93, b"read_pull", br#"{"repository":"","number":1}"#);
    assert!(
        matches!(answer, engine::CallAnswer::ReadRepositoryRefused { named: None, ref repositories }
        if repositories.as_ref() == [Box::from(b"org/repo".as_slice()), Box::from(b"org/extra".as_slice())]),
        "{answer:?}"
    );
}

#[test]
fn a_read_with_an_unknown_name_names_the_project_repositories() {
    let (mut world, chat) = chat_world(true);
    let answer = named_read(&mut world, &chat, 93, b"read_branch", br#"{"repository":"org/missing","branch":"main"}"#);
    assert!(
        matches!(answer, engine::CallAnswer::ReadRepositoryRefused { named: Some(ref named), ref repositories }
        if named.as_ref() == b"org/missing" && repositories.as_ref() == [Box::from(b"org/repo".as_slice())]),
        "{answer:?}"
    );
    let facts: Vec<_> = std::iter::from_fn(|| world.root.pop_fact()).collect();
    assert!(facts.iter().any(|fact| matches!(fact,
        engine::Fact::ReadRefused { task, attempt, tool, layer: "forge-route", reason: "repository name unresolved", .. }
        if *task == chat.task && *attempt == chat.attempt && tool.as_ref() == b"engine"
    )), "repository resolution retains its diagnostic layer: {facts:?}");
}

#[test]
fn a_named_context_repository_is_readable() {
    let (mut world, chat) = several_repositories();
    world.send(engine::Event::Party(engine::PartyInput::Application {
        scope: people::ApplicationScope::Project(1),
        reply_to: ReplyTo::new(Token::new(96)),
        sign_in: world.signed_in.expect("owner session"),
        key: [96; 16],
        digest: [6; 32],
        carried: None,
        request: engine::Application::Forge(forge_top::Ask::Adopt(forge_top::Adoption {
            project: 1,
            home: true,
            forge: forge_world::REPO.forge,
            host: b"forge.example".as_slice().into(),
            owner: b"org".as_slice().into(),
            name: b"repo".as_slice().into(),
            prefix: b"temper/".as_slice().into(),
            role: forge_top::Role::Context,
            landing: b"main".as_slice().into(),
            ci: true,
            checks: Box::new([]),
        })),
    }));
    for _ in 0..200 {
        world.tick();
        if world.adopted.len() == 3 {
            break;
        }
    }
    assert_eq!(world.adopted.len(), 3);
    let answer = named_read(&mut world, &chat, 93, b"read_branch", br#"{"repository":"org/repo","branch":"main"}"#);
    assert!(matches!(answer, engine::CallAnswer::ForgeRead(ref result) if result.is_ok()), "{answer:?}");
}

#[test]
fn a_chat_read_pull_for_reaches_the_fake_forge_through_named_routing() {
    let (mut world, chat) = chat_world(true);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: b"work".as_slice().into(), commit: 1 })),
        raw::Answer::Branch(_)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"work", b"file", b"change", 1).expect("head");
    let raw::Answer::Created(_) = world.external(
        2,
        raw::Op::Write(raw::Write::OpenPull {
            opens_empty: false,
            title: b"Named read".as_slice().into(),
            body: Box::new([]),
            head: b"work".as_slice().into(),
            base: b"main".as_slice().into(),
        }),
    ) else {
        panic!("pull opened");
    };
    let before = world.forge_calls;
    let answer = named_read(
        &mut world,
        &chat,
        93,
        b"read_pull_for",
        br#"{"repository":"org/repo","head":"work","base":"main"}"#,
    );
    assert!(matches!(answer, engine::CallAnswer::ForgeRead(ref result) if result.is_ok()), "{answer:?}");
    assert!(world.forge_calls > before, "the read reached the fake forge");
    let rendered = temper_engine_smith::answer(&answer);
    assert!(!rendered.error());
    assert!(rendered.text().starts_with(b"Pull 1 head work commit "), "{:?}", rendered.text());
    let answer = named_read(&mut world, &chat, 94, b"read_pull", br#"{"repository":"","number":1}"#);
    assert!(matches!(answer, engine::CallAnswer::ForgeRead(ref result) if result.is_ok()), "{answer:?}");
    assert!(!temper_engine_smith::answer(&answer).error());
}

#[test]
fn a_chat_reads_its_landed_pulls_state_and_the_merge_the_fake_made() {
    let (mut world, chat, producer, branch) = change_world(false, 1000, None);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
        .expect("producer pushed its branch");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Change".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.merges == 1 {
            break;
        }
    }
    assert_eq!(world.merges, 1, "the chat's change landed");
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number: 1 })) else {
        panic!("the fake's pull");
    };
    let merge = pull.merged.expect("the fake merged the pull");
    assert_ne!(merge, head, "the merge differs from the producer's head");
    let answer = named_read(&mut world, &chat, 94, b"read_pull", br#"{"repository":"org/repo","number":1}"#);
    let rendered = temper_engine_smith::answer(&answer);
    assert!(!rendered.error(), "{answer:?}");
    let mut expected = String::from("state closed merged commit ");
    for byte in fake::object_bytes(merge) {
        write!(&mut expected, "{byte:02x}").expect("merge digits");
    }
    assert!(rendered.text().windows(expected.len()).any(|part| part == expected.as_bytes()), "{:?}", rendered.text());
}

#[test]
fn an_omitted_file_limit_uses_the_engine_read_bound() {
    let (mut world, chat) = chat_world(true);
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"contents", 1)
        .expect("file head");
    let mut digits = String::new();
    for byte in translate::commit(head) {
        write!(&mut digits, "{byte:02x}").expect("commit hex");
    }
    let input = format!(r#"{{"head":"{digits}","path":"file"}}"#);
    let answer = named_read(&mut world, &chat, 93, b"read_file", input.as_bytes());
    assert!(matches!(answer, engine::CallAnswer::ForgeRead(ref result) if result.is_ok()), "{answer:?}");
    let rendered = temper_engine_smith::answer(&answer);
    assert!(rendered.text().windows(b"contents".len()).any(|part| part == b"contents"), "{:?}", rendered.text());
}

#[test]
fn a_chat_reads_the_job_attempt_printed_by_its_checks() {
    let mut world = World::configured_checks(true, true, false, 0);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let ask = adoption_chat(&world);
    let chat = assigned_chat(&mut world, 92, ask);
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"contents", 1)
        .expect("checked head");
    let digits = engine::commit_text(&translate::commit(head));
    let input = format!(r#"{{"commit":"{}"}}"#, std::str::from_utf8(&digits).expect("commit hex"));
    let checks = named_read(&mut world, &chat, 93, b"read_checks", input.as_bytes());
    let rendered = temper_engine_smith::answer(&checks);
    assert!(!rendered.error(), "{checks:?}");
    let text = std::str::from_utf8(rendered.text()).expect("check text");
    assert!(text.contains("/actions/runs/1/jobs/0\n"), "{text}");
    let line = text
        .lines()
        .find(|line| line.starts_with("read_job "))
        .unwrap_or_else(|| panic!("resolved job attempt: {text}"));
    let fields: Vec<_> = line.split_ascii_whitespace().collect();
    assert_eq!(fields.len(), 9, "{line}");
    assert_eq!([fields[0], fields[1], fields[3], fields[5], fields[7]], ["read_job", "head", "run", "job", "attempt"]);
    assert_eq!(fields[6], "1", "the API job id differs from the link's index 0");
    let input =
        format!(r#"{{"head":"{}","run":{},"job":{},"attempt":{}}}"#, fields[2], fields[4], fields[6], fields[8]);
    let job = named_read(&mut world, &chat, 94, b"read_job", input.as_bytes());
    assert!(
        matches!(&job, engine::CallAnswer::ForgeRead(result) if matches!(result.as_ref(), Ok(skein_forge::api::Answer::Job { .. }))),
        "{job:?}"
    );
    let rendered = temper_engine_smith::answer(&job);
    assert!(!rendered.error(), "{job:?}");
    let text = std::str::from_utf8(rendered.text()).expect("job text");
    assert_eq!(
        text,
        "Job 1 run 1, failed step 1 Run build:\nfailing check build 0\nfailing check build 1\nfailing check build 2\nfailing check build 3\n"
    );
}

fn pull_request_ci_world() -> World {
    let small = temper_service::limits::profile(temper_service::limits::Profile::Small);
    let mut world = World::configured_limits(
        true,
        true,
        false,
        0,
        Gate::default(),
        None,
        StoryLimits { brief: Some(small.root.brief), result_bytes: Some(512), ..StoryLimits::default() },
    );
    world.fake = fake::Domain::new(&world.fake_env.limits, 71);
    let checks = raw::Checks {
        contexts: Box::new([Box::from(*b"build")]),
        cues: Box::new([]),
        latency_min: Duration::ZERO,
        latency_max: Duration::ZERO,
        silent: 0,
        passes: 0,
        reruns: 0,
        cue: Some(raw::Cue::PullFile {
            path: Box::from(*b"changes/{pull}"),
            no_pull: raw::Check::Skipped,
            step: Box::from(*b"changes"),
            log: Box::from(*b"missing changes/{pull}"),
        }),
    };
    let setup = raw::Setup {
        id: 2,
        name: Box::from(*b"org/repo"),
        default: Box::from(*b"main"),
        tree: Box::new([]),
        labels: Box::new([]),
        checks: checks.clone(),
        workflows: Box::new([]),
        protection: None,
        hooked: false,
    };
    fake::repository(&mut world.fake, &world.fake_env.limits, setup.clone());
    fake::grant(&mut world.fake, b"org/repo", 1, raw::Permission::Admin);
    fake::grant(&mut world.fake, b"org/repo", 2, raw::Permission::Write);
    fake::grant(&mut world.fake, b"org/repo", 7, raw::Permission::Read);
    fake::repository(
        &mut world.fake,
        &world.fake_env.limits,
        raw::Setup {
            id: 3,
            name: Box::from(*b"org/other"),
            checks: raw::Checks { contexts: Box::new([Box::from(*b"build"), Box::from(*b"fmt")]), ..checks },
            ..setup
        },
    );
    for n in 0..6 {
        fake::advance(&mut world.fake, &world.fake_env, b"org/other", b"main", b"file", &[n], 1).expect("other run");
    }
    world
}

#[test]
fn a_pull_request_ci_repair_carries_the_failed_step_and_reads_its_named_attempt() {
    let (mut world, _chat, producer, branch) = change_world_in(pull_request_ci_world(), Change::default());
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"first", 1)
        .expect("producer head");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        producer_terminal(b"Small fix", 0),
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..1000 {
        world.tick();
        if world.assigned.len() == 3 {
            break;
        }
    }
    let repair = world.assigned.get(2).expect("CI repair assigned").clone();
    let raw::Answer::Runs(runs) = world.external(
        1,
        raw::Op::Read(raw::Read::Runs { workflow: None, commit: Some(head), number: None, page: 1, limit: 2 }),
    ) else {
        panic!("pull runs")
    };
    assert_eq!(runs.len(), 1, "{runs:?}");
    let run = &runs[0];
    assert_eq!((run.commit, run.event), (head, raw::RunEvent::PullRequest));
    assert_ne!(run.id, run.number);
    let brief = std::str::from_utf8(ci_words(&repair)).expect("CI brief");
    assert!(brief.contains("Failed step: 1 changes\nEnd of its log:\n"), "{brief}");
    assert!(brief.contains("missing changes/1\n"), "{brief}");
    let digits = engine::commit_text(&translate::commit(head));
    let digits = std::str::from_utf8(&digits).expect("head");
    let checks = named_read(&mut world, &repair, 195, b"read_checks", format!(r#"{{"commit":"{digits}"}}"#).as_bytes());
    let rendered = temper_engine_smith::answer(&checks);
    let text = std::str::from_utf8(rendered.text()).expect("checks text");
    let line = text
        .lines()
        .find(|line| line.starts_with("read_job "))
        .unwrap_or_else(|| panic!("resolved attempt: {checks:?}: {text}"));
    let fields: Vec<_> = line.split_ascii_whitespace().collect();
    assert_eq!(fields[2], digits);
    assert_eq!(fields[4], run.id.to_string());
    assert_ne!(fields[6], "0", "the link's index is not the job id");
    let input =
        format!(r#"{{"head":"{}","run":{},"job":{},"attempt":{}}}"#, fields[2], fields[4], fields[6], fields[8]);
    let job = named_read(&mut world, &repair, 196, b"read_job", input.as_bytes());
    let rendered = temper_engine_smith::answer(&job);
    assert!(!rendered.error());
    assert!(rendered.text().ends_with(b"missing changes/1\n"));
    world.close();
}

#[test]
fn a_decoded_job_zero_read_reaches_the_fake_and_is_answered() {
    let (mut world, chat) = chat_world(true);
    world.protocol.defer_render = true;
    let input = smith_domain_run::HostInput::attested(br#"{"head":"0100000000000000000000000000000000000000","run":2,"job":0,"attempt":1,"max_bytes":32}"#.as_slice().into()).expect("bounded input");
    let temper_engine_smith::Decoded::Call(call) = temper_engine_smith::call(
        smith_domain_run::CallName { activation: chat.attempt, completion: 1, position: 1 },
        b"read_job",
        &input,
    )
    .expect("job zero decodes") else {
        panic!("read call")
    };
    let engine::Tool::ReadNamed { read: skein_forge::api::Read::Job { attempt, .. }, .. } = &call.tool else {
        panic!("job")
    };
    let pin = *attempt;
    assert_eq!((pin.run, pin.job, pin.attempt), (2, 0, 1));
    let before = world.forge_calls;
    let mut event = world.protocol.call(Token::new(7), chat.task, chat.attempt, 93, call);
    let engine::Event::Worker(engine::WorkerInput::Link {
        link: engine::Link::Up { up: host::LinkUp::Call { tool, .. }, .. },
        ..
    }) = &mut event
    else {
        panic!("worker call")
    };
    *tool = b"read_job".as_slice().into();
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if !world.protocol.pending_renders.is_empty() {
            break;
        }
    }
    assert!(world.forge_calls > before, "read reaches the independent fake");
    let (to, name) = world.protocol.pending_renders.pop().expect("refusal to render");
    let answer = &world.protocol.answers.last().expect("typed answer").1;
    assert!(
        matches!(answer, engine::CallAnswer::ForgeRead(result) if result.as_ref() == &Err(skein_forge::api::Error::MissingJob))
    );
    assert_eq!(world.protocol.read_jobs.get(&name), Some(&pin));
    let event = temper_service::render_for_task(
        to,
        answer,
        b"read_job",
        1024,
        chat.task,
        1024,
        temper_engine_smith::ForgeReadBounds::STOCK,
        world.protocol.read_jobs.get(&name).copied(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if !world.call_errors.is_empty() {
            break;
        }
    }
    assert_eq!(world.call_errors.last().expect("host refusal").text.as_ref(), b"Forge read failed: job missing. No job attempt at head 0100000000000000000000000000000000000000, run 2, job 0, attempt 1, as asked. read_checks at this head names each Actions job's run, job and attempt as read_job takes them; a link's run number and its index of the job are not those ids.");
    let mut observed = false;
    while let Some(fact) = world.root.pop_fact() {
        if let engine::Fact::ReadRefused { task, attempt, tool, layer, error, job, .. } = fact {
            assert_eq!(
                (task, attempt, tool.as_ref(), layer, error),
                (
                    chat.task,
                    chat.attempt,
                    b"read_job".as_slice(),
                    "forge-client",
                    Some(skein_forge::api::Error::MissingJob)
                )
            );
            assert_eq!(job, Some(pin), "diagnostic names what was asked");
            observed = true;
        }
    }
    assert!(observed, "refused read diagnostic");
}

#[test]
fn a_chats_two_kib_change_lands_with_its_words_whole_in_the_producers_brief() {
    const WORD_BYTES: usize = 2048;
    let mut words = vec![b'x'; WORD_BYTES];
    words[..b"Export CSV\n".len()].copy_from_slice(b"Export CSV\n");
    let small = temper_service::limits::profile(temper_service::limits::Profile::Small);
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits { brief: Some(small.root.brief), spec_bytes: Some(4096), ..StoryLimits::default() },
    );
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let ask = adoption_chat(&world);
    let chat = assigned_chat(&mut world, 92, ask);
    let engine::CallAnswer::Delegated(made) = change(&mut world, &chat, 93, None, &words) else { panic!("delegated") };
    let change_task = made[0].task;
    world.until(Until::SecondAssignment);
    let producer = world.assigned[1].clone();
    assert!(task_brief(&producer).windows(words.len()).any(|part| part == words));
    let row = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::Change(row) if row.task == change_task => Some(row),
                _ => None,
            },
            Record::Core(_) => None,
        })
        .expect("registered past the operation bound");
    assert_eq!(row.body.as_ref(), words.as_slice());
    let branch = row.branch.clone();
    land_change(&mut world, &producer, &branch);
    assert_eq!(world.merges, 1, "the long change lands on the independent fake forge");
}

pub(super) fn land_change(world: &mut World, producer: &engine::Assignment, branch: &[u8]) {
    land_titled_change(world, producer, branch, b"Small fix");
}

fn land_titled_change(world: &mut World, producer: &engine::Assignment, branch: &[u8], title: &[u8]) {
    push_change(world, branch);
    let tasks::Contract::Change { connector, .. } = live(world, producer.task).contract else {
        panic!("change contract")
    };
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        producer_terminal(title, connector),
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..1000 {
        if world.merges == 1 {
            break;
        }
        world.tick();
    }
}

fn producer_terminal(title: &[u8], connector: u16) -> tasks::End {
    use smith_domain_run::{Answer, Spend, outcome};
    temper_engine_smith::result(
        Answer::Accepted {
            outcome: outcome::Declared::Change(outcome::Change {
                fields: Box::new([
                    outcome::Field { name: b"title".as_slice().into(), value: title.into() },
                    outcome::Field { name: b"body".as_slice().into(), value: b"pushed".as_slice().into() },
                ]),
            }),
            spent: Spend {
                input: None,
                output: None,
                cache_read: None,
                cache_write: None,
                reasoning: None,
                units: 5,
                turns: 1,
            },
            turns: 1,
        },
        Some(temper_engine_smith::ChangeResource {
            connector,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
        }),
    )
    .expect("producer result")
    .end
}

#[test]
fn producer_titles_name_changes_when_the_brief_opens_with_context() {
    for (words, title) in [
        (b"CSV has landed on main (PR #2, acd48a6). Add JSON export.".as_slice(), b"Add JSON export".as_slice()),
        (b"In src/main.rs (which has `const TABLE`), add CSV export.".as_slice(), b"Add CSV export".as_slice()),
        (b"Context. Add a versioned export.".as_slice(), b"Add v1.2. JSON export! Keep CSV.".as_slice()),
    ] {
        let (mut world, producer) = producer(words);
        let assigned = world.assigned[1].clone();
        let tasks::Party::Task(change) = producer.requester else { panic!("change requester") };
        let branch = world
            .store
            .rows
            .values()
            .find_map(|record| match record {
                Record::Forge { row } => match row.as_ref() {
                    forge_top::Stored::Change(row) if row.task == change => Some(row.branch.clone()),
                    _ => None,
                },
                Record::Core(_) => None,
            })
            .expect("change branch");
        land_titled_change(&mut world, &assigned, &branch, title);
        assert_eq!(world.merges, 1, "the titled change lands");
        let raw::Answer::Item { item: pull, .. } =
            world.external(1, raw::Op::Read(raw::Read::Item { number: 1, after: 0, since: None }))
        else {
            panic!("created pull")
        };
        assert_eq!(pull.title.as_ref(), title);
        assert!(task_brief(&assigned).windows(words.len()).any(|part| part == words));
    }
}

fn handed_title(title: &[u8]) -> Box<[u8]> {
    let (mut world, chat) = chat_world(true);
    world.fake_env.limits.limits.title_bytes = 256;
    let branch = b"temper/handed".as_slice();
    push_change(&mut world, branch);
    let mut delegate =
        temper_engine_domain_world::fixtures::report_delegate(b"Context. The complete brief.", Box::new([]));
    delegate.executor = tasks::Executor::Procedure { connector: 0, code: 2 };
    delegate.contract = tasks::Contract::Change { connector: 0, kind: 1, words: 128 };
    delegate.authority = live(&world, chat.task).authority.clone();
    delegate.authority.budget.spend = 40;
    delegate.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    delegate.spec.parameters = Box::new([
        tasks::Parameter::Resource {
            name: 1,
            connector: 0,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
        },
        tasks::Parameter::Bytes { name: 2, value: b"main".as_slice().into() },
        tasks::Parameter::Bytes { name: 3, value: branch.into() },
        tasks::Parameter::Bytes { name: 10, value: title.into() },
    ]);
    let event = world.protocol.call(
        Token::new(7),
        chat.task,
        chat.attempt,
        93,
        engine::Call { completion: 1, position: 1, tool: engine::Tool::Delegate { batch: Box::new([delegate]) } },
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.merges == 1 {
            break;
        }
    }
    assert_eq!(world.merges, 1, "the handed branch lands: {:?}", world.answers);
    assert_eq!(world.assigned.len(), 1, "no producer is needed");
    let raw::Answer::Item { item, .. } =
        world.external(1, raw::Op::Read(raw::Read::Item { number: 1, after: 0, since: None }))
    else {
        panic!("handed pull")
    };
    assert_eq!(item.body.as_ref(), b"Context. The complete brief.");
    item.title
}

#[test]
fn handed_branches_keep_explicit_titles_and_fit_only_the_forge_bound() {
    let punctuation = b"Export v1.2. JSON! Keep CSV.";
    assert_eq!(handed_title(punctuation).as_ref(), punctuation);
    for length in [255, 256, 257] {
        let title = vec![b'x'; length];
        let expected =
            if length <= 256 { title.clone() } else { [vec![b'x'; 253], "…".as_bytes().to_vec()].concat() };
        assert_eq!(handed_title(&title).as_ref(), expected);
    }
    let title = "é".repeat(129);
    let expected = format!("{}…", "é".repeat(126));
    assert_eq!(handed_title(title.as_bytes()).as_ref(), expected.as_bytes());
}

fn push_change(world: &mut World, branch: &[u8]) {
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.into(), commit: 1 }));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", branch, b"file", b"fixed", 1)
        .expect("producer pushes its work");
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: head,
            context: b"build".as_slice().into(),
            state: raw::Check::Passed,
            description: Box::new([]),
            link: Box::new([]),
        }),
    );
}

#[test]
fn questions_are_a_plans_member() {
    let (mut world, chat) = plan_world(false);
    let rows = world.store.rows.clone();
    let assigned = world.assigned.len();
    assert_eq!(plan_change(&mut world, &chat, br#"{"words":"Small fix","questions":"none"}"#), None);
    assert_eq!(world.call_errors.len(), 1);
    assert_eq!(world.call_errors[0].text.as_ref(), b"change's landing, branch, name, after, questions, repairs, changes, producer and llm are a plan's: this task's authority lacks the control family (amend, cancel, release).");
    assert_eq!(world.store.rows, rows, "no delegate or call record is built");
    assert_eq!(world.assigned.len(), assigned, "nothing is delegated");
}

#[test]
fn questions_foreman_is_offered_alone_and_in_a_batch() {
    const ONE_CHANGE: usize = 1;
    for (input, count) in [
        (br#"{"words":"Small fix","questions":"foreman"}"#.as_slice(), ONE_CHANGE),
        (
            br#"{"changes":[{"name":"a","words":"First fix"},{"words":"Small fix","questions":"foreman","after":["a"]}]}"#.as_slice(),
            TWO_CHANGES,
        ),
    ] {
        let (mut world, chat) = plan_world(true);
        let answer = plan_change(&mut world, &chat, input);
        let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("delegated: {answer:?}") };
        assert_eq!(made.len(), count);
        assert!(world.call_errors.is_empty());
        let task = made.last().expect("foreman change").task;
        let change = live(&world, task);
        assert_eq!(change.spec.words.as_ref(), b"Small fix");
        assert!(change.spec.parameters.contains(&tasks::Parameter::Number { name: 14, value: 1 }));
        assert!(change.authority.delegation.kinds.contains(&tasks::AuthorityExecutor::Charter(6)));
    }
}

#[test]
fn questions_none_makes_the_change_as_today() {
    let (mut world, chat) = plan_world(true);
    let answer = plan_change(&mut world, &chat, br#"{"words":"Small fix","questions":"none"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("delegated: {answer:?}") };
    assert_eq!(made.len(), 1);
    assert_eq!(live(&world, made[0].task).spec.words.as_ref(), b"Small fix");
    world.until(Until::SecondAssignment);
    let producer = world.assigned[1].clone();
    let branch = stored_change(&world, made[0].task).branch.clone();
    land_change(&mut world, &producer, &branch);
    assert_eq!(world.merges, 1, "the change lands on the fake forge");
}
