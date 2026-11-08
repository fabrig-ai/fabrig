use crate::views::{Part, parts};

#[test]
#[expect(clippy::disallowed_methods, reason = "the table borrows ASCII fixture bytes as text for the splitter")]
fn only_top_level_one_line_view_fences_split_text() {
    let long = address_text(513);
    let long = core::str::from_utf8(&long).unwrap();
    let at_address_bound = address_text(512);
    let at_address_bound = core::str::from_utf8(&at_address_bound).unwrap();
    let limit = "```view\na\n```\n".repeat(17);
    let at_limit = "```view\na\n```\n".repeat(16);
    let mut bounded = [Part::View(b"a"); 16];
    *bounded.last_mut().unwrap() = Part::Text(&limit[15 * "```view\na\n```\n".len()..]);
    let rows: &[(&str, &str, &[Part<'_>])] = &[
        ("one", "```view\nway Free\n```", &[Part::View(b"way Free")]),
        ("two", "```view\na\n```\n~~~view\nb\n~~~", &[Part::View(b"a"), Part::View(b"b")]),
        ("paragraphs", "Before\n\n```view\n a \n```\n\nAfter", &[Part::Text("Before\n\n"), Part::View(b"a"), Part::Text("\nAfter")]),
        ("two lines", "```view\na\nb\n```", &[Part::Text("```view\na\nb\n```")]),
        ("indented", " ```view\na\n```", &[Part::Text(" ```view\na\n```")]),
        ("list", "- ```view\n  a\n  ```", &[Part::Text("- ```view\n  a\n  ```")]),
        ("in text fence", "````text\n```view\na\n```\n````", &[Part::Text("````text\n```view\na\n```\n````")]),
        ("tilde", "~~~view\na\n~~~~", &[Part::View(b"a")]),
        ("unclosed", "```view\na", &[Part::Text("```view\na")]),
        ("mermaid", "```mermaid\na\n```", &[Part::Text("```mermaid\na\n```")]),
        ("address at bound", at_address_bound, &[Part::View(&at_address_bound.as_bytes()[8..520])]),
        ("indented close", "```view\na\n ```", &[Part::Text("```view\na\n ```")]),
        ("quoted before view", "> ```view\n> a\n> ```\n```view\nb\n```", &[Part::Text("> ```view\n> a\n> ```\n"), Part::View(b"b")]),
        ("long address", long, &[Part::Text(long)]),
        ("at bound", &at_limit, &[Part::View(b"a"); 16]),
        ("past bound", &limit, &bounded),
        ("empty", "", &[]),
        ("blank address", "```view\n \n```", &[Part::Text("```view\n \n```")]),
        ("blank body line", "```view\na\n\n```", &[Part::Text("```view\na\n\n```")]),
        ("trim info CRLF", "``` view \r\n a \r\n```\r\n", &[Part::View(b"a")]),
        ("short close", "````view\na\n```", &[Part::Text("````view\na\n```")]),
        ("unlike close", "```view\na\n~~~", &[Part::Text("```view\na\n~~~")]),
        ("close with info", "```view\na\n```text", &[Part::Text("```view\na\n```text")]),
        ("other tag", "```views\na\n```", &[Part::Text("```views\na\n```")]),
    ];
    for (name, text, expected) in rows {
        assert_eq!(parts(text).as_slice(), *expected, "{name}");
    }
}

fn address_text(bytes: usize) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(bytes.checked_add(b"```view\n\n```".len()).unwrap());
    writer.put(b"```view\n").unwrap();
    for _ in 0..bytes { writer.put(b"a").unwrap(); }
    writer.put(b"\n```").unwrap();
    writer.finish()
}
