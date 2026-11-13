//! A goal's changes ask, settle their questions and land into its held branch
//! (temper/docs/design/domain/forge/change/questions.md, 1, and temper/docs/design/domain/forge/branches.md, 1;
//! temper/docs/design/draft/reference-code-production.md, 8).
use super::goal_branch::{branch_tip, forge_hold, goal_request};
use super::questions::{replies, row};
use super::*;
use temper_engine_domain_forge_change::{Route, Stands};

const CHANNEL: Token = Token::new(7);
const WORKER_SLOTS: u32 = 3; // The goal and its producers may run together.
const TWO_CHANGES: usize = 2;
const DEPENDENCIES: u32 = 2; // B keeps A, whose work it started from, while adding C.
const GOAL_AND_A: usize = 2;
const STEPS: u32 = 1000; // The goal-change world's bound for landing and hold release.
const QUESTION: &[u8] = b"Which behavior?\nKeep the design or widen it? I recommend the design.";
const SECOND: &[u8] = b"May I widen the scope?\nKeep it small or add a file? I recommend keeping it small.";
const ANSWER: &[u8] = b"Keep the design.";
const DEPENDENCY_ANSWER: &[u8] = b"Wait for C, then use its design.";
const A_REPORT: &[u8] = b"Done: added A.\n## Left\n- none";
const B_REPORT: &[u8] = b"Done: added B.\n## Left\n- none";
const C_REPORT: &[u8] = b"Done: clarified the design.\n## Left\n- none";
// A goal plus three changes, each with its six delegates.
const STORY_TASKS: u32 = 1 + 3 * (1 + 6);
// Each result retains the producer's report before the check evidence is cut.
const RESULT_BYTES: u32 = 256;
// Three change results, two topic notices, core news, a question and a hold escalation.
const INBOX_BYTES: u32 = 8 * RESULT_BYTES;

struct Story {
    world: World,
    goal: engine::Assignment,
    a: u64,
    b: u64,
    branch: Box<[u8]>,
    main: u64,
    producer: engine::Assignment,
}

fn start(bound: Option<u32>) -> Story {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits {
            slots: Some(WORKER_SLOTS),
            result_bytes: Some(RESULT_BYTES),
            questions: bound,
            ..StoryLimits::default()
        },
    );
    world.env.limits.tasks.dependencies = DEPENDENCIES;
    let mut world = world.with_goal_question_capacity(STORY_TASKS, INBOX_BYTES);
    let segments = [b"forge".as_slice(), b"forge.example", b"org", b"repo", b"branch"].map(Box::from);
    world = world.with_landing_deployment(Box::new([
        engine::LandingRule {
            sets: Box::new([]),
            connector: 0,
            kind: 4,
            enforce: true,
            pattern: authority::Pattern {
                segments: segments.clone().into(),
                last: authority::Last::Exact(b"main".as_slice().into()),
            },
            ci: true,
            up_to_date: true,
            gates: Box::new([]),
            approvals: Box::new([]),
        },
        engine::LandingRule {
            sets: Box::new([]),
            connector: 0,
            kind: 4,
            enforce: true,
            pattern: authority::Pattern {
                segments: segments.into(),
                last: authority::Last::Open(b"temper".as_slice().into()),
            },
            ci: false,
            up_to_date: false,
            gates: Box::new([]),
            approvals: Box::new([]),
        },
    ]));
    adopt_fresh(&mut world);
    world.send(protocol::hello_slots(CHANNEL, WORKER_SLOTS, Box::new([])));
    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal template") };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    let goal = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    world.until(Until::Ready);
    // Keep the goal's inbox credit distinct from its producer's question credit.
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(Token::new(96)),
        sign_in: world.signed_in.expect("owner"),
        key: [96; 16],
        ask: people::Ask::Say { project: 1, task: goal.task, words: b"Follow the plan.".as_slice().into() },
    }));
    world.until(Until::Ready);
    world.scripted_read_inbox(&goal);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    assert_eq!(forge_hold(&world, &branch).expect("goal holds g").task, goal.task);
    let main = branch_tip(&mut world, b"main");
    assert_eq!(branch_tip(&mut world, &branch), main);
    let input = smith_domain_run::HostInput::attested(
        br#"{"changes":[{"name":"a","words":"Add A","landing":"goal","questions":"up"},{"name":"b","words":"Add B","landing":"goal","after":["a"],"questions":"up"}]}"#.as_slice().into(),
    ).expect("bounded change input");
    let temper_engine_smith::Decoded::Call(call) = temper_engine_smith::call(
        smith_domain_run::CallName { activation: goal.attempt, completion: 1, position: 1 },
        b"change",
        &input,
    )
    .expect("change decodes") else {
        panic!("change call")
    };
    let engine::CallAnswer::Delegated(made) = world.scripted_call(&goal, 93, call.tool) else {
        panic!("both changes admitted")
    };
    assert_eq!(made.len(), TWO_CHANGES);
    let (a, b) = (made[0].task, made[1].task);
    assert_eq!(live_task(&world, b).dependencies.as_ref(), &[a]);
    let producer = next_producer(&mut world, a);
    world.until(Until::Ready);
    assert_eq!(world.assigned.len(), GOAL_AND_A, "B waits although a worker slot is free");
    assert_eq!(row(&world, a).base, branch);
    assert_eq!(row(&world, a).questions.route, Route::Up);
    assert!(live_task(&world, b).spec.parameters.contains(&tasks::Parameter::Number { name: 14, value: 2 }));
    Story { world, goal, a, b, branch, main, producer }
}

