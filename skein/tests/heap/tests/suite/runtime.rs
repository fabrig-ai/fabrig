//! The runtime's own heap is never a step's (skein-heap's "The runtime's own").

use core::cell::{Cell, LazyCell};
use skein_heap::{Meter, Span};

std::thread_local! {
    static FIRST: Cell<Option<Box<u8>>> = const { Cell::new(None) };
    static SECOND: Cell<Option<Box<u8>>> = const { Cell::new(None) };
}

#[test]
fn a_threads_first_destructor_record_is_neither_a_spans_nor_a_steps() {
    let span = Span::start();
    FIRST.with(|first| first.set(None));
    let grown = span.end();
    assert_eq!((grown.peak, grown.net), (0, 0), "a span over the thread's first thread-local with a destructor");
    let meter = Meter::new();
    meter.start();
    SECOND.with(|second| second.set(None));
    let step = meter.end();
    assert_eq!((step.peak(), step.held()), (0, 0), "a step over another");
}

#[test]
fn a_worlds_lazy_storage_is_charged_only_when_first_reached_inside_a_span() {
    let cold = LazyCell::new(|| Box::new([0_u8; 64]));
    let span = Span::start();
    std::hint::black_box(LazyCell::force(&cold));
    let grown = span.end();
    assert_eq!((grown.peak, grown.net), (64, 64), "first reach inside the call belongs to it");

    let warm = LazyCell::new(|| Box::new([0_u8; 64]));
    std::hint::black_box(LazyCell::force(&warm));
    let span = Span::start();
    std::hint::black_box(LazyCell::force(&warm));
    let grown = span.end();
    assert_eq!((grown.peak, grown.net), (0, 0), "world storage reached before its base is not the call's");
}

#[test]
fn the_first_write_to_standard_error_is_neither_a_spans_nor_a_steps() {
    let span = Span::start();
    eprintln!("a span's first line to the standard error");
    let grown = span.end();
    assert_eq!((grown.peak, grown.net), (0, 0), "a span over the first write to the standard error");
    let meter = Meter::new();
    meter.start();
    eprintln!("a step's line to the standard error");
    let step = meter.end();
    assert_eq!((step.peak(), step.held()), (0, 0), "a step over another");
}
