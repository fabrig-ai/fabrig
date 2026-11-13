use super as fixtures;
use jig_client::contract as c;

pub const CHANGE: u64 = 2;

pub const TITLE: &[u8] = b"Export widget's table as CSV";

pub fn text(bytes: &[u8]) -> c::Text {
    c::Text::new(Box::from(bytes), 256).expect("fixture text")
}

pub fn document(bytes: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(text(bytes))
}

pub fn change() -> c::Task {
    let mut task = fixtures::task::document();
    task.chip.task = CHANGE;
    task.chip.title = document(TITLE);
    task.chip.phase = c::Phase::Running;
    task.hold = None;
    task.result = None;
    task.result_contract = c::TaskContractDocument::Change { connector: fixtures::FORGE, kind: 1, words: 256 };
    let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
    details.ending = None;
    task
}
