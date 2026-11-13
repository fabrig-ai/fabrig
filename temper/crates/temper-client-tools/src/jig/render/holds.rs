//! Supplied hold facts in the same words for task reads and change panels.
use super::{Parts, Text, document};
use alloc::boxed::Box;
use jig_client::{contract as c, words::Word};
use skein_ui::phrases::Hole;

pub(crate) fn summary(task: &c::Task, vocabulary: &dyn Parts) -> Box<[u8]> {
    let supplied = match task.hold.as_ref() {
        Some(hold) => hold.words.is_some(),
        None => false,
    };
    if budget(task.hold.as_deref()) && !supplied {
        return skein_lib::bytes::copy_of(b"Insufficient activation budget");
    }
    let mut measure = Text::measure();
    reason(task, vocabulary, &mut measure);
    let mut out = Text::writing_exact(measure);
    reason(task, vocabulary, &mut out);
    out.finish()
}

pub(crate) fn text(task: &c::Task, vocabulary: &dyn Parts, out: &mut Text) {
    if task.hold.is_none() {
        return;
    }
    out.put(b"held: ");
    reason(task, vocabulary, out);
    out.put(b"\nway out: ");
    way_out(task, vocabulary, out);
    out.put(b"\n");
}

pub(crate) fn way_out(task: &c::Task, vocabulary: &dyn Parts, out: &mut Text) {
    let uncertain = task.hold.as_ref().expect("supplied hold").uncertain();
    if uncertain {
        out.put(b"check whether the effect happened before deciding to repeat it; ");
    } else {
        out.put(recovery(task.hold.as_ref().expect("supplied hold"), vocabulary));
    }
    out.put(b"task release ");
    out.number(task.chip.task);
    out.put(b" --hold ");
    out.number(task.hold.as_ref().expect("supplied hold").revision().value);
    if uncertain {
        out.put(b" --again (only to repeat the effect)");
    }
    out.put(b"; or cancel it: task cancel ");
    out.number(task.chip.task);
    out.put(b" REASON");
}

fn recovery(hold: &c::ChipHold, vocabulary: &dyn Parts) -> &'static [u8] {
    match hold.reason {
        c::HoldCategory::Exceeded(value) if value.bound == c::Bound::Items => {
            b"reduce calls per response or raise agent.json's calls_per_response, then "
        }
        c::HoldCategory::Exceeded(_) => b"reduce the request or raise its configured allowance, then ",
        c::HoldCategory::Budget => b"",
        c::HoldCategory::Deadline => b"amend the task's deadline, then ",
        c::HoldCategory::Effects | c::HoldCategory::EffectFailed => b"review and resolve the effects, then ",
        c::HoldCategory::Failures { .. } => vocabulary.failures_recovery(),
        c::HoldCategory::Stalled
        | c::HoldCategory::Dependency { .. }
        | c::HoldCategory::Stopped
        | c::HoldCategory::Placed { .. }
        | c::HoldCategory::Drift
        | c::HoldCategory::Uncertain { .. }
        | c::HoldCategory::HoldsWaited
        | c::HoldCategory::Circle
        | c::HoldCategory::ResourceUnavailable
        | c::HoldCategory::ResourceAccess
        | c::HoldCategory::PoolLost { .. }
        | c::HoldCategory::Procedure => b"resolve the reason, then ",
    }
}

fn budget(hold: Option<&c::ChipHold>) -> bool {
    if let Some(hold) = hold {
        return hold.reason == c::HoldCategory::Budget;
    }
    false
}

fn budget_reason(task: &c::Task, vocabulary: &dyn Parts, out: &mut Text) {
    let book = vocabulary.phrasebook(4096);
    let said = book
        .say(0, Word::hold(&c::HoldCategory::Budget).id(), &[Hole::Number(task.chip.task)])
        .expect("budget hold phrase");
    out.put(&said.text);
}

fn reason(task: &c::Task, vocabulary: &dyn Parts, out: &mut Text) {
    match task.hold.as_ref() {
        None => out.put(b"reason unavailable"),
        Some(hold) => {
            if let Some(words) = &hold.words {
                out.put(words.bytes());
                if words.cut {
                    out.put(b"...");
                }
            } else if budget(task.hold.as_deref()) {
                budget_reason(task, vocabulary, out);
            } else {
                owner(&hold.reason, out);
            }
            if let Some(source) = &hold.source {
                out.put(b"; source: ");
                vocabulary.owner_part(source, out);
            }
        }
    }
}

fn owner(reason: &c::HoldCategory, out: &mut Text) {
    let number = match reason {
        c::HoldCategory::Exceeded(value) if value.bound != c::Bound::History => Some(value.value),
        c::HoldCategory::Dependency { task } => Some(*task),
        c::HoldCategory::Uncertain { entry } => Some(*entry),
        c::HoldCategory::PoolLost { pool } => Some(*pool),
        c::HoldCategory::Exceeded(_)
        | c::HoldCategory::Failures { .. }
        | c::HoldCategory::Stalled
        | c::HoldCategory::Stopped
        | c::HoldCategory::Placed { .. }
        | c::HoldCategory::Drift
        | c::HoldCategory::Effects
        | c::HoldCategory::EffectFailed
        | c::HoldCategory::Budget
        | c::HoldCategory::Deadline
        | c::HoldCategory::HoldsWaited
        | c::HoldCategory::Circle
        | c::HoldCategory::ResourceUnavailable
        | c::HoldCategory::ResourceAccess
        | c::HoldCategory::Procedure => None,
    };
    let holes: Box<[_]> = match number {
        Some(n) => Box::new([Hole::Number(n)]),
        None => Box::new([]),
    };
    let book = document::phrasebook(4096);
    let said = book.say(0, Word::hold(reason).id(), &holes).expect("hold phrase");
    out.put(said.text.strip_prefix(b"Held: ").unwrap_or(&said.text));
    match reason {
        c::HoldCategory::Placed { hold } => {
            out.put(b" ");
            out.number(*hold);
        }
        c::HoldCategory::Failures { class } => {
            out.put(b": ");
            out.put(match class {
                c::HoldFailure::Transient => b"transient failures",
                c::HoldFailure::Permanent => b"permanent failures",
                c::HoldFailure::Run => b"run failures",
                c::HoldFailure::Agent => b"agent failures",
                c::HoldFailure::Lost => b"lost runs",
                c::HoldFailure::Invalid => b"invalid results",
                c::HoldFailure::Infrastructure => b"infrastructure failures",
            });
        }
        c::HoldCategory::Exceeded(_)
        | c::HoldCategory::Stalled
        | c::HoldCategory::Dependency { .. }
        | c::HoldCategory::Stopped
        | c::HoldCategory::Drift
        | c::HoldCategory::Effects
        | c::HoldCategory::EffectFailed
        | c::HoldCategory::Uncertain { .. }
        | c::HoldCategory::Budget
        | c::HoldCategory::Deadline
        | c::HoldCategory::HoldsWaited
        | c::HoldCategory::Circle
        | c::HoldCategory::ResourceUnavailable
        | c::HoldCategory::ResourceAccess
        | c::HoldCategory::PoolLost { .. }
        | c::HoldCategory::Procedure => {}
    }
}
