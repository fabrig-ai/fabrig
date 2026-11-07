//! A chat from first words through its change landing and closure (client/README.md, 3.5 and 5.7).
use jig_client_kit::harness::Until;
use skein_fake_person::{Find, Step};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_world::fixtures;
#[test]
fn a_message_in_a_chat_is_answered_in_its_conversation() {
    let mut world = fixtures::world(fixtures::SEED);
    // This result-ended chat stays running until the person sends the message;
    // its initial task watch need not beat an already offered completion.
    world.front().reply_in_pieces(&[b"Working"]);
    let task = fixtures::chat(world.front(), b"Hello");
    let person = world.client(fixtures::PARTY, fixtures::chat_page(task));
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"Message"), within: fixtures::WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"Message"), words: Box::from(*b"And then?") },
            Step::Press { find: Find::named(Role::Button, b"Send") },
            Step::See { find: Find::text(b"And then?"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert!(world.front().release_reply(), "finish the held run after the person's message");
    world.person(person, vec![Step::See { find: Find::text(b"Answered by the run"), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}
#[test]
fn a_task_with_the_chat_label_opens_as_a_chat() {
    let mut world = fixtures::world(fixtures::SEED);
    let task = fixtures::chat(world.front(), b"Hello");
    let person = world.client(fixtures::PARTY, temper_client_domain::Address::Task { task });
    world.person(person, vec![Step::See { find: Find::named(Role::TextArea, b"Message"), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}

const DIAGNOSTIC_NOW: skein_lib::Time = skein_lib::Time::ZERO;
const DIAGNOSTIC_WALL: skein_lib::Wall = skein_lib::Wall::EPOCH;
const POLL_ROUNDS: usize = 12;
const START_OFFSET: i32 = 0;
const DIAGNOSTIC_CLIENT: jig_client_kit::harness::ClientId = jig_client_kit::harness::ClientId(0);
#[test]
fn prepared_chat_answer_reaches_the_pages_task_watch() {
    use jig_client_kit::harness::{App, Duty, Front};
    use temper_client_world::{app::Temper, front::Joint};
    let limits = fixtures::domain_limits();
    let mut front = Joint::new(fixtures::SEED, limits);
    front.answer_runs = true;
    let task = fixtures::chat(&mut front, b"Hello");
    front.bind(DIAGNOSTIC_CLIENT, fixtures::PARTY);
    let mut root =
        temper_client_world::app::Root::Person(Box::new(temper_client_domain::unlinked(&limits, fixtures::SEED)));
    let env = skein_lib::Env { now: DIAGNOSTIC_NOW, wall: DIAGNOSTIC_WALL, limits };
    let mut out = skein_lib::Queue::uncapped();
    Temper::step(&mut root, &env, Temper::start(fixtures::chat_page(task), Vec::new(), START_OFFSET), &mut out);
    for _ in 0..POLL_ROUNDS {
        Temper::take(&mut root, &env, &mut out);
        while let Some(request) = out.pop() {
            if let Duty::Wire(request) = Temper::duty(request) {
                front.wire(DIAGNOSTIC_CLIENT, env.now, env.wall, request);
            }
        }
        for (_, event) in front.poll(env.now, env.wall) {
            front.received(DIAGNOSTIC_CLIENT, &event);
            Temper::step(&mut root, &env, event, &mut out);
        }
    }
    let chat = root.person().pages().chat().expect("chat page");
    assert!(front.engine.task(task).is_none(), "the answer ends the real run");
    assert_eq!(
        chat.task(root.person().client()).expect("task document").chip.phase,
        jig_client::contract::Phase::Ended
    );
}
type ChatWorld = jig_client_kit::harness::World<temper_client_world::app::Temper, temper_client_world::front::Joint>;
const CHANGE_BRANCH: &[u8] = b"temper/chat-story";
const CHANGE_TITLE: &[u8] = b"Export the table as CSV";
const CHANGE_CALL: u64 = 1109;
const STORY_STEPS: u32 = 200;
const STORY_WORKER: skein_lib::Token = skein_lib::Token::new(7);
const PAST_CLOSE_GRACE: skein_lib::Duration = skein_lib::Duration::from_secs(6);

fn printed(world: &ChatWorld, person: jig_client_kit::harness::ClientId) -> String {
    let limits = fixtures::domain_limits();
    let words = fixtures::phrasebooks(&limits);
    let bounds = fixtures::view_bounds(&limits, &words);
    let mut print = skein_ui_text::Print::new(&bounds.tree).expect("print bounds");
    skein_ui_text::render(world.tree(person), b"temper", None, &mut print);
    String::from_utf8(print.text.as_slice().to_vec()).expect("printed UTF-8")
}

// client/README.md, 5.7; domain/forge.md, 8.2–8.8: the chat delegates an already pushed
// branch to the landing procedure; CI and the review gate still run in the real engine.
fn delegate_change(front: &mut temper_client_world::front::Joint, chat: u64) -> u64 {
    use jig_domain_tasks as tasks;
    use temper_engine_domain as engine;
    let driver = &mut front.engine;
    let parent = driver.task(chat).expect("requesting chat");
    assert_eq!(parent.authority.delegation.depth, 2, "chat authority: {:?}", parent.authority);
    let attempt = parent.attempt;
    let read = parent.inbox.last().map(|message| message.number);
    let mut authority = parent.authority.clone();
    authority.tools = tasks::Tools(0);
    authority.delegation.kinds = Box::new([tasks::AuthorityExecutor::Charter(4)]);
    authority.delegation.depth = 1;
    authority.delegation.tasks = 1;
    authority.budget.spend = 40;
    driver.call(
        STORY_WORKER,
        chat,
        attempt,
        CHANGE_CALL,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::Delegate {
                batch: Box::new([engine::Delegate {
                    executor: tasks::Executor::Procedure { connector: 1, code: 2 },
                    spec: tasks::Spec {
                        job: None,
                        asks: false,
                        words: CHANGE_TITLE.into(),
                        parameters: Box::new([
                            tasks::Parameter::Resource {
                                name: 1,
                                connector: 1,
                                resource: temper_engine_domain_forge::tag(1, 7),
                            },
                            tasks::Parameter::Bytes { name: 3, value: CHANGE_BRANCH.into() },
                        ]),
                        inputs: Box::new([]),
                    },
                    contract: tasks::Contract::Change { connector: 1, kind: 1, words: 128 },
                    authority,
                    ..temper_engine_domain_world::fixtures::report_delegate(CHANGE_TITLE, Box::new([]))
                }]),
            },
        },
    );
    for _ in 0..STORY_STEPS {
        driver.advance(true);
        if driver.answered(CHANGE_CALL).is_some() {
            break;
        }
    }
    let Some(engine::CallAnswer::Delegated(children)) = driver.answered(CHANGE_CALL) else {
        panic!("chat delegates its change: {:?}", driver.answered(CHANGE_CALL));
    };
    let change = children[0].task;
    record_delegation(driver, chat, attempt, change, read);
    driver.answer(STORY_WORKER, chat, attempt, 1, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    change
}

fn record_delegation(
    driver: &mut temper_engine_domain_world::direct::Driver,
    chat: u64,
    attempt: u64,
    change: u64,
    read: Option<u64>,
) {
    use smith_domain_session::llm::{Block, Decoded, Message, Returned, Role, ToolCall};
    let mut turn = temper_worker_domain_world::protocol::session_turn(1, 1, 0, true);
    turn.messages = Box::new([
        Message {
            role: Role::Assistant,
            content: Box::new([Block::ToolCall(ToolCall {
                id: b"change".as_slice().into(),
                name: b"delegate".as_slice().into(),
                input: b"{}".as_slice().into(),
                call: Decoded::Historical,
                replay: None,
            })]),
        },
        Message {
            role: Role::User,
            content: Box::new([Block::ToolResult {
                id: b"change".as_slice().into(),
                result: Returned::Text {
                    text: format!("Delegated tasks: {change}").into_bytes().into(),
                    error: false,
                    replay: None,
                },
            }]),
        },
    ]);
    let body = smith_protocol_channel::encode_turn(
        &turn,
        &smith_transcript::CEILINGS,
        &temper_worker_domain_world::protocol::endpoints(),
    )
    .expect("delegation turn encodes");
    driver.send(temper_engine_domain_world::protocol::turn(STORY_WORKER, chat, attempt, 1, 1, read, &body));
}

fn chat_from_new_chat(world: &mut ChatWorld, task: u64) -> jig_client_kit::harness::ClientId {
    use temper_client_domain::Address;
    let person = world.client(fixtures::PARTY, fixtures::chats_page());
    world.at_text_shell(person);
    let see = |find| Step::See { find, within: fixtures::WITHIN };
    let press = |role, name: &[u8]| Step::Press { find: Find::exactly_named(role, name) };
    let composer = || Find::exactly_named(Role::TextArea, b"What do you want done?");
    world.person(person, vec![see(Find::exactly_named(Role::Link, b"+ New chat"))]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert!(printed(world, person).contains("+ New chat"));
    world.person(person, vec![press(Role::Link, b"+ New chat"), see(composer())]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.root(person).person().pages().address(), &Address::NewChat { draft: false });
    assert!(printed(world, person).contains("What do you want done?"));
    world.person(
        person,
        vec![
            Step::Type { find: composer(), words: b"Add CSV export".as_slice().into() },
            Step::Send { find: composer() },
            see(Find::exactly_named(Role::Heading { level: 1 }, "T1 · Add CSV export".as_bytes())),
            see(Find::exactly_named(Role::TextArea, b"Message")),
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.root(person).person().pages().address(), &Address::Chat { task });
    assert!(printed(world, person).contains("Add CSV export"));
    world.back(person);
    world.person(person, vec![see(Find::named(Role::Heading { level: 1 }, b"Talk to an agent"))]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "Back skips NewChat: {outcome:?}");
    assert_eq!(world.root(person).person().pages().address(), &fixtures::chats_page());
    world.forward(person);
    world.person(person, vec![see(Find::exactly_named(Role::TextArea, b"Message"))]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    person
}

fn approve_change(world: &mut ChatWorld, task: u64) {
    let gate = (0..STORY_STEPS)
        .find_map(|_| {
            let gate = world.front().assignments.iter().find(|run| run.task != task).cloned();
            if gate.is_none() {
                let outcome = world.run(Until::Steps(2));
                assert!(matches!(outcome.verdict, Verdict::Passed | Verdict::Open { .. }), "{outcome:?}");
            }
            gate
        })
        .expect("review gate assigned");
    world.front().engine.answer(
        STORY_WORKER,
        gate.task,
        gate.attempt,
        0,
        jig_domain_tasks::End::Finished {
            result: jig_domain_tasks::TaskResult::Verdict {
                code: 1,
                fields: Box::new([]),
                words: b"approved".as_slice().into(),
            },
            cancel_delegates: false,
        },
        temper_engine_domain::Parts::none(),
    );
    world.front().engine.settle();
}

#[test]
fn a_new_chat_lands_its_delegated_change_and_closes_in_your_chats() {
    use temper_client_world::front::Joint;
    use temper_engine_domain_world::forge_stand::ForgeStand;
    let mut world = fixtures::world(fixtures::SEED);
    *world.front() = Joint::with_change_landing(fixtures::SEED, &fixtures::domain_limits());
    world.front().engine.forge = Some(ForgeStand::with_checks(fixtures::REPOSITORIES, &[b"build"]));
    fixtures::adopt_change_repository(world.front());
    world.front().engine.forge.as_mut().expect("story's forge").push_branch(7, CHANGE_BRANCH);
    let task = 1;
    let person = chat_from_new_chat(&mut world, task);
    let see = |find| Step::See { find, within: fixtures::WITHIN };
    let press = |role, name: &[u8]| Step::Press { find: Find::exactly_named(role, name) };
    let change = delegate_change(world.front(), task);
    let card = |find: Find| {
        find.within(Box::new([skein_fake_person::Target::ExactRole {
            role: Role::Region,
            name: format!("Task {change}").into_bytes().into(),
        }]))
    };
    world.person(person, vec![see(card(Find::exactly_named(Role::Status, b"Running")))]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let running = printed(&world, person);
    assert!(running.contains("Running") && running.contains(std::str::from_utf8(CHANGE_TITLE).expect("title")));
    approve_change(&mut world, task);
    world.person(
        person,
        vec![
            see(card(Find::exactly_named(Role::Status, b"Landed"))),
            see(card(Find::text(b"CI passed. Landed"))),
            see(card(Find::exactly_named(Role::Link, format!("Task {change}").as_bytes()))),
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let landed = printed(&world, person);
    assert!(landed.contains("CI passed. Landed"), "result's words: {landed}");
    assert!(landed.contains(&format!("Task {change} ↗")), "named task link: {landed}");
    world.person(
        person,
        vec![
            press(Role::Button, b"Close"),
            see(Find::exactly_named(Role::Dialog, b"Close this chat?")),
            press(Role::Button, b"Close chat"),
            Step::Pause { span: PAST_CLOSE_GRACE },
            see(Find::text(b"Closed at 00:00")),
            Step::Gone { find: Find::exactly_named(Role::TextArea, b"Message"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert!(printed(&world, person).contains("Closed"));
    world.person(
        person,
        vec![
            press(Role::Link, b"Chats"),
            see(Find::named(Role::Heading { level: 1 }, b"Talk to an agent")),
            press(Role::Button, b"Closed"),
            see(Find::exactly_named(Role::Link, b"T1")),
            see(Find::exactly_named(Role::Status, b"Closed").within(Box::new([
                skein_fake_person::Target::ExactRole { role: Role::List, name: b"Your chats".as_slice().into() },
            ]))),
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let closed = printed(&world, person);
    assert!(closed.contains("Closed") && closed.contains("Add CSV export"), "closed row: {closed}");
    assert!(world.front().engine.task(task).is_none(), "the closed chat left live work");
}
const CLOSE_REQUEST: skein_lib::Token = skein_lib::Token::new(1107);
const CLOSE_KEY: [u8; 16] = [37; 16];
const CLOSE_GRACE: skein_lib::Duration = skein_lib::Duration::from_secs(5);
const AFTER_CANCEL_ANSWER: skein_lib::Time = skein_lib::Time::from_nanos(6_100_000_000);
const AFTER_GRACE: skein_lib::Time = skein_lib::Time::from_nanos(6_000_000_000);
#[test]
fn a_prepared_close_expires_through_the_fronts_clock() {
    use jig_client_kit::harness::Front;
    use temper_engine_domain::{self as engine, PartyInput};
    let mut front = temper_client_world::front::Joint::new(fixtures::SEED, fixtures::domain_limits());
    let task = fixtures::chat(&mut front, b"A chat to close");
    front.engine.send(engine::Event::Party(PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(CLOSE_REQUEST),
        sign_in: front.engine.session(),
        key: CLOSE_KEY,
        ask: jig_domain_people::Ask::Cancel {
            project: 1,
            task,
            reason: b"Chat closed".as_slice().into(),
            grace: Some(CLOSE_GRACE),
        },
    }));
    front.engine.settle();
    let _events = front.poll(AFTER_GRACE, skein_lib::Wall::from_nanos(AFTER_GRACE.as_nanos()));
    let _events = front.poll(AFTER_CANCEL_ANSWER, skein_lib::Wall::from_nanos(AFTER_CANCEL_ANSWER.as_nanos()));
    assert!(
        front.engine.task(task).is_none(),
        "grace timer ends live work: task={:?}; deadline={:?}; host={:?}; deliveries={:?}",
        front.engine.task(task),
        front.engine.root.next_deadline(&front.engine.env),
        front.engine.host_requests,
        front.engine.delivered
    );
}

const PROPOSE_CALL: u64 = 1108;
const PROPOSE_COMPLETION: u32 = 1;
const PROPOSE_POSITION: u32 = 1;
const PROPOSE_WORKER: skein_lib::Token = skein_lib::Token::new(7);
const PROPOSE_BUDGET: u64 = 150;
const GOAL_PRIORITY: u32 = 1;
const BEFORE_INBOX_DECISION: skein_lib::Duration = skein_lib::Duration::from_millis(100);
const AFTER_INBOX_DECISION: skein_lib::Duration = skein_lib::Duration::from_millis(500);
fn proposed_goal(front: &mut temper_client_world::front::Joint) -> u64 {
    use temper_engine_domain as engine;
    *front = temper_client_world::front::Joint::with_goal_proposals(fixtures::SEED, fixtures::domain_limits());
    let task = fixtures::chat(front, b"Propose a goal");
    let run = front.engine.last_assignment();
    let mut goal = temper_engine_domain_world::fixtures::report_delegate(b"A lasting goal", Box::new([]));
    goal.authority.budget.spend = PROPOSE_BUDGET;
    goal.tracked = Some(GOAL_PRIORITY);
    front.engine.call(
        PROPOSE_WORKER,
        task,
        run.attempt,
        PROPOSE_CALL,
        engine::Call {
            completion: PROPOSE_COMPLETION,
            position: PROPOSE_POSITION,
            tool: engine::Tool::Propose {
                action: engine::ProposedAction::Batch(Box::new([goal])),
                reason: b"A lasting goal".as_slice().into(),
                as_holder: true,
            },
        },
    );
    front.engine.settle();
    assert!(
        matches!(front.engine.answered(PROPOSE_CALL), Some(engine::CallAnswer::Proposed { .. })),
        "run proposed a goal: {:?}",
        front.engine.answered(PROPOSE_CALL)
    );
    task
}
#[test]
fn the_goal_proposal_fixture_waits_for_the_chats_person() {
    let mut front = temper_client_world::front::Joint::new(fixtures::SEED, fixtures::domain_limits());
    let task = proposed_goal(&mut front);
    assert!(
        matches!(&front.engine.task(task).expect("chat").proposal,
        Some(proposal) if matches!(proposal.state, jig_domain_tasks::ProposalState::Pending {
            holder: jig_domain_tasks::ProposalHolder::Person(fixtures::PARTY), ..
        })),
        "pending: {:?}",
        front.engine.task(task).expect("chat").proposal
    );
}
#[test]
fn a_goal_proposed_in_a_chat_accepted_in_the_inbox_leaves_the_chat() {
    let mut world = fixtures::world(fixtures::SEED);
    let task = proposed_goal(world.front());
    let chat = world.client(fixtures::PARTY, fixtures::chat_page(task));
    let inbox = world.client(fixtures::PARTY, fixtures::inbox());
    world.person(
        chat,
        vec![
            Step::See { find: Find::named(Role::Button, b"Accept"), within: fixtures::WITHIN },
            Step::Pause { span: AFTER_INBOX_DECISION },
        ],
    );
    world.person(
        inbox,
        vec![
            Step::Pause { span: BEFORE_INBOX_DECISION },
            Step::See { find: Find::named(Role::Button, b"Accept"), within: fixtures::WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Accept") },
            Step::See { find: Find::named(Role::Dialog, b"Accept this proposal?"), within: fixtures::WITHIN },
            Step::Press {
                find: Find::named(Role::Button, b"Accept")
                    .within(Box::new([skein_fake_person::Target::Role { role: Role::Dialog, name: Box::new([]) }])),
            },
            Step::See { find: Find::text(b"Accepted"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert!(
        world
            .root(chat)
            .person()
            .pages()
            .chat()
            .expect("chat")
            .cards
            .iter()
            .all(|card| matches!(&card.page, Some(jig_client::contract::ReadValue::Ended(_)))),
        "only the ended read remains in the live chat"
    );
    assert!(
        world.front().engine.store.rows.values().any(|record| matches!(record,
            temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(row)))
            if row.requester == jig_domain_tasks::Party::Person(fixtures::PARTY)
                && row.tracked == Some(GOAL_PRIORITY) && row.spec.words.as_ref() == b"A lasting goal"
        )),
        "engine holds the accepted goal"
    );
}

const INBOX_CLIENT: jig_client_kit::harness::ClientId = jig_client_kit::harness::ClientId(1);
#[test]
fn the_proposal_fixture_admits_both_chat_and_inbox_clients() {
    use jig_client_kit::harness::Front;
    let mut front = temper_client_world::front::Joint::new(fixtures::SEED, fixtures::domain_limits());
    proposed_goal(&mut front);
    for client in [DIAGNOSTIC_CLIENT, INBOX_CLIENT] {
        front.bind(client, fixtures::PARTY);
        front.poll(DIAGNOSTIC_NOW, DIAGNOSTIC_WALL);
    }
}

type Probe = (temper_client_world::app::Root, skein_lib::Queue<temper_client_domain::Request>);
const PROBE_ROUNDS: usize = 24;
fn probe(seed: u64, address: temper_client_domain::Address) -> Probe {
    use jig_client_kit::harness::App;
    use temper_client_world::app::Temper;
    let limits = fixtures::domain_limits();
    let mut root = temper_client_world::app::Root::Person(Box::new(temper_client_domain::unlinked(&limits, seed)));
    let mut out = skein_lib::Queue::uncapped();
    Temper::step(
        &mut root,
        &skein_lib::Env { now: DIAGNOSTIC_NOW, wall: DIAGNOSTIC_WALL, limits },
        Temper::start(address, Vec::new(), START_OFFSET),
        &mut out,
    );
    (root, out)
}
fn pump(front: &mut temper_client_world::front::Joint, probes: &mut [Probe]) {
    use jig_client_kit::harness::{App, ClientId, Duty, Front};
    use temper_client_world::app::Temper;
    let env = skein_lib::Env { now: DIAGNOSTIC_NOW, wall: DIAGNOSTIC_WALL, limits: fixtures::domain_limits() };
    for _ in 0..PROBE_ROUNDS {
        for (index, (root, out)) in probes.iter_mut().enumerate() {
            Temper::take(root, &env, out);
            while let Some(request) = out.pop() {
                if let Duty::Wire(request) = Temper::duty(request) {
                    front.wire(ClientId(index), env.now, env.wall, request);
                }
            }
        }
        for (who, event) in front.poll(env.now, env.wall) {
            front.received(who, &event);
            let (root, out) = &mut probes[who.0];
            Temper::step(root, &env, event, out);
        }
    }
}
#[test]
fn accepting_the_goal_with_chat_and_inbox_watches_settles_the_engine() {
    use jig_client_kit::harness::{App, Front};
    use temper_client_domain::{Action, Event, pages::Act};
    let mut front = temper_client_world::front::Joint::new(fixtures::SEED, fixtures::domain_limits());
    let task = proposed_goal(&mut front);
    let mut probes =
        vec![probe(fixtures::SEED, fixtures::chat_page(task)), probe(fixtures::SEED + 1, fixtures::inbox())]
            .into_boxed_slice();
    for client in [DIAGNOSTIC_CLIENT, INBOX_CLIENT] {
        front.bind(client, fixtures::PARTY);
    }
    pump(&mut front, &mut probes);
    let object = probes[1]
        .0
        .person()
        .pages()
        .inbox()
        .expect("inbox")
        .cards
        .as_slice()
        .iter()
        .find(|card| matches!(card.object, jig_client::contract::ObjectKey::Proposal(_)))
        .expect("proposal")
        .object;
    let env = skein_lib::Env { now: DIAGNOSTIC_NOW, wall: DIAGNOSTIC_WALL, limits: fixtures::domain_limits() };
    for act in [Act::Decide { object, revision: None, act: jig_client::contract::ActKind::Accept }, Act::Confirm] {
        temper_client_world::app::Temper::step(
            &mut probes[1].0,
            &env,
            Event::Act { action: Action::Pages(act) }.into(),
            &mut probes[1].1,
        );
        pump(&mut front, &mut probes);
    }
    assert!(
        probes[0]
            .0
            .person()
            .pages()
            .chat()
            .expect("chat")
            .cards
            .iter()
            .all(|card| matches!(&card.page, Some(jig_client::contract::ReadValue::Ended(_))))
    );
}

#[test]
fn opening_a_chat_clears_its_reply_from_the_engines_inbox() {
    let mut world = fixtures::world(fixtures::SEED);
    let task = fixtures::replied_chat(world.front(), b"Hello");
    let person = world.client(fixtures::PARTY, fixtures::inbox());
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Heading { level: 3 }, b"replied"), within: fixtures::WITHIN },
            Step::Go { address: fixtures::chat_page(task) },
            Step::See { find: Find::named(Role::TextArea, b"Message"), within: fixtures::WITHIN },
            Step::Press { find: Find::exactly_named(Role::Link, b"Needs you") },
            Step::See { find: Find::named(Role::Heading { level: 1 }, b"Inbox"), within: fixtures::WITHIN },
            Step::Gone { find: Find::named(Role::Heading { level: 3 }, b"replied"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

#[test]
fn a_refused_close_stays_in_its_sheet_until_kept() {
    let mut world = fixtures::world(fixtures::SEED);
    let task = fixtures::replied_chat(world.front(), b"Finish while I consider closing");
    let person = world.client(fixtures::PARTY, fixtures::chat_page(task));
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Button, b"Close"), within: fixtures::WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Close") },
            Step::See { find: Find::named(Role::Dialog, b"Close this chat?"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    fixtures::finish_replied_chat(world.front(), task);
    world.person(
        person,
        vec![
            Step::Press { find: Find::named(Role::Button, b"Close chat") },
            Step::See {
                find: Find::text(b"No visible object with that name").within(Box::new([
                    skein_fake_person::Target::Role { role: Role::Dialog, name: b"Close this chat?".as_slice().into() },
                ])),
                within: fixtures::WITHIN,
            },
            Step::Press { find: Find::named(Role::Button, b"Keep it") },
            Step::Gone { find: Find::named(Role::Dialog, b"Close this chat?"), within: fixtures::WITHIN },
            Step::See { find: Find::text(b"The chat is complete"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

#[test]
fn a_chat_pauses_resumes_and_a_message_resumes_it() {
    let mut world = fixtures::world(fixtures::SEED);
    let task = fixtures::chat(world.front(), b"Hello");
    let person = world.client(fixtures::PARTY, fixtures::chat_page(task));
    let see = |find| Step::See { find, within: fixtures::WITHIN };
    let gone = |find| Step::Gone { find, within: fixtures::WITHIN };
    let button = |name: &[u8]| Find::exactly_named(Role::Button, name);
    let paused = || Find::exactly_named(Role::Status, b"Paused");
    world.person(
        person,
        vec![
            see(button(b"Pause")),
            Step::Press { find: button(b"Pause") },
            see(paused()),
            see(button(b"Resume")),
            gone(button(b"Pause")),
            gone(Find::text(b"Applied")),
            see(Find::text(b"Sending resumes this chat.")),
            Step::Press { find: button(b"Resume") },
            gone(paused()),
            gone(Find::text(b"Applied")),
            see(button(b"Pause")),
            Step::Press { find: button(b"Pause") },
            see(paused()),
        ],
    );
    let result = world.run(Until::Done);
    assert_eq!(result.verdict, Verdict::Passed, "{result:?}");
    // The message starts a new attempt; Pause has cancelled both earlier runs.
    world.front().reply_in_pieces(&[b"Working"]);
    world.person(
        person,
        vec![
            Step::Type { find: Find::named(Role::TextArea, b"Message"), words: b"And then?".as_slice().into() },
            Step::Press { find: button(b"Send") },
            gone(paused()),
            see(Find::text(b"And then?")),
        ],
    );
    let result = world.run(Until::Done);
    assert_eq!(result.verdict, Verdict::Passed, "{result:?}");
    let chat = world.front().engine.task(task).expect("live chat");
    assert!(!matches!(chat.phase, jig_domain_tasks::Phase::Held { .. }));
    assert_eq!(chat.inbox.iter().filter(|message| message.words.as_ref() == b"And then?").count(), 1);
    assert!(world.front().release_reply());
    world.person(person, vec![see(Find::text(b"Answered by the run"))]);
    let result = world.run(Until::Done);
    assert_eq!(result.verdict, Verdict::Passed, "{result:?}");
}
