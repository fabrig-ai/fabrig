//! What the root refuses or answers at the entrance, and how a start comes
//! back decoded.

use super::fixtures::{ACCOUNT, ACCOUNT_RESET, assignment, fields, items, refused};
use super::harness::Harness;
use crate::{Event, Request, agent, host, translate};
use alloc::boxed::Box;
use skein_lib::Duration;

#[test]
fn workspace_items_that_did_not_decode_are_refused_naming_them() {
    let mut h = Harness::new();
    let assign = Event::Assign { assignment: assignment(true, Box::new([])), items: Err(crate::wire::Undecodable) };
    assert_eq!(&*h.step(assign), [refused(b"workspace items")]);
    assert_eq!(h.domain.host().hosted(), 0);
}

#[test]
fn workspace_items_are_refused_naming_the_item_or_carry_their_program_to_checkout() {
    let mut bound = Box::new([b'x'; 4096]);
    *bound.first_mut().expect("nonempty program") = b'/';
    let mut long = Box::new([b'x'; 4097]);
    *long.first_mut().expect("nonempty program") = b'/';
    for (row, program, duplicate, why) in [
        ("duplicate", None, true, Some(b"repository named twice".as_slice())),
        ("relative", Some(b"git".as_slice()), false, Some(b"repository git".as_slice())),
        ("past bound", Some(long.as_slice()), false, Some(b"repository git".as_slice())),
        ("NUL", Some(b"/bin/g\0it".as_slice()), false, Some(b"repository git".as_slice())),
        ("absolute", Some(b"/opt/fake/git".as_slice()), false, None),
        ("at bound", Some(bound.as_slice()), false, None),
        ("none", None, false, None),
    ] {
        let mut h = Harness::new();
        let mut workspace = if duplicate { items(&[b"app", b"app"]) } else { items(&[b"app"]) };
        workspace.repositories.first_mut().expect("one repository").git = match program {
            Some(program) => Some(Box::from(program)),
            None => None,
        };
        if let Some(why) = why {
            let assign = Event::Assign { assignment: assignment(true, Box::new([])), items: Ok(workspace) };
            assert_eq!(&*h.step(assign), [refused(why)], "{row}");
            assert_eq!(h.domain.host().hosted(), 0, "{row}");
        } else {
            assert_eq!(crate::assignment::check(&workspace, &h.env.limits), Ok(()), "{row}");
            let spec = translate::spec(7, workspace);
            assert_eq!(
                spec.repositories.first().expect("one repository").git.as_deref(),
                program,
                "{row}: program reaches checkout"
            );
        }
    }
}

#[test]
fn workspace_items_without_a_workstream_are_refused_as_an_invalid_start() {
    let mut h = Harness::new();
    let assign = Event::Assign { assignment: assignment(false, Box::new([])), items: Ok(items(&[b"app"])) };
    assert_eq!(&*h.step(assign), [refused(b"workstream")]);
}

#[test]
fn an_undecodable_call_name_refuses_the_start_as_invalid_naming_it() {
    let mut h = Harness::new();
    let (agent, _) = h.decode(Box::new([]));
    let rows = h.step(Event::Decoded { agent, start: Err(host::Undecoded::CallName) });
    assert!(rows.contains(&refused(b"call name")));
}

#[test]
fn undecodable_delivery_evidence_refuses_the_start_as_invalid_naming_it() {
    let mut h = Harness::new();
    let (agent, _) = h.decode(Box::new([]));
    let rows = h.step(Event::Decoded { agent, start: Err(host::Undecoded::Evidence) });
    assert!(rows.contains(&refused(b"delivery evidence")));
}

#[test]
fn a_start_refused_as_invalid_frees_its_slot_unacknowledged() {
    let mut h = Harness::new();
    let (agent, _) = h.decode(Box::new([]));
    h.step(Event::Decoded { agent, start: Err(host::Undecoded::Charter) });
    assert_eq!(h.domain.host().hosted(), 0);
}

#[test]
fn an_assignment_hosted_already_is_a_wire_retry_and_dropped() {
    let mut h = Harness::new();
    h.decode(Box::new([]));
    let again = Event::Link(host::LinkDown::Assign { assignment: assignment(false, Box::new([])) });
    assert!(h.step(again).is_empty());
}

#[test]
fn a_worker_shutting_down_refuses_assignments_as_busy() {
    let mut h = Harness::new();
    h.step(Event::Shutdown { drain: true });
    let rows = h.step(Event::Link(host::LinkDown::Assign { assignment: assignment(false, Box::new([])) }));
    let [Request::Answer { answer, .. }] = &*rows else { panic!("one answer: {rows:?}") };
    assert_eq!(answer.ending, host::Ending::Refused(host::Refusal::Busy));
}

