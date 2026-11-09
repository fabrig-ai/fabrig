//! Private turn and live call parts shown to their requester alone (`jig/docs/design/domain/engine.md`, 10.1).
use crate::settings_support as support;
use jig_core_world::{
    client_peer::ClientPeer,
    fixtures,
    world::{Boot, Setup, World},
};
use jig_domain_core::{self as core, documents, reads};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Token};

const PRIVATE: &[u8] = b"private";
const PUBLIC: &[u8] = b"ordinary";
const INPUT: &[u8] = b"{\"line\":\"kept\"}";
const TRANSCRIPT: &[u8] = b"the whole committed turn";

fn chat() -> (World, u64, u64, u64) {
    let (mut config, mut limits) = support::run_configuration();
    limits.core.fleet.slots = 2;
    jig_core_world::peers::journal(&mut limits);
    config.core.settings.private_tools = Box::new([PRIVATE.into()]);
    let mut world = World::new(Setup { configuration: Some(config), limits, boot: Boot::Empty, ..Setup::default() });
    world.client_peer = Some(ClientPeer::default());
    let p = support::sign_in(&mut world);
    let (_, q) = support::party(&mut world, b"reader-q", Some((fixtures::PROJECT, people::Role::Member)));
    world.send(root::Event::Core(core::Event::Period {
        project: fixtures::PROJECT,
        period: 1,
        budget: 100,
        next: None,
    }));
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: jig_domain_link::Hello {
            jobs: Box::default(),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: Duration::from_millis(1),
            slots: 2,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
    let mut chat = fixtures::chat(&world, 1, b"Explain this work".as_slice().into());
    let people::Ask::Create { batch, .. } = &mut chat else { panic!("chat creation") };
    batch[0].authority.tools |= core::ToolFamilies::JIG.ask.0;
    let reply = support::ask(&mut world, p, 101, chat);
    let people::Reply::Outcome(people::Outcome::Created { first, .. }) = reply else { panic!("chat: {reply:?}") };
    assert!(fixtures::live_row(&world, first).flags.conversation);
    (world, p, q, first)
}

fn delegate(world: &mut World) -> (u64, root::Assignment) {
    let mut authority = world.assigned.as_ref().expect("parent run").run.authority.clone();
    authority.tools = tasks::Tools(0);
    authority.grants = Box::new([]);
    authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    authority.budget.spend = 20;
    let key = world.call_key(1);
    let parent = world.assigned.take().expect("parent run");
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(501)),
        key,
        batch: Box::new([core::Delegate {
            deadline: tasks::Deadline::Named,
            label: 0,
            flags: tasks::Flags::PLAIN,
            tracked: None,
            budget: people::Budget::Named,
            executor: tasks::Executor::Agent { charter: 1 },
            spec: tasks::Spec {
                job: None,
                asks: false,
                words: b"Delegate's work".as_slice().into(),
                parameters: Box::new([]),
                inputs: Box::new([]),
            },
            contract: tasks::Contract::Report { words: 128 },
            authority,
            llm: None,
            symbolic_grants: Box::new([]),
            dependencies: Box::new([]),
            wake: Some(tasks::WakePolicy::DEFAULT),
        }]),
        stubs: Box::new([]),
    }));
    let core::CallPart::Delegated(children) = &world.answers.last().expect("delegate answer").2 else {
        panic!("delegate refused")
    };
    (children[0].task, parent)
}

fn read(world: &mut World, signed: Option<u64>, read: reads::Read) -> reads::Answer {
    if let Some(signed) = signed {
        support::read(world, signed, read)
    } else {
        world.person_read(read);
        world.person_answers.pop().expect("run reads as its person").2
    }
}

fn assert_turn(answer: reads::Answer, withheld: bool, task: u64) {
    let reads::Answer::Ended(ended) = answer else { panic!("turn answer: {answer:?}") };
    let (record, marked) = match ended.value {
        reads::EndedValue::Turn(record) => (record, false),
        reads::EndedValue::WithheldTurn(record) => (record, true),
        value => panic!("turn carrier: {value:?}"),
    };
    assert_eq!(marked, withheld, "task {task}'s turn mark");
    assert_eq!(record.task, task);
    assert_eq!(record.transcript.as_ref(), TRANSCRIPT, "the core carries the whole archive");
}

fn assert_page(answer: reads::Answer, withheld: bool, task: u64) {
    let reads::Answer::Page(page) = answer else { panic!("activity page: {answer:?}") };
    let mut turns = 0;
    for row in &page.rows {
        if let reads::PageRow::Activity { turn: Some(turn), .. } = row {
            let (record, marked) = match turn {
                reads::PageTurn::Carried(record) => (record, false),
                reads::PageTurn::Withheld(record) => (record, true),
                value => panic!("carried page turn: {value:?}"),
            };
            assert_eq!(marked, withheld, "task {task}'s page turn mark");
            assert_eq!(record.task, task);
            assert_eq!(record.transcript.as_ref(), TRANSCRIPT);
            turns += 1;
        }
    }
    assert_eq!(turns, 1, "the page includes the committed turn");
}

