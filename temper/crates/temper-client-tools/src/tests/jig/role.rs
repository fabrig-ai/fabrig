//! Standing roles fill a template, join their notes and answer talks.
use crate::tests::fixtures as f;
use jig_client::contract as c;
use skein_lib::{List, Wall};

const TALK: &[&str] = &["role", "talk", "41", "From now on, label flaky tests.", "--key", "talk"];

#[test]
fn role_talk_waits_then_prints_only_the_new_turns_across_pages() {
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let argv = f::extended(TALK, if json { &["--json"] } else { &[] });
        let call = root.submit(argv.as_slice());
        root.serve(call, f::task_result(f::goal(f::TASK, c::WaitsOn::Nothing)));
        assert_eq!(
            f::act_ask(&root, call),
            &c::Ask::Talk {
                project: f::PROJECT,
                task: 41,
                words: c::Text::new(b"From now on, label flaky tests.".as_slice().into(), crate::limits::TEXT).unwrap()
            }
        );
        assert_eq!(root.sent_name(call), Some(f::named("talk")));
        root.done(call, c::Outcome::Document(c::DocumentOutcome::Said { task: 41, message: 24 }));
        root.deliver_subject(&f::task_watch(), f::writing_chat());
        assert!(root.answer(call).is_none());
        let c::Delivery::Task(mut task) = f::parked() else { panic!("task") };
        task.activity = 29;
        root.serving = Some(call);
        root.deliver_subject(&f::task_watch(), c::Delivery::Task(task));
        root.serving = None;
        assert!(root.answer(call).is_none(), "must read the committed reply");
        let c::ReadResult::Value(c::ReadValue::Activity(page)) = f::settings_conversation(true) else {
            panic!("activity")
        };
        let mut newest = List::with_capacity(3);
        let mut older = List::with_capacity(7);
        for mut row in page.rows.into_boxed() {
            if row.source.position == 26
                && let Some(turn) = &mut row.decoded
                && let c::TurnRead::Decoded(turn) = &mut turn.turn
            {
                for message in turn.messages.iter_mut() {
                    for block in message.blocks.iter_mut() {
                        if let c::Block::Text(text) = block {
                            *text =
                                c::Text::new(b"Add it to my rules?".as_slice().into(), crate::limits::TEXT).unwrap();
                        }
                    }
                }
            }
            if row.source.position >= 28 {
                newest.push(row).unwrap();
            } else {
                older.push(row).unwrap();
            }
        }
        root.serve(
            call,
            c::ReadResult::Value(c::ReadValue::Activity(Box::new(c::ActivityPage {
                rows: newest,
                before: Some(f::cursor()),
                after: None,
                cut: false,
            }))),
        );
        assert!(root.answer(call).is_none(), "read back to the admitted words");
        root.serve(
            call,
            c::ReadResult::Value(c::ReadValue::Activity(Box::new(c::ActivityPage {
                rows: older,
                before: None,
                after: None,
                cut: false,
            }))),
        );
        let printed = root.answer(call).expect("reply");
        assert_eq!(printed.status, 0, "{printed:?}");
        let expected = b"Add it to my rules?\nProposal 2 was accepted and made revision 2.\n";
        if json {
            f::unique_keys(&printed.stdout).unwrap();
            assert!(f::has(&printed, br#""met":true"#));
            assert!(f::has(&printed, b"Add it to my rules?"));
        } else {
            assert_eq!(printed.stdout.as_ref(), expected);
        }
        assert!(!f::has(&printed, f::CONVERSATION_WORDS[0]));
        assert!(!f::has(&printed, b"settings_read"));
    }
}

#[test]
fn role_talk_and_leave_name_their_refusals_and_leave_success() {
    for (leave, refusal, expected) in [
        (false, Some(c::Refusal::NotOffered), b"not a standing role\n".as_slice()),
        (false, Some(c::Refusal::Taken { by: 74 }), b"talking with party 74\n"),
        (true, Some(c::Refusal::NotWaiting), b"no talk to leave\n"),
        (true, None, b"left: task 41\n"),
    ] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(if leave { &["role", "talk", "41", "--leave", "--key", "left"] } else { TALK });
        root.serve(call, f::task_result(f::goal(f::TASK, c::WaitsOn::Nothing)));
        if leave {
            assert_eq!(f::act_ask(&root, call), &c::Ask::Leave { project: f::PROJECT, task: 41 });
        }
        if let Some(refusal) = &refusal {
            root.answered(call, c::Answer::Refused { refusal: refusal.clone(), at: None });
        } else {
            root.done(call, c::Outcome::Document(c::DocumentOutcome::Left { task: 41 }));
        }
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, if refusal.is_some() { 2 } else { 0 }, "{printed:?}");
        assert_eq!(if refusal.is_some() { printed.stderr.as_ref() } else { printed.stdout.as_ref() }, expected);
    }
}

