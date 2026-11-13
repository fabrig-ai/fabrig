//! Chat render adapters, which depend on the page and confirmation parts.
use super as fixtures;
pub use crate::view_kit::chat::*;
use jig_client::{contract as c, thread::Thread};
use skein_lib::List;
use skein_ui::Builder;

pub fn chat_header() -> skein_ui::Tree {
    let mut b = Builder::new(&fixtures::tree_limits());
    let mut bindings = crate::frame::Bindings::new(fixtures::tree_limits().bindings);
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
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
    );
    b.finish()
}
pub fn closing(grace: Option<skein_lib::Duration>) -> skein_ui::Tree {
    let mut operations = List::with_capacity(CLOSE_OPERATIONS);
    let reason = c::Text::new(Box::from(*b"Chat closed"), TEXT).expect("reason");
    operations
        .push(c::Operation::Cancel { project: fixtures::PROJECT, task: DELEGATE, reason: reason.clone(), grace: None })
        .expect("delegate cancel");
    operations
        .push(c::Operation::Move { project: fixtures::PROJECT, task: OTHER_DELEGATE, reason: reason.clone() })
        .expect("delegate move");
    operations
        .push(c::Operation::Cancel { project: fixtures::PROJECT, task: TASK, reason, grace })
        .expect("chat close");
    let mut b = Builder::new(&fixtures::tree_limits());
    let root = chat_root();
    let mut task = root.pages().chat().unwrap().task(root.client()).unwrap().clone();
    task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Running { attempt: 1 });
    crate::confirm::fates(
        &mut b,
        &task,
        operations.as_slice(),
        crate::words::Word::WhatClosingDoes,
        &fixtures::words(),
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
    );
    b.finish()
}
