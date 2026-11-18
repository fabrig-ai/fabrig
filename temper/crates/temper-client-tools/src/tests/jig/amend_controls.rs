use crate::{
    jig::{
        argv::Usage,
        call::{Value, Values},
    },
    tests::fixtures as f,
};
use jig_client::contract as c;
use skein_lib::{List, Wall};

const REVISION: Option<c::Revision> = Some(c::Revision { value: 7 });
const DEADLINE: Wall = Wall::from_nanos(1_893_553_440_000_000_000); // 2030-01-02T03:04Z.
const DEADLINE_CALL: &[&str] = &["task", "amend", "41", "--rev", "7", "--deadline", "2030-01-02T03:04Z", "because"];
fn task() -> c::Task {
    let mut task = f::choice_task(c::Admission::Allowed);
    task.authority.cut = false;
    task.authority.authority.deadline = None;
    task.spec.cut = false;
    task.spec.specification.words.cut = false;
    task.dependencies = List::with_capacity(1);
    task.dependencies.push(c::TaskLink::Stored { number: 43, status: None }).unwrap();
    task
}
fn person() -> c::Task {
    let mut task = task();
    task.person_revision = Some(c::Revision { value: 23 });
    task.chip.executor = c::ChipExecutor::Party { party: 1 };
    task
}
fn no_change() -> c::Amendment {
    c::Amendment {
        addressed: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        instructions: None,
        tracked: None,
        llm: None,
    }
}
fn expected(change: c::Amendment, revision: Option<c::Revision>) -> c::Ask {
    c::Ask::Amend {
        project: f::PROJECT,
        task: f::TASK,
        revision,
        change: Box::new(change),
        reason: f::act_text(b"because"),
    }
}
fn sends(argv: &[&str], task: c::Task, ask: c::Ask) {
    let (root, call) = f::send_choice_amend(argv, task);
    assert_eq!(f::act_ask(&root, call), &ask);
}
fn refuses(argv: &[&str], task: c::Task, error: &[u8]) {
    let (mut root, call) = f::send_choice_amend(argv, task);
    assert!(root.sends.is_empty());
    let answer = root.answer(call).expect("local refusal");
    assert_eq!((answer.status, answer.stderr.as_ref()), (1, error));
}
fn deadline_change(task: &c::Task, deadline: Option<Wall>) -> c::Amendment {
    let mut authority = task.authority.whole().expect("whole authority");
    authority.deadline = deadline;
    c::Amendment { authority: Some(authority), ..no_change() }
}
#[test]
fn amend_sends_a_person_address_even_where_it_waits() {
    sends(
        &["task", "amend", "41", "--to-party", "1", "because"],
        person(),
        expected(
            c::Amendment {
                addressed: Some(c::AddressChange {
                    revision: c::Revision { value: 23 },
                    address: c::PersonAddress::Party(1),
                }),
                ..no_change()
            },
            REVISION,
        ),
    );
}
#[test]
fn amend_sends_a_role_address() {
    sends(
        &["task", "amend", "41", "--to-role", "2", "because"],
        person(),
        expected(
            c::Amendment {
                addressed: Some(c::AddressChange {
                    revision: c::Revision { value: 23 },
                    address: c::PersonAddress::Role(2),
                }),
                ..no_change()
            },
            REVISION,
        ),
    );
}
#[test]
fn amend_sends_a_keyed_address() {
    sends(
        &["task", "amend", "41", "--to-role", "2", "because", "--key", "pass-up"],
        person(),
        expected(
            c::Amendment {
                addressed: Some(c::AddressChange {
                    revision: c::Revision { value: 23 },
                    address: c::PersonAddress::Role(2),
                }),
                ..no_change()
            },
            REVISION,
        ),
    );
}
#[test]
fn amend_sends_a_priority_beside_an_address_without_rev() {
    sends(
        &["task", "amend", "41", "--priority", "3", "--to-party", "1", "because"],
        person(),
        expected(
            c::Amendment {
                addressed: Some(c::AddressChange {
                    revision: c::Revision { value: 23 },
                    address: c::PersonAddress::Party(1),
                }),
                tracked: Some(c::Tracking::Tracked { priority: 3 }),
                ..no_change()
            },
            REVISION,
        ),
    );
}
#[test]
fn amend_checks_an_address_explicit_spec_revision() {
    refuses(
        &["task", "amend", "41", "--rev", "6", "--to-party", "1", "because"],
        person(),
        b"task 41 moved: its spec revision is 7, not 6; read again: task show 41\n",
    );
}
#[test]
fn amend_refuses_an_address_on_no_person_task() {
    refuses(
        &["task", "amend", "41", "--to-party", "1", "because"],
        task(),
        b"task 41 is not a person task: only a person task takes --to-party or --to-role\n",
    );
}
#[test]
fn amend_refuses_no_person_task_before_other_checks() {
    refuses(
        &["task", "amend", "41", "--rev", "6", "--to-role", "2", "because"],
        task(),
        b"task 41 is not a person task: only a person task takes --to-party or --to-role\n",
    );
}
#[test]
fn amend_renders_a_moved_person_revision_refusal() {
    let (mut root, call) = f::send_choice_amend(&["task", "amend", "41", "--to-party", "1", "because"], person());
    assert_eq!(
        f::act_ask(&root, call),
        &expected(
            c::Amendment {
                addressed: Some(c::AddressChange {
                    revision: c::Revision { value: 23 },
                    address: c::PersonAddress::Party(1)
                }),
                ..no_change()
            },
            REVISION
        )
    );
    root.answered(
        call,
        c::Answer::Refused {
            refusal: c::Refusal::Moved { now: c::Revision { value: 24 }, what: c::Moved::PersonTask },
            at: None,
        },
    );
    let answer = root.answer(call).expect("refusal");
    assert_eq!(
        (answer.status, answer.stderr.as_ref()),
        (2, b"task amend: The object changed\nread again: task show 41\n".as_slice())
    );
}
#[test]
fn amend_address_keeps_other_changes_revision_rules() {
    assert_eq!(
        f::parse(&["task", "amend", "41", "--to-party", "1", "--untracked", "because"]),
        Err(Usage::AmendmentRevision),
    );
}
#[test]
fn amend_role_address_checks_its_range() {
    for role in ["0", "4294967295"] {
        f::parse(&["task", "amend", "41", "--to-role", role, "because"]).expect("role in range");
    }
    assert_eq!(
        f::parse(&["task", "amend", "41", "--to-role", "4294967296", "because"]),
        Err(Usage::Kind { tool: 31, form: 0, argument: 15 })
    );
}
#[test]
fn amend_sends_a_priority_alone_without_a_revision() {
    sends(
        &["task", "amend", "41", "--priority", "3", "because"],
        task(),
        expected(c::Amendment { tracked: Some(c::Tracking::Tracked { priority: 3 }), ..no_change() }, None),
    );
}
#[test]
fn amend_sends_a_priority_with_an_explicit_revision() {
    sends(
        &["task", "amend", "41", "--rev", "7", "--priority", "3", "because"],
        task(),
        expected(c::Amendment { tracked: Some(c::Tracking::Tracked { priority: 3 }), ..no_change() }, REVISION),
    );
}
#[test]
fn amend_needs_a_revision_to_track_an_untracked_task() {
    let mut task = task();
    task.priority = None;
    refuses(&["task", "amend", "41", "--priority", "3", "because"], task, b"task amend needs --rev REV\n");
}
#[test]
fn amend_untracks_at_the_shown_revision() {
    sends(
        &["task", "amend", "41", "--rev", "7", "--untracked", "because"],
        task(),
        expected(c::Amendment { tracked: Some(c::Tracking::Untracked), ..no_change() }, REVISION),
    );
}
#[test]
fn amend_refuses_a_moved_revision_locally() {
    let mut task = task();
    task.spec_revision = c::Revision { value: 9 };
    refuses(DEADLINE_CALL, task, b"task 41 moved: its spec revision is 9, not 7; read again: task show 41\n");
}
#[test]
fn amend_restates_the_authority_with_its_deadline() {
    let task = task();
    let change = deadline_change(&task, Some(DEADLINE));
    sends(DEADLINE_CALL, task, expected(change, REVISION));
}
#[test]
fn amend_removes_the_deadline() {
    let mut task = task();
    task.authority.authority.deadline = Some(DEADLINE);
    let change = deadline_change(&task, None);
    sends(&["task", "amend", "41", "--rev", "7", "--no-deadline", "because"], task, expected(change, REVISION));
}
#[test]
fn amend_refuses_a_deadline_on_a_cut_authority() {
    let mut task = task();
    task.authority.cut = true;
    refuses(
        DEADLINE_CALL,
        task,
        b"task 41's authority is cut in this read; amend its deadline, budget or tasks from a client with wider bounds\n",
    );
}
fn loaded(argv: &[&str], bytes: &[u8], index: usize, task: c::Task, ask: c::Ask) {
    let mut call = f::call(argv);
    call.values[index] = Some(Values::One(Value::Text(bytes.into())));
    let mut root = f::choice_root(f::choice_me(f::CHAT_BUDGET));
    let call = root.submit_prepared(call, false, None);
    root.serve(call, f::task_result(task));
    assert_eq!(f::act_ask(&root, call), &ask);
}
#[test]
fn amend_reads_instructions_from_a_file() {
    // The host supplies the loaded bytes; its file IO is tested by the shell's world.
    loaded(
        &["task", "amend", "41", "--rev", "7", "--instructions", "instructions.txt", "because"],
        b"Run checks",
        6,
        task(),
        expected(c::Amendment { instructions: Some(f::act_text(b"Run checks")), ..no_change() }, REVISION),
    );
}
#[test]
fn amend_reads_instructions_from_stdin() {
    loaded(
        &["task", "amend", "41", "--rev", "7", "--instructions", "-", "because"],
        b"Run checks",
        6,
        task(),
        expected(c::Amendment { instructions: Some(f::act_text(b"Run checks")), ..no_change() }, REVISION),
    );
}
#[test]
fn amend_sends_empty_instructions() {
    let mut task = task();
    task.instructions = Some(f::document_text(b"Run checks"));
    loaded(
        &["task", "amend", "41", "--rev", "7", "--instructions", "-", "because"],
        b"",
        6,
        task,
        expected(c::Amendment { instructions: Some(f::act_text(b"")), ..no_change() }, REVISION),
    );
}
#[test]
fn amend_sets_the_spec_words_keeping_its_parameters() {
    let mut task = task();
    task.spec.specification.parameters = List::with_capacity(1);
    task.spec
        .specification
        .parameters
        .push(c::ParameterDocument::Bytes { name: 1, value: f::document_text(b"debug") })
        .unwrap();
    task.spec.specification.inputs = List::with_capacity(1);
    task.spec.specification.inputs.push(43).unwrap();
    let mut spec = task.spec.whole().expect("whole spec");
    spec.words = f::act_text(b"Build and check");
    loaded(
        &["task", "amend", "41", "--rev", "7", "--spec", "-", "because"],
        b"Build and check",
        7,
        task,
        expected(c::Amendment { spec: Some(spec), ..no_change() }, REVISION),
    );
}
#[test]
fn amend_refuses_words_on_a_cut_spec() {
    let mut task = task();
    task.spec.cut = true;
    let mut call = f::call(&["task", "amend", "41", "--rev", "7", "--spec", "-", "because"]);
    call.values[7] = Some(Values::One(Value::Text(b"Build".as_slice().into())));
    let mut root = f::choice_root(f::choice_me(f::CHAT_BUDGET));
    let call = root.submit_prepared(call, false, None);
    root.serve(call, f::task_result(task));
    assert!(root.sends.is_empty());
    assert_eq!(
        root.answer(call).expect("cut spec").stderr.as_ref(),
        b"task 41's spec is cut in this read; amend its words from a client with wider bounds\n"
    );
}
#[test]
fn amend_sends_the_whole_dependency_list() {
    let mut dependencies = List::with_capacity(2);
    dependencies.push(47).unwrap();
    dependencies.push(43).unwrap();
    sends(
        &["task", "amend", "41", "--rev", "7", "--dependency", "47", "--dependency", "43", "because"],
        task(),
        expected(c::Amendment { dependencies: Some(dependencies), ..no_change() }, REVISION),
    );
}
#[test]
fn amend_removes_every_dependency() {
    sends(
        &["task", "amend", "41", "--rev", "7", "--no-dependencies", "because"],
        task(),
        expected(c::Amendment { dependencies: Some(List::with_capacity(0)), ..no_change() }, REVISION),
    );
}
#[test]
fn amend_proposes_a_deadline_past_the_ceiling() {
    let mut task = task();
    task.authority.authority.deadline = Some(Wall::from_nanos(1));
    let mut ceiling = task.authority.clone();
    ceiling.authority.deadline = Some(Wall::from_nanos(1_893_553_441_000_000_000));
    task.acts.get_mut(0).unwrap().admission = c::Admission::Bounded {
        within: c::Bounds::Amend {
            budget_left: 0,
            grants: List::with_capacity(0),
            implications: List::with_capacity(0),
            authority: Box::new(task.authority.clone()),
            ceiling: Box::new(ceiling),
        },
        beyond: c::Beyond::Proposed { to: c::Recipient::Party(f::RECIPIENT) },
    };
    let change = Box::new(deadline_change(&task, Some(DEADLINE)));
    sends(
        DEADLINE_CALL,
        task,
        c::Ask::Propose {
            project: f::PROJECT,
            action: c::ProposeAction::Amendment { task: f::TASK, revision: c::Revision { value: 7 }, change },
            reason: f::act_text(b"because"),
        },
    );
}
#[test]
fn amend_proposes_a_priority_at_the_base_revision() {
    let mut task = task();
    task.acts.get_mut(0).unwrap().admission = c::Admission::Proposed { to: c::Recipient::Party(f::RECIPIENT) };
    sends(
        &["task", "amend", "41", "--priority", "3", "because"],
        task,
        c::Ask::Propose {
            project: f::PROJECT,
            action: c::ProposeAction::Amendment {
                task: f::TASK,
                revision: c::Revision { value: 7 },
                change: Box::new(c::Amendment { tracked: Some(c::Tracking::Tracked { priority: 3 }), ..no_change() }),
            },
            reason: f::act_text(b"because"),
        },
    );
}
#[test]
fn amend_refuses_both_of_a_pair() {
    for (flags, argument) in [
        (["--priority", "3", "--untracked"].as_slice(), 3),
        (["--deadline", "2030-01-02T03:04Z", "--no-deadline"].as_slice(), 5),
        (["--dependency", "43", "--no-dependencies"].as_slice(), 9),
        (["--to-party", "1", "--to-role", "2"].as_slice(), 15),
    ] {
        let argv = f::extended(&["task", "amend", "41", "--rev", "7", "because"], flags);
        assert_eq!(f::parse(argv.as_slice()), Err(Usage::Twice { tool: 31, form: 0, argument }), "{flags:?}");
    }
}
#[test]
fn amend_refuses_a_malformed_deadline() {
    for deadline in [
        "2030-1-02T03:04Z",
        "2030-02-29T03:04Z",
        "2100-02-29T03:04Z",
        "2030-01-02T24:00Z",
        "2030-01-02T03:60Z",
        "2030-01-02T03:04:00Z",
        "2030-01-02T03:04+00:00",
        "1969-01-01T00:00Z",
        "9999-01-01T00:00Z",
    ] {
        assert_eq!(
            f::parse(&["task", "amend", "41", "--rev", "7", "--deadline", deadline, "because"]),
            Err(Usage::Kind { tool: 31, form: 0, argument: 4 })
        );
    }
}
#[test]
fn amend_changes_nothing_is_refused() {
    refuses(&["task", "amend", "41", "--priority", "2", "because"], task(), b"amend changes nothing on task 41\n");
}
#[test]
fn the_old_spec_revision_form_asks_for_rev() {
    assert_eq!(
        f::parse(&["task", "amend", "41", "--spec", "8", "--model", "chosen-model", "because"]),
        Err(Usage::AmendmentRevision)
    );
}
#[test]
fn amend_refuses_a_repeated_flag() {
    for (flags, argument) in [
        (["--priority", "3", "--priority", "4"].as_slice(), 2),
        (["--untracked", "--untracked"].as_slice(), 3),
        (["--instructions", "one", "--instructions", "two"].as_slice(), 6),
        (["--model", "chosen-model", "--model", "plain-model"].as_slice(), 10),
        (["--to-party", "1", "--to-party", "2"].as_slice(), 14),
        (["--to-role", "1", "--to-role", "2"].as_slice(), 15),
    ] {
        let argv = f::extended(&["task", "amend", "41", "--rev", "7", "because"], flags);
        assert_eq!(f::parse(argv.as_slice()), Err(Usage::Twice { tool: 31, form: 0, argument }), "{flags:?}");
    }
}
#[test]
fn amend_uses_stdin_only_once() {
    assert_eq!(
        f::parse(&["task", "amend", "41", "--rev", "7", "--instructions", "-", "--spec", "-", "because"]),
        Err(Usage::Stdin)
    );
}
#[test]
fn amend_requires_a_model_for_an_effort() {
    assert_eq!(
        f::parse(&["task", "amend", "41", "--rev", "7", "--effort", "high", "because"]),
        Err(Usage::Missing { tool: 31, form: 0, argument: 10 })
    );
}
#[test]
fn amend_json_carries_texts_inline_including_a_dash() {
    let call = f::from_json(
        br#"{"tool":"task amend","arguments":{"task":41,"rev":7,"instructions":"-","spec":"Build","reason":"because"}}"#,
    );
    assert_eq!(call.values[6], Some(Values::One(Value::Text(b"-".as_slice().into()))));
    assert_eq!(f::from_json(&crate::jig::json::write_call(crate::table::TOOLS, &call).unwrap()), call);
}

