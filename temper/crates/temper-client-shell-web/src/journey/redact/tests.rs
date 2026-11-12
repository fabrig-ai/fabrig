use super::*;
use crate::journey::{Kind, Line};
use jig_client_protocol_wire as w;
use jig_client_shell_web::adapter::Heard;
use skein_lib::{
    Reader, Writer,
    base64::{self, Alphabet},
    bytes::find,
    sha256::digest,
};
use skein_ui::Act;

// Sixteen fake bytes, with URL, html and Debug escapes all differing from the bytes.
const VALUE: &[u8] = b"ab/+=\"<&\\sample!";
const HIDDEN: &[u8] = b"xxxxxxxxxxxxxxxx";
const UPPER: &[u8] = b"ab%2F%2B%3D%22%3C%26%5Csample%21";
const LOWER: &[u8] = b"ab%2f%2b%3d%22%3c%26%5csample%21";
const HTML: &[u8] = b"ab/+=&quot;&lt;&amp;\\sample!";
const DEBUG: &[u8] = b"ab/+=\\\"<&\\\\sample!";

fn written(lines: &[Line]) -> Box<[u8]> {
    let mut out = Vec::new();
    for line in lines {
        line.write(&mut out);
    }
    out.into()
}
fn changed(lines: &[Line], given: &[Box<[u8]>]) -> (Redacted, Vec<Line>) {
    let result = redact(&written(lines), given).expect("readable recording redacts");
    let lines = lines_of(&result);
    (result, lines)
}
fn lines_of(result: &Redacted) -> Vec<Line> {
    lines(&result.recording).expect("redacted lines read")
}
fn input(value: &[u8]) -> Line {
    Line::Act {
        act: Act::Input { value: value.into() },
        event: b"AccountKey { value: Secret { len: 16 } }".as_slice().into(),
        name: b"Key".as_slice().into(),
    }
}
fn kept(record: &[u8]) -> Box<[u8]> {
    let value = w::Kept::new(&w::CEILINGS, w::KeptParts { store_version: 1, record: record.into() })
        .expect("bounded fake record");
    let mut writer = Writer::new(skein_lib::length::usize_of(value.measure()));
    value.encode(&mut writer).expect("measured fake record");
    writer.finish()
}
fn exchange(body: &[u8]) -> Line {
    Line::Exchange {
        id: 1,
        bound: 256,
        method: b"POST".as_slice().into(),
        path: b"/".as_slice().into(),
        body: body.into(),
        headers: Vec::new(),
    }
}
fn framed(value: &[u8]) -> Box<[u8]> {
    // A counted field between two neighbours, repeated to check every occurrence.
    [
        b"before".as_slice(),
        &u32::try_from(value.len()).expect("fixture length").to_le_bytes(),
        value,
        b"between",
        value,
        b"after",
    ]
    .concat()
    .into()
}
fn event(record: &[u8]) -> Box<[u8]> {
    let len = base64::encoded_len(Alphabet::Standard, u32::try_from(record.len()).expect("fixture length"))
        .expect("fixture encoding");
    let mut writer = Writer::new(usize::try_from(len).expect("fixture length"));
    base64::encode(Alphabet::Standard, record, &mut writer).expect("measured encoding");
    [b"data: ".as_slice(), &writer.finish(), b"\n\n"].concat().into()
}
fn piece(bytes: &[u8]) -> Line {
    Line::Heard(Heard::Piece { bytes: bytes.into() })
}
fn split(bytes: &[u8], cut: usize) -> Vec<Line> {
    vec![piece(&bytes[..cut]), Line::Due, piece(&bytes[cut..])]
}
fn query(value: &[u8]) -> Box<[u8]> {
    [b"/?value=".as_slice(), value, b"&tail=yes"].concat().into()
}
fn html(value: &[u8]) -> Box<[u8]> {
    [b"<p>".as_slice(), value, b"</p>"].concat().into()
}

