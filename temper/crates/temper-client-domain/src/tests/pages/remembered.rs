//! Requests and the page after each answer (temper/docs/design/client/pages/remembered.md).
use super::{accounts::deliver, fixtures};
use crate::{
    Address,
    pages::{
        self, Pages,
        remembered::{Act, Refusal, Row},
    },
};
use jig_client::{Answered, Outcome, Readable, Readout, Told, contract as c};
use skein_lib::{Duration, List, Queue, Time, Wall};

fn text(words: &[u8]) -> c::Text {
    c::Text::new(words.into(), u32::try_from(words.len()).unwrap()).unwrap()
}
fn note(number: u64, line: &[u8]) -> c::Note {
    let mut tasks = List::uncapped();
    tasks.push(14).unwrap();
    tasks.push(19).unwrap();
    c::Note {
        number,
        revision: c::Revision { value: number.checked_add(20).unwrap() },
        scope: c::NoteScope::Party { party: fixtures::PARTY },
        description: c::DocumentText::complete(text(line)),
        body: c::DocumentText::complete(text(b"")),
        writer: if number == 2 {
            c::NoteAuthor::Party { party: fixtures::PARTY }
        } else {
            c::NoteAuthor::Task { task: 99, attempt: 3 }
        },
        tasks,
        changed: Some(Wall::from_nanos(100_000_000_000)),
        acts: List::uncapped(),
        cut: false,
    }
}
fn cursor() -> c::Cursor {
    c::Cursor::new(b"next".as_slice().into(), 32).unwrap()
}
#[derive(Clone, Copy)]
enum Input {
    Enter,
    Me,
    Cut,
    Leave,
    Notes { first: u64, count: u64, more: bool, line: &'static [u8] },
    Change(u64),
    Words(&'static [u8]),
    Save,
    Cancel,
    Delete(u64),
    Done,
    Refused,
    Moved,
    Long(Option<u32>),
    Busy,
    ReadRefused,
    Fire,
    LateRead,
}
#[derive(Clone, Copy)]
enum Sent {
    None,
    Read(bool),
    Edit(u64, &'static [u8]),
    Delete(u64),
}
struct Step {
    input: Input,
    sent: Sent,
    count: usize,
    answered: bool,
    field: Option<(u64, &'static [u8])>,
    refusal: Option<(u64, &'static str)>,
}
fn row(input: Input, sent: Sent, count: usize, answered: bool) -> Step {
    Step { input, sent, count, answered, field: None, refusal: None }
}
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one request and state table follows the remembered page through its answers"
)]
fn remembered_requests_and_answers() {
    use Input::{
        Busy, Cancel, Change, Cut, Delete, Done, Enter, Fire, LateRead, Leave, Long, Me, Moved, Notes, ReadRefused,
        Refused, Save, Words,
    };
    let cases: [Box<[Step]>; 4] = [
        Box::new([
            row(Enter, Sent::None, 0, false),
            row(Me, Sent::Read(false), 0, false),
            row(Notes { first: 1, count: 0, more: false, line: b"line" }, Sent::None, 0, true),
        ]),
        Box::new([
            row(Enter, Sent::None, 0, false),
            row(Me, Sent::Read(false), 0, false),
            row(Notes { first: 7, count: 6, more: true, line: b"line" }, Sent::Read(true), 0, false),
            row(Notes { first: 1, count: 6, more: false, line: b"line" }, Sent::None, 12, true),
        ]),
        Box::new([
            row(Enter, Sent::None, 0, false),
            row(Me, Sent::Read(false), 0, false),
            row(Notes { first: 1, count: 3, more: false, line: b"line" }, Sent::None, 3, true),
            Step { field: Some((1, b"line")), ..row(Change(1), Sent::None, 3, true) },
            Step { field: Some((2, b"line")), ..row(Change(2), Sent::None, 3, true) },
            row(Cancel, Sent::None, 3, true),
            Step { field: Some((1, b"line")), ..row(Change(1), Sent::None, 3, true) },
            Step { field: Some((1, b" \t\n")), ..row(Words(b" \t\n"), Sent::None, 3, true) },
            Step { field: Some((1, b" \t\n")), ..row(Save, Sent::None, 3, true) },
            Step { field: Some((1, b" new line \n")), ..row(Words(b" new line \n"), Sent::None, 3, true) },
            Step { field: Some((1, b" new line \n")), ..row(Save, Sent::Edit(1, b"new line"), 3, true) },
            Step { field: Some((1, b" new line \n")), ..row(Delete(2), Sent::None, 3, true) },
            Step { field: Some((1, b" new line \n")), ..row(Change(3), Sent::None, 3, true) },
            row(Done, Sent::Read(false), 3, false),
            row(Busy, Sent::None, 3, false),
            row(Fire, Sent::Read(false), 3, false),
            row(Notes { first: 1, count: 3, more: false, line: b"new line" }, Sent::None, 3, true),
            Step { field: Some((1, b"new line")), ..row(Change(1), Sent::None, 3, true) },
            Step { field: Some((1, b"new line")), ..row(Save, Sent::Edit(1, b"new line"), 3, true) },
            Step { refusal: Some((1, "moved")), ..row(Moved, Sent::Read(false), 3, false) },
            Step {
                refusal: Some((1, "moved")),
                ..row(Notes { first: 1, count: 3, more: false, line: b"moved line" }, Sent::None, 3, true)
            },
            row(Delete(2), Sent::Delete(2), 3, true),
            row(Done, Sent::Read(false), 3, false),
            row(Notes { first: 3, count: 1, more: false, line: b"left" }, Sent::None, 1, true),
            row(Delete(3), Sent::Delete(3), 1, true),
            Step { refusal: Some((3, "other")), ..row(Refused, Sent::Read(false), 1, false) },
            Step { refusal: Some((3, "other")), ..row(ReadRefused, Sent::None, 1, false) },
            Step { field: Some((3, b"left")), ..row(Change(3), Sent::None, 1, false) },
            Step { field: Some((3, b"left")), ..row(Save, Sent::Edit(3, b"left"), 1, false) },
            Step { refusal: Some((3, "long")), ..row(Long(Some(64)), Sent::Read(false), 1, false) },
            Step {
                refusal: Some((3, "long")),
                ..row(Notes { first: 3, count: 1, more: false, line: b"left" }, Sent::None, 1, true)
            },
            Step { field: Some((3, b"left")), ..row(Change(3), Sent::None, 1, true) },
            Step { field: Some((3, b"left")), ..row(Save, Sent::Edit(3, b"left"), 1, true) },
            Step { refusal: Some((3, "long")), ..row(Long(None), Sent::Read(false), 1, false) },
            row(Leave, Sent::None, 0, false),
            row(LateRead, Sent::None, 0, false),
            row(Enter, Sent::Read(false), 0, false),
            row(Notes { first: 1, count: 0, more: false, line: b"" }, Sent::None, 0, true),
        ]),
        Box::new([
            row(Enter, Sent::None, 0, false),
            row(Me, Sent::Read(false), 0, false),
            row(Cut, Sent::None, 0, false),
        ]),
    ];
    for (case, steps) in cases.into_iter().enumerate() {
        let limits = fixtures::limits();
        let mut root = crate::unlinked(&limits, fixtures::SEED);
        let watch = fixtures::start(&mut root, List::uncapped());
        let mut party = None;
        let mut page = Pages::new();
        let mut env = fixtures::env(&limits);
        let mut read = None;
        let mut key = None;
        let mut known = List::<c::Note>::uncapped();
        let mut reading = List::<c::Note>::uncapped();
        for (step, expected) in steps.iter().enumerate() {
            let mut out = Queue::uncapped();
            let event = match expected.input {
                Enter => pages::Event::Went { address: Address::Remembered },
                Me => {
                    deliver(&mut root, watch, fixtures::frame(fixtures::PARTY));
                    party = Some(fixtures::PARTY);
                    pages::Event::Told { told: Told::Changed { handle: pages::me_handle() } }
                }
                Cut => pages::Event::Told {
                    told: Told::Page {
                        handle: read.unwrap(),
                        page: Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(c::NotePage {
                            cut: true,
                            rows: List::uncapped(),
                            before: None,
                            after: None,
                        })))),
                    },
                },
                Leave => {
                    known.clear();
                    reading.clear();
                    pages::Event::Went { address: Address::Accounts }
                }
                Notes { first, count, more, line } => {
                    let mut rows = List::uncapped();
                    for number in first..first.checked_add(count).unwrap() {
                        let row = note(number, line);
                        reading.push(row.clone()).unwrap();
                        rows.push(row).unwrap();
                    }
                    if !more {
                        known = core::mem::replace(&mut reading, List::uncapped());
                    }
                    pages::Event::Told {
                        told: Told::Page {
                            handle: read.unwrap(),
                            page: Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(c::NotePage {
                                cut: false,
                                rows,
                                before: None,
                                after: if more { Some(cursor()) } else { None },
                            })))),
                        },
                    }
                }
                Busy | ReadRefused | LateRead => pages::Event::Told {
                    told: Told::Page {
                        handle: read.unwrap(),
                        page: Readout::Jig(c::ReadResult::None(if let Busy = expected.input {
                            c::ReadNone::Busy
                        } else {
                            c::ReadNone::Refused
                        })),
                    },
                },
                Done | Refused | Moved | Long(_) => {
                    let answer = match expected.input {
                        Done => c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::NoteEdited { name: 1 })),
                        Moved => c::Answer::Refused {
                            refusal: c::Refusal::Moved { now: c::Revision { value: 50 }, what: c::Moved::Note },
                            at: None,
                        },
                        Long(bound) => c::Answer::Refused {
                            refusal: c::Refusal::Limit {
                                which: match bound {
                                    Some(limit) => c::Limit::Words { limit },
                                    None => c::Limit::Admission { which: c::AdmissionLimit::Words },
                                },
                            },
                            at: None,
                        },
                        Refused => c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None },
                        Enter
                        | Me
                        | Cut
                        | Leave
                        | Notes { .. }
                        | Change(_)
                        | Words(_)
                        | Save
                        | Cancel
                        | Delete(_)
                        | Busy
                        | ReadRefused
                        | Fire
                        | LateRead => unreachable!("an act answer"),
                    };
                    pages::Event::Told {
                        told: Told::Answered { key: key.unwrap(), outcome: Outcome::Answered(Answered::Jig(answer)) },
                    }
                }
                Fire => {
                    env.now = Time::ZERO.saturating_add(Duration::from_secs(1));
                    pages::fire(&mut page, &env, root.client(), &mut out);
                    pages::Event::Told { told: Told::Dropped { count: 0 } }
                }
                Change(note) => pages::Event::Act { act: pages::Act::Remembered(Act::Change { note }) },
                Words(words) => pages::Event::Act { act: pages::Act::Remembered(Act::Words { words: words.into() }) },
                Save => pages::Event::Act { act: pages::Act::Remembered(Act::Save) },
                Cancel => pages::Event::Act { act: pages::Act::Remembered(Act::Cancel) },
                Delete(note) => pages::Event::Act { act: pages::Act::Remembered(Act::Delete { note }) },
            };
            pages::step(&mut page, &env, root.client(), event, &mut out);
            let mut sent = List::uncapped();
            for request in &out {
                if let pages::Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(request) }) =
                    request
                    && let c::Read::Notes { .. } = request
                {
                    read = Some(*handle);
                    sent.push((Some(request.clone()), None)).unwrap();
                }
                if let pages::Request::Client(jig_client::Event::Send {
                    key: asked,
                    request: jig_client::Ask::Jig(request),
                    ..
                }) = request
                {
                    key = Some(*asked);
                    sent.push((None, Some(request.as_ref().clone()))).unwrap();
                }
            }
            let expected_sent: Box<[(Option<c::Read>, Option<c::Ask>)]> = match expected.sent {
                Sent::None => Box::new([]),
                Sent::Read(after) => Box::new([(
                    Some(c::Read::Notes {
                        scope: c::NoteScope::Party { party: fixtures::PARTY },
                        description: None,
                        after: if after { Some(cursor()) } else { None },
                    }),
                    None,
                )]),
                Sent::Edit(number, words) => Box::new([(
                    None,
                    Some(c::Ask::EditNote {
                        project: 0,
                        note: number,
                        scope: c::NoteScope::Party { party: fixtures::PARTY },
                        revision: note(number, b"").revision,
                        description: text(words),
                        body: text(b""),
                        references: note(number, b"").tasks,
                    }),
                )]),
                Sent::Delete(number) => Box::new([(
                    None,
                    Some(c::Ask::DeleteNote {
                        project: 0,
                        note: number,
                        scope: c::NoteScope::Party { party: fixtures::PARTY },
                        revision: note(number, b"").revision,
                    }),
                )]),
            };
            assert_eq!(sent.as_slice(), expected_sent.as_ref(), "case {case} step {step}: requests");
            let Some(remembered) = page.remembered() else {
                assert_eq!(page.address(), &Address::Accounts);
                continue;
            };
            assert_eq!(remembered.party(), party);
            assert_eq!(remembered.answered(), expected.answered, "case {case} step {step}: answered");
            assert_eq!(remembered.field(), expected.field, "case {case} step {step}: field");
            assert_eq!(
                match remembered.refusal() {
                    Some((note, why)) => Some((
                        note,
                        match why {
                            Refusal::Moved => "moved",
                            Refusal::TooLong { bound } => {
                                assert_eq!(
                                    *bound,
                                    if let Long(Some(bound)) = expected.input {
                                        bound
                                    } else if let Long(None) = expected.input {
                                        120
                                    } else {
                                        64
                                    }
                                );
                                "long"
                            }
                            Refusal::Other(why) => {
                                assert_eq!(*why, c::Refusal::NotOffered);
                                "other"
                            }
                        }
                    )),
                    None => None,
                },
                expected.refusal,
                "case {case} step {step}: refusal"
            );
            assert_eq!(remembered.rows().count(), expected.count, "case {case} step {step}: rows");
            for (index, (row, source)) in remembered.rows().zip(&known).enumerate() {
                let place = u32::try_from(index.checked_add(1).unwrap()).unwrap();
                let wanted = match source.writer {
                    c::NoteAuthor::Task { .. } => Row::Task {
                        place,
                        note: source.number,
                        line: source.description.bytes(),
                        chat: Some(14),
                        changed: source.changed,
                    },
                    c::NoteAuthor::Party { party } => Row::Party {
                        place,
                        note: source.number,
                        line: source.description.bytes(),
                        party,
                        changed: source.changed,
                    },
                };
                assert_eq!(row, wanted, "case {case} step {step}: row");
            }
        }
    }
}
