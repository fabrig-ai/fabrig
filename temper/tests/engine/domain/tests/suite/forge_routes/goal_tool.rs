//! A goal uses ordinary delegation admission and can plan before making a change.
use super::*;

const WORDS: &[u8] = b"Plan the fix, ask me, then make it";
const ADVISOR_SUFFIX: &[u8] = b"\n\nYou are an advisor: you read what your brief asks and report what you found; you change nothing and ask no one.";
const GOAL_SUFFIX: &[u8] = b"\n\nYou are a goal: the work your person asked for, yours to plan and to change; you start no goal but an advisor.";
const AGREED_SUFFIX: &[u8] = b"\n\nYou are a goal: the work your person asked for, yours to plan and to change; you start no goal but an advisor.\nThe work is to be agreed: before anything is changed, ask your requester your plan with message, as a question, and wait for the answer.";
const PERSON_HEADING: &[u8] = b"\n\nYour person's words, whole:\n";
const PERSON_REQUEST: &[u8] = b"First: read the code and ask me.\nThen: keep the second paragraph too.";
const PERSON_BYTES: usize = 128; // The root world's existing people.words bound and adoption request size.
const GOAL_CALL: u64 = 93; // The fixture's first call name, after chat key 92.
const AT_BOUND: usize = 0;
const ONE_PAST_BOUND: usize = 1; // One additional byte crosses the spec's bound.
const NESTED_GOAL_CALL: u64 = GOAL_CALL + 1;
const CONTINUED_CALL: u64 = NESTED_GOAL_CALL + 1;
const ASK_PLAN: &[u8] =
    b"this work is to be agreed: ask your requester your plan with message, as a question, and wait";

#[test]
fn an_agreed_goals_nested_goal_is_refused_with_its_plan_instruction() {
    for (advisor, agree) in [(false, true), (false, false), (true, true)] {
        let (mut world, chat, run) = running_goal_with_agreement(true);
        refused_tool(
            &mut world,
            &run,
            NESTED_GOAL_CALL,
            engine::Tool::Goal { words: WORDS.into(), advisor, agree },
            ASK_PLAN,
        );
        ask_plan_and_wait(&mut world, &chat, &run);
    }
}

#[test]
fn an_answered_agreed_goals_nested_goal_points_to_change() {
    const REQUEST: &[u8] = b"Add CSV export to widget. Before anything is changed, have the work tell me its plan and wait for my go-ahead.";
    const APPROVAL: &[u8] = b"Go ahead. The CSV's first line must be the header fruit,count.";
    let (mut world, chat, run) = running_goal_for_request(true, REQUEST);
    refused_tool(
        &mut world,
        &run,
        NESTED_GOAL_CALL,
        engine::Tool::Goal { words: REQUEST.into(), advisor: false, agree: true },
        ASK_PLAN,
    );
    let question = ask_plan_and_wait(&mut world, &chat, &run);
    let resumed = answer_plan(&mut world, &chat, &run, question, APPROVAL);
    for name in [CONTINUED_CALL + 4, CONTINUED_CALL + 6] {
        refused_tool(
            &mut world,
            &resumed,
            name,
            engine::Tool::Goal { words: REQUEST.into(), advisor: false, agree: true },
            b"a goal starts an advisor alone: set advisor, or make the change with change",
        );
    }
    let change = delegated(call(
        &mut world,
        &resumed,
        CONTINUED_CALL + 8,
        engine::Tool::Change { changes: Box::new([engine::ChangeCall::words(APPROVAL.into(), None)]) },
    ));
    assert_eq!(live(&world, change).requester, tasks::Party::Task(run.task));
    assert_eq!(live(&world, change).spec.words.as_ref(), APPROVAL);
    assert!(live(&world, run.task).spec.asks, "answering preserves the agreement mark");
    let chat_work: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))
                if row.requester == tasks::Party::Task(chat.task) =>
            {
                Some(row.number)
            }
            _ => None,
        })
        .collect();
    assert_eq!(chat_work, [run.task], "the chat starts no replacement work");
    world.close();
}

fn answer_plan(
    world: &mut World,
    chat: &engine::Assignment,
    run: &engine::Assignment,
    question: u64,
    words: &[u8],
) -> engine::Assignment {
    const ANSWER_KEY: u8 = 110; // Distinct from chat creation and tool call keys.
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(ANSWER_KEY))),
        sign_in: world.signed_in.expect("owner session"),
        key: [ANSWER_KEY; 16],
        ask: people::Ask::AnswerQuestion { project: 1, task: chat.task, question, words: words.into() },
    }));
    world.until(Until::Ready);
    let resumed = world.assigned.iter().rev().find(|a| a.task == run.task).expect("goal resumed").clone();
    assert!(resumed.attempt > run.attempt, "answer wakes a new run");
    let row = live(world, run.task);
    assert!(row.asked_requester && row.questions.is_empty(), "the question is answered");
    assert!(
        row.inbox
            .iter()
            .any(|message| message.kind == tasks::MessageKind::Answer { question } && message.words.as_ref() == words),
        "the approved header reaches the goal"
    );
    resumed
}

