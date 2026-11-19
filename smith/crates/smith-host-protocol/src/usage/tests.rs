use super::{result, words};
use skein_lib::{Civil, Duration, Wall};
use smith_host_domain::{ProgramUsage, UsageState, UsageWindow};

fn wall(year: u32, month: u8, day: u8, hour: u64, minute: u64) -> Wall {
    let midnight = Civil { year, month, day }.wall().expect("fixture date");
    let hour = hour.checked_mul(3600).expect("fixture hour");
    let minute = minute.checked_mul(60).expect("fixture minute");
    let seconds = hour.checked_add(minute).expect("fixture clock");
    let nanos = seconds.checked_mul(1_000_000_000).expect("fixture clock");
    Wall::from_nanos(midnight.as_nanos().checked_add(nanos).expect("fixture wall time"))
}

fn asked() -> Wall {
    wall(2026, 11, 19, 4, 0)
}

#[test]
fn recorded_one_line_windows_end_before_the_sessions() {
    let text = "You are currently using your subscription to power your Claude Code usage\n\n\
Current session: 16% used · resets Nov 19, 4:30am (UTC)\n\
Current week (all models): 60% used · resets Nov 23, 5am (UTC)\n\
Current week (Fable): 8% used · resets Nov 23, 4:59am (UTC)\n\n\
What's contributing to your usage?\nLast 24h · 3 requests · 2 sessions\n  50%  advice · detail\n\
Current week (later): 9% used · resets Nov 23, 5am (UTC)\n";
    let expected = [
        (b"five_hour".as_slice(), 18_000, 1600, wall(2026, 11, 19, 4, 30)),
        (b"seven_day".as_slice(), 604_800, 6000, wall(2026, 11, 23, 5, 0)),
        (b"seven_day_fable".as_slice(), 604_800, 800, wall(2026, 11, 23, 4, 59)),
    ]
    .map(|(label, seconds, used, resets)| UsageWindow {
        label: label.into(),
        length: Some(Duration::from_secs(seconds)),
        used: Some(used),
        resets: Some(resets),
        state: UsageState::Open,
    });
    assert_eq!(words::read(text.as_bytes(), asked()).as_ref(), expected);
}

#[test]
fn resets_take_the_nearest_year_on_either_side_of_the_ask() {
    for (text, asked, expected) in [
        ("Current session: 1% used · resets Jan 2, 5am (UTC)", wall(2026, 12, 30, 12, 0), wall(2027, 1, 2, 5, 0)),
        ("Current session: 1% used · resets Dec 31, 11pm (UTC)", wall(2027, 1, 1, 0, 10), wall(2026, 12, 31, 23, 0)),
    ] {
        let windows = words::read(text.as_bytes(), asked);
        assert_eq!(windows.first().expect("session").resets, Some(expected));
    }
}

#[test]
fn twelve_am_is_midnight_and_twelve_pm_is_noon() {
    for (clock, hour) in [("12am", 0), ("12pm", 12), ("12:30am", 0), ("12:30pm", 12)] {
        let text = skein_lib::bytes::concat(&[
            "Current session: 1% used · resets Nov 19, ".as_bytes(),
            clock.as_bytes(),
            b" (UTC)",
        ]);
        let windows = words::read(&text, asked());
        let minute = if clock.contains(':') { 30 } else { 0 };
        assert_eq!(windows.first().expect("session").resets, Some(wall(2026, 11, 19, hour, minute)));
    }
}

#[test]
fn unreadable_resets_keep_the_share() {
    for reset in [
        "Nov 19, 4:30pm (Europe/Rome)",
        "soon",
        "Feb 30, 5am (UTC)",
        "Nov 19, 0am (UTC)",
        "Nov 19, 13pm (UTC)",
        "Nov 19, 4:60am (UTC)",
        "Nov 19, 4:3am (UTC)",
        "Never 19, 4am (UTC)",
    ] {
        let text = skein_lib::bytes::concat(&["Current session: 16% used · resets ".as_bytes(), reset.as_bytes()]);
        let windows = words::read(&text, asked());
        let window = windows.first().expect("share still reads");
        assert_eq!(window.used, Some(1600));
        assert_eq!(window.resets, None, "{reset}");
    }
}

