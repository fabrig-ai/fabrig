//! Largest values at the ceilings, or at the largest clamp under the cap.

#![cfg(test)]

use super::codec_memory::family;
use skein_codec_tests::every_v3::Record;

const CAP: u64 = 64 * 1024 * 1024;

macro_rules! every {
    ($module:path) => {
        family!(
            CAP,
            skein_codec_tests::EVERY_SCHEMA,
            $module,
            roots [Every, Kinds, Lists, Options, Grouped, Held, LongRecord]
        );
    };
}

#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(CAP, skein_codec_tests::SAMPLE_SCHEMA, skein_codec_tests::v1, roots [Choice, Marker]);
    every!(skein_codec_tests::every_v3);
}
