use crate::{fixtures, referee::secrets};

#[test]
fn a_secret_in_a_root_fails_the_referee_naming_it() {
    let root = format!("root {:?}", fixtures::LLM_TOKEN);
    for (name, bytes, expected) in [
        ("formatted root", root.into_bytes(), Some("LLM access token")),
        ("binary frame", [b"\xff".as_slice(), fixtures::LLM_TOKEN, b"\x80"].concat(), Some("LLM access token")),
        ("interrupted token", [b"test-\xfftoken".as_slice()].concat(), None),
    ] {
        assert_eq!(secrets::in_bytes(&bytes), expected, "{name}");
    }
}
