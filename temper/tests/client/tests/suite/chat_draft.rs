use jig_client_kit::harness::{App, ClientId, Duty, Front};
use skein_lib::{Env, Queue, Time, Token, Wall};
use smith_domain_session::llm::{Block, Decoded, Message, Returned, Role, ToolCall};
use temper_client_domain::{
    Action, Event,
    pages::{
        Act,
        chat::{
            Chat,
            document::{Mode, NoteChange, NoteWrite},
        },
    },
};
use temper_client_world::{
    app::{Root, Temper},
    fixtures,
    front::Joint,
};

const CLIENT: ClientId = ClientId(0);
// A bounded pump window covers the client's request and watch continuations.
const ROUNDS: usize = 32;
// The joined front's seeded project and scripted worker.
const PROJECT: u32 = 1;
const WORKER: Token = Token::new(7);
// Two complete note exchanges, including their JSON inputs and rendered outcomes.
const NOTE_TURN_TEXT: u32 = 1024;
type Probe = (Root, Queue<temper_client_domain::Request>, Vec<Message>);
fn limits() -> temper_client_domain::Limits {
    let mut limits = fixtures::domain_limits();
    // Two complete note exchanges are carried together in the second turn.
    limits.client.domain.turn_text = NOTE_TURN_TEXT;
    limits.client.domain.long_text = NOTE_TURN_TEXT;
    limits
}
fn env() -> Env<temper_client_domain::Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }
}
fn pump(front: &mut Joint, probe: &mut Probe) {
    pump_at(front, probe, &env());
}
fn pump_at(front: &mut Joint, probe: &mut Probe, env: &Env<temper_client_domain::Limits>) {
    for _ in 0..ROUNDS {
        Temper::take(&mut probe.0, env, &mut probe.1);
        while let Some(request) = probe.1.pop() {
            match Temper::duty(request) {
                Duty::Wire(request) => front.wire(CLIENT, env.now, env.wall, request),
                Duty::Show { address, .. } => Temper::step(&mut probe.0, env, Temper::went(address), &mut probe.1),
                _ => {}
            }
        }
        for (who, event) in front.poll(env.now, env.wall) {
            assert_eq!(who, CLIENT);
            front.received(who, &event);
            Temper::step(&mut probe.0, env, event, &mut probe.1);
        }
    }
}
fn opened() -> (Joint, u64, Probe) {
    opened_with_goal(false)
}
fn opened_with_goal(goal: bool) -> (Joint, u64, Probe) {
    let mut front = Joint::with_chat_draft(fixtures::SEED, limits());
    let task = if goal {
        fixtures::adopt_change_repository(&mut front);
        let jig_domain_people::Ask::Create { project, mut batch } = front.engine.chat_ask(PROJECT, b"Write beside us")
        else {
            panic!("chat")
        };
        // Leave 400 of the fixture's 500 units for starting its goal.
        batch[0].budget = jig_domain_people::Budget::Named;
        batch[0].authority.spend = 100;
        front.engine.send(temper_engine_domain::Event::Party(temper_engine_domain::PartyInput::Ask {
            models: Box::new([]),
            reply_to: skein_lib::ReplyTo::new(Token::new(901)),
            sign_in: front.engine.session(),
            key: [57; 16],
            ask: jig_domain_people::Ask::Create { project, batch },
        }));
        front.engine.settle();
        front.engine.last_assignment().task
    } else {
        fixtures::chat(&mut front, b"Write beside us")
    };
    front.bind(CLIENT, fixtures::PARTY);
    let root = Root::Person(Box::new(temper_client_domain::unlinked(&limits(), fixtures::SEED)));
    let mut probe = (root, Queue::uncapped(), Vec::new());
    Temper::step(&mut probe.0, &env(), Temper::start(fixtures::chat_page(task), Vec::new(), 0), &mut probe.1);
    pump(&mut front, &mut probe);
    (front, task, probe)
}
fn chat(probe: &Probe) -> &Chat {
    probe.0.person().pages().chat().expect("chat page")
}
fn act(front: &mut Joint, probe: &mut Probe, act: Act) {
    Temper::step(&mut probe.0, &env(), Event::Act { action: Action::Pages(act) }.into(), &mut probe.1);
    pump(front, probe);
}
fn write(front: &mut Joint, task: u64, sequence: u32, previous: Option<NoteWrite>, body: &str, probe: &mut Probe) {
    let run = front.assignments.last().expect("running chat assignment").clone();
    assert_eq!(run.task, task);
    let previous =
        previous.map_or_else(String::new, |write| format!(",\"name\":{},\"recalled\":{}", write.note, write.revision));
    let input = format!(
        "{{\"scope\":{{\"kind\":\"goal\",\"project\":{PROJECT},\"goal\":{task}}},\"description\":\"Our draft\",\"body\":\"{}{previous}}}",
        body.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n") + "\""
    );
    // The real root decides NoteWritten and engine-smith renders its acknowledgement.
    let (words, failed) =
        temper_engine_domain_world::host_tool::host_tool(&mut front.engine, &run, sequence, b"note", input.as_bytes());
    assert!(!failed, "note write: {}", String::from_utf8_lossy(&words));
    let id = format!("note-{sequence}").into_bytes().into_boxed_slice();
    let mut turn = temper_worker_domain_world::protocol::session_turn(sequence, 0, 0, sequence == 1);
    probe.2.extend([
        Message {
            role: Role::Assistant,
            content: Box::new([Block::ToolCall(ToolCall {
                id: id.clone(),
                name: b"note".as_slice().into(),
                input: input.into_bytes().into_boxed_slice(),
                call: Decoded::Historical,
                replay: None,
            })]),
        },
        Message {
            role: Role::User,
            content: Box::new([Block::ToolResult {
                id,
                result: Returned::Text { text: words, error: false, replay: None },
            }]),
        },
    ]);
    turn.messages = probe.2.clone().into_boxed_slice();
    let body = smith_protocol_channel::encode_turn(
        &turn,
        &smith_transcript::CEILINGS,
        &temper_worker_domain_world::protocol::endpoints(),
    )
    .expect("note transcript");
    front.engine.send(temper_engine_domain_world::protocol::turn(
        WORKER,
        task,
        run.attempt,
        sequence,
        u64::from(sequence),
        None,
        &body,
    ));
    front.engine.settle();
}
#[test]
fn a_note_call_reads_the_document() {
    let (mut front, task, mut probe) = opened();
    write(&mut front, task, 1, None, "# Our draft\nFirst\n", &mut probe);
    pump(&mut front, &mut probe);
    assert!(front.note_reads >= 2);
    let chat = chat(&probe);
    let draft = chat.draft.as_ref().expect("the call triggered a document read");
    assert_eq!(draft.title.as_ref(), b"Our draft");
    assert_eq!(draft.body.as_ref(), b"# Our draft\nFirst\n");
    assert_eq!(
        chat.newest_write(),
        Some(NoteWrite { note: draft.note, revision: 1, change: NoteChange::First { lines: 2 } })
    );
}
#[test]
fn a_second_note_call_replaces_it() {
    let (mut front, task, mut probe) = opened();
    write(&mut front, task, 1, None, "# Our draft\nFirst\n", &mut probe);
    pump(&mut front, &mut probe);
    let first = chat(&probe).newest_write().expect("first write");
    write(&mut front, task, 2, Some(first), "# Our draft\nSecond\n", &mut probe);
    pump(&mut front, &mut probe);
    let chat = chat(&probe);
    let draft = chat.draft.as_ref().expect("document");
    assert_eq!((draft.note, draft.revision, draft.body.as_ref()), (first.note, 2, b"# Our draft\nSecond\n".as_slice()));
    assert_eq!(
        chat.newest_write(),
        Some(NoteWrite { note: first.note, revision: 2, change: NoteChange::Later { added: 1, removed: 1, first: 2 } })
    );
}
#[test]
fn a_refused_read_leaves_it() {
    let (mut front, task, mut probe) = opened();
    write(&mut front, task, 1, None, "# Our draft\nFirst\n", &mut probe);
    pump(&mut front, &mut probe);
    let first = chat(&probe).newest_write().expect("first write");
    let draft = chat(&probe).draft.clone();
    front.note_read_ends.push_back(jig_client::contract::ReadNone::Refused);
    write(&mut front, task, 2, Some(first), "# Our draft\nSecond\n", &mut probe);
    pump(&mut front, &mut probe);
    assert!(front.note_read_ends.is_empty(), "the read was refused");
    assert_eq!(chat(&probe).draft, draft);
    assert_eq!(chat(&probe).newest_write().expect("second write").revision, 2);
}
#[test]
fn a_chat_opened_again_starts_in_chat() {
    let (mut front, task, mut probe) = opened();
    act(&mut front, &mut probe, Act::Mode { mode: Mode::Draft });
    assert_eq!(chat(&probe).mode, Mode::Draft);
    for address in [fixtures::inbox(), fixtures::chat_page(task)] {
        Temper::step(
            &mut probe.0,
            &env(),
            Event::Went { place: temper_client_domain::Place::Address(address) }.into(),
            &mut probe.1,
        );
        pump(&mut front, &mut probe);
    }
    assert_eq!(chat(&probe).mode, Mode::Chat);
}

