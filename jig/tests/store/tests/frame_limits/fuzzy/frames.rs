//! Public decode against fixture bytes (`jig/docs/design/domain/testing-workloads.md`, 7).
use crate::frame_reads;
use frame_reads::{decode, turn};
use jig_test_layouts::frames;
use skein_lib::{Rng, length::len_u64};
use skein_world::{
    sweep::{Case, cases},
    this_test,
};
use std::convert::Infallible;

const FRAMES: u32 = 512;
const SEED: u64 = 0x0003_5a17_2026;

#[test]
fn random_fixture_frames_match_converted_bytes_or_unread() -> Result<(), Case<Infallible>> {
    cases(this_test!("random_fixture_frames_match_converted_bytes_or_unread"), [SEED], |seed| {
        let (mut component, env) = frame_reads::closed(frame_reads::SETS, &jig_store_world::root::limits());
        frame_reads::open(&mut component, &env);
        let mut rng = Rng::new(seed);
        for frame in 0..FRAMES {
            let body = u16::try_from(4 + rng.below(4)).expect("body version");
            let version = u16::try_from(rng.below(4)).expect("turn version");
            let said = vec![b's'; usize::try_from(rng.below(65)).expect("said")];
            let replay = vec![b'r'; usize::try_from(rng.below(65)).expect("replay")];
            let truncate_old_field = rng.below(4) == 0;
            let corrupt = rng.below(8) == 0;
            let short = rng.below(8) == 0;
            let mut transcript = turn(version, &said, &replay, frame);
            if short {
                transcript = transcript[..usize::try_from(rng.below(2)).expect("short")].into();
            }
            if version == 1 && truncate_old_field && !short {
                transcript = transcript[..transcript.len() - 1].into();
            }
            let mut raw = frames::frame(body, &transcript);
            if corrupt {
                let index = usize::try_from(rng.below(len_u64(raw.len()))).expect("index");
                raw[index] ^= 1;
            }
            let valid = [5, 6].contains(&body)
                && [1, 2].contains(&version)
                && !short
                && !corrupt
                && (version == 2 || !truncate_old_field);
            let expected = if valid { Some(turn(2, &said, &[], if version == 1 { 0 } else { frame })) } else { None };
            assert_eq!(decode(&mut component, &env, &raw), expected, "frame {frame}, body {body}, turn {version}");
        }
        Ok(())
    })
}
