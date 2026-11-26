//! Match choices at the window and block boundaries, and lazy matching.

use skein_lib::Rng;

use crate::encoder::BLOCK;
use crate::tests::{random, round_trip_best};

use super::{EMPTY_BEST, LONGEST, WINDOW, block_at, parse};

#[test]
fn best_follows_an_older_link_to_the_far_end_of_the_window() {
    const NEAR: usize = WINDOW / 2;
    let mut body = random(&mut Rng::new(21), WINDOW);
    // The nearer occurrence of the prefix would give a short match.
    body.copy_within(..4, NEAR);
    body[NEAR + 4] = body[4] ^ 1;
    body.extend_from_within(..LONGEST);

    let mut best = EMPTY_BEST;
    parse(&mut best, &body, block_at(0, 1, body.len()).unwrap()).unwrap();
    let parsed = &best.encoder.parse;
    let last = parsed.count - 1;
    assert_eq!(usize::from(parsed.starts[last]), WINDOW);
    assert_eq!(usize::from(parsed.distances[last]) + 1, WINDOW);
    assert_eq!(usize::from(parsed.lengths[last]) + 3, LONGEST);
    round_trip_best(&body);
}

#[test]
fn best_carries_its_chain_across_a_block_start() {
    const BACK: usize = 1024;
    let mut body = random(&mut Rng::new(53), BLOCK);
    body.extend_from_within(BLOCK - BACK..BLOCK - BACK + LONGEST);

    let mut best = EMPTY_BEST;
    parse(&mut best, &body, block_at(0, 2, body.len()).unwrap()).unwrap();
    parse(&mut best, &body, block_at(1, 2, body.len()).unwrap()).unwrap();
    let parsed = &best.encoder.parse;
    assert_eq!(parsed.count, 1, "one match covers the second block");
    assert_eq!(parsed.starts[0], 0, "the match starts at the block boundary");
    assert_eq!(usize::from(parsed.distances[0]) + 1, BACK);
    assert_eq!(usize::from(parsed.lengths[0]) + 3, LONGEST);
    round_trip_best(&body);
}

#[test]
fn best_defers_a_match_when_the_next_position_has_a_longer_one() {
    // At ABCDEFGHI, ABCD is a match, but one byte later BCDEFGHI is longer.
    let body = b"ABCDx|BCDEFGHI|ABCDEFGHI";
    let mut best = EMPTY_BEST;
    parse(&mut best, body, block_at(0, 1, body.len()).unwrap()).unwrap();
    let parsed = &best.encoder.parse;
    assert_eq!(parsed.count, 1);
    assert_eq!(usize::from(parsed.starts[0]), b"ABCDx|BCDEFGHI|A".len());
    assert_eq!(usize::from(parsed.lengths[0]) + 3, b"BCDEFGHI".len());
}
