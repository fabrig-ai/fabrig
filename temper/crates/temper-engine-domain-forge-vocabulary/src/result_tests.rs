use super::*;

const HEAD: Commit = [3; 32];
const BOUND: u32 = 512;

#[test]
fn a_persons_request_repair_names_its_reason_without_borrowing_head_ci() {
    let mut result = ChangeResult::empty();
    result.checked(HEAD, &[check(b"old check", Check::Failed)], BOUND);
    result.repaired(change::Repair::Review { head: [2; 32] }, Some(HEAD), BOUND);
    assert_eq!(result.repair_count, 1);
    assert_eq!(result.repairs.bytes.as_ref(), b"a person asked for changes on the forge");
}

#[test]
fn a_landing_repair_names_its_gate_without_borrowing_head_ci() {
    let mut result = ChangeResult::empty();
    result.checked(HEAD, &[check(b"old check", Check::Failed)], BOUND);
    result.repaired(change::Repair::Landing { gate: 3 }, Some(HEAD), BOUND);
    assert_eq!(result.repair_count, 1);
    assert_eq!(result.repairs.bytes.as_ref(), b"landing gate 3 failed");
}

fn check(context: &[u8], check: Check) -> Status {
    Status {
        at: skein_lib::Time::from_nanos(0),
        context: context.into(),
        check,
        description: Box::new([]),
        url: Box::new([]),
        job: None,
    }
}
#[test]
fn a_landed_result_names_two_checks_one_ci_repair_and_its_approved_review() {
    let mut result = ChangeResult::empty();
    result.checked([2; 32], &[check(b"release note", Check::Failed)], BOUND);
    result.repaired(change::Repair::Ci, Some([2; 32]), BOUND);
    result.checked(HEAD, &[check(b"build", Check::Passed), check(b"release note", Check::Passed)], BOUND);
    result.reviewed(&HEAD, b"approved", BOUND);
    let expected = "checks at 0303030303030303030303030303030303030303030303030303030303030303: build passed; release note passed\nrepairs: 1: CI failed: release note\nreviews: approved at 0303030303030303030303030303030303030303030303030303030303030303";
    assert_eq!(result.words(&HEAD, BOUND).fitted(BOUND).unwrap().as_ref(), expected.as_bytes());
}
#[test]
fn a_landed_result_with_no_repairs_names_no_unread_checks() {
    let mut result = ChangeResult::empty();
    result.checked([2; 32], &[check(b"old tests", Check::Passed)], BOUND);
    let expected =
        "checks at 0303030303030303030303030303030303030303030303030303030303030303: none\nrepairs: 0\nreviews: none";
    assert_eq!(result.words(&HEAD, BOUND).fitted(BOUND).unwrap().as_ref(), expected.as_bytes());
}
#[test]
fn the_result_marks_the_exact_cut_including_history_omitted_before_landing() {
    let mut result = ChangeResult::empty();
    result.reviewed(&HEAD, b"changes requested", 40);
    result.repaired(change::Repair::Gate(1), Some(HEAD), BOUND);
    result.reviewed(&HEAD, b"approved", 40);
    let whole = b"checks at 0303030303030303030303030303030303030303030303030303030303030303: none\nrepairs: 1: review asked for changes\nreviews: changes requested at 0303030303030303030303030303030303030303030303030303030303030303; approved at 0303030303030303030303030303030303030303030303030303030303030303";
    for bound in [100, 160, 512] {
        let words = result.words(&HEAD, bound).fitted(bound).unwrap();
        let mut kept = None;
        for (at, pair) in words.windows(2).enumerate() {
            if pair == b"\n[" {
                kept = Some(at);
            }
        }
        let kept = kept.expect("cut marker");
        assert!(whole.starts_with(&words[..kept]));
        let lost = Decimal::of(u64::try_from(whole.len() - kept).unwrap());
        assert_eq!(&words[kept + 2..words.len() - b" bytes cut]\n".len()], lost.as_bytes());
        assert!(words.ends_with(b" bytes cut]\n"));
        assert!(words.len() <= usize::try_from(bound).unwrap());
    }
}
#[test]
fn every_review_verdict_keeps_its_own_head_and_order() {
    let mut result = ChangeResult::empty();
    for (head, verdict) in
        [(HEAD, b"failed".as_slice()), ([2; 32], b"cancelled"), (HEAD, b"changes requested"), ([2; 32], b"approved")]
    {
        result.reviewed(&head, verdict, BOUND);
    }
    assert_eq!(result.reviews.bytes.as_ref(), b"failed at 0303030303030303030303030303030303030303030303030303030303030303; cancelled at 0202020202020202020202020202020202020202020202020202020202020202; changes requested at 0303030303030303030303030303030303030303030303030303030303030303; approved at 0202020202020202020202020202020202020202020202020202020202020202");
}

