//! A span measures the heap's growth around one call, signed: at its peak,
//! and in all, which a call that frees what it did not allocate takes below
//! zero.

use std::hint::black_box;

use skein_heap::{Grown, Span};

#[test]
fn a_span_counts_the_allocations_its_thread_made() {
    let mut before = black_box(vec![0_u8; 8]);
    let span = Span::start();
    assert_eq!(span.made(), 0, "the vector predates the span");
    before.reserve_exact(before.capacity().checked_add(1).expect("small capacity"));
    assert_eq!(span.made(), 1, "a reallocation counts once");
    drop(black_box(Box::new(7_u8)));
    assert_eq!(span.made(), 2, "a freed allocation still counts");
    drop(before);
    assert_eq!(span.made(), 2, "a free adds no allocation");
}

#[test]
fn a_span_measures_the_peak_and_the_net_growth_of_a_call() {
    let span = Span::start();
    let kept = black_box(vec![0_u8; 100]);
    drop(black_box(vec![0_u8; 1000]));
    assert_eq!(span.end(), Grown { peak: 1100, net: 100 });
    drop(kept);
}

#[test]
fn a_call_that_frees_what_it_did_not_allocate_shrinks_the_heap() {
    let before = black_box(vec![0_u8; 500]);
    let span = Span::start();
    drop(before);
    let kept = black_box(vec![0_u8; 200]);
    assert_eq!(span.end(), Grown { peak: 0, net: -300 }, "never above where it began");
    drop(kept);
}