#[test]
fn amend_deadlines_parse_epoch_and_leap_dates() {
    for (text, nanos) in [(b"1970-01-01T00:00Z".as_slice(), 0), (b"2000-02-29T00:00Z", 951_782_400_000_000_000)] {
        assert_eq!(crate::jig::argv::deadline(text), Some(Wall::from_nanos(nanos)));
    }
}

fn dependencies_at(count: u32) -> List<&'static str> {
    let mut argv = f::extended(&["task", "amend", "41", "--rev", "7", "because"], &[]);
    for _ in 0..count {
        argv.push("--dependency").expect("fixture word bound");
        argv.push("43").expect("fixture word bound");
    }
    argv
}
#[test]
fn amend_accepts_dependencies_at_the_argument_bound() {
    f::parse(dependencies_at(crate::limits::ROWS).as_slice()).expect("dependencies at the argument bound");
}
#[test]
fn amend_refuses_dependencies_past_the_argument_bound() {
    assert_eq!(
        f::parse(dependencies_at(crate::limits::ROWS + 1).as_slice()),
        Err(Usage::Many { tool: 31, form: 0, argument: 8 })
    );
}

#[test]
fn amend_needs_a_revision_for_any_other_change() {
    for flags in [
        ["--untracked"].as_slice(),
        &["--deadline", "2030-01-02T03:04Z"],
        &["--no-deadline"],
        &["--instructions", "missing-file"],
        &["--spec", "8"],
        &["--dependency", "43"],
        &["--no-dependencies"],
        &["--model", "chosen-model"],
    ] {
        let argv = f::extended(&["task", "amend", "41", "because"], flags);
        assert_eq!(f::parse(argv.as_slice()), Err(Usage::AmendmentRevision), "{flags:?}");
    }
}