#[test]
fn an_unread_failed_head_never_borrows_a_context_from_an_older_head() {
    let mut result = ChangeResult::empty();
    result.checked([2; 32], &[check(b"old check", Check::Failed)], BOUND);
    result.repaired(change::Repair::Ci, Some(HEAD), BOUND);
    assert_eq!(result.repairs.bytes.as_ref(), b"CI failed");
}

#[test]
fn a_report_precedes_what_the_landing_passed() {
    let mut result = ChangeResult::empty();
    result.checked(HEAD, &[check(b"build", Check::Passed)], BOUND);
    let evidence = result.words(&HEAD, BOUND).fitted(BOUND).unwrap();
    let words = result.reported(b"Done: edited README.\n## Left\n- none", &HEAD, BOUND).fitted(BOUND).unwrap();
    let prefix = b"Report:\nDone: edited README.\n## Left\n- none\n";
    let mut expected = Writer::new(prefix.len() + evidence.len());
    expected.put(prefix).unwrap();
    expected.put(&evidence).unwrap();
    assert_eq!(words, expected.finish());
}

#[test]
fn a_long_report_is_cut_from_the_end_and_marked() {
    const REPORT: &[u8] =
        b"Done: edited README.\n## Left\n- the remaining work is described here in the producer's own words.";
    const ROOM: u32 = 64;
    let result = ChangeResult::empty();
    let whole = result.reported(REPORT, &HEAD, BOUND).fitted(BOUND).unwrap();
    let words = result.reported(REPORT, &HEAD, ROOM).fitted(ROOM).unwrap();
    let mut kept = None;
    for (at, pair) in words.windows(2).enumerate() {
        if pair == b"\n[" {
            kept = Some(at);
        }
    }
    let kept = kept.expect("cut marker");
    assert!(words.starts_with(b"Report:\nDone: edited README."));
    assert!(whole.starts_with(&words[..kept]));
    for part in words.windows(b"checks at".len()) {
        assert_ne!(part, b"checks at");
    }
    let lost = Decimal::of(u64::try_from(whole.len() - kept).unwrap());
    assert_eq!(&words[kept + 2..words.len() - b" bytes cut]\n".len()], lost.as_bytes());
    assert!(words.ends_with(b" bytes cut]\n"));
    assert!(words.len() <= usize::try_from(ROOM).unwrap());
}

#[test]
fn an_empty_report_adds_no_line() {
    let result = ChangeResult::empty();
    assert_eq!(result.reported(b"", &HEAD, BOUND), result.words(&HEAD, BOUND));
}

#[test]
fn repair_accounts_precede_stale_caveats_with_utf8_cuts_and_completion_order() {
    let mut result = ChangeResult::empty();
    result.repaired(change::Repair::Ci, Some(HEAD), BOUND);
    result.completed(6, b"Adds changes/2.md", BOUND);
    result.repaired(change::Repair::Gate(1), Some(HEAD), BOUND);
    result.completed(9, b"Corrects changes/3.md", BOUND);
    let caveats = "exports CSV; missing note é".repeat(50);
    let whole = result.reported(caveats.as_bytes(), &HEAD, 4096).fitted(4096).unwrap();
    let words = result.reported(caveats.as_bytes(), &HEAD, 256).fitted(256).unwrap();
    let prefix = b"repairs: 2: CI failed\nsuccessful repair task 6: Adds changes/2.md; review asked for changes\nsuccessful repair task 9: Corrects changes/3.md\nProducer report (historical; before repairs):\nexports CSV;";
    assert!(words.starts_with(prefix));
    assert!(skein_lib::bytes::is_utf8(&words));
    assert!(words.len() <= 256);
    let cut = skein_lib::bytes::find(&words, b"\n[").expect("cut marker");
    assert!(whole.starts_with(&words[..cut]));
    let lost = Decimal::of(u64::try_from(whole.len() - cut).unwrap());
    assert_eq!(&words[cut + 2..words.len() - b" bytes cut]\n".len()], lost.as_bytes());
    assert!(words.ends_with(b" bytes cut]\n"));
    let mut bounded = ChangeResult::empty();
    bounded.repaired(change::Repair::Ci, Some(HEAD), 64);
    bounded.completed(10, "é".repeat(600).as_bytes(), 64);
    assert!(bounded.repairs.bytes.len() <= 64);
    assert!(bounded.repairs.cut > 0);
    let retained = bounded.repairs.fitted(64).unwrap();
    assert!(skein_lib::bytes::is_utf8(&retained));
    assert!(retained.ends_with(b" bytes cut]\n"));
}