fn goal_from_document() -> (Joint, u64, u64, Probe) {
    let (mut front, task, mut probe) = opened_with_goal(true);
    write(&mut front, task, 1, None, "# Our draft\nFirst\n", &mut probe);
    pump(&mut front, &mut probe);
    act(
        &mut front,
        &mut probe,
        Act::OpenStart { draft: Some(b"docs/design/draft/our-draft.md".as_slice().into()), chat: Some(task) },
    );
    assert!(probe.0.person().pages().start().is_some(), "the chat opens the shared sheet");
    act(&mut front, &mut probe, Act::StartGoal);
    let goal = probe.0.person().pages().task().expect("the goal page opens").number;
    (front, task, goal, probe)
}
#[test]
fn a_goal_starts_from_a_document() {
    let (mut front, task, goal, mut probe) = goal_from_document();
    let root = probe.0.person();
    let goal_row = front.engine.task(goal).expect("created goal");
    let expected = format!(
        "our-draft\n\nImplement this draft. First commit it, as it stands, at docs/design/draft/our-draft.md.\n\n# Our draft\nFirst\n\n\nYou are a goal: the work your person asked for, yours to plan and to change; you start no goal but an advisor.\nDraft: docs/design/draft/our-draft.md\nChat: T{task}"
    );
    assert_eq!(goal_row.spec.words.as_ref(), expected.as_bytes());
    assert!(root.pages().start().is_none());
    assert_eq!(root.forge().goals_from(PROJECT, task).collect::<Vec<_>>(), [(goal, env().wall.as_nanos())]);
    assert!(root.pages().task().unwrap().document(root.client()).is_some());
    Temper::step(&mut probe.0, &env(), Temper::went(fixtures::chat_page(task)), &mut probe.1);
    pump(&mut front, &mut probe);
    let root = probe.0.person();
    let chat = root.pages().chat().unwrap();
    assert_eq!(chat.goal_from(root), Some((goal, env().wall.as_nanos())));
    assert_eq!(chat.goal_chip(root).expect("linked goal chip").task, goal);
    act(
        &mut front,
        &mut probe,
        Act::OpenStart { draft: Some(b"docs/design/draft/our-draft.md".as_slice().into()), chat: Some(task) },
    );
    assert!(probe.0.person().pages().start().is_none(), "a linked goal prevents another start");
}

