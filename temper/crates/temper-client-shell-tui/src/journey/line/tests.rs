use super::{Line, Sent, Streamed, Unreadable, lines};
use jig_client::Message;
use skein_lib::{Time, Token, Wall};
use skein_ui_screen::{Rgb, Size};
use skein_ui_screen_terminal::Key;
use temper_client_tui::{Note, Scheme};

fn bytes(text: &[u8]) -> Box<[u8]> {
    text.into()
}

fn key(key: Key) -> Line {
    Line::Key { key, event: None }
}

fn send(message: &Message) -> Line {
    Line::Send { exchange: Token::new(u64::MAX).raw(), message: Sent::of(message), credential: None }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table holds the exact text of every record, key and message kind")]
fn records_have_their_exact_text_and_round_trip() {
    let rows: Vec<(Line, &[u8])> = vec![
        (Line::Journey { version: 2 }, b"journey 2\n"),
        (Line::Seed(u64::MAX), b"seed 18446744073709551615\n"),
        (Line::Clocks { now: Time::ZERO, wall: Wall::EPOCH }, b"clocks 0 0\n"),
        (Line::Screen { size: Size { rows: 24, columns: 80 }, scheme: Some(Scheme::Dark) }, b"screen 24 80 dark\n"),
        (Line::Screen { size: Size { rows: 24, columns: 80 }, scheme: None }, b"screen 24 80 none\n"),
        (
            Line::Screen { size: Size { rows: 0, columns: u16::MAX }, scheme: Some(Scheme::Light) },
            b"screen 0 65535 light\n",
        ),
        (Line::Address(bytes(b"localhost:8367")), b"address \"localhost:8367\"\n"),
        (Line::Background(Rgb { red: 0, green: 43, blue: 255 }), b"background 0 43 255\n"),
        (Line::Local, b"local\n"),
        (Line::Front, b"front\n"),
        (Line::BoxLine(bytes(b"log recent")), b"box-line \"log recent\"\n"),
        (Line::BoxStage(bytes(b"fetching 7 -")), b"box-stage \"fetching 7 -\"\n"),
        (Line::StopBox, b"stop-box\n"),
        (Line::StopBoxAgain, b"stop-box-again\n"),
        (Line::Box, b"box\n"),
        (Line::Noted(Note::Until(Wall::EPOCH)), b"noted until 0\n"),
        (Line::Noted(Note::Until(Wall::from_nanos(u64::MAX))), b"noted until 18446744073709551615\n"),
        (Line::Noted(Note::SignedOut), b"noted signed-out\n"),
        (Line::Found(None), b"found none\n"),
        (Line::Found(Some(bytes(b"http://127.0.0.1:8367/link"))), b"found \"http://127.0.0.1:8367/link\"\n"),
        (Line::Note(Note::Until(Wall::from_nanos(300_000_000_000))), b"note until 300000000000\n"),
        (Line::Note(Note::SignedOut), b"note signed-out\n"),
        (Line::Read, b"read\n"),
        (Line::Stored { key: bytes(b"key"), bytes: bytes(b"value") }, b"stored \"key\" \"value\"\n"),
        (Line::Credential(bytes(b"")), b"credential \"\"\n"),
        (
            Line::Turn { now: Time::from_nanos(u64::MAX), wall: Wall::from_nanos(u64::MAX) },
            b"turn 18446744073709551615 18446744073709551615\n",
        ),
        (Line::Start, b"start\n"),
        (Line::Due, b"due\n"),
        (Line::Key { key: Key::Char('é'), event: Some(bytes(b"Typed")) }, b"key char \"\\xC3\\xA9\" \"Typed\"\n"),
        (key(Key::Char(' ')), b"key char \" \" none\n"),
        (key(Key::Alt('🦀')), b"key alt \"\\xF0\\x9F\\xA6\\x80\" none\n"),
        (key(Key::Control(b'a')), b"key control \"a\" none\n"),
        (key(Key::Control(b'Z')), b"key control \"Z\" none\n"),
        (key(Key::Paste(bytes(b"\\\"\x00\x7F\xFF ~"))), b"key paste \"\\\\\\\"\\x00\\x7F\\xFF ~\" none\n"),
        (key(Key::Enter), b"key enter none\n"),
        (key(Key::Tab), b"key tab none\n"),
        (key(Key::BackTab), b"key backtab none\n"),
        (key(Key::Backspace), b"key backspace none\n"),
        (key(Key::Delete), b"key delete none\n"),
        (key(Key::Up), b"key up none\n"),
        (key(Key::Down), b"key down none\n"),
        (key(Key::Left), b"key left none\n"),
        (key(Key::Right), b"key right none\n"),
        (key(Key::Home), b"key home none\n"),
        (key(Key::End), b"key end none\n"),
        (key(Key::PageUp), b"key pageup none\n"),
        (key(Key::PageDown), b"key pagedown none\n"),
        (key(Key::Escape), b"key escape none\n"),
        (Line::Resize(Size { rows: u16::MAX, columns: 0 }), b"resize 65535 0\n"),
        (
            Line::Reply { exchange: Token::new(u64::MAX).raw(), bytes: bytes(b"answer") },
            b"reply 18446744073709551615 \"answer\"\n",
        ),
        (Line::Unanswered(Token::new(u64::MAX).raw()), b"unanswered 18446744073709551615\n"),
        (Line::Streamed(Streamed::Record(bytes(b"record"))), b"streamed record \"record\"\n"),
        (Line::Streamed(Streamed::Heartbeat), b"streamed heartbeat\n"),
        (Line::Streamed(Streamed::Full { transport: false, size: u32::MAX }), b"streamed full component 4294967295\n"),
        (Line::Streamed(Streamed::Full { transport: true, size: 0 }), b"streamed full transport 0\n"),
        (Line::Streamed(Streamed::SignedOut), b"streamed signed-out\n"),
        (Line::Gone, b"gone\n"),
        (Line::Saved, b"saved\n"),
        (Line::Unsaved, b"unsaved\n"),
        (Line::Kept { written: true }, b"kept done\n"),
        (Line::Kept { written: false }, b"kept failed\n"),
        (Line::Opened { opened: true }, b"opened done\n"),
        (Line::Opened { opened: false }, b"opened failed\n"),
        (
            send(&Message::Authorize(bytes(b"authorize-body"))),
            b"send 18446744073709551615 authorize \"authorize-body\" none\n",
        ),
        (
            send(&Message::ApprovalAsk(bytes(b"ask-body"))),
            b"send 18446744073709551615 approval-ask \"ask-body\" none\n",
        ),
        (
            send(&Message::ApprovalRead(bytes(b"read-body"))),
            b"send 18446744073709551615 approval-read \"read-body\" none\n",
        ),
        (
            send(&Message::ApprovalDecide(bytes(b"decide-body"))),
            b"send 18446744073709551615 approval-decide \"decide-body\" none\n",
        ),
        (
            send(&Message::SignInApproval(bytes(b"approval-body"))),
            b"send 18446744073709551615 sign-in-approval \"approval-body\" none\n",
        ),
        (send(&Message::Request(bytes(b"request-body"))), b"send 18446744073709551615 request \"request-body\" none\n"),
        (send(&Message::Read(bytes(b"read-body"))), b"send 18446744073709551615 read \"read-body\" none\n"),
        (send(&Message::Watch(bytes(b"watch-body"))), b"send 18446744073709551615 watch \"watch-body\" none\n"),
        (send(&Message::Unwatch(bytes(b"unwatch-body"))), b"send 18446744073709551615 unwatch \"unwatch-body\" none\n"),
        (
            send(&Message::StreamGone(bytes(b"gone-body"))),
            b"send 18446744073709551615 stream-gone \"gone-body\" none\n",
        ),
        (
            send(&Message::SignInLink(bytes(b"link-body"))),
            b"send 18446744073709551615 sign-in-link \"link-body\" none\n",
        ),
        (send(&Message::SignInService), b"send 18446744073709551615 sign-in-service \"\" none\n"),
        (send(&Message::SignOut), b"send 18446744073709551615 sign-out \"\" none\n"),
        (
            Line::Send { exchange: 0, message: Sent::Read(bytes(b"")), credential: Some(bytes(b"fake")) },
            b"send 0 read \"\" \"fake\"\n",
        ),
        (Line::Stream(Some(bytes(b"fake"))), b"stream \"fake\"\n"),
        (Line::Stream(None), b"stream none\n"),
        (Line::Unstream, b"unstream\n"),
        (Line::Save { key: bytes(b"key"), bytes: bytes(b"value") }, b"save \"key\" \"value\"\n"),
        (Line::Erase(bytes(b"key")), b"erase \"key\"\n"),
        (Line::Keep(Some(bytes(b""))), b"keep \"\"\n"),
        (Line::Keep(None), b"keep none\n"),
        (Line::Signed(bytes(b"localhost:9000")), b"signed \"localhost:9000\"\n"),
        (Line::Browse(bytes(b"http://localhost:8367/approve")), b"browse \"http://localhost:8367/approve\"\n"),
        (Line::Quit, b"quit\n"),
        (Line::Deadline(Some(Time::from_nanos(u64::MAX))), b"deadline 18446744073709551615\n"),
        (Line::Deadline(None), b"deadline none\n"),
        (Line::Mark(Some(bytes(b"frame\n"))), b"mark \"frame\\x0A\"\n"),
        (Line::Mark(Some(bytes(b""))), b"mark \"\"\n"),
        (Line::Mark(None), b"mark\n"),
        (Line::End(Some(bytes(b"frame"))), b"end \"frame\"\n"),
        (Line::End(Some(bytes(b""))), b"end \"\"\n"),
        (Line::End(None), b"end\n"),
    ];
    let mut recording = Vec::new();
    let mut expected = Vec::new();
    for (index, (line, text)) in rows.into_iter().enumerate() {
        let mut out = b"prefix\n".to_vec();
        line.write(&mut out);
        assert!(out.strip_prefix(b"prefix\n") == Some(text), "row {index}: {} text", line.kind());
        assert!(text.starts_with(line.kind().as_bytes()), "row {index}: kind");
        let decoded = lines(text).expect("table row is readable");
        assert!(decoded == vec![line.clone()], "row {index}: {} round trip", line.kind());
        recording.extend_from_slice(text);
        expected.push(line);
    }
    assert!(lines(&recording).expect("all rows are readable") == expected);
}

#[test]
fn unreadable_rows_name_their_line() {
    let rows: &[(&str, &[u8], &str)] = &[
        ("another version", b"journey 3\n", "unsupported journey version"),
        ("version overflow", b"journey 4294967296\n", "number overflow"),
        ("unknown kind", b"unknown\n", "unknown record kind"),
        ("unknown scheme", b"screen 24 80 other\n", "unknown scheme"),
        ("background red overflow", b"background 256 0 0\n", "number overflow"),
        ("background green overflow", b"background 0 256 0\n", "number overflow"),
        ("background blue overflow", b"background 0 0 256\n", "number overflow"),
        ("background missing red", b"background\n", "expected field"),
        ("background missing green", b"background 0\n", "expected field"),
        ("background missing blue", b"background 0 0\n", "expected field"),
        ("background extra", b"background 0 0 0 0\n", "extra field"),
        ("box extra", b"box extra\n", "extra field"),
        ("unknown key", b"key other none\n", "unknown key"),
        ("unknown streamed", b"streamed other\n", "unknown streamed kind"),
        ("unknown full", b"streamed full other 0\n", "unknown full kind"),
        ("unknown outcome", b"kept other\n", "unknown outcome"),
        ("unknown opened", b"opened other\n", "unknown outcome"),
        ("unknown message", b"send 0 other \"\" none\n", "unknown message kind"),
        ("unknown optional", b"stream other\n", "expected bytes or none"),
        ("unknown start note", b"noted other\n", "unknown note"),
        ("unknown note", b"note other\n", "unknown note"),
        ("note missing time", b"noted until\n", "expected field"),
        ("note overflow", b"note until 18446744073709551616\n", "number overflow"),
        ("note leading zero", b"noted until 01\n", "invalid number"),
        ("signed out extra", b"note signed-out 0\n", "extra field"),
        ("found missing", b"found\n", "expected field"),
        ("found invalid", b"found other\n", "expected bytes or none"),
        ("front fields", b"front extra\n", "extra field"),
        ("box-line missing", b"box-line\n", "expected field"),
        ("local fields", b"local extra\n", "extra field"),
        ("read fields", b"read extra\n", "extra field"),
        ("missing field", b"seed\n", "expected field"),
        ("extra field", b"start extra\n", "extra field"),
        ("seed overflow", b"seed 18446744073709551616\n", "number overflow"),
        ("exchange overflow", b"unanswered 18446744073709551616\n", "number overflow"),
        ("time overflow", b"turn 18446744073709551616 0\n", "number overflow"),
        ("wall overflow", b"clocks 0 18446744073709551616\n", "number overflow"),
        ("rows overflow", b"resize 65536 0\n", "number overflow"),
        ("columns overflow", b"screen 0 65536 dark\n", "number overflow"),
        ("full size overflow", b"streamed full component 4294967296\n", "number overflow"),
        ("unterminated quote", b"address \"open\n", "unclosed quote"),
        ("bad escape", b"address \"\\q\"\n", "bad escape"),
        ("short escape", b"address \"\\x0\n", "incomplete escape"),
        ("invalid hex", b"address \"\\xGG\"\n", "bad hex digit"),
        ("lower case hex", b"address \"\\x7f\"\n", "bad hex digit"),
        ("escaped printable", b"address \"\\x41\"\n", "escaped printable byte"),
        ("unescaped byte", b"address \"\x7F\"\n", "unescaped byte"),
        ("unescaped UTF-8", "address \"é\"\n".as_bytes(), "unescaped byte"),
        ("blank line", b"\n", "unknown record kind"),
        ("two characters", b"key char \"ab\" none\n", "expected one character"),
        ("empty character", b"key char \"\" none\n", "expected one character"),
        ("invalid UTF-8", b"key char \"\\xFF\" none\n", "invalid character"),
        ("two alt characters", b"key alt \"ab\" none\n", "expected one character"),
        ("control not letter", b"key control \"0\" none\n", "expected ASCII letter"),
        ("two control letters", b"key control \"ab\" none\n", "expected ASCII letter"),
        ("control not ASCII", b"key control \"\\xC3\\xA9\" none\n", "expected ASCII letter"),
        ("body on sign-out", b"send 0 sign-out \"body\" none\n", "unexpected message body"),
        ("body on service", b"send 0 sign-in-service \"body\" none\n", "unexpected message body"),
        ("leading space", b" start\n", "unknown record kind"),
        ("doubled space", b"seed  1\n", "invalid word"),
        ("trailing space", b"seed 1 \n", "extra field"),
        ("tab separator", b"seed\t1\n", "unknown record kind"),
        ("field not quoted", b"address bare\n", "expected quoted bytes"),
        ("quote not separated", b"stored \"key\"\"value\"\n", "expected field"),
        ("quoted number", b"seed \"1\"\n", "invalid word"),
        ("negative number", b"seed -1\n", "invalid number"),
        ("leading zero", b"seed 01\n", "invalid number"),
        ("leading plus", b"seed +1\n", "invalid word"),
        ("mark none word", b"mark none\n", "expected quoted bytes"),
        ("missing newline", b"seed 1", "missing final newline"),
    ];
    for (name, bad, why) in rows {
        let mut recording = b"journey 2\nseed 7\n".to_vec();
        recording.extend_from_slice(bad);
        let Err(error) = lines(&recording) else {
            panic!("{name}: should be unreadable");
        };
        assert_eq!(error, Unreadable { line: 3, why }, "{name}");
    }
    assert!(lines(b"") == Err(Unreadable { line: 1, why: "empty recording" }));
    assert!(lines(b"journey 3\n") == Err(Unreadable { line: 1, why: "unsupported journey version" }));
}
