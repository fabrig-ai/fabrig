use super::{result, words};
use smith_host_domain::{ProgramUsage, UsageState};

#[test]
fn a_heading_without_a_share_does_not_borrow_the_next_windows_share() {
    let windows = words::read(b"Current session\nbar\nCurrent week (all models)\n34% used\n");
    assert_eq!(windows.len(), 1);
    assert_eq!(windows.first().expect("week window").label.as_ref(), b"seven_day");
}

#[test]
fn only_the_first_eight_completed_windows_are_reported() {
    let windows = words::read(b"Current session\n1% used\nCurrent session\n2% used\nCurrent session\n3% used\nCurrent session\n4% used\nCurrent session\n5% used\nCurrent session\n6% used\nCurrent session\n7% used\nCurrent session\n8% used\nCurrent session\n9% used\n");
    assert_eq!(windows.len(), 8);
    assert_eq!(windows.last().expect("eighth window").used, Some(800));
}

#[test]
fn decimal_percents_are_read_in_basis_points_without_floating_point() {
    let windows = words::read(b"Current session\n12.25% used\n99% used\nCurrent week (Opus only)\n100% used\n");
    assert_eq!(windows.first().expect("session").used, Some(1225));
    assert_eq!(windows.last().expect("opus").state, UsageState::Limited);
    for percent in [b"-1".as_slice(), b"101", b"1e1", b"1.2.3", b".5"] {
        let words = skein_lib::bytes::concat(&[b"Current session\n", percent, b"% used\n"]);
        assert!(words::read(&words).is_empty());
    }
}

#[test]
fn a_model_label_past_thirty_two_bytes_is_not_reported() {
    let windows = words::read(b"Current week (abcdefghijklmnopqrstuvw only)\n12% used\nCurrent week (abcdefghijklmnopqrstuv only)\n13% used\n");
    assert_eq!(windows.len(), 1);
    assert_eq!(windows.first().expect("bounded model").label.len(), 32);
    assert_eq!(windows.first().expect("bounded model").used, Some(1300));
}

#[test]
fn a_result_is_one_bounded_object_and_spend_precedes_the_text() {
    for bytes in [
        b"[]".as_slice(),
        br#"{"type":"other","is_error":false,"result":"Current session\n12% used"}"#,
        br#"{"type":"result","is_error":false,"result":"Current session\n12% used"} {}"#,
        br#"{"type":"result","is_error":null,"result":"Current session\n12% used"}"#,
    ] {
        assert_eq!(result(bytes, 4), None);
    }
    let oversized = skein_lib::bytes::zeroed(16_385);
    assert_eq!(result(&oversized, 4), None);
    assert_eq!(
        result(br#"{"type":"result","usage":{"cache_creation_input_tokens":1},"result":null}"#, 4),
        Some(ProgramUsage::Spent)
    );
    assert_eq!(result(br#"{"type":"result","total_cost_usd":1e-99,"result":null}"#, 4), Some(ProgramUsage::Spent));
    let Some(ProgramUsage::Failed(failure)) =
        result(br#"{"type":"result","is_error":true,"result":"failure detail"}"#, 4)
    else {
        unreachable!("error result");
    };
    assert_eq!(failure.detail.as_ref(), b"fail");
}