fn ask_plan_and_wait(world: &mut World, chat: &engine::Assignment, run: &engine::Assignment) -> u64 {
    const PLAN: &[u8] = b"Add CSV export with a header. Go ahead?";
    let engine::CallAnswer::Sent { message } = call(
        world,
        run,
        NESTED_GOAL_CALL + 2,
        engine::Tool::Message { target: chat.task, form: engine::MessageForm::Question, words: PLAN.into() },
    ) else {
        panic!("plan question sent after refusal")
    };
    assert_plan_inbox(world, chat.task, message, PLAN);
    refused_tool(
        world,
        run,
        NESTED_GOAL_CALL + 3,
        engine::Tool::Change {
            changes: Box::new([engine::ChangeCall::words(b"Add CSV export".as_slice().into(), None)]),
        },
        format!("wait for the answer to question {message}").as_bytes(),
    );
    let read = Some(live(world, run.task).last_message);
    world.send(protocol::turn(Token::new(7), run.task, run.attempt, 1, 0, read, b"Wait for the answer"));
    world.until(Until::Ready);
    let event =
        world.protocol.answer(Token::new(7), run.task, run.attempt, 0, tasks::End::Parked, engine::Parts::none());
    world.send(event);
    world.until(Until::Ready);
    assert_eq!(live(world, run.task).phase, tasks::Phase::Active(tasks::Active::Idle));
    assert_eq!(live(world, run.task).questions.len(), 1);
    assert_eq!(world.assigned.len(), 2, "no change starts before the answer");
    assert!(
        !world.store.rows.values().any(|record| matches!(record,
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))
                if row.requester == tasks::Party::Task(run.task)
        )),
        "no change task before the answer"
    );
    assert!(!world.writes.iter().any(|write| matches!(
        write,
        skein_forge::api::Write::OpenPull { .. } | skein_forge::api::Write::CreateBranch { .. }
    )));
    assert_eq!(world.merges, 0);
    message
}

fn assert_plan_inbox(world: &mut World, chat: u64, message: u64, plan: &[u8]) {
    use jig_domain_core::{documents, reads};
    let reads::Answer::Page(page) = world.party_read(reads::Read::Inbox {
        group: reads::Group::Questions,
        filter: reads::Filter { project: Some(1), goal: None },
        from: None,
    }) else {
        panic!("person's question inbox")
    };
    assert!(page.rows.iter().any(|r| matches!(r, reads::PageRow::Inbox(entry)
        if matches!(&entry.task, documents::TaskLink::Live(chip) if chip.number == chat)
        && entry.object == documents::InboxObject::Question { message })));
    let reads::Answer::Document(document) = world.party_read(reads::Read::Question { task: chat, message }) else {
        panic!("person reads the plan question")
    };
    let documents::Document::Question(question) = document.as_ref() else { panic!("question document") };
    assert_eq!(question.words.bytes.as_ref(), plan);
}

fn chat_world(cap: u64, tasks: u32, depth: u32) -> (World, engine::Assignment) {
    start_chat(World::configured(true, true), cap, tasks, depth, None, None)
}

pub(super) fn chat_with_note_scope(scopes: authority::Scopes) -> (World, engine::Assignment) {
    task_with_note_scope(scopes, None)
}

fn task_with_note_scope(scopes: authority::Scopes, tracked: Option<u32>) -> (World, engine::Assignment) {
    let world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits { notes: Some(scopes), ..StoryLimits::default() },
    );
    start_chat(world, 400, 11, 3, None, tracked)
}

fn start_chat(
    mut world: World,
    cap: u64,
    tasks: u32,
    depth: u32,
    person_words: Option<&[u8]>,
    tracked: Option<u32>,
) -> (World, engine::Assignment) {
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { panic!("create") };
    if let Some(words) = person_words {
        batch[0].words = words.into();
    }
    batch[0].tracked = tracked;
    batch[0].budget = people::Budget::AsAllowed { cap: Some(cap) };
    batch[0].contract = people::Contract::Report { words: 32 };
    batch[0].authority.tools = 1 | 2 | 4 | 512;
    batch[0].authority.delegation.tasks = tasks;
    batch[0].authority.delegation.depth = depth;
    let chat = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    (world, chat)
}

