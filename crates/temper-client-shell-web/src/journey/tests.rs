use super::*;
use jig_client_shell_web::adapter::Heard;
use skein_ui::Act;

#[test]
fn every_record_reads_back_as_written() {
    let bytes = || Box::from(b"bytes".as_slice());
    let headers = || vec![(Box::from(b"name".as_slice()), Box::from(b"value".as_slice()))];
    let control = || Control { role: Role::TextField, name: b"Name".as_slice().into(), order: 1 };
    let rows = [
        Line::Journey { version: 1 },
        Line::Seed(u64::MAX),
        Line::Clocks { now: Time::ZERO, wall: Wall::EPOCH },
        Line::Primary(Primary::Control),
        Line::Primary(Primary::Meta),
        Line::Ways { content: bytes(), origin: bytes() },
        Line::Forge(bytes()),
        Line::Path(bytes()),
        Line::Secret(bytes()),
        Line::Stored { key: bytes(), value: bytes() },
        Line::Turn { now: Time::from_nanos(u64::MAX), wall: Wall::from_nanos(u64::MAX) },
        Line::Start,
        Line::Due,
        Line::Place(bytes()),
        Line::Heard(Heard::Answered { exchange: u32::MAX, status: 200, body: bytes() }),
        Line::Heard(Heard::Failed { exchange: 2 }),
        Line::Heard(Heard::StreamOpened),
        Line::Heard(Heard::StreamRefused { status: 403 }),
        Line::Heard(Heard::Piece { bytes: bytes() }),
        Line::Heard(Heard::StreamEnded),
        Line::Carried { writes: 2 },
        Line::Written(bytes()),
        Line::Unwritten(bytes()),
        Line::Lost(3),
        Line::Act { control: control(), act: Act::Press },
        Line::Act { control: Control { role: Role::Heading { level: 3 }, ..control() }, act: Act::Press },
        Line::Act { control: control(), act: Act::Submit },
        Line::Act { control: control(), act: Act::Toggle },
        Line::Act { control: control(), act: Act::Input { value: b"ab".as_slice().into() } },
        Line::Act { control: control(), act: Act::Drawing { data: bytes() } },
        Line::Exchange {
            id: 4,
            bound: 100,
            method: b"POST".as_slice().into(),
            path: bytes(),
            body: bytes(),
            headers: headers(),
        },
        Line::Open { piece: 100, path: bytes(), headers: headers() },
        Line::Close,
        Line::Save { key: bytes(), value: bytes() },
        Line::Erase(bytes()),
        Line::Writes(2),
        Line::Navigate(bytes()),
        Line::Replace(bytes()),
        Line::Show { path: b"/accounts".as_slice().into(), replace: false },
        Line::Show { path: bytes(), replace: true },
        Line::Deadline(None),
        Line::Deadline(Some(Time::ZERO)),
        Line::Mark(Some(bytes())),
        Line::Mark(None),
        Line::End(Some(bytes())),
        Line::End(None),
    ];
    for line in rows {
        let mut out = Vec::new();
        line.write(&mut out);
        assert_eq!(Line::read(&out[..out.len() - 1]), Ok(line.clone()));
        if matches!(&line, Line::Act { act: Act::Input { .. }, .. }) {
            assert_eq!(out, b"act text-field \"Name\" 1 input \"ab\"\n");
        }
        if matches!(&line, Line::Show { replace: false, .. }) {
            assert_eq!(out, b"show \"/accounts\" push\n");
        }
        if line == Line::Deadline(None) {
            assert_eq!(out, b"deadline none\n");
        }
    }
}
#[test]
fn bytes_outside_printable_ascii_are_escaped() {
    let line =
        Line::Path("a\"b\\c\0\u{1f}\u{7f}".as_bytes().iter().copied().chain([0xff]).chain(" é".bytes()).collect());
    let mut out = Vec::new();
    line.write(&mut out);
    assert_eq!(out, b"path \"a\\\"b\\\\c\\x00\\x1f\\x7f\\xff \\xc3\\xa9\"\n");
    assert_eq!(Line::read(&out[..out.len() - 1]), Ok(line));
}
#[test]
fn a_malformed_line_is_unreadable() {
    for bad in [
        "unknown",
        "start extra",
        "start ",
        "path \"open",
        "path \"\\x4\"",
        "path \"\\q\"",
        "seed -1",
        "seed 01",
        "",
        "seed",
        "seed +1",
        "seed 18446744073709551616",
        "heard unknown",
        "primary other",
        "show \"/\" other",
        "act alien \"x\" 1 press",
        "act button \"x\" 1 unknown",
        "exchange 1 1 OTHER \"/\" \"\"",
        "open 1 \"/\" \"name\"",
        "path \"\\xAF\"",
        "path \"\\x41\"",
        "path \"a\"extra",
        "start  extra",
        "seed\t1",
        "path \"é\"",
        "path \"\r\"",
    ] {
        let mut recording = b"journey 1\n".to_vec();
        recording.extend_from_slice(bad.as_bytes());
        recording.push(b'\n');
        let error = lines(&recording).expect_err(bad);
        assert_eq!(error.line, 2, "{bad}");
        assert!(!error.why.is_empty());
        assert_eq!(Line::read(bad.as_bytes()).expect_err(bad).line, 1);
    }
    assert_eq!(lines(b"journey 1\nseed 1").expect_err("no final newline").line, 2);
    assert_eq!(lines(b"").expect_err("empty recording").line, 1);
}
