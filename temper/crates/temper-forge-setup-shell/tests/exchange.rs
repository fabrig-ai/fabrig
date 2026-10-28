use skein_lib::{Duration, Time};
use temper_forge_setup_shell::exchange::{Gather, Next, pause};
#[test]
fn pause_doubles_from_the_backoff_to_its_cap() {
    for (attempt, expected) in [(0, 100), (3, 800), (6, 5000), (200, 5000)] {
        assert_eq!(
            pause(
                Duration::from_millis(100),
                Duration::from_secs(5),
                Duration::from_secs(120),
                attempt,
                Time::from_nanos(0),
                Time::from_nanos(0)
            ),
            Some(Duration::from_millis(expected)),
            "attempt {attempt}"
        );
    }
}
#[test]
fn pause_ends_at_the_deadline() {
    for (now, expected) in [(60, Some(Duration::from_millis(40))), (100, None)] {
        assert_eq!(
            pause(
                Duration::from_millis(50),
                Duration::from_millis(100),
                Duration::from_millis(100),
                0,
                Time::from_nanos(0),
                Time::from_nanos(now * 1_000_000)
            ),
            expected
        );
    }
}
#[test]
fn a_listing_ends_as_the_rows_it_gathered_in_order() {
    use skein_forge::forgejo::{
        decode::Step,
        document::{Document, Row},
    };
    let first =
        skein_forge::forgejo::id::ObjectId { format: skein_forge::forgejo::id::ObjectFormat::Sha256, bytes: [1; 32] };
    let second = skein_forge::forgejo::id::ObjectId { bytes: [2; 32], ..first };
    let mut gather = Gather::default();
    for head in [first, second] {
        assert!(matches!(
            gather.take(Step::Row(Row::Commit(head), skein_json::tokenizer::Request::Next)),
            Next::Demand(_)
        ));
    }
    let Next::Reply(reply) = gather.take(Step::Done(Document::Listed { skipped: 0 })) else { panic!("listing reply") };
    assert_eq!(reply, skein_forge_setup::reconcile::Reply::Rows(Box::new([Row::Commit(first), Row::Commit(second)])));
}
