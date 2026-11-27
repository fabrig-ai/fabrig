//! One text stream per case (skein/docs/design/testing/sweeps.md, 4.2 to 4.5).

use skein_fake_forge_domain::api::{Answer, Op, Read};
use skein_forge_kit::joint::Joint;
use skein_lib::Rng;
use skein_world::{sweep::cases, this_test};

const TEXT_SEED: u64 = 0x51_2e;

#[test]
fn pull_size_conserves_lines_and_equal_sides_count_no_file() {
    cases(this_test!("pull_size_conserves_lines_and_equal_sides_count_no_file"), [TEXT_SEED], |seed| {
        check_stream(seed);
        Ok::<(), String>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}

fn round_seed(seed: u64, round: u64) -> u64 {
    // At the gate the offset is zero, so the world's seed is the round index.
    seed.wrapping_add(round).wrapping_sub(TEXT_SEED)
}

#[test]
fn pull_size_round_draw_keeps_gate_offset_and_wraps_full_width_seed() {
    assert_eq!(round_seed(TEXT_SEED, 255), 255);
    assert_eq!(round_seed(u64::MAX, 1), u64::MAX - (TEXT_SEED - 1));
}

fn check_stream(seed: u64) {
    let mut rng = Rng::new(seed);
    for round in 0..256_u64 {
        let case = round_seed(seed, round);
        let mut before = String::new();
        let mut after = String::new();
        for side in [&mut before, &mut after] {
            for _ in 0..rng.below(12) {
                side.push_str(match rng.below(4) {
                    0 | 2 => "a\n",
                    1 => "b\n",
                    _ => "\n",
                });
            }
        }
        if case % 8 == 0 {
            after.clone_from(&before);
        }
        let mut joint = Joint::new(case);
        let base = joint.push(b"main", before.as_bytes());
        if before == after {
            joint.branch(b"work", base);
        } else {
            joint.push(b"work", after.as_bytes());
        }
        let number = joint.pull(b"work", b"main");
        let Answer::Pull(pull) = joint.direct(Op::Read(Read::Pull { number })).expect("pull size") else {
            panic!("pull answer");
        };
        assert_eq!(
            i128::from(pull.size.additions) - i128::from(pull.size.deletions),
            i128::try_from(after.lines().count()).expect("short list")
                - i128::try_from(before.lines().count()).expect("short list"),
            "round {round}, fixture seed {case}"
        );
        assert_eq!(pull.size.files, u64::from(before != after), "round {round}, fixture seed {case}");
    }
}