#[test]
fn a_worker_shutting_down_still_says_its_slots() {
    let mut h = Harness::new();
    h.step(Event::Shutdown { drain: true });
    let rows = h.step(Event::Ready);
    let [Request::Hello(hello), Request::Link(host::LinkUp::Draining)] = &*rows else {
        panic!("hello and draining: {rows:?}");
    };
    assert_eq!(hello.slots, 2);
}

#[test]
fn the_hello_states_the_configured_stop_bound_whole() {
    let mut h = Harness::new();
    let rows = h.step(Event::Ready);
    let [Request::Hello(hello)] = &*rows else { panic!("one hello: {rows:?}") };
    assert_eq!(hello.stop_bound, Duration::from_secs(20));
}

#[test]
fn a_worker_told_to_shut_down_with_nothing_left_says_stop_once() {
    let mut h = Harness::new();
    assert_eq!(&*h.step(Event::Shutdown { drain: true }), [Request::Link(host::LinkUp::Draining), Request::Stop]);
    assert!(h.step(Event::Lost).is_empty());
}

#[test]
fn an_exhausted_account_stays_a_failure_of_its_own() {
    let exhausted = agent::ModelFault::Limited { account: ACCOUNT, reset: Some(ACCOUNT_RESET) };
    assert_eq!(
        translate::run_failure(&agent::RunFailure::Model(exhausted)),
        host::RunFailure::Limited { account: ACCOUNT, reset: Some(ACCOUNT_RESET) }
    );
}

#[test]
fn a_delivery_is_exactly_a_title_and_a_body() {
    let both = fields(&[(b"title", b"a title"), (b"body", b"a body")]);
    let delivered = crate::smith_delivery::fields(both, 512).expect("a commit message");
    assert_eq!((&*delivered.title, &*delivered.body), (&b"a title"[..], &b"a body"[..]));
}

#[test]
fn a_landed_receipt_says_pushed_at_which_head_on_which_branch() {
    let delivery = crate::smith_delivery::landings(
        &[crate::checkout::Landing::Landed { commit: crate::checkout::git::Commit::new([0xab; 32]) }],
        &[Some(Box::from(b"topic".as_slice()))],
        false,
    );
    let agent::Delivery::Delivered(delivered) = delivery else { panic!("landed delivery: {delivery:?}") };
    assert_eq!(delivered.receipts().len(), 1);
    assert_eq!(delivered.receipts()[0].directory(), 0);
    assert_eq!(
        delivered.receipts()[0].text(),
        b"pushed abababababababababababababababababababababababababababababababab to branch topic; your checkout stays detached, as expected"
    );
}

#[test]
fn a_sha1_receipt_spells_the_head_in_40_digits() {
    let mut commit = [0; 32];
    commit[..20].fill(0xab);
    let delivery = crate::smith_delivery::landings(
        &[crate::checkout::Landing::Landed { commit: crate::checkout::git::Commit::new(commit) }],
        &[Some(Box::from(b"topic".as_slice()))],
        false,
    );
    let agent::Delivery::Delivered(delivered) = delivery else { panic!("landed delivery: {delivery:?}") };
    assert_eq!(
        delivered.receipts()[0].text(),
        b"pushed abababababababababababababababababababab to branch topic; your checkout stays detached, as expected"
    );
}

#[test]
fn a_branch_past_the_receipts_bound_leaves_its_words_out() {
    const SHA256: &str = "abababababababababababababababababababababababababababababababab";
    const SHA1: &str = "abababababababababababababababababababab";
    const DETACHED: &str = "; your checkout stays detached, as expected";
    let mut sha1 = [0; 32];
    sha1[..20].fill(0xab);
    let sha256_room =
        agent::Receipt::CAPACITY - b"pushed ".len() - SHA256.len() - DETACHED.len() - b" to branch ".len();
    let sha1_room = sha256_room + 24; // SHA-1 leaves out twelve padding bytes, or 24 digits.
    for (commit, digits, length, included) in [
        ([0xab; 32], SHA256, sha256_room, true),
        ([0xab; 32], SHA256, sha256_room + 1, false),
        (sha1, SHA1, sha1_room, true),
        (sha1, SHA1, sha1_room + 1, false),
    ] {
        let branch = "b".repeat(length);
        let delivery = crate::smith_delivery::landings(
            &[crate::checkout::Landing::Landed { commit: crate::checkout::git::Commit::new(commit) }],
            &[Some(Box::from(branch.as_bytes()))],
            false,
        );
        let agent::Delivery::Delivered(delivered) = delivery else { panic!("landed delivery: {delivery:?}") };
        let len = if included { agent::Receipt::CAPACITY } else { b"pushed ".len() + digits.len() + DETACHED.len() };
        let mut expected = skein_lib::Writer::new(len);
        expected.put(b"pushed ").expect("receipt prefix");
        expected.put(digits.as_bytes()).expect("head digits");
        if included {
            expected.put(b" to branch ").expect("branch prefix");
            expected.put(branch.as_bytes()).expect("branch at bound");
        }
        expected.put(DETACHED.as_bytes()).expect("receipt suffix");
        assert_eq!(
            delivered.receipts()[0].text(),
            expected.finish().as_ref(),
            "head digits: {}; branch bytes: {length}",
            digits.len()
        );
    }
}