fn call(world: &mut World, run: &engine::Assignment, name: u64, tool: engine::Tool) -> engine::CallAnswer {
    let before = world.answers.len();
    send_call(world, run, name, tool);
    for _ in 0..200 {
        world.tick();
        if world.answers.len() > before {
            return world.answers[before].clone();
        }
    }
    panic!("call was not answered");
}

fn send_call(world: &mut World, run: &engine::Assignment, name: u64, tool: engine::Tool) {
    let event = world.protocol.call(
        Token::new(7),
        run.task,
        run.attempt,
        name,
        engine::Call { completion: u32::try_from(name).expect("bounded call name"), position: 1, tool },
    );
    world.send(event);
}

fn goal(world: &mut World, run: &engine::Assignment, name: u64, advisor: bool) -> engine::CallAnswer {
    call(world, run, name, engine::Tool::Goal { agree: false, words: WORDS.into(), advisor })
}

fn live(world: &World, task: u64) -> &tasks::TaskRecord {
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task))))
    else {
        panic!("live task {task}")
    };
    row
}

fn delegated(answer: engine::CallAnswer) -> u64 {
    let engine::CallAnswer::Delegated(made) = answer else { panic!("delegated: {answer:?}") };
    assert_eq!(made.len(), 1);
    made[0].task
}

#[test]
fn a_chats_goal_is_unchanged() {
    let (mut world, chat) = chat_world(100, 11, 3);
    let task = delegated(goal(&mut world, &chat, 93, false));
    let row = live(&world, task);
    assert_eq!(row.executor, tasks::Executor::Agent { charter: 1 });
    assert_eq!(row.requester, tasks::Party::Task(chat.task));
    assert_eq!(row.project, live(&world, chat.task).project);
    assert_eq!(row.tracked, Some(0));
    assert_eq!(row.label, 2);
    assert!(row.flags.listed);
    assert!(matches!(row.contract, tasks::Contract::Report { .. }));
    assert_eq!(row.spec.words.as_ref(), [WORDS, GOAL_SUFFIX].concat());
    assert!(row.spec.parameters.is_empty() && row.spec.inputs.is_empty());
    assert_eq!(delegated(goal(&mut world, &chat, 93, false)), task, "named retry makes no second goal");
}

#[test]
fn a_goals_report_is_bounded_by_the_deployments_result_bytes() {
    let (mut world, chat, run) = running_goal();
    let tasks::Contract::Report { words } = live(&world, chat.task).contract else { panic!("chat report") };
    let result_bytes = world.env.limits.tasks.result_bytes;
    assert!(words < result_bytes, "the chat's own contract is smaller");
    let advisor = delegated(goal(&mut world, &run, NESTED_GOAL_CALL, true));
    assert_eq!(live(&world, run.task).contract, tasks::Contract::Report { words: result_bytes });
    assert_eq!(live(&world, advisor).contract, tasks::Contract::Report { words: result_bytes });
}

#[test]
fn a_goals_authority_is_the_runs_narrowed() {
    let (mut world, chat) = chat_world(100, 11, 3);
    let parent = live(&world, chat.task).clone();
    let task = delegated(goal(&mut world, &chat, 93, false));
    let row = live(&world, task);
    assert_eq!(row.authority.tools, parent.authority.tools);
    assert_eq!(
        row.authority.grants,
        parent
            .authority
            .grants
            .iter()
            .filter(|g| g.connector == 0 && [1, 2, 3, 4, 10, 11, 12, 17].contains(&g.kind))
            .cloned()
            .collect::<Box<[_]>>()
    );
    assert_eq!(row.authority.delegation.kinds, parent.authority.delegation.kinds);
    assert_eq!(row.authority.delegation.tasks, 10);
    assert_eq!(row.authority.delegation.depth, 2);
    assert_eq!(row.authority.notes, tasks::Scopes(0));
    assert!(row.authority.note_resources.is_empty());
    assert_eq!(
        row.authority.budget.spend,
        (parent.numbers.budget - parent.numbers.spent - parent.numbers.spent_below - parent.numbers.reserved) / 2
    );
    assert_eq!(row.authority.budget.deadline, parent.authority.budget.deadline);
    assert_eq!(row.authority.models, parent.authority.models);
}

