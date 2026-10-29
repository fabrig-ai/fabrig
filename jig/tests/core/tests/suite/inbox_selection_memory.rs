//! Independent allocator accounting of the whole recursive selection payload.
use jig_domain_core::documents::{Cursor, InboxObject, InboxSelection, inbox_selection_bound};
use skein_lib::List;
use skein_world::domain::heap::Meter;

// Peak 1,200 bytes measured 10-29; testing-strategy.md 6.
const INBOX_SELECTION_PEAK_HEAP_CEILING: u64 = 1_500;

#[test]
fn inbox_selection_allocations_fit_the_whole_recursive_bound_including_empty_reserved_roots() {
    let rows = 7;
    let depth = 9;
    let bound = inbox_selection_bound(rows, depth).expect("small recursive context bound");
    let meter = Meter::new();
    meter.start();
    let mut contexts = List::with_capacity(rows);
    for number in 0..u64::from(rows) {
        let mut goals = List::with_capacity(depth + 1);
        if number != 0 {
            for root in 0..=depth {
                goals.push(u64::MAX - u64::from(root)).expect("reserved root capacity");
            }
        }
        contexts
            .push(InboxSelection {
                cursor: Cursor::Inbox {
                    task: u64::MAX - number,
                    object: InboxObject::Result { position: number },
                    at: None,
                },
                goals,
            })
            .expect("reserved context capacity");
    }
    let measured = meter.end();
    // The whole selection peak stays under its measured ceiling; retained
    // storage at the step's end stays within the full recursive formula.
    meter.check(measured, INBOX_SELECTION_PEAK_HEAP_CEILING, &(rows, depth));
    assert!(meter.held() <= bound, "held selection storage within the recursive formula");
    assert_eq!(contexts.len(), rows);
    assert!(contexts.get(0).expect("first reserved context").goals.is_empty());
    assert_eq!(contexts.get(0).expect("first reserved context").goals.capacity(), depth + 1);
    for context in contexts.iter().skip(1) {
        assert_eq!(context.goals.len(), depth + 1);
    }
    drop(contexts);
    assert_eq!(meter.held(), 0);
}