#[test]
fn amend_sends_and_answers_a_replacement_when_only_the_old_words_are_cut() {
    let mut task = task();
    let mut spec = task.spec.whole().unwrap();
    spec.words = f::act_text(b"Build and check");
    task.spec.specification.words.cut = true;
    let mut call = f::call(&["task", "amend", "41", "--rev", "7", "--spec", "replacement.txt", "because"]);
    call.values[7] = Some(Values::One(Value::Text(b"Build and check".as_slice().into())));
    let mut root = f::choice_root(f::choice_me(f::CHAT_BUDGET));
    let call = root.submit_prepared(call, false, None);
    root.serve(call, f::task_result(task));
    assert_eq!(f::act_ask(&root, call), &expected(c::Amendment { spec: Some(spec), ..no_change() }, REVISION));
    root.done(call, c::Outcome::Document(c::DocumentOutcome::Amended { task: f::TASK }));
    let answer = root.answer(call).expect("engine answered the replacement");
    assert_eq!(answer.status, 0);
}

const BUDGET_CALL: &[&str] = &["task", "amend", "41", "--rev", "7", "--budget", "20", "because"];

fn budget_task(budget_left: u64, held: u64, ceiling: u64) -> c::Task {
    let mut task = task();
    task.authority.authority.spend = 5;
    let mut authority = task.authority.clone();
    authority.authority.spend = held;
    let mut ceiling_authority = task.authority.clone();
    ceiling_authority.authority.spend = ceiling;
    task.acts.get_mut(0).unwrap().admission = c::Admission::Bounded {
        within: c::Bounds::Amend {
            budget_left,
            grants: List::with_capacity(0),
            implications: List::with_capacity(0),
            authority: Box::new(authority),
            ceiling: Box::new(ceiling_authority),
        },
        beyond: c::Beyond::Proposed { to: c::Recipient::Party(f::RECIPIENT) },
    };
    task
}
fn budget_change(task: &c::Task, spend: u64) -> c::Amendment {
    let mut authority = task.authority.whole().expect("whole authority");
    authority.spend = spend;
    c::Amendment { authority: Some(authority), ..no_change() }
}

