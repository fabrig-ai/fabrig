use super::*;
use crate::Kind;
use skein_lib::sha256::digest;
// Sixteen fake bytes, with URL, html and Debug escapes all differing from the bytes.
const VALUE: &[u8] = b"ab/+=\"<&\\sample!";
const HIDDEN: &[u8] = b"xxxxxxxxxxxxxxxx";
const UPPER: &[u8] = b"ab%2F%2B%3D%22%3C%26%5Csample%21";
const LOWER: &[u8] = b"ab%2f%2b%3d%22%3c%26%5csample%21";
const HTML: &[u8] = b"ab/+=&quot;&lt;&amp;\\sample!";
const DEBUG: &[u8] = b"ab/+=\\\"<&\\\\sample!";

fn framed(value: &[u8]) -> Box<[u8]> {
    // A counted field between two neighbours, repeated to check every occurrence.
    [
        b"before".as_slice(),
        &u32::try_from(value.len()).expect("fixture length").to_le_bytes(),
        value,
        b"between",
        value,
        b"after",
    ]
    .concat()
    .into()
}
fn query(value: &[u8]) -> Box<[u8]> {
    [b"/?value=".as_slice(), value, b"&tail=yes"].concat().into()
}
#[test]
fn candidate_kinds_redact_each_form_in_each_framing() {
    for kind in [Kind::Approval, Kind::Credential, Kind::Bearer] {
        let value = [b"ab/+=".as_slice(), b"\"<&\\sample!"].concat();
        let originals = Originals::new(vec![(kind, value.into())], &[]);
        for form in [VALUE, UPPER, LOWER, HTML, DEBUG] {
            for framing in [Framing::Framed, Framing::Text] {
                let hidden = match framing {
                    Framing::Framed => vec![b'x'; form.len()],
                    Framing::Text => HIDDEN.to_vec(),
                };
                let (mut field, expected, places) = match framing {
                    Framing::Framed => (framed(form), framed(&hidden), 2),
                    Framing::Text => (query(form), query(&hidden), 1),
                };
                assert_eq!(crate::redact_field(&mut field, framing, &originals), places, "{kind:?}");
                assert_eq!(field, expected, "{kind:?}");
            }
        }
        for framing in [Framing::Framed, Framing::Text] {
            let mut field = framed(&digest(VALUE));
            assert_eq!(crate::redact_field(&mut field, framing, &originals), 2, "{kind:?} digest");
            assert_eq!(field, framed(&digest(HIDDEN)), "{kind:?} placeholder digest");
        }
    }
}
