//! The application's typed ask and snapshot (temper/docs/design/domain/forge/reds.md, 5.4).
use crate::{Domain, RunRed, reds_for_run};
use alloc::boxed::Box;
use skein_lib::Token;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ask {
    Reds,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Answer {
    Reds { reds: Box<[RunRed]> },
}

/// Keep the wide core event's construction out of the root entrance's frame.
pub(crate) fn asked(
    domain: &mut Domain,
    channel: Token,
    run: Token,
    attempt: Token,
    number: u64,
    answer: jig_domain_link::AskAnswer,
) {
    let event = jig_domain_core::Event::Fleet(jig_domain_fleet::Event::Asked { channel, run, attempt, number, answer });
    domain.work.push(crate::boundary::Work::Core(event));
}

/// Read an admitted ask without deciding or recording anything.
#[must_use]
pub fn answer(domain: &Domain, task: u64, attempt: u64, ask: Ask) -> Answer {
    match ask {
        Ask::Reds => Answer::Reds { reds: reds_for_run(domain, task, attempt) },
    }
}
