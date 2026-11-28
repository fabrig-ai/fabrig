//! Mmdebstrap's seeded reference comparisons (hearth/docs/design/testing/README.md, 4.5).

use hearth_mmdebstrap_world::{fixtures::LIMITS, generate, reference, world};
use skein_lib::{Rng, len_u64};
use skein_world::{sweep::cases, this_test};
use std::convert::Infallible;

const SEEDS: u64 = 1_000;
const PINNED: &[u64] = &[];
const RECORDING_SEEDS: u64 = 128;
const OUTPUT_OFFSET: u64 = 0x0702_f000;
const LIST_OFFSET: u64 = 0x3801_f000;
const RECORDING_OFFSET: u64 = 0x3802_f000;

fn input_rng(seed: u64, offset: u64) -> Rng {
    Rng::new(seed.wrapping_add(offset))
}

fn recording_index(seed: u64, len: usize) -> usize {
    usize::try_from(seed % len_u64(len)).expect("the index fits the corpus length")
}

#[test]
fn input_offsets_wrap_at_the_largest_seed() {
    for offset in [OUTPUT_OFFSET, LIST_OFFSET, RECORDING_OFFSET] {
        assert_eq!(input_rng(u64::MAX, offset), Rng::new(offset - 1));
    }
}

#[test]
fn recording_choice_uses_the_full_width_seed() {
    // This corpus length makes the high bits affect the index.
    const RECORDINGS: usize = 7;
    assert_eq!(recording_index(u64::MAX, RECORDINGS), 1);
}

#[test]
fn generated_and_mutated_output_reads_as_the_reference_reads_it() {
    let mut mutations = generate::MutationBatch::default();
    cases(
        this_test!("generated_and_mutated_output_reads_as_the_reference_reads_it"),
        PINNED.iter().copied().chain(0..SEEDS),
        |seed| -> Result<(), Infallible> {
            let mut rng = input_rng(seed, OUTPUT_OFFSET);
            let output = generate::output(&mut rng, &LIMITS);
            let output = if rng.chance(500) { generate::mutate(&mut rng, &output) } else { output.into() };
            mutations.judge(output, |output| {
                let end = world::end(&mut rng, &output);
                let run = world::read_output(&output, seed, &LIMITS, end);
                assert_eq!(run.decoded, reference::output(&output, end, &LIMITS), "seed {seed} end {end:?}");
            });
            Ok(())
        },
    )
    .unwrap_or_else(|case| panic!("{case}"));
    mutations.finish();
}
#[test]
fn generated_and_mutated_lists_read_as_the_reference_reads_them() {
    let mut mutations = generate::MutationBatch::default();
    cases(
        this_test!("generated_and_mutated_lists_read_as_the_reference_reads_them"),
        PINNED.iter().copied().chain(0..SEEDS),
        |seed| -> Result<(), Infallible> {
            let mut rng = input_rng(seed, LIST_OFFSET);
            let index = generate::index(&mut rng, &LIMITS);
            let index = if rng.chance(500) { generate::mutate(&mut rng, &index) } else { index.into() };
            mutations.judge(index, |index| {
                let end = world::end(&mut rng, &index);
                assert_eq!(
                    world::read_index(&index, seed, &LIMITS, end),
                    reference::index(&index, end, &LIMITS),
                    "seed {seed} end {end:?}"
                );
            });
            Ok(())
        },
    )
    .unwrap_or_else(|case| panic!("{case}"));
    mutations.finish();
}
#[test]
fn recordings_cut_and_mutated_read_as_the_reference_reads_them() {
    let recordings = hearth_mmdebstrap_world::recordings::all();
    let mut mutations = generate::MutationBatch::default();
    cases(
        this_test!("recordings_cut_and_mutated_read_as_the_reference_reads_them"),
        0..RECORDING_SEEDS,
        |seed| -> Result<(), Infallible> {
            let case = &recordings[recording_index(seed, recordings.len())];
            let mut rng = input_rng(seed, RECORDING_OFFSET);
            for bytes in [&case.recording.stderr, &case.recording.stdout] {
                let bytes = generate::mutate(&mut rng, bytes);
                mutations.judge(bytes, |bytes| {
                    let end = world::end(&mut rng, &bytes);
                    assert_eq!(
                        world::read_output(&bytes, seed, &LIMITS, end).decoded,
                        reference::output(&bytes, end, &LIMITS),
                        "{}: seed {seed}, {end:?}",
                        case.case
                    );
                });
            }
            if let Some(bytes) = &case.recording.stanzas {
                let bytes = generate::mutate(&mut rng, bytes);
                mutations.judge(bytes, |bytes| {
                    let end = world::end(&mut rng, &bytes);
                    assert_eq!(
                        world::read_index(&bytes, seed, &LIMITS, end),
                        reference::index(&bytes, end, &LIMITS),
                        "{} stanzas: seed {seed}, {end:?}",
                        case.case
                    );
                });
            }
            Ok(())
        },
    )
    .unwrap_or_else(|case| panic!("{case}"));
    mutations.finish();
}
