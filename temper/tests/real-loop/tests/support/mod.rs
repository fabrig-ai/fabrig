//! Rail steps shared by the terminal stories (temper/docs/design/client/README.md, 5.3).
use temper_real_loop_world::{calls::Step, terminal};

pub fn start_change(drafts: bool) -> Vec<Step> {
    if drafts {
        // The terminal prints a menu's items inline, with no separate open act.
        vec![terminal::see(b"New draft or change"), terminal::type_on(b"Change", b"")]
    } else {
        vec![terminal::type_on(b"New change", b"")]
    }
}
