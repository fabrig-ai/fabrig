//! Patch emission measured at the browser's configured bounds.
use skein_lib::{Time, Wall};
use skein_world::domain::heap::Meter;
use temper_client_shell_web::host;

#[test]
fn a_turn_reserves_no_patch_queue() {
    // Scenario peak 28,230 bytes plus 25%, rounded up to two significant digits.
    const PATCH_HEAP_CEILING: u64 = 36_000;
    let mut host = host::Host::new(11, Time::ZERO, Wall::EPOCH, temper_client_view::sign_in::Ways::NONE);
    host.start(b"/chats", Vec::new(), None);
    let meter = Meter::new();
    meter.start();
    let patches = host.patches();
    let peak = meter.end().peak();
    assert!(!patches.is_empty(), "the turn emits patches");
    assert_eq!(patches.capacity(), patches.len(), "delivery reserves only its emitted length");
    assert!(peak <= PATCH_HEAP_CEILING, "turn reservation: {peak} <= {PATCH_HEAP_CEILING}");
}
