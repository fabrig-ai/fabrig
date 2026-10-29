use jig_client_world::harness::{ClientId, Until, World};
use skein_fake_person::{Find, Step};
use skein_world::domain::Verdict;
use temper_client_tools::jig::Printed;
use temper_client_world::{app::Temper, fixtures, front::Joint};

#[test]
fn a_chat_the_agent_starts_is_seen_by_the_person() {
    let (mut world, _agent, task, started) = chat(false);
    let person = world.client(fixtures::PARTY, fixtures::chat_page(task));
    world.person(person, vec![Step::See { find: Find::text(b"Hello"), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_chat(&started, task);
    assert_eq!(fixtures::chats(&world.front().engine), 1);
}

fn call(world: &mut World<Temper, Joint>, agent: ClientId, words: &[&[u8]]) -> Printed {
    let number = fixtures::give(world, agent, words);
    fixtures::answered(world, agent, number)
}

fn chat(park_runs: bool) -> (World<Temper, Joint>, ClientId, u64, Printed) {
    let mut world = fixtures::world(fixtures::SEED);
    if park_runs {
        world.front().engine.hello(skein_lib::Token::new(7));
        world.front().park_runs = true;
    }
    let agent = fixtures::agent(&mut world);
    let printed = call(&mut world, agent, &[b"chat", b"start", b"--project", b"1", b"--key", b"hello", b"Hello"]);
    let task = fixtures::done_task(&printed);
    (world, agent, task, printed)
}

fn assert_chat(printed: &Printed, task: u64) {
    assert_eq!((printed.status, printed.stdout.as_ref()), (0, format!("done: task {task}, chat\n").as_bytes()));
}

fn wait_for_reply(world: &mut World<Temper, Joint>, agent: ClientId, task: u64) -> Printed {
    let task = task.to_string();
    call(world, agent, &[b"task", b"show", task.as_bytes(), b"--until", b"replied"])
}

#[test]
fn the_agent_waits_for_its_chats_reply() {
    let (mut world, agent, task, started) = chat(true);
    let printed = wait_for_reply(&mut world, agent, task);
    assert_chat(&started, task);
    assert_eq!(printed.status, 0, "{printed:?}");
    assert!(printed.stdout.ends_with(b"met\n"), "{printed:?}");
    assert_eq!(
        world.front().engine.task(task).expect("parked chat").phase,
        jig_domain_tasks::Phase::Active(jig_domain_tasks::Active::Idle)
    );
}

#[test]
fn a_message_is_followed_by_its_reply_waited_for() {
    let (mut world, agent, task, started) = chat(true);
    wait_for_reply(&mut world, agent, task);
    let first_attempt = world.front().engine.task(task).expect("live chat").attempt;
    let printed =
        call(&mut world, agent, &[b"task", b"message", task.to_string().as_bytes(), b"More", b"--key", b"more"]);
    let reply = wait_for_reply(&mut world, agent, task);
    assert_chat(&started, task);
    assert_eq!((printed.status, printed.stdout.as_ref()), (0, format!("done: task {task}, message\n").as_bytes()));
    assert_eq!(reply.status, 0, "{reply:?}");
    assert!(reply.stdout.ends_with(b"met\n"), "{reply:?}");
    assert_eq!(world.front().engine.task(task).expect("live chat").attempt, first_attempt + 1);
    assert_eq!(
        world.front().engine.task(task).expect("chat parked after More").phase,
        jig_domain_tasks::Phase::Active(jig_domain_tasks::Active::Idle)
    );
}

#[test]
fn a_chat_the_agent_closes_ends() {
    let (mut world, agent, task, started) = chat(true);
    wait_for_reply(&mut world, agent, task);
    world.front().park_runs = false;
    let closed = call(&mut world, agent, &[b"chat", b"close", task.to_string().as_bytes(), b"--key", b"bye"]);
    let number =
        fixtures::give(&mut world, agent, &[b"task", b"show", task.to_string().as_bytes(), b"--until", b"ended"]);
    // The later point read crosses the same client's wire after its watch.
    // Its answer is an event barrier while the closing run is still held here.
    let current = call(&mut world, agent, &[b"task", b"show", task.to_string().as_bytes()]);
    assert_eq!(current.status, 0, "{current:?}");
    assert!(world.root(agent).agent().printed(number).is_none(), "watch waits before the run ends");
    let attempt = world.front().engine.task(task).expect("closing chat").attempt;
    world.front().engine.answer(
        skein_lib::Token::new(7),
        task,
        attempt,
        0,
        jig_domain_tasks::End::Finished {
            result: jig_domain_tasks::TaskResult::Report { words: b"Closed by the run".as_slice().into() },
            cancel_delegates: false,
        },
        temper_engine_domain::Parts::none(),
    );
    let printed = fixtures::answered(&mut world, agent, number);
    assert_chat(&started, task);
    assert_eq!((closed.status, closed.stdout.as_ref()), (0, format!("done: task {task}, close\n").as_bytes()));
    assert!(printed.stdout.starts_with(format!("task {task} ").as_bytes()), "{printed:?}");
    assert!(
        printed
            .stdout
            .windows(b"result: Completed \xc2\xb7 Closed by the run\n".len())
            .any(|w| w == b"result: Completed \xc2\xb7 Closed by the run\n"),
        "{printed:?}"
    );
    assert_eq!(printed.status, 0, "{printed:?}");
    assert!(printed.stdout.ends_with(b"met\n"), "{printed:?}");

    // The committed ended row must exist before opening another wait; this
    // exercises real watch refusal and the authorized historical read path.
    let key = temper_engine_domain::Key::Core(jig_domain_core::Key::Tasks(jig_domain_tasks::Key::Ended(task)));
    let Some(temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ended(row)))) =
        world.front().engine.store.rows.get(&key)
    else {
        panic!("task end was committed before the late wait");
    };
    let jig_domain_tasks::Phase::Ended(jig_domain_tasks::Ending::Done(jig_domain_tasks::TaskResult::Report { words })) =
        &row.phase
    else {
        panic!("the committed task has its completed report");
    };
    assert_eq!(words.as_ref(), b"Closed by the run");
    assert!(world.front().engine.task(task).is_none(), "live task name was retired");
    let late = call(&mut world, agent, &[b"task", b"show", task.to_string().as_bytes(), b"--until", b"ended"]);
    assert_eq!(late.status, 0, "{late:?}");
    assert_eq!(late.stdout, printed.stdout, "late wait prints the same real final report");
    assert!(late.stderr.is_empty(), "{late:?}");
}

// The fixture forge has repository acme/shop as 7 on forge 1.
const FORGE: u16 = 1;
const REPOSITORY: u32 = 7;
// A successful adoption is one admitted application, per protocol/forge.md.
const ADOPTIONS: usize = 1;
#[test]
fn a_repository_the_agent_adopts_is_held_by_the_engine() {
    let mut world = fixtures::world(fixtures::SEED);
    let agent = fixtures::agent(&mut world);
    let printed = call(
        &mut world,
        agent,
        &[
            b"repository",
            b"adopt",
            b"--project",
            b"1",
            b"--owner",
            b"acme",
            b"--name",
            b"shop",
            b"--landing",
            b"main",
            b"--key",
            b"shop",
        ],
    );
    assert_eq!(printed.status, 0, "{printed:?}");
    assert!(world.front().engine_adopted(FORGE, REPOSITORY));
    assert_eq!(world.front().applications_admitted(), ADOPTIONS);
}

#[test]
fn a_held_task_the_agent_releases_from_the_printed_act_runs_again() {
    let mut world = fixtures::world(fixtures::SEED);
    let (_goal, held) = fixtures::goal_with_held_delegate(world.front());
    let agent = fixtures::agent(&mut world);
    let printed = call(&mut world, agent, &[b"task", b"show", held.to_string().as_bytes()]);
    let mut words = fixtures::printed_act(&printed.stdout, b"task release");
    for word in &mut words {
        if word.as_ref() == b"KEY" {
            *word = Box::from(*b"release");
        }
    }
    let words: Vec<&[u8]> = words.iter().map(AsRef::as_ref).collect();
    let printed = call(&mut world, agent, &words);
    assert_eq!((printed.status, printed.stdout.as_ref()), (0, format!("done: task {held}, release\n").as_bytes()));
    assert_eq!(world.front().engine.task(held).expect("released delegate").tries, jig_domain_tasks::Tries::NONE);
}

#[test]
fn a_cut_question_is_answered_through_the_tools_and_reaches_the_chat() {
    use jig_domain_tasks::{MessageKind, Party};
    const TEXT_BOUND: u32 = 60;
    const QUESTION_BYTES: usize = 61;
    let mut world = fixtures::world(fixtures::SEED);
    let words = vec![b'q'; QUESTION_BYTES];
    let chat = fixtures::question_with_text_bound(world.front(), &words, Some(TEXT_BOUND));
    let (question, asking) = {
        let task = world.front().engine.task(chat).expect("asked chat");
        let question = task.inbox.iter().find(|word| word.kind == MessageKind::Question).expect("question");
        let Party::Task(asking) = question.from else { panic!("asking delegate") };
        (question.number, asking)
    };
    let agent = fixtures::agent(&mut world);
    answer_question_from_plain_inbox(&mut world, agent, chat, question, asking, &words, Some(TEXT_BOUND));
}

fn answer_question_from_plain_inbox(
    world: &mut World<Temper, Joint>,
    agent: ClientId,
    chat: u64,
    question: u64,
    asking: u64,
    words: &[u8],
    cut_at: Option<u32>,
) {
    use jig_domain_tasks::{MessageKind, Party};
    let inbox = call(world, agent, &[b"inbox"]);
    assert_eq!(inbox.status, 0, "{inbox:?}");
    let shown = std::str::from_utf8(&inbox.stdout).expect("UTF-8");
    if let Some(bound) = cut_at {
        let words = std::str::from_utf8(&words[..usize::try_from(bound).expect("bound")]).expect("question words");
        assert!(shown.contains(&format!("{words} (cut)\n")), "{shown}");
        assert!(shown.contains(&format!("read whole: temper ctl DIR task activity {asking}\n")), "{shown}");
    } else {
        let words = std::str::from_utf8(words).expect("question words");
        assert!(shown.contains(&format!("{words}\n")), "{shown}");
        assert!(!shown.contains(" (cut)"), "{shown}");
        assert!(!shown.contains("read whole:"), "{shown}");
    }
    assert_ne!(chat, asking, "answer and whole-read calls name different tasks");
    assert!(shown.contains(&format!("answer: temper ctl DIR task answer {chat} {question} TEXT\n")), "{shown}");
    let line = shown.lines().find_map(|line| line.strip_prefix("answer: temper ctl DIR ")).expect("answer call");
    let mut words: Vec<&[u8]> = Vec::new();
    for word in line.split_ascii_whitespace() {
        words.push(if word == "TEXT" { b"Go ahead." } else { word.as_bytes() });
    }
    let answer = call(world, agent, &words);
    assert_eq!(answer.status, 0, "{answer:?}");
    assert!(world.front().engine.task(chat).expect("answered chat").inbox.iter().any(|word| {
        word.from == Party::Person(fixtures::PARTY)
            && word.kind == MessageKind::Answer { question }
            && word.words.as_ref() == b"Go ahead."
    }));
}

#[test]
fn a_waiting_question_is_answered_from_its_plain_inbox_call() {
    const QUESTION: &[u8] = b"May I make this change?";
    let mut world = fixtures::world(fixtures::SEED);
    let chat = fixtures::question(world.front(), QUESTION);
    let (question, asking) = {
        let task = world.front().engine.task(chat).expect("asked chat");
        let question =
            task.inbox.iter().find(|word| word.kind == jig_domain_tasks::MessageKind::Question).expect("question");
        let jig_domain_tasks::Party::Task(asking) = question.from else { panic!("asking delegate") };
        (question.number, asking)
    };
    let agent = fixtures::agent(&mut world);
    answer_question_from_plain_inbox(&mut world, agent, chat, question, asking, QUESTION, None);
}

fn run_ended_without_waiting(world: &mut World<Temper, Joint>, agent: ClientId, task: u64) {
    // One second is well below the explicit sixty-second wait timeout.
    const IMMEDIATE: skein_lib::Duration = skein_lib::Duration::from_secs(1);
    let start = world.now();
    let printed =
        call(world, agent, &[b"task", b"run", task.to_string().as_bytes(), b"--until", b"ended", b"--timeout", b"60"]);
    assert_eq!(printed.status, 0, "{printed:?}");
    assert!(printed.stdout.starts_with(b"run: none\n"), "{printed:?}");
    assert!(printed.stdout.ends_with(b"met\n"), "{printed:?}");
    assert!(world.now() < start.saturating_add(IMMEDIATE), "answers before the timeout");
}

#[test]
fn run_until_ended_answers_at_once_after_the_chats_run_ended() {
    let (mut world, agent, task, _) = chat(true);
    wait_for_reply(&mut world, agent, task);
    let current = world.front().engine.task(task).expect("parked chat");
    assert!(current.attempt > 0, "a run took place");
    assert_eq!(current.phase, jig_domain_tasks::Phase::Active(jig_domain_tasks::Active::Idle));
    run_ended_without_waiting(&mut world, agent, task);
}

#[test]
fn run_until_ended_answers_at_once_for_a_task_that_never_had_a_run() {
    let mut world = fixtures::refused_hello_world(fixtures::SEED);
    let agent = fixtures::agent(&mut world);
    let printed = call(&mut world, agent, &[b"chat", b"start", b"--project", b"1", b"--key", b"no-run", b"Hello"]);
    let task = fixtures::done_task(&printed);
    assert_eq!(world.front().engine.task(task).expect("unstarted chat").attempt, 0);
    run_ended_without_waiting(&mut world, agent, task);
    assert_eq!(world.front().engine.task(task).expect("unstarted chat").attempt, 0);
}

const CLOSING_QUESTION: &[u8] = b"Close this running conversation";
const CLOSING_PREFIX: &[u8] = b"An uncommitted reply";
const CLOSING_COMMITTED: &[u8] = b"The committed conversation before closing";

fn chat_with_a_turn_in_flight() -> (World<Temper, Joint>, u64) {
    use temper_engine_domain as engine;
    let mut world = fixtures::open_world(fixtures::SEED);
    let task = fixtures::chat(world.front(), CLOSING_QUESTION);
    let attempt = world.front().engine.task(task).expect("running chat").attempt;
    let mut turn = temper_worker_domain_world::protocol::session_turn(1, 0, CLOSING_COMMITTED.len(), true);
    turn.messages[0].content =
        Box::new([smith_domain_session::llm::Block::Text { text: CLOSING_COMMITTED.into(), replay: None }]);
    let body = smith_protocol_channel::encode_turn(
        &turn,
        &smith_transcript::CEILINGS,
        &temper_worker_domain_world::protocol::endpoints(),
    )
    .expect("committed reply encodes");
    world.front().engine.send(temper_engine_domain_world::protocol::turn(
        skein_lib::Token::new(7),
        task,
        attempt,
        1,
        0,
        None,
        &body,
    ));
    world.front().engine.settle();
    world.front().engine.send(temper_engine_domain_world::protocol::up(
        skein_lib::Token::new(7),
        jig_domain_host::LinkUp::Piece {
            run: skein_lib::Token::new(task),
            attempt: skein_lib::Token::new(attempt),
            piece: jig_domain_host::Piece {
                turn: 2,
                retry: 0,
                kind: jig_domain_host::LiveKind::Text,
                offset: 0,
                bytes: CLOSING_PREFIX.into(),
            },
        },
        engine::Parts::none(),
    ));
    world.front().engine.settle();
    (world, task)
}

// temper/docs/design/client/tools.md, 4; client/README.md, 3.5–3.6.
#[test]
fn closing_during_a_turn_keeps_the_cancelled_conversation_readable() {
    use jig_domain_tasks as tasks;
    use skein_lib::Duration;
    use skein_ui::Role;
    use temper_engine_domain as engine;

    const REASON: &[u8] = b"Close this chat during its turn";
    const IN_FLIGHT_AND_ONE_CLOSING: u64 = 2;
    let (mut world, task) = chat_with_a_turn_in_flight();
    let person = world.client(fixtures::PARTY, fixtures::chat_page(task));
    world.person(person, vec![Step::See { find: Find::text(CLOSING_PREFIX), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    let in_flight = world.front().engine.task(task).expect("running chat").attempt;
    let agent = fixtures::agent(&mut world);
    let closed = call(
        &mut world,
        agent,
        &[b"chat", b"close", task.to_string().as_bytes(), b"--reason", REASON, b"--key", b"close-running"],
    );
    assert_eq!(closed.status, 0, "{closed:?}");
    let grace =
        world.front().engine.task(task).expect("chat finishing its turn").grace.as_ref().expect("finish grace").until;
    let settled_by =
        skein_lib::Time::from_nanos(grace.as_nanos()).saturating_add(world.front().engine.env.limits.fleet.grace);
    let waiting =
        fixtures::give(&mut world, agent, &[b"task", b"show", task.to_string().as_bytes(), b"--until", b"ended"]);
    assert_eq!(world.run(Until::Time(settled_by)).verdict, Verdict::Passed);
    let ended = fixtures::answered(&mut world, agent, waiting);
    assert_eq!(ended.status, 0, "{ended:?}");
    assert!(world.now() <= settled_by.saturating_add(fixtures::WITHIN), "closing settles within its configured graces");
    let key = engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)));
    let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
        world.front().engine.store.rows.get(&key)
    else {
        panic!("the cancelled conversation is retained");
    };
    assert!(
        matches!(&row.phase, tasks::Phase::Ended(tasks::Ending::Cancelled { by: tasks::Party::Person(fixtures::PARTY), reason, result: None, .. }) if reason.as_ref() == REASON),
        "{:?}",
        row.phase
    );
    assert!(row.attempt < in_flight + IN_FLIGHT_AND_ONE_CLOSING, "at most one closing run");
    assert!(world.front().engine.task(task).is_none());
    assert_eq!(world.front().runs_answered, 0, "uncommitted text is no completed result");

    let conversation = call(&mut world, agent, &[b"chat", b"show", task.to_string().as_bytes()]);
    assert_eq!(conversation.status, 0, "{conversation:?}");
    let text = std::str::from_utf8(&conversation.stdout).expect("conversation UTF-8");
    assert!(text.contains(std::str::from_utf8(CLOSING_COMMITTED).expect("committed UTF-8")), "{text}");
    assert!(
        !text.contains(std::str::from_utf8(CLOSING_PREFIX).expect("prefix UTF-8")),
        "provisional text is not committed: {text}"
    );
    world.go(person, fixtures::chat_page(task));
    world.person(
        person,
        vec![
            Step::See { find: Find::text(CLOSING_QUESTION), within: fixtures::WITHIN },
            Step::Gone { find: Find::named(Role::TextArea, b"Message"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    let until = world.now().saturating_add(Duration::from_millis(100));
    assert_eq!(world.run(Until::Time(until)).verdict, Verdict::Passed);
    assert!(world.front().engine.task(task).is_none(), "reading does not restart closing work");
}