#[test]
fn a_delivery_not_made_never_says_pushed() {
    for (landing, expected) in [
        (crate::checkout::Landing::Moved, agent::Delivery::Stale),
        (crate::checkout::Landing::Unchanged, agent::Delivery::Nothing),
        (
            crate::checkout::Landing::Failed,
            agent::Delivery::Failed(agent::DeliveryFailure {
                directory: 0,
                reason: agent::DeliveryReason::Unknown,
                diagnostic: agent::Diagnostic::empty(),
            }),
        ),
    ] {
        assert_eq!(crate::smith_delivery::landings(&[landing], &[None], false), expected);
    }
}

#[test]
fn a_delivery_with_a_field_beyond_its_title_and_body_is_refused() {
    let extra = fields(&[(b"title", b"t"), (b"body", b"b"), (b"tag", b"x")]);
    assert!(crate::smith_delivery::fields(extra, 512).is_none());
}

#[test]
fn a_delivery_with_an_empty_title_is_refused() {
    let empty = fields(&[(b"title", b""), (b"body", b"b")]);
    assert!(crate::smith_delivery::fields(empty, 512).is_none());
}

#[test]
fn landing_branch_is_bounded_and_translated_to_checkout() {
    for length in [0, 32, 33] {
        let mut h = Harness::new();
        let mut workspace = items(&[b"app"]);
        workspace.repositories[0].access = crate::wire::Access::Writable {
            push: Box::from(&b"topic"[..]),
            expected: Some([1; 32]),
            landing: Some(skein_lib::bytes::zeroed(length)),
        };
        if length == 32 {
            assert_eq!(crate::assignment::check(&workspace, &h.env.limits), Ok(()));
            let spec = translate::spec(7, workspace);
            assert_eq!(spec.repositories[0].landing.as_ref().unwrap().len(), length);
            assert_eq!(spec.repositories[0].expected.unwrap().raw(), [1; 32]);
        } else {
            let assign = Event::Assign { assignment: assignment(true, Box::new([])), items: Ok(workspace) };
            assert_eq!(&*h.step(assign), [refused(b"landing branch")]);
        }
    }
}

#[test]
fn merged_heads_are_bounded_at_the_workers_entrance_and_keep_their_order() {
    use crate::wire;
    const HEADS: usize = 64; // channel.md, 4.
    let mut h = Harness::new();
    h.env.limits.checkout.conflicts = 1;
    h.env.limits.checkout.path_bytes = 32;
    for heads in [&[][..], &[[3; 32]; HEADS + 1][..]] {
        let mut workspace = items(&[b"app"]);
        workspace.repositories[0].start =
            wire::Start::Merged { branch: b"main".as_slice().into(), heads: heads.into() };
        assert_eq!(crate::assignment::check(&workspace, &h.env.limits), Err(b"merged heads".as_slice()));
    }
    let mut workspace = items(&[b"app"]);
    let mut heads = Box::new([[3; 32]; HEADS]);
    heads[HEADS - 1] = [4; 32];
    workspace.repositories[0].start = wire::Start::Merged { branch: b"main".as_slice().into(), heads };
    assert_eq!(crate::assignment::check(&workspace, &h.env.limits), Ok(()));
    let spec = translate::spec(7, workspace);
    let crate::checkout::Start::Merged { branch, heads } = &spec.repositories[0].start else { panic!("merged start") };
    assert_eq!(branch.as_ref(), b"main");
    assert_eq!(heads.len(), HEADS);
    assert_eq!(heads[0].raw(), [3; 32]);
    assert_eq!(heads[HEADS - 1].raw(), [4; 32]);
}