#[test]
fn a_goal_from_the_sheet_asks_its_person_who_answers_under_needs_you() {
    let (mut front, _, goal, mut probe) = goal_from_document();
    answer_person_task(&mut front, goal, &mut probe);
}

fn answer_person_task(front: &mut Joint, goal: u64, probe: &mut Probe) {
    use jig_client::contract::ObjectKey;
    use jig_domain_tasks as tasks;
    use temper_client_domain::{Address, address::Filter, pages::needs};
    use temper_engine_domain as engine;
    // The note's host tool defers rendering; typed driver calls render their own replies.
    front.engine.protocol.defer_render = false;
    let attempt = front.engine.task(goal).expect("running goal").attempt;
    let mut child = temper_engine_domain_world::fixtures::report_delegate(
        b"Which colour?\n\nChoose the colour for the work.\n\nSaw: Both colours work.\nChoice: Blue (recommended)\nChoice: Green",
        Box::new([]),
    );
    child.executor = tasks::Executor::Person(tasks::PersonAddress::Person(fixtures::PARTY));
    child.contract = tasks::Contract::Report { words: 4096 };
    child.authority.tools = tasks::Tools(0);
    child.authority.budget.spend = 0;
    front.engine.call(
        WORKER,
        goal,
        attempt,
        28,
        engine::Call { completion: 28, position: 1, tool: engine::Tool::Delegate { batch: Box::new([child]) } },
    );
    front.engine.settle();
    let Some(engine::CallAnswer::Delegated(children)) = front.engine.answered(28) else {
        panic!("person delegation: {:?}", front.engine.answered(28));
    };
    let child = children[0].task;
    front.engine.answer(WORKER, goal, attempt, 0, tasks::End::Parked, engine::Parts::none());
    front.engine.settle();
    Temper::step(&mut probe.0, &env(), Temper::went(Address::Inbox { filter: Filter::ALL }), &mut probe.1);
    pump(front, probe);
    let root = probe.0.person();
    let card = root.pages().needs().shown(root.pages().inbox().expect("inbox")).expect("person question");
    let object = ObjectKey::PersonTask(child);
    assert_eq!(card.object, object);
    act(front, probe, Act::Needs(needs::Act::Choose { object, words: Box::from(*b"Blue") }));
    assert!(probe.0.person().pages().needs().held(object));
    let mut due = env();
    due.now = due.now.saturating_add(needs::HOLD);
    due.wall = Wall::from_nanos(needs::HOLD.as_nanos());
    Temper::fire(&mut probe.0, &due, &mut probe.1);
    pump_at(front, probe, &due);
    let stored = front.engine.store.rows.get(&engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(child))));
    let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) = stored else {
        panic!("completed person task: {stored:?}");
    };
    assert!(
        matches!(&row.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words })) if words.as_ref() == b"Blue")
    );
    let resumed = front.assignments.last().expect("goal resumed");
    assert_eq!(resumed.task, goal);
    assert!(
        resumed.messages.iter().any(|message| matches!(
            message.kind,
            tasks::MessageKind::Result(tasks::ResultKind::Report)
        ) && message.words.as_ref() == b"Blue"
            && message.from == tasks::Party::Task(child)),
        "goal wake: {:?}",
        resumed.messages
    );
}

