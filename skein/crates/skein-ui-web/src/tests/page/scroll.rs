use std::cell::Cell;

use skein_ui::NodeId;

use super::{Event, Op, Scroll, Turn, measure, replay, to_end};

// A 100 px log with room to scroll below its 20 px viewport.
const HEIGHT: i32 = 100;
const VIEWPORT: i32 = 20;
const END: i32 = HEIGHT - VIEWPORT;
const LOG: NodeId = NodeId(1);
const AWAY_LOG: NodeId = NodeId(2);
const LINK: NodeId = NodeId(3);
const ONE_LAYOUT: usize = 1;

#[derive(Default)]
struct Log {
    top: Cell<i32>,
    height_reads: Cell<usize>,
}

impl Scroll for Log {
    fn top(&self) -> i32 {
        self.top.get()
    }

    fn height(&self) -> i32 {
        self.height_reads.set(self.height_reads.get() + 1);
        HEIGHT
    }

    fn viewport(&self) -> i32 {
        VIEWPORT
    }

    fn set_top(&self, top: i32) {
        self.top.set(top.clamp(0, END));
    }
}

#[test]
fn following_sets_the_end_without_reading_scroll_height() {
    let log = Log::default();
    to_end(&log);
    assert_eq!((log.top.get(), log.height_reads.get()), (END, 0));
}

#[test]
fn measurement_reports_the_current_scroll_geometry() {
    let log = Log { top: Cell::new(END), ..Log::default() };
    let Event::Scrolled { node, top, height, viewport } = measure(&log, LOG) else {
        panic!("measurement reports a scroll");
    };
    assert_eq!((node, top, height, viewport), (LOG, END, HEIGHT, VIEWPORT));
}

#[test]
fn log_settlement_flushes_layout_once_after_the_last_write() {
    let href = Op::Href { node: LINK, path: b"/notes/1".as_slice().into() };
    let ops = [Op::ScrollEnd { node: LOG }, Op::PutView { node: AWAY_LOG }, Op::Measure { node: LOG }, href.clone()];
    // A patch before these operations has dirtied layout. The late href and
    // derived writes can dirty it again if movement flushes it too soon.
    let mut dirty = true;
    let mut layouts = 0;
    let mut settled = Vec::new();
    replay(&ops, |part| match part {
        Turn::Now(op) => dirty |= op == &href,
        Turn::FinishWrites => dirty = true,
        Turn::Follow(node) | Turn::Restore(node) | Turn::Measure(node) => {
            settled.push(node);
            layouts += usize::from(std::mem::replace(&mut dirty, false));
        }
    });
    assert_eq!((layouts, settled.as_slice()), (ONE_LAYOUT, [LOG, AWAY_LOG, LOG].as_slice()));
}
