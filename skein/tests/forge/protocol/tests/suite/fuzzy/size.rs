use skein_fake_forge_domain::api::{Answer, Op, Read};
use skein_forge_kit::joint::Joint;
use skein_lib::Rng;

#[test]
fn pull_size_conserves_lines_and_equal_sides_count_no_file() {
    let mut rng = Rng::new(0x51_2e);
    for case in 0..256_u64 {
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
            "case {case}"
        );
        assert_eq!(pull.size.files, u64::from(before != after), "case {case}");
    }
}
