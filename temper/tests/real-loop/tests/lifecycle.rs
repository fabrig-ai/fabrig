//! Standalone startup and termination share the real kernel and checked ownership.
use skein_world::domain::heap::Counting;
#[global_allocator]
static HEAP: Counting = Counting;
#[test]
fn a_started_service_drains_on_termination() {
    use temper_real_loop_world::{End, run, scripts};
    let ran = run(|| scripts::chat_scripts(1), Vec::new);
    assert_eq!(ran.served(), Some(&End::Drained), "link {:?}", ran.link());
}
