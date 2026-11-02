//! Both roots and Smith's domain share every durable and process crossing.
use jig_domain_tasks as tasks;
use temper_engine_domain::{Key, Record};
use temper_system_world::composed::{World, crossed_script, report_script, text_script};

#[test]
fn a_committed_chat_runs_through_the_worker_and_smith_and_returns_a_report() {
    let mut world = World::chat(910, report_script(false));
    world.run();
    let observed = &world.observed;
    assert_eq!(observed.assignments, 1);
    assert_eq!(observed.spawns, 1);
    assert_eq!(observed.answers, 1);
    assert_eq!(observed.gone, 1);
    assert_eq!(observed.turns, observed.turn_acks);
    assert!(observed.turns > 0);
    let task = world.assignment().task;
    assert!(matches!(world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)))),
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row))))
        if matches!(row.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { .. })))));
}

#[test]
fn an_open_chats_text_reply_ends_its_turn_after_one_call() {
    use jig_domain_core::{CoreRecord, Record as CoreRecordRow};
    use skein_lib::Duration;

    let reply = b"Hello there.";
    let mut world = World::open_chat(917, text_script(reply));
    world.run();
    let task = world.assignment().task;
    let attempt = world.assignment().attempt;
    let idle = world.assignment().run.policy.waiting;
    for elapsed in [Duration::ZERO, idle.saturating_add(Duration::from_secs(1))] {
        world.advance(elapsed);
        assert_eq!(world.agent().prompts().len(), 1, "a text reply needs no finite-run nudge");
        assert_eq!(world.observed.assignments, 1, "idle time starts no new attempt");
        assert_eq!(world.observed.answers, 1);
        let mut replies = 0;
        let mut terminals = 0;
        for row in world.store.rows.values() {
            match row {
                Record::Core(CoreRecordRow::Core(CoreRecord::Turn(turn))) if turn.task == task => {
                    replies += turn.transcript.windows(reply.len()).filter(|bytes| *bytes == reply).count();
                }
                Record::Core(CoreRecordRow::Core(CoreRecord::Terminal(terminal))) if terminal.task == task => {
                    assert_eq!(terminal.attempt, attempt);
                    assert!(!matches!(terminal.end, tasks::End::Failed(_)), "no failed run");
                    assert_eq!(terminal.end, tasks::End::Parked, "the Open reply parks its run");
                    terminals += 1;
                }
                Record::Core(_) | Record::Forge { .. } => {}
            }
        }
        assert_eq!(replies, 1, "the reply is committed once in this chat's conversation");
        assert_eq!(terminals, 1, "one committed activation terminal");
        let Some(Record::Core(CoreRecordRow::Tasks(tasks::Stored::Live(live)))) =
            world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task))))
        else {
            panic!("the chat remains live");
        };
        assert_eq!(live.tries, tasks::Tries::NONE);
        assert_eq!(live.phase, tasks::Phase::Active(tasks::Active::Idle), "the chat waits for words");
        assert_eq!(live.attempt, attempt);
        assert_eq!(live.last_answer, Some(attempt));
        assert!(!world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)))));
    }
}

#[test]
fn a_finite_chats_text_reply_is_nudged_and_finishes_with_its_report() {
    use jig_domain_core::{CoreRecord, Record as CoreRecordRow};
    use skein_fake_llm_domain::api::Part;

    let reply = b"Hello there.";
    let mut script = text_script(reply);
    let mut turns = script.turns.to_vec();
    turns.extend(report_script(false).turns);
    script.turns = turns.into_boxed_slice();
    let mut world = World::chat(919, script);
    world.run();

    let prompts = world.agent().prompts();
    assert_eq!(prompts.len(), 2, "a Finite reply is followed by one nudged call");
    // smith's prompt::render_nudge, Mode::Finite and Stop::EndTurn.
    let nudge = b"You stopped without calling `finish`, so the work is not done. Carry on with the work, and call `finish` when it is done. This is reminder ";
    assert!(
        prompts[1].messages.iter().any(|message| message.parts.iter().any(|part| {
            matches!(part, Part::Text { text } if text.windows(nudge.len()).any(|bytes| bytes == nudge))
        })),
        "the second prompt carries smith's Finite nudge"
    );
    assert_eq!(world.observed.assignments, 1);
    assert_eq!(world.observed.spawns, 1);
    assert_eq!(world.observed.answers, 1);

    let task = world.assignment().task;
    let attempt = world.assignment().attempt;
    let Some(Record::Core(CoreRecordRow::Tasks(tasks::Stored::Ended(ended)))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task))))
    else {
        panic!("the Finite task ended");
    };
    assert_eq!(
        ended.phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report {
            words: Box::from(&b"The composed run answered."[..]),
        }))
    );
    assert_eq!(ended.tries, tasks::Tries::NONE, "the nudge spends no try");
    let mut replies = 0;
    let mut terminals = 0;
    for row in world.store.rows.values() {
        match row {
            Record::Core(CoreRecordRow::Core(CoreRecord::Turn(turn))) if turn.task == task => {
                replies += turn.transcript.windows(reply.len()).filter(|bytes| *bytes == reply).count();
            }
            Record::Core(CoreRecordRow::Core(CoreRecord::Terminal(terminal))) if terminal.task == task => {
                assert_eq!(terminal.attempt, attempt);
                assert!(!matches!(terminal.end, tasks::End::Failed(_)), "no failed run");
                terminals += 1;
            }
            Record::Core(_) | Record::Forge { .. } => {}
        }
    }
    assert_eq!(terminals, 1, "one committed activation terminal");
    assert_eq!(replies, 1, "the text reply is committed once in this chat's conversation");
}

