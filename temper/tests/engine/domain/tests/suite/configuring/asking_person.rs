//! Joined person reads (temper/docs/design/domain/agent/ask-temper.md, 1).
use super::*;
use serde_json::json;
use temper_engine_domain_world::{fixtures::report_delegate, protocol, walking};

const GOAL: &[u8] = b"Add CSV export";
const QUESTION: &[u8] = b"Should the export include headers?";
const HEADLINE: &str = "report: The export lands this week.";

pub(crate) struct PersonWorld {
    pub world: World,
    pub owner: engine::Assignment,
    pub member: engine::Assignment,
    pub delegate: engine::Assignment,
    pub goal: engine::Assignment,
    pub change: u64,
    pub producer: engine::Assignment,
    pub elsewhere: u64,
    pub turn_text: String,
}

fn call(world: &mut World, run: &engine::Assignment, number: u64, tool: engine::Tool) -> engine::CallAnswer {
    world.driver.protocol.defer_render = false;
    world.driver.call(
        CHANNEL,
        run.task,
        run.attempt,
        number,
        engine::Call { completion: 1, position: u32::try_from(number).expect("fixture call"), tool },
    );
    world.driver.settle();
    world
        .driver
        .answered(number)
        .unwrap_or_else(|| panic!("call {number} answered: {:?}", world.driver.delivered))
        .clone()
}

fn producer(world: &mut World, change: u64) -> engine::Assignment {
    for _ in 0..20 {
        if let Some(run) = world.driver.delivered.iter().find_map(|delivery| {
            if let Delivery::Assigned { assignment, .. } = delivery
                && world.driver.task(assignment.task).is_some_and(|row| row.requester == tasks::Party::Task(change))
            {
                Some(assignment.clone())
            } else {
                None
            }
        }) {
            return run;
        }
        let next = world.driver.env.now.as_nanos() + 1_000_000_000;
        world.driver.fire_at(skein_lib::Duration::from_nanos(next));
        world.driver.settle();
    }
    let changes: Vec<_> = world.driver.store.rows.values().filter(|row| matches!(row, engine::Record::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::Change(_)))).collect();
    panic!("producer assigned: {changes:?}");
}

