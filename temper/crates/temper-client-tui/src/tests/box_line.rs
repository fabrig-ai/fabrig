//! The box's lines (temper/docs/design/client/tui/box.md).
use crate::BoxLine;
use skein_lib::{
    Secret,
    bytes::{concat, copy_of},
};

#[test]
fn each_box_line_reads_and_prints_back() {
    let printed = b"http://example.invalid/the-printed-link";
    let longest = concat(&[b"log ", &[b'x'; BoxLine::MOST - 5]]);
    let too_long = concat(&[&longest, b"x"]);
    let rows: &[(&str, &[u8], Option<BoxLine>)] = &[
        ("the printed link", &concat(&[b"link ", printed]), Some(BoxLine::Link { url: Secret::new(copy_of(printed)) })),
        ("log", b"log a line", Some(BoxLine::Log { line: copy_of(b"a line") })),
        ("restarting", b"restarting forgejo-3", Some(BoxLine::Restarting { step: copy_of(b"forgejo-3") })),
        ("running", b"running forgejo-3", Some(BoxLine::Running { step: copy_of(b"forgejo-3") })),
        ("gave up", b"gave-up forgejo-3", Some(BoxLine::GaveUp { step: copy_of(b"forgejo-3") })),
        ("spaces kept", b"log  a line ", Some(BoxLine::Log { line: copy_of(b" a line ") })),
        ("empty field", b"log ", None),
        ("unknown word", b"stop forgejo-3", None),
        ("word alone", b"log", None),
        ("no space", b"logx", None),
        ("last fitting line", &longest, Some(BoxLine::Log { line: copy_of(&longest[4..]) })),
        ("past the bound", &too_long, None),
        ("non UTF-8", b"log \xff\x80", Some(BoxLine::Log { line: copy_of(b"\xff\x80") })),
    ];
    let mut failures = skein_lib::List::with_capacity(u32::try_from(rows.len()).unwrap());
    for (name, bytes, want) in rows {
        let read = BoxLine::read(bytes);
        let roundtrip = match &read {
            Some(line) => line.print().as_ref() == *bytes,
            None => true,
        };
        if read != *want || !roundtrip {
            failures.push(*name).unwrap();
        }
    }
    assert!(failures.is_empty(), "box line rows: {failures:?}");
}
