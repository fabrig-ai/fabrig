//! Refusals expose positions and kinds alone (client/web/journeys.md, 3.7 and 6).
use jig_client_protocol_wire as w;
use skein_lib::{
    Time, Wall, Writer,
    base64::{self, Alphabet},
    bytes::find,
};
use skein_scratch::{Scratch, paths};
use skein_ui_web::mapping::Primary;
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use temper_client_shell_web::journey::{self, Line};

fn fixture() -> Vec<Line> {
    let facts = journey::Facts {
        seed: 11,
        now: Time::ZERO,
        wall: Wall::EPOCH,
        primary: Primary::Control,
        zone_minutes: 0,
        ways: None,
        forges: Vec::new(),
        path: b"/welcome".as_slice().into(),
        secret: Some(["fake", "-link"].concat().into_bytes().into()),
        stored: Vec::new(),
    };
    let (mut host, mut page) = facts.start();
    let mut recorder = journey::Recorder::new(&facts);
    recorder.turn(facts.now, facts.wall);
    recorder.report(&mut host, &page, temper_client_shell_web::host::Report::Start);
    host.draw(&mut page, &mut Vec::new());
    recorder.deadline(host.deadline());
    journey::lines(&recorder.save(&page)).expect("fixture lines")
}
fn written(lines: &[Line]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for line in lines {
        line.write(&mut bytes);
    }
    bytes
}
fn listing(path: &Path) -> Vec<PathBuf> {
    let mut entries: Vec<_> =
        fs::read_dir(path).expect("directory").map(|entry| entry.expect("entry").path()).collect();
    entries.sort();
    entries
}
fn invoke(name: &str, recording: &[u8], options: &[OsString], private: &[&[u8]], code: i32) -> Output {
    let scratch = Scratch::new("journeys-process-refusal");
    let file = scratch.path().join("recording");
    fs::write(&file, recording).expect("scratch recording");
    let journeys = paths::tree().join("docs/design/client/web/journeys");
    let tool = paths::tree().join("tools/journeys");
    let tui = paths::tree().join("docs/design/client/tui/journeys");
    let before = (listing(&journeys), listing(&tool), listing(&tui));
    assert!(!journeys.join(name).exists() && !tui.join(name).exists(), "unused refusal name");
    let output = Command::new(paths::binary("temper-client-journeys"))
        .arg("add")
        .args(options)
        .arg(name)
        .arg(&file)
        .output()
        .expect("run add");
    for bytes in [&output.stdout, &output.stderr] {
        for value in private {
            assert!(find(bytes, value).is_none(), "no fake bytes in either stream");
        }
    }
    assert_eq!(output.status.code(), Some(code));
    assert!(output.stdout.is_empty(), "refusal has no stdout");
    assert!(!journeys.join(name).exists() && !tui.join(name).exists(), "no refused journey");
    assert!(before == (listing(&journeys), listing(&tool), listing(&tui)), "no temporary or journey paths");
    output
}
fn messages(output: &Output, facts: &[&str]) {
    let text = String::from_utf8_lossy(&output.stderr);
    for fact in facts {
        assert!(text.contains(fact), "refusal states its position, kind and that nothing was written");
    }
}
fn insert_before_deadline(lines: &mut Vec<Line>, inserted: Vec<Line>) -> usize {
    let at = lines.iter().position(|line| matches!(line, Line::Deadline(_))).expect("deadline");
    lines.splice(at..at, inserted);
    at + 1
}
fn heard(prefix: &[u8], value: &[u8]) -> Line {
    let mut quoted = Vec::new();
    Line::Secret(value.into()).write(&mut quoted);
    journey::lines(&[prefix, quoted.strip_prefix(b"secret ").expect("quoted bytes")].concat())
        .expect("fake heard record")
        .remove(0)
}
#[test]
fn a_shape_prints_no_bytes() {
    let mut missing = 0;
    for (fake, kind) in [
        (["s", "k", "-", "Ab0123456789_-cdefGHI"].concat(), "key"),
        (["ey", "JabcDEF.Abcdef12.Zyxwvu98"].concat(), "JWT"),
        (["Bear", "er ", "Ab0123456789_+/-="].concat(), "bearer"),
        (["co", "de=", "Ab0123%+"].concat(), "code"),
        (["refresh", "_token=", "Ab0123%+"].concat(), "refresh token"),
    ] {
        let mut lines = fixture();
        let line = insert_before_deadline(&mut lines, vec![heard(b"heard answered 999 200 ", fake.as_bytes())]);
        make_stale(&mut lines);
        let output = invoke("guard-shape-refusal", &written(&lines), &[], &[fake.as_bytes(), b"fake-link"], 1);
        let text = String::from_utf8_lossy(&output.stderr);
        if ![format!("line {line}"), "heard".into(), "field".into(), kind.into(), "nothing was written".into()]
            .iter()
            .all(|fact| text.contains(fact))
        {
            eprintln!("missing refusal for {kind} in heard field");
            missing += 1;
        }
    }
    assert_eq!(missing, 0, "each shape gets its scan refusal");
}
fn data(value: &[u8], from: u32) -> Vec<u8> {
    let through = from + u32::try_from(value.len()).expect("fixture length");
    let turn =
        w::ProvisionalTurnName::new(&w::CEILINGS, w::ProvisionalTurnNameParts { attempt: 1, number: 1, retry: 0 })
            .expect("turn");
    let value = w::RunText::new(
        &w::CEILINGS,
        w::RunTextParts {
            attempt: 1,
            began: None,
            turn,
            from,
            through,
            text: value.into(),
            cut: false,
            call: None,
            writing: None,
        },
    )
    .expect("run text");
    let event = w::StreamEvent::Text(
        w::StreamText::new(&w::CEILINGS, w::StreamTextParts { watch: 1, value }).expect("stream text"),
    );
    let mut writer = Writer::new(usize::try_from(event.measure()).expect("event length"));
    event.encode(&mut writer).expect("event encoding");
    let record = writer.finish();
    let size = base64::encoded_len(Alphabet::Standard, u32::try_from(record.len()).expect("record length"))
        .expect("base64 length");
    let mut writer = Writer::new(usize::try_from(size).expect("encoded length"));
    base64::encode(Alphabet::Standard, &record, &mut writer).expect("base64");
    [b"data: ".as_slice(), &writer.finish(), b"\n\n"].concat()
}
#[test]
fn a_split_token_prints_no_bytes() {
    let fake = ["s", "k", "-", "Ab0123456789_-cdefGHI"].concat().into_bytes();
    let cut = fake.len() / 2;
    let stream = [data(&fake[..cut], 0), data(&fake[cut..], u32::try_from(cut).expect("offset"))].concat();
    let middle = stream.len() / 2;
    let mut lines = fixture();
    let line = insert_before_deadline(
        &mut lines,
        vec![heard(b"heard piece ", &stream[..middle]), heard(b"heard piece ", &stream[middle..])],
    );
    make_stale(&mut lines);
    let output = invoke("guard-split-refusal", &written(&lines), &[], &[&fake, b"fake-link"], 1);
    messages(&output, &[&format!("line {line}"), "heard", "run", "key", "nothing was written"]);
}
#[test]
fn a_failed_proof_prints_no_bytes() {
    let fake = ["private", "-stale-exchange"].concat().into_bytes();
    let given = ["private", "-given-word"].concat();
    let mut lines = fixture();
    let mut count = 0;
    let mut position = 0;
    for (at, line) in lines.iter_mut().enumerate() {
        if let Line::Exchange { body, .. } = line {
            *body = fake.clone().into();
            count += 1;
            position = at + 1;
        }
    }
    assert_eq!(count, 1, "edited exchange");
    let output = invoke(
        "guard-proof-refusal",
        &written(&lines),
        &["--redact".into(), given.clone().into()],
        &[&fake, given.as_bytes(), b"fake-link"],
        1,
    );
    messages(
        &output,
        &["turn 1", &format!("line {position}"), "recorded exchange", "asked exchange", "nothing was written"],
    );
}
#[test]
fn an_undecodable_line_prints_no_bytes() {
    let fake = ["private", "-not-base64!"].concat().into_bytes();
    let mut lines = fixture();
    let piece = [b"data: ".as_slice(), &fake, b"\n"].concat();
    let line = insert_before_deadline(&mut lines, vec![heard(b"heard piece ", &piece)]);
    make_stale(&mut lines);
    let output = invoke("guard-undecoded-refusal", &written(&lines), &[], &[&fake, b"fake-link"], 1);
    messages(&output, &[&format!("line {line}"), "undecodable", "nothing was written"]);
}
#[test]
fn a_differing_mark_prints_no_bytes() {
    let fake = ["private", "-mark-marker"].concat().into_bytes();
    let mut lines = fixture();
    let mut count = 0;
    for line in &mut lines {
        if let Line::End(html) = line {
            *html = Some(fake.clone().into());
            count += 1;
        }
    }
    assert_eq!(count, 1, "edited end mark");
    let output = invoke("guard-mark-refusal", &written(&lines), &[], &[&fake, b"fake-link"], 1);
    messages(&output, &["mark 1", "byte", "nothing was written"]);
}
#[test]
fn usage_echoes_nothing_given() {
    let fake = ["private", "-usage-text"].concat();
    for options in [vec!["--redact".into()], vec!["--bogus".into(), fake.clone().into()]] {
        let output = invoke("guard-usage-refusal", b"", &options, &[fake.as_bytes()], 2);
        messages(&output, &["usage:"]);
    }
    let output =
        Command::new(paths::binary("temper-client-journeys")).args(["add", "--redact", &fake]).output().expect("usage");
    for bytes in [&output.stdout, &output.stderr] {
        assert!(find(bytes, fake.as_bytes()).is_none(), "usage echoes no given text");
    }
    assert_eq!(output.status.code(), Some(2));
}

