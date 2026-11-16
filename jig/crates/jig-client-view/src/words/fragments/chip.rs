//! The chip sayings a `Words` method draws beyond its own text
//! (jig/docs/design/client/domain/view-crates.md, 3.1).
use crate::words::Words;
use crate::words::chip_words::ChipPhrase as P;
use crate::words::phrases::jig::JigPhrase;
use jig_client::contract::{ChipEnding, DocumentText, FailureClass, HoldFailure, Requester};
use skein_lib::Wall;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Run};

pub(crate) fn text(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    let saying =
        words.book().say(words.language(), phrase.id(), holes).expect("checked chip words and exact typed holes");
    b.text(Run::Plain, &saying.text);
}

pub(crate) fn number(b: &mut Builder, words: &Words, phrase: P, value: u64) {
    text(b, words, phrase, &[Hole::Number(value)]);
}

pub(crate) fn wall(b: &mut Builder, words: &Words, phrase: P, at: Wall) {
    text(b, words, phrase, &[Hole::Wall(words.adjusted_wall(at))]);
}

pub(crate) fn document(b: &mut Builder, value: &DocumentText, words: &Words) {
    crate::words::literal_segment::render(b, value.bytes());
    if value.cut {
        text(b, words, P::Cut, &[]);
    }
}

pub(crate) fn hold_failure(b: &mut Builder, value: HoldFailure, words: &Words) {
    let phrase = match value {
        HoldFailure::Transient => P::Transient,
        HoldFailure::Permanent => P::Permanent,
        HoldFailure::Run => P::RunFailure,
        HoldFailure::Agent => P::AgentFailure,
        HoldFailure::Lost => P::Lost,
        HoldFailure::Invalid => P::InvalidFailure,
        HoldFailure::Infrastructure => P::InfrastructureFailure,
    };
    text(b, words, phrase, &[]);
}

/// The departure caption and its supplied reason, author and time.
/// The complete result payload is rendered by the document's content below this state.
pub(crate) fn ending_summary(b: &mut Builder, value: &ChipEnding, words: &Words) {
    match value {
        ChipEnding::Done(_) => text(b, words, P::Done, &[]),
        ChipEnding::Failed(reason) => {
            text(b, words, P::Failed, &[]);
            document(b, reason, words);
        }
        ChipEnding::Cancelled { reason, by, at, .. } => {
            text(b, words, P::Cancelled, &[]);
            document(b, reason, words);
            match by {
                Requester::Task(task) => number(b, words, P::RequesterTask, *task),
                Requester::Person(person) => number(b, words, P::RequesterPerson, *person),
                Requester::Deployment { project } => number(b, words, P::RequesterDeployment, u64::from(*project)),
            }
            wall(b, words, P::CancelledAt, *at);
        }
    }
}

/// A supplied retry reason follows the retry count in the selected language.
pub(crate) fn retry_reason(b: &mut Builder, reason: Option<&DocumentText>, words: &Words) {
    if let Some(reason) = reason {
        words.text(b, JigPhrase::RunRetryReason, &[crate::words::name(reason.bytes())]);
        if reason.cut {
            words.text(b, JigPhrase::DocumentTextCut, &[]);
        }
    }
}

/// Failure classes have one whole table so a new class must acquire words.
pub fn failure(b: &mut Builder, class: FailureClass, words: &Words) {
    let phrase = match class {
        FailureClass::HostLost => JigPhrase::HostLost,
        FailureClass::Temporary => JigPhrase::Temporary,
        FailureClass::Permanent => JigPhrase::Permanent,
        FailureClass::Stall => JigPhrase::Stall,
    };
    words.text(b, phrase, &[]);
}
