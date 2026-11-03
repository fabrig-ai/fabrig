//! The bridge's messages and malformed buffers (playground.md, 5).
use crate::{
    bridge::{Clock, Input, Malformed, Printed, READY, Speed, Stamp, Start, Stream, ToPage, ToWorld},
    fetch::{Exchange, Report},
    transport::StreamDuty,
};
use skein_lib::{Duration, Time, Wall};

// Enough empty pairs to exercise a list whose slots outweigh its byte strings.
const EMPTY_HEADERS: usize = 64;
// A Clock has three u64 fields before its idle and next-due tags.
const CLOCK_FIELDS: usize = 24;

fn to_world() -> Vec<ToWorld> {
    vec![
        ToWorld::Start(Start {
            story: b"story\0\xff".as_slice().into(),
            seed: u64::MAX,
            start: Wall::from_nanos(u64::MAX),
        }),
        ToWorld::Start(Start { story: Box::from([]), seed: 0, start: Wall::EPOCH }),
        ToWorld::Input(Input::Exchange(Exchange {
            id: u32::MAX,
            path: b"/api/notes".as_slice().into(),
            method: "POST".into(),
            headers: vec![("x-name".into(), "value".into()), ("empty".into(), String::new())],
            body: b"body\0\xff".as_slice().into(),
            bound: u32::MAX,
        })),
        ToWorld::Input(Input::Exchange(Exchange {
            id: 0,
            path: Box::from([]),
            method: "GET".into(),
            headers: Vec::new(),
            body: Box::from([]),
            bound: 0,
        })),
        ToWorld::Input(Input::Exchange(Exchange {
            id: 1,
            path: Box::from([]),
            method: "méthode".into(),
            headers: vec![(String::new(), String::new()); EMPTY_HEADERS],
            body: Box::from([]),
            bound: 1,
        })),
        ToWorld::Input(Input::Abandon { exchange: u32::MAX }),
        ToWorld::Input(Input::Stream(StreamDuty::Open {
            path: b"/events".as_slice().into(),
            headers: vec![
                (b"x-name".as_slice().into(), b"value".as_slice().into()),
                (Box::from([]), b"\xff".as_slice().into()),
            ],
            bound: u32::MAX,
        })),
        ToWorld::Input(Input::Stream(StreamDuty::Open { path: Box::from([]), headers: Vec::new(), bound: 0 })),
        ToWorld::Input(Input::Stream(StreamDuty::Close)),
        ToWorld::Input(Input::ReadOn),
        ToWorld::Pace(Speed::PAUSED),
        ToWorld::Pace(Speed::ONE),
        ToWorld::Pace(Speed(u32::MAX)),
    ]
}

fn clock(next_due: Option<Time>, idle: bool) -> Clock {
    Clock {
        step: u64::MAX,
        now: Time::from_nanos(u64::MAX),
        wall: Wall::from_nanos(u64::MAX),
        idle,
        next_due,
        memory: u64::MAX,
        slice: Duration::from_nanos(u64::MAX),
    }
}

fn to_page() -> Vec<ToPage> {
    let mut messages = vec![
        ToPage::Report(Report::Answered {
            exchange: u32::MAX,
            status: u16::MAX,
            body: b"answer\0\xff".as_slice().into(),
        }),
        ToPage::Report(Report::Answered { exchange: 0, status: 0, body: Box::from([]) }),
        ToPage::Report(Report::Failed { exchange: u32::MAX }),
        ToPage::Report(Report::StreamOpened),
        ToPage::Report(Report::StreamRefused { status: u16::MAX }),
        ToPage::Report(Report::StreamPiece { bytes: b"piece\0\xff".as_slice().into() }),
        ToPage::Report(Report::StreamPiece { bytes: Box::from([]) }),
        ToPage::Report(Report::StreamEnded),
        ToPage::Stamp(Stamp { step: u64::MAX, instant: Time::from_nanos(u64::MAX) }),
        ToPage::Clock(clock(Some(Time::from_nanos(u64::MAX)), true)),
        ToPage::Clock(clock(None, false)),
        ToPage::Clock(Clock {
            step: 0,
            now: Time::ZERO,
            wall: Wall::EPOCH,
            idle: true,
            next_due: Some(Time::ZERO),
            memory: 0,
            slice: Duration::ZERO,
        }),
        ToPage::Stopped,
        ToPage::Failed { words: Box::from([]) },
        ToPage::Failed { words: b"failed\0\xff".as_slice().into() },
    ];
    for stream in [Stream::Output, Stream::Error] {
        messages.push(ToPage::Printed(Printed {
            process: b"process\0\xff".as_slice().into(),
            stream,
            line: b"line\0\xff".as_slice().into(),
        }));
        messages.push(ToPage::Printed(Printed { process: Box::from([]), stream, line: Box::from([]) }));
    }
    messages
}

