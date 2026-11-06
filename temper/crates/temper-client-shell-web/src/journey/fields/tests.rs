use super::*;
use jig_client_shell_web::adapter::Heard;
use skein_lib::bytes::find;

const MARKER: &[u8] = b"marker";

fn rows() -> Vec<(&'static str, Vec<Framing>)> {
    use Framing::{Framed as F, Text as T};
    vec![
        ("journey 2", vec![]),
        ("seed 1", vec![]),
        ("clocks 0 0", vec![]),
        ("primary control", vec![]),
        ("zone 0", vec![]),
        ("ways \"marker\" \"marker\"", vec![T, T]),
        ("forge \"marker\"", vec![T]),
        ("path \"marker\"", vec![T]),
        ("secret \"marker\"", vec![T]),
        ("stored \"marker\" \"marker\"", vec![F, F]),
        ("turn 0 0", vec![]),
        ("start", vec![]),
        ("due", vec![]),
        ("place \"marker\"", vec![T]),
        ("heard answered 1 200 \"marker\"", vec![F]),
        ("heard failed 1", vec![]),
        ("heard opened", vec![]),
        ("heard refused 409", vec![]),
        ("heard piece \"stream\"", vec![]),
        ("heard ended", vec![]),
        ("carried 1", vec![]),
        ("written \"marker\"", vec![F]),
        ("unwritten \"marker\"", vec![F]),
        ("lost 1", vec![]),
        ("act press \"marker\" \"marker\"", vec![T, T]),
        ("act submit \"marker\" \"marker\"", vec![T, T]),
        ("act toggle \"marker\" \"marker\"", vec![T, T]),
        ("act input \"marker\" \"marker\" \"marker\"", vec![T, T, T]),
        ("act drawing \"marker\" \"marker\" \"marker\"", vec![T, T, T]),
        (
            "exchange 1 64 POST \"marker\" \"marker\" \"marker\" \"marker\" \"marker\" \"marker\"",
            vec![T, F, T, T, T, T],
        ),
        ("open 64 \"marker\" \"marker\" \"marker\" \"marker\" \"marker\"", vec![T, T, T, T, T]),
        ("close", vec![]),
        ("save \"marker\" \"marker\"", vec![F, F]),
        ("erase \"marker\"", vec![F]),
        ("writes 1", vec![]),
        ("navigate \"marker\"", vec![T]),
        ("replace \"marker\"", vec![T]),
        ("show \"marker\" push", vec![T]),
        ("deadline none", vec![]),
        ("mark \"marker\"", vec![T]),
        ("mark", vec![]),
        ("end \"marker\"", vec![T]),
        ("end", vec![]),
    ]
}

#[test]
fn every_field_is_listed() {
    for (text, framing) in rows() {
        let mut line = Line::read(text.as_bytes()).expect("row");
        let fields = fields_mut(&mut line);
        assert_eq!(fields.len(), framing.len(), "field count");
        for (bytes, _) in fields {
            bytes.fill(b'x');
        }
        let mut out = Vec::new();
        line.write(&mut out);
        assert!(find(&out, MARKER).is_none(), "each byte field was visited");
    }
    for heard in [Heard::Stored { key: MARKER.into() }, Heard::NotStored { key: MARKER.into() }] {
        assert!(fields_mut(&mut Line::Heard(heard)).is_empty(), "storage reports are carried, not byte fields");
    }
}

#[test]
fn framing_is_the_contracts() {
    for (text, expected) in rows() {
        let mut line = Line::read(text.as_bytes()).expect("row");
        let framing: Vec<_> = fields_mut(&mut line).into_iter().map(|(_, framing)| framing).collect();
        assert_eq!(framing, expected, "field framing in recording order");
    }
}
