//! The maps command writes a translation or prints a refusal (data/codec/command.md, 9).

use std::{fs, process::Command};

use skein_codegen::{parse, translate};

#[test]
fn maps_command_uses_split_schema_order_and_the_first_stem() {
    let scratch = skein_scratch::Scratch::new("maps-command");
    let first = scratch.path().join("worker-wire-v1.schema");
    let second = scratch.path().join("worker-wire-v1-part1.schema");
    let maps = scratch.path().join("worker-wire.maps");
    let out = scratch.path().join("generated");
    let source = "family worker_wire 1\nrecord Row { value: u64 }\n";
    let tail = "record Root { row: Row }\n";
    let mapping = "records r\nrecord Row = d::Row\nrecord Root by hand\n";
    fs::write(&first, source).expect("first schema");
    fs::write(&second, tail).expect("second schema");
    fs::write(&maps, mapping).expect("maps");
    let result = Command::new(skein_scratch::paths::binary("skein-codegen"))
        .args([first.as_os_str(), second.as_os_str(), maps.as_os_str(), out.as_os_str()])
        .output()
        .expect("run maps command");
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let schema = parse(&format!("{source}{tail}")).expect("split schema");
    assert_eq!(
        fs::read_to_string(out.join("worker_wire.rs")).expect("translation"),
        translate(&schema, mapping).expect("translate")
    );
    assert_eq!(fs::read_dir(out).expect("output directory").count(), 1);
}

#[test]
fn maps_command_prints_a_refusal_and_writes_nothing() {
    let scratch = skein_scratch::Scratch::new("maps-refusal");
    let schema = scratch.path().join("refusal-v1.schema");
    let maps = scratch.path().join("refusal.maps");
    let out = scratch.path().join("generated");
    fs::write(&schema, "family refusal 1\nrecord Scalar { value: u64 }\n").expect("schema input");
    fs::write(&maps, "records r\nrecord Absent = d::Absent").expect("maps input");
    let result = Command::new(skein_scratch::paths::binary("skein-codegen"))
        .args([schema.as_os_str(), maps.as_os_str(), out.as_os_str()])
        .output()
        .expect("run maps command");
    assert!(!result.status.success(), "command accepted invalid maps");
    let error = String::from_utf8(result.stderr).expect("error is text");
    assert!(error.contains("line 2: schema refusal"), "{error}");
    assert!(!out.exists(), "command wrote output");
}