fn next_producer(world: &mut World, change: u64) -> engine::Assignment {
    for _ in 0..STEPS {
        world.tick();
        if let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(change)))
            && let forge_top::Stored::Change(row) = row.as_ref()
            && let Some((task, temper_engine_domain_forge_change::Delegate::Produce)) = row.delegate
            && let Some(run) = world.assigned.iter().rev().find(|run| run.task == task)
        {
            return run.clone();
        }
    }
    panic!("change {change} has an assigned producer");
}

fn park(world: &mut World, producer: &engine::Assignment) {
    let turns = live_task(world, producer.task).turn;
    park_after_turn(world, producer, None, turns);
}

fn ask_up(world: &mut World, producer: &engine::Assignment, change: u64) -> (u64, u64) {
    let question = world.producer_question(producer, 100 + producer.attempt, QUESTION);
    assert!(live_task(world, producer.task).questions.iter().any(|q| q.number == question));
    park(world, producer);
    let tasks::Party::Task(requester) = live_task(world, change).requester else { panic!("change's requester") };
    let up = passed_up(world, requester, change, QUESTION);
    assert_panel(world, change, producer.task, question, QUESTION, forge_top::PanelStands::Asked);
    (question, up)
}

fn assert_panel(world: &mut World, change: u64, asker: u64, number: u64, words: &[u8], stands: forge_top::PanelStands) {
    let watcher = Token::new(500 + skein_lib::len_u64(world.panels.len()));
    world.send(engine::Event::Party(engine::PartyInput::Watch {
        watcher,
        sign_in: world.signed_in.expect("goal's owner session"),
        subject: jig_domain_core::documents::Watched::Application {
            scope: people::ApplicationScope::Project(1),
            connector: 0,
            subject: Token::new(change),
        },
        window: jig_domain_core::documents::Window { rows: world.env.limits.documents.rows, after: None },
    }));
    let wanted_watcher = watcher;
    for _ in 0..STEPS {
        world.tick();
        if world.panels.iter().any(|(watcher, _)| *watcher == wanted_watcher) {
            break;
        }
    }
    let panel =
        &world.panels.iter().rev().find(|(watcher, _)| *watcher == wanted_watcher).expect("forge change panel").1;
    assert_eq!(panel.task, change);
    assert!(panel.within());
    assert_eq!(panel.questions.route, Route::Up);
    assert_eq!(panel.questions.taken.limit, world.env.limits.forge.change_policy.questions);
    assert_eq!(panel.questions.taken.used, row(world, change).questions.taken);
    let line = words.split(|byte| *byte == b'\n').next().expect("question first line");
    assert_eq!(panel.questions.open.as_ref(), &[forge_top::PanelQuestion { asker, number, line: line.into(), stands }]);
    world.send(engine::Event::Party(engine::PartyInput::Unwatch { watcher }));
    world.until(Until::Ready);
}