#[test]
fn an_open_chats_reply_reaches_its_run_watch_in_pieces_before_its_turn_commits() {
    use jig_domain_core::{CoreRecord, Record as CoreRecordRow};
    use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};

    // The process face plays the service's live producer; the world plays
    // the party's run watch (domain/testing.md, 2.1).
    let reply = b"Hello there.";
    let pieces = 3;
    let mut world = World::open_chat_in_pieces(
        918,
        Script {
            cue: Box::from(&b"Complete the task within its authority."[..]),
            turns: Box::new([Turn {
                lines: Box::new([Line::Text { text: reply.as_slice().into() }]),
                finish: Finish::Stop,
                tokens: 20,
            }]),
        },
        pieces,
    );
    for piece in 1..=pieces {
        let prefix = &reply[..reply.len() * piece / pieces];
        world.until_reply(prefix);
        let watch = world.run_watch.as_ref().expect("the party's run watch");
        assert_eq!(watch.text, prefix);
        assert_eq!(watch.turn, Some((world.assignment().attempt, 1, 0)));
        assert_eq!(world.observed.pieces, piece);
        assert_eq!(world.observed.turns, 0, "the reply turn is still behind its pieces");
        let task = world.assignment().task;
        assert!(
            !world.store.rows.values().any(|row| matches!(
                row,
                Record::Core(CoreRecordRow::Core(CoreRecord::Turn(turn))) if turn.task == task
            )),
            "no turn is committed while its reply is held"
        );
        world.release_reply();
    }
    world.run();
    assert_eq!(world.observed.pieces, pieces);
    assert_eq!(world.observed.turns, 1);
    assert_eq!(world.observed.turn_acks, 1);
    assert_eq!(world.run_watch.as_ref().expect("the party's run watch").text, reply);
    let task = world.assignment().task;
    let turns = world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            Record::Core(CoreRecordRow::Core(CoreRecord::Turn(turn))) if turn.task == task => Some(turn),
            Record::Core(_) | Record::Forge { .. } => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(turns.len(), 1, "one committed reply turn");
    assert_eq!(turns[0].attempt, world.assignment().attempt);
    assert!(turns[0].transcript.windows(reply.len()).any(|bytes| bytes == reply));
}

#[test]
fn smiths_host_call_and_its_durable_reply_traverse_the_worker_process_channel() {
    let mut world = World::chat(911, report_script(true));
    world.run();
    assert_eq!(world.observed.relays, 1);
    assert_eq!(world.observed.replies, 1);
    assert_eq!(world.agent().host_submissions().len(), 1);
    assert_eq!(world.agent().host_terminals().len(), 1);
    assert_eq!(world.observed.turns, world.observed.turn_acks);
}

#[test]
fn a_persons_words_reach_smith_through_the_live_worker_channel() {
    let mut world = World::chat(912, crossed_script());
    world.until_call();
    world.act(
        jig_fake_parties::Ask::Say {
            task: jig_fake_parties::Task::Last,
            words: Box::from(&b"continue with these words"[..]),
            release: None,
        },
        [42; 16],
    );
    world.run();
    assert!(world.observed.messages.iter().any(|(_, words)| words.as_ref() == b"continue with these words"));
    assert!(
        world
            .agent()
            .messages_seen()
            .iter()
            .any(|(_, seen)| matches!(seen, smith_agent_world::messages_referee::Seen::Input { .. }))
    );
}

