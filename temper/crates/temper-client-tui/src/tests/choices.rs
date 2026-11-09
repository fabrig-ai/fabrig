use crate::{ReplyChoice, ReplyChoices, reply_choices};

fn choices(rows: &[(&[u8], Option<&[u8]>)]) -> Option<ReplyChoices> {
    Some(ReplyChoices {
        choices: rows.iter().map(|(words, reason)| ReplyChoice {
            words: (*words).into(),
            reason: reason.map(Into::into),
        }).collect(),
    })
}

#[test]
fn reply_choices_read_a_list_that_ends_in_tell_me_something_else() {
    let two = || choices(&[(b"Move it at once", None), (b"Side by side", None)]);
    for (name, text, expected) in [
        ("drafter", "One point is yours.\n\n1. Move it at once — no mixed state\n2. Side by side\n3. Tell me something else\n \t", choices(&[(b"Move it at once", Some(b"no mixed state")), (b"Side by side", None)])),
        ("colon", "1. Move it at once: no mixed state\n2. Side by side: fallback\n3. Tell me something else", choices(&[(b"Move it at once", Some(b"no mixed state")), (b"Side by side", Some(b"fallback"))])),
        ("no reason", "1. Move it at once\n2. Side by side\n3. Tell me something else", two()),
        ("it and stop", "1. Move it at once\n2. Side by side\n3. Tell it something else.", two()),
        ("not at end", "1. Move it at once\n2. Side by side\n3. Tell me something else\n\nAnother paragraph.", None),
        ("no escape", "1. Move it at once\n2. Side by side", None),
        ("one item", "1. Tell me something else", None),
        ("not from one", "2. Move it at once\n3. Tell me something else", None),
        ("text after list", "1. Move it at once\n2. Tell me something else\nMore", None),
        ("bold", "1. **Move it at once** — __no mixed state__\n2. __Side by side__\n3. **Tell me something else**", choices(&[(b"Move it at once", Some(b"no mixed state")), (b"Side by side", None)])),
        ("en dash", "1. Move it at once – no mixed state\n2. Tell me something else", choices(&[(b"Move it at once", Some(b"no mixed state"))])),
        ("hyphen", "1. Move it at once - no mixed state\n2. Tell me something else", choices(&[(b"Move it at once", Some(b"no mixed state"))])),
        ("first separator", "1. Move it at once: no mixed state — now\n2. Tell me something else", choices(&[(b"Move it at once", Some("no mixed state — now".as_bytes()))])),
        ("gap", "1. Move it at once\n3. Tell me something else", None),
        ("duplicate", "1. Move it at once\n1. Side by side\n2. Tell me something else", None),
        ("indented", "   1. Move it at once\n   2. Side by side\n   3. Tell me something else", two()),
        ("continuation", "1. Move it at once\n    A second line.\n2. Side by side\n3. Tell me something else", two()),
        ("code indent", "    1. Move it at once\n    2. Tell me something else", None),
        ("CRLF", "1. Move it at once\r\n2. Side by side\r\n3. Tell me something else.\r\n", two()),
        ("empty", "", None),
    ] {
        assert_eq!(reply_choices(text.as_bytes()), expected, "{name}");
    }
}
