//! Provider-drawn workspace references and their recorded answers.

use skein_fake_llm_domain::api::{Bounds, Line, Script, Source, Turn};
use skein_lib::Time;
use smith_domain_session::{llm::Endpoint, record};
use smith_domain_tools::Grants;
use smith_session_world::fixture::{calls, says};
use smith_session_world::provider::{SLOT_NAMES, UNISSUED_PATH, menu};
use smith_session_world::shapes::{Answer, Class};
use smith_session_world::{Settings, World, bytes, spec};

const SYSTEM: &[u8] = b"You are a coding agent.";
const TASK: &[u8] = b"Inspect `src/lib.rs` and `README.md`.";
const ITERATIONS: u32 = 10_000;

fn call(name: &[u8], arguments: &[u8]) -> Turn {
    calls(Box::new([Line::Call { name: bytes(name), arguments: bytes(arguments) }]), 48)
}

fn script(turns: Vec<Turn>) -> Box<[Script]> {
    Box::new([Script { cue: bytes(SYSTEM), turns: turns.into() }])
}

#[test]
fn the_menu_has_every_field_and_the_five_slots() {
    let menu = menu();
    let numeric = Some(Bounds { low: 1, high: u64::from(u32::MAX) });
    let expected = [
        (b"job".as_slice(), b"job ".as_slice(), b" ".as_slice(), b"999999".as_slice(), numeric),
        (b"offset", b"[next line ", b"]", b"999999", numeric),
        (b"listed", b"\n", b"\t", UNISSUED_PATH, None),
        (b"found", b"\n", b":", UNISSUED_PATH, None),
        (b"named", b"`", b"`", UNISSUED_PATH, None),
    ];
    assert_eq!(menu.slots.len(), expected.len());
    for (slot, (name, before, after, unissued, kind)) in menu.slots.iter().zip(expected) {
        assert_eq!(
            (&*slot.name, &*slot.before, &*slot.after, &*slot.unissued, slot.numeric, slot.weight),
            (name, before, after, unissued, kind, 100)
        );
    }
    let fields: &[&[u8]] = &[
        b"path",
        b"offset",
        b"window",
        b"depth",
        b"glob",
        b"pattern",
        b"content",
        b"edits",
        b"command",
        b"timeout_seconds",
        b"output_bytes",
        b"yield_seconds",
        b"job",
        b"wait_seconds",
        b"stop",
    ];
    let client = skein_llm_kit::limits();
    for body in menu.arguments.iter().chain(menu.invalid[3..].iter().map(|entry| &entry.arguments)) {
        let size = u32::try_from(body.len()).expect("bounded fixture");
        let bounds = skein_llm::DocumentLimits { bytes: size, strings: size, tokens: size, depth: client.depth };
        let json = skein_llm::Json::from_bytes(body, &bounds).expect("the template is a JSON object");
        let document = json.document();
        for field in fields {
            assert!(document.member(0, field).is_some(), "missing {field:?} in {body:?}");
        }
        let edits = document.member(0, b"edits").expect("edits");
        for field in [b"old".as_slice(), b"new", b"all"] {
            assert!(document.member(edits + 1, field).is_some(), "each replacement has {field:?}");
        }
    }
    assert!(skein_fake_llm_domain::Domain::configured(&Settings::calm(0).provider, 0, Box::new([]), menu).is_ok());
}