#[test]
fn a_persons_stop_crosses_the_worker_and_cancels_smith_before_the_task_answers() {
    let mut world = World::chat(913, report_script(true));
    world.until_call();
    world.act(jig_fake_parties::Ask::Stop { task: jig_fake_parties::Task::Last }, [43; 16]);
    world.run();
    assert_eq!(world.observed.cancels, 1);
    assert!(matches!(
        world.agent().answer(),
        smith_domain_run::Answer::Failed { failure: smith_domain_run::Failure::Cancelled, .. }
    ));
    assert_eq!(world.observed.answers, 1);
    assert_eq!(world.observed.gone, 1);
}

#[test]
fn an_adopted_repository_is_prepared_by_the_worker_and_mounted_in_smith() {
    let mut world = World::with_repository(914, report_script(false));
    world.run();
    assert!(world.observed.checkouts >= 4);
    assert_eq!(world.assignment().workspace.repositories.len(), 1);
    let repository = &world.assignment().workspace.repositories[0];
    assert!(repository.push.is_none(), "inspection keeps the mount read only");
    assert_eq!(
        world.agent().disk().files().get(b"repo/src/lib.rs".as_slice()),
        Some(&b"pub fn answer() -> u32 { 42 }\n".as_slice())
    );
    assert_eq!(world.observed.answers, 1);
}

#[test]
fn a_parked_smith_run_resumes_from_the_prefix_restored_through_both_roots() {
    use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
    let mut world = World::open_chat(
        915,
        Script {
            cue: Box::from(&b"Complete the task within its authority."[..]),
            turns: Box::new([
                Turn {
                    lines: Box::new([Line::Call { name: Box::from(&b"wait"[..]), arguments: Box::from(&b"{}"[..]) }]),
                    finish: Finish::ToolCalls,
                    tokens: 20,
                },
                Turn {
                    lines: Box::new([Line::Text { text: Box::from(&b"Waiting for the person."[..]) }]),
                    finish: Finish::Stop,
                    tokens: 20,
                },
                Turn {
                    lines: Box::new([Line::Call { name: Box::from(&b"wait"[..]), arguments: Box::from(&b"{}"[..]) }]),
                    finish: Finish::ToolCalls,
                    tokens: 20,
                },
            ]),
        },
    );
    world.run();
    assert!(matches!(world.agent().answer(), smith_domain_run::Answer::Parked { .. }));
    let previous = world.agent().turns().last().expect("committed conversation").sequence;
    let attempt = world.assignment().attempt;
    world.resume(b"continue", report_script(false));
    world.run();
    assert!(world.assignment().attempt > attempt);
    assert!(world.agent().turns().first().expect("resumed output").sequence > previous);
    assert_eq!(world.observed.assignments, 2);
    assert_eq!(world.observed.spawns, 2);
    assert_eq!(world.observed.answers, 2);
}

#[test]
fn a_withheld_store_commit_holds_the_party_reply_and_worker_assignment() {
    let mut world = World::chat(916, report_script(false));
    world.store.fault(temper_engine_domain_world::store::Fault::Hold { commits: 1 });
    for _ in 0..200 {
        world.tick();
    }
    assert!(world.store.applied > 0);
    assert!(world.party.replies.is_empty(), "the sign-in reply waits for durability acknowledgement");
    assert_eq!(world.observed.assignments, 0);
    assert_eq!(world.observed.spawns, 0);
    world.release_commits();
    world.run();
    assert_eq!(world.observed.answers, 1);
}

// This system world is the lowest one with real engine, worker and smith tool choice.
#[test]
fn an_agreed_goals_seventh_completion_question_parks_at_the_stock_turn_boundary() {
    agreed_goal_walk(920, true);
}

#[test]
fn an_agreed_goals_earlier_wait_parks_and_resumes_with_stock_turns() {
    agreed_goal_walk(921, false);
}

