//! The application's typed asks and answers (temper/docs/design/domain/forge/reds.md, 5.4 and 5.6;
//! temper/docs/design/domain/worker/checkouts.md, A landed dependency's catch-up).
use crate::route::forge_route::landed_for_run;
use crate::{Domain, RunRed, reds_for_run};
use alloc::boxed::Box;
use skein_lib::Token;

/// A check's report (temper/docs/design/domain/forge/reds.md, 5.6).
#[derive(Hash, PartialEq, Eq, Debug)]
pub struct CheckResults {
    pub tag: u64,
    pub check: u64,
    pub tree: Box<[u8]>,
    pub base: Option<Box<[u8]>>,
    pub verdict: Verdict,
    pub passed: Box<[Box<[u8]>]>,
    pub failed: Box<[Box<[u8]>]>,
    pub flakes: Box<[Flake]>,
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct Flake {
    pub key: Box<[u8]>,
    pub log: Box<[u8]>,
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum Verdict {
    Passed,
    Refused,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Ask {
    Reds,
    Results(CheckResults),
    Admit { tag: u64, branch: Box<[u8]>, expected: Option<[u8; 32]>, commit: [u8; 32] },
    Landed { head: [u8; 32] },
}

#[derive(PartialEq, Eq, Debug)]
pub enum Answer {
    Kept,
    Landed,
    Unlanded,
    Unadmitted { words: Box<[u8]> },
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
        Ask::Results(_) => Answer::Kept,
        Ask::Admit { .. } => Answer::Unadmitted { words: b"the engine admits no pushes yet".as_slice().into() },
        Ask::Landed { head } => {
            if landed_for_run(domain, task, attempt, &head) {
                Answer::Landed
            } else {
                Answer::Unlanded
            }
        }
    }
}
