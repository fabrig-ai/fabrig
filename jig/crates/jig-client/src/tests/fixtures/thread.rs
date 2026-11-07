use crate::{
    contract::*,
    thread::{Limits, Thread},
};
use skein_lib::{List, Wall};
pub fn text(bytes: &[u8]) -> Text {
    Text::new(bytes.into(), 100).unwrap()
}
pub fn cursor(bytes: &[u8]) -> Cursor {
    Cursor::new(Box::from(bytes), 1024).expect("bounded opaque cursor")
}
pub fn row(position: u64) -> DecodedActivityLine {
    DecodedActivityLine {
        source: Box::new(ActivityLine {
            task: 7,
            position,
            at: Wall::EPOCH,
            line: Activity::RunBegan { attempt: 1, why: RunWhy::Made, resumed: false },
        }),
        decoded: None,
    }
}
pub fn page(positions: &[u64], older: Option<&[u8]>, newer: Option<&[u8]>) -> ActivityPage {
    let mut rows = List::with_capacity(10);
    for position in positions {
        rows.push(row(*position)).unwrap();
    }
    ActivityPage {
        cut: false,
        rows,
        before: match older {
            Some(bytes) => {
                let next = cursor(bytes);
                Some(next)
            }
            None => None,
        },
        after: match newer {
            Some(bytes) => {
                let next = cursor(bytes);
                Some(next)
            }
            None => None,
        },
    }
}
pub fn positions(c: &Thread) -> Box<[u64]> {
    let mut positions = List::with_capacity(c.lines.rows.capacity());
    for row in &c.lines.rows {
        positions.push(row.source.position).unwrap();
    }
    positions.into_boxed()
}

pub fn model() -> Thread {
    Thread::new(super::SUBJECT_TASK, &Limits { lines: 4, bytes: 65536, text: 100, live: 100 })
}

pub fn sized_page(positions: &[u64], older: Option<&[u8]>, newer: Option<&[u8]>) -> ActivityPage {
    let mut page = page(positions, older, newer);
    for at in 0..page.rows.len() {
        let row = page.rows.get_mut(at).unwrap();
        let mut blocks = List::with_capacity(1);
        blocks.push(Block::Text(text(b"x"))).unwrap();
        let mut messages = List::with_capacity(1);
        messages.push(Message { speaker: Speaker::Assistant, blocks }).unwrap();
        row.source.task = super::SUBJECT_TASK;
        row.source.line = Activity::Turn {
            reply: row.source.position,
            attempt: 1,
            turn: u32::try_from(row.source.position).unwrap(),
            read: None,
        };
        row.decoded = Some(ArchivedTurn {
            task: super::SUBJECT_TASK,
            name: TurnName { attempt: 1, number: u32::try_from(row.source.position).unwrap() },
            cumulative: 0,
            read: None,
            at: Wall::EPOCH,
            turn: TurnRead::Decoded(Turn {
                messages,
                used: Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
                spent: 0,
                read_through: None,
                llm: None,
                cut: false,
            }),
        });
    }
    page
}