// temper/docs/design/domain/agent/agents.md, 2: a chat's goal cannot address a person.
#[test]
fn a_chats_goal_keeps_no_person_executor() {
    let world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits { person_executor: true, ..StoryLimits::default() },
    );
    let (mut world, chat) = start_chat(world, 100, 11, 3, None, None);
    assert!(live(&world, chat.task).authority.delegation.kinds.contains(&tasks::AuthorityExecutor::Role(0)));
    let task = delegated(goal(&mut world, &chat, GOAL_CALL, false));
    assert!(
        !live(&world, task)
            .authority
            .delegation
            .kinds
            .iter()
            .any(|kind| matches!(kind, tasks::AuthorityExecutor::Role(_)))
    );
    world.close();
}

#[test]
fn a_goal_keeps_its_own_note_scope() {
    for (held, expected) in [(0, 0), (2, 0), (7, 1)] {
        let (mut world, chat) = chat_with_note_scope(authority::Scopes(held));
        let task = delegated(goal(&mut world, &chat, GOAL_CALL, false));
        assert_eq!(live(&world, task).authority.notes, tasks::Scopes(expected));
        assert!(live(&world, task).authority.note_resources.is_empty());
    }
}

#[test]
fn an_advisor_keeps_no_note_scope() {
    let (mut world, run) = task_with_note_scope(authority::Scopes(7), Some(0));
    let advisor = delegated(goal(&mut world, &run, GOAL_CALL, true));
    assert_eq!(live(&world, advisor).authority.notes, tasks::Scopes(0));
    assert!(live(&world, advisor).authority.note_resources.is_empty());
}

#[test]
fn an_untracked_chats_goal_comes_last_among_the_projects_goals() {
    for (existing, expected) in [(8, 9), (u32::MAX, u32::MAX)] {
        let (mut world, chat) = chat_world(400, 11, 3);
        let mut first = temper_engine_domain_world::fixtures::report_delegate(b"existing goal", Box::new([]));
        first.tracked = Some(existing);
        first.authority.tools = tasks::Tools(1 | 2 | 4 | 512);
        let earlier = delegated(call(&mut world, &chat, 93, engine::Tool::Delegate { batch: Box::new([first]) }));
        let task = delegated(goal(&mut world, &chat, 94, false));
        assert_eq!(live(&world, task).tracked, Some(expected));
        assert!(task > earlier, "equal priorities are ordered by task number");
    }
}

#[test]
fn an_advisor_reads_alone_untracked() {
    let (mut world, _, run) = running_goal();
    let task = delegated(goal(&mut world, &run, NESTED_GOAL_CALL, true));
    let row = live(&world, task);
    assert_eq!(row.tracked, None);
    assert_eq!(row.label, 0);
    assert_eq!(row.flags, tasks::Flags::PLAIN);
    assert_eq!(row.authority.tools, tasks::Tools(jig_domain_core::ToolFamilies::JIG.read.0));
    assert!(!row.authority.grants.is_empty());
    assert!(row.authority.grants.iter().all(|g| g.connector == 0 && g.kind == 1));
    assert!(row.authority.delegation.kinds.is_empty());
    assert_eq!(row.authority.delegation.tasks, 0);
    assert_eq!(row.authority.delegation.depth, 0);
    assert_eq!(row.authority.notes, tasks::Scopes(0));
    assert!(row.authority.note_resources.is_empty());
}

#[test]
fn a_goal_with_one_task_left_is_refused_as_the_tree_limit() {
    let (mut world, chat) = chat_world(100, 1, 3);
    assert!(
        matches!(goal(&mut world, &chat, 93, false), engine::CallAnswer::DelegationRefused(p) if p.why == tasks::Refusal::Tree)
    );
}

#[test]
fn a_goal_with_one_level_left_is_refused_as_the_depth_limit() {
    let (mut world, chat) = chat_world(100, 11, 1);
    assert!(
        matches!(goal(&mut world, &chat, 93, false), engine::CallAnswer::DelegationRefused(p) if p.why == tasks::Refusal::Depth)
    );
}

#[test]
fn a_goal_with_half_the_remainder_at_minimum_is_refused_as_funding() {
    let (mut world, chat) = chat_world(62, 11, 3);
    assert!(
        matches!(goal(&mut world, &chat, 93, false), engine::CallAnswer::DelegationRefused(p) if p.why == tasks::Refusal::Funding)
    );
}

fn running_goal() -> (World, engine::Assignment, engine::Assignment) {
    running_goal_with_agreement(false)
}

