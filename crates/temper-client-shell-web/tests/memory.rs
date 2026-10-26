//! Patch emission measured at the browser's configured bounds.
use skein_lib::{Time, Wall};
use skein_ui::Patch;
use skein_world::domain::heap::{Counting, Meter};
use temper_client_shell_web::host;

#[global_allocator]
static HEAP: Counting = Counting;

#[test]
fn a_turn_reserves_no_patch_queue() {
    let mut host = host::Host::new(11, Time::ZERO, Wall::EPOCH, temper_client_view::sign_in::Ways::NONE);
    host.start(b"/chats", Vec::new(), None);
    let bound = u64::from(host.bounds.tree.patches) * u64::try_from(size_of::<Patch>()).expect("patch size fits");
    let meter = Meter::new();
    meter.start();
    let patches = host.patches();
    let peak = meter.end().peak();
    assert!(!patches.is_empty(), "the turn emits patches");
    assert_eq!(patches.capacity(), patches.len(), "delivery reserves only its emitted length");
    assert!(peak < bound, "turn reservation: {peak} < {bound}");
}
