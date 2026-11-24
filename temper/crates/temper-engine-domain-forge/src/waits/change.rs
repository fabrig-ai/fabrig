//! The waits of a change (temper/docs/design/domain/forge/waits.md, 4).
use crate::{Domain, Limits, room};
use jig_domain_core_vocabulary::waits::Reported;
use skein_lib::{Env, List};
use temper_client_protocol_wire::waits as code;
use temper_engine_domain_forge_change::State;

pub(super) fn waits(d: &Domain, env: &Env<Limits>, task: u64) -> List<Reported> {
    let mut waits = List::with_capacity(1);
    if let Some(row) = d.changes.get(&task)
        && let Some(step) = step(&row.change.state)
        && row.delegate.is_none()
        && let Some(wait) = room::waited(d, env, task, step)
    {
        waits.push(wait).expect("one room wait");
    }
    waits
}

fn step(state: &State) -> Option<u32> {
    let code = match state {
        State::Taking { .. } => code::CHANGE_TAKING,
        State::Producing { .. } => code::CHANGE_PRODUCING,
        State::Opening { .. } => code::CHANGE_OPENING,
        State::Recreating { .. } => code::CHANGE_RECREATING,
        State::Reopening { .. } => code::CHANGE_REOPENING,
        State::Checking { .. } => code::CHANGE_CHECKING,
        State::Gating { .. } => code::CHANGE_GATING,
        State::Ready { .. } => code::CHANGE_READY,
        State::Preparing { .. } => code::CHANGE_PREPARING,
        State::CancellingGates { .. } => code::CHANGE_CANCELLING_GATES,
        State::Queued { .. } => code::CHANGE_QUEUED,
        State::First { .. } => code::CHANGE_FIRST,
        State::Updating { .. } => code::CHANGE_UPDATING,
        State::WaitingProducer { .. } => code::CHANGE_WAITING_PRODUCER,
        State::Resolving { .. } => code::CHANGE_RESOLVING,
        State::Repairing { .. } => code::CHANGE_REPAIRING,
        State::Landing { .. } => code::CHANGE_LANDING,
        State::Landed { .. } => code::CHANGE_LANDED,
        State::Red { .. } => code::CHANGE_RED,
        State::Cleared { .. } => code::CHANGE_CLEARED,
        State::Uncleared { .. } => code::CHANGE_UNCLEARED,
        State::Judging { .. } => code::CHANGE_JUDGING,
        State::Held { .. } | State::NoChange { .. } | State::Ended => return None,
    };
    Some(code)
}
