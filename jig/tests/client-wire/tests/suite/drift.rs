//! The client records' drift (skein's data/codec/golden.md, 1).
use jig_client_protocol_wire::Record as _;
#[path = "../../../codec-families.rs"]
mod families;

#[test]
fn codec_families_match_their_schemas() {
    families::check();
    current_client_layout();
}

#[expect(clippy::disallowed_methods, reason = "jig's drift write mode refreshes the current layout")]
fn current_client_layout() {
    use std::{fmt::Write, fs};
    let root = skein_scratch::paths::tree().join("crates/jig-client-protocol-wire/schema");
    let text = fs::read_to_string(root.join("client-v43.schema")).expect("schema")
        + &fs::read_to_string(root.join("client-v43-part1.schema")).expect("schema fragment");
    let schema = skein_codec::schema::parse(text.as_bytes()).expect("schema valid");
    let mut line = format!("{} ", schema.version());
    for byte in schema.fingerprints().family().digest() {
        write!(&mut line, "{byte:02x}").expect("string write");
    }
    line.push('\n');
    let path = root.join("jig-client-wire.layouts");
    let current = fs::read_to_string(&path).expect("layouts");
    let version = format!("{} ", schema.version());
    let mut prefix = String::new();
    for row in current.lines() {
        if !row.starts_with(&version) {
            writeln!(&mut prefix, "{row}").expect("string write");
        }
    }
    let expected = prefix + &line;
    if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1") {
        fs::write(path, expected).expect("write current layout");
    } else {
        assert_eq!(current, expected, "run the drift in write mode");
    }
}

#[test]
fn the_hello_alone_keeps_version_27_bytes() {
    use jig_client_protocol_wire as w;
    use skein_lib::{Reader, Writer};
    let full: &[u8] = &[
        0, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 0, 0, 0, 1, 0, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    ];
    for (expected, fixed_value) in [
        (full, super::client_values::fixed_value("enum_stream_event_hello_full.bin")),
        ([0_u8; 29].as_slice(), super::client_values::fixed_value("enum_stream_event_hello_smallest.bin")),
    ] {
        assert_eq!(fixed_value, expected);
        let value = w::StreamEvent::decode(&w::CEILINGS, &mut Reader::new(expected)).expect("version 27 hello");
        assert!(matches!(value, w::StreamEvent::Hello(_)));
        let mut writer = Writer::new(
            value
                .measure(&jig_client_protocol_wire::CEILINGS)
                .expect("bounded fixture")
                .try_into()
                .expect("short hello"),
        );
        value.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("hello fits");
        assert_eq!(writer.finish().as_ref(), expected);
    }
}
