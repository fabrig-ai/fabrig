//! Project deletion (temper/docs/design/client/ctl/tools.md, 3; results.md, 1).
use crate::{
    jig::{argv::Usage, rows},
    tests::fixtures as f,
};
use jig_client::{Ask, contract as c};
use skein_lib::{Duration, List};

const DELETE: &[&str] = &["project", "delete", "3", "--key", "drop-3"];

fn me(chat: bool, name: &[u8]) -> c::Me {
    let mut me = f::chat_me(f::CHAT_BUDGET);
    let project = me.projects.get_mut(0).expect("fixture project");
    project.project = 3;
    project.name = f::document_text(name);
    if chat {
        project.templates.get_mut(0).expect("chat template").grace = Some(Duration::from_secs(5));
    } else {
        project.templates = List::with_capacity(0);
    }
    me
}

#[test]
fn project_delete_builds_the_ask_from_the_call_alone() {
    for (selector, chat, name, expected) in [
        ("3", true, Some(b"fabrig".as_slice()), b"deleted: project 3, fabrig\n".as_slice()),
        ("fabrig", false, Some(b"fabrig".as_slice()), b"deleted: project 3, fabrig\n".as_slice()),
        ("3", false, Some(b"fabrig".as_slice()), b"deleted: project 3, fabrig\n".as_slice()),
        ("3", true, Some(b"".as_slice()), b"deleted: project 3\n".as_slice()),
        ("3", false, None, b"deleted: project 3\n".as_slice()),
    ] {
        let mut root = f::signed_in(f::PARTY);
        let mut membership = me(chat, name.unwrap_or(b""));
        if name.is_none() {
            membership.projects = List::with_capacity(0);
        }
        root.deliver(c::Delivery::Me(Box::new(membership)));
        let call = root.submit(&["project", "delete", selector, "--key", "drop-3"]);
        assert_eq!(root.sends.len(), 1, "number sent even when absent from me");
        assert_eq!(root.sends[&call].2, Ask::Jig(Box::new(c::Ask::RemoveProject { project: 3, grace: None })));
        assert_eq!(root.sent_name(call), Some(f::named("drop-3")));
        assert!(root.reads.is_empty());
        // A deletion can leave me before its answer arrives; print the call's name.
        let mut gone = me(chat, b"renamed");
        gone.projects = List::with_capacity(0);
        root.deliver(c::Delivery::Me(Box::new(gone)));
        root.done(call, c::Outcome::Document(c::DocumentOutcome::ProjectRemoved { project: 3 }));
        let printed = root.answer(call).expect("deletion answered");
        assert_eq!(printed.status, 0);
        assert_eq!(printed.stdout.as_ref(), expected);
        assert!(printed.stderr.is_empty());
    }
}

#[test]
fn project_delete_needs_a_key_and_a_project() {
    let root = f::signed_in(f::PARTY);
    for (argv, expected) in [
        (&["project", "delete", "3"][..], Usage::Key),
        (
            &["project", "delete", "--key", "drop-3"][..],
            Usage::Missing { tool: rows::PROJECT_DELETE_AT, form: 0, argument: 0 },
        ),
    ] {
        assert_eq!(f::parse(argv), Err(expected));
    }
    assert!(root.sends.is_empty());
}

#[test]
fn an_unknown_project_removal_sends_nothing() {
    let mut root = f::signed_in(f::PARTY);
    root.deliver(c::Delivery::Me(Box::new(me(true, b"fabrig"))));
    let call = root.submit(&["project", "delete", "unknown", "--key", "unknown"]);
    let printed = root.answer(call).expect("unknown project");
    assert_eq!(printed.status, 2);
    assert_eq!(printed.stderr.as_ref(), b"unknown project\n");
    assert!(printed.stdout.is_empty());
    assert!(root.sends.is_empty());
}