#[expect(clippy::too_many_lines, reason = "one joined fixture seeds both projects and their real work")]
pub(crate) fn world() -> PersonWorld {
    let mut config = config();
    let mut projects = config.core.seed_projects.into_vec();
    projects.push(core::SeedProject { number: 2, name: "Other project".into(), policy: projects[0].policy.clone() });
    config.core.seed_projects = projects.into();
    let mut owners = config.core.owners.into_vec();
    let mut other_owner = owners[1].clone();
    other_owner.project = 2;
    owners.push(other_owner);
    config.core.owners = owners.into();
    let mut bounds = limits();
    bounds.authority.projects = 2;
    bounds.authority.segments = 8;
    bounds.authority.batch = 2;
    bounds.people.projects = 3;
    bounds.people.initial_owners = 3;
    bounds.people.people = 6;
    bounds.people.holdings = 6;
    bounds.people.sign_ins = 3;
    bounds.people.requests = 16;
    bounds.people.inbox_entries = 4;
    bounds.people.inbox_tasks = 24;
    bounds.people.inbox_recipients = 64;
    bounds.tasks.tasks = 12;
    bounds.tasks.funders = 8;
    bounds.tasks.pools = 3;
    bounds.tasks.project_tasks = 12;
    bounds.tasks.tree_tasks = 12;
    bounds.tasks.delegates = 4;
    bounds.tasks.batch = 4;
    bounds.tasks.spec_bytes = 512;
    bounds.tasks.parameters = 8;
    // Nine narrowed forge grants, each with this repository's 30 name bytes.
    bounds.tasks.authority_bytes = 270;
    bounds.tasks.inbox_messages = 8;
    bounds.tasks.inbox_bytes = 8 * bounds.tasks.message_bytes;
    bounds.tasks.run_inbox = 8;
    bounds.tasks.depth = 4;
    bounds.documents.rows = 12;
    bounds.documents.chips = 12;
    bounds.fleet.slots = 6;
    bounds.fleet.attempts = 6;
    bounds.fleet.workstreams = 6;
    bounds.call_records = 4;
    bounds.journal.transcript_bytes = 65_536;
    bounds.loads.rows = 8;
    let period_budget = config.core.defaults.rules.period_spend;
    let mut world = World::with_limits(config, &bounds);
    world.driver.send(Event::Worker(engine::WorkerInput::Link { channel: CHANNEL, link: engine::Link::Lost }));
    world.driver.settle();
    let mut greeting = protocol::hello_slots(CHANNEL, 6, Box::new([]));
    let Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut greeting else {
        panic!("worker hello")
    };
    hello.bounds.time = skein_lib::Duration::from_secs(28_800);
    hello.bounds.turns = 800;
    hello.bounds.message_bytes = bounds.tasks.message_bytes;
    hello.bounds.inbox = bounds.tasks.run_inbox;
    world.driver.send(greeting);
    world.driver.settle();
    world.driver.send(Event::Credential { number: 1, generation: 1, valid: skein_lib::Duration::from_secs(3600) });
    world.driver.settle();
    world.driver.forge = Some(temper_engine_domain_world::forge_stand::ForgeStand::new());
    world.driver.connect_forges();
    adopt_repository(&mut world, PROJECT, b"repo", 29);

    let mut ask = walking::goal_from(
        world.driver.chat_ask(PROJECT, GOAL),
        people::TaskExecutor::Agent { charter: CHAT },
        jig_test_profile::fixtures::Goal { budget: 512, priority: 1, deadline: None },
    );
    let people::Ask::Create { batch, .. } = &mut ask else { panic!("goal") };
    // Its conversation carries the child's question to the goal's person.
    batch[0].flags.conversation = true;
    world.ask(world.owner, 30, ask);
    assert!(
        matches!(world.reply(30), people::Reply::Outcome(people::Outcome::Created { .. })),
        "{:?}",
        world.reply(30)
    );
    let goal = world.driver.last_assignment();
    let (note, error) = host_tool(
        &mut world.driver,
        &goal,
        10,
        b"note",
        json!({
            "scope":{"kind":"goal","project":PROJECT,"goal":goal.task},
            "description":HEADLINE,"body":"I have planned the export."
        })
        .to_string()
        .as_bytes(),
    );
    assert!(!error, "{}", String::from_utf8_lossy(&note));

    let children = [b"Build the export".as_slice(), b"Check the export format".as_slice()].map(|words| {
        let mut child = report_delegate(words, Box::new([]));
        child.authority.tools = world.driver.task(goal.task).expect("goal").authority.tools;
        child.authority.budget.spend = 32;
        child.flags.retention = tasks::Retention::Kept;
        child
    });
    let answer = call(&mut world, &goal, 40, engine::Tool::Delegate { batch: children.into() });
    let engine::CallAnswer::Delegated(made) = answer else { panic!("goal's children delegated: {answer:?}") };
    let ended = world
        .driver
        .delivered
        .iter()
        .find_map(|delivery| {
            if let Delivery::Assigned { assignment, .. } = delivery
                && assignment.task == made[1].task
            {
                Some(assignment.clone())
            } else {
                None
            }
        })
        .expect("ended child's assignment");
    let live = world
        .driver
        .delivered
        .iter()
        .find_map(|delivery| {
            if let Delivery::Assigned { assignment, .. } = delivery
                && assignment.task == made[0].task
            {
                Some(assignment.clone())
            } else {
                None
            }
        })
        .expect("live child's assignment");
    world.driver.answer(
        CHANNEL,
        ended.task,
        ended.attempt,
        0,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"Format checked.".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.driver.settle();
    let answer = call(
        &mut world,
        &live,
        42,
        engine::Tool::Message { target: goal.task, form: engine::MessageForm::Question, words: QUESTION.into() },
    );
    let engine::CallAnswer::Sent { message: question } = answer else { panic!("{answer:?}") };
    world.driver.send(protocol::turn(
        CHANNEL,
        goal.task,
        goal.attempt,
        1,
        0,
        Some(question),
        &transcript(b"Read the question; wait for the person."),
    ));
    world.driver.settle();
    let answer = call(
        &mut world,
        &goal,
        41,
        engine::Tool::Change {
            changes: Box::new([engine::ChangeCall::words(
                b"Implement CSV export".as_slice().into(),
                Some(b"org/repo".as_slice().into()),
            )]),
        },
    );
    let engine::CallAnswer::Delegated(made) = answer else { panic!("{answer:?}") };
    let change = made[0].task;
    let producer = producer(&mut world, change);
    let turn_text = "The export keeps each café row in order.\n".repeat(180);
    let transcript = transcript(turn_text.as_bytes());
    world.driver.send(protocol::turn(CHANNEL, producer.task, producer.attempt, 1, 0, None, &transcript));
    world.driver.settle();
    let key = engine::Key::Core(core::Key::Core(core::CoreKey::Turn {
        task: producer.task,
        attempt: producer.attempt,
        turn: 1,
    }));
    let Some(engine::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn)))) =
        world.driver.store.rows.get(&key)
    else {
        panic!("producer's committed turn")
    };
    assert_eq!(turn.transcript, transcript);
    let branch = producer.workspace.repositories[0].push.as_ref().expect("producer's branch");
    world.driver.forge.as_mut().expect("fake forge").push_branch(2, branch);
    world.driver.answer(
        CHANNEL,
        producer.task,
        producer.attempt,
        0,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"CSV export implemented".as_slice().into()),
                connector: 1,
                kind: 2,
                resource: temper_engine_domain_forge::tag(1, 2),
                words: b"Export pushed.".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.driver.settle();

    world.driver.send(Event::Party(PartyInput::Period { project: 2, period: 1, budget: period_budget, next: None }));
    world.driver.settle();
    let mut ask = world.driver.chat_ask(2, b"ELSEWHERE private work");
    let people::Ask::Create { batch, .. } = &mut ask else { panic!("other task") };
    batch[0].executor = people::TaskExecutor::Person(people::PersonAddress::Role(0));
    batch[0].flags.conversation = false;
    batch[0].flags.listed = true;
    batch[0].budget = people::Budget::Named;
    batch[0].authority.spend = 32;
    batch[0].authority.delegation = people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let mut other = batch[0].clone();
    other.words = b"ELSEWHERE more private work".as_slice().into();
    *batch = Box::new([batch[0].clone(), other]);
    world.ask(world.owner, 31, ask);
    let people::Reply::Outcome(people::Outcome::Created { first: elsewhere, .. }) = *world.reply(31) else {
        panic!("{:?}", world.reply(31))
    };
    let member = world.member();
    let owner = world.chat(ASK_TEMPLATE, world.owner, 32);
    let member = world.chat(ASK_TEMPLATE, member, 33);
    assert_eq!(owner.charter, ASK_CHARTER);
    assert_eq!(member.charter, ASK_CHARTER);
    PersonWorld { world, owner, member, delegate: live, goal, change, producer, elsewhere, turn_text }
}

