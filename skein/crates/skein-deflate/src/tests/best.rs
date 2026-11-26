//! Best-level round trips, deterministic bytes and compression ratio (data/deflate.md, 4).

use skein_lib::Rng;

use super::{BLOCK, random, round_trip, round_trip_best};
use crate::{bound, encode_best};

#[test]
#[expect(clippy::disallowed_methods, reason = "the fixed phrase fills a finite test input")]
fn best_inputs_round_trip_within_the_bound() {
    let mut rng = Rng::new(37);
    assert_eq!(round_trip_best(b""), [3, 0], "the empty input keeps its fixed block");
    round_trip_best(b"abc");
    round_trip_best(&b"now is the time\n".repeat(256));
    round_trip_best(&random(&mut rng, 4096));
    round_trip_best(&random(&mut rng, 2 * BLOCK + 1));
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the known output rooms use vec!")]
fn best_encoding_is_independent_of_the_output_room() {
    let input = b"scratch output is reused, scratch output is written, scratch output is reused";
    let expected = round_trip_best(input);
    assert_eq!(round_trip_best(input), expected, "a second call gives the same bytes");
    let room = bound(input.len()).unwrap();
    let mut out = vec![0xa5; room + 100];
    assert_eq!(encode_best(input, &mut out), Some(expected.len()));
    assert_eq!(&out[..expected.len()], expected, "the previous contents do not matter");
    assert_eq!(encode_best(input, &mut out[..room - 1]), None, "room short of the bound is refused");
}

#[test]
#[expect(clippy::disallowed_types, reason = "the finite seeded text is built in a vector")]
fn best_is_strictly_shorter_on_repetitive_text() {
    const RECORDS: &[&[u8]] = &[
        b"worker: lease requested; queue: interactive; status: waiting\n",
        b"worker: lease requested; queue: background; status: waiting\n",
        b"worker: lease requested; queue: interactive; status: assigned\n",
        b"worker: lease requested; queue: background; status: assigned\n",
        b"worker: lease renewed; queue: interactive; status: active\n",
        b"worker: lease renewed; queue: background; status: active\n",
        b"worker: lease released; queue: interactive; status: finished\n",
        b"worker: lease released; queue: background; status: finished\n",
    ];
    const COUNT: usize = 4_096;
    let mut rng = Rng::new(319);
    let mut input = Vec::new();
    for _ in 0..COUNT {
        let i = usize::try_from(rng.below(u64::try_from(RECORDS.len()).unwrap())).unwrap();
        input.extend_from_slice(RECORDS[i]);
    }
    let fast = round_trip(&input).len();
    let best = round_trip_best(&input).len();
    assert!(best < fast, "best output is {best} bytes, fast output is {fast} bytes");
}
