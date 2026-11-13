//! A goal's red bounds are refused by name at start (domain/forge/reds.md, 7).
use super::fixtures;
use crate::startup::{Problems, application, lines};
use jig_protocol_config::{self as config, read};

#[test]
#[expect(clippy::disallowed_macros, reason = "the test formats configuration inputs and expected refusal paths")]
fn red_limit_overrides_outside_the_range_refuse_start_naming_the_limit() {
    const DOCUMENT: config::DocumentLimits = config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 };
    for name in ["forge.branch_reds", "forge.red_key_bytes"] {
        for value in [0_u64, 1025, u64::MAX] {
            let input = fixtures::with(
                &fixtures::document(),
                b"application.limits.overrides",
                format!("{{\"{name}\":{value}}}").as_bytes(),
            );
            let document = config::document(&input, &DOCUMENT).unwrap();
            let at = read::Object::new(&document, 0).unwrap().member(&document, b"application").unwrap();
            let mut problems = Problems::empty();
            let members = application::parse(
                &document,
                at,
                &fixtures::engine().jig,
                &config::Directories::working(),
                &mut problems,
            );
            assert!(members.is_none());
            let expected = format!("application.limits.overrides.{name}: outside its type");
            assert!(skein_lib::bytes::find(&lines(&problems), expected.as_bytes()).is_some(), "{:?}", lines(&problems));
        }
    }
}