struct Case {
    label: String,
    before: Vec<Line>,
    after: Vec<Line>,
}
fn case(label: &str, before: Line, after: Line) -> Case {
    Case { label: label.into(), before: vec![before], after: vec![after] }
}
type TextField = (&'static str, fn(Box<[u8]>) -> Line);

#[expect(
    clippy::too_many_lines,
    reason = "the table names each field, form and piece boundary of client/web/journeys.md, 3.6"
)]
fn places() -> Vec<Case> {
    let mut cases = vec![
        case("secret bytes", Line::Secret(VALUE.into()), Line::Secret(HIDDEN.into())),
        case(
            "stored value",
            Line::Stored { key: b"other".as_slice().into(), value: VALUE.into() },
            Line::Stored { key: b"other".as_slice().into(), value: HIDDEN.into() },
        ),
        case("framed percent", exchange(&framed(UPPER)), exchange(&framed(&vec![b'x'; UPPER.len()]))),
        case("exchange body", exchange(&framed(VALUE)), exchange(&framed(HIDDEN))),
        case(
            "answered body",
            Line::Heard(Heard::Answered { exchange: 1, status: 200, body: framed(VALUE), headers: Vec::new() }),
            Line::Heard(Heard::Answered { exchange: 1, status: 200, body: framed(HIDDEN), headers: Vec::new() }),
        ),
        case(
            "act event",
            Line::Act { act: Act::Press, event: VALUE.into(), name: Box::default() },
            Line::Act { act: Act::Press, event: HIDDEN.into(), name: Box::default() },
        ),
        case(
            "act Debug event",
            Line::Act { act: Act::Press, event: DEBUG.into(), name: Box::default() },
            Line::Act { act: Act::Press, event: HIDDEN.into(), name: Box::default() },
        ),
        case(
            "act name",
            Line::Act { act: Act::Press, event: Box::default(), name: VALUE.into() },
            Line::Act { act: Act::Press, event: Box::default(), name: HIDDEN.into() },
        ),
        case(
            "plain act value",
            Line::Act { act: Act::Input { value: VALUE.into() }, event: Box::default(), name: Box::default() },
            Line::Act { act: Act::Input { value: HIDDEN.into() }, event: Box::default(), name: Box::default() },
        ),
        case(
            "drawing",
            Line::Act { act: Act::Drawing { data: VALUE.into() }, event: Box::default(), name: Box::default() },
            Line::Act { act: Act::Drawing { data: HIDDEN.into() }, event: Box::default(), name: Box::default() },
        ),
        case("piece record", piece(&event(&framed(VALUE))), piece(&event(&framed(HIDDEN)))),
        case(
            "stream comment",
            piece(&[b": ".as_slice(), VALUE, b"\n"].concat()),
            piece(&[b": ".as_slice(), HIDDEN, b"\n"].concat()),
        ),
        case("stream refused body", piece(VALUE), piece(HIDDEN)),
        case(
            "cut event record",
            piece(event(&framed(VALUE)).strip_suffix(b"\n\n").expect("event terminator")),
            piece(event(&framed(HIDDEN)).strip_suffix(b"\n\n").expect("event terminator")),
        ),
    ];
    // Each text field is an independent row, including fields outside the minimum sketch.
    let text_fields: &[TextField] = &[
        ("path", Line::Path),
        ("place", Line::Place),
        ("navigate", Line::Navigate),
        ("replace", Line::Replace),
        ("show", |path| Line::Show { path, replace: false }),
        ("ways content", |content| Line::Ways { content, origin: Box::default() }),
        ("ways origin", |origin| Line::Ways { content: Box::default(), origin }),
        ("forge", Line::Forge),
        ("exchange path", |path| request(path, Vec::new())),
        ("open path", |path| Line::Open { piece: 64, path, headers: Vec::new() }),
    ];
    for (label, make) in text_fields {
        for (form_name, form) in [("bytes", VALUE), ("percent upper", UPPER), ("percent lower", LOWER)] {
            cases.push(case(&format!("{label} {form_name}"), make(query(form)), make(query(HIDDEN))));
        }
    }
    for (label, make) in [("mark", Line::Mark as fn(_) -> _), ("end", Line::End as fn(_) -> _)] {
        for (form_name, form) in [("bytes", VALUE), ("html", HTML)] {
            cases.push(case(&format!("{label} {form_name}"), make(Some(html(form))), make(Some(html(HIDDEN)))));
        }
    }
    for (label, make) in [
        ("stored key", (|key| Line::Stored { key, value: Box::default() }) as fn(Box<[u8]>) -> Line),
        ("save key", |key| Line::Save { key, value: Box::default() }),
        ("save value", |value| Line::Save { key: b"other".as_slice().into(), value }),
        ("written", Line::Written),
        ("unwritten", Line::Unwritten),
        ("erase", Line::Erase),
    ] {
        cases.push(case(label, make(VALUE.into()), make(HIDDEN.into())));
    }
    for open in [false, true] {
        for name in [false, true] {
            let make = |bytes: &[u8]| {
                let header =
                    if name { (bytes.into(), b"v".as_slice().into()) } else { (b"h".as_slice().into(), bytes.into()) };
                if open {
                    Line::Open { piece: 64, path: b"/".as_slice().into(), headers: vec![header] }
                } else {
                    request(b"/".as_slice().into(), vec![header])
                }
            };
            cases.push(case(&format!("header open={open} name={name}"), make(VALUE), make(HIDDEN)));
        }
    }
    let before = event(&framed(VALUE));
    let after = event(&framed(HIDDEN));
    for cut in 0..=before.len() {
        cases.push(Case {
            label: format!("piece split {cut}"),
            before: split(&before, cut),
            after: split(&after, cut),
        });
    }
    cases
}

