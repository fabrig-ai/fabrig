//! Pure list reading (skein/docs/design/llm/README.md, 4.9).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "table fixtures use ordinary Rust to assemble bounded JSON bodies"
)]

// The field and row bounds are skein/docs/design/llm/README.md, 4.9's.
const NAME_BYTES: usize = 128;
const SHOWN_BYTES: usize = 64;
const ROWS: usize = 256;
const EMPTY_LIST: &[u8] = br#"{"data":[]}"#;

use crate::Provider::{Anthropic, OpenAi};
use crate::list::{Limits, Listed, Unreadable, parse, target};

fn named(name: &[u8]) -> Listed {
    Listed { name: name.into(), shown: None, window: None, output: None, efforts: Box::new([]) }
}

fn rows(rows: &[Listed]) -> Box<[Listed]> {
    rows.into()
}

#[test]
fn route_lists() {
    let mut whole = named(b"claude");
    whole.shown = Some(b"Claude".as_slice().into());
    whole.window = Some(200_000);
    whole.output = Some(64_000);
    whole.efforts = [
        b"low".as_slice().into(),
        b"medium".as_slice().into(),
        b"high".as_slice().into(),
        b"xhigh".as_slice().into(),
        b"max".as_slice().into(),
    ]
    .into();
    let mut visible = named(b"gpt");
    visible.shown = Some(b"GPT".as_slice().into());
    let mut no_high = named(b"efforts");
    no_high.efforts = [b"low".as_slice().into(), b"max".as_slice().into()].into();
    let mut maximum = named(b"bounds");
    maximum.shown = Some(vec![b's'; SHOWN_BYTES].into());
    maximum.window = Some(u32::MAX);
    maximum.output = Some(1);
    let mut first = Vec::new();
    let mut many = Vec::new();
    for number in 0..=ROWS {
        many.push(format!(r#"{{"id":"row-{number}"}}"#));
        if number < ROWS {
            first.push(named(format!("row-{number}").as_bytes()));
        }
    }
    // label, route, body, limits, expected rows; every row exercises the same parser boundary.
    let cases = [
        ("plan visibility", OpenAi, br#"{"models":[{"slug":"gpt","display_name":"GPT","visibility":"list"},{"slug":"hidden","visibility":"hide"},{"slug":"missing"}]}"#.to_vec(), Limits::DEFAULT, Ok(rows(&[visible]))),
        ("key names", OpenAi, br#"{"data":[{"id":"gpt"},{"id":"other"}]}"#.to_vec(), Limits::DEFAULT, Ok(rows(&[named(b"gpt"), named(b"other")]))),
        ("anthropic whole", Anthropic, br#"{"data":[{"id":"claude","display_name":"Claude","max_input_tokens":200000,"max_tokens":64000,"capabilities":{"effort":{"max":{"supported":true},"high":{"supported":true},"low":{"supported":true},"xhigh":{"supported":true},"medium":{"supported":true}}}}]}"#.to_vec(), Limits::DEFAULT, Ok(rows(&[whole]))),
        ("unsupported effort", Anthropic, br#"{"data":[{"id":"efforts","capabilities":{"effort":{"max":{"supported":true},"high":{"supported":false},"low":{"supported":true},"unknown":{"supported":true}}}}]}"#.to_vec(), Limits::DEFAULT, Ok(rows(&[no_high]))),
        ("no name", OpenAi, br#"{"data":[{}, {"display_name":"absent"}, {"id":2}, {"id":"kept"}]}"#.to_vec(), Limits::DEFAULT, Ok(rows(&[named(b"kept")]))),
        ("name 129 bytes", Anthropic, format!(r#"{{"data":[{{"id":"{}"}},{{"id":"kept"}}]}}"#, "n".repeat(NAME_BYTES + 1)).into_bytes(), Limits::DEFAULT, Ok(rows(&[named(b"kept")]))),
        ("name 128 bytes", OpenAi, format!(r#"{{"data":[{{"id":"{}"}}]}}"#, "n".repeat(NAME_BYTES)).into_bytes(), Limits::DEFAULT, Ok(rows(&[named(&[b'n'; NAME_BYTES])]))),
        ("shown 65 bytes", Anthropic, format!(r#"{{"data":[{{"id":"kept","display_name":"{}"}}]}}"#, "s".repeat(SHOWN_BYTES + 1)).into_bytes(), Limits::DEFAULT, Ok(rows(&[named(b"kept")]))),
        ("numeric bounds", Anthropic, br#"{"data":[{"id":"zero","max_input_tokens":0,"max_tokens":0},{"id":"large","max_input_tokens":4294967296,"max_tokens":18446744073709551615}]}"#.to_vec(), Limits::DEFAULT, Ok(rows(&[named(b"zero"), named(b"large")]))),
        ("bounds included", Anthropic, format!(r#"{{"data":[{{"id":"bounds","display_name":"{}","max_input_tokens":4294967295,"max_tokens":1}}]}}"#, "s".repeat(SHOWN_BYTES)).into_bytes(), Limits::DEFAULT, Ok(rows(&[maximum]))),
        ("first 256", OpenAi, format!(r#"{{"data":[{}]}}"#, many.join(",")).into_bytes(), Limits::DEFAULT, Ok(first.into())),
        ("row bound counts kept", OpenAi, br#"{"models":[{"slug":"hide","visibility":"hide"},{"slug":"one","visibility":"list"},{"slug":"two","visibility":"list"}]}"#.to_vec(), Limits { rows: 1, ..Limits::DEFAULT }, Ok(rows(&[named(b"one")]))),
        ("zero rows", OpenAi, br#"{"data":[{"id":"one"}]}"#.to_vec(), Limits { rows: 0, ..Limits::DEFAULT }, Ok(rows(&[]))),
        ("over body", OpenAi, EMPTY_LIST.to_vec(), Limits { body: u32::try_from(EMPTY_LIST.len() - 1).unwrap(), ..Limits::DEFAULT }, Err(Unreadable)),
        ("at body", OpenAi, EMPTY_LIST.to_vec(), Limits { body: u32::try_from(EMPTY_LIST.len()).unwrap(), ..Limits::DEFAULT }, Ok(rows(&[]))),
        ("neither member", OpenAi, br#"{"other":[]}"#.to_vec(), Limits::DEFAULT, Err(Unreadable)),
        ("not JSON", Anthropic, b"not JSON".to_vec(), Limits::DEFAULT, Err(Unreadable)),
        ("empty list", Anthropic, EMPTY_LIST.to_vec(), Limits::DEFAULT, Ok(rows(&[]))),
        ("unknown members", Anthropic, br#"{"unknown":{"data":[{"id":"nested"}]},"data":[{"unknown":{"id":"wrong","max_tokens":17},"id":"kept","capabilities":{"unknown":true}}],"tail":[null,{},[false]]}"#.to_vec(), Limits::DEFAULT, Ok(rows(&[named(b"kept")]))),
        ("dense unknown", OpenAi, format!(r#"{{"data":[],"unknown":[{}]}}"#, "[] ,".repeat(400).trim_end_matches(',')).into_bytes(), Limits::DEFAULT, Ok(rows(&[]))),
        ("deep unknown", Anthropic, format!(r#"{{"data":[],"unknown":{}0{}}}"#, "[".repeat(100), "]".repeat(100)).into_bytes(), Limits::DEFAULT, Ok(rows(&[]))),
        ("wrong optional kinds", Anthropic, br#"{"data":[{"id":"kept","display_name":4,"max_input_tokens":"20","max_tokens":-1,"capabilities":{"effort":{"low":{"supported":"true"}}}}]}"#.to_vec(), Limits::DEFAULT, Ok(rows(&[named(b"kept")]))),
        ("wrong rows kind", OpenAi, br#"{"data":{}}"#.to_vec(), Limits::DEFAULT, Err(Unreadable)),
        ("anthropic requires data", Anthropic, br#"{"models":[]}"#.to_vec(), Limits::DEFAULT, Err(Unreadable)),
        ("skip nonobjects", Anthropic, br#"{"data":[null,4,[{"id":"nested"}],{"id":"kept"}]}"#.to_vec(), Limits::DEFAULT, Ok(rows(&[named(b"kept")]))),
        ("JSON beyond row bound", OpenAi, br#"{"data":[{"id":"one"}, broken]}"#.to_vec(), Limits { rows: 1, ..Limits::DEFAULT }, Err(Unreadable)),
    ];
    for (label, provider, body, limits, expected) in cases {
        assert_eq!(
            target(provider),
            match provider {
                OpenAi => b"/v1/models".as_slice(),
                Anthropic => b"/v1/models?limit=1000",
            }
        );
        assert_eq!(parse(provider, &body, &limits), expected, "{label}");
    }
}