#[test]
fn an_agreed_goals_repeated_delegation_receives_the_plan_refusal_then_asks_and_waits() {
    use skein_fake_llm_domain::api::Part;
    use temper_system_world::composed::call;
    const REQUEST: &[u8] = br#"{"words":"Add CSV export to widget. Before anything is changed, have the work tell me its plan and wait for my go-ahead.","agree":true}"#;
    const REFUSAL: &[u8] =
        b"this work is to be agreed: ask your requester your plan with message, as a question, and wait";
    let mut script = plan_script(false);
    // The repeated goal call is the observed transcript; asking and waiting are the repair oracle.
    let mut turns = script.turns.into_vec();
    turns.insert(0, call(b"goal", REQUEST));
    script.turns = turns.into_boxed_slice();
    let mut world = World::agreed_goal(922, script);
    world.run();
    assert!(
        world
            .agent()
            .prompts()
            .iter()
            .flat_map(|prompt| &prompt.messages)
            .flat_map(|message| &message.parts)
            .any(|part| matches!(part,
            Part::ToolOutput { output, is_error: true, .. } if output.as_ref() == REFUSAL))
    );
    assert!(matches!(world.agent().answer(), smith_domain_run::Answer::Parked { spent, .. } if spent.turns == 3));
    world.pending_plan();
    world.hold_goal(skein_lib::Duration::from_secs(121));
    world.pending_plan();
}

fn plan_script(reads: bool) -> skein_fake_llm_domain::api::Script {
    use temper_system_world::composed::call;
    let mut turns = Vec::new();
    if reads {
        turns.push(call(b"read_branch", br#"{"repository":".","branch":"main"}"#));
        turns.push(call(b"read_branch", br#"{"branch":"main"}"#));
        for path in [".", "src", "README.md", "src/main.rs"] {
            turns.push(call(
                b"read_file",
                format!(r#"{{"head":"0000000000000000000000000000000000000001","path":"{path}","max_bytes":4096}}"#)
                    .as_bytes(),
            ));
        }
    }
    turns
        .push(call(b"message", br#"{"target":1,"form":"question","words":"Add CSV export with a header. Go ahead?"}"#));
    turns.push(call(b"wait", b"{}"));
    skein_fake_llm_domain::api::Script {
        cue: b"Help the person with their work.".as_slice().into(),
        turns: turns.into_boxed_slice(),
    }
}

fn agreed_goal_walk(seed: u64, reads: bool) {
    use skein_fake_llm_domain::api::Script;
    use temper_system_world::composed::call;
    let mut world = World::agreed_goal(seed, plan_script(reads));
    world.run();
    assert_eq!(world.assignment().task, 2);
    assert_eq!((world.assignment().run.policy.turns, world.assignment().run.policy.reserve_turns), (8, 1));
    assert!(
        matches!(world.agent().answer(), smith_domain_run::Answer::Parked { spent, .. } if spent.turns == if reads { 8 } else { 2 })
    );
    if reads {
        assert_plan_reads(&world);
    }
    world.pending_plan();
    world.hold_goal(skein_lib::Duration::from_secs(121));
    world.pending_plan();
    world.next_script(Script {
        cue: b"Read only what the change needs".as_slice().into(),
        turns: Box::new([
            call(b"write", br#"{"path":"export.csv","content":"fruit,count\n"}"#),
            call(b"finish", br#"{"title":"Add CSV export","body":"Add the agreed header."}"#),
        ]),
    });
    world.answer_goal(
        b"Go ahead. The CSV's first line must be the header fruit,count.",
        Script {
            cue: b"Help the person with their work.".as_slice().into(),
            turns: Box::new([
                call(b"change", br#"{"words":"Add export.csv containing fruit,count and a newline"}"#),
                call(b"wait", b"{}"),
                call(b"finish", br#"{"report":"The agreed CSV export landed."}"#),
            ]),
        },
    );
    world.finish_goal();
    world.assert_goal_landed();
}

fn assert_plan_reads(world: &World) {
    use smith_domain_session::llm::{Block, Returned};
    let outcomes = world
        .agent()
        .turns()
        .iter()
        .take(6)
        .map(|turn| {
            turn.messages
                .iter()
                .flat_map(|message| &message.content)
                .filter_map(|block| match block {
                    Block::ToolResult { result: Returned::Text { error, .. }, .. } => Some(*error),
                    Block::Opaque { .. }
                    | Block::Refusal { .. }
                    | Block::Text { .. }
                    | Block::ToolCall { .. }
                    | Block::ToolResult { .. } => None,
                })
                .next_back()
                .expect("each read has a result")
        })
        .collect::<Vec<_>>();
    assert_eq!(outcomes, [true, false, false, false, false, false]);
}