fn resumed(
    world: &mut World,
    producer: &engine::Assignment,
    change: u64,
    question: u64,
    answer: &[u8],
) -> engine::Assignment {
    for _ in 0..STEPS {
        world.tick();
        if let Some(run) =
            world.assigned.iter().rev().find(|run| run.task == producer.task && run.attempt > producer.attempt)
            && world.store.pending.is_empty()
        {
            assert!(run.messages.iter().any(|word| word.from == tasks::Party::Task(change)
                && word.kind == tasks::MessageKind::Answer { question }
                && word.words.as_ref() == answer));
            assert!(live_task(world, producer.task).questions.is_empty());
            assert!(row(world, change).questions.open.is_empty());
            return run.clone();
        }
    }
    panic!("producer resumes with its answer: {:?}", live_task(world, producer.task));
}

fn ended(world: &World, task: u64) -> Option<&tasks::TaskRecord> {
    match world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)))) {
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) => Some(row),
        _ => None,
    }
}

fn land(
    world: &mut World,
    goal: u64,
    change: u64,
    producer: &engine::Assignment,
    base: u64,
    file: &[u8],
    report: &[u8],
) -> u64 {
    let branch = producer.workspace.repositories[0].push.clone().expect("producer's push branch");
    assert_eq!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: base })),
        raw::Answer::Branch(raw::Created::Created)
    );
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, file, b"added", 2)
        .expect("producer pushes");
    assert_eq!(
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: head,
                context: b"build".as_slice().into(),
                state: raw::Check::Pending,
                description: Box::new([]),
                link: Box::new([]),
            })
        ),
        raw::Answer::Done
    );
    let event = world.protocol.answer(
        CHANNEL,
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(file.into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: report.into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..STEPS {
        world.tick();
        if ended(world, change).is_some() {
            break;
        }
    }
    let change_row = row(world, change);
    let temper_engine_domain_forge_change::State::Landed { merge, .. } = change_row.change.state else {
        panic!("change {change} lands: {change_row:?}; task {:?}", live_task(world, change))
    };
    assert_eq!(change_row.report.as_ref(), report);
    assert_eq!((change_row.change.repairs, change_row.change.updates, change_row.change.resolutions), (0, 0, 0));
    let landing = change_row.base.clone();
    let number = change_row.pull.expect("landed pull");
    let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { words, .. })) =
        &ended(world, change).expect("change ended").phase
    else {
        panic!("change ended done")
    };
    let expected = format!(
        "Landed: pull request #{number} merged into {} as ",
        core::str::from_utf8(&landing).expect("goal branch")
    );
    assert!(words.windows(expected.len()).any(|part| part == expected.as_bytes()));
    let evidence = [b"Report:\n".as_slice(), report, b"\n"].concat();
    assert!(
        words.windows(evidence.len()).any(|part| part == evidence),
        "result carries producer's report: {}",
        String::from_utf8_lossy(words)
    );
    assert!(
        replies(world, goal, change)
            .iter()
            .any(|word| matches!(word.kind, tasks::MessageKind::Result(tasks::ResultKind::Change { .. }))
                && word.words == *words)
    );
    assert!(world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Merge { number: pull, head: merged, .. } if *pull == number && *merged == translate::commit(head))));
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else { panic!("pull") };
    assert_eq!(pull.base, landing);
    assert_eq!(pull.commit, head);
    assert!(
        pull.statuses.iter().any(|status| status.context.as_ref() == b"build" && status.state == raw::Check::Pending)
    );
    let tip = branch_tip(world, &landing);
    assert_eq!(translate::commit(tip), merge);
    assert_eq!(forge_hold(world, &landing).expect("goal retains g").task, goal);
    tip
}

