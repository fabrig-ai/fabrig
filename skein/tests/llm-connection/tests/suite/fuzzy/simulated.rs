//! Fragmented simulated loopback runs with seeded short sends and receives
//! (skein/docs/design/testing/sweeps.md, 4.2 to 4.5).

use skein_fake_peers::Transport;
use skein_llm_connection_world::simulated::{assert_replay, run_with_input, scenario};
use skein_world::sweep::{cases, gate};
use skein_world::{Memory, this_test};
use std::convert::Infallible;

#[test]
fn scripted_peer_settles_under_short_socket_operations() {
    let test = this_test!("scripted_peer_settles_under_short_socket_operations");
    cases(test, 0..16, |seed| {
        scenario(seed, true);
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}

#[test]
fn batched_uploads_replay_under_actual_short_operations() {
    let test = this_test!("batched_uploads_replay_under_actual_short_operations");
    let at_gate = gate();
    let mut short_recv = false;
    let mut short_send = false;
    cases(test, 0..4, |seed| {
        let first = run_with_input(seed, true, Transport::Plaintext, false, Memory::Unchecked, 1500);
        let second = run_with_input(seed, true, Transport::Plaintext, false, Memory::Unchecked, 1500);
        assert_replay(&first, &second);
        if at_gate {
            short_recv |= first
                .trace
                .iter()
                .any(|entry| matches!(entry.event, skein_sim::Event::Fault(skein_sim::Fault::ShortRecv)));
            short_send |= first
                .trace
                .iter()
                .any(|entry| matches!(entry.event, skein_sim::Event::Fault(skein_sim::Fault::ShortSend)));
        }
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    if at_gate {
        assert!(short_recv && short_send, "configured short reads and writes actually fell");
    }
}