#[test]
fn a_chats_ended_goal_is_listed_by_project() {
    use jig_domain_core::{documents, reads};
    let (mut world, _, run) = running_goal();
    let project = live(&world, run.task).project;
    let event = world.protocol.answer(
        Token::new(7),
        run.task,
        run.attempt,
        1,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    let ended = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(run.task)));
    for _ in 0..200 {
        world.tick();
        if world.store.rows.contains_key(&ended) {
            break;
        }
    }
    assert!(world.store.rows.contains_key(&ended), "the goal ended");
    assert!(
        world.store.rows.values().any(|record| matches!(record,
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::EndedByProject(row)))
                if row.project == project && row.task == run.task && row.label == 2
        )),
        "the ended goal is indexed by project and goal label"
    );
    for label in [None, Some(2)] {
        let query = documents::TaskQuery {
            project: Some(project),
            requester: None,
            phase: Some(documents::PhaseClass::Ended),
            label,
            ..jig_test_profile::fixtures::OWN_TASKS
        };
        let answer = super::paths::read(&mut world, reads::Read::Tasks { query, live: None, ended: None });
        let Some(reads::Answer::Tasks(page)) = answer else { panic!("ended project tasks: {answer:?}") };
        assert!(page.live.is_empty());
        let reads::EndedGroup::Rows { rows, next, cut } = &page.ended else { panic!("ended project group") };
        assert_eq!(rows.len(), 1, "project query with label {label:?}");
        assert_eq!(rows.get(0).expect("one ended goal").chip.number, run.task);
        assert!(next.is_none() && !cut);
    }
}

fn running_goal_with_agreement(agree: bool) -> (World, engine::Assignment, engine::Assignment) {
    running_goal_for_request(agree, PERSON_REQUEST)
}

fn running_goal_for_request(agree: bool, request: &[u8]) -> (World, engine::Assignment, engine::Assignment) {
    let (mut world, chat) = start_chat(World::configured(true, true), 400, 11, 3, Some(request), None);
    let task = delegated(call(
        &mut world,
        &chat,
        GOAL_CALL,
        engine::Tool::Goal { words: WORDS.into(), advisor: false, agree },
    ));
    world.until(Until::SecondAssignment);
    let run = world.assigned.iter().find(|a| a.task == task).expect("goal assigned").clone();
    (world, chat, run)
}

#[test]
fn a_goal_calls_change_and_its_change_is_made() {
    let (mut world, _, run) = running_goal();
    let charter = temper_engine_smith::charter(&run);
    let change_tool =
        charter.grants.host_tools.iter().find(|tool| tool.name.as_ref() == b"change").expect("change offered");
    let schema: serde_json::Value = serde_json::from_slice(&change_tool.schema).expect("change schema");
    for member in ["changes", "name", "landing", "after", "branch"] {
        assert!(schema["properties"].get(member).is_some(), "plan member {member} offered");
    }
    let change = delegated(call(
        &mut world,
        &run,
        94,
        engine::Tool::Change { changes: Box::new([engine::ChangeCall::words(b"Small fix".as_slice().into(), None)]) },
    ));
    let row = live(&world, change);
    assert_eq!(row.requester, tasks::Party::Task(run.task));
    assert_eq!(row.executor, tasks::Executor::Procedure { connector: 0, code: 2 });
    for _ in 0..200 {
        world.tick();
        if world.store.rows.values().any(|r| {
            matches!(r, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))
            if row.requester == tasks::Party::Task(change) && matches!(row.executor, tasks::Executor::Agent { .. }))
        }) {
            return;
        }
    }
    panic!("change producer not delegated");
}

#[test]
fn a_goals_advisor_is_admitted() {
    for agree in [false, true] {
        let (mut world, _, run) = running_goal_with_agreement(agree);
        let advisor = delegated(goal(&mut world, &run, 94, true));
        assert_eq!(live(&world, advisor).requester, tasks::Party::Task(run.task));
        assert_eq!(live(&world, advisor).tracked, None);
    }
}

#[test]
fn a_goal_asks_its_requester_and_the_person_sees_the_question() {
    use jig_domain_core::{documents, reads};
    let (mut world, chat, run) = running_goal();
    let engine::CallAnswer::Sent { message } = call(
        &mut world,
        &run,
        94,
        engine::Tool::Message {
            target: chat.task,
            form: engine::MessageForm::Question,
            words: b"Shall I make this plan?".as_slice().into(),
        },
    ) else {
        panic!("question sent")
    };
    let reply_to = ReplyTo::new(Token::new(95));
    world.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to,
        sign_in: world.signed_in.expect("owner session"),
        read: reads::Read::Inbox {
            group: reads::Group::Questions,
            filter: reads::Filter { project: Some(1), goal: None },
            from: None,
        },
    }));
    for _ in 0..200 {
        world.tick();
        if let Some((_, reads::Answer::Page(page))) =
            world.reads.iter().find(|(to, _)| *to == ReplyTo::new(Token::new(95)))
        {
            assert!(page.rows.iter().any(|r| matches!(r, reads::PageRow::Inbox(entry)
                if matches!(&entry.task, documents::TaskLink::Live(chip) if chip.number == chat.task)
                && entry.object == documents::InboxObject::Question { message })));
            return;
        }
    }
    panic!("person's inbox not read");
}