fn transcript(text: &[u8]) -> Box<[u8]> {
    use smith_transcript::v7 as s;
    let bounds = &s::CEILINGS;
    let block =
        s::Block::Text(s::Text::new(bounds, s::TextParts { text: text.into(), replay: None }).expect("turn text"));
    let message = s::Message::new(
        bounds,
        s::MessageParts { role: s::Role::Assistant, blocks: skein_lib::List::from_boxed(Box::new([block])) },
    )
    .expect("assistant message");
    let source = s::Turn::new(
        bounds,
        s::TurnParts {
            system: None,
            model: b"scripted".as_slice().into(),
            effort: None,
            kind: s::TurnKind::Ordinary,
            opening: None,
            endpoint: b"scripted".as_slice().into(),
            dialect: 1,
            place: 1,
            usage: s::Usage::new(
                bounds,
                s::UsageParts { input: None, output: None, cache_read: None, cache_write: None, reasoning: None },
            )
            .expect("unused token counts"),
            spent: 0,
            jobs: 0,
            messages: skein_lib::List::from_boxed(Box::new([message])),
        },
    )
    .expect("scripted turn");
    let mut out = skein_lib::Writer::new(usize::try_from(source.measure()).expect("bounded turn"));
    source.encode(&mut out).expect("measured transcript");
    out.finish()
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table joins the person's addresses, refusals and continuations")]
fn an_asking_chat_reads_its_project_as_its_person() {
    let mut person = world();
    let goal = person.goal.task;
    let change = person.change;
    let producer = person.producer.task;
    let attempt = person.producer.attempt;
    let elsewhere = person.elsewhere;
    let mut completion = 1;
    let mut owner_tasks = None;
    for (run, line, heading, content, following) in [
        (
            &person.owner,
            "me".to_owned(),
            "you: Person · owner in project 1, Project 1".to_owned(),
            "1 question",
            Some("inbox · project · goals · tasks"),
        ),
        (
            &person.owner,
            "project".to_owned(),
            "project 1, Project 1".to_owned(),
            "org/repo",
            Some("goals · tasks · changes · inbox · settings"),
        ),
        (
            &person.owner,
            "goals".to_owned(),
            "goals of project 1, Project 1: 1, by priority".to_owned(),
            HEADLINE,
            Some("task N · task N tree · goals ended"),
        ),
        (
            &person.owner,
            "goals ended".to_owned(),
            "ended goals of project 1, Project 1: 0, the latest ended first".to_owned(),
            "0",
            Some("task N · goals"),
        ),
        (
            &person.owner,
            "tasks".to_owned(),
            "live tasks of project 1, Project 1: 5, the latest active first".to_owned(),
            "Add CSV export",
            Some("task N · goals · inbox"),
        ),
        (
            &person.owner,
            format!("task {goal}"),
            format!("task {goal}, Add CSV export · a goal · working, waiting on Person"),
            HEADLINE,
            Some("task N tree · task N activity · change N · tasks"),
        ),
        (
            &person.owner,
            format!("task {change}"),
            format!("task {change}, Implement CSV export · a change · idle, waiting for news"),
            "a procedure",
            Some("task N tree · task N activity · change N · tasks"),
        ),
        (
            &person.owner,
            format!("task {goal} tree"),
            format!("delegates of task {goal}: 2 live, 1 ended"),
            "Check the export format",
            Some("task N · task N activity · task N tree"),
        ),
        (
            &person.owner,
            format!("task {change} activity"),
            format!("activity of task {change}: 2 events, oldest first"),
            "Export pushed.",
            Some("task N turn A.T · task N · task N tree"),
        ),
        (
            &person.owner,
            format!("task {producer} turn {attempt}.1"),
            format!("turn {attempt}.1 of task {producer} · $0.00"),
            "café row",
            Some("task N activity · task N"),
        ),
        (
            &person.owner,
            format!("change {change}"),
            format!("change {change} · org/repo temper/1/c{change} into main · checking"),
            "gate 1: waiting",
            Some("task N · task N activity · changes · settings gates OWNER/NAME"),
        ),
        (
            &person.owner,
            "changes".to_owned(),
            "changes of project 1, Project 1: 0 queued, 0 landing by turn".to_owned(),
            "landing by turn: 0",
            Some("change N · settings landing · ways"),
        ),
        (
            &person.owner,
            format!("task {elsewhere}"),
            format!("see task {elsewhere}: not read: nothing you may read in this project is there."),
            "not read",
            None,
        ),
        (
            &person.owner,
            format!("task {elsewhere} activity"),
            format!("see task {elsewhere} activity: not read: nothing you may read in this project is there."),
            "not read",
            None,
        ),
        (
            &person.owner,
            format!("change {goal}"),
            format!("see change {goal}: not read: nothing you may read in this project is there."),
            "not read",
            None,
        ),
        (
            &person.owner,
            "task 999999".to_owned(),
            "see task 999999: not read: nothing you may read in this project is there.".to_owned(),
            "not read",
            None,
        ),
        (
            &person.member,
            "me".to_owned(),
            "you: Member · member in project 1, Project 1".to_owned(),
            "member",
            Some("inbox · project · goals · tasks"),
        ),
        (
            &person.member,
            "tasks".to_owned(),
            "live tasks of project 1, Project 1: 5, the latest active first".to_owned(),
            "Add CSV export",
            Some("task N · goals · inbox"),
        ),
        (
            &person.delegate,
            "tasks".to_owned(),
            "see tasks: not read: this chat has no person to read as.".to_owned(),
            "no person",
            None,
        ),
        (
            &person.owner,
            "inbox".to_owned(),
            "waiting on you in project 1, Project 1: 1 or more, oldest first; each is answered on Needs you".to_owned(),
            std::str::from_utf8(QUESTION).unwrap(),
            Some("task N · me"),
        ),
    ] {
        let before = person.world.driver.store.rows.clone();
        let mut address = line.clone();
        let mut whole = String::new();
        let mut from = 0;
        let mut parts = 0;
        loop {
            let (body, error) = host_tool(
                &mut person.world.driver,
                run,
                completion,
                b"see",
                json!({"address":address}).to_string().as_bytes(),
            );
            completion += 1;
            parts += 1;
            let text = std::str::from_utf8(&body).unwrap();
            assert_eq!(error, following.is_none(), "{address}: {text}");
            assert!(!text.contains("ELSEWHERE"), "{address}: {text}");
            assert!(body.len() <= usize::try_from(person.world.driver.env.limits.call_answer_bytes - 3).unwrap());
            assert!(body.len() <= usize::try_from(smith_agent_world::LIMITS.run.host_reply_bytes).unwrap());
            if let Some(bound) = person.world.driver.env.limits.host_reply_bytes {
                assert!(body.len() <= usize::try_from(bound).unwrap());
            }
            assert_eq!(person.world.driver.store.rows, before, "{address} commits nothing");
            let Some(following) = following else {
                assert_eq!(text, heading);
                whole.push_str(text);
                break;
            };
            assert!(text.ends_with(&format!("see: {following}\n")), "{address}: {text}");
            let end = text.find("more: ").or_else(|| text.find("see: ")).unwrap();
            assert!(end <= 6144, "{address}: body within its part");
            let next = text.lines().find_map(|line| line.strip_prefix("more: "));
            let bytes = next.map_or(end, |token| {
                assert!(token.starts_with(&format!("{}@", line.replace(' ', "+"))));
                let offset = token.rsplit_once('@').unwrap().1.parse::<usize>().unwrap();
                assert!(offset > from, "continuation advances");
                offset - from
            });
            whole.push_str(&text[..bytes]);
            from += bytes;
            if let Some(token) = next {
                address = format!("more {token}");
            } else {
                break;
            }
        }
        eprintln!("{line}: {}", whole.lines().next().unwrap());
        assert_eq!(whole.lines().next(), Some(heading.as_str()), "{line}: {whole}");
        assert!(whole.contains(content), "{line}: {whole}");
        if line.starts_with(&format!("task {producer} turn ")) {
            assert!(parts > 1, "the turn needs more");
            assert_eq!(whole, format!("{heading}\n{}\n", person.turn_text));
        }
        if line == "tasks" && following.is_some() {
            if run.task == person.owner.task {
                owner_tasks = Some(whole);
            } else {
                assert_eq!(Some(whole), owner_tasks, "the member reads the same tasks");
            }
        }
    }
}
