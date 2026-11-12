//! Mid-work offers through the real run/session seam and the shared scripted provider.
use skein_fake_checkout::Exit;
use skein_fake_llm_domain::api::{Line, Part, Script, Turn};
use skein_lib::{Duration, Time, Token};
use skein_world::domain::Span;
use smith_agent_world::{Boundary, Job, Settings, World, scripted_charter};
use smith_domain::{Fact, run, tools};
use smith_tools_world::fixture::{Fixture, program};

fn call(name: &[u8], arguments: &[u8]) -> Turn {
    smith_agent_world::fixture::calls(Box::new([Line::Call { name: name.into(), arguments: arguments.into() }]), 1)
}

fn settings() -> Settings {
    let mut settings = Settings::calm(926);
    settings.job = Job::Reporting;
    settings.network = Span::millis(0, 0);
    settings.provider.latency_min = Duration::from_millis(100);
    settings.provider.latency_max = Duration::from_millis(100);
    settings
}

fn working(settings: &Settings) -> World {
    scripted(
        settings,
        Box::new([
            call(b"shell", br#"{"command":"cargo build"}"#),
            call(b"shell", br#"{"command":"cargo build"}"#),
            call(b"finish", br#"{"report":"Work settled.","source":"workspace"}"#),
        ]),
        Vec::new(),
    )
}

fn scripted(settings: &Settings, turns: Box<[Turn]>, children: Vec<Script>) -> World {
    scripted_in(settings, turns, children, Fixture::new())
}

fn scripted_in(settings: &Settings, turns: Box<[Turn]>, children: Vec<Script>, fixture: Fixture) -> World {
    scripted_mode(settings, turns, children, fixture, run::Mode::Finite)
}

fn scripted_mode(
    settings: &Settings,
    turns: Box<[Turn]>,
    mut children: Vec<Script>,
    mut fixture: Fixture,
    mode: run::Mode,
) -> World {
    fixture.checkout.program(b"cargo build", program(200, b"Finished\n", Exit::Code(0), &[]));
    let workspace = run::Workspace {
        directories: Box::new([run::Directory {
            host: b"/work/temper".as_slice().into(),
            name: b"work".as_slice().into(),
            root: fixture.repos[0].root,
            writable: true,
            git: true,
            conflicts: Box::default(),
        }]),
    };
    let mut charter = scripted_charter(settings);
    charter.mode = mode;
    children.push(Script { cue: charter.brief.sections[0].text.clone(), turns });
    World::with_workspace_scripts_charter(*settings, None, Some(workspace), fixture.checkout, children.into(), charter)
}

fn message(world: &mut World, millis: u64, name: u64, text: &[u8]) {
    world.message_at(
        Time::ZERO.saturating_add(Duration::from_millis(millis)),
        Token::new(name),
        b"delegate".as_slice().into(),
        text.into(),
    );
}

#[test]
fn mid_work_host_messages_reach_main_and_not_its_working_child() {
    let turns = Box::new([
        call(b"sub_agent", br#"{"brief":"@mid-child inspect","tools":["inspect"]}"#),
        call(b"finish", br#"{"report":"Child settled.","source":"workspace"}"#),
    ]);
    let children = vec![Script {
        cue: b"@mid-child".as_slice().into(),
        turns: Box::new([
            call(b"read", br#"{"path":"src/lib.rs"}"#),
            smith_agent_world::fixture::says(Box::new([Line::Text { text: b"child done".as_slice().into() }]), 1),
        ]),
    }];
    let mut world = scripted(&settings(), turns, children);
    message(&mut world, 150, 7, b"main only");
    world.run(10_000);
    assert_eq!(world.prompts().len(), 4);
    for child in &world.prompts()[1..3] {
        assert!(!child.messages.iter().flat_map(|message| &message.parts).any(|part| matches!(part,
            Part::Text { text } if text.as_ref() == b"delegate: main only")));
    }
    assert!(matches!(tail(&world, 3).as_slice(),
        [Part::ToolOutput { .. }, Part::Text { text }] if text.as_ref() == b"delegate: main only"));
    assert_eq!(fences(&world), [None, Some(Token::new(7))]);
}

fn tail(world: &World, request: usize) -> Vec<&Part> {
    world.prompts()[request].messages.iter().rev().take(2).rev().flat_map(|message| &message.parts).collect()
}

fn fences(world: &World) -> Vec<Option<Token>> {
    world.turn_metadata().iter().map(|(_, read, _)| *read).collect()
}

#[test]
fn queued_work_is_offered_after_the_calls_results_and_read_by_the_next_told_turn() {
    let mut world = working(&settings());
    message(&mut world, 150, 7, b"change ended");
    world.run(10_000);
    let started = world
        .boundaries()
        .iter()
        .find_map(|(at, boundary)| matches!(boundary, Boundary::Io { op: tools::Op::Spawn { .. } }).then_some(*at))
        .expect("ordinary shell call ran");
    let admitted = world
        .facts()
        .iter()
        .find_map(|fact| match fact {
            Fact::Run {
                fact: run::facts::Fact { at, kind: run::facts::FactKind::MessageReceived { name, .. }, .. },
                ..
            } if *name == Token::new(7) => Some(*at),
            _ => None,
        })
        .expect("message admitted");
    assert!(started < admitted && admitted < world.completions()[1].started, "message crosses the running call");
    assert_eq!(world.prompts().len(), 3, "no wait, yield or extra completion");
    assert!(
        matches!(tail(&world, 1).as_slice(),
        [Part::ToolOutput { output, is_error: false, .. }, Part::Text { text }]
        if output.starts_with(b"Owned { outcome: Exited") && text.as_ref() == b"delegate: change ended"),
        "{:?}",
        tail(&world, 1)
    );
    assert_eq!(fences(&world), [None, Some(Token::new(7)), Some(Token::new(7))]);
    assert!(world.waiting().is_empty());
    assert!(matches!(world.answer(), run::Answer::Accepted { turns: 3, .. }));
    assert_eq!(terminals(&world), [(Token::new(7), "read")]);
}

#[test]
fn one_mid_work_offer_keeps_arrival_order_and_leaves_its_remainder_for_the_next_completion() {
    for byte_limited in [false, true] {
        let mut settings = settings();
        if byte_limited {
            settings.limits.run.offer_bytes = 33;
            settings.limits.run.message_bytes = 33;
        } else {
            settings.limits.run.offer_messages = 2;
        }
        let mut world = working(&settings);
        message(&mut world, 150, 99, b"first");
        message(&mut world, 160, 0, b"second");
        message(&mut world, 170, 7, b"third");
        world.run(10_000);
        assert_eq!(world.prompts().len(), 3);
        assert!(matches!(tail(&world, 1).as_slice(),
            [Part::ToolOutput { .. }, Part::Text { text }]
            if text.as_ref() == b"delegate: first\n\ndelegate: second"));
        assert!(matches!(tail(&world, 2).as_slice(),
            [Part::ToolOutput { .. }, Part::Text { text }]
            if text.as_ref() == b"delegate: third"));
        assert_eq!(fences(&world), [None, Some(Token::new(0)), Some(Token::new(7))]);
    }
}

#[test]
fn a_message_crossing_an_in_flight_completion_changes_only_the_following_request() {
    let mut world = working(&settings());
    message(&mut world, 350, 7, b"after send");
    world.run(10_000);
    assert_eq!(world.prompts().len(), 3);
    assert!(matches!(
        world.prompts()[1].messages.last().expect("settled results").parts.as_ref(),
        [Part::ToolOutput { .. }]
    ));
    assert!(matches!(tail(&world, 2).as_slice(),
        [Part::ToolOutput { .. }, Part::Text { text }] if text.as_ref() == b"delegate: after send"));
    assert_eq!(fences(&world), [None, None, Some(Token::new(7))]);
}

#[test]
fn an_offer_to_a_completion_never_told_ends_unread_once() {
    let mut settings = settings();
    settings.cancel_at = Some(Duration::from_millis(350));
    let mut world = working(&settings);
    message(&mut world, 150, 7, b"change ended");
    world.run(10_000);
    assert_eq!(world.prompts().len(), 2);
    assert!(matches!(tail(&world, 1).as_slice(),
        [Part::ToolOutput { .. }, Part::Text { text }] if text.as_ref() == b"delegate: change ended"));
    assert_eq!(terminals(&world), [(Token::new(7), "unread")]);
    assert_eq!(fences(&world), [None]);
}

fn terminals(world: &World) -> Vec<(Token, &'static str)> {
    world
        .facts()
        .iter()
        .filter_map(|fact| match fact {
            Fact::Run {
                fact: run::facts::Fact { kind: run::facts::FactKind::MessageRead { name, .. }, .. }, ..
            } => Some((*name, "read")),
            Fact::Run {
                fact: run::facts::Fact { kind: run::facts::FactKind::MessageUnread { name, .. }, .. }, ..
            } => Some((*name, "unread")),
            _ => None,
        })
        .collect()
}

#[test]
fn drain_keeps_the_completion_and_tools_whole_without_reading_a_later_message() {
    for mode in [run::Mode::Finite, run::Mode::Open] {
        for at in [50, 150] {
            let turns = Box::new([
                smith_agent_world::fixture::calls(
                    Box::new([
                        Line::Call {
                            name: b"read".as_slice().into(),
                            arguments: br#"{"path":"src/lib.rs"}"#.as_slice().into(),
                        },
                        Line::Call {
                            name: b"shell".as_slice().into(),
                            arguments: br#"{"command":"cargo build"}"#.as_slice().into(),
                        },
                    ]),
                    1,
                ),
                call(b"finish", br#"{"report":"Not reached.","source":"workspace"}"#),
            ]);
            let mut world = scripted_mode(&settings(), turns, Vec::new(), Fixture::new(), mode);
            world.drain_at(Time::from_nanos(at * 1_000_000));
            message(&mut world, at + 10, 7, b"next activation");
            world.run(10_000);
            assert_eq!(world.prompts().len(), 1, "no next main completion");
            assert!(matches!(
                world.completions()[0].terminal,
                Some((_, smith_agent_world::CompletionTerminal::Completed(_)))
            ));
            assert!(matches!(
                world.answer(),
                run::Answer::Failed { failure: run::RunFailure::Cancelled, turns: 1, .. }
            ));
            assert_eq!(fences(&world), [None]);
            assert_eq!(terminals(&world), [(Token::new(7), "unread")]);
            let results: Vec<_> = world.turns()[0]
                .messages
                .iter()
                .flat_map(|message| &message.content)
                .filter_map(|block| {
                    if let smith_domain::session::llm::Block::ToolResult { result, .. } = block {
                        Some(result)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(results.len(), 2, "both calls have their results in the turn");
            assert!(
                results.iter().any(|result| matches!(
                    result,
                    smith_domain::session::llm::Returned::Owned { outcome: tools::Outcome::Exited { .. } }
                )),
                "the shell result is in the told turn"
            );
        }
    }
}

#[test]
fn drain_waits_for_the_sub_agents_whole_conversation() {
    let turns = Box::new([
        call(b"sub_agent", br#"{"brief":"@drain-child inspect","tools":["inspect"]}"#),
        call(b"finish", br#"{"report":"Not reached.","source":"workspace"}"#),
    ]);
    let children = vec![Script {
        cue: b"@drain-child".as_slice().into(),
        turns: Box::new([
            call(b"read", br#"{"path":"src/lib.rs"}"#),
            smith_agent_world::fixture::says(Box::new([Line::Text { text: b"child done".as_slice().into() }]), 1),
        ]),
    }];
    let mut world = scripted(&settings(), turns, children);
    world.drain_at(Time::from_nanos(150_000_000));
    world.run(10_000);
    assert_eq!(world.prompts().len(), 3, "main and both child completions, no next main");
    assert!(world.completions().iter().all(|completion| matches!(
        completion.terminal,
        Some((_, smith_agent_world::CompletionTerminal::Completed(_)))
    )));
    assert!(matches!(world.answer(), run::Answer::Failed { failure: run::RunFailure::Cancelled, turns: 1, .. }));
    assert!(world.turns()[0].messages.iter().flat_map(|message| &message.content).any(
        |block| matches!(block, smith_domain::session::llm::Block::ToolResult {
            result: smith_domain::session::llm::Returned::Text { text, error: false, .. }, ..
        } if skein_lib::bytes::find(text, b"child done").is_some())
    ));
}

#[test]
fn drain_preserves_an_accepted_finish_and_its_checks() {
    for at in [50, 150] {
        let mut settings = settings();
        settings.job = Job::Coding;
        settings.check = Span::millis(200, 200);
        let mut fixture = Fixture::new();
        fixture.checkout.write(b"work/temper/src/lib.rs", b"pub fn answer() -> u32 { 43 }\n");
        fixture.checkout.write(b"work/temper/.temper/pre-pr", b"#!checks\n");
        let mut world = scripted_in(
            &settings,
            Box::new([call(b"finish", br#"{"title":"Whole checked turn","body":"Work settled."}"#)]),
            Vec::new(),
            fixture,
        );
        world.drain_at(Time::from_nanos(at * 1_000_000));
        world.run(10_000);
        assert_eq!(world.prompts().len(), 1);
        assert_eq!(world.checked(), [true]);
        assert!(matches!(world.answer(), run::Answer::Accepted { turns: 1, .. }));
    }
}

#[test]
fn cancel_after_drain_cuts_the_completion() {
    let mut settings = settings();
    settings.cancel_at = Some(Duration::from_millis(75));
    let mut world = working(&settings);
    world.drain_at(Time::from_nanos(50_000_000));
    world.run(10_000);
    assert!(matches!(world.completions()[0].terminal, Some((_, smith_agent_world::CompletionTerminal::Cancelled))));
    assert!(world.turns().is_empty());
    assert!(matches!(world.answer(), run::Answer::Failed { failure: run::RunFailure::Cancelled, turns: 0, .. }));
}

#[test]
fn drain_while_awaiting_waiting_or_listening_starts_no_completion() {
    for phase in ["awaiting", "waiting", "listening", "wait-call"] {
        let settings = settings();
        let mut charter = scripted_charter(&settings);
        let cue = charter.brief.sections[0].text.clone();
        let turn = if phase == "listening" {
            charter.mode = run::Mode::Open;
            smith_agent_world::fixture::says(Box::new([Line::Text { text: b"reply".as_slice().into() }]), 1)
        } else {
            call(b"wait", b"{}")
        };
        if phase == "awaiting" {
            charter.brief.sections = Box::default();
        }
        let mut world = World::with_workspace_scripts_charter(
            settings,
            None,
            None,
            skein_fake_checkout::Checkout::new(),
            Box::new([Script { cue, turns: Box::new([turn]) }]),
            charter,
        );
        world.drain_at(Time::from_nanos(if phase == "wait-call" { 50_000_000 } else { 150_000_000 }));
        world.run(10_000);
        assert_eq!(world.prompts().len(), usize::from(phase != "awaiting"));
        assert!(world.answered_at() < Time::from_nanos(200_000_000));
        assert!(matches!(world.answer(), run::Answer::Failed { failure: run::RunFailure::Cancelled, .. }));
    }
}

#[test]
fn drain_preserves_a_turn_that_parks_without_waiting() {
    for open in [false, true] {
        let mut settings = settings();
        settings.waiting = Duration::ZERO;
        let mut charter = scripted_charter(&settings);
        let cue = charter.brief.sections[0].text.clone();
        let turn = if open {
            charter.mode = run::Mode::Open;
            smith_agent_world::fixture::says(Box::new([Line::Text { text: b"reply".as_slice().into() }]), 1)
        } else {
            call(b"wait", b"{}")
        };
        let mut world = World::with_workspace_scripts_charter(
            settings,
            None,
            None,
            skein_fake_checkout::Checkout::new(),
            Box::new([Script { cue, turns: Box::new([turn]) }]),
            charter,
        );
        world.drain_at(Time::from_nanos(50_000_000));
        world.run(10_000);
        assert_eq!(world.prompts().len(), 1);
        assert!(matches!(world.answer(), run::Answer::Parked { turns: 1, .. }));
    }
}