#[test]
fn an_unchecked_receipt_names_the_saved_head_branch_and_missing_checks() {
    for branch in [b"topic".as_slice(), &[b'x'; 512]] {
        let delivery = crate::smith_delivery::landings(
            &[crate::checkout::Landing::Landed { commit: crate::checkout::git::Commit::new([1; 32]) }],
            &[Some(branch.into())],
            true,
        );
        let agent::Delivery::Delivered(delivered) = delivery else { panic!("landed save") };
        let text = delivered.receipts()[0].text();
        assert!(text.starts_with(b"saved 01010101"));
        assert!(text.ends_with(b", unchecked: no check ran; your checkout stays detached, as expected"));
        assert_eq!(skein_lib::bytes::find(text, b" to branch topic").is_some(), branch == b"topic");
        assert!(text.len() <= agent::Receipt::CAPACITY);
    }
}

#[test]
fn a_design_receipt_names_the_tip_it_merged_first() {
    let delivery = crate::smith_delivery::landings(
        &[crate::checkout::Landing::DesignMerged {
            commit: crate::checkout::git::Commit::new([0xab; 32]),
            tip: crate::checkout::git::Commit::new([0xcd; 32]),
        }],
        &[Some(b"g".as_slice().into())],
        false,
    );
    let agent::Delivery::Delivered(delivered) = delivery else { panic!("landed design") };
    assert_eq!(delivered.receipts()[0].text(), b"pushed abababababababababababababababababababababababababababababababab to branch g, merging its tip cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd first; your checkout stays detached, as expected");
}

#[test]
fn design_refusals_keep_the_run_going_and_name_the_offending_path() {
    use crate::checkout::Landing;
    for (landing, path, words) in [
        (
            Landing::DesignCode { path: b"src/lib.rs".as_slice().into() },
            Some(b"src/lib.rs".as_slice()),
            b"a delivery to the goal branch carries Markdown only; code goes through a change".as_slice(),
        ),
        (
            Landing::DesignConflict { files: Box::new([b"docs/a.md".as_slice().into()]) },
            Some(b"docs/a.md".as_slice()),
            b"settle the files in conflict and deliver again".as_slice(),
        ),
        (Landing::DesignMoving, None, b"the goal branch kept moving; deliver again".as_slice()),
        (Landing::DesignUnchecked, None, b"the goal branch takes checked deliveries alone; deliver instead".as_slice()),
    ] {
        let delivered = crate::smith_delivery::landings(&[landing], &[Some(b"g".as_slice().into())], false);
        let expected = agent::Delivery::Refused(
            agent::DeliveryRefusal::new(
                match path {
                    Some(path) => Some(agent::Marker::new(0, path.into()).expect("path")),
                    None => None,
                },
                words.into(),
            )
            .expect("bounded refusal"),
        );
        assert_eq!(delivered, expected);
    }
}

#[test]
fn merged_receipts_keep_both_heads_within_the_512_byte_bound() {
    let mut sha1 = [0; 32];
    sha1.get_mut(..20).expect("SHA-1 bytes").fill(0xab);
    for (commit, tip, head_bytes, tip_bytes) in [
        (sha1, sha1, 40_usize, 40_usize),
        (sha1, [0xcd; 32], 40, 64),
        ([0xab; 32], sha1, 64, 40),
        ([0xab; 32], [0xcd; 32], 64, 64),
    ] {
        let words = b"pushed , merging its tip  first; your checkout stays detached, as expected".len();
        let base = words.checked_add(head_bytes).expect("fixed head").checked_add(tip_bytes).expect("fixed tip");
        let room = agent::Receipt::CAPACITY
            .checked_sub(base)
            .expect("fixed words fit")
            .checked_sub(b" to branch ".len())
            .expect("branch prefix fits");
        for (length, included) in [(room, true), (room.checked_add(1).expect("past bound"), false)] {
            let mut branch = skein_lib::bytes::zeroed(length);
            branch.fill(b'b');
            let delivery = crate::smith_delivery::landings(
                &[crate::checkout::Landing::DesignMerged {
                    commit: crate::checkout::git::Commit::new(commit),
                    tip: crate::checkout::git::Commit::new(tip),
                }],
                &[Some(branch)],
                false,
            );
            let agent::Delivery::Delivered(delivered) = delivery else { panic!("landed design") };
            let text = delivered.receipts()[0].text();
            assert_eq!(text.len(), if included { agent::Receipt::CAPACITY } else { base });
            assert_eq!(skein_lib::bytes::find(text, b" to branch ").is_some(), included);
            assert!(skein_lib::bytes::find(text, b", merging its tip ").is_some());
            assert!(text.ends_with(b" first; your checkout stays detached, as expected"));
        }
    }
}
