//! Startup admission edges (temper/docs/design/protocol/configuration/README.md, 4, temper/docs/design/protocol/configuration/limits.md, temper/docs/design/protocol/configuration/README.md, 9).
use crate::support;

use serde_json::{Value, json};
use std::{fs, mem::size_of};

#[derive(Clone, Copy)]
enum Part {
    Engine,
    Application,
}

impl Part {
    fn limits(self) -> &'static str {
        match self {
            Self::Engine => "/engine/limits/overrides",
            Self::Application => "/application/limits/overrides",
        }
    }

    fn path(self) -> &'static str {
        match self {
            Self::Engine => "engine.limits.overrides",
            Self::Application => "application.limits.overrides",
        }
    }
}

fn number(document: &Value, part: Part, name: &str) -> u64 {
    let worker = temper_service::startup::box_worker(
        &serde_json::to_vec(document).expect("fixture JSON"),
        &jig_protocol_config::Directories::working(),
        &temper_service::stock::Given::Stock,
        &temper_service::stock::agents(),
        &[],
    )
    .expect("fixture composition")
    .expect("box worker");
    for limit in worker.figures.expect("composed figures").limits() {
        if limit.name() == name.as_bytes() {
            return limit.value();
        }
    }
    panic!("{}.{name}: missing composed figure", part.path());
}

fn set(document: &mut Value, part: Part, name: &str, bytes: u64) {
    document.pointer_mut(part.limits()).expect("limits")[name] = json!(bytes);
}

// The read's row has no public size query in this world. Check its advertised
// required figure against admission on both sides, rather than inventing a row size.
fn edge(part: Part, name: &str) {
    let scratch = support::Scratch::startup(&format!("plan-chat-{name}"));
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let directories = support::configuration::directories(scratch.path());
    let mut document: Value =
        serde_json::from_slice(&fs::read(scratch.path().join("temper.json")).expect("fixture document"))
            .expect("fixture JSON");
    let admit = |document: &Value| crate::admission(&directories, &serde_json::to_vec(document).expect("JSON"));
    admit(&document).expect("unmodified fixture must start");
    let turn = number(&document, Part::Engine, "hub.turn_bytes");
    let resume = number(&document, Part::Engine, "core.resume_bytes");
    let required = match name {
        "hub.turn_bytes" | "root.loads.transcript_bytes" => turn,
        "core.resume_bytes" => turn + 4,
        "root.journal.transcript_bytes" => resume,
        "hub.transcript_bytes" => {
            let box_bytes = u64::try_from(size_of::<Box<[u8]>>()).expect("box size");
            resume + (box_bytes - 4) * (resume / 4)
        }
        "root.loads.bytes" => number(&document, part, name),
        "root.loads.read_bytes" => {
            set(&mut document, part, name, turn);
            let refusal = admit(&document).expect_err("a bare turn leaves no read row");
            let figure = refusal
                .split(|character: char| !character.is_ascii_digit())
                .rfind(|word| !word.is_empty())
                .expect("refusal must name required bytes")
                .parse::<u64>()
                .expect("required byte figure");
            assert!(figure > turn, "a read includes its row: {refusal}");
            assert!(figure <= turn * 2, "the documented read room fits: {refusal}");
            figure
        }
        _ => panic!("unlisted turn bound: {name}"),
    };
    set(&mut document, part, name, required);
    admit(&document).unwrap_or_else(|why| panic!("{name} at {required} must start: {why}"));
    let short = required - 1;
    set(&mut document, part, name, short);
    let refusal = admit(&document).expect_err(&format!("one byte short must refuse startup for {name} at {short}"));
    let path = format!("{}.{name}", part.path());
    assert!(refusal.contains(&path), "missing {path}: {refusal}");
    assert!(refusal.contains(&format!("{short} bytes")), "missing configured value: {refusal}");
    assert!(refusal.contains(&format!("{required} bytes")), "missing required figure: {refusal}");
    scratch.remove();
}

#[test]
fn transcript_padding_covers_all_resume_window_remainders() {
    const LENGTH_BYTES: u64 = 4;
    let box_bytes = u64::try_from(size_of::<Box<[u8]>>()).expect("box size");
    for remainder in 0..LENGTH_BYTES {
        let scratch = support::Scratch::startup(&format!("plan-chat-window-{remainder}"));
        support::configuration(scratch.path(), &support::Forge::Unreached);
        let directories = support::configuration::directories(scratch.path());
        let mut document: Value =
            serde_json::from_slice(&fs::read(scratch.path().join("temper.json")).expect("fixture document"))
                .expect("fixture JSON");
        let base = number(&document, Part::Engine, "core.resume_bytes");
        let window = (base / LENGTH_BYTES + 1) * LENGTH_BYTES + remainder;
        let transcript = window + (box_bytes - LENGTH_BYTES) * (window / LENGTH_BYTES);
        set(&mut document, Part::Engine, "core.resume_bytes", window);
        set(&mut document, Part::Application, "root.journal.transcript_bytes", window);
        {
            let part = Part::Engine;
            set(&mut document, part, "hub.transcript_bytes", transcript);
        }
        let admit = |document: &Value| crate::admission(&directories, &serde_json::to_vec(document).expect("JSON"));
        admit(&document).expect("both hubs hold the padded configured window");
        {
            let part = Part::Engine;
            set(&mut document, part, "hub.transcript_bytes", transcript - 1);
            let refusal = admit(&document).expect_err("one padding byte short");
            let path = format!("{}.hub.transcript_bytes", part.path());
            assert!(refusal.contains(&path), "missing {path}: {refusal}");
            assert!(refusal.contains(&format!("{} bytes", transcript - 1)), "missing offered size: {refusal}");
            assert!(refusal.contains(&format!("{transcript} bytes")), "missing padded window: {refusal}");
            set(&mut document, part, "hub.transcript_bytes", transcript);
        }
        scratch.remove();
    }
}

#[test]
fn turn_and_resume_bounds_admit_the_edge_and_refuse_one_byte_short() {
    for (part, name) in [
        (Part::Engine, "hub.turn_bytes"),
        (Part::Engine, "core.resume_bytes"),
        (Part::Application, "root.journal.transcript_bytes"),
        (Part::Application, "root.loads.transcript_bytes"),
        (Part::Application, "root.loads.bytes"),
        (Part::Application, "root.loads.read_bytes"),
        (Part::Engine, "hub.transcript_bytes"),
    ] {
        edge(part, name);
    }
}
#[test]
fn obsolete_owned_organization_members_refuse_startup() {
    let scratch = support::Scratch::startup("machine-moved");
    support::first_repository::generated(scratch.path());
    let file = scratch.path().join("temper.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&file).expect("configuration")).expect("JSON");
    value["application"]["machine"] = json!({});
    let why = temper_box_shell::operations_in(
        &[],
        b"/",
        &support::configuration::directories(scratch.path()),
        &serde_json::to_vec(&value).expect("JSON"),
    )
    .expect_err("machine moved");
    assert!(
        why.contains("application.machine: moved: a forge's place is chosen when it's set up, and kept with the forge"),
        "{why}"
    );
    scratch.remove();
}
