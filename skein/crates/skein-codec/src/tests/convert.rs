//! Shared scalar maps (data/codec/maps.md, 5).

use alloc::boxed::Box;
use skein_lib::{Duration, Time, Token, Wall};

use crate::{Reason, convert};

#[test]
fn scalar_pairs_preserve_every_raw_value() {
    for raw in [0, 1, u64::MAX] {
        let wall = Wall::from_nanos(raw);
        assert_eq!(convert::wall_to_record(&wall), Ok(raw));
        assert_eq!(convert::wall_to_domain(raw), Ok(wall));
        let time = Time::from_nanos(raw);
        assert_eq!(convert::time_to_record(&time), Ok(raw));
        assert_eq!(convert::time_to_domain(raw), Ok(time));
        let duration = Duration::from_nanos(raw);
        assert_eq!(convert::duration_to_record(&duration), Ok(raw));
        assert_eq!(convert::duration_to_domain(raw), Ok(duration));
        let token = Token::new(raw);
        assert_eq!(convert::token_to_record(&token), Ok(raw));
        assert_eq!(convert::token_to_domain(raw), Ok(token));
    }
}

#[test]
fn text_pair_preserves_utf8_and_refuses_other_bytes() {
    for text in ["", "maps", "é🦀"] {
        let text = Box::<str>::from(text);
        let bytes = convert::text_to_record(&text).expect("text encodes");
        assert_eq!(bytes.as_ref(), text.as_bytes());
        assert_eq!(convert::text_to_domain(bytes), Ok(text));
    }
    for bytes in [&[0xff][..], &[0xc3][..], &[0xc0, 0xaf][..]] {
        assert_eq!(convert::text_to_domain(Box::from(bytes)), Err(Reason::Utf8));
    }
}