#[test]
fn a_refused_project_removal_exits_2() {
    let mut root = f::signed_in(f::PARTY);
    let mut membership = me(true, b"fabrig");
    membership.projects.get_mut(0).expect("fixture project").role = c::HeldRole::Member;
    root.deliver(c::Delivery::Me(Box::new(membership)));
    let call = root.submit(DELETE);
    assert_eq!(root.sends.len(), 1, "jig decides ownership");
    root.answered(call, c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None });
    let printed = root.answer(call).expect("refused removal");
    assert_eq!(printed.status, 2);
    assert!(printed.stdout.is_empty());
    assert_eq!(printed.stderr.as_ref(), b"project delete: This act is not offered\n");
}

#[test]
fn a_live_project_removal_names_its_tasks_and_exits_2() {
    for (more, expected) in [
        (0, b"project delete: This project has live tasks; end them first: 12, 15\n".as_slice()),
        (2, b"project delete: This project has live tasks; end them first: 12, 15, and 2 more\n".as_slice()),
    ] {
        let mut root = f::signed_in(f::PARTY);
        root.deliver(c::Delivery::Me(Box::new(me(true, b"fabrig"))));
        let call = root.submit(DELETE);
        let mut tasks = List::with_capacity(2);
        tasks.push(12).unwrap();
        tasks.push(15).unwrap();
        root.answered(call, c::Answer::Refused { refusal: c::Refusal::LiveTasks { tasks, more }, at: None });
        let printed = root.answer(call).expect("live removal refused");
        assert_eq!(printed.status, 2);
        assert!(printed.stdout.is_empty());
        assert_eq!(printed.stderr.as_ref(), expected);
    }
}

#[test]
fn a_live_project_removal_json_carries_tasks_and_more() {
    let mut root = f::signed_in(f::PARTY);
    root.deliver(c::Delivery::Me(Box::new(me(true, b"fabrig"))));
    let argv = f::extended(DELETE, &["--json"]);
    let call = root.submit(argv.as_slice());
    let mut tasks = List::with_capacity(2);
    tasks.push(12).unwrap();
    tasks.push(15).unwrap();
    root.answered(call, c::Answer::Refused { refusal: c::Refusal::LiveTasks { tasks, more: 2 }, at: None });
    let printed = root.answer(call).expect("JSON live removal refused");
    assert_eq!(printed.status, 2);
    assert_eq!(printed.stderr.as_ref(), br#"{"operation":"remove-project","outcome":{"refused":{"refusal":{"live_tasks":{"tasks":[12,15],"more":2}},"at":null}},"task":null,"to":null}"#);
    assert!(printed.stdout.is_empty());
}

#[test]
fn a_project_removal_json_carries_its_answer_number() {
    let mut root = f::signed_in(f::PARTY);
    root.deliver(c::Delivery::Me(Box::new(me(true, b"fabrig"))));
    let argv = f::extended(DELETE, &["--json"]);
    let call = root.submit(argv.as_slice());
    root.done(call, c::Outcome::Document(c::DocumentOutcome::ProjectRemoved { project: 3 }));
    let printed = root.answer(call).expect("JSON removal");
    assert_eq!(printed.status, 0);
    assert_eq!(printed.stdout.as_ref(), br#"{"operation":"remove-project","outcome":{"done":{"document":{"project_removed":{"project":3}}}},"task":null,"to":null}"#);
    assert!(printed.stderr.is_empty());
}

#[test]
fn project_show_says_a_project_was_deleted() {
    for selector in ["3", "fabrig"] {
        let mut root = f::signed_in(f::PARTY);
        root.deliver(c::Delivery::Me(Box::new(me(false, b"fabrig"))));
        let call = root.submit(&["project", "show", selector]);
        assert_eq!(root.reads[&call].1, c::Read::Project { project: 3 });
        let mut gone = me(false, b"renamed");
        gone.projects = List::with_capacity(0);
        root.deliver(c::Delivery::Me(Box::new(gone)));
        root.serve(call, c::ReadResult::None(c::ReadNone::Gone));
        let printed = root.answer(call).expect("deleted project read");
        assert_eq!(printed.status, 2);
        assert_eq!(printed.stderr.as_ref(), b"project 3 was deleted\n");
        assert!(printed.stdout.is_empty());
    }
}
