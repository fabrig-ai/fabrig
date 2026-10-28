//! Engine-shaped first-run transcripts: owner lifecycle rows, generated Person
//! inputs, assistant calls, and Person results (client/README.md, 3.5).
use jig_client::contract as c;
use skein_lib::{List, Wall};
pub const CSV: &[u8] = b"Add CSV export to widget";
pub const DARK: &[u8] = b"Add a dark theme to widget";
pub const DELEGATING: &[u8] = "I’ll delegate this change in playground/widget. It will push a branch, open a pull request, check CI and land into main on Playground forge.".as_bytes();
pub const LANDED: &[u8] =
    "CSV export is available from widget’s table. The change landed on Playground forge. This was a scripted change."
        .as_bytes();
pub const DARK_ANSWER: &[u8] = b"I would give widget a dark theme that follows the system's preference, with its colours as variables in one place so both themes stay in step. I am the scripted model and play this from a script, so nothing is changed: connect a model from Accounts to have the work done.";
pub const MERGE: &[u8] = b"1a2b3c4abcdef0123456789abcdef0123456789abcd";
const OPENING: &[u8] = b"Begin the work your brief describes.";
const ACK: &[u8] = b"Delegated tasks: 2";
// The configured delegate's arguments, minified as on the transcript wire.
const DELEGATE_INPUT: &[u8] = br#"{"batch":[{"executor":{"kind":"procedure","connector":1,"code":2},"spec":{"words":"Export widget's table as CSV","parameters":[{"name":1,"kind":"resource","connector":1,"resource":1},{"name":2,"kind":"bytes","value":"main"}]},"contract":{"kind":"change","connector":1,"change_kind":1,"words":128},"authority":{"tools":1,"grants":[{"connector":1,"kind":2,"segments":["forge"],"terminal":"open","last":""},{"connector":1,"kind":3,"segments":["forge"],"terminal":"open","last":""},{"connector":1,"kind":4,"segments":["forge"],"terminal":"open","last":""},{"connector":1,"kind":10,"segments":["forge"],"terminal":"open","last":""},{"connector":1,"kind":11,"segments":["forge"],"terminal":"open","last":""},{"connector":1,"kind":12,"segments":["forge"],"terminal":"open","last":""}],"delegation":{"kinds":[{"kind":"agent","number":2}],"tasks":4,"depth":2},"budget":{"spend":10},"notes":0}}]}"#;
pub fn text(bytes: &[u8]) -> c::Text {
    c::Text::new(Box::from(bytes), u32::try_from(bytes.len()).expect("engine words fit")).expect("uncut engine words")
}
pub fn message(speaker: c::Speaker, blocks: &[c::Block]) -> c::Message {
    let mut kept = List::with_capacity(u32::try_from(blocks.len()).expect("blocks fit"));
    for block in blocks {
        kept.push(block.clone()).expect("block");
    }
    c::Message { speaker, blocks: kept }
}
#[expect(clippy::manual_map, reason = "fixtures spell out options under the step-code lints")]
pub fn call(number: u64, name: &[u8], input: &[u8], outcome: Option<&[u8]>) -> c::Block {
    c::Block::Call(c::Call {
        number,
        tool: text(name),
        input: text(input),
        outcome: match outcome {
            Some(value) => Some(text(value)),
            None => None,
        },
        failed: false,
        objects: List::with_capacity(0),
        parts: List::with_capacity(0),
    })
}
// Jig's translated vocabulary shares one text budget across a turn, including
// call inputs and repeated result summaries (jig-protocol-transcript::Budget).
fn received(value: &c::Text, left: &mut usize, cut: &mut bool, turn_text: u32) -> c::Text {
    let length = value.bytes().len().min(*left);
    *cut |= length < value.bytes().len();
    *left = left.checked_sub(length).expect("receiving text budget");
    c::Text::new(Box::from(&value.bytes()[..length]), turn_text).expect("receiving prefix")
}
#[expect(clippy::manual_map, reason = "fixtures spell out options under the step-code lints")]
pub fn turn(position: u64, attempt: u64, number: u32, messages: &[c::Message], turn_text: u32) -> c::ActivityLine {
    let mut kept = List::with_capacity(u32::try_from(messages.len()).expect("messages fit"));
    let mut left = usize::try_from(turn_text).expect("receiving turn text fits");
    let mut cut = messages.len() > 3;
    for message in messages.iter().take(3) {
        let mut blocks = List::with_capacity(3);
        cut |= message.blocks.len() > 3;
        for block in message.blocks.as_slice().iter().take(3) {
            let block = match block {
                c::Block::Text(value) => c::Block::Text(received(value, &mut left, &mut cut, turn_text)),
                c::Block::Call(call) => {
                    let mut call = call.clone();
                    call.tool = received(&call.tool, &mut left, &mut cut, turn_text);
                    call.input = received(&call.input, &mut left, &mut cut, turn_text);
                    call.outcome = match &call.outcome {
                        Some(value) => Some(received(value, &mut left, &mut cut, turn_text)),
                        None => None,
                    };
                    c::Block::Call(call)
                }
                c::Block::Result { call, summary, failed } => c::Block::Result {
                    call: *call,
                    summary: received(summary, &mut left, &mut cut, turn_text),
                    failed: *failed,
                },
                c::Block::Opaque { .. } => block.clone(),
            };
            blocks.push(block).expect("block");
        }
        kept.push(c::Message { speaker: message.speaker, blocks }).expect("message");
    }
    c::ActivityLine {
        position,
        at: Wall::EPOCH,
        event: c::Activity::Turn {
            name: c::TurnName { attempt, number },
            turn: c::TurnRead::Decoded(c::Turn {
                messages: kept,
                used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
                spent: 0,
                read_through: None,
                cut,
            }),
        },
    }
}
fn owner(task: u64, position: u64, line: c::OwnerActivity) -> c::ActivityLine {
    c::ActivityLine {
        position,
        at: Wall::EPOCH,
        event: c::Activity::Owner {
            source: Box::new(c::OwnerActivityLine { task, position, at: Wall::EPOCH, line }),
            decoded: None,
        },
    }
}
fn result(summary: &[u8]) -> c::Message {
    message(c::Speaker::Person, &[c::Block::Result { call: 3, summary: text(summary), failed: false }])
}
/// Nine retained rows for settled T1; four for T4. No invented Words/News rows.
pub fn activity(dark: bool, landed: bool, turn_text: u32) -> c::ActivityPage {
    let task = if dark { 4 } else { 1 };
    let attempt = if dark { 4 } else { 1 };
    let mut rows = List::with_capacity(9);
    if dark {
        rows.push(owner(task, 4, c::OwnerActivity::RunEnded { attempt, end: c::OwnerRunEnd::Parked, spent: 0 }))
            .expect("parked");
    } else {
        if landed {
            rows.push(owner(task, 9, c::OwnerActivity::RunEnded { attempt: 3, end: c::OwnerRunEnd::Parked, spent: 0 }))
                .expect("parked");
            rows.push(turn(8, 3, 1, &[
                result(b"Begin the work your brief describes.\n\n2: Result from task 2:\n1a2b3c4abcdef0123456789abcdef0123456789abcd"),
                message(c::Speaker::Assistant, &[c::Block::Text(text(LANDED)), call(3, b"wait", b"{}", None)]),
            ], turn_text)).expect("resumed turn");
            rows.push(owner(
                task,
                7,
                c::OwnerActivity::RunBegan { attempt: 3, why: c::OwnerRunWhy::Ready, resumed: true },
            ))
            .expect("resumed");
            rows.push(owner(
                task,
                6,
                c::OwnerActivity::Heard(c::OwnerWord {
                    number: 2,
                    from: c::OwnerRequester::Task(2),
                    kind: c::OwnerMessageKind::Result(c::OwnerWordResult::Change {
                        connector: 1,
                        kind: 1,
                        resource: 2,
                    }),
                    words: text(MERGE),
                    at: Wall::EPOCH,
                    hits: 0,
                    eligible: true,
                }),
            ))
            .expect("heard");
        }
        rows.push(owner(task, 5, c::OwnerActivity::RunEnded { attempt, end: c::OwnerRunEnd::Parked, spent: 0 }))
            .expect("parked");
        rows.push(turn(4, attempt, 2, &[message(c::Speaker::Assistant, &[call(3, b"wait", b"{}", None)])], turn_text))
            .expect("waiting turn");
    }
    let opening = message(c::Speaker::Person, &[c::Block::Text(text(OPENING))]);
    let first = if dark {
        turn(3, attempt, 1, &[opening, message(c::Speaker::Assistant, &[c::Block::Text(text(DARK_ANSWER))])], turn_text)
    } else {
        turn(
            3,
            attempt,
            1,
            &[
                opening,
                message(
                    c::Speaker::Assistant,
                    &[c::Block::Text(text(DELEGATING)), call(3, b"delegate", DELEGATE_INPUT, Some(ACK))],
                ),
                result(ACK),
            ],
            turn_text,
        )
    };
    rows.push(first).expect("first turn");
    rows.push(owner(task, 2, c::OwnerActivity::RunBegan { attempt, why: c::OwnerRunWhy::Made, resumed: false }))
        .expect("began");
    rows.push(owner(task, 1, c::OwnerActivity::Milestone { position: 1 })).expect("initial milestone");
    c::ActivityPage { rows, before: None, after: None, cut: false }
}

/// Serve the retained activity in receiving-sized pages, as the real read does.
pub fn page(activity: &c::ActivityPage, before: Option<&c::Cursor>, rows: u32) -> c::ActivityPage {
    let start = match before {
        Some(cursor) => u32::from_be_bytes(cursor.value.bytes().try_into().expect("fixture cursor")),
        None => 0,
    };
    let start = usize::try_from(start).expect("page offset");
    let end = activity
        .rows
        .as_slice()
        .len()
        .min(start.checked_add(usize::try_from(rows).expect("page rows")).expect("page end"));
    let mut kept = List::with_capacity(rows);
    for line in &activity.rows.as_slice()[start..end] {
        kept.push(line.clone()).expect("page row");
    }
    let before = if end < activity.rows.as_slice().len() {
        Some(c::Cursor::new(Box::from(u32::try_from(end).expect("offset").to_be_bytes()), 4).expect("cursor"))
    } else {
        None
    };
    c::ActivityPage { rows: kept, before, after: None, cut: false }
}
