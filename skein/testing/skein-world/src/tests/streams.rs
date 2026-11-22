use crate::streams::{Stream, Streams};
use skein_lib::Rng;

const STREAMS: [Stream; 4] = [Stream::Workload, Stream::Faults, Stream::Parties, Stream::Scheduling];

fn draws<const N: usize>(mut rng: Rng) -> [u64; N] {
    let mut values = [0; N];
    for value in &mut values {
        *value = rng.next_u64();
    }
    values
}

#[test]
fn four_streams_four_sequences() {
    let streams = Streams::new(11);
    for (index, stream) in STREAMS.iter().enumerate() {
        let sequence = draws::<8>(streams.rng(*stream, ""));
        for other in STREAMS.iter().skip(index.saturating_add(1)) {
            assert_ne!(sequence, draws::<8>(streams.rng(*other, "")), "{stream:?} and {other:?}");
        }
    }
}

#[test]
fn independent_of_other_draws() {
    let untouched = Streams::new(11);
    let expected = draws::<16>(untouched.rng(Stream::Workload, ""));
    let streams = Streams::new(11);
    let mut faults = streams.rng(Stream::Faults, "");
    let mut goals = streams.rng(Stream::Workload, "goals");
    for _ in 0..1000_u16 {
        faults.next_u64();
        goals.next_u64();
    }
    assert_eq!(expected, draws::<16>(streams.rng(Stream::Workload, "")));
}

#[test]
fn parts_differ() {
    let streams = Streams::new(11);
    let forge = draws::<8>(streams.rng(Stream::Faults, "forge"));
    let llm = draws::<8>(streams.rng(Stream::Faults, "llm"));
    let own = draws::<8>(streams.rng(Stream::Faults, ""));
    assert_ne!(forge, llm);
    assert_ne!(forge, own);
    assert_ne!(llm, own);
}

#[test]
fn replay() {
    let first = Streams::new(42);
    let second = Streams::new(42);
    for stream in STREAMS {
        for part in ["", "forge", "llm", "goals", "restarts", "party/é"] {
            assert_eq!(first.seed_of(stream, part), second.seed_of(stream, part));
            assert_eq!(
                draws::<16>(first.rng(stream, part)),
                draws::<16>(second.rng(stream, part)),
                "{stream:?}/{part}"
            );
        }
    }
}

#[test]
fn pinned_derivation() {
    let streams = Streams::new(0);
    // Workload, by hand: tag = 0x64616f6c6b726f77; fnv("") = 0xcbf29ce484222325.
    // mix(0 ^ tag) = 0x19d1207b8501a2a5; XOR fnv("") = 0xd223bc9f01238180.
    // mix(mix(0 ^ tag) ^ fnv("")) = mix(0xd223bc9f01238180) = 0xdf3ba3ee200f4529.
    for (stream, part, expected) in [
        (Stream::Workload, "", 0xdf3b_a3ee_200f_4529),
        (Stream::Faults, "", 0x8e2f_566a_ebfa_ba6c),
        (Stream::Parties, "", 0xa52e_e87a_2798_b415),
        (Stream::Scheduling, "", 0xe4a9_d581_75ed_a90c),
        (Stream::Faults, "forge", 0x9792_8b73_3a53_f67c),
    ] {
        assert_eq!(streams.seed_of(stream, part), expected, "{stream:?}/{part}");
    }
}

#[test]
fn seed_returns_original() {
    for seed in [0, 11, 42, u64::MAX] {
        assert_eq!(Streams::new(seed).seed(), seed);
    }
}