fn settle_a(story: &mut Story) -> u64 {
    let (question, up) = ask_up(&mut story.world, &story.producer, story.a);
    story.world.goal_answer(&story.goal, 105, story.a, up, ANSWER);
    let producer = resumed(&mut story.world, &story.producer, story.a, question, ANSWER);
    land(&mut story.world, story.goal.task, story.a, &producer, story.main, b"A", A_REPORT)
}

fn escalate_b(story: &mut Story) -> (engine::Assignment, u64, u64) {
    story.world.until(Until::Ready);
    story.world.scripted_read_inbox(&story.goal);
    let producer = next_producer(&mut story.world, story.b);
    assert!(ended(&story.world, story.a).is_some(), "B starts after A lands");
    assert_eq!(row(&story.world, story.b).base, story.branch);
    let (question, up) = ask_up(&mut story.world, &producer, story.b);
    assert_panel(&mut story.world, story.b, producer.task, question, QUESTION, forge_top::PanelStands::Asked);
    assert_eq!(replies(&story.world, producer.task, story.b).len(), 0);
    (producer, question, up)
}

fn passed_up(world: &mut World, goal: u64, change: u64, words: &[u8]) -> u64 {
    for _ in 0..STEPS {
        world.tick();
        if row(world, change).questions.open.first().is_some_and(|open| open.stands == Stands::Asked)
            && world.store.pending.is_empty()
        {
            let message = replies(world, goal, change)
                .into_iter()
                .find(|word| word.kind == tasks::MessageKind::Question && word.words.as_ref() == words)
                .expect("goal receives question and reasons whole");
            return message.number;
        }
    }
    panic!(
        "question reaches goal: relay {:?}; phase {:?}; goal inbox {:?}",
        row(world, change).questions,
        live_task(world, change).phase,
        live_task(world, goal).inbox
    );
}

