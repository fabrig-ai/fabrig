//! The checkout's capacity, whole (protocol/configuration/limits.md).
//! Git's earlier check defers stdout here, where both links' rooms are known.
use crate::{settings, worker_settings::Limits};
use jig_protocol_config::{self as config, Path};
use temper_protocol_worker_wire::v1 as payload;
use temper_service_core::derived::{checkout_answer as answer, checkout_payload as payload};
use temper_service_core::startup::problems::{Problems, What};

fn short(has: u64, needs: u64) -> Option<(u32, u32)> {
    if has < needs {
        Some((u32::try_from(has).expect("configured bound"), u32::try_from(needs).expect("codec bound")))
    } else {
        None
    }
}

fn room(jig: Option<&config::Limits>, payload: &payload::Limits) -> Option<(u32, u32)> {
    let jig = jig?;
    short(u64::from(jig.io.largest_room()), answer(jig, payload)?)
}

/// Gather every checkout-derived shortfall into one refusal before later
/// domain or link construction can return a less complete one.
pub fn capacity<E>(
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
        Some(worker) => room(worker.limits.as_ref(), &payload),
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
