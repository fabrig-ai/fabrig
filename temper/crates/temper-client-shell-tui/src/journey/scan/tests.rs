use super::*;
use crate::{
    host::Report,
    journey::{
        Sent,
        redact::tests::{World, printed_link, written},
    },
};
use skein_ui_screen::Size;
use temper_client_redaction::{Kind, Shape};

#[test]
fn codes_and_shapes_are_scanned_in_fields_and_turn_frames() {
    // Short fake bytes keep both TUI link forms visible in one frame's row.
    let originals = Originals::new(vec![(Kind::Link, Box::new([255; 4]))], &[]);
    let shown = b"WDJB-MJHT".as_slice();
    let browse = Line::Browse([b"http://127.0.0.1:8367/approve?code=".as_slice(), shown].concat().into());
    let codes = codes(&[browse]);
    let key = [b"sk-".as_slice(), &[b'A'; 20]].concat();
    let bearer = [b"Bearer ".as_slice(), &[b'B'; 16]].concat();
    let cases = [
        ("approval code", b"code=WDJB-MJHT".as_slice(), None),
        ("other valid approval", b"code=BCDF-GHJK".as_slice(), Some(What::Shape(Shape::Code))),
        ("other token code", b"code=abcdefgh".as_slice(), Some(What::Shape(Shape::Code))),
        ("known code prefix", b"code=WDJB-MJHTsuffix".as_slice(), Some(What::Shape(Shape::Code))),
        ("key", key.as_slice(), Some(What::Shape(Shape::Key))),
        ("bearer", bearer.as_slice(), Some(What::Shape(Shape::Bearer))),
        ("link base64", b"_____w".as_slice(), Some(What::Original)),
        ("link decimal", b"[95, 95, 95, 95, 95, 119]".as_slice(), Some(What::Original)),
    ];
    let mut failures = Vec::new();
    if codes != vec![Box::<[u8]>::from(shown)] {
        failures.push("browse code reader".to_owned());
    }
    for (name, bytes, expected) in cases {
        if scan_bytes(bytes, &codes, &originals) != expected {
            failures.push(format!("{name}: bytes"));
        }
        // A field is checked before replay; no valid seam is needed to report it.
        if let Some(what) = expected {
            let found = scan(&written(&[Line::Signed(bytes.into())]), &originals).expect("field scan");
            if found != Some(Found { line: 1, record: "signed", place: Place::Field, what }) {
                failures.push(format!("{name}: field"));
            }
        }
        // The renderer drops a zero-width character. No recorded field holds
        // the whole shape, but the first turn's frame does. A resize clears
        // that frame before the end, and there are no intermediate marks.
        let address = [&bytes[..3], "\u{200b}".as_bytes(), &bytes[3..], b"&"].concat();
        let mut world = World::at_address(&address);
        let exchange = world.exchange(false);
        let mut asked = vec![0];
        asked.extend_from_slice(&32_u32.to_be_bytes());
        asked.extend_from_slice(&[7; 32]);
        asked.extend_from_slice(b"WDJBMJHT");
        asked.extend_from_slice(&300_000_000_000_u64.to_be_bytes());
        world.report(Report::Heard(jig_client::Event::Reply { exchange, message: asked.into() }));
        world.report(Report::Opened { opened: true });
        world.report(Report::Resize(Size { rows: 0, columns: 0 }));
        let mut records = crate::journey::lines(&world.save()).expect("world lines");
        for record in &mut records {
            if let Line::Mark(print) | Line::End(print) = record {
                *print = None;
            }
        }
        let file = written(&records);
        match scan(&file, &originals) {
            Ok(found) => {
                if found.map(|found| found.what) != expected {
                    failures.push(format!("{name}: frame result"));
                }
                if let Some(found) = found {
                    let turn = 1;
                    let at = records
                        .iter()
                        .enumerate()
                        .filter(|(_, line)| matches!(line, Line::Deadline(_)))
                        .nth(usize::try_from(turn - 1).expect("turn index"))
                        .expect("deadline")
                        .0;
                    if found
                        != (Found {
                            line: u32::try_from(at + 1).expect("line"),
                            record: "deadline",
                            place: Place::Frame { turn },
                            what: found.what,
                        })
                    {
                        failures.push(format!("{name}: frame location"));
                    }
                }
            }
            Err(_) => failures.push(format!("{name}: replay refused")),
        }
    }
    let originals = Originals::new(vec![(Kind::Given, shown.into())], &[]);
    if scan_bytes(b"code=WDJB-MJHT", &codes, &originals) != Some(What::Original) {
        failures.push("original still sought inside allowed code".to_owned());
    }
    for address in [
        b"http://host/?other=code=WDJB-MJHT".as_slice(),
        b"http://host/?code=abcdefgh",
        b"http://host/?code=WDJB-MJHTsuffix",
    ] {
        if !super::codes(&[Line::Browse(address.into())]).is_empty() {
            failures.push("invalid browse query admitted".to_owned());
        }
    }
    sign_in_fields(&key, &mut failures);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn sign_in_fields(key: &[u8], failures: &mut Vec<String>) {
    // A wire-refused sign-in body is no original: a matching shape remains a refusal.
    for message in [Sent::SignInApproval, Sent::SignInLink] {
        let records = [Line::Send { exchange: 1, message: message(key.into()), credential: None }];
        let originals = crate::journey::originals(&records, &[]);
        if !originals.sought().is_empty()
            || scan(&written(&records), &originals).expect("shape in refused body").is_none()
        {
            failures.push("wire-refused body hidden".to_owned());
        }
    }
    let source = Line::BoxLine([b"link ".as_slice(), &printed_link()].concat().into());
    let originals = crate::journey::originals(&[source], &[]);
    for (name, field) in [
        ("box-line printed link", Line::BoxLine([b"log ".as_slice(), &printed_link()].concat().into())),
        ("box-line raw secret", Line::Signed(Box::new([0; 32]))),
        ("box-line decimal secret", Line::Signed(format!("[{}]", ["65"; 43].join(", ")).into_bytes().into())),
    ] {
        let expected = Found { line: 1, record: field.kind(), place: Place::Field, what: What::Original };
        if scan(&written(&[field]), &originals).ok().flatten() != Some(expected) {
            failures.push(format!("{name}: original not refused in {} field", expected.record));
        }
    }
}
