//! Shell-independent redaction (temper/docs/design/client/web/journeys.md, 3.6 and 3.7).
use temper_client_redaction::{Framing, Kind, Originals, redact_field, scan};

#[test]
fn a_framed_original_is_redacted_and_no_longer_found() {
    let originals = Originals::new(vec![(Kind::Bearer, b"abc123secret".as_slice().into())], &[]);
    let before: Box<[u8]> = [b"before\x0c\0\0\0".as_slice(), b"abc123secret", b"after"].concat().into();
    let mut field = before.clone();
    assert_eq!(redact_field(&mut field, Framing::Framed, &originals), 1);
    assert_eq!(field.as_ref(), b"before\x0c\0\0\0xxxxxxxxxxxxafter");
    assert_eq!(field.len(), before.len());
    assert!(!scan::original(&field, &originals));
    assert!(scan::original(&before, &originals));
}
