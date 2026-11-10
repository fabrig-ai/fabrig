//! Shared counters preserve a step's highs, handoff and signed release (testing/README.md, 5).

use core::hint::black_box;

use crate::{Counting, Grown, Meter, Span};

#[global_allocator]
static HEAP: Counting = Counting;

#[test]
fn shared_counters_keep_handoffs_and_signed_spans_exact() {
    let meter = Meter::new();
    meter.start();
    let handed = black_box(Box::new([0_u8; 64]));
    drop(black_box(Box::new([0_u8; 128])));
    let held = black_box(Box::new([0_u8; 16]));
    let measured = meter.end();
    assert_eq!((measured.peak(), measured.held()), (192, 80));
    drop(handed);
    assert_eq!(meter.check(measured, 128, &"a step's own heap"), 128);
    assert_eq!(meter.held(), 16);
    let span = Span::start();
    drop(held);
    assert_eq!(span.end(), Grown { peak: 0, net: -16 });
    assert_eq!(meter.held(), 0);
}