fn finish_story(story: &mut Story) {
    if ended(&story.world, story.b).is_none() {
        let producer = next_producer(&mut story.world, story.b);
        let base = branch_tip(&mut story.world, &story.branch);
        land(&mut story.world, story.goal.task, story.b, &producer, base, b"B", B_REPORT);
    }
    let turns = live_task(&story.world, story.goal.task).turn;
    let mut event = story.world.protocol.answer(
        CHANNEL,
        story.goal.task,
        story.goal.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"Done: the changes landed.".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    let engine::Event::Worker(engine::WorkerInput::Link {
        link: engine::Link::Up { up: host::LinkUp::Answer { answer, .. }, .. },
        ..
    }) = &mut event
    else {
        panic!("worker result")
    };
    answer.turns = turns;
    story.world.send(event);
    for _ in 0..STEPS {
        story.world.tick();
        if ended(&story.world, story.goal.task).is_some() && story.world.store.pending.is_empty() {
            break;
        }
    }
    assert!(matches!(
        ended(&story.world, story.goal.task).expect("goal ended").phase,
        tasks::Phase::Ended(tasks::Ending::Done(_))
    ));
    story.world.until(Until::Ready);
    assert!(forge_hold(&story.world, &story.branch).is_none(), "the goal releases g when done");
    story.world.close();
}

fn amend_spec(story: &mut Story, target: u64, spec: &str, completion: u32) {
    let revision = live_task(&story.world, target).revisions.spec;
    let input = format!(
        "{{\"target\":{target},\"revision\":{revision},\"amendment\":{{\"spec\":{spec},\"reason\":\"Clarify scope.\"}}}}"
    );
    let input = smith_domain_run::HostInput::attested(input.into_bytes().into_boxed_slice()).expect("bounded amend");
    let temper_engine_smith::Decoded::Call(call) = temper_engine_smith::call(
        smith_domain_run::CallName { activation: story.goal.attempt, completion, position: 1 },
        b"amend",
        &input,
    )
    .expect("amend decodes") else {
        panic!("amend call")
    };
    let answer = story.world.scripted_call(&story.goal, u64::from(completion), call.tool);
    assert!(matches!(answer, engine::CallAnswer::Amended { .. }), "words amended: {answer:?}");
}

#[test]
fn amending_with_explicit_parameters_replaces_them() {
    let cases: [(u32, &str, Box<[tasks::Parameter]>); 2] = [
        (94, "[]", Box::new([])),
        (95, r#"[{"kind":"number","name":14,"value":2}]"#, Box::new([tasks::Parameter::Number { name: 14, value: 2 }])),
    ];
    let mut story = start(None);
    let change = story.producer.task;
    for (completion, parameters, expected) in cases {
        let spec = format!(r#"{{"words":"Widen the close-out scope.","parameters":{parameters}}}"#);
        amend_spec(&mut story, change, &spec, completion);
        assert_eq!(live_task(&story.world, change).spec.parameters, expected, "explicit replacement: {parameters}");
    }
    settle_a(&mut story);
    finish_story(&mut story);
}

#[test]
fn amending_only_words_preserves_change_parameters() {
    let mut story = start(None);
    let before = live_task(&story.world, story.a);
    let parameters = before.spec.parameters.clone();
    let revision = before.revisions.spec;
    assert!(!parameters.is_empty(), "the live change has its procedure parameters");
    let change = story.a;
    amend_spec(&mut story, change, r#"{"words":"Widen the close-out scope."}"#, 94);
    let after = live_task(&story.world, story.a);
    assert_eq!(after.spec.words.as_ref(), b"Widen the close-out scope.");
    assert_eq!(after.spec.parameters, parameters, "a words amendment keeps every procedure parameter");
    assert_eq!(after.revisions.spec, revision + 1);
    let tip = settle_a(&mut story);
    assert_ne!(tip, story.main, "the amended change proceeds through questions and landing");
    finish_story(&mut story);
}

#[test]
fn a_goal_answers_a_question_and_the_change_lands() {
    let mut story = start(None);
    let tip = settle_a(&mut story);
    assert_ne!(tip, story.main);
    assert_eq!(branch_tip(&mut story.world, b"main"), story.main);
    assert_eq!(story.world.merges, 1);
    finish_story(&mut story);
}

#[test]
fn an_escalated_question_reaches_the_goal_and_its_answer_the_producer() {
    let mut story = start(None);
    let base = settle_a(&mut story);
    let (producer, question, up) = escalate_b(&mut story);
    story.world.goal_answer(&story.goal, 110, story.b, up, ANSWER);
    let producer = resumed(&mut story.world, &producer, story.b, question, ANSWER);
    land(&mut story.world, story.goal.task, story.b, &producer, base, b"B", B_REPORT);
    assert_eq!(story.world.merges, 2);
    assert_eq!(branch_tip(&mut story.world, b"main"), story.main);
    finish_story(&mut story);
}

#[test]
fn an_answer_that_adds_a_dependency_waits_for_it() {
    let mut story = start(None);
    let base = settle_a(&mut story);
    let (producer, question, up) = escalate_b(&mut story);
    let mut change = engine::ChangeCall::words(b"Clarify the design".as_slice().into(), None);
    change.landing = Some(engine::ChangeLanding::Goal);
    change.questions = Some(engine::ChangeQuestions::Up);
    let engine::CallAnswer::Delegated(made) =
        story.world.scripted_call(&story.goal, 110, engine::Tool::Change { changes: Box::new([change]) })
    else {
        panic!("C admitted")
    };
    let c = made[0].task;
    let upstream = next_producer(&mut story.world, c);
    assert_eq!(row(&story.world, c).base, story.branch);
    let mut amendment = temper_engine_domain_world::fixtures::task_amendment(None, None, b"Wait for C's design");
    amendment.dependencies = Some(Box::new([story.a, c]));
    let revision = live_task(&story.world, story.b).revisions.spec;
    let answer = story.world.scripted_call(
        &story.goal,
        111,
        engine::Tool::Amend { keep_parameters: false, target: story.b, revision, amendment },
    );
    assert!(matches!(answer, engine::CallAnswer::Amended { .. }), "dependency added: {answer:?}");
    story.world.goal_answer(&story.goal, 112, story.b, up, DEPENDENCY_ANSWER);
    for _ in 0..STEPS {
        story.world.tick();
        if row(&story.world, story.b)
            .questions
            .open
            .first()
            .is_some_and(|open| matches!(open.stands, Stands::Held { .. }))
            && story.world.store.pending.is_empty()
        {
            break;
        }
    }
    assert_panel(&mut story.world, story.b, producer.task, question, QUESTION, forge_top::PanelStands::Held);
    assert_eq!(replies(&story.world, producer.task, story.b).len(), 0);
    assert!(!story.world.assigned.iter().any(|run| run.task == producer.task && run.attempt > producer.attempt));
    assert_eq!(branch_tip(&mut story.world, &story.branch), base);
    let c_merge = land(&mut story.world, story.goal.task, c, &upstream, base, b"C", C_REPORT);
    let producer = resumed(&mut story.world, &producer, story.b, question, DEPENDENCY_ANSWER);
    assert!(ended(&story.world, c).is_some(), "the answer waits for C's landing");
    land(&mut story.world, story.goal.task, story.b, &producer, c_merge, b"B", B_REPORT);
    assert_eq!(story.world.merges, 3);
    assert_eq!(branch_tip(&mut story.world, b"main"), story.main);
    finish_story(&mut story);
}

#[test]
fn a_change_past_its_questions_holds_and_the_goal_releases_it() {
    let mut story = start(Some(1));
    let (first, up) = ask_up(&mut story.world, &story.producer, story.a);
    story.world.goal_answer(&story.goal, 105, story.a, up, ANSWER);
    story.world.scripted_read_inbox(&story.goal);
    let producer = resumed(&mut story.world, &story.producer, story.a, first, ANSWER);
    let question = story.world.producer_question(&producer, 110, SECOND);
    park(&mut story.world, &producer);
    for _ in 0..STEPS {
        story.world.tick();
        if matches!(live_task(&story.world, story.a).phase, tasks::Phase::Held { why: tasks::Hold::Procedure, .. })
            && story.world.store.pending.is_empty()
        {
            break;
        }
    }
    let expected = format!(
        "its questions are spent: 1 of 1 taken; question {question} of task {} waits; release it to pass the question up",
        producer.task
    );
    let tasks::Phase::Held { why: tasks::Hold::Procedure, source: Some(source), .. } =
        &live_task(&story.world, story.a).phase
    else {
        panic!("held past question bound")
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("hold's words") };
    assert_eq!(words.as_ref(), expected.as_bytes());
    assert!(
        replies(&story.world, story.goal.task, story.a).iter().any(|word| word.words.as_ref() == expected.as_bytes())
    );
    assert!(
        !replies(&story.world, story.goal.task, story.a).iter().any(|word| word.kind == tasks::MessageKind::Question)
    );
    let revision = live_task(&story.world, story.a).revisions.hold;
    assert!(matches!(
        story.world.scripted_call(&story.goal, 111, engine::Tool::Release { target: story.a, revision, again: None }),
        engine::CallAnswer::Revised { .. }
    ));
    let up = passed_up(&mut story.world, story.goal.task, story.a, SECOND);
    assert_panel(&mut story.world, story.a, producer.task, question, SECOND, forge_top::PanelStands::Asked);
    story.world.goal_answer(&story.goal, 112, story.a, up, ANSWER);
    let producer = resumed(&mut story.world, &producer, story.a, question, ANSWER);
    land(&mut story.world, story.goal.task, story.a, &producer, story.main, b"A", A_REPORT);
    finish_story(&mut story);
}

fn fresh_goal(story: &mut Story) {
    story.world.until(Until::Ready);
    story.world.scripted_read_inbox(&story.goal);
    park(&mut story.world, &story.goal);
    story.world.until(Until::Ready);
    story.world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(96)),
        sign_in: story.world.signed_in.expect("owner session"),
        key: [96; 16],
        ask: people::Ask::Say { project: 1, task: story.goal.task, words: ANSWER.into() },
    }));
    for _ in 0..STEPS {
        story.world.tick();
        if let Some(run) = story
            .world
            .assigned
            .iter()
            .rev()
            .find(|run| run.task == story.goal.task && run.attempt > story.goal.attempt)
        {
            assert!(run.transcript.is_empty(), "the goal charter starts fresh");
            assert!(
                run.messages.iter().all(|word| word.kind != tasks::MessageKind::Question),
                "the original question delivery was consumed"
            );
            story.goal = run.clone();
            return;
        }
    }
    panic!("person's choice wakes a fresh goal");
}

