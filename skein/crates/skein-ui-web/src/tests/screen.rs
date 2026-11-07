use crate::screen::{Report, changed};

#[test]
fn a_change_of_full_screen_reports_once() {
    for (was, now, expected) in [
        (false, false, None),
        (false, true, Some(Report::Entered)),
        (true, false, Some(Report::Left)),
        (true, true, None),
    ] {
        assert_eq!(changed(was, now), expected, "{was} -> {now}");
    }
}
