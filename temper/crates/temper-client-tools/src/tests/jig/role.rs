//! Standing roles fill a template and join each task's Now.
use crate::tests::fixtures as f;
use jig_client::contract as c;
use skein_lib::{List, Wall};

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
    let mut rows = List::with_capacity(u32::try_from(bodies.len()).unwrap());
    for (number, body) in bodies {
        rows.push(c::Note {
            number: *number,
            revision: c::Revision { value: 4 },
            scope: c::NoteScope::Task { project: f::PROJECT, task: f::TASK },
            description: f::document_text(b"now"),
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
    assert_eq!(
        root.reads.get(&call).unwrap().1,
        c::Read::Notes {
            scope: c::NoteScope::Task { project: f::PROJECT, task },
            description: Some(c::Text::new(b"now".as_slice().into(), 3).unwrap()),
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