fn recovered_question(story: &mut Story) -> u64 {
    let answer = story.world.scripted_call(
        &story.goal,
        120,
        engine::Tool::ReadTask { task: story.a, field: jig_domain_core::reads::TaskTextField::Specification, from: 0 },
    );
    let rendered = temper_engine_smith::answer(&answer);
    let words = core::str::from_utf8(rendered.text()).expect("host read is UTF-8");
    let prefix = format!("Answer target {}, question ", story.a);
    let number = words
        .split(&prefix)
        .nth(1)
        .expect("durable answer target and question in read_task")
        .split_whitespace()
        .next()
        .expect("question number")
        .parse()
        .expect("numeric question");
    assert!(words.contains(&format!("answerer task {}", story.goal.task)));
    number
}

fn waiting_text(run: &engine::Assignment) -> Option<&[u8]> {
    run.sections.iter().find_map(|section| match &section.body {
        engine::BriefBody::Text(text) if section.kind == engine::BriefKind::Core(jig_domain_brief::Core::Waiting) => {
            Some(text.as_ref())
        }
        _ => None,
    })
}

fn assert_waiting_question(run: &engine::Assignment, asker: u64, number: u64, words: &[u8]) {
    let heading = format!(
        "Questions asked of you, still open, oldest first:\nTask {asker} asks (question {number}); answer with message target={asker}, form=answer, question={number}:\n"
    );
    assert_eq!(waiting_text(run), Some([heading.as_bytes(), words, b"\n"].concat().as_slice()));
}

