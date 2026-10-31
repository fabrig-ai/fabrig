//! Plain peer answers shared by application conformance adapters
//! (`jig/docs/design/domain/testing.md`, 5).
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_notes as notes;
use skein_lib::{Duration, List, Token};
use std::collections::BTreeMap;

/// Answer the scripted account component's shared request vocabulary.
#[must_use]
pub fn account(request: accounts::ToComponent, valid: Duration) -> Option<core::Event> {
    let answer = match request {
        accounts::ToComponent::Want { account } => accounts::FromComponent::Granted {
            form: accounts::Form::Credential(accounts::Scheme::Bearer),
            account,
            generation: 1,
            valid,
        },
        accounts::ToComponent::Close => accounts::FromComponent::Closed,
        accounts::ToComponent::Rejected { .. } | accounts::ToComponent::Release { .. } => return None,
    };
    Some(core::Event::Account(accounts::Event::Component(answer)))
}

/// The externally visible position of a core-owned restart phase.
#[must_use]
pub fn restart_position(phase: core::start::Phase, connectors: u16) -> Option<u32> {
    use core::start::Phase;
    match phase {
        Phase::LoadCore => Some(0),
        Phase::RestoreConnector { connector } => Some(u32::from(connector)),
        Phase::ReadAfresh { connector } => Some(u32::from(connectors) + u32::from(connector)),
        Phase::LoadInbox
        | Phase::Sweep
        | Phase::Switch
        | Phase::Link
        | Phase::Wires
        | Phase::AdoptRuns
        | Phase::SettleOutbox { .. }
        | Phase::Open => None,
    }
}

/// Return the notes page supplied by the adapter's durable store.
#[must_use]
pub fn notes_loaded(owner: Token, records: List<notes::Record>, more: bool) -> core::Event {
    core::Event::Notes(notes::Event::Loaded { owner, rows: notes::Rows { records }, more })
}

/// Load a committed transcript window in attempt/turn order.
///
/// # Errors
/// Returns an unread `TranscriptLoaded` answer when the requested opening turn is missing.
pub fn transcript(
    records: &BTreeMap<core::Key, &core::Record>,
    task: u64,
    from: Option<(u64, u32)>,
) -> Result<Vec<core::TurnRecord>, Box<core::Event>> {
    let mut rows: Vec<_> = records
        .values()
        .filter_map(|record| {
            if let core::Record::Core(core::CoreRecord::Turn(row)) = record
                && row.task == task
                && from.is_some_and(|start| (row.attempt, row.turn) >= start)
            {
                Some(row.clone())
            } else {
                None
            }
        })
        .collect();
    rows.sort_by_key(|row| (row.attempt, row.turn));
    if let Some(start) = from
        && !rows.first().is_some_and(|row| (row.attempt, row.turn) == start && row.opens_window)
    {
        return Err(Box::new(core::Event::TranscriptLoaded { task, load: core::TranscriptLoad::Unread }));
    }
    Ok(rows)
}