fn request(path: Box<[u8]>, headers: Vec<skein_ui_web::transport::Header>) -> Line {
    Line::Exchange { id: 1, bound: 256, method: b"POST".as_slice().into(), path, body: Box::default(), headers }
}

#[test]
fn each_kind_goes_from_each_place() {
    let cases = places();
    let sources = [
        ("link", vec![Line::Secret(VALUE.into())], Vec::new()),
        ("stored", vec![Line::Stored { key: jig_client::STREAM_KEY.into(), value: kept(VALUE) }], Vec::new()),
        ("save", vec![Line::Save { key: jig_client::STREAM_KEY.into(), value: kept(VALUE) }], Vec::new()),
        ("typed", vec![input(VALUE)], Vec::new()),
        ("given", Vec::new(), vec![VALUE.into()]),
    ];
    let mut failures = 0;
    let mut rows = 0;
    for (kind, source, given) in &sources {
        for case in &cases {
            rows += 1;
            let mut recording = source.clone();
            recording.extend(case.before.clone());
            let (_, output) = changed(&recording, given);
            if written(&output[source.len()..]) != written(&case.after) {
                eprintln!("failed row: {kind}, {}", case.label);
                failures += 1;
            }
        }
    }
    eprintln!("kind/place table: {rows} rows, {failures} failures");
    assert_eq!(failures, 0, "each kind must go from each place");
}

#[test]
fn a_framed_field_keeps_its_length_and_its_neighbours() {
    let source = [Line::Secret(VALUE.into()), exchange(&framed(VALUE))];
    let (_, output) = changed(&source, &[]);
    assert!(written(&output[1..]) == written(&[exchange(&framed(HIDDEN))]), "length prefixes and neighbours stand");
}

#[test]
fn a_stored_value_keeps_its_kept_head() {
    for source in [
        Line::Stored { key: jig_client::STREAM_KEY.into(), value: kept(VALUE) },
        Line::Save { key: jig_client::STREAM_KEY.into(), value: kept(VALUE) },
    ] {
        let (_, output) = changed(&[source], &[]);
        let (Line::Stored { value, .. } | Line::Save { value, .. }) = &output[0] else { panic!("stored or save") };
        let mut reader = Reader::new(value);
        let value = w::Kept::decode(&w::CEILINGS, &mut reader).expect("kept head still decodes");
        assert!(value.record() == HIDDEN, "sixteen x bytes");
        assert!(reader.is_empty(), "no trailing bytes");
    }
}

#[test]
fn a_percent_form_in_a_text_field_becomes_the_placeholder() {
    for form in [UPPER, LOWER] {
        let (_, output) = changed(&[Line::Secret(VALUE.into()), Line::Path(query(form))], &[]);
        assert!(written(&output[1..]) == written(&[Line::Path(query(HIDDEN))]), "text uses original length");
    }
}

#[test]
fn a_percent_form_in_a_framed_field_keeps_its_length() {
    for form in [UPPER, LOWER] {
        let (_, output) = changed(&[Line::Secret(VALUE.into()), exchange(&framed(form))], &[]);
        let expected = vec![b'x'; form.len()];
        assert!(written(&output[1..]) == written(&[exchange(&framed(&expected))]), "framed uses form length");
    }
}

#[test]
fn a_digest_becomes_the_placeholders() {
    let body = [digest(b"abcd").as_slice(), b"middle", &digest(b"abcd\n"), &digest(b"abcd")].concat();
    let expected = [digest(b"xxxx").as_slice(), b"middle", &digest(b"xxxx\n"), &digest(b"xxxx")].concat();
    let (_, output) = changed(&[input(b"abcd\n"), exchange(&body)], &[]);
    assert!(written(&output[1..]) == written(&[exchange(&expected)]), "both digests replaced, every occurrence");
}

