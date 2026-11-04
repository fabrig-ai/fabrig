//! Visibility and public paths (temper/docs/design/client/tools.md, 5.2 and 5.3).
use crate::{jig, tests::fixtures as f};
use jig_client::{Ask, contract as c};

const AMEND: &[&str] = &["project", "amend", "shop", "--visibility", "public", "--key", "visibility"];
// The design's example exercises every hexadecimal digit position.
const PUBLIC: u64 = 0x3f2a_9c01_d4e5_b6a7;
// Leading zeros must survive public identifier formatting.
const PADDED: u64 = 0x00ab_0000_0000_0001;

fn me(public: Option<u64>) -> c::Me {
    let mut me = f::me(f::PARTY);
    me.projects.get_mut(0).expect("fixture project").public = public;
    me
}
fn deliver(root: &mut f::Harness, public: Option<u64>) {
    root.deliver(c::Delivery::Me(Box::new(me(public))));
}
fn done(root: &mut f::Harness, call: u32, public: bool) {
    root.done(call, c::Outcome::Document(c::DocumentOutcome::VisibilitySet { project: f::PROJECT, public }));
}
fn amended(argv: &[&str], public: Option<u64>) -> jig::Printed {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(argv);
    done(&mut root, call, public.is_some());
    deliver(&mut root, public);
    root.answer(call).expect("visibility agrees")
}
#[test]
fn project_amend_sends_set_visibility_under_its_key() {
    for (project, visibility, public) in
        [("1", "public", true), ("shop", "public", true), ("1", "private", false), ("shop", "private", false)]
    {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["project", "amend", project, "--visibility", visibility, "--key", "visibility"]);
        assert_eq!(root.sends[&call].2, Ask::Jig(Box::new(c::Ask::SetVisibility { project: f::PROJECT, public })));
        assert_eq!(root.sent_name(call), Some(f::named("visibility")));
        assert!(root.reads.is_empty());
    }
}
#[test]
fn project_amend_needs_a_key_and_a_visibility() {
    let root = f::signed_in(f::PARTY);
    for argv in [
        &["project", "amend", "shop", "--visibility", "public"][..],
        &["project", "amend", "shop", "--key", "visibility"][..],
        &["project", "amend", "shop", "--visibility", "open", "--key", "visibility"][..],
    ] {
        f::parse(argv).expect_err("invalid visibility call is usage");
    }
    assert!(root.sends.is_empty());
}
#[test]
fn made_public_prints_the_page_from_me() {
    let printed = amended(AMEND, Some(PADDED));
    assert_eq!(printed.status, 0);
    assert_eq!(printed.stdout.as_ref(), b"project 1: public\npage: /public/projects/00ab000000000001\n");
    assert!(printed.stderr.is_empty());
}
#[test]
fn the_page_waits_for_me_to_agree() {
    for (flags, golden) in [(&[][..], "project_amend"), (&["--json"][..], "project_amend_json")] {
        let mut root = f::signed_in(f::PARTY);
        let argv = f::extended(AMEND, flags);
        let call = root.submit(argv.as_slice());
        done(&mut root, call, true);
        assert!(root.answer(call).is_none(), "the durable answer alone carries no public identifier");
        deliver(&mut root, None);
        assert!(root.answer(call).is_none(), "a changed me must agree with the answer");
        deliver(&mut root, Some(PUBLIC));
        let printed = root.answer(call).expect("me carries the page");
        assert_eq!(printed.status, 0);
        f::check_golden(golden, &printed.stdout);
        deliver(&mut root, Some(PADDED));
        assert!(root.answer(call).is_none(), "the call prints once");
    }
}
#[test]
fn made_private_prints_no_page() {
    for (flags, expected) in [
        (&[][..], b"project 1: private\n".as_slice()),
        (&["--json"][..], br#"{"answer":{"project":1,"public":false,"page":null}}"#.as_slice()),
    ] {
        let mut root = f::signed_in(f::PARTY);
        deliver(&mut root, Some(PUBLIC));
        let argv = f::extended(&["project", "amend", "shop", "--visibility", "private", "--key", "visibility"], flags);
        let call = root.submit(argv.as_slice());
        done(&mut root, call, false);
        assert!(root.answer(call).is_none(), "the stale public page cannot print");
        deliver(&mut root, None);
        let printed = root.answer(call).expect("me agrees it is private");
        assert_eq!(printed.status, 0);
        assert_eq!(printed.stdout.as_ref(), expected);
    }
}
#[test]
fn a_refused_visibility_exits_2() {
    let mut root = f::signed_in(f::PARTY);
    let mut membership = me(None);
    membership.projects.get_mut(0).expect("fixture project").role = c::HeldRole::Member;
    root.deliver(c::Delivery::Me(Box::new(membership)));
    let call = root.submit(AMEND);
    root.answered(call, c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None });
    let printed = root.answer(call).expect("refusal needs no me change");
    assert_eq!(printed.status, 2);
    assert!(printed.stdout.is_empty());
    assert!(!printed.stderr.is_empty());
}
#[test]
fn project_show_says_whether_it_is_public() {
    for (public, flags, cut, golden) in [
        (None, &[][..], false, "project_show"),
        (None, &["--json"][..], true, "project_show_json"),
        (Some(PUBLIC), &[][..], false, "project_show_public"),
        (Some(PUBLIC), &["--json"][..], true, "project_show_public_json"),
    ] {
        let mut root = f::signed_in(f::PARTY);
        deliver(&mut root, public);
        let argv = f::extended(&["project", "show", "shop"], flags);
        let call = root.submit(argv.as_slice());
        let mut project = super::projects::project();
        project.cut = cut;
        project.configured_pools_cut = cut;
        root.serve(call, c::ReadResult::Value(c::ReadValue::Project(Box::new(project))));
        let printed = root.answer(call).expect("project read");
        assert_eq!(printed.status, 0);
        f::check_golden(golden, &printed.stdout);
    }
}
#[test]
fn an_unknown_project_sends_nothing() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["project", "amend", "unknown", "--visibility", "public", "--key", "visibility"]);
    let printed = root.answer(call).expect("unknown project");
    assert_eq!(printed.status, 1);
    assert_eq!(printed.stderr.as_ref(), b"unknown project\n");
    assert!(root.sends.is_empty());
}
#[test]
fn a_replayed_visibility_prints_the_first_answer() {
    let mut root = f::signed_in(f::PARTY);
    let first = root.submit(AMEND);
    done(&mut root, first, true);
    deliver(&mut root, Some(PUBLIC));
    let original = root.answer(first).expect("first answer");
    let replay = root.submit(AMEND);
    done(&mut root, replay, true);
    let repeated = root.answer(replay).expect("me already agrees");
    assert_eq!(root.sent_name(first), root.sent_name(replay));
    assert_eq!(repeated.status, 0);
    assert_eq!(repeated.stdout, original.stdout);
}