fn refused_goal(world: &mut World, run: &engine::Assignment, advisor: bool, expected: &[u8]) {
    refused_tool(
        world,
        run,
        NESTED_GOAL_CALL,
        engine::Tool::Goal { agree: false, words: WORDS.into(), advisor },
        expected,
    );
}

fn refused_tool(world: &mut World, run: &engine::Assignment, name: u64, tool: engine::Tool, expected: &[u8]) {
    world.until(Until::Ready);
    let (applied, rows) = (world.store.applied, world.store.rows.clone());
    let errors = world.call_errors.len();
    send_call(world, run, name, tool);
    world.until(Until::Ready);
    assert_eq!(
        world.call_errors[errors..],
        [temper_engine_domain_world::forge_routes::CallError {
            call: name.to_be_bytes().into(),
            text: expected.into()
        }]
    );
    assert_eq!(world.store.applied, applied, "the refusal commits nothing");
    assert_eq!(world.store.rows, rows, "the refusal makes no task or call record");
    assert!(
        matches!(
            call(world, run, name + 1, engine::Tool::ReadForge {
                repository: forge_world::REPO,
                read: skein_forge::api::Read::Branch { branch: b"main".as_slice().into() },
            }),
            engine::CallAnswer::ForgeRead(answer) if answer.is_ok()
        ),
        "the same run's next call is served"
    );
}

#[test]
fn a_goals_goal_without_advisor_is_refused_by_name() {
    let (mut world, _, run) = running_goal();
    refused_goal(
        &mut world,
        &run,
        false,
        b"a goal starts an advisor alone: set advisor, or make the change with change",
    );
}

#[test]
fn a_chats_advisor_is_refused_by_name() {
    let (mut world, chat) = chat_world(100, 11, 3);
    refused_goal(
        &mut world,
        &chat,
        true,
        b"to change a repository, call change; a goal is for work they want to agree first or a survey, and its goal asks for an advisor",
    );
}

#[test]
fn a_goal_uses_the_subtrees_remaining_task_allotment() {
    let (mut world, chat) = chat_world(400, 11, 3);
    delegated(goal(&mut world, &chat, 93, false));
    let task = delegated(goal(&mut world, &chat, 94, false));
    assert_eq!(live(&world, task).authority.delegation.tasks, 9);
}

#[test]
fn oversized_goal_words_are_refused_as_a_delegate_spec() {
    let (mut world, chat) = chat_world(100, 11, 3);
    let words = vec![b'x'; skein_lib::usize_of(world.env.limits.tasks.spec_bytes) + 1];
    assert!(
        matches!(call(&mut world, &chat, 93, engine::Tool::Goal { agree: false, words: words.into(), advisor: false }),
        engine::CallAnswer::DelegationRefused(p) if p.why == tasks::Refusal::Spec)
    );
}

#[test]
fn an_advisors_brief_puts_the_roots_line_after_the_callers_words() {
    let (mut world, _, run) = running_goal();
    let task = delegated(goal(&mut world, &run, NESTED_GOAL_CALL, true));
    assert_eq!(live(&world, task).spec.words.as_ref(), [WORDS, ADVISOR_SUFFIX].concat());
}

fn advisor_at_spec_edge(extra: usize) -> (World, engine::CallAnswer) {
    let (mut world, _, run) = running_goal();
    let bound = skein_lib::usize_of(world.env.limits.tasks.spec_bytes);
    let words = vec![b'x'; bound - ADVISOR_SUFFIX.len() + extra];
    let answer = call(
        &mut world,
        &run,
        NESTED_GOAL_CALL,
        engine::Tool::Goal { agree: false, words: words.into(), advisor: true },
    );
    (world, answer)
}

#[test]
fn an_advisors_brief_fits_at_the_spec_bound() {
    let (world, answer) = advisor_at_spec_edge(AT_BOUND);
    let task = delegated(answer);
    assert_eq!(live(&world, task).spec.words.len(), skein_lib::usize_of(world.env.limits.tasks.spec_bytes));
}