#[test]
fn amend_budget_restates_the_authority_in_ctls_amount_within_bounds() {
    for (amount, spend) in [("20", 2000), ("20.5", 2050), ("0.07", 7)] {
        let task = budget_task(spend - 5, spend, spend);
        let change = budget_change(&task, spend);
        sends(&["task", "amend", "41", "--rev", "7", "--budget", amount, "because"], task, expected(change, REVISION));
    }
}

#[test]
fn amend_budget_proposes_a_raise_past_the_partys_bounds() {
    for task in [budget_task(0, 2000, 2000), budget_task(1995, 1999, 2000)] {
        let change = Box::new(budget_change(&task, 2000));
        sends(
            BUDGET_CALL,
            task,
            c::Ask::Propose {
                project: f::PROJECT,
                action: c::ProposeAction::Amendment { task: f::TASK, revision: c::Revision { value: 7 }, change },
                reason: f::act_text(b"because"),
            },
        );
    }
}

#[test]
fn amend_budget_refuses_a_raise_past_the_ceiling() {
    refuses(BUDGET_CALL, budget_task(1995, 2000, 1999), b"amendment not offered\n");
}

#[test]
fn amend_budget_refuses_a_malformed_amount() {
    for amount in ["1.005", "1.", ".5", "-1", "abc", "1,5", "184467440737095516.16"] {
        assert_eq!(
            f::parse(&["task", "amend", "41", "--rev", "7", "--budget", amount, "because"]),
            Err(Usage::Kind { tool: 31, form: 0, argument: 16 }),
            "{amount}",
        );
    }
}

