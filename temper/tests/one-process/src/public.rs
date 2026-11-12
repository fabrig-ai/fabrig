//! Visitors through the service and a caching proxy (temper/docs/design/protocol/sign-in.md, 2.5).
mod abandoned;
mod checks;
mod deployment;
mod http;
mod identifiers;
mod load;
mod observation;
mod owners;
mod previous;
mod programs;
mod proxy;
mod requests;
mod revocation;
mod service;
mod service_checks;
mod visitor;
mod wire;

use std::cell::Cell;
thread_local! { static TRACKED: Cell<bool> = const { Cell::new(true) }; }
thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }
// The forge's largest public record is PublicLandings; two reservations fit, three do not.
pub(super) const PUBLIC_ANSWER_BYTES: u32 = 67_405;
pub(super) const PUBLIC_BYTES: u32 = PUBLIC_ANSWER_BYTES * 2 + 1;
pub(crate) fn configure(value: &mut serde_json::Value) {
    if !ACTIVE.with(Cell::get) {
        return;
    }
    if value["world_settings"]["policies"][0]["policy"]["ceiling"]["grants"]
        .as_array()
        .is_some_and(|grants| !grants.is_empty())
    {
        for project in value["world_settings"]["policies"].as_array_mut().expect("policies") {
            for pointer in ["/policy/ceiling/grants", "/policy/roles/0/authority/grants"] {
                let grants = project.pointer_mut(pointer).expect("world grants");
                let prototype = grants[0].clone();
                *grants = temper_engine_domain_forge::CHANGE_KINDS
                    .into_iter()
                    .chain([15])
                    .map(|kind| {
                        let mut grant = prototype.clone();
                        grant["kind"] = kind.into();
                        grant
                    })
                    .collect();
            }
        }
        for (name, count) in [
            ("tasks.authority_grants", 8),
            ("authority.grants", 8),
            ("notes.scopes", 12),
            ("people.amendment_bytes", 2048),
            ("tasks.authority_bytes", 256),
        ] {
            value["engine"]["limits"]["overrides"][name] = count.into();
        }
    }
    value["application"]["limits"]["overrides"]["forge.judge_projects"] = 3.into();
    // Workflow discovery can share the fixture's four foreground logical-call slots.
    value["application"]["limits"]["overrides"]["forge.client.pending"] = 5.into();
    value["application"]["limits"]["overrides"]["forge.client.op_bytes"] = 4096.into();
    // Merge setup and workflow discovery fit one forge rate window before visitor reads.
    value["application"]["limits"]["overrides"]["forge.client.rate"] = 1000.into();
    value["engine"]["limits"]["overrides"]["core.call_answer_bytes"] = 4098.into();
    for worker in value["workers"].as_array_mut().expect("workers") {
        worker["limits"]["overrides"]["core.call_answer_bytes"] = 4098.into();
    }
    value["application"]["clients"]["public"] = "https://one.example".into();
    value["application"]["landing"] = serde_json::json!({"projects": [{"project": 1, "by": "turn"}]});
    for (name, count) in [
        ("clients.public.reads", 4),
        ("clients_http.public_calls", 4),
        ("clients.public.rate", 8),
        ("clients.public.rate_burst", 8),
        ("clients.public.bytes", PUBLIC_BYTES),
        ("clients.public.bytes_burst", PUBLIC_BYTES),
        ("clients.public.answer_bytes", PUBLIC_ANSWER_BYTES),
        ("clients.public.max_age", 2),
        ("core.public.goals", 4),
        ("core.public.ended", 4),
        ("core.public.lines", 8),
        ("core.public.depth", 4),
        ("people.projects", 4),
        ("authority.projects", 3),
        ("tasks.spec_bytes", 16_384),
        ("tasks.result_bytes", 4096),
        ("tasks.message_bytes", 4096),
        ("tasks.inbox_bytes", 4096),
    ] {
        value["engine"]["limits"]["overrides"][name] = count.into();
    }
    temper_playground::documents::turn_bounds::apply(value, service::agent_settings().limits.bodies.turn_body);
    previous::configure(value);

}

/// Replay a visitor's negative walk with failures naming its seed.
pub fn walk(seed: u64, name: &str) {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if name == "each_broken_relation_refuses_the_start" {
            deployment::relations();
        } else if name.starts_with("the_proxy_")
            || matches!(
                name,
                "a_stopped_upstream_serves_copies_until_their_freshness_ends"
                    | "two_deployments_behind_one_proxy_keep_their_own_copies"
                    | "a_refused_host_is_never_answered_from_the_cache"
            )
        {
            checks::run(seed, name);
        } else if name == "a_store_written_before_public_reading_reads_every_project_private" {
            previous::run(seed);
        } else if name == "every_inaccessible_identifier_is_one_unknown_byte_for_byte" {
            TRACKED.with(|tracked| tracked.set(true));
            service::run(seed, "every_owner_version_and_kind_answers_as_declared");
            TRACKED.with(|tracked| tracked.set(false));
            for proxied in [false, true] {
                identifiers::PROXIED.with(|flag| flag.set(proxied));
                service::run(seed, name);
            }
            TRACKED.with(|tracked| tracked.set(true));
        } else {
            service::run(seed, name);
        }
    }));
    ACTIVE.with(|active| active.set(false));
    if let Err(error) = result {
        let detail = error
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| error.downcast_ref::<&str>().copied())
            .unwrap_or("non-text panic");
        panic!("public walk {name}, seed {seed}: {detail}");
    }
}