#[test]
fn a_turn_is_withheld_from_all_but_its_person() {
    let (mut world, p, q, parent) = chat();
    world.turn(1, 1, None, TRANSCRIPT);
    let (child, parent_run) = delegate(&mut world);
    assert_eq!(fixtures::live_row(&world, child).requester, tasks::Party::Task(parent));
    let child_attempt = world.attempt();
    world.turn(1, 1, None, TRANSCRIPT);
    assert_eq!(world.assigned.as_ref().expect("child run").task, child);
    let parent_attempt = parent_run.attempt;
    world.assigned = Some(parent_run);
    for (task, attempt, reader, withheld) in [
        (parent, parent_attempt, Some(p), false),
        (parent, parent_attempt, Some(q), true),
        (parent, parent_attempt, None, true),
        (child, child_attempt, Some(p), false),
        (child, child_attempt, Some(q), false),
        (child, child_attempt, None, true),
    ] {
        assert_turn(read(&mut world, reader, reads::Read::Turn { task, attempt, turn: 1 }), withheld, task);
        assert_page(read(&mut world, reader, reads::Read::Activity { task, before: None }), withheld, task);
        assert_page(
            read(&mut world, reader, reads::Read::ActivityBeside { task, position: 1, side: reads::Side::Later }),
            withheld,
            task,
        );
    }
    world.broken();
}

fn watch(world: &mut World, signed: u64, task: u64, watcher: u64) -> documents::Run {
    world.send(root::Event::Core(core::Event::WatchRun {
        watcher: Token::new(watcher),
        sign_in: signed,
        task,
        turn: None,
        offset: 0,
    }));
    let (_, root::Delivery::Core(core::Held::Delivery { document, .. })) =
        world.client_peer.as_mut().expect("client peer").delivered.pop().expect("opening snapshot")
    else {
        panic!("watch snapshot")
    };
    let documents::Document::Run(run) = *document else { panic!("run snapshot") };
    world.send(root::Event::Core(core::Event::Delivered { watcher: Token::new(watcher) }));
    let now = std::mem::take(&mut world.client_peer.as_mut().expect("client peer").now);
    for output in now {
        let core::Now::RunText { watcher: output, .. } = output else { panic!("opening stream") };
        assert_eq!(output, Token::new(watcher));
        world.send(root::Event::Core(core::Event::StreamDelivered { watcher: output }));
    }
    run
}

fn assert_call(call: &views::Call, tool: &[u8], input: &[u8]) {
    assert_eq!(call.tool.as_ref(), tool);
    assert_eq!(call.input.as_ref(), input);
    assert!(!call.cut);
    assert!(call.began.is_some());
}

fn assert_writing(call: &views::WritingCall, tool: &[u8], input: &[u8]) {
    assert_eq!(call.tool.as_ref(), tool);
    assert_eq!(call.input.as_ref(), input);
    assert_eq!((call.from, call.through, call.cut), (0, u32::try_from(INPUT.len()).expect("short input"), false));
}

fn streams(world: &mut World, tool: &[u8], writing: bool) {
    let output = std::mem::take(&mut world.client_peer.as_mut().expect("client peer").now);
    let mut followers = Vec::new();
    for now in output {
        let core::Now::RunText { watcher, call, writing: partial, .. } = now else { panic!("live call") };
        let input = if watcher.raw() == 10 || tool == PUBLIC { INPUT } else { b"" };
        if writing {
            assert!(call.is_none());
            assert_writing(partial.as_ref().expect("writing call"), tool, input);
        } else {
            assert!(partial.is_none());
            assert_call(call.as_ref().expect("made call"), tool, input);
        }
        followers.push(watcher.raw());
        world.send(root::Event::Core(core::Event::StreamDelivered { watcher }));
    }
    followers.sort_unstable();
    assert_eq!(followers, [10, 11]);
}

#[test]
fn a_private_live_calls_input_reaches_its_requester_alone() {
    let (mut world, p, q, task) = chat();
    let attempt = world.attempt();
    watch(&mut world, p, task, 10);
    watch(&mut world, q, task, 11);
    for (sequence, tool) in [(1, PRIVATE), (2, PUBLIC)] {
        world.send(root::Event::Core(core::Event::RunInput {
            run: Token::new(task),
            attempt: Token::new(attempt),
            turn: 1,
            retry: 0,
            offset: 0,
            id: Box::new([sequence]),
            tool: tool.into(),
            input: INPUT.into(),
        }));
        streams(&mut world, tool, true);
        for (signed, watcher, input) in [(p, 20, INPUT), (q, 21, if tool == PRIVATE { b"" } else { INPUT })] {
            let run = watch(&mut world, signed, task, watcher);
            assert!(run.call.is_none());
            assert_writing(
                run.writing.expect("writing snapshot").writing.as_ref().expect("partial input"),
                tool,
                input,
            );
            world.send(root::Event::Core(core::Event::Unwatch { watcher: Token::new(watcher) }));
        }
        world.send(root::Event::Core(core::Event::RunText {
            run: Token::new(task),
            attempt: Token::new(attempt),
            turn: 1,
            retry: 0,
            offset: 0,
            text: Box::new([]),
            call: Some(views::Call {
                identity: views::CallIdentity {
                    owner: Token::new(1),
                    sequence: u32::from(sequence),
                    retry: 0,
                    id: Box::new([sequence]),
                },
                tool: tool.into(),
                input: INPUT.into(),
                cut: false,
                began: None,
            }),
        }));
        streams(&mut world, tool, false);
        for (signed, watcher, input) in [(p, 20, INPUT), (q, 21, if tool == PRIVATE { b"" } else { INPUT })] {
            let run = watch(&mut world, signed, task, watcher);
            assert_call(run.call.as_ref().expect("current call snapshot"), tool, input);
            assert_call(run.writing.expect("writing snapshot").call.as_ref().expect("made call"), tool, input);
            world.send(root::Event::Core(core::Event::Unwatch { watcher: Token::new(watcher) }));
        }
    }
    world.broken();
}