#[test]
fn amend_budget_needs_a_whole_authority_and_the_shown_revision() {
    let mut cut = task();
    cut.authority.cut = true;
    refuses(
        BUDGET_CALL,
        cut,
        b"task 41's authority is cut in this read; amend its deadline, budget or tasks from a client with wider bounds\n",
    );
    let mut moved = task();
    moved.spec_revision = c::Revision { value: 9 };
    refuses(BUDGET_CALL, moved, b"task 41 moved: its spec revision is 9, not 7; read again: task show 41\n");
    assert_eq!(f::parse(&["task", "amend", "41", "--budget", "20", "because"]), Err(Usage::AmendmentRevision));
}

#[test]
fn amend_budget_combines_with_a_deadline_and_tasks_on_both_paths() {
    let argv = [
        "task",
        "amend",
        "41",
        "--rev",
        "7",
        "--budget",
        "20",
        "--deadline",
        "2030-01-02T03:04Z",
        "--tasks",
        "12",
        "because",
    ];
    let keyed = f::extended(&argv, &["--key", "more-budget"]);
    for argv in [argv.as_slice(), keyed.as_slice()] {
        let task = task();
        let mut authority = task.authority.whole().expect("whole authority");
        authority.spend = 2000;
        authority.deadline = Some(DEADLINE);
        authority.delegation.tasks = 12;
        sends(argv, task, expected(c::Amendment { authority: Some(authority), ..no_change() }, REVISION));
    }
}

