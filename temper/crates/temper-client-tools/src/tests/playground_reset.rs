//! The ctl removals (temper/docs/design/draft/playground-reset.md, 4 and 6.2).
use super::fixtures as f;
use crate::{demo, jig};
use jig_client::{self as client, contract as c};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn request(root: &f::Harness, call: u32) -> w::Request {
    assert_eq!(root.sends.len(), 1);
    let client::Ask::Family { scope, record } = root.adoption_bytes(call) else { panic!("family request") };
    assert_eq!(*scope, c::Scope::Deployment);
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
}
fn printed(argv: &[&str], outcome: w::Outcome, text: &[u8], json: &[u8], status: u8) {
    for as_json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit_prepared(f::call(argv), as_json, None);
        assert!(root.answer(call).is_none(), "wait for the engine's answer");
        root.adoption_answer(call, outcome.clone());
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, status);
        let (bytes, empty) =
            if status == 0 { (&answer.stdout, &answer.stderr) } else { (&answer.stderr, &answer.stdout) };
        assert!(empty.is_empty());
        assert_eq!(bytes.as_ref(), if as_json { json } else { text });
    }
}
#[test]
fn playground_reset_sends_end_playground_and_prints_reset() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["playground", "reset", "--key", "reset"]);
    assert_eq!(request(&root, call), w::Request::EndPlayground);
    assert_eq!(root.sent_name(call), Some(crate::key::engine(&f::name(b"reset"))));
    printed(&["playground", "reset", "--key", "reset"], w::Outcome::Reset, b"reset\n", br#"{"reset":{}}"#, 0);
}
#[test]
fn playground_reset_refused_is_exit_two() {
    printed(
        &["playground", "reset"],
        w::Outcome::EndRefused(
            w::EndRefused::new(&w::CEILINGS, w::EndRefusedParts { why: w::EndRefusal::NotOffered }).unwrap(),
        ),
        b"refused: not offered\n",
        br#"{"end_refused":{"why":"not_offered"}}"#,
        2,
    );
}
#[test]
fn forge_remove_sends_remove_forge_and_prints_removing() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["forge", "remove", "3"]);
    let w::Request::RemoveForge(value) = request(&root, call) else { panic!("remove forge") };
    assert_eq!(value.forge(), 3);
    printed(&["forge", "remove", "3"], w::Outcome::Removing, b"removing\n", br#"{"removing":{}}"#, 0);
}
#[test]
fn forge_remove_refusals_say_why() {
    for (why, text, json) in [
        (
            w::RemoveRefusal::NotOffered,
            b"refused: not offered\n".as_slice(),
            br#"{"remove_refused":{"why":"not_offered"}}"#.as_slice(),
        ),
        (w::RemoveRefusal::NoSuchForge, b"refused: no such forge\n", br#"{"remove_refused":{"why":"no_such_forge"}}"#),
        (w::RemoveRefusal::Role, b"refused: not an owner\n", br#"{"remove_refused":{"why":"role"}}"#),
    ] {
        printed(
            &["forge", "remove", "3", "--key", "remove"],
            w::Outcome::RemoveRefused(w::RemoveRefused::new(&w::CEILINGS, w::RemoveRefusedParts { why }).unwrap()),
            text,
            json,
            2,
        );
    }
}
#[test]
fn forge_setup_without_a_forge_sends_set_up() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["forge", "setup"]);
    let w::Request::SetUp(value) = request(&root, call) else { panic!("set up") };
    assert_eq!((value.org(), value.repository()), (b"work".as_slice(), b"hello".as_slice()));
    for argv in [&["forge", "setup", "1"][..], &["forge", "setup", "65535"]] {
        let mut other = f::signed_in(f::PARTY);
        let other_call = other.submit(argv);
        assert_eq!(root.adoption_bytes(call), other.adoption_bytes(other_call), "the positional is unused");
    }
}
#[test]
fn removals_keep_drawn_keys_and_recover_their_own_answers() {
    for (argv, outcome, text, json) in [
        (
            &["playground", "reset"][..],
            w::Outcome::Reset,
            b"kept: playground-reset reset\n".as_slice(),
            b"{\"operation\":\"playground-reset\",\"outcome\":{\"reset\":{}}}\n".as_slice(),
        ),
        (
            &["forge", "remove", "3"],
            w::Outcome::Removing,
            b"kept: forge-remove removing\n",
            b"{\"operation\":\"forge-remove\",\"outcome\":{\"removing\":{}}}\n",
        ),
    ] {
        for as_json in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let call = root.submit(argv);
            assert!(root.saved.contains_key(&call));
            let mut records = f::restored_adoption(f::PARTY);
            let record = records.get_mut(0).unwrap();
            record.key = root.sent_name(call).unwrap();
            record.request.request = root.adoption_bytes(call).clone();
            let mut root = f::started_with(records, f::SEED, as_json);
            root.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
            root.adoption_answer(0, outcome.clone());
            assert_eq!(root.kept_lines.as_slice(), &[Box::<[u8]>::from(if as_json { json } else { text })]);
        }
    }
}
#[test]
fn unknown_and_unrelated_answers_are_unknown_and_result_limits_hold() {
    for outcome in [w::Outcome::Unknown, w::Outcome::Running] {
        for argv in [&["playground", "reset"][..], &["forge", "remove", "3"]] {
            printed(argv, outcome.clone(), b"unknown\n", br#"{"unknown":{}}"#, 1);
        }
    }
    for as_json in [false, true] {
        for kept in [false, true] {
            let mut limits = f::limits_root();
            limits.tools.text = 1;
            let reset = client::Outcome::Answered(f::family_answer(w::Outcome::Reset));
            let remove = client::Outcome::Answered(f::family_answer(w::Outcome::Removing));
            assert_eq!(demo::print_reset(&reset, as_json, kept, &limits).status, 1);
            assert_eq!(demo::print_remove(&remove, as_json, kept, &limits).status, 1);
        }
    }
    let printed = demo::print_reset(&client::Outcome::Unknown, false, false, &f::limits_root());
    assert_eq!((printed.status, printed.stderr.as_ref()), (1, b"unknown\n".as_slice()));
    assert_eq!(printed.file, jig::FileEnd::Keep);
}