#[test]
fn an_advisors_brief_past_the_spec_bound_is_refused() {
    let (_, answer) = advisor_at_spec_edge(ONE_PAST_BOUND);
    assert!(matches!(answer, engine::CallAnswer::DelegationRefused(p) if p.why == tasks::Refusal::Spec));
}

#[test]
fn agree_with_advisor_is_refused_by_name() {
    let (mut world, chat) = chat_world(400, 11, 3);
    refused_tool(
        &mut world,
        &chat,
        NESTED_GOAL_CALL,
        engine::Tool::Goal { words: WORDS.into(), advisor: true, agree: true },
        b"an advisor agrees nothing: set agree or advisor, not both",
    );
}

#[test]
fn a_goals_goal_with_agree_is_refused_by_name() {
    let (mut world, _, run) = running_goal();
    refused_tool(
        &mut world,
        &run,
        NESTED_GOAL_CALL,
        engine::Tool::Goal { words: WORDS.into(), advisor: false, agree: true },
        b"a goal starts an advisor alone: set advisor, or make the change with change",
    );
}

#[test]
fn an_agreed_goals_brief_carries_its_persons_words_whole() {
    let (world, _, run) = running_goal_with_agreement(true);
    let row = live(&world, run.task);
    assert!(row.spec.asks);
    assert_eq!(row.spec.words.as_ref(), [WORDS, AGREED_SUFFIX, PERSON_HEADING, PERSON_REQUEST].concat());
}

#[test]
fn every_goals_brief_puts_the_roots_lines_after_the_callers_words() {
    for (agree, suffix) in [(false, GOAL_SUFFIX), (true, AGREED_SUFFIX)] {
        let (world, _, run) = running_goal_with_agreement(agree);
        assert!(live(&world, run.task).spec.words.starts_with(&[WORDS, suffix].concat()));
    }
}

fn agreed_at_spec_edge(extra: usize, person_words: &[u8]) -> (World, u64, Box<[u8]>) {
    let world = World::configured(true, true);
    let bound = skein_lib::usize_of(world.env.limits.tasks.spec_bytes);
    let fixed = AGREED_SUFFIX.len() + PERSON_HEADING.len() + PERSON_BYTES;
    let words = vec![b's'; bound - fixed + extra].into_boxed_slice();
    let (mut world, chat) = start_chat(world, 400, 11, 3, Some(person_words), None);
    let task = delegated(call(
        &mut world,
        &chat,
        GOAL_CALL,
        engine::Tool::Goal { words: words.clone(), advisor: false, agree: true },
    ));
    (world, task, words)
}

#[test]
fn an_agreed_brief_fits_its_persons_words_at_the_spec_bound() {
    let person = [b'x'; PERSON_BYTES];
    let (world, task, words) = agreed_at_spec_edge(AT_BOUND, &person);
    assert_eq!(live(&world, task).spec.words.as_ref(), [&words, AGREED_SUFFIX, PERSON_HEADING, &person].concat());
    assert_eq!(live(&world, task).spec.words.len(), skein_lib::usize_of(world.env.limits.tasks.spec_bytes));
}

#[test]
fn an_agreed_brief_cuts_its_persons_words_at_the_spec_bound() {
    // One excess byte and the 16-byte root cut marker omit 17 original bytes.
    const OMITTED: usize = 17;
    let (world, task, words) = agreed_at_spec_edge(ONE_PAST_BOUND, &[b'x'; PERSON_BYTES]);
    let kept = [b'x'; PERSON_BYTES - OMITTED];
    assert_eq!(
        live(&world, task).spec.words.as_ref(),
        [&words, AGREED_SUFFIX, PERSON_HEADING, &kept, b"\n[17 bytes cut]\n"].concat()
    );
    assert_eq!(live(&world, task).spec.words.len(), skein_lib::usize_of(world.env.limits.tasks.spec_bytes));
}

#[test]
fn an_agreed_brief_cuts_at_a_whole_character() {
    // Cutting the 17 bytes above would split é: the root omits 18 instead.
    const OMITTED: usize = 18;
    let person = "é".repeat(PERSON_BYTES / "é".len());
    let (world, task, words) = agreed_at_spec_edge(ONE_PAST_BOUND, person.as_bytes());
    let kept = "é".repeat((PERSON_BYTES - OMITTED) / "é".len());
    assert_eq!(
        live(&world, task).spec.words.as_ref(),
        [&words, AGREED_SUFFIX, PERSON_HEADING, kept.as_bytes(), b"\n[18 bytes cut]\n"].concat()
    );
    assert!(core::str::from_utf8(&live(&world, task).spec.words).is_ok());
}

