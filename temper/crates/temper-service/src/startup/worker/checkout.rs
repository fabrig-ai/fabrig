//! The checkout's capacity, whole (protocol/configuration/limits.md).
//! Git's earlier check defers stdout here, where both links' rooms are known.
use temper_service_core::startup::problems::{Problems, What};
use crate::{worker_settings::Limits, settings};
use temper_service_core::limits;
use jig_protocol_config::{self as config, Path};
use temper_protocol_worker_wire::v1 as payload;

fn short(has: u64, needs: u64) -> Option<(u32, u32)> {
    if has < needs {
        Some((u32::try_from(has).expect("configured bound"), u32::try_from(needs).expect("codec bound")))
    } else {
        None
    }
}

/// The generated payload bounds need only the checkout's three capacity values.
fn payload(checkout: &limits::Checkout) -> payload::Limits {
    payload::Limits {
        saved_repositories: checkout.repositories,
        left_landed: checkout.repositories,
        files_files: checkout.conflicts,
        files_files_item: checkout.path_bytes,
        ..payload::CEILINGS
    }
}

/// Measure the same answer as jig's sizes, with its frame header. Other body
/// fields remain at their ceilings: they do not occur in an Answer.
fn answer(jig: &config::Limits, payload: &payload::Limits) -> Option<u64> {
    use jig_protocol_worker_wire::{CEILINGS, v9};
    let saved = payload::Saved::encoded_bound(payload)?;
    let ceiling = u64::from(CEILINGS.work_saved_some);
    let limits = v9::Limits {
        work_left_some: u32::try_from(payload::Left::encoded_bound(payload)?).ok()?,
        work_saved_some: u32::try_from(saved.min(ceiling)).ok()?,
        answer_refused: jig.core.tasks.run_inbox,
        ended_outcome: u32::try_from(jig.hub.outcome_bytes).ok()?,
        failed_detail: jig.hub.detail_bytes,
        invalid_start_why: jig.hub.detail_bytes,
        resource_resource_some: u32::try_from(payload::Preparation::encoded_bound(payload)?).ok()?,
        ..CEILINGS
    };
    // At the saved ceiling, the largest ending contains Work, not a refusal.
    // Any excess extends that same byte field; measure even a ceiling shortfall
    // so it cannot hide the rooms that must rise with it.
    v9::Answer::encoded_bound(&limits)?.checked_add(saved.saturating_sub(ceiling))?.checked_add(8)
}

fn room(jig: Option<&config::Limits>, payload: &payload::Limits) -> Option<(u32, u32)> {
    let jig = jig?;
    short(u64::from(jig.io.largest_room()), answer(jig, payload)?)
}

/// Gather every checkout-derived shortfall into one refusal before later
/// domain or link construction can return a less complete one.
pub(crate) fn capacity<E>(
    engine: Option<&config::Limits>,
    workers: &[config::Worker],
    own: &Limits,
    agent: Option<&settings::Agent>,
    problems: &mut Problems<What<E>>,
) {
    let checkout = &own.checkout;
    let stdout = match temper_worker_protocol_git::limits::largest_output(&crate::worker_settings::checkout(*checkout))
    {
        Some(needs) => short(u64::from(own.git_component.stdout_bytes), u64::from(needs)),
        None => None, // The payload ceiling check names the checkout field.
    };
    let paths = u64::from(checkout.conflicts).checked_mul(u64::from(checkout.path_bytes).saturating_add(1));
    let stdout_of: &'static [u8] = match stdout {
        Some((_, needs)) if paths == Some(u64::from(needs)) => b"checkout.conflicts * (checkout.path_bytes + 1)",
        Some(_) | None => b"checkout.name_bytes and git's push output",
    };
    // These are the conflict and path rules of temper-worker-domain::unfit;
    // the remaining worker rules still run at their own construction phase.
    let (agent_conflicts, agent_paths) = match agent {
        Some(agent) => (
            short(u64::from(agent.host_limits.conflicts), u64::from(checkout.conflicts)),
            short(u64::from(agent.host_limits.path_bytes), u64::from(checkout.path_bytes)),
        ),
        None => (None, None),
    };
    let payload = payload(checkout);
    let saved = match payload::Saved::encoded_bound(&payload) {
        Some(needs) => short(u64::from(jig_protocol_worker_wire::CEILINGS.work_saved_some), needs),
        None => None, // The payload ceiling check names the checkout field.
    };
    let engine = room(engine, &payload);
    let worker = match workers.first() {
        Some(worker) => room(Some(&worker.limits), &payload),
        None => None,
    };
    if stdout.is_some()
        || agent_conflicts.is_some()
        || agent_paths.is_some()
        || saved.is_some()
        || engine.is_some()
        || worker.is_some()
    {
        problems.push(
            Path::root(),
            What::Checkout {
                conflicts: checkout.conflicts,
                path_bytes: checkout.path_bytes,
                repositories: checkout.repositories,
                stdout,
                stdout_of,
                agent_conflicts,
                agent_paths,
                saved,
                engine,
                worker,
            },
        );
    }
}
