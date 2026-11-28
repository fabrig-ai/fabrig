//! The walking root with real children, an ordered paged fake store, scripted
//! person and worker, independent durable-delivery referee and restart cuts.
pub mod acts;
pub mod checks;
pub mod host_tool;
pub mod journal_limits;
pub mod outcome;
/// The protocol layer the world plays: link records, tool decode and render, outcomes, accounts.
pub mod protocol;
pub mod replay;
pub mod store;
pub mod walking;
pub mod walking_referee;
pub mod workload;

/// Real held-chat escalation driver.
pub mod escalation;
/// Independent escalation obligations.
pub mod escalation_referee;

/// Authenticated role-change driver over real held-chat records.
pub mod roles;
/// Independent role-cohort and recipient evidence.
pub mod roles_referee;

pub mod budget;
pub mod check;
/// Shared direct root route driver for focused and randomized scenarios.
pub mod direct;
pub mod ends;
pub mod fixtures;
/// Requester-backed forge change fixture and delayed required-read control.
pub mod forge_routes;
pub mod forge_stand;
pub mod kinds;
pub mod oracle;
pub mod tier_runs;
pub mod tier_task_show;

/// Take ready journal output, then admit one owner continuation if none was ready.
pub fn poll(
    root: &mut temper_engine_domain::Domain,
    env: &skein_lib::Env<temper_engine_domain::Limits>,
    out: &mut skein_lib::Queue<temper_engine_domain::Request>,
) {
    temper_engine_domain::release(root, env, out);
    if out.is_empty() {
        temper_engine_domain::release(root, env, out);
    }
    if out.is_empty() {
        temper_engine_domain::step(root, env, temper_engine_domain::Event::Resume);
        temper_engine_domain::release(root, env, out);
    }
}

/// Peer input waits for durable Wires (temper/docs/design/domain/root-restart.md, 1.1).
pub(crate) fn waits_for_start(root: &temper_engine_domain::Domain, event: &temper_engine_domain::Event) -> bool {
    !root.start_runtime()
        && matches!(
            event,
            temper_engine_domain::Event::Credential { .. }
                | temper_engine_domain::Event::Worker(temper_engine_domain::WorkerInput::Link { .. })
        )
}