#[test]
fn an_agreed_goals_change_before_the_answer_is_refused_by_name() {
    let (mut world, chat, run) = running_goal_with_agreement(true);
    let engine::CallAnswer::Sent { message } = call(
        &mut world,
        &run,
        NESTED_GOAL_CALL,
        engine::Tool::Message {
            target: chat.task,
            form: engine::MessageForm::Question,
            words: b"May I go ahead?".as_slice().into(),
        },
    ) else {
        panic!("question sent")
    };
    let expected = format!("wait for the answer to question {message}");
    refused_tool(
        &mut world,
        &run,
        CONTINUED_CALL,
        engine::Tool::Change { changes: Box::new([engine::ChangeCall::words(b"Small fix".as_slice().into(), None)]) },
        expected.as_bytes(),
    );
}

#[test]
fn an_agreed_goals_change_before_asking_is_refused_by_name() {
    let (mut world, _, run) = running_goal_with_agreement(true);
    refused_tool(
        &mut world,
        &run,
        NESTED_GOAL_CALL,
        engine::Tool::Change { changes: Box::new([engine::ChangeCall::words(b"Small fix".as_slice().into(), None)]) },
        ASK_PLAN,
    );
}

#[test]
fn an_agreed_brief_uses_the_persons_newest_consumed_chat_words() {
    const MESSAGE_KEY: u8 = 96; // Distinct from the chat creation and agent call keys.
    const STEPS: u32 = 200; // The root world's durable round-trip bound.
    const NEWEST: &[u8] = b"Newest request: read both files, then ask me before changing either.";
    let (mut world, chat) = start_chat(World::configured(true, true), 400, 11, 3, Some(PERSON_REQUEST), None);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(MESSAGE_KEY))),
        sign_in: world.signed_in.expect("owner session"),
        key: [MESSAGE_KEY; 16],
        ask: people::Ask::Say { project: 1, task: chat.task, words: NEWEST.into() },
    }));
    for _ in 0..STEPS {
        world.tick();
    }
    let read = Some(live(&world, chat.task).last_message);
    world.send(protocol::turn(Token::new(7), chat.task, chat.attempt, 1, 0, read, b"Read the latest request"));
    world.until(Until::Ready);
    let task = delegated(call(
        &mut world,
        &chat,
        GOAL_CALL,
        engine::Tool::Goal { words: WORDS.into(), advisor: false, agree: true },
    ));
    assert_eq!(live(&world, task).spec.words.as_ref(), [WORDS, AGREED_SUFFIX, PERSON_HEADING, NEWEST].concat());
}

#[test]
fn a_held_goal_under_a_chat_waits_in_its_persons_inbox_at_once() {
    use jig_domain_core::{documents, reads};
    let mut world = World::configured(true, true);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { panic!("create") };
    batch[0].flags.ended_by = people::EndedBy::Requester;
    batch[0].budget = people::Budget::AsAllowed { cap: Some(400) };
    batch[0].authority.tools = 1 | 2 | 4 | 512;
    batch[0].authority.delegation.tasks = 11;
    batch[0].authority.delegation.depth = 3;
    let chat = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    let task = delegated(call(
        &mut world,
        &chat,
        GOAL_CALL,
        engine::Tool::Goal { words: WORDS.into(), advisor: false, agree: true },
    ));
    world.until(Until::SecondAssignment);
    let run = world.assigned.iter().find(|run| run.task == task).expect("goal assigned").clone();
    let event = world.protocol.answer(
        Token::new(7),
        task,
        run.attempt,
        0,
        tasks::End::Exceeded(tasks::Exceeded { bound: tasks::Bound::Reasoning, value: 2048 }),
        engine::Parts::none(),
    );
    world.send(event);
    world.until(Until::Ready);
    assert!(matches!(live(&world, task).phase, tasks::Phase::Held { .. }), "{:?}", live(&world, task).phase);
    let answer = world.party_read(reads::Read::Inbox {
        group: reads::Group::Escalations,
        filter: reads::Filter { project: Some(project), goal: None },
        from: None,
    });
    let reads::Answer::Page(page) = answer else { panic!("person's escalation inbox: {answer:?}") };
    assert!(
        page.rows.iter().any(|row| matches!(row, reads::PageRow::Inbox(entry)
        if matches!(&entry.task, documents::TaskLink::Live(chip) if chip.number == task)
        && entry.object == documents::InboxObject::Escalation { revision: 1 })),
        "inbox: {page:?}; escalation: {:?}",
        live(&world, task).escalation
    );
    assert!(matches!(
        live(&world, task).escalation,
        tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Person(_), .. }
    ));
    world.close();
}
