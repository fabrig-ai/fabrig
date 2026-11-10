//! Stderr keeps its newest lines in order (hearth/docs/design/domain/render/README.md, 2.2).
use hearth_git_world::{fixtures::SMALL, generate, world};
use hearth_protocol_git::limits::Limits;
use skein_lib::Rng;

#[test]
fn the_tail_keeps_the_newest_lines_oldest_first_whatever_the_cuts() {
    let mut twice_the_lines = false;
    let mut past_line_limit = false;
    for seed in 0..32 {
        let stderr = generate::stderr(&mut Rng::new(seed), &SMALL);
        twice_the_lines |= skein_lib::bytes::count(&stderr.bytes, b"\n", u32::MAX) == SMALL.tail_lines * 2;
        past_line_limit |= stderr
            .bytes
            .split(|&byte| byte == b'\n')
            .any(|line| line.len() > usize::try_from(SMALL.tail_line).expect("small line"));
        assert_eq!(world::tail(&stderr.bytes, &SMALL, seed), stderr.lines, "seed {seed}");
    }
    assert!(twice_the_lines && past_line_limit, "drawn twice the line count and oversized lines");
}

#[test]
fn an_unended_last_line_is_the_newest() {
    let rows: &[(&[u8], &[&[u8]])] = &[
        (b"one\ntwo\nthree", &[b"two", b"three"]),
        (b"one\nthree", &[b"one", b"three"]),
        (b"three", &[b"three"]),
        (b"one\ntwo\n", &[b"one", b"two"]),
    ];
    for (input, expected) in rows {
        for seed in 0..8 {
            let lines = world::tail(input, &SMALL, seed);
            let lines: Vec<_> = lines.iter().map(AsRef::as_ref).collect();
            assert_eq!(lines, *expected, "input {input:?}, seed {seed}");
        }
    }
}

#[test]
fn a_tail_of_no_lines_holds_nothing() {
    let limits = Limits { tail_lines: 0, ..SMALL };
    for input in [b"one\ntwo\n".as_slice(), b"one\ntwo\nthree"] {
        for seed in 0..8 {
            assert!(world::tail(input, &limits, seed).is_empty(), "input {input:?}, seed {seed}");
        }
    }
}