#[test]
fn role_talk_timeout_hold_or_end_prints_what_came_and_exits_three() {
    for stop in [None, Some(c::Phase::Held), Some(c::Phase::Ended)] {
        let mut root = f::signed_in(f::PARTY);
        let argv = f::extended(TALK, &["--timeout", "1s"]);
        let call = root.submit(argv.as_slice());
        root.serve(call, f::task_result(f::goal(f::TASK, c::WaitsOn::Nothing)));
        root.done(call, c::Outcome::Document(c::DocumentOutcome::Said { task: 41, message: 24 }));
        let mut delivery = f::writing_chat();
        let c::Delivery::Task(task) = &mut delivery else { panic!("task") };
        task.activity = 27;
        if let Some(phase) = stop {
            task.chip.phase = phase;
        }
        root.serving = Some(call);
        root.deliver_subject(&f::task_watch(), delivery);
        if stop.is_none() {
            assert!(root.answer(call).is_none());
            root.now = skein_lib::Time::ZERO.checked_add(skein_lib::Duration::from_secs(1)).unwrap();
            root.fire();
        }
        root.serving = None;
        root.serve(call, f::settings_conversation(true));
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 3, "{printed:?}");
        assert_eq!(printed.stdout.as_ref(), b"Proposal 2 is ready for review.\n");
    }
}

#[test]
fn role_talk_requires_words_or_leave_and_help_lists_both() {
    for argv in [
        &["role", "talk", "41", "--key", "k"][..],
        &["role", "talk", "41", "words", "--leave", "--key", "k"],
        &["role", "talk", "41", "words", "--timeout", "3601s", "--key", "k"],
    ] {
        assert!(f::parse(argv).is_err(), "{argv:?}");
    }
    for argv in [TALK, &["role", "talk", "41", "--leave", "--key", "k"]] {
        assert_eq!(f::printed(&f::call(argv)), f::parse(argv).unwrap());
    }
    let help = f::help(Some(crate::jig::rows::ROLE_TALK_AT));
    for form in [b"role talk TASK TEXT".as_slice(), b"role talk TASK --leave"] {
        assert!(skein_lib::bytes::find(&help, form).is_some(), "{help:?}");
    }
}

#[test]
fn role_show_reads_the_whole_report_and_names_an_open_talk() {
    let body = "Landed tests; chose focused checks; waiting on CI; spent 2.\n".repeat(1800);
    for json in [false, true] {
        for written in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let argv = f::extended(&["role", "show", "41", "--report"], if json { &["--json"] } else { &[] });
            let call = root.submit(argv.as_slice());
            let mut task = f::goal(f::TASK, c::WaitsOn::Nothing);
            if written {
                task.talk = Some(c::Talk { party: f::PARTY, opened: Wall::EPOCH, last: Wall::EPOCH });
            }
            root.serve(call, f::task_result(task));
            assert_note(&root, call, 41, b"report");
            let notes = [(9, b"Later report".as_slice()), (2, body.as_bytes())];
            root.serve(call, described_notes(b"report", if written { &notes } else { &[] }));
            let printed = root.answer(call).unwrap();
            assert_eq!(printed.status, 0, "{printed:?}");
            if json {
                f::unique_keys(&printed.stdout).unwrap();
                assert!(f::has(
                    &printed,
                    if written {
                        br#""report":{"number":2,"revision":4,"written":0,"body":"#
                    } else {
                        br#""report":null"#
                    }
                ));
            } else if written {
                assert!(f::has(&printed, b"talking with you since 1970-01-01 00:00 UTC\nreport: note 2 revision 4 written 1970-01-01 00:00 UTC\n"));
                assert!(printed.stdout.ends_with(body.as_bytes()));
            } else {
                assert!(printed.stdout.ends_with(b"no report yet\n"));
                assert!(!f::has(&printed, b"talking with"));
            }
        }
    }
}

