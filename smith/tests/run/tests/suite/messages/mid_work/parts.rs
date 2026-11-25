//! Parts and answer-boundary drains through the composed agent and scripted checkout.
mod resume;
use super::*;
use smith_domain::session::{
    llm::{Block, Returned, Role},
    record::{Kind, Part as Told, PartKind},
};

const GRACE: Duration = Duration::from_secs(10);

#[test]
fn a_late_completion_tells_its_answer_and_not_run_calls() {
    let mut settings = settings();
    settings.cancel_at = Some(Duration::from_millis(50));
    settings.races = 1000;
    let mut world = scripted(&settings, Box::new([pair()]), vec![]);
    world.tell_parts();
    world.run(10_000);
    assert_eq!(world.parts().len(), 3);
    assert_eq!(world.parts()[0].3.kind, PartKind::Answer);
    for (_, _, _, part) in &world.parts()[1..] {
        assert!(matches!(result(part).1, Returned::NotRun { .. }));
    }
    assert_eq!(world.turns().len(), 1);
    assert!(world.boundaries().iter().all(|(_, b)| !matches!(b, Boundary::Io { .. })));
}

#[test]
fn a_child_answer_in_flight_delays_the_grace_and_its_cut_has_no_main_result() {
    let mut fixture = Fixture::new();
    fixture.checkout.program(b"child command", program(20_000, b"child step\n", Exit::Code(0), &[]));
    let mut world = scripted_in(
        &settings(),
        Box::new([smith_agent_world::fixture::calls(
            Box::new([
                Line::Text { text: b"delegate".as_slice().into() },
                Line::Call {
                    name: b"sub_agent".as_slice().into(),
                    arguments: br#"{"brief":"@cut-child","tools":["shell"]}"#.as_slice().into(),
                },
            ]),
            1,
        )]),
        vec![Script {
            cue: b"@cut-child".as_slice().into(),
            turns: Box::new([call(b"shell", br#"{"command":"child command"}"#)]),
        }],
        fixture,
    );
    world.tell_parts();
    world.command_writes(b"child command", vec![(Duration::ZERO, b"child step\n".to_vec())]);
    world.drain_with_grace_at(Time::from_nanos(150_000_000), Some(GRACE));
    world.run(10_000);
    assert_eq!(world.completions().len(), 2);
    assert_eq!(
        world.parts().iter().map(|(_, _, _, p)| (p.thread, p.call, p.kind)).collect::<Vec<_>>(),
        [(0, 0, PartKind::Answer), (1, 1, PartKind::Answer), (1, 1, PartKind::Result), (1, 1, PartKind::Result)]
    );
    assert_start(&world.parts()[2].3);
    let last = &world.parts()[3];
    assert_eq!(last.0, Time::from_nanos(10_200_000_000));
    assert!(
        matches!(result(&last.3).1, Returned::Owned { outcome: tools::Outcome::Interrupted { ran: Some(ran), .. } } if *ran == GRACE)
    );
    assert_eq!(world.turns(), []);
}

#[test]
fn calls_never_started_at_the_cut_tell_nothing() {
    let mut fixture = Fixture::new();
    fixture.checkout.program(b"long command", program(20_000, b"step 1\n", Exit::Code(0), &[]));
    let mut world = scripted_in(
        &settings(),
        Box::new([smith_agent_world::fixture::calls(
            Box::new([
                Line::Call {
                    name: b"read".as_slice().into(),
                    arguments: br#"{"path":"src/lib.rs"}"#.as_slice().into(),
                },
                Line::Call {
                    name: b"shell".as_slice().into(),
                    arguments: br#"{"command":"long command"}"#.as_slice().into(),
                },
                Line::Call { name: b"list".as_slice().into(), arguments: br#"{"path":"."}"#.as_slice().into() },
            ]),
            1,
        )]),
        vec![],
        fixture,
    );
    world.tell_parts();
    world.drain_with_grace_at(Time::from_nanos(50_000_000), Some(GRACE));
    world.run(10_000);
    assert_eq!(world.parts().len(), 4);
    assert!(matches!(result(&world.parts()[1].3).1, Returned::Owned { outcome: tools::Outcome::Read { .. } }));
    assert_start(&world.parts()[2].3);
    assert!(matches!(result(&world.parts()[3].3).1, Returned::Owned { outcome: tools::Outcome::Interrupted { .. } }));
    assert!(world.boundaries().iter().all(|(_, b)| !matches!(b, Boundary::Io { op: tools::Op::List { .. } })));
    assert_eq!(world.turns(), []);
}

#[test]
fn a_host_call_in_flight_is_withdrawn_without_a_part() {
    let mut settings = settings();
    settings.job = Job::HostTools;
    settings.host = smith_agent_world::HostSchedule::Withdrawn;
    settings.host_tool_timeout = Duration::from_secs(30);
    let mut world = scripted(&settings, Box::new([call(b"host_action", b"{}")]), vec![]);
    world.tell_parts();
    world.drain_with_grace_at(Time::from_nanos(150_000_000), Some(GRACE));
    world.run(10_000);
    assert_eq!(world.parts().len(), 1);
    assert_eq!(world.turns(), []);
    assert_eq!(world.host_terminals().len(), 1);
    assert_eq!(world.host_withdrawals().len(), 1);
    assert_eq!(world.host_withdrawals()[0].0, Time::from_nanos(10_150_000_000));
    assert_eq!(world.answered_at(), Time::from_nanos(30_120_000_000));
}

#[test]
fn a_cut_job_wait_tells_unread_output_and_its_elapsed_wait() {
    let mut fixture = Fixture::new();
    fixture.checkout.program(b"long command", program(20_000, b"step 1\nstep 2\n", Exit::Code(0), &[]));
    let mut world = scripted_in(
        &settings(),
        Box::new([
            call(b"shell", br#"{"command":"long command","yield_seconds":0}"#),
            call(b"job", br#"{"job":1,"wait_seconds":30}"#),
        ]),
        vec![],
        fixture,
    );
    world.tell_parts();
    world.command_writes(
        b"long command",
        vec![(Duration::ZERO, b"step 1\n".to_vec()), (Duration::from_secs(1), b"step 2\n".to_vec())],
    );
    world.drain_with_grace_at(Time::from_nanos(250_000_000), Some(GRACE));
    world.run(10_000);
    assert_eq!(world.turns().len(), 1);
    let (_, _, turn, last) = world.parts().last().expect("cut wait result");
    assert_eq!(*turn, 2);
    assert!(
        matches!(result(last).1, Returned::Owned { outcome: tools::Outcome::Interrupted { ran: Some(ran), head, .. } }
        if *ran == Duration::from_millis(10_050) && head.as_ref() == b"step 2\n")
    );
}

fn assert_start(part: &Told) {
    assert!(matches!(result(part).1,
        Returned::Owned { outcome: tools::Outcome::Interrupted { ran: None, head, last, dropped: 0 } }
        if head.is_empty() && last.is_empty()));
}

fn result(part: &Told) -> (&[u8], &Returned) {
    assert_eq!(part.kind, PartKind::Result);
    let [message] = part.body.messages.as_ref() else { panic!("one result message") };
    assert_eq!(message.role, Role::User);
    let [Block::ToolResult { id, result }] = message.content.as_ref() else { panic!("one result block") };
    (id, result)
}

#[test]
fn commands_tell_a_start_then_their_exit_only_when_parts_are_enabled() {
    for enabled in [false, true] {
        let mut world = scripted(
            &settings(),
            Box::new([
                call(b"shell", br#"{"command":"cargo build"}"#),
                call(b"finish", br#"{"report":"Done.","source":"workspace"}"#),
            ]),
            vec![],
        );
        if enabled {
            world.tell_parts();
        }
        world.run(10_000);
        if !enabled {
            assert_eq!(world.parts(), []);
            continue;
        }
        let parts = world.parts();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].3.kind, PartKind::Answer);
        assert_eq!((parts[0].1, parts[1].1, parts[2].1), (1, 2, 3));
        assert_start(&parts[1].3);
        assert!(matches!(result(&parts[2].3).1, Returned::Owned { outcome: tools::Outcome::Exited { .. } }));
        assert_eq!(result(&parts[1].3).0, result(&parts[2].3).0);
        assert!(parts[1].0 < parts[2].0);
        assert_ne!(parts[1].3.body, parts[2].3.body);
    }
}

#[test]
fn a_job_wait_tells_a_start_but_a_stop_and_an_unknown_job_do_not() {
    for stop in [false, true] {
        let mut fixture = Fixture::new();
        fixture.checkout.program(b"held", program(1000, b"finished", Exit::Code(0), &[]));
        let job =
            if stop { br#"{"job":1,"stop":true}"#.as_slice() } else { br#"{"job":1,"wait_seconds":30}"#.as_slice() };
        let mut world = scripted_in(
            &settings(),
            Box::new([
                call(b"shell", br#"{"command":"held","yield_seconds":0}"#),
                call(b"job", job),
                call(b"job", br#"{"job":99}"#),
                call(b"finish", br#"{"report":"Done.","source":"workspace"}"#),
            ]),
            vec![],
            fixture,
        );
        world.tell_parts();
        world.run(10_000);
        let job_parts: Vec<_> = world.parts().iter().filter(|(_, _, turn, _)| *turn == 2).collect();
        assert_eq!(job_parts[0].3.kind, PartKind::Answer);
        if stop {
            assert_eq!(job_parts.len(), 2);
        } else {
            assert_eq!(job_parts.len(), 3);
            assert_start(&job_parts[1].3);
            assert_eq!(result(&job_parts[1].3).0, result(&job_parts[2].3).0);
        }
        assert!(!matches!(
            result(&job_parts.last().expect("job end").3).1,
            Returned::Owned { outcome: tools::Outcome::Interrupted { .. } }
        ));
        assert_eq!(world.parts().iter().filter(|(_, _, turn, _)| *turn == 3).count(), 2);
    }
}

#[test]
fn an_ungranted_child_command_tells_no_start() {
    let mut world = scripted(
        &settings(),
        Box::new([
            call(b"sub_agent", br#"{"brief":"@ungranted","tools":["inspect"]}"#),
            call(b"finish", br#"{"report":"Done.","source":"workspace"}"#),
        ]),
        vec![Script {
            cue: b"@ungranted".as_slice().into(),
            turns: Box::new([
                call(b"shell", br#"{"command":"cargo build"}"#),
                smith_agent_world::fixture::says(Box::new([Line::Text { text: b"done".as_slice().into() }]), 1),
            ]),
        }],
    );
    world.tell_parts();
    world.run(10_000);
    let child: Vec<_> = world.parts().iter().filter(|(_, _, _, part)| part.thread == 1).collect();
    assert_eq!(child.len(), 3);
    assert!(matches!(result(&child[1].3).1, Returned::Invalid { .. }));
    assert!(world.boundaries().iter().all(|(_, b)| !matches!(b, Boundary::Io { op: tools::Op::Spawn { .. } })));
}

#[test]
fn result_parts_follow_completion_order_not_call_order() {
    let mut settings = settings();
    settings.network = Span::millis(1, 100);
    let mut world = scripted(&settings, Box::new([pair()]), vec![]);
    world.tell_parts();
    while world.completions().is_empty() {
        assert!(!world.drive(1));
    }
    world.drain_at(world.now());
    world.run(10_000);
    assert_eq!(world.parts().len(), 3);
    assert!(matches!(result(&world.parts()[1].3).1, Returned::Owned { outcome: tools::Outcome::Listed { .. } }));
    assert!(matches!(result(&world.parts()[2].3).1, Returned::Owned { outcome: tools::Outcome::Read { .. } }));
    assert!(world.parts()[1].0 < world.parts()[2].0);
}

fn pair() -> Turn {
    smith_agent_world::fixture::calls(
        Box::new([
            Line::Call { name: b"read".as_slice().into(), arguments: br#"{"path":"src/lib.rs"}"#.as_slice().into() },
            Line::Call { name: b"list".as_slice().into(), arguments: br#"{"path":"."}"#.as_slice().into() },
        ]),
        1,
    )
}

#[test]
fn parts_precede_the_whole_turn_and_are_opt_in() {
    for enabled in [false, true] {
        let mut world = scripted(&settings(), Box::new([pair()]), vec![]);
        if enabled {
            world.tell_parts();
        }
        world.drain_at(Time::from_nanos(50_000_000));
        world.run(10_000);
        assert_eq!(world.turns().len(), 1);
        if !enabled {
            assert_eq!(world.parts(), []);
            continue;
        }
        let parts = world.parts();
        assert_eq!(parts.len(), 3);
        assert_eq!(
            parts.iter().map(|(_, n, turn, p)| (*n, *turn, p.thread, p.call)).collect::<Vec<_>>(),
            [(1, 1, 0, 0), (2, 1, 0, 0), (3, 1, 0, 0)]
        );
        let answer = &parts[0].3;
        assert_eq!(answer.kind, PartKind::Answer);
        assert!(matches!(answer.body.kind, Kind::First(_)));
        assert_eq!(answer.body.messages.len(), 2);
        assert_eq!(answer.body.messages.last().expect("answer message").role, Role::Assistant);
        assert_eq!(answer.body.usage, world.turns()[0].usage);
        assert!(matches!(result(&parts[1].3).1, Returned::Owned { outcome: tools::Outcome::Read { .. } }));
        assert!(matches!(result(&parts[2].3).1, Returned::Owned { outcome: tools::Outcome::Listed { .. } }));
        let calls: Vec<_> = answer
            .body
            .messages
            .last()
            .expect("answer message")
            .content
            .iter()
            .filter_map(|block| block.tool_call().map(|call| call.id.as_ref()))
            .collect();
        assert_eq!(result(&parts[1].3).0, calls[0]);
        assert_eq!(result(&parts[2].3).0, calls[1]);
    }
}

#[test]
fn child_parts_keep_their_thread_call_and_opening() {
    let mut world = scripted(
        &settings(),
        Box::new([smith_agent_world::fixture::calls(
            Box::new([
                Line::Text { text: b"delegate".as_slice().into() },
                Line::Call {
                    name: b"sub_agent".as_slice().into(),
                    arguments: br#"{"brief":"@parts-child","tools":["inspect"]}"#.as_slice().into(),
                },
            ]),
            1,
        )]),
        vec![Script {
            cue: b"@parts-child".as_slice().into(),
            turns: Box::new([
                call(b"read", br#"{"path":"src/lib.rs"}"#),
                smith_agent_world::fixture::says(Box::new([Line::Text { text: b"done".as_slice().into() }]), 1),
            ]),
        }],
    );
    world.tell_parts();
    world.drain_at(Time::from_nanos(50_000_000));
    world.run(10_000);
    let parts = world.parts();
    assert_eq!(
        parts.iter().map(|(_, _, _, p)| (p.thread, p.call, p.kind)).collect::<Vec<_>>(),
        [
            (0, 0, PartKind::Answer),
            (1, 1, PartKind::Answer),
            (1, 1, PartKind::Result),
            (1, 1, PartKind::Answer),
            (0, 0, PartKind::Result),
        ]
    );
    let Kind::First(opening) = &parts[1].3.body.kind else { panic!("child opening") };
    assert_eq!(opening.affinity.thread, 1);
    assert!(!opening.system.as_ref().expect("opening system").is_empty());
    assert_eq!(
        parts[3].3.body.messages.first().expect("prior results").role,
        Role::User,
        "previous result precedes child's next answer"
    );
    assert!(matches!(result(&parts[4].3).1, Returned::Text { error: false, .. }));
}

fn command_world() -> World {
    let mut fixture = Fixture::new();
    fixture.checkout.program(b"long command", program(20_000, b"step 1\n", Exit::Code(0), &[]));
    let mut world = scripted_in(
        &settings(),
        Box::new([
            call(b"shell", br#"{"command":"long command"}"#),
            call(b"finish", br#"{"report":"Not reached.","source":"workspace"}"#),
        ]),
        vec![],
        fixture,
    );
    world.tell_parts();
    world.command_writes(b"long command", vec![(Duration::ZERO, b"step 1\n".to_vec())]);
    world
}

#[test]
fn drain_waits_for_the_answer_then_cuts_a_command_at_the_grace() {
    let mut world = command_world();
    world.drain_with_grace_at(Time::from_nanos(50_000_000), Some(GRACE));
    world.run(10_000);
    assert_eq!(world.completions().len(), 1);
    assert_eq!(world.parts().len(), 3);
    assert_eq!(world.parts()[0].0, Time::from_nanos(100_000_000));
    assert_eq!(world.parts()[1].0, Time::from_nanos(100_000_000));
    assert_start(&world.parts()[1].3);
    assert_eq!(world.parts()[2].0, Time::from_nanos(10_100_000_000));
    assert!(matches!(result(&world.parts()[2].3).1,
        Returned::Owned { outcome: tools::Outcome::Interrupted { ran: Some(ran), head, last, dropped: 0 } }
        if *ran == GRACE && head.as_ref() == b"step 1\n" && last.is_empty()));
    assert_eq!(world.turns(), []);
    assert!(matches!(world.answer(), run::Answer::Failed { failure: run::RunFailure::Cancelled, turns: 0, .. }));
}

#[test]
fn a_turn_that_ends_within_the_grace_is_still_told_whole() {
    let mut world = scripted(&settings(), Box::new([pair()]), vec![]);
    world.tell_parts();
    world.drain_with_grace_at(Time::from_nanos(50_000_000), Some(GRACE));
    world.run(10_000);
    assert_eq!(world.parts().len(), 3);
    assert_eq!(world.turns().len(), 1);
    assert!(matches!(world.answer(), run::Answer::Failed { failure: run::RunFailure::Cancelled, turns: 1, .. }));
    assert!(world.answered_at() < Time::from_nanos(10_100_000_000));
}

#[test]
fn drain_aborts_checks_without_telling_a_delivery_result() {
    let mut settings = settings();
    settings.job = Job::MidChange;
    settings.check = Span::millis(20_000, 20_000);
    let mut fixture = Fixture::new();
    fixture.checkout.write(b"work/temper/src/lib.rs", b"pub fn answer() -> u32 { 43 }\n");
    fixture.checkout.write(b"work/temper/.temper/pre-pr", b"#!checks\n");
    let mut world =
        scripted_in(&settings, Box::new([call(b"deliver", br#"{"ticket":"Not reached."}"#)]), vec![], fixture);
    world.tell_parts();
    world.drain_with_grace_at(Time::from_nanos(150_000_000), Some(GRACE));
    world.run(10_000);
    assert_eq!(world.parts().len(), 1);
    assert_eq!(world.parts()[0].3.kind, PartKind::Answer);
    assert_eq!(world.turns(), []);
    assert_eq!(world.checked(), []);
    assert!(world.delivery_submissions().is_empty());
    assert_eq!(world.answered_at(), Time::from_nanos(10_150_000_000));
    assert!(matches!(world.answer(), run::Answer::Failed { failure: run::RunFailure::Cancelled, turns: 0, .. }));
}

#[test]
fn cancel_after_an_answer_drain_cuts_immediately_without_more_parts() {
    let mut world = command_world();
    world.drain_with_grace_at(Time::from_nanos(50_000_000), Some(GRACE));
    message(&mut world, 200, 7, b"not read");
    while world.parts().is_empty() {
        assert!(!world.drive(1));
    }
    assert_eq!(world.parts().len(), 2);
    assert_start(&world.parts()[1].3);
    world.cancel_run();
    world.run(10_000);
    assert_eq!(world.parts().len(), 2);
    assert_eq!(world.turns(), []);
    assert!(world.answered_at() < Time::from_nanos(10_100_000_000));
}

#[test]
fn part_kept_releases_the_shared_record_window() {
    let mut world = scripted(
        &settings(),
        Box::new([pair(), call(b"finish", br#"{"report":"Done.","source":"workspace"}"#)]),
        vec![],
    );
    world.tell_parts();
    world.record_window(smith_domain::Window { turns: 1, bytes: u64::MAX });
    world.part_kept_at(Time::from_nanos(200_000_000), 1);
    world.part_kept_at(Time::from_nanos(300_000_000), 2);
    world.part_kept_at(Time::from_nanos(400_000_000), 3);
    world.acknowledge_at(Time::from_nanos(500_000_000), 1);
    world.part_kept_at(Time::from_nanos(700_000_000), 4);
    world.part_kept_at(Time::from_nanos(800_000_000), 5);
    world.run(10_000);
    assert_eq!(
        world.parts().iter().map(|(at, _, _, _)| *at).collect::<Vec<_>>(),
        [
            Time::from_nanos(100_000_000),
            Time::from_nanos(200_000_000),
            Time::from_nanos(300_000_000),
            Time::from_nanos(600_000_000),
            Time::from_nanos(700_000_000),
        ]
    );
    assert_eq!(world.completions()[1].started, Time::from_nanos(500_000_000));
    assert_eq!(world.turns().len(), 2);
}

fn resume_from(first: &World, settings: &Settings, turns: Box<[Turn]>, fixture: Fixture) -> World {
    resume_with(first, settings, turns, fixture, vec![])
}

fn resume_with(
    first: &World,
    settings: &Settings,
    turns: Box<[Turn]>,
    fixture: Fixture,
    mut children: Vec<Script>,
) -> World {
    let part = &first.parts()[0].3.body;
    let history = smith_domain::Transcript {
        version: part.version,
        endpoint: part.endpoint,
        dialect: part.dialect,
        turns: first.turns().into(),
    };
    let workspace = run::Workspace {
        directories: Box::new([run::Directory {
            results: None,
            host: b"/work/temper".as_slice().into(),
            name: b"work".as_slice().into(),
            root: fixture.repos[0].root,
            writable: true,
            git: true,
            conflicts: Box::default(),
        }]),
    };
    let settings = Settings { resume: true, ..*settings };
    let charter = scripted_charter(&settings);
    children.push(Script { cue: charter.brief.sections[0].text.clone(), turns });
    let scripts = children.into_boxed_slice();
    let mut world = World::with_workspace_scripts_charter(
        settings,
        Some(history),
        Some(workspace),
        fixture.checkout,
        scripts,
        charter,
    );
    let parts = first
        .parts()
        .iter()
        .map(|(_, _, turn, part)| smith_domain::session::record::Resumed {
            activation: 1,
            turn: *turn,
            part: part.clone(),
        })
        .collect();
    world.resume_parts(parts, 2);
    world
}

#[test]
fn a_cut_command_is_not_repeated_and_its_interruption_reaches_the_next_request() {
    let mut first = command_world();
    first.drain_with_grace_at(Time::from_nanos(50_000_000), Some(GRACE));
    first.run(10_000);
    let mut next = resume_from(
        &first,
        &settings(),
        Box::new([
            call(b"shell", br#"{"command":"long command"}"#),
            call(b"finish", br#"{"report":"Resumed.","source":"workspace"}"#),
        ]),
        Fixture::new(),
    );
    next.run(10_000);
    assert_eq!(next.completions().len(), 1, "only the completion after the cut answer: {:?}", next.answer());
    assert!(
        next.boundaries().iter().all(|(_, boundary)| !matches!(boundary, Boundary::Io { .. })),
        "the interrupted command never starts again"
    );
    assert!(next.prompts()[0].messages.iter().flat_map(|m| &m.parts).any(|part| matches!(part,
        Part::ToolOutput { output, .. } if String::from_utf8_lossy(output).contains("interrupted by a worker restart after 10 s") && String::from_utf8_lossy(output).contains("step 1"))));
    assert_eq!(next.turns()[0].sequence, first.parts()[0].3.body.sequence);
}
