//! Observation ordering across delayed fake-forge answers and restarts.
#![expect(clippy::disallowed_methods, clippy::disallowed_macros, reason = "ordinary Rust refusal and tree oracle")]
use crate::{
    DotTemperRevision,
    tests::temper_revision::{GOOD, PROJECT, WAYS, World, file},
};
use alloc::collections::BTreeMap;
use skein_forge::api::{Answer, Error, Read};
use skein_lib::Rng;

#[test]
fn fuzzy_temper_revision_follows_the_latest_taken_observation() {
    for seed in 0..24_u64 {
        let mut rng = Rng::new(seed);
        let mut w = World::new(seed, vec![file(WAYS, GOOD)]);
        w.restore_home();
        let mut kept: Option<DotTemperRevision> = None;
        let mut delayed = None;
        let mut expected = BTreeMap::new();
        expected.insert(w.tip(), GOOD);
        for _ in 0..60_u32 {
            let choice = rng.below(8);
            let mut final_answer = None;
            if choice == 0 {
                let bytes: &[u8] = if rng.chance(400) {
                    b"format = 2\n"
                } else if rng.chance(500) {
                    GOOD
                } else {
                    b"format = 1\n# change\n"
                };
                let head = w.advance(WAYS, bytes);
                expected.insert(head, bytes);
                w.trigger(Some(head));
            } else if choice == 1 {
                w.restart(kept.clone());
                delayed = None; // Old client tokens belong to the ended engine.
            } else if choice == 2 {
                w.trigger(None);
            } else if let Some((call, read, result)) = delayed.take() {
                if choice == 3 && matches!(read, Read::File { .. }) {
                    w.answer(call, Err(Error::Forbidden));
                } else {
                    if let Ok(Answer::File { head, path, .. }) = &result
                        && path.as_ref() == WAYS
                    {
                        final_answer = Some(*head);
                    }
                    w.answer(call, result);
                }
            } else if let Some((call, repository, read)) = w.next() {
                let result = w.answer_for(repository, &read);
                delayed = Some((call, read, result));
            }
            if final_answer.is_some() {
                assert_eq!(w.saved.len(), 1, "seed {seed}: a whole tree takes exactly one observation");
            }
            for new in w.saved.drain(..) {
                let previous = kept.as_ref();
                assert!(new.observed > previous.map_or(0, |old| old.observed), "seed {seed}");
                assert!(new.number >= previous.map_or(0, |old| old.number), "seed {seed}");
                if let Some(head) = final_answer {
                    let body = expected[&head];
                    assert_eq!(new.refused.is_some(), body.starts_with(b"format = 2"), "seed {seed}");
                    if new.refused.is_none() {
                        assert_eq!(new.files[0].bytes.as_ref(), body, "seed {seed}");
                        let changed = previous.is_none_or(|old| old.files != new.files);
                        let wanted_commit = if changed { Some(head) } else { previous.and_then(|old| old.commit) };
                        assert_eq!(new.commit, wanted_commit, "seed {seed}");
                        assert_eq!(
                            new.number,
                            previous.map_or(0, |old| old.number) + u64::from(changed),
                            "seed {seed}"
                        );
                    }
                }
                if let Some(refused) = &new.refused {
                    assert_eq!(new.observed, refused.observed, "seed {seed}");
                    assert_eq!(new.number, previous.map_or(0, |old| old.number), "seed {seed}");
                    assert_eq!(new.commit, previous.and_then(|old| old.commit), "seed {seed}");
                }
                kept = Some(new);
            }
            assert_eq!(w.d.temper_revision(PROJECT), kept.as_ref(), "seed {seed}");
        }
    }
}
