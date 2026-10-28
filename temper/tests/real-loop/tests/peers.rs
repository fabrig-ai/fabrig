//! The fake peer's listener comes from the kernel, not a fixture port.
use skein_world::domain::heap::Counting;
#[global_allocator]
static HEAP: Counting = Counting;

#[test]
fn the_scripted_peer_listens_on_a_kernel_assigned_port() {
    use temper_real_loop_world::{llm, scripts};
    let bound = llm::bound(|| llm::scripted(scripts::chat_scripts(1)));
    assert!(bound.ip().is_loopback() && bound.port() != 0, "{bound}");
}
