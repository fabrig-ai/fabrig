//! Git stream settlement sweeps (skein/docs/design/testing/sweeps.md, 4).

use hearth_git_world::{
    fixtures, generate,
    world::{self, Outcome, Settings},
};
use hearth_protocol_git::listing::Error;
use skein_lib::{Rng, stream::Fault};
use skein_world::sweep::{Case, cases};
use std::{convert::Infallible, ops::Range};

// The gate's samples for stream settlement and mutation sweeps.
const STREAM_SEEDS: Range<u64> = 0..512;
const MUTATION_SEEDS: Range<u64> = 0..256;

#[test]
fn random_listings_cut_ended_failed_and_closed_settle() -> Result<(), Case<Infallible>> {
    cases(skein_world::this_test!("random_listings_cut_ended_failed_and_closed_settle"), STREAM_SEEDS, |seed| {
        let mut rng = Rng::new(seed);
        let listing = generate::listing(&mut rng, &fixtures::SMALL);
        let end = usize::try_from(rng.between(0, listing.stdout.len() as u64)).expect("small input");
        let settings = Settings {
            end: Some(end),
            fail: rng.chance(250),
            close: if rng.chance(250) { Some(u32::try_from(rng.below(8)).expect("small call")) } else { None },
            ..Settings::cut(seed)
        };
        let outcome = world::list(&listing.stdout, &fixtures::SMALL, &settings);
        let expected = if settings.fail {
            Outcome::Failed(Error::Stream(Fault::Reset))
        } else if end == 0 || listing.stdout[end - 1] == 0 {
            Outcome::Done(
                listing
                    .entries
                    .iter()
                    .take(
                        usize::try_from(skein_lib::bytes::count(&listing.stdout[..end], b"\0", u32::MAX))
                            .expect("small count"),
                    )
                    .cloned()
                    .collect(),
            )
        } else {
            Outcome::Failed(Error::Truncated)
        };
        assert!(outcome == Outcome::Closed || outcome == expected, "seed {seed}: {outcome:?}, expected {expected:?}");
        Ok(())
    })
}

#[test]
fn random_batches_cut_ended_failed_and_closed_settle() -> Result<(), Case<Infallible>> {
    use hearth_git_world::world::BatchOutcome;
    use hearth_protocol_git::batch;
    cases(skein_world::this_test!("random_batches_cut_ended_failed_and_closed_settle"), STREAM_SEEDS, |seed| {
        let mut rng = Rng::new(seed);
        let batch = generate::batch(&mut rng, &fixtures::SMALL);
        let end = usize::try_from(rng.between(0, batch.stdout.len() as u64)).expect("small input");
        let settings = Settings {
            end: Some(end),
            fail: rng.chance(250),
            close: if rng.chance(250) { Some(u32::try_from(rng.below(24)).expect("small call")) } else { None },
            ..Settings::cut(seed)
        };
        let expected = if end == batch.stdout.len() {
            BatchOutcome::Done(batch.blobs)
        } else if settings.fail {
            BatchOutcome::Failed(batch::Error::Stream(Fault::Reset))
        } else {
            BatchOutcome::Failed(batch::Error::Truncated)
        };
        let outcome = world::read(&batch.stdout, &batch.requests, &fixtures::SMALL, &settings);
        assert!(
            outcome == BatchOutcome::Closed || outcome == expected,
            "seed {seed}: {outcome:?}, expected {expected:?}"
        );
        Ok(())
    })
}

#[test]
fn random_recordings_cut_mutated_ended_and_failed_settle() -> Result<(), Case<Infallible>> {
    use hearth_git_world::{expect, mutate};
    let recordings = hearth_recordings::all("git");
    let mut judgments = mutate::MutationBatch::default();
    cases(skein_world::this_test!("random_recordings_cut_mutated_ended_and_failed_settle"), MUTATION_SEEDS, |seed| {
        let mut rng = Rng::new(seed);
        let (_case, recording) = &recordings[usize::try_from(rng.below(recordings.len() as u64)).expect("case index")];
        let expected = expect::decode(recording.expect.as_ref().expect("expectation"));
        let mut variants = mutate::all(&recording.stdout);
        let index = usize::try_from(rng.below(variants.len() as u64)).expect("mutation index");
        let selected = variants.nth(index).expect("selected mutation");
        judgments.judge(selected, |stdout| {
            let mut damaged = recording.clone();
            damaged.stdout = stdout;
            let settings = Settings {
                end: Some(usize::try_from(rng.between(0, damaged.stdout.len() as u64)).expect("end")),
                fail: rng.chance(250),
                close: Some(u32::try_from(rng.below(32)).expect("close step")),
                ..Settings::cut(seed)
            };
            drop(world::replay(&damaged, &expected, &fixtures::SHIPPED, &settings));
        });
        Ok(())
    })?;
    judgments.finish();
    Ok(())
}

#[test]
fn random_hunks_cut_ended_failed_and_closed_settle() -> Result<(), Case<Infallible>> {
    use hearth_git_world::mutate;
    let mut judgments = mutate::MutationBatch::default();
    cases(skein_world::this_test!("random_hunks_cut_ended_failed_and_closed_settle"), MUTATION_SEEDS, |seed| {
        let mut rng = Rng::new(seed);
        for (input, _) in fixtures::HUNKS {
            let stdout = generate::hunks(&mut rng, input);
            for damaged in mutate::all(&stdout) {
                judgments.judge(damaged, |damaged| {
                    let settings = Settings {
                        end: Some(usize::try_from(rng.between(0, damaged.len() as u64)).expect("end")),
                        fail: rng.chance(250),
                        close: Some(u32::try_from(rng.below(48)).expect("close step")),
                        ..Settings::cut(seed)
                    };
                    drop(world::diff(&damaged, &fixtures::SMALL, &settings));
                });
            }
        }
        Ok(())
    })?;
    judgments.finish();
    Ok(())
}