fn make_stale(lines: &mut [Line]) {
    let mut count = 0;
    for line in lines {
        if let Line::Deadline(deadline) = line {
            *deadline = Some(Time::ZERO);
            count += 1;
        }
    }
    assert_eq!(count, 1, "edited deadline makes the process fixture refuse even without a scan");
}

fn tui_fixture(frame: bool) -> Vec<u8> {
    use skein_ui_screen::Size;
    use temper_client_shell_tui::{host::Report, journey as tj};
    let address = if frame { "pri\u{200b}vate-frame-marker&".as_bytes() } else { b"127.0.0.1:8367" };
    let facts = tj::Facts {
        seed: 7,
        now: Time::ZERO,
        wall: Wall::EPOCH,
        size: Size { rows: 16, columns: 100 },
        scheme: Some(temper_client_tui::Scheme::Dark),
        background: None,
        boxed: false,
        address: address.into(),
        local: false,
        front: false,
        note: None,
        kept: Vec::new(),
        credential: None,
    };
    let (mut host, start) = facts.start();
    let mut recorder = tj::Recorder::new(&facts);
    recorder.turn(facts.now, facts.wall);
    let duties = recorder.report(&mut host, start);
    recorder.deadline(host.deadline());
    if frame {
        let exchange =
            duties
                .asked
                .iter()
                .find_map(|request| {
                    if let jig_client::Request::Exchange { exchange, .. } = request { Some(*exchange) } else { None }
                })
                .expect("approval ask");
        let mut reply = vec![0];
        reply.extend_from_slice(&32_u32.to_be_bytes());
        reply.extend_from_slice(&[7; 32]);
        reply.extend_from_slice(b"WDJBMJHT");
        reply.extend_from_slice(&300_000_000_000_u64.to_be_bytes());
        for (index, report) in [
            Report::Heard(jig_client::Event::Reply { exchange, message: reply.into() }),
            Report::Opened { opened: true },
            Report::Resize(Size { rows: 0, columns: 0 }),
        ]
        .into_iter()
        .enumerate()
        {
            let now = u64::try_from(index + 1).expect("turn time");
            let now_time = Time::from_nanos(now);
            let wall = Wall::from_nanos(now);
            host.clocks(now_time, wall);
            recorder.turn(now_time, wall);
            recorder.report(&mut host, report);
            host.frame();
            recorder.deadline(host.deadline());
        }
    }
    let mut lines = tj::lines(&recorder.save(&mut host)).expect("TUI recording");
    if !frame {
        let mut changed = 0;
        for line in &mut lines {
            if let tj::Line::End(print) = line {
                *print = Some(b"private-proof-marker".as_slice().into());
                changed += 1;
            }
        }
        assert_eq!(changed, 1, "one end proof changed");
    }
    let mut bytes = Vec::new();
    for line in lines {
        line.write(&mut bytes);
    }
    bytes
}

#[test]
fn a_tui_refusal_prints_positions_and_no_original_bytes() {
    for (frame, options, private, expected) in [
        (
            true,
            vec!["--redact".into(), "private-frame-marker".into()],
            b"private-frame-marker".as_slice(),
            vec!["deadline", "frame at turn", "found an original"],
        ),
        (false, vec![], b"private-proof-marker".as_slice(), vec!["mark 1", "frame line"]),
    ] {
        let output = invoke("guard-tui-refusal", &tui_fixture(frame), &options, &[private], 1);
        messages(&output, &expected);
        messages(&output, &["line", "nothing was written"]);
    }
}
