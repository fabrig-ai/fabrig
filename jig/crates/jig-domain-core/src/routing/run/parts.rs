//! Instructions and offered kinds chosen by run reasons (jig/docs/design/domain/tasks.md, 8.6;
//! jig/docs/design/domain/engine.md, 7.1).
use crate::Charter;
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::Writer;

/// The task's own instructions, or its charter's followed by the parts its reasons meet.
pub(crate) fn run_instructions(charter: &Charter, own: Option<&[u8]>, reasons: tasks::Reasons) -> Box<[u8]> {
    if let Some(own) = own {
        return own.into();
    }
    let mut length = charter.policy.instructions.len();
    for part in &charter.parts {
        if part.on.meets(reasons) {
            length = length
                .checked_add(2)
                .expect("checked charter joining blank line")
                .checked_add(part.instructions.len())
                .expect("checked charter instructions");
        }
    }
    let mut instructions = Writer::new(length);
    instructions.put(&charter.policy.instructions).expect("charter instructions counted");
    for part in &charter.parts {
        if part.on.meets(reasons) {
            instructions.put(b"\n\n").expect("joining blank line counted");
            instructions.put(&part.instructions).expect("part instructions counted");
        }
    }
    instructions.finish()
}

/// Kinds beside the always-offered: every kind, or the selected parts' `on` without `first`.
pub(crate) fn offered_kinds(charter: &Charter, own: bool, reasons: tasks::Reasons) -> tasks::Reasons {
    if own || charter.parts.is_empty() {
        return tasks::Reasons::ALL;
    }
    let mut offered = tasks::Reasons::EMPTY;
    for part in &charter.parts {
        if part.on.meets(reasons) {
            offered = offered.union(part.on);
        }
    }
    tasks::Reasons(offered.0 & !tasks::Reasons::FIRST.0)
}
