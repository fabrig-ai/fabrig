//! First-user ownership of Open briefs and activation-scoped resume instructions.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "programming-model.md, section 10.2: literal prompt fixtures use ordinary Rust"
)]

use super::*;
use crate::{Brief, Mode, Section};

fn policy(mode: Mode) -> Charter {
    Charter {
        mode,
        instructions: bytes(b"Host instructions.\n"),
        brief: Brief {
            sections: Box::new([
                Section { title: bytes(b"Request"), text: bytes("Read café; ask before coding.".as_bytes()) },
                Section { title: Box::default(), text: Box::default() },
                Section { title: bytes(b"Request"), text: bytes(b"Preserve this final line.\n") },
            ]),
        },
        ..charter()
    }
}

// Literal rendering from the pre-change host fixture, including empty and repeated titles.
const BRIEF: &[u8] =
    "## Request\n\nRead café; ask before coding.\n\n## \n\n\n\n## Request\n\nPreserve this final line.\n\n".as_bytes();
const OFFER: &[u8] = b"person: first\n\nperson: second";

fn opened(mode: Mode, resumed: bool, messages: Box<[crate::Message]>) -> Opening {
    opened_workspace(mode, resumed, messages, None)
}

fn opened_workspace(mode: Mode, resumed: bool, messages: Box<[crate::Message]>, mounted: Option<Workspace>) -> Opening {
    let mut start = carried_start(policy(mode), messages, mounted);
    let Event::Start { resumed: selected, transcript, .. } = &mut start else { panic!("Start fixture") };
    *selected = resumed;
    *transcript = resumed.then_some(Token::new(77));
    let mut harness = Harness::new(LIMITS);
    let emitted = harness.step(start);
    let Ok(emitted) = Box::<[Request; 2]>::try_from(emitted) else { panic!("two requests") };
    match *emitted {
        [Request::Admitted { .. }, Request::Open { opening, .. }] => opening,
        [Request::Admitted { .. }, Request::Read { owner, .. }] => {
            let emitted = harness.step(Event::Read { owner, read: Read::Missing });
            let Ok(emitted) = Box::<[Request; 1]>::try_from(emitted) else { panic!("one request after discovery") };
            let [Request::Open { opening, .. }] = *emitted else { panic!("main opens after discovery") };
            opening
        }
        _ => panic!("main opens or discovers its workspace"),
    }
}

fn messages() -> Box<[crate::Message]> {
    Box::new([
        crate::Message { finish_requested: false, name: Token::new(1), label: bytes(b"person"), text: bytes(b"first") },
        crate::Message {
            finish_requested: false,
            name: Token::new(2),
            label: bytes(b"person"),
            text: bytes(b"second"),
        },
    ])
}

#[test]
fn open_first_user_message_is_the_whole_literal_brief() {
    // Instructions and Open mechanics, with no workspace sections.
    const SYSTEM_BYTES: usize = 640;
    let opening = opened(Mode::Open, false, Box::default());
    assert_eq!(opening.prompt.as_ref(), BRIEF);
    let system = core::str::from_utf8(&opening.system).unwrap();
    assert!(system.starts_with("Host instructions.\n\nCall `wait`"));
    for absent in ["## Tools", "## Checkout", "## Request", "Read café", "Preserve this final line.", "Begin the work"]
    {
        assert!(!system.contains(absent));
    }
    for retained in ["Call `wait`", "## Replying", "## Budget"] {
        assert!(system.contains(retained));
    }
    assert_eq!(opening.system.len(), SYSTEM_BYTES);
    let mounted = opened_workspace(Mode::Open, false, Box::default(), Some(workspace()));
    assert_eq!(mounted.prompt.as_ref(), BRIEF);
    let mounted = core::str::from_utf8(&mounted.system).unwrap();
    for heading in ["## Tools", "## Checkout"] {
        assert!(mounted.contains(heading));
    }
}

#[test]
fn open_pre_turn_messages_follow_the_whole_brief() {
    let opening = opened(Mode::Open, false, messages());
    let expected = [BRIEF, OFFER].concat();
    assert_eq!(opening.prompt.as_ref(), expected);
}

#[test]
fn finite_opening_keeps_the_literal_system_and_begin_instruction() {
    let opening = opened(Mode::Finite, false, messages());
    assert_eq!(opening.system.as_ref(), include_bytes!("finite_system.txt"));
    assert_eq!(opening.prompt.as_ref(), b"Begin the work your brief describes.\n\nperson: first\n\nperson: second");
    let mounted = opened_workspace(Mode::Finite, false, messages(), Some(workspace()));
    assert_eq!(mounted.prompt, opening.prompt);
    for heading in [b"## Tools".as_slice(), b"## Checkout"] {
        assert!(skein_lib::bytes::find(&opening.system, heading).is_none());
        assert!(skein_lib::bytes::find(&mounted.system, heading).is_some());
    }
}

#[test]
fn resumed_windows_supersede_exhaustion_and_keep_waking_words() {
    for mode in [Mode::Open, Mode::Finite] {
        for messages in [Box::<[crate::Message]>::default(), messages()] {
            let has_words = !messages.is_empty();
            let opening = opened(mode, true, messages);
            let waking = core::str::from_utf8(&opening.prompt).unwrap();
            assert!(
                waking.starts_with("A new activation starts with the fresh bounded turns and wall time in ## Budget.")
            );
            assert!(waking.contains("Earlier exhaustion and finish-now instructions are superseded"));
            assert!(waking.contains("spending remains bounded by the task's remaining funds"));
            assert_eq!(waking.contains("person: first\n\nperson: second"), has_words);
            assert!(!waking.contains("Begin the work"));
            assert_eq!(opening.transcript, Some(Token::new(77)));
            if mode == Mode::Open {
                assert_eq!(opening.system.as_ref(), include_bytes!("open_system.txt"));
                let mounted = opened_workspace(mode, true, Box::default(), Some(workspace()));
                for heading in [b"## Tools".as_slice(), b"## Checkout"] {
                    assert!(skein_lib::bytes::find(&opening.system, heading).is_none());
                    assert!(skein_lib::bytes::find(&mounted.system, heading).is_some());
                }
            }
        }
    }
}

#[test]
fn moving_the_brief_keeps_its_host_part_charge() {
    // Pre-change rendered charter charge: 20 instruction bytes and 90 brief bytes.
    const HOST_BYTES: u64 = 110;
    let selected = policy(Mode::Open);
    assert_eq!(crate::prompt::host_bytes(&selected, None, 0), Some(HOST_BYTES));
}

#[test]
fn empty_open_brief_keeps_the_message_driven_opening() {
    for (sections, expected) in [
        (Box::<[Section]>::default(), OFFER),
        (
            Box::from([Section { title: Box::default(), text: Box::default() }]),
            b"## \n\n\n\nperson: first\n\nperson: second".as_slice(),
        ),
    ] {
        let selected = Charter { brief: Brief { sections }, ..policy(Mode::Open) };
        assert_eq!(crate::prompt::opening(&selected, false, Some(bytes(OFFER))).as_ref(), expected);
    }
}
