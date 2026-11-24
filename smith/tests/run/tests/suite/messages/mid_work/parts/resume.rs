//! A later activation receives only what the earlier activation told its host.
use super::*;
use smith_domain::session::record::Resumed;

fn done() -> Turn {
    call(b"finish", br#"{"report":"Resumed.","source":"workspace"}"#)
}

fn kept(world: &World, activation: u64) -> Vec<Resumed> {
    world.parts().iter().map(|(_, _, turn, part)| Resumed { activation, turn: *turn, part: part.clone() }).collect()
}

fn outputs(world: &World) -> Vec<u8> {
    world.prompts()[0]
        .messages
        .iter()
        .flat_map(|message| &message.parts)
        .flat_map(|part| match part {
            Part::Text { text } => text.to_vec(),
            Part::ToolOutput { output, .. } => output.to_vec(),
            Part::Opaque { .. } | Part::ToolCall { .. } => vec![],
        })
        .collect()
}

#[test]
fn a_crashed_command_resumes_its_start_without_running_again() {
    let mut first = command_world();
    while !first.boundaries().iter().any(|(_, b)| matches!(b, Boundary::Io { op: tools::Op::Spawn { .. } })) {
        assert!(!first.drive(1));
    }
    assert_eq!(first.parts().len(), 2);
    assert_start(&first.parts()[1].3);
    assert_eq!(first.turns(), []);
    let mut next = resume_from(
        &first,
        &settings(),
        Box::new([call(b"shell", br#"{"command":"long command"}"#), done()]),
        Fixture::new(),
    );
    next.run(10_000);
    assert_eq!(next.completions().len(), 1);
    assert!(next.boundaries().iter().all(|(_, b)| !matches!(b, Boundary::Io { .. })));
    let output = String::from_utf8_lossy(&outputs(&next)).into_owned();
    assert!(output.contains("interrupted by a worker restart after an unknown time"));
    assert!(output.contains("This call may be run again"));
    assert!(!output.contains("step 1"));
    assert_eq!(next.turns()[0].sequence, first.parts()[0].3.body.sequence);
}

#[test]
fn a_command_exit_supersedes_its_start_when_another_command_crashes() {
    let mut fixture = Fixture::new();
    fixture.checkout.program(b"long command", program(20_000, b"unfinished", Exit::Code(0), &[]));
    let batch = smith_agent_world::fixture::calls(
        Box::new([
            Line::Call {
                name: b"shell".as_slice().into(),
                arguments: br#"{"command":"cargo build"}"#.as_slice().into(),
            },
            Line::Call {
                name: b"shell".as_slice().into(),
                arguments: br#"{"command":"long command"}"#.as_slice().into(),
            },
        ]),
        1,
    );
    let mut first = scripted_in(&settings(), Box::new([batch.clone(), done()]), vec![], fixture);
    first.command_writes(b"cargo build", vec![(Duration::ZERO, b"Finished\n".to_vec())]);
    first.tell_parts();
    while first.boundaries().iter().filter(|(_, b)| matches!(b, Boundary::Io { op: tools::Op::Spawn { .. } })).count()
        < 2
    {
        assert!(!first.drive(1));
    }
    assert_eq!(first.parts().len(), 4);
    assert_start(&first.parts()[1].3);
    assert!(matches!(result(&first.parts()[2].3).1, Returned::Owned { outcome: tools::Outcome::Exited { .. } }));
    assert_start(&first.parts()[3].3);
    let mut next = resume_from(&first, &settings(), Box::new([batch, done()]), Fixture::new());
    next.run(10_000);
    assert_eq!(next.completions().len(), 1);
    assert!(next.boundaries().iter().all(|(_, b)| !matches!(b, Boundary::Io { .. })));
    let results: Vec<_> = next.turns()[0]
        .messages
        .iter()
        .flat_map(|m| &m.content)
        .filter_map(|block| if let Block::ToolResult { result, .. } = block { Some(result) } else { None })
        .collect();
    assert_eq!(results.len(), 2);
    assert!(
        matches!(results[0], Returned::Owned { outcome: tools::Outcome::Exited { head, .. } } if head.as_ref() == b"Finished\n")
    );
    assert!(matches!(results[1], Returned::Owned { outcome: tools::Outcome::Interrupted { ran: None, .. } }));
}

#[test]
fn a_crashed_check_reuses_the_tree_and_results_directory() {
    let mut setup = settings();
    setup.job = Job::MidReport;
    setup.check = Span::millis(20_000, 20_000);
    let mut fixture = Fixture::new();
    fixture.checkout.write(b"work/temper/src/lib.rs", b"pub fn answer() -> u32 { 43 }\n");
    fixture.checkout.write(b"work/temper/.temper/pre-pr", b"#!checks\n");
    let turns = || Box::new([call(b"deliver", br#"{"ticket":"resume"}"#), done()]) as Box<[Turn]>;
    let mut first = scripted_in(&setup, turns(), vec![], fixture);
    first.results_directory(b"/work/temper/check-results");
    first.check_marker(b"passed");
    first.tell_parts();
    while !first.boundaries().iter().any(|(_, b)| matches!(b, Boundary::Check { .. })) {
        assert!(!first.drive(1));
    }
    assert_eq!(first.parts().len(), 1);
    assert_eq!(first.parts()[0].3.kind, PartKind::Answer);
    assert_eq!(first.turns(), []);
    assert_eq!(first.check_results(), [(Box::from(b"/work/temper/check-results".as_slice()), false)]);
    let mut fixture = Fixture::new();
    fixture.checkout = first.take_crashed_checkout();
    setup.check = Span::millis(1, 1);
    let mut next = resume_from(&first, &setup, turns(), fixture);
    next.results_directory(b"/work/temper/check-results");
    next.check_marker(b"passed");
    next.run(10_000);
    assert_eq!(next.check_results(), [(Box::from(b"/work/temper/check-results".as_slice()), true)]);
    assert_eq!(next.delivery_names().len(), 1);
    assert_eq!(next.completions().len(), 1);
    assert_eq!(next.code(), b"pub fn answer() -> u32 { 43 }\n");
}

#[test]
fn an_answer_in_flight_at_a_crash_is_planned_afresh() {
    let mut first = scripted(
        &settings(),
        Box::new([call(b"read", br#"{"path":"src/lib.rs"}"#), call(b"shell", br#"{"command":"cargo build"}"#), done()]),
        vec![],
    );
    first.tell_parts();
    while first.completions().len() < 2 {
        assert!(!first.drive(1));
    }
    assert_eq!(first.turns().len(), 1);
    assert!(first.parts().iter().all(|(_, _, turn, _)| *turn == 1));
    let mut next = resume_from(
        &first,
        &settings(),
        Box::new([call(b"read", br#"{"path":"src/lib.rs"}"#), call(b"shell", br#"{"command":"cargo build"}"#), done()]),
        Fixture::new(),
    );
    next.run(10_000);
    assert_eq!(next.completions().len(), 2);
    assert_eq!(next.turns()[0].sequence, 2);
    assert_eq!(
        next.boundaries().iter().filter(|(_, b)| matches!(b, Boundary::Io { op: tools::Op::Spawn { .. } })).count(),
        1
    );
}

#[test]
fn cut_checks_reuse_the_tree_and_results_directory_before_delivery() {
    let mut setup = settings();
    setup.job = Job::MidReport;
    setup.check = Span::millis(20_000, 20_000);
    let mut fixture = Fixture::new();
    fixture.checkout.write(b"work/temper/src/lib.rs", b"pub fn answer() -> u32 { 43 }\n");
    fixture.checkout.write(b"work/temper/.temper/pre-pr", b"#!checks\n");
    let turns = || Box::new([call(b"deliver", br#"{"ticket":"resume"}"#), done()]) as Box<[Turn]>;
    let mut first = scripted_in(&setup, turns(), vec![], fixture);
    first.results_directory(b"/work/temper/check-results");
    first.check_marker(b"passed");
    first.tell_parts();
    first.drain_with_grace_at(Time::from_nanos(150_000_000), Some(GRACE));
    first.run(10_000);
    assert_eq!(first.check_results(), [(Box::from(b"/work/temper/check-results".as_slice()), false)]);
    let mut fixture = Fixture::new();
    fixture.checkout = first.take_checkout();
    setup.check = Span::millis(1, 1);
    let mut next = resume_from(&first, &setup, turns(), fixture);
    next.results_directory(b"/work/temper/check-results");
    next.check_marker(b"passed");
    next.run(10_000);
    assert_eq!(next.check_results(), [(Box::from(b"/work/temper/check-results".as_slice()), true)]);
    assert_eq!(next.delivery_names().len(), 1);
    assert_eq!(next.delivery_names()[0].0.activation, 2);
    assert_eq!(next.completions().len(), 1);
    assert!(String::from_utf8_lossy(&outputs(&next)).contains("scripted receipt"));
    assert_eq!(next.code(), b"pub fn answer() -> u32 { 43 }\n");
}

#[test]
fn host_decisions_fill_the_call_and_unanswered_calls_get_a_new_name() {
    let mut setup = settings();
    setup.job = Job::HostTools;
    setup.host = smith_agent_world::HostSchedule::Withdrawn;
    setup.host_tool_timeout = Duration::from_secs(30);
    let turns = || Box::new([call(b"host_action", b"{}"), done()]) as Box<[Turn]>;
    let mut first = scripted(&setup, turns(), vec![]);
    first.tell_parts();
    first.drain_with_grace_at(Time::from_nanos(150_000_000), Some(GRACE));
    first.run(10_000);
    let name = first.host_submissions()[0].name;
    setup.host = smith_agent_world::HostSchedule::Answer;
    for decided in [true, false] {
        let mut next = resume_from(&first, &setup, turns(), Fixture::new());
        if decided {
            next.resume_answers(Box::new([smith_domain::AnsweredCall {
                name,
                tool: b"host_action".as_slice().into(),
                answer: smith_domain::Answered::Host(
                    run::HostAnswer::new(b"old decision".as_slice().into(), false).expect("bounded"),
                ),
            }]));
        }
        next.run(10_000);
        assert_eq!(next.completions().len(), 1);
        if decided {
            assert_eq!(next.host_submissions(), []);
            let output = outputs(&next);
            assert_eq!(skein_lib::bytes::count(&output, b"old decision", 2), 1);
        } else {
            assert_eq!(next.host_submissions().len(), 1);
            assert_eq!(next.host_submissions()[0].name.activation, 2);
        }
    }
}

#[test]
fn calls_with_results_stay_in_block_order_and_only_unstarted_calls_run() {
    let batch = || {
        smith_agent_world::fixture::calls(
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
        )
    };
    let mut fixture = Fixture::new();
    fixture.checkout.program(b"long command", program(20_000, b"partial", Exit::Code(0), &[]));
    let mut first = scripted_in(&settings(), Box::new([batch(), done()]), vec![], fixture);
    first.tell_parts();
    first.drain_with_grace_at(Time::from_nanos(50_000_000), Some(GRACE));
    first.run(10_000);
    let mut next = resume_from(&first, &settings(), Box::new([batch(), done()]), Fixture::new());
    next.waking_messages(Box::new([run::Message {
        finish_requested: false,
        name: Token::new(91),
        label: b"host".as_slice().into(),
        text: b"waking message".as_slice().into(),
    }]));
    next.run(10_000);
    assert_eq!(next.completions().len(), 1);
    let io = next
        .boundaries()
        .iter()
        .filter_map(|(_, boundary)| match boundary {
            Boundary::Io { op } => Some(op),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(io.len(), 1);
    assert!(matches!(io[0], tools::Op::List { .. }));
    let ids = next.turns()[0]
        .messages
        .iter()
        .flat_map(|m| &m.content)
        .filter_map(|block| match block {
            Block::ToolResult { id, .. } => Some(id),
            _ => None,
        })
        .collect::<Vec<_>>();
    let calls = first.parts()[0]
        .3
        .body
        .messages
        .last()
        .expect("answer")
        .content
        .iter()
        .filter_map(Block::tool_call)
        .map(|call| &call.id)
        .collect::<Vec<_>>();
    assert_eq!(ids, calls);
    assert!(String::from_utf8_lossy(&outputs(&next)).contains("waking message"));
}

#[test]
fn a_cut_turn_can_be_cut_again_and_resume_both_activations_results() {
    let batch = || {
        smith_agent_world::fixture::calls(
            Box::new([
                Line::Call { name: b"shell".as_slice().into(), arguments: br#"{"command":"one"}"#.as_slice().into() },
                Line::Call { name: b"shell".as_slice().into(), arguments: br#"{"command":"two"}"#.as_slice().into() },
            ]),
            1,
        )
    };
    let fixture = || {
        let mut f = Fixture::new();
        f.checkout.program(b"one", program(20_000, b"one", Exit::Code(0), &[]));
        f.checkout.program(b"two", program(20_000, b"two", Exit::Code(0), &[]));
        f
    };
    let mut first = scripted_in(&settings(), Box::new([batch(), done()]), vec![], fixture());
    first.tell_parts();
    first.drain_with_grace_at(Time::from_nanos(50_000_000), Some(GRACE));
    first.run(10_000);
    let mut second = resume_from(&first, &settings(), Box::new([batch(), done()]), fixture());
    second.drain_with_grace_at(Time::from_nanos(50_000_000), Some(GRACE));
    second.run(10_000);
    assert!(second.completions().is_empty());
    assert_eq!(second.turns(), []);
    let mut parts = kept(&first, 1);
    parts.extend(kept(&second, 2));
    let mut third = resume_from(&first, &settings(), Box::new([batch(), done()]), Fixture::new());
    third.resume_parts(parts.into(), 3);
    third.run(10_000);
    assert_eq!(third.completions().len(), 1);
    assert!(third.boundaries().iter().all(|(_, b)| !matches!(b, Boundary::Io { .. })));
    assert_eq!(third.turns()[0].sequence, 1);
}

#[test]
fn unrelated_parts_are_ignored_instead_of_suppressing_a_completion() {
    let mut first = command_world();
    first.drain_with_grace_at(Time::from_nanos(50_000_000), Some(GRACE));
    first.run(10_000);
    for answer in [false, true] {
        let mut parts = kept(&first, 1);
        if answer {
            parts[0].part.body.sequence = 3;
        } else {
            parts.remove(0);
        }
        let mut next = resume_from(
            &first,
            &settings(),
            Box::new([call(b"shell", br#"{"command":"long command"}"#), done()]),
            Fixture::new(),
        );
        next.resume_parts(parts.into(), 2);
        next.run(10_000);
        assert_eq!(next.completions().len(), 2, "invalid parts do not supply the first answer");
    }
}

#[test]
fn a_resumed_wait_gets_the_waking_prompt_once_and_does_not_park() {
    let batch = || {
        smith_agent_world::fixture::calls(
            Box::new([
                Line::Call {
                    name: b"shell".as_slice().into(),
                    arguments: br#"{"command":"long command"}"#.as_slice().into(),
                },
                Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
            ]),
            1,
        )
    };
    let mut f = Fixture::new();
    f.checkout.program(b"long command", program(20_000, b"partial", Exit::Code(0), &[]));
    let mut first = scripted_in(&settings(), Box::new([batch(), done()]), vec![], f);
    first.tell_parts();
    first.drain_with_grace_at(Time::from_nanos(50_000_000), Some(GRACE));
    first.run(10_000);
    let mut next = resume_from(&first, &settings(), Box::new([batch(), done()]), Fixture::new());
    next.run(10_000);
    assert_eq!(next.completions().len(), 1);
    assert_eq!(next.waiting(), []);
    let output = outputs(&next);
    let waking = smith_agent_world::fixture::RESUME;
    assert_eq!(skein_lib::bytes::count(&output, waking, 2), 1);
}

#[test]
fn an_unanswered_save_is_asked_again_under_this_activation() {
    let setup = Settings { job: Job::MidReport, ..settings() };
    let batch = || {
        smith_agent_world::fixture::calls(
            Box::new([
                Line::Call {
                    name: b"shell".as_slice().into(),
                    arguments: br#"{"command":"long command"}"#.as_slice().into(),
                },
                Line::Call { name: b"save".as_slice().into(), arguments: b"{}".as_slice().into() },
            ]),
            1,
        )
    };
    let mut f = Fixture::new();
    f.checkout.program(b"long command", program(20_000, b"partial", Exit::Code(0), &[]));
    let mut first = scripted_in(&setup, Box::new([batch(), done()]), vec![], f);
    first.grant_save();
    first.tell_parts();
    first.drain_with_grace_at(Time::from_nanos(50_000_000), Some(GRACE));
    first.run(10_000);
    let mut next = resume_from(&first, &setup, Box::new([batch(), done()]), Fixture::new());
    next.grant_save();
    next.run(10_000);
    assert_eq!(next.delivery_names().len(), 1);
    assert_eq!(next.delivery_names()[0].0.activation, 2);
    assert_eq!(next.delivery_names()[0].0.position, 1);
    assert_eq!(next.completions().len(), 1);
}

#[test]
fn children_resume_by_call_position_with_their_first_system_and_a_new_ordinal() {
    let batch = || {
        smith_agent_world::fixture::calls(
            Box::new([
                Line::Call {
                    name: b"sub_agent".as_slice().into(),
                    arguments: br#"{"brief":"@quick","tools":["inspect"]}"#.as_slice().into(),
                },
                Line::Call {
                    name: b"sub_agent".as_slice().into(),
                    arguments: br#"{"brief":"@cut-child","tools":["inspect","shell"],"max_turns":3}"#.as_slice().into(),
                },
            ]),
            1,
        )
    };
    let children = || {
        vec![
            Script {
                cue: b"@quick".as_slice().into(),
                turns: Box::new([smith_agent_world::fixture::says(
                    Box::new([Line::Text { text: b"quick done".as_slice().into() }]),
                    1,
                )]),
            },
            Script {
                cue: b"@cut-child".as_slice().into(),
                turns: Box::new([
                    call(b"read", br#"{"path":"src/lib.rs"}"#),
                    call(b"shell", br#"{"command":"child command"}"#),
                    smith_agent_world::fixture::says(
                        Box::new([Line::Text { text: b"child done".as_slice().into() }]),
                        1,
                    ),
                ]),
            },
        ]
    };
    let fixture = || {
        let mut f = Fixture::new();
        f.checkout.program(b"child command", program(20_000, b"partial", Exit::Code(0), &[]));
        f
    };
    let mut first = scripted_in(&settings(), Box::new([batch(), done()]), children(), fixture());
    first.tell_parts();
    first.drain_with_grace_at(Time::from_nanos(350_000_000), Some(GRACE));
    first.run(10_000);
    let original = first
        .parts()
        .iter()
        .find(|(_, _, _, p)| p.thread == 2 && p.kind == PartKind::Answer)
        .expect("second child answered");
    let Kind::First(opening) = &original.3.body.kind else { panic!("child opening") };
    for retained in [true, false] {
        let mut next = resume_with(&first, &settings(), Box::new([batch(), done()]), fixture(), children());
        if !retained {
            next.resume_parts(
                kept(&first, 1).into_iter().filter(|part| part.part.thread == 0 || part.part.call != 1).collect(),
                2,
            );
        }
        next.run(10_000);
        assert_eq!(next.completions().len(), if retained { 2 } else { 4 });
        if retained {
            assert!(next.boundaries().iter().all(|(_, b)| !matches!(b, Boundary::Io { .. })));
            assert_eq!(&next.prompts()[0].system, opening.system.as_ref().expect("first child system"));
            let child_parts = next.parts().iter().filter(|(_, _, _, p)| p.thread != 0).collect::<Vec<_>>();
            assert_ne!(child_parts.len(), 0);
            assert!(child_parts.iter().all(|(_, _, _, p)| p.thread == 1 && p.call == 1));
        }
    }
}

#[test]
fn a_delivery_the_host_decided_fills_its_original_call_without_checks_or_submission() {
    let setup = Settings { job: Job::MidReport, ..settings() };
    let turns = || Box::new([call(b"deliver", br#"{"ticket":"already decided"}"#), done()]) as Box<[Turn]>;
    let mut fixture = Fixture::new();
    fixture.checkout.write(b"work/temper/src/lib.rs", b"pub fn answer() -> u32 { 43 }\n");
    fixture.checkout.write(b"work/temper/.temper/pre-pr", b"#!checks\n");
    let mut first = scripted_in(&setup, turns(), vec![], fixture);
    first.tell_parts();
    first.run(10_000);
    let answer = first.parts()[0].3.clone();
    let name = first.delivery_names()[0].0;
    let delivery = first.pushes()[0].clone();
    let mut fixture = Fixture::new();
    fixture.checkout = first.take_checkout();
    let mut next = resume_from(&first, &setup, turns(), fixture);
    // The host kept the answer and its own decision, but no whole turn.
    next.resume_history(smith_domain::Transcript {
        version: answer.body.version,
        endpoint: answer.body.endpoint,
        dialect: answer.body.dialect,
        turns: Box::default(),
    });
    next.resume_parts(Box::new([Resumed { activation: 1, turn: 1, part: answer }]), 2);
    next.resume_answers(Box::new([smith_domain::AnsweredCall {
        name,
        tool: b"deliver".as_slice().into(),
        answer: smith_domain::Answered::Delivery(Box::new(delivery)),
    }]));
    next.run(10_000);
    assert_eq!(next.delivery_names(), []);
    assert!(next.boundaries().iter().all(|(_, b)| !matches!(b, Boundary::Check { .. })));
    assert_eq!(next.completions().len(), 1);
    let output = outputs(&next);
    assert_eq!(skein_lib::bytes::count(&output, b"scripted receipt", 2), 1);
}
