use super::*;
use jig_domain_people as p;
#[path = "answers/outcomes.rs"]
mod outcomes;
#[path = "answers/refusals.rs"]
mod refusals;
pub use outcomes::*;
pub use refusals::*;
fn done(outcome: w::Outcome) -> w::Final {
    w::Final::Done(w::Done { outcome })
}
fn owner(value: w::DocumentOutcome) -> w::Final {
    done(w::Outcome::Document(
        w::OutcomeDocument { value },
    ))
}
pub fn started() -> w::Final {
    owner(w::DocumentOutcome::Said(
        w::OutcomeSaid { task: 8, message: 9 },
    ))
}
pub fn moved_hold() -> w::Final {
    let reason = w::HoldCategory::Dependency(
        w::HoldCategoryDependency { task: 9 },
    );
    w::Final::Refused(
        w::Refused { at: None, refusal: hold_refusal(reason) },
    )
}
pub const SERVICE_ANSWER: [u8; 25] = [1, 0, 0, 0, 0, 37, 0, 0, 0, 15, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0];
pub fn application_answer() -> w::KeyedAnswer {
    let owner = w::Owner::Family(w::Family { number: 9 });
    w::KeyedAnswer::FinalAnswer(
        w::Owned { owner, version: 2, record: Box::from(b"outcome".as_slice()) },
    )
}

#[path = "answers/admission.rs"]
mod admission;
#[path = "answers/lacks.rs"]
mod lacks;
pub use admission::*;
pub use lacks::*;

pub fn response_bounds() -> [(u32, Option<w::KeyedAnswer>); 2] {
    let limits = w::Limits { owned_record: 512, ..wire() };
    let owner = w::Owner::Family(w::Family { number: 9 });
    let answer = w::KeyedAnswer::FinalAnswer(
        w::Owned { owner, version: 2, record: Box::new([3; 257]) },
    );
    [(512, Some(answer)), (1, None)]
}