#[test]
fn a_goal_from_goals_asks_its_person_and_the_answer_wakes_it() {
    let (mut front, _, mut probe) = opened_with_goal(true);
    Temper::step(
        &mut probe.0,
        &env(),
        Temper::went(temper_client_domain::Address::Goals {
            project: PROJECT,
            filter: temper_client_domain::GoalsFilter::All,
        }),
        &mut probe.1,
    );
    pump(&mut front, &mut probe);
    let limits = limits();
    let words = fixtures::phrasebooks(&limits);
    let bounds = fixtures::view_bounds(&limits, &words);
    let mut view = temper_client_view::View::new(&bounds, &limits, words, fixtures::FORGE);
    temper_client_view::build(&mut view, probe.0.person(), env().wall, &mut Queue::uncapped());
    let button = view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == skein_ui::Role::Button && node.name.as_deref() == Some(b"Start one from a draft"))
        .expect("Goals start button")
        .id;
    let action = temper_client_view::act(&mut view, button, skein_ui::Act::Press).expect("start action");
    assert_eq!(action, Action::Pages(Act::OpenStart { draft: None, chat: None }));
    Temper::step(&mut probe.0, &env(), Event::Act { action }.into(), &mut probe.1);
    pump(&mut front, &mut probe);
    assert_eq!(probe.0.person().pages().start().expect("sheet").project, PROJECT);
    act(&mut front, &mut probe, Act::StartDraft { words: b" docs/design/draft/our-draft.md ".as_slice().into() });
    act(&mut front, &mut probe, Act::StartGoal);
    let goal = probe.0.person().pages().task().expect("goal page").number;
    assert_eq!(
        front.engine.task(goal).expect("goal").spec.words.as_ref(),
        b"our-draft\n\nBuild docs/design/draft/our-draft.md.\nDraft: docs/design/draft/our-draft.md"
    );
    answer_person_task(&mut front, goal, &mut probe);
}
