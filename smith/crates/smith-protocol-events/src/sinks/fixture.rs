//! Tiny destination capacities and a deterministic IO peer for projection tests.

#![expect(
    clippy::disallowed_types,
    reason = "programming-model.md, section 10.2: deterministic IO peer uses ordinary Rust"
)]

use super::{Capture, Duration, Queue, Record, SinkLimits, Sinks, Time, Token, io, largest_line, stream};
use alloc::vec::Vec;

pub const DEADLINE: Duration = Duration::from_secs(30);

#[must_use]
pub fn small() -> SinkLimits {
    let records = smith_events::Limits { string: 64, content: 1024, items: 19 };
    let line = largest_line(&records, Capture::None, records.content, records.string)
        .expect("bounded fixture input and complete destination output");
    SinkLimits {
        destinations: 1,
        output: line.checked_mul(6).expect("bounded fixture input and complete destination output"),
        records,
        answer_bytes: records.content,
        metadata_string: records.string,
        metadata_items: records.items,
        scalar_content: records.content,
        observation_line: 0,
        fact_extra: 0,
        conversations: 1,
    }
}

/// Destination count override on the small projection preset.
#[must_use]
pub fn limits(destinations: u32) -> SinkLimits {
    SinkLimits { destinations, ..small() }
}

pub fn grant(sinks: &mut Sinks, token: Token) {
    sinks.up(Time::ZERO, io::Event::Stream { owner: token, up: stream::Up::Room });
}

pub fn drain(sinks: &mut Sinks, token: Token) -> Vec<Record> {
    drain_until(sinks, token, true)
}

pub fn drain_ready(sinks: &mut Sinks, token: Token) -> Vec<Record> {
    drain_until(sinks, token, false)
}

fn drain_until(sinks: &mut Sinks, token: Token, ended: bool) -> Vec<Record> {
    let mut out = Queue::with_capacity(2);
    let mut read = Vec::new();
    let mut reader = smith_events::Reader::with_ceiling(&sinks.limits.records, sinks.line)
        .expect("bounded fixture input and complete destination output");
    for _ in 0_u32..10_000 {
        sinks.down(Time::ZERO, &mut out);
        match out.pop() {
            Some(io::Request::Stream { down: stream::Down::Demand { room, .. }, .. }) => {
                assert_eq!(room, sinks.io_output, "destination demands its configured piece");
                grant(sinks, token);
            }
            Some(io::Request::Stream { down: stream::Down::Send(bytes), .. }) => {
                assert!(
                    bytes.len() <= usize::try_from(sinks.io_output).expect("bounded fixture output"),
                    "output respects its piece limit"
                );
                let line = reader.feed(&bytes).expect("bounded fixture input and complete destination output");
                assert_eq!(line.consumed, bytes.len(), "the reader consumes one written piece");
                if let Some(record) = line.record {
                    read.push(record);
                }
            }
            Some(io::Request::Close { .. }) => {
                reader.finish().expect("bounded fixture input and complete destination output");
                sinks.up(Time::ZERO, io::Event::Closed { owner: token });
                return read;
            }
            None => {
                assert!(!ended, "writer failed to advance before close");
                reader.finish().expect("bounded fixture input and complete destination output");
                return read;
            }
            Some(other) => unreachable!("unexpected append request: {other:?}"),
        }
    }
    unreachable!("bounded writer did not close");
}