#[test]
fn a_stock_goal_keeps_a_passed_up_question_in_front_of_each_fresh_run() {
    let (mut world, goal, _) = super::goal_branch::running_stock_goal();
    let change = super::questions::make_change(&mut world, &goal, 93, engine::ChangeQuestions::Up);
    let producer = super::questions::next_producer(&mut world, change);
    park(&mut world, &goal);
    world.until(Until::Ready);
    assert_eq!(live_task(&world, goal.task).phase, tasks::Phase::Active(tasks::Active::Idle));
    let question = world.producer_question(&producer, 100, QUESTION);
    park(&mut world, &producer);
    let first = next_goal_run(&mut world, &goal);
    let up = first
        .messages
        .iter()
        .find(|word| word.from == tasks::Party::Task(change) && word.kind == tasks::MessageKind::Question)
        .expect("start carries the change's question")
        .number;
    assert_eq!(waiting_text(&first), None, "start question is not repeated");
    let defaults = super::goal_branch::stock_defaults();
    let charter = defaults.charters.iter().find(|charter| charter.number == 5).expect("stock coordinator");
    let part = charter.parts.iter().find(|part| part.on.contains(tasks::Reasons::QUESTION)).expect("questions part");
    assert!(
        first.run.policy.instructions.windows(part.instructions.len()).any(|words| words == part.instructions.as_ref())
    );
    world.scripted_read_inbox(&first);
    park(&mut world, &first);
    for _ in 0..STEPS {
        world.tick();
        if live_task(&world, goal.task).last_answer == Some(first.attempt) {
            break;
        }
    }
    assert!(
        matches!(
            live_task(&world, goal.task).phase,
            tasks::Phase::Active(tasks::Active::Due | tasks::Active::Preparing)
        ),
        "question is due, or already preparing, never idle"
    );
    let next = next_goal_run(&mut world, &first);
    assert_waiting_question(&next, change, up, QUESTION);
    assert!(live_task(&world, goal.task).reasons.contains(tasks::Reasons::QUESTION));
    world.goal_answer(&next, 110, change, up, ANSWER);
    resumed(&mut world, &producer, change, question, ANSWER);
    world.close();
}