#[test]
fn a_typing_runs_heads_stay_in_their_acts() {
    let value = [b"s".as_slice(), b"k", b"-", b"sample"].concat();
    let mut source: Vec<_> = (1..=value.len()).map(|end| input(&value[..end])).collect();
    source.push(exchange(&[b"s beside ".as_slice(), &value].concat()));
    let (_, output) = changed(&source, &[]);
    for (index, line) in output[..value.len()].iter().enumerate() {
        let Line::Act { act: Act::Input { value }, .. } = line else { panic!("input act") };
        assert!(value.len() == index + 1 && value.iter().all(|byte| *byte == b'x'), "each head in its own act");
    }
    let expected = [b"s beside ".as_slice(), &vec![b'x'; value.len()]].concat();
    assert!(written(&output[value.len()..]) == written(&[exchange(&expected)]), "short head elsewhere stands");
}

#[test]
fn a_pasted_values_whitespace_stands() {
    let (_, output) = changed(&[input(b"abcd\n"), exchange(b"abcd")], &[]);
    assert!(written(&output) == written(&[input(b"xxxx\n"), exchange(b"xxxx")]), "paste newline stands");
}

#[test]
fn longest_first() {
    let (_, output) =
        changed(&[exchange(b"abcdef abcd abcdef")], &[b"abcd".as_slice().into(), b"abcdef".as_slice().into()]);
    assert!(written(&output) == written(&[exchange(b"xxxxxx xxxx xxxxxx")]), "longest original replaced first");
}

#[test]
fn counts_each_kind_and_its_places() {
    // Four source occurrences, six body occurrences, one event and one plain run.
    const PLACES: u32 = 12;
    let source = [
        Line::Secret(b"link-value".as_slice().into()),
        Line::Stored { key: jig_client::STREAM_KEY.into(), value: kept(b"stored-value") },
        Line::Save { key: jig_client::STREAM_KEY.into(), value: kept(b"stored-value") },
        input(b"typed-value"),
        exchange(b"link-value link-value stored-value typed-value given-value given-value"),
        piece(&event(b"given-value")),
        piece(b": given-value\n"),
    ];
    let (result, _) = changed(&source, &[b"given-value".as_slice().into()]);
    for kind in [Kind::Link, Kind::Stored, Kind::Typed, Kind::Given] {
        assert_eq!(result.originals.count(kind), 1, "one distinct original per kind");
    }
    assert_eq!(result.places, PLACES);
    let (again, _) = changed(&lines_of(&result), &[]);
    assert_eq!(again.places, 0, "already redacted values are not replacements");
}

#[test]
fn an_undecodable_data_line_refuses() {
    let result = redact(&written(&[Line::Due, piece(b"data: %%%%\n")]), &[]);
    assert!(matches!(result, Err(Unredacted::Undecoded(Undecoded { line: 2 }))), "undecodable first piece refused");
}

#[test]
fn the_committed_recordings_redact() {
    let journeys = skein_scratch::paths::tree().join("docs/design/client/web/journeys");
    let mut count = 0;
    for entry in std::fs::read_dir(journeys).expect("journeys") {
        let path = entry.expect("journey").path().join("recording");
        if !path.is_file() {
            continue;
        }
        count += 1;
        let recording = std::fs::read(path).expect("recording");
        let output = redact(&recording, &[]).expect("committed recording redacts");
        for line in lines(&recording).expect("committed lines") {
            if let Line::Secret(value) = line
                && !value.iter().all(|byte| *byte == b'x')
            {
                assert!(find(&output.recording, &value).is_none(), "committed secret gone");
            }
        }
    }
    eprintln!("committed recordings checked: {count}");
    assert!(count > 0, "committed recordings checked");
}

#[test]
fn an_escaped_form_precedes_its_byte_prefix() {
    for (source, expected) in [
        (Line::Path(b"a&amp; a&".as_slice().into()), Line::Path(b"xx xx".as_slice().into())),
        (exchange(b"a&amp; a&"), exchange(b"xxxxxx xx")),
    ] {
        let (_, output) = changed(&[source], &[b"a&".as_slice().into()]);
        assert!(written(&output) == written(&[expected]), "a whole escaped form replaces its byte prefix");
    }
}

#[test]
fn a_digest_replacement_is_not_searched_again() {
    let (_, output) = changed(&[exchange(&digest(b"-"))], &[b"-".as_slice().into()]);
    assert!(written(&output) == written(&[exchange(&digest(b"x"))]), "placeholder digest stands whole");
}

#[test]
fn unreadable_input_is_refused_by_its_line() {
    let result = redact(b"due\nnot-a-record\n", &[]);
    let Err(Unredacted::Unreadable(error)) = result else { panic!("unreadable input refused") };
    assert_eq!(error.line, 2);
}