#[test]
fn every_message_to_the_world_round_trips() {
    for message in to_world() {
        assert_eq!(ToWorld::decode(&message.encode()), Ok(message));
    }
}

#[test]
fn every_message_to_the_page_round_trips() {
    for message in to_page() {
        assert_eq!(ToPage::decode(&message.encode()), Ok(message));
    }
}

#[test]
fn no_message_encodes_as_ready() {
    for bytes in to_world().iter().map(ToWorld::encode).chain(to_page().iter().map(ToPage::encode)) {
        assert_ne!(bytes.as_ref(), READY);
    }
}

fn malformed<T: std::fmt::Debug>(messages: impl Iterator<Item = Box<[u8]>>, decode: fn(&[u8]) -> Result<T, Malformed>) {
    for message in messages {
        for end in 0..message.len() {
            let prefix = message.get(..end).expect("proper prefix");
            assert!(decode(prefix).is_err(), "prefix {end} of {message:?}");
        }
        let mut long = message.into_vec();
        long.push(0);
        assert!(decode(&long).is_err(), "trailing byte in {long:?}");
    }
}

#[test]
fn a_short_or_long_message_is_malformed() {
    malformed(to_world().iter().map(ToWorld::encode), ToWorld::decode);
    malformed(to_page().iter().map(ToPage::encode), ToPage::decode);
    // The first byte string's declared length exceeds the whole short buffer.
    let world_bad: &[&[u8]] = &[
        &[255],
        &[1, 255],
        &[1, 2, 255],
        &[0, 1, 0, 0, 0],
        &[0, 255, 255, 255, 255],
        // Empty path and method, then a header count of u32::MAX.
        &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255],
        // Empty stream path, then a header count of u32::MAX.
        &[1, 2, 0, 0, 0, 0, 0, 255, 255, 255, 255],
        // A method must be UTF-8.
        &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 255],
    ];
    for bytes in world_bad {
        assert_eq!(ToWorld::decode(bytes), Err(Malformed), "{bytes:?}");
    }
    for bytes in [
        &[255][..],
        &[0, 255],
        &[5, 1, 0, 0, 0],
        &[5, 255, 255, 255, 255],
        // Empty process name followed by an unknown printed stream.
        &[1, 0, 0, 0, 0, 255],
    ] {
        assert_eq!(ToPage::decode(bytes), Err(Malformed), "{bytes:?}");
    }
    for tags in [&[255][..], &[0, 255]] {
        let mut bytes = vec![3];
        bytes.extend_from_slice(&[0; CLOCK_FIELDS]);
        bytes.extend_from_slice(tags);
        assert_eq!(ToPage::decode(&bytes), Err(Malformed), "{bytes:?}");
    }
}

#[test]
fn integers_and_tags_have_their_fixed_wire_layout() {
    assert_eq!(ToWorld::Pace(Speed::ONE).encode().as_ref(), &[2, 232, 3, 0, 0]);
    assert_eq!(ToPage::Stopped.encode().as_ref(), &[4]);
    assert_eq!(ToPage::Failed { words: Box::from([]) }.encode().as_ref(), &[5, 0, 0, 0, 0]);
    assert_eq!(ToWorld::Input(Input::ReadOn).encode().as_ref(), &[1, 3]);
}
