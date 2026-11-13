//! Chat page render adapters.
use super as fixtures;
pub use temper_client_view_kit::chat::*;
use jig_client::{contract as c, thread::Thread};
use skein_ui::Builder;

pub fn chat_header() -> skein_ui::Tree {
    let mut b = Builder::new(&fixtures::tree_limits());
    let mut bindings = crate::bindings::Bindings::new(fixtures::tree_limits().bindings);
    crate::pages::chat::main(
        &mut b,
        &chat_root(),
        &fixtures::words(),
        &mut bindings,
        LINES,
        TEXT,
        &mut crate::words::Times::new(fixtures::NOW),
    );
    b.finish()
}
pub fn projected_tree(thread: &Thread) -> skein_ui::Tree {
    drawn_tree(thread, crate::pages::chat::Drawn::Chat { request: None, delegations: None })
}
pub fn requested_tree(thread: &Thread, request: &c::SpecificationDocument) -> skein_ui::Tree {
    drawn_tree(thread, crate::pages::chat::Drawn::Chat { request: Some(request), delegations: None })
}
pub fn tree(thread: &Thread) -> skein_ui::Tree {
    drawn_tree(thread, crate::pages::chat::Drawn::Lines)
}
fn drawn_tree(thread: &Thread, drawn: crate::pages::chat::Drawn<'_>) -> skein_ui::Tree {
    let mut b = Builder::new(&fixtures::tree_limits());
    crate::pages::chat::conversation(
        &mut b,
        thread,
        &fixtures::words(),
        (LINES, TEXT),
        crate::pages::chat::Speakers { person: None, requester: false },
        drawn,
        &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
    );
    b.finish()
}
