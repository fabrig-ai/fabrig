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

#[test]
fn each_box_stage_reads_and_prints_back() {
    use crate::{BoxEnd, BoxStage};
    let rows: &[(&[u8], BoxStage)] = &[
        (b"finding", BoxStage::Finding),
        (b"fetching 5 -", BoxStage::Fetching { done: 5, total: None }),
        (b"fetching 0 18446744073709551615", BoxStage::Fetching { done: 0, total: Some(u64::MAX) }),
        (b"starting", BoxStage::Starting),
        (b"running", BoxStage::Running),
        (b"stopping", BoxStage::Stopping),
        (b"ended exited 2", BoxStage::Ended { end: BoxEnd::Exited { code: 2 } }),
        (b"ended killed 255", BoxStage::Ended { end: BoxEnd::Killed { signal: 255 } }),
        (b"failed no link", BoxStage::Failed { why: copy_of(b"no link") }),
        (b"failed \xff\x80", BoxStage::Failed { why: copy_of(b"\xff\x80") }),
        (b"failed ", BoxStage::Failed { why: Box::default() }),
    ];
    for (bytes, stage) in rows {
        assert_eq!(stage.print().as_ref(), *bytes, "print {stage:?}");
        assert_eq!(BoxStage::read(bytes).as_ref(), Some(stage), "read {stage:?}");
    }
    for bytes in [
        b"fetching 5".as_slice(),
        b"ended",
        b"stopped",
        b"",
        b"finding ",
        b"starting now",
        b"running x",
        b"stopping x",
        b"fetching - 5",
        b"fetching 5 x",
        b"fetching 5 - extra",
        b"fetching 18446744073709551616 -",
        b"fetching 05 -",
        b"fetching 5 00",
        b"ended exited 02",
        b"ended exited 256",
        b"ended killed -1",
        b"ended killed 9 extra",
        b"ended other 2",
        b"failed",
    ] {
        assert_eq!(BoxStage::read(bytes), None, "reject {bytes:?}");
    }
}