#[test]
fn a_heading_without_a_share_does_not_borrow_the_next_windows_share() {
    let windows =
        words::read(b"Current session: resets soon\n34% used\nCurrent week (all models): 34% used\n", asked());
    assert_eq!(windows.len(), 1);
    assert_eq!(windows.first().expect("week window").label.as_ref(), b"seven_day");
    for text in [b"usage refused".as_slice(), b"No subscription usage: 34% used", b"\xff"] {
        assert!(words::read(text, asked()).is_empty());
    }
    assert_eq!(
        result(br#"{"type":"result","is_error":false,"result":"No subscription usage: 34% used"}"#, 4, asked()),
        None
    );
}

#[test]
fn only_the_first_eight_completed_windows_are_reported() {
    let windows = words::read(b"Current session: 1% used\nCurrent session: 2% used\nCurrent session: 3% used\nCurrent session: 4% used\nCurrent session: 5% used\nCurrent session: 6% used\nCurrent session: 7% used\nCurrent session: 8% used\nCurrent session: 9% used\n", asked());
    assert_eq!(windows.len(), 8);
    assert_eq!(windows.last().expect("eighth window").used, Some(800));
}

#[test]
fn decimal_percents_are_read_in_basis_points_without_floating_point() {
    let windows =
        words::read(b"Current session: 12.25% used\n99% used\nCurrent week (Opus only): 100% used\n", asked());
    assert_eq!(windows.first().expect("session").used, Some(1225));
    assert_eq!(windows.last().expect("opus").state, UsageState::Limited);
    for percent in [b"-1".as_slice(), b"101", b"1e1", b"1.2.3", b".5"] {
        let words = skein_lib::bytes::concat(&[b"Current session: ", percent, b"% used\n"]);
        assert!(words::read(&words, asked()).is_empty());
    }
}

#[test]
fn a_model_label_past_thirty_two_bytes_is_not_reported() {
    let windows = words::read(b"Current week (abcdefghijklmnopqrstuvw only): 12% used\nCurrent week (abcdefghijklmnopqrstuv only): 13% used\n", asked());
    assert_eq!(windows.len(), 1);
    assert_eq!(windows.first().expect("bounded model").label.len(), 32);
    assert_eq!(windows.first().expect("bounded model").used, Some(1300));
}

#[test]
fn a_result_is_one_bounded_object_and_spend_precedes_the_text() {
    for bytes in [
        b"[]".as_slice(),
        br#"{"type":"other","is_error":false,"result":"Current session: 12% used"}"#,
        br#"{"type":"result","is_error":false,"result":"Current session: 12% used"} {}"#,
        br#"{"type":"result","is_error":null,"result":"Current session: 12% used"}"#,
    ] {
        assert_eq!(result(bytes, 4, asked()), None);
    }
    let oversized = skein_lib::bytes::zeroed(16_385);
    assert_eq!(result(&oversized, 4, asked()), None);
    assert_eq!(
        result(br#"{"type":"result","usage":{"cache_creation_input_tokens":1},"result":null}"#, 4, asked()),
        Some(ProgramUsage::Spent)
    );
    assert_eq!(
        result(br#"{"type":"result","total_cost_usd":1e-99,"result":null}"#, 4, asked()),
        Some(ProgramUsage::Spent)
    );
    let Some(ProgramUsage::Failed(failure)) =
        result(br#"{"type":"result","is_error":true,"result":"failure detail"}"#, 4, asked())
    else {
        unreachable!("error result");
    };
    assert_eq!(failure.detail.as_ref(), b"fail");
}
