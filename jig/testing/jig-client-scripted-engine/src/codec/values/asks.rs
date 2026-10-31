//! Wire projections of asks facts (client/domain/testing.md, 4).
use super::super::{c, list, w};
use super::{
    wire_document_outcome, wire_note_scope, wire_outcome_moved_to_newest, wire_outcome_settings_changed,
    wire_person_address, wire_revision,
};
pub(in crate::codec) fn wire_address_change(
    value: &c::AddressChange,
    limits: &w::Limits,
) -> Result<w::AddressChange, w::Problem> {
    w::AddressChange::new(
        limits,
        w::AddressChangeParts {
            revision: wire_revision(&value.revision, limits)?,
            address: wire_person_address(&value.address, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_seen_through(
    value: &c::SeenThrough,
    limits: &w::Limits,
) -> Result<w::SeenThrough, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SeenThrough::Results { through } => {
            w::SeenThrough::Results(w::SeenResults::new(limits, w::SeenResultsParts { through: *(through) })?)
        }
        c::SeenThrough::Conversation { task, through } => w::SeenThrough::Conversation(w::SeenConversation::new(
            limits,
            w::SeenConversationParts { task: *(task), through: *(through) },
        )?),
    })
}

pub(in crate::codec) fn wire_outcome(value: &c::Outcome, limits: &w::Limits) -> Result<w::Outcome, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Outcome::SettingsChanged(value) => {
            w::Outcome::SettingsChanged(wire_outcome_settings_changed(value, limits)?)
        }
        c::Outcome::MovedToNewest(value) => w::Outcome::MovedToNewest(wire_outcome_moved_to_newest(value, limits)?),
        c::Outcome::Document(value) => w::Outcome::Document(w::OutcomeDocument::new(
            limits,
            w::OutcomeDocumentParts { value: wire_document_outcome(value, limits)? },
        )?),
        c::Outcome::Made { what } => {
            w::Outcome::Made(w::OutcomeMade::new(limits, w::OutcomeMadeParts { what: wire_made(what, limits)? })?)
        }
    })
}

pub(in crate::codec) fn wire_made(value: &c::Made, limits: &w::Limits) -> Result<w::Made, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Made::Tasks { tasks } => w::Made::Tasks(w::MadeTasks::new(
            limits,
            w::MadeTasksParts { tasks: list((tasks).iter().map(|item| Ok::<_, w::Problem>(*(item))))? },
        )?),
        c::Made::Service { party } => w::Made::Service(w::MadeService::new(
            limits,
            w::MadeServiceParts { party: *(party), credential: skein_lib::Secret::new(Box::new([])) },
        )?),
        c::Made::PersonResult { task } => {
            w::Made::PersonResult(w::MadePersonResult::new(limits, w::MadePersonResultParts { task: *(task) })?)
        }
        c::Made::Note { note, scope } => w::Made::Note(w::MadeNote::new(
            limits,
            w::MadeNoteParts { note: *(note), scope: wire_note_scope(scope, limits)? },
        )?),
        c::Made::MovingOut { account, number, record } => w::Made::MovingOut(w::MadeMovingOut::new(
            limits,
            w::MadeMovingOutParts { account: *(account), number: *(number), record: (record).clone() },
        )?),
    })
}
