//! Wire projections of asks facts (client/domain/testing.md, 4).
use super::super::{c, list, w};
use super::{
    wire_document_outcome, wire_note_scope, wire_outcome_moved_to_newest, wire_outcome_settings_changed,
    wire_person_address, wire_revision, wire_setting,
};
pub(in crate::codec) fn wire_address_change(value: &c::AddressChange) -> w::AddressChange {
    w::AddressChange { revision: wire_revision(&value.revision), address: wire_person_address(&value.address) }
}

pub(in crate::codec) fn wire_seen_through(value: &c::SeenThrough) -> w::SeenThrough {
    match value {
        c::SeenThrough::Results { through } => w::SeenThrough::Results(w::SeenResults { through: *(through) }),
        c::SeenThrough::Conversation { task, through } => {
            w::SeenThrough::Conversation(w::SeenConversation { task: *(task), through: *(through) })
        }
    }
}

pub(in crate::codec) fn wire_outcome(value: &c::Outcome) -> w::Outcome {
    match value {
        c::Outcome::Account(c::AccountAnswer::Reading { count }) => w::Outcome::Account(w::OutcomeAccount {
            answer: w::AccountAnswer::Reading(w::AccountReading { count: *count }),
        }),
        c::Outcome::SettingsChanged(value) => w::Outcome::SettingsChanged(wire_outcome_settings_changed(value)),
        c::Outcome::WorkerPauseAsked { worker } => {
            w::Outcome::WorkerPauseAsked(w::OutcomeWorkerPauseAsked { worker: *worker })
        }
        c::Outcome::Restored { proposal, to, left_out } => {
            let mut settings = skein_lib::List::with_capacity(left_out.len());
            for setting in left_out {
                settings.push(wire_setting(setting)).expect("same capacity");
            }
            w::Outcome::Restored(w::OutcomeRestored { proposal: *proposal, to: *to, left_out: settings })
        }
        c::Outcome::MovedToNewest(value) => w::Outcome::MovedToNewest(wire_outcome_moved_to_newest(value)),
        c::Outcome::Document(value) => w::Outcome::Document(w::OutcomeDocument { value: wire_document_outcome(value) }),
        c::Outcome::Made { what } => w::Outcome::Made(w::OutcomeMade { what: wire_made(what) }),
    }
}

pub(in crate::codec) fn wire_made(value: &c::Made) -> w::Made {
    match value {
        c::Made::Tasks { tasks } => w::Made::Tasks(w::MadeTasks { tasks: list((tasks).iter().copied()) }),
        c::Made::Service { party } => {
            w::Made::Service(w::MadeService { party: *(party), credential: skein_lib::Secret::new(Box::new([])) })
        }
        c::Made::Reissued { party } => {
            w::Made::Reissued(w::MadeService { party: *party, credential: skein_lib::Secret::new(Box::new([])) })
        }
        c::Made::PersonResult { task } => w::Made::PersonResult(w::MadePersonResult { task: *(task) }),
        c::Made::Note { note, scope } => w::Made::Note(w::MadeNote { note: *(note), scope: wire_note_scope(scope) }),
        c::Made::MovingOut { account, number, record } => {
            w::Made::MovingOut(w::MadeMovingOut { account: *(account), number: *(number), record: (record).clone() })
        }
    }
}
