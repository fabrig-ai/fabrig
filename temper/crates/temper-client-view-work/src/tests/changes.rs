//! Every form of Changes' words and links (temper/docs/design/client/pages/README.md, temper/docs/design/client/view.md, 3, temper/docs/design/client/pages/task.md, 3).
use super::fixtures as f;

use crate::pages::changes::ordinal;

#[test]
fn ordinals_follow_english_endings_including_the_teens() {
    for (number, expected) in [
        (1, "1st"),
        (2, "2nd"),
        (3, "3rd"),
        (4, "4th"),
        (11, "11th"),
        (12, "12th"),
        (13, "13th"),
        (21, "21st"),
        (22, "22nd"),
        (23, "23rd"),
        (111, "111th"),
    ] {
        assert_eq!(&*ordinal(number, &f::words()), expected.as_bytes());
    }
}
