//! Lifecycle transitions (skein/docs/design/lib.md, 3).

use crate::Lifecycle;

#[test]
fn close_and_abort_only_advance_the_lifecycle() {
    use Lifecycle::{Aborting, Closed, Closing, Live};

    for (initial, after_close, close_changed, after_abort, abort_changed, settling) in [
        (Live, Closing, true, Aborting, true, false),
        (Closing, Closing, false, Aborting, true, true),
        (Aborting, Aborting, false, Aborting, false, true),
        (Closed, Closed, false, Closed, false, false),
    ] {
        let mut state = initial;
        assert_eq!(state.is_settling(), settling);
        assert_eq!(state.close(), close_changed);
        assert_eq!(state, after_close);
        assert!(!state.close(), "second close changes nothing");
        state = initial;
        assert_eq!(state.abort(), abort_changed);
        assert_eq!(state, after_abort);
        assert!(!state.abort(), "second abort changes nothing");
    }
    for mut state in [Closing, Aborting] {
        state.closed();
        assert_eq!(state, Closed);
        assert!(!state.is_settling());
    }
}

#[test]
#[should_panic(expected = "only a settling lifecycle can become closed")]
fn a_live_lifecycle_cannot_be_closed() {
    Lifecycle::Live.closed();
}

#[test]
#[should_panic(expected = "only a settling lifecycle can become closed")]
fn a_closed_lifecycle_cannot_settle_twice() {
    Lifecycle::Closed.closed();
}
