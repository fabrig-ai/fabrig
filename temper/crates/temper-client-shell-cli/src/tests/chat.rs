use super::fixtures::Frame;
use jig_client_protocol_wire as w;
use jig_client_test_records::{activity_rows, turn_line};
use skein_io::kernel::Exit;
use skein_lib::{List, Writer, bytes};
use smith_transcript::{self as transcript, Record as _};

const REPLY: &[u8] = b"World answer";
const EARLY: &[u8] = b"World";
const TASK: u64 = 7;
const ATTEMPT: u64 = 44;

fn committed_turn() -> Box<[u8]> {
    let mut blocks = List::with_capacity(1);
    blocks.push(transcript::Block::Text(transcript::Text { text: REPLY.into(), replay: None })).expect("one block");
    let mut messages = List::with_capacity(1);
    messages.push(transcript::Message { role: transcript::Role::Assistant, blocks }).expect("one message");
    let turn = transcript::Turn {
        kind: transcript::TurnKind::Ordinary,
        opening: None,
        endpoint: b"fake".as_slice().into(),
        model: b"fake".as_slice().into(),
        effort: None,
        dialect: 0,
        place: 0,
        usage: transcript::Usage { input: None, output: None, cache_read: None, cache_write: None, reasoning: None },
        spent: 0,
        jobs: 0,
        messages,
        system: None,
    };
    let size = turn.measure(&transcript::CEILINGS).expect("bounded turn");
    let mut writer = Writer::new(skein_lib::length::usize_of(size));
    turn.write(&transcript::CEILINGS, &mut writer).expect("measured turn");
    writer.finish()
}

fn show(rows: &[w::DecodedActivityLine], committed_turn: u32, writing: Option<w::RunText>) -> Frame {
    let mut frame = Frame::read(&["chat", "show", "7"]);
    let activity = w::ActivityPage { cut: false, after: None, ..activity_rows(rows) };
    frame.script_chat_read(
        activity,
        w::RunState { task: TASK, attempt: Some(ATTEMPT), committed_turn, writing, call: None },
    );
    frame.sign_in_then_deliver_me();
    assert_eq!(
        frame.exit(),
        Some(Exit::Code(0)),
        "after {} requests at committed turn {committed_turn}: {} {}",
        frame.requests(),
        core::str::from_utf8(frame.stdout()).expect("UTF-8 stdout"),
        core::str::from_utf8(frame.stderr()).expect("UTF-8 stderr")
    );
    frame
}

#[test]
fn a_chat_read_without_live_text_prints_the_committed_reply_once() {
    let began = w::DecodedActivityLine {
        source: w::ActivityLine {
            task: TASK,
            position: 9,
            at: 19,
            line: w::Activity::RunBegan(w::ActivityRunBegan {
                attempt: ATTEMPT,
                why: w::RunWhy::Made,
                resumed: false,
                fresh: None,
            }),
        },
        decoded: None,
    };
    let early = show(core::slice::from_ref(&began), 0, None);
    assert_eq!(bytes::find(early.stdout(), EARLY), None, "no published text yet");

    let committed = show(&[began.clone(), turn_line(10, 20, 1, &committed_turn())], 1, None);
    let stdout = core::str::from_utf8(committed.stdout()).expect("chat output");
    let reply = core::str::from_utf8(REPLY).expect("reply text");
    assert_eq!(stdout.matches(reply).count(), 1, "{stdout}");

    // The same activity page as the early read, but the run supplies live text.
    let with_live = show(
        &[began],
        0,
        Some(w::RunText {
            attempt: ATTEMPT,
            began: None,
            turn: w::ProvisionalTurnName { attempt: ATTEMPT, number: 1, retry: 0 },
            from: 0,
            through: u32::try_from(EARLY.len()).expect("short prefix"),
            text: EARLY.into(),
            cut: false,
            call: None,
            writing: None,
        }),
    );
    assert!(bytes::find(with_live.stdout(), EARLY).is_some(), "supplied provisional text is printed");
}
