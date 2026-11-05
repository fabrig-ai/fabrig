//! Hostile document mutations (hearth/docs/design/domain/testing.md, 2.9).
use skein_lib::Rng;
const HOSTILE: &[&[u8]] = &[
    br#"{"a":1,"a":2}"#,
    br#"{"x":{"a":1,"a":2}}"#,
    br#"{"a":1.1}"#,
    br#"{"a":1e2}"#,
    br#"{"a":9223372036854775808}"#,
    br#"{"a":-9223372036854775809}"#,
    br#"{"a":"\u0000"}"#,
    br#"{"\u0000":1}"#,
    br#"{"_note":null}"#,
    b"[]",
    b"null",
    b"{",
    br#"{"a":"\q"}"#,
    br#"{"a":[1,]}"#,
    br#"{"a":[[[[[0]]]]]}"#,
    br#"{"a":0,"b":0,"c":0,"d":0,"e":0}"#,
];
#[must_use]
pub fn mutate(rng: &mut Rng, bytes: &[u8]) -> Vec<u8> {
    match rng.below(4) {
        0 | 1 => HOSTILE[usize::try_from(rng.below(HOSTILE.len() as u64)).expect("draw fits usize")].to_vec(),
        2 => bytes[..usize::try_from(rng.below(bytes.len() as u64 + 1)).expect("draw fits usize")].to_vec(),
        3 => {
            let mut out = bytes.to_vec();
            if !out.is_empty() {
                let i = usize::try_from(rng.below(out.len() as u64)).expect("draw fits usize");
                out[i] = b'!';
            }
            out
        }
        _ => unreachable!(),
    }
}