#[test]
fn workspace_references_record_their_classes_beside_the_answer() {
    let turns = vec![
        call(b"read", br#"{"path":"src/lib.rs","offset":1}"#),
        call(b"write", br#"{"path":"notes.md","content":"notes.md: noted"}"#),
        call(b"remove", br#"{"path":"notes.md"}"#),
        call(b"read", br#"{"path":"notes.md"}"#),
        call(b"read", br#"{"path":"../outside.txt"}"#),
        call(b"read", br#"{"path":"never-issued.txt","offset":1}"#),
        call(b"read", br#"{"path":"bad\u0000path","offset":1}"#),
        call(b"read", br#"{"path":"src/lib.rs","offset":999999}"#),
        call(b"search", br#"{"pattern":"guide"}"#),
        says(Box::new([Line::Text { text: bytes(b"done") }]), 48),
    ];
    let mut world = World::with_scripts(Settings::calm(1), script(turns));
    let opener = world.submit(Time::ZERO, spec(TASK));
    world.run(ITERATIONS);
    let calls =
        world.session(opener).path_calls.iter().map(|call| (call.path, call.offset, call.answer)).collect::<Vec<_>>();
    assert_eq!(
        calls,
        [
            (Some(Class::Live), Some(Class::Live), Answer::Ok),
            (Some(Class::Unissued), None, Answer::Ok),
            (Some(Class::Live), None, Answer::Ok),
            (Some(Class::Stale), None, Answer::NotFound),
            (Some(Class::Foreign), None, Answer::Outside),
            (Some(Class::Unissued), Some(Class::Unknown), Answer::NotFound),
            (Some(Class::Unknown), Some(Class::Unknown), Answer::Malformed),
            (Some(Class::Live), Some(Class::Unissued), Answer::Ok),
        ]
    );
    let reached = world.reached();
    for shape in [
        "read:ok:path=live:offset=live",
        "read:not_found:path=stale",
        "read:outside:path=foreign",
        "read:not_found:path=unissued:offset=unknown",
        "read:malformed:path=unknown:offset=unknown",
        "read:ok:path=live:offset=unissued",
        "search:ok",
    ] {
        assert!(reached.contains(&format!("{shape} end=unknown begin=unknown wake=unknown")), "missing {shape}");
    }
}

#[test]
fn drawn_workspace_slots_use_rendered_candidates_and_unissued_values() {
    const SHOWN_PATHS_AND_UNISSUED_OFFSET: u64 = 2;
    const SHOWN_OFFSET_AND_UNISSUED_LISTING: u64 = 5;

    // The four scripted turns put a continuation, a later list entry and a
    // later search hit in the provider's history; the opening names two files.
    let mut first = World::with_scripts(
        Settings::calm(1),
        script(vec![
            call(b"read", br#"{"path":"docs/guide.md","offset":1,"window":28}"#),
            call(b"read", br#"{"path":"docs/guide.md","offset":2,"window":28}"#),
            call(b"list", br#"{"path":"."}"#),
            call(b"search", br#"{"pattern":"guide"}"#),
        ]),
    );
    let opener = first.submit(Time::ZERO, spec(TASK));
    first.run(ITERATIONS);
    let history = record::Transcript {
        version: record::VERSION,
        endpoint: Some(Endpoint(0)),
        dialect: 2,
        turns: first.session(opener).records.clone().into(),
    };
    let mut draws = Vec::new();
    for seed in [SHOWN_PATHS_AND_UNISSUED_OFFSET, SHOWN_OFFSET_AND_UNISSUED_LISTING] {
        let mut settings = Settings::calm(seed);
        settings.provider.tool_rounds = 12;
        settings.provider.calls_per_answer = 2;
        settings.provider.capture_answers = 16;
        settings.provider.capture_bytes = 32_768;
        let mut world = World::new(settings);
        let mut opening = spec(TASK);
        opening.authority.grants = Grants { inspect: true, modify: false, shell: false };
        world.submit_restored(Time::ZERO, opening, history.clone());
        world.run(ITERATIONS);
        let capture = world.capture();
        assert!(!capture.full);
        for answer in capture.answers {
            for draw in answer.slots {
                let index = usize::try_from(draw.slot).expect("bounded slot");
                let Line::Call { name, .. } =
                    &answer.answer.lines[usize::try_from(draw.call).expect("bounded position")]
                else {
                    unreachable!("slots belong to calls");
                };
                if index > 0 && (index != 1 || name.as_ref() == b"read") {
                    draws.push((SLOT_NAMES[index], draw.source, draw.value, draw.class.clone()));
                }
                assert!(!draw.class.is_empty(), "every captured draw carries a class");
            }
        }
    }
    let expected: &[(&[u8], Source, &[u8], Class)] = &[
        (b"offset", Source::Conversation, b"2", Class::Live),
        (b"offset", Source::Unissued, b"999999", Class::Unissued),
        (b"listed", Source::Conversation, b"src/lib.rs", Class::Live),
        (b"listed", Source::Unissued, UNISSUED_PATH, Class::Unissued),
        (b"found", Source::Conversation, b"docs/guide.md", Class::Live),
        (b"found", Source::Unissued, UNISSUED_PATH, Class::Unissued),
        (b"named", Source::Conversation, b"README.md", Class::Live),
        (b"named", Source::Unissued, UNISSUED_PATH, Class::Unissued),
    ];
    for &(slot, source, value, class) in expected {
        assert!(
            draws.iter().any(|(name, drawn, bytes, label)| {
                (*name, *drawn, &**bytes, &**label) == (slot, source, value, class.word().as_bytes())
            }),
            "missing {slot:?}={value:?} from {source:?}, class {class:?}"
        );
    }
}
