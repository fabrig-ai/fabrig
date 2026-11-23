//! Drawn kept turns follow their schema's decode through patches (smith/docs/design/protocol/transcript.md, 12).

mod sweep;

/// A reproducible normal-gate draw, separate from the explicit seed sweep.
const SEED: u64 = 0x51_17_70_01;
/// The explicit sweep starts at the owner's requested seed family.
const EXTRA_START: u64 = 713_000;
/// Thirty-two additional seeds exercise the same body outside the normal gate.
const EXTRA_END: u64 = 713_032;

#[test]
fn drawn_version_one_turns_patch_or_refuse_as_their_decode_does() {
    sweep::run(SEED);
}

#[test]
#[ignore = "explicit 713 extra-seed sweep, outside the normal gate"]
fn extra_seed_713_drawn_version_one_turns() {
    println!("713 extra sweep: seeds {EXTRA_START}..{EXTRA_END}, {} rounds each", sweep::ROUNDS);
    for seed in EXTRA_START..EXTRA_END {
        sweep::run(seed);
    }
}