#[test]
fn amend_budget_sends_a_narrowing_without_room_for_a_raise() {
    let mut task = budget_task(0, 0, 0);
    task.authority.authority.spend = 2001;
    let change = budget_change(&task, 2000);
    sends(BUDGET_CALL, task, expected(change, REVISION));
}

#[test]
fn amend_budget_compares_away_an_unchanged_authority_unless_keyed() {
    let mut task = task();
    task.authority.authority.spend = 2000;
    refuses(BUDGET_CALL, task.clone(), b"amend changes nothing on task 41\n");
    let argv = f::extended(BUDGET_CALL, &["--key", "same-budget"]);
    let change = budget_change(&task, 2000);
    sends(argv.as_slice(), task, expected(change, REVISION));
}

#[test]
fn amend_budget_json_keeps_spend_units_and_accepts_no_budget() {
    let call =
        f::from_json(br#"{"tool":"task amend","arguments":{"task":41,"rev":7,"budget":2050,"reason":"because"}}"#);
    assert_eq!(call.values[16], Some(Values::One(Value::Number(2050))));
    assert_eq!(f::from_json(&crate::jig::json::write_call(crate::table::TOOLS, &call).unwrap()), call);
    let old = f::from_json(br#"{"tool":"task amend","arguments":{"task":41,"rev":7,"tasks":12,"reason":"because"}}"#);
    assert_eq!(old.values[13], Some(Values::One(Value::Number(12))));
    assert_eq!(old.values[16], None);
}

const TASKS_CALL: &[&str] = &["task", "amend", "41", "--rev", "7", "--tasks", "12", "because"];

#[test]
fn amend_tasks_preserves_the_rest_of_the_authority_and_combines_with_a_deadline() {
    let argv = [
        "task",
        "amend",
        "41",
        "--rev",
        "7",
        "--deadline",
        "2030-01-02T03:04Z",
        "because",
        "--tasks",
        "12",
        "--key",
        "more-tasks",
    ];
    for argv in [&argv[..10], &argv[..]] {
        let task = task();
        let mut authority = task.authority.whole().expect("whole authority");
        authority.delegation.tasks = 12;
        authority.deadline = Some(DEADLINE);
        sends(argv, task, expected(c::Amendment { authority: Some(authority), ..no_change() }, REVISION));
    }
}

#[test]
fn amend_tasks_needs_a_whole_authority_and_the_shown_revision() {
    let mut cut = task();
    cut.authority.cut = true;
    refuses(
        TASKS_CALL,
        cut,
        b"task 41's authority is cut in this read; amend its deadline, budget or tasks from a client with wider bounds\n",
    );
    let mut moved = task();
    moved.spec_revision = c::Revision { value: 9 };
    refuses(TASKS_CALL, moved, b"task 41 moved: its spec revision is 9, not 7; read again: task show 41\n");
    assert_eq!(f::parse(&["task", "amend", "41", "--tasks", "12", "because"]), Err(Usage::AmendmentRevision));
    assert_eq!(
        f::parse(&["task", "amend", "41", "--rev", "7", "--tasks", "4294967296", "because"]),
        Err(Usage::Kind { tool: 31, form: 0, argument: 13 })
    );
}