pub(super) const START: &[&str] =
    &["role", "start", "--project", "2", "--budget", "50", "Owns the tests", "--key", "k"];
pub(super) fn me() -> c::Me {
    let mut me = f::chat_me(100);
    let project = me.projects.get_mut(0).unwrap();
    project.project = 2;
    let template = project.templates.get_mut(0).unwrap();
    template.number = 7;
    template.executor = c::TemplateExecutor::Chosen;
    template.budget = c::TemplateBudget::Given;
    template.deadline = c::TemplateDeadline::None;
    template.flags = c::TaskFlags {
        ended_by: c::EndedBy::Standing,
        conversation: false,
        listed: true,
        retention: c::Retention::Kept,
    };
    template.tracked = None;
    template.label = 3;
    let creating = project.creating.as_mut().unwrap();
    creating.within.charters = List::from_boxed(Box::new([9, 13]));
    me
}
#[test]
fn role_start_fills_the_named_template() {
    for charter in [None, Some("9")] {
        let argv = f::extended(START, if charter.is_some() { &["--charter", "9"] } else { &[] });
        let mut root = f::choice_root(me());
        let call = root.submit(argv.as_slice());
        let c::Ask::Create { project, tasks } = f::act_ask(&root, call) else { panic!("create") };
        assert_eq!(*project, 2);
        assert_eq!(tasks.len(), 1);
        let task = tasks.get(0).unwrap();
        assert_eq!(task.executor, c::Executor::Agent { charter: if charter.is_some() { 9 } else { 13 } });
        assert_eq!(task.spec.words.bytes(), b"Owns the tests");
        assert_eq!(task.contract, c::NewContract::Report { words: crate::limits::TEXT });
        assert_eq!(task.authority.spend, 50);
        assert_eq!(task.budget, c::NewBudget::Named);
        assert_eq!(task.authority.deadline, None);
        assert_eq!(task.deadline, c::NewDeadline::Named);
        assert_eq!(task.tracked, None);
        assert_eq!(task.label, 3);
        assert_eq!(task.flags.ended_by, c::EndedBy::Standing);
        assert!(!task.flags.conversation && task.flags.listed);
        assert_eq!(task.flags.retention, c::Retention::Kept);
        assert!(task.llm.is_none());
        assert_eq!(root.sent_name(call), Some(f::named("k")));
        root.done(call, c::Outcome::Document(c::DocumentOutcome::Created { first: 41, last: 41, given: None }));
        assert_eq!(root.answer(call).unwrap().stdout.as_ref(), b"done: task 41, role\n");
    }
}
fn notes(bodies: &[(u64, &[u8])]) -> c::ReadResult {
    described_notes(b"now", bodies)
}
fn described_notes(description: &[u8], bodies: &[(u64, &[u8])]) -> c::ReadResult {
    let mut rows = List::with_capacity(u32::try_from(bodies.len()).unwrap());
    for (number, body) in bodies {
        rows.push(c::Note {
            number: *number,
            revision: c::Revision { value: 4 },
            scope: c::NoteScope::Task { project: f::PROJECT, task: f::TASK },
            description: f::document_text(description),
            body: c::DocumentText::complete(c::Text::new((*body).into(), crate::limits::LONG_TEXT).unwrap()),
            writer: c::NoteAuthor::Task { task: f::TASK, attempt: 1 },
            tasks: List::with_capacity(0),
            changed: Some(Wall::EPOCH),
            acts: List::with_capacity(0),
            cut: false,
        })
        .unwrap();
    }
    c::ReadResult::Value(c::ReadValue::Notes(Box::new(c::NotePage { rows, cut: false, before: None, after: None })))
}
fn assert_now(root: &f::Harness, call: u32, task: u64) {
    assert_note(root, call, task, b"now");
}
fn assert_note(root: &f::Harness, call: u32, task: u64, description: &[u8]) {
    assert_eq!(
        root.reads.get(&call).unwrap().1,
        c::Read::Notes {
            scope: c::NoteScope::Task { project: f::PROJECT, task },
            description: Some(c::Text::new(description.into(), 6).unwrap()),
            after: None,
        }
    );
}
#[test]
fn role_list_reads_label_three_and_each_now_headline() {
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let argv = f::extended(
            &["role", "list", "--project", "shop", "--after", "1122"],
            if json { &["--json"] } else { &[] },
        );
        let call = root.submit(argv.as_slice());
        assert_eq!(
            root.reads.get(&call).unwrap().1,
            c::Read::Tasks {
                query: c::TaskQuery {
                    project: Some(f::PROJECT),
                    requester: None,
                    root: None,
                    parent: None,
                    label: Some(3),
                    phase: None,
                    executor: None,
                    tracked: None,
                    order: c::QueryOrder::Number
                },
                live: Some(f::cursor()),
                ended: None,
            }
        );
        let mut row = f::task_line();
        row.chip.executor = c::ChipExecutor::Agent { charter: 13 };
        row.chip.phase = c::Phase::Idle;
        row.budget.left = 47;
        let mut second = row.clone();
        second.chip.task = 42;
        let tasks = c::TaskPage {
            live: List::from_boxed(Box::new([row, second])),
            live_after: Some(f::cursor()),
            ended: c::EndedGroup::None,
        };
        root.serve(call, c::ReadResult::Value(c::ReadValue::Tasks(Box::new(tasks))));
        assert_now(&root, call, 41);
        root.serve(call, notes(&[(9, b"Not this later note"), (2, b"Waiting for CI\nThen check the tests")]));
        assert_now(&root, call, 42);
        root.serve(call, notes(&[]));
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 0);
        if json {
            f::unique_keys(&printed.stdout).unwrap();
            assert_eq!(printed.stdout.as_ref(), br#"{"roles":[{"task":41,"charter":13,"state":"idle","left":47,"now":"Waiting for CI"},{"task":42,"charter":13,"state":"idle","left":47,"now":null}],"after":"1122"}"#);
        } else {
            assert_eq!(printed.stdout.as_ref(), b"41 \xc2\xb7 charter 13 \xc2\xb7 idle \xc2\xb7 left 47 \xc2\xb7 Waiting for CI\n42 \xc2\xb7 charter 13 \xc2\xb7 idle \xc2\xb7 left 47 \xc2\xb7 no Now yet\nread on with: role list --project shop --after 1122\n");
        }
    }
}
#[test]
fn role_show_prints_the_task_and_its_whole_now_or_none() {
    let body = "Where things stand\nNext: check the tests.\n".repeat(2500);
    for json in [false, true] {
        for written in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let argv = f::extended(&["role", "show", "41"], if json { &["--json"] } else { &[] });
            let call = root.submit(argv.as_slice());
            assert_eq!(root.reads.get(&call).unwrap().1, c::Read::Task { task: 41 });
            root.serve(call, f::task_result(f::goal(f::TASK, f::waiting_on_me())));
            assert_now(&root, call, 41);
            let rows = [(2, body.as_bytes())];
            root.serve(call, notes(if written { &rows } else { &[] }));
            let printed = root.answer(call).unwrap();
            assert_eq!(printed.status, 0, "{printed:?}");
            if json {
                f::unique_keys(&printed.stdout).unwrap();
                assert!(printed.stdout.starts_with(br#"{"task":"#));
                if written {
                    assert!(f::has(&printed, br#""now":{"number":2,"revision":4,"written":0,"body":"#));
                    let document = crate::jig::json::document_depth(&printed.stdout, 128).unwrap();
                    let mut whole = false;
                    for at in 0..document.len() {
                        whole |= document.text(document.token(at).unwrap()) == Some(body.as_bytes());
                    }
                    assert!(whole);
                } else {
                    assert!(printed.stdout.ends_with(br#","now":null}"#));
                }
            } else {
                assert!(printed.stdout.starts_with(b"task 41"));
                if written {
                    assert!(f::has(&printed, b"Now: note 2 revision 4 written 1970-01-01 00:00 UTC"));
                    assert!(printed.stdout.ends_with(body.as_bytes()));
                } else {
                    assert!(printed.stdout.ends_with(b"no Now yet\n"));
                }
            }
        }
    }
}
