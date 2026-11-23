use super::{common, worded};
use skein_lib::Duration;

#[test]
fn message_words_give_only_provider_stated_waits() {
    const BEFORE_UNIT: &str = "Try again in 8";
    let cap = u32::try_from(BEFORE_UNIT.len()).expect("fixture length fits");
    let clipped = common::clipped(b"Try again in 8 hours", cap);
    let clipped = core::str::from_utf8(&clipped).expect("ASCII fixture");
    for (message, expected) in [
        ("Please try again in 6.5s.", Some(Duration::from_millis(6500))),
        ("TRY AGAIN IN   120MS", Some(Duration::from_millis(120))),
        ("try again in 1m30s", Some(Duration::from_secs(90))),
        ("try again in 2h0m4.25s", Some(Duration::from_millis(7_204_250))),
        ("try again in 20 seconds", Some(Duration::from_secs(20))),
        ("try again in 1 Minute", Some(Duration::from_secs(60))),
        ("try again in 1 hour", Some(Duration::from_secs(3600))),
        ("try again in 3 milliseconds", Some(Duration::from_millis(3))),
        ("try again in 0s", Some(Duration::ZERO)),
        ("try again in 0.0004s", Some(Duration::from_millis(1))),
        ("try again in 0.0004s0.0004s", Some(Duration::from_millis(1))),
        ("try again in 0.000000001ms", Some(Duration::from_millis(1))),
        ("try again in 999999999ms", Some(Duration::from_millis(999_999_999))),
        ("try again in 6.5", None),
        ("try again in s", None),
        (clipped, None),
        ("try again in 1m30", None),
        ("try again in 1m30 seconds", None),
        ("try again in 1.5 minutes", None),
        ("try again in 1  minute", None),
        ("try again in 1234567890ms", None),
        ("try again in 0.0000000001ms", None),
        ("try again in 999999999h", None),
        ("try again in 6try again in 7s", None),
        ("try again in 3s; try again in 7s", Some(Duration::from_secs(3))),
        ("3 tokens per min (TPM)", Some(Duration::from_secs(60))),
        ("per minute.", Some(Duration::from_secs(60))),
        ("per day (TPD)", Some(Duration::from_secs(86_400))),
        ("PER HOUR", Some(Duration::from_secs(3600))),
        ("per sec", Some(Duration::from_secs(1))),
        ("per second", Some(Duration::from_secs(1))),
        ("per minute. try again in 6.5", Some(Duration::from_secs(60))),
        ("permit", None),
        ("per minutes", None),
        ("upper min", None),
        ("per  min", None),
        ("per min; per sec", Some(Duration::from_secs(60))),
    ] {
        assert_eq!(worded(message.as_bytes()), expected, "{message}");
    }
}