fn next_goal_run(world: &mut World, previous: &engine::Assignment) -> engine::Assignment {
    for _ in 0..STEPS {
        world.tick();
        if let Some(run) =
            world.assigned.iter().rev().find(|run| run.task == previous.task && run.attempt > previous.attempt)
        {
            return run.clone();
        }
    }
    panic!("goal is assigned again: {:?}", live_task(world, previous.task));
}

#[test]
fn a_fresh_goal_run_recovers_and_answers_an_escalated_question() {
    let mut story = start(None);
    let (question, _) = ask_up(&mut story.world, &story.producer, story.a);
    for _ in 0..STEPS {
        story.world.tick();
        if row(&story.world, story.a).questions.open.first().is_some_and(|open| open.stands == Stands::Asked)
            && story.world.store.pending.is_empty()
        {
            break;
        }
    }
    fresh_goal(&mut story);
    unsuccessful_answers(&mut story, question);
    let number = recovered_question(&mut story);
    assert_waiting_question(&story.goal, story.a, number, QUESTION);
    assert_ne!(number, question, "escalation and producer credits differ");
    story.world.goal_answer(&story.goal, 121, story.a, number, ANSWER);
    let producer = resumed(&mut story.world, &story.producer, story.a, question, ANSWER);
    land(&mut story.world, story.goal.task, story.a, &producer, story.main, b"A", A_REPORT);
    finish_story(&mut story);
}

fn adopt_fresh(world: &mut World) {
    use jig_domain_core::settings as s;
    world.adopt();
    let mut charter = walking::charter();
    charter.policy.resume = false;
    world.send(engine::Event::Party(engine::PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(300)),
        sign_in: world.signed_in.expect("deployment owner"),
        key: [30; 16],
        digest: [30; 32],
        kinds: Box::new([]),
        change: Box::new(s::Change {
            against: 1,
            items: Box::new([s::Item {
                setting: s::Setting::Charter(1),
                edit: s::Edit::Set(s::Value::Charter(Box::new(charter))),
            }]),
            running: s::Running::Stays,
        }),
    }));
    world.until(Until::Ready);
}

fn unsuccessful_answers(story: &mut Story, producer_question: u64) {
    for number in [3, producer_question] {
        let refusal = story.world.scripted_call(
            &story.goal,
            115,
            engine::Tool::Message {
                target: story.a,
                form: engine::MessageForm::Answer { question: number },
                words: ANSWER.into(),
            },
        );
        assert!(matches!(refusal, engine::CallAnswer::MessageRefused(_)));
        let rendered = temper_engine_smith::answer(&refusal);
        let text = core::str::from_utf8(rendered.text()).expect("host refusal");
        assert!(text.contains("Use read_task on the target task"));
        assert!(text.contains("form=answer"));
    }
    let sent = story.world.scripted_call(
        &story.goal,
        116,
        engine::Tool::Message { target: story.a, form: engine::MessageForm::Words, words: ANSWER.into() },
    );
    assert!(matches!(sent, engine::CallAnswer::Sent { .. }));
    story.world.until(Until::Ready);
    assert!(live_task(&story.world, story.goal.task).inbox.iter().any(|word| {
        word.from == tasks::Party::Task(story.a)
            && word.words.windows(b"Words do not answer".len()).any(|part| part == b"Words do not answer")
    }));
    let target = story.a;
    amend_spec(story, target, r#"{"words":"Keep the design."}"#, 117);
    assert_eq!(live_task(&story.world, target).questions.len(), 1, "words and amendment reasons leave the credit open");
    assert_eq!(row(&story.world, target).questions.open[0].stands, Stands::Asked);
}