#[test]
fn a_green_head_records_why_its_ci_repair_ended() {
    let head = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0,
    ];
    let mut result = ChangeResult::empty();
    result.checked(head, &[check(b"build", Check::Failed)], BOUND);
    result.repaired(change::Repair::Ci, Some(head), BOUND);
    result.ci_passed(&head, BOUND);
    assert_eq!(
        result.repairs.bytes.as_ref(),
        b"CI failed: build, then CI passed at 0123456789ab0000000000000000000000000000"
    );
    assert_eq!(result.repair_count, 1);
}

#[test]
fn a_green_head_with_no_repair_records_no_repair_words() {
    let mut result = ChangeResult::empty();
    result.ci_passed(&HEAD, BOUND);
    assert_eq!(result, ChangeResult::empty());
}

// temper/docs/design/domain/agent/charter.md, 3: the report shares room with its evidence.
#[test]
fn the_largest_report_beside_its_evidence_is_whole_then_marked() {
    const ROOM: u32 = 10_660;
    // Report label and newline: 9; checks at HEAD, build passed, repairs and reviews: 113.
    const WHOLE_REPORT: usize = 10_538;
    // Both boundary rows measured 2.6 ms CPU together.
    for (bytes, cut) in [(WHOLE_REPORT, false), (WHOLE_REPORT + 1, true)] {
        let mut report = skein_lib::bytes::zeroed(bytes);
        report.fill(b'r');
        let mut result = ChangeResult::empty();
        result.checked(HEAD, &[check(b"build", Check::Passed)], ROOM);
        let evidence = b"checks at 0303030303030303030303030303030303030303030303030303030303030303: build passed\nrepairs: 0\nreviews: none";
        let mut whole = Writer::new(b"Report:\n".len() + report.len() + 1 + evidence.len());
        whole.put(b"Report:\n").unwrap();
        whole.put(&report).unwrap();
        whole.put(b"\n").unwrap();
        whole.put(evidence).unwrap();
        let whole = whole.finish();
        let words = result.reported(&report, &HEAD, ROOM).fitted(ROOM).unwrap();
        assert_eq!(words.len(), usize::try_from(ROOM).unwrap());
        if cut {
            let prefix = words.strip_suffix(b"\n[17 bytes cut]\n").expect("one byte past room plus its cut marker");
            assert!(whole.starts_with(prefix));
            assert_eq!(whole.len() - prefix.len(), 17);
        } else {
            assert_eq!(words, whole);
        }
    }
}

#[test]
fn checks_name_later_attempts_only_at_their_head() {
    for (head, attempt, state, expected) in [
        (HEAD, 1, Check::Passed, "ci/test passed"),
        (HEAD, 2, Check::Passed, "ci/test passed at attempt 2"),
        (HEAD, 2, Check::Failed, "ci/test failed at attempt 2"),
        (HEAD, 2, Check::Pending, "ci/test pending"),
        ([4; 32], 2, Check::Passed, "ci/test passed"),
    ] {
        let mut status = check(b"ci/test", state);
        status.job = Some(skein_forge::api::JobAttempt { head, run: 7, job: 41, attempt });
        let mut result = ChangeResult::empty();
        result.checked(HEAD, &[status], BOUND);
        assert_eq!(result.checks.bytes.as_ref(), expected.as_bytes());
    }
}
