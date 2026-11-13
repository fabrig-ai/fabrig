//! Chat closing confirmation render adapter.
use super as fixtures;
pub use temper_client_view_kit::chat::*;
use jig_client::contract as c;
use skein_lib::List;
use skein_ui::Builder;
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
        &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
    );
    b.finish()
}
